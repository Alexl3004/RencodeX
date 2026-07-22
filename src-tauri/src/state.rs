//! État global de l'encodeur
//!
//! ## Architecture de concurrence
//!
//! L'état est découpé en deux couches selon la fréquence d'accès :
//!
//! ### Couche atomique (lock-free)
//! Signaux de contrôle et métriques de progression — lus/écrits des dizaines
//! de fois par seconde depuis le thread d'encodage et le handler de commandes Tauri.
//!
//! | Atomique         | Type         | Auteur           | Lecteurs            |
//! |------------------|--------------|------------------|---------------------|
//! | `CANCEL`         | `AtomicBool` | commande Tauri   | boucle stdout       |
//! | `SKIP`           | `AtomicBool` | commande Tauri   | boucle stdout       |
//! | `PAUSE`          | `AtomicBool` | commande Tauri   | boucle stdout       |
//! | `ENCODING`       | `AtomicBool` | thread encodage  | commandes, UI       |
//! | `CHILD_PID`      | `AtomicU32`  | thread encodage  | pause / kill        |
//! | `PERCENT/SPEED/REMAINING` | `AtomicU64` | boucle stdout | UI poll    |
//! | `FILE_INDEX/TOTAL` | `AtomicUsize` | thread encodage | UI poll          |
//!
//! ### Couche Mutex — `META`
//! Métadonnées stables : mises à jour au maximum une fois par fichier encodé,
//! jamais depuis la boucle chaude. Le lock n'est donc jamais contesté avec le
//! chemin critique.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;
use once_cell::sync::Lazy;


// ─── Helpers Windows/Unix – inchangés ────────────────────────────────────────

#[cfg(target_os = "windows")]
fn suspend_process(pid: u32) -> bool {
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_SUSPEND_RESUME};
    use windows::Win32::Foundation::CloseHandle;
    unsafe {
        let Ok(handle) = OpenProcess(PROCESS_SUSPEND_RESUME, false, pid) else {
            return false;
        };
        type FnNtSuspend = unsafe extern "system" fn(windows::Win32::Foundation::HANDLE) -> i32;
        let ntdll = windows::Win32::System::LibraryLoader::GetModuleHandleA(
            windows::core::s!("ntdll.dll")
        ).unwrap_or_default();
        let addr = windows::Win32::System::LibraryLoader::GetProcAddress(
            ntdll, windows::core::s!("NtSuspendProcess")
        );
        let result = if let Some(f) = addr {
            let f: FnNtSuspend = std::mem::transmute(f);
            f(handle) >= 0
        } else {
            false
        };
        let _ = CloseHandle(handle);
        result
    }
}

#[cfg(target_os = "windows")]
fn resume_process(pid: u32) -> bool {
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_SUSPEND_RESUME};
    use windows::Win32::Foundation::CloseHandle;
    unsafe {
        let Ok(handle) = OpenProcess(PROCESS_SUSPEND_RESUME, false, pid) else {
            return false;
        };
        type FnNtResume = unsafe extern "system" fn(windows::Win32::Foundation::HANDLE) -> i32;
        let ntdll = windows::Win32::System::LibraryLoader::GetModuleHandleA(
            windows::core::s!("ntdll.dll")
        ).unwrap_or_default();
        let addr = windows::Win32::System::LibraryLoader::GetProcAddress(
            ntdll, windows::core::s!("NtResumeProcess")
        );
        let result = if let Some(f) = addr {
            let f: FnNtResume = std::mem::transmute(f);
            f(handle) >= 0
        } else {
            false
        };
        let _ = CloseHandle(handle);
        result
    }
}

#[cfg(not(target_os = "windows"))]
fn suspend_process(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, libc::SIGSTOP) == 0 }
}

#[cfg(not(target_os = "windows"))]
fn resume_process(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, libc::SIGCONT) == 0 }
}

// ─── Valeur sentinelle pour « aucun process » ─────────────────────────────────
//
// On ne peut pas stocker un `Option<u32>` dans un AtomicU32. On réserve la
// valeur u32::MAX (4 294 967 295) comme sentinelle « pas de PID ». Un vrai
// PID ne peut pas valoir u32::MAX sur Windows (max DWORD valide ≈ 0xFFFC) ni
// sur Linux (max pid_t ≈ 4 194 304).
const NO_PID: u32 = u32::MAX;

// ─── Signaux de contrôle ─────────────────────────────────────────────────────

/// `true` dès qu'une annulation a été demandée.
/// Ordering : `Release` à l'écriture (commande Tauri), `Relaxed` en lecture
/// (boucle stdout). Un faux-négatif ponctuel ne fait traiter qu'une ligne
/// supplémentaire — acceptable ; on évite un `SeqCst` inutile sur le chemin
/// chaud.
pub static CANCEL: AtomicBool = AtomicBool::new(false);

/// `true` quand le fichier courant a été volontairement ignoré via `!skip`.
/// Distinct de `CANCEL` : la boucle `for` continue vers le fichier suivant,
/// le statut émis est `"skipped"` (ni erreur, ni annulation globale), et
/// aucune notification d'erreur Discord n'est envoyée.
/// Reset à `false` au début de chaque fichier par `media.rs`.
/// Ordering : `Release` à l'écriture, `Relaxed` en lecture (même raisonnement que `CANCEL`).
pub static SKIP: AtomicBool = AtomicBool::new(false);

/// `true` quand l'encodage est suspendu.
/// Même politique d'ordering que `CANCEL`.
pub static PAUSE: AtomicBool = AtomicBool::new(false);

/// `true` quand un encodage est actif.
/// `Release` à l'écriture pour que les lecteurs voient l'état initial cohérent.
pub static ENCODING: AtomicBool = AtomicBool::new(false);

/// PID du process FFmpeg en cours. Vaut `NO_PID` (u32::MAX) si absent.
/// `Release` à l'écriture (après spawn), `Acquire` à la lecture (avant kill/
/// suspend) pour garantir le happens-before entre spawn et signal.
pub static CHILD_PID: AtomicU32 = AtomicU32::new(NO_PID);

// ─── Métriques de progression ─────────────────────────────────────────────────

/// Progression en % (0.0 – 100.0) encodée en bits IEEE 754.
pub static PERCENT: AtomicU64 = AtomicU64::new(0);
/// Vitesse d'encodage (multiple de 1×) encodée en bits IEEE 754.
pub static SPEED: AtomicU64 = AtomicU64::new(0);
/// Temps restant estimé en secondes, encodé en bits IEEE 754.
pub static REMAINING: AtomicU64 = AtomicU64::new(0);
/// Index (base 0) du fichier en cours dans la file.
pub static FILE_INDEX: AtomicUsize = AtomicUsize::new(0);
/// Nombre total de fichiers dans la file.
pub static FILE_TOTAL: AtomicUsize = AtomicUsize::new(0);

// ─── État du bot Discord ──────────────────────────────────────────────────────

/// `true` dès qu'une annulation d'extraction de sous-titres a été demandée.
/// Positionné par la commande `cancel_subtitle_extraction`, remis à `false`
/// en début de chaque extraction batch par `reset_subtitle_extraction`.
/// Vérifié avant chaque `invoke("extract_subtitles")` dans la boucle JS, et
/// côté Rust avant de lancer FFmpeg pour un kill immédiat.
pub static CANCEL_EXTRACTION: AtomicBool = AtomicBool::new(false);

/// `true` quand le bot Discord est authentifié et prêt.
/// Écrit depuis le handler `ready` du bot, lu par la commande `get_bot_status`.
pub static BOT_CONNECTED: AtomicBool = AtomicBool::new(false);

/// Nom d'utilisateur Discord du bot (ex: "RenCodeX#1234").
/// Protégé par Mutex car écrit rarement (connexion/déconnexion).
pub static BOT_NAME: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));

// ─── Helpers f64 ↔ AtomicU64 ─────────────────────────────────────────────────
//
// Les métriques de progression sont des valeurs d'affichage : un seul thread
// écrit, plusieurs lisent. `Relaxed` suffit — une lecture légèrement décalée
// n'a aucune conséquence fonctionnelle.

/// Stocke un `f64` dans un `AtomicU64` via sa représentation bits IEEE 754.
#[inline]
pub fn store_f64(atom: &AtomicU64, v: f64) {
    atom.store(v.to_bits(), Ordering::Relaxed);
}

/// Lit un `f64` depuis un `AtomicU64`.
#[inline]
pub fn load_f64(atom: &AtomicU64) -> f64 {
    f64::from_bits(atom.load(Ordering::Relaxed))
}

// ─── Métadonnées stables (Mutex légitimement justifié) ───────────────────────
//
// Ces champs ne sont mis à jour qu'une fois par fichier encodé, jamais depuis
// la boucle chaude stdout → aucune contention avec le chemin critique.

pub struct EncoderMeta {
    /// Nom du fichier en cours (pour l'affichage UI).
    pub current_file: String,
    /// Chemin de sortie du fichier en cours (pour suppression si annulation).
    pub current_out: Option<String>,
    /// Noms des fichiers dans la file (ordre d'encodage).
    pub queue_names: Vec<String>,
    /// Horodatage du début de la session d'encodage (pour le chronomètre global).
    pub start_time: Option<Instant>,
}

impl Default for EncoderMeta {
    fn default() -> Self {
        Self {
            current_file: String::new(),
            current_out: None,
            queue_names: Vec::new(),
            start_time: None,
        }
    }
}

pub static META: Lazy<Mutex<EncoderMeta>> =
    Lazy::new(|| Mutex::new(EncoderMeta::default()));

/// Acquiert le lock sur les métadonnées stables.
/// Récupère le MutexGuard même en cas de poison (thread panique avec le lock).
pub fn lock_meta() -> std::sync::MutexGuard<'static, EncoderMeta> {
    META.lock().unwrap_or_else(|e| e.into_inner())
}

// ─── Snapshot (lecture cohérente pour les commandes Tauri) ───────────────────

/// Vue instantanée de tout l'état encodeur, utilisable en réponse à une
/// commande `get_encoder_state` sans tenir plusieurs locks simultanément.
#[derive(serde::Serialize, Clone)]
pub struct EncoderSnapshot {
    pub encoding: bool,
    pub cancel: bool,
    pub skip: bool,
    pub pause: bool,
    pub child_pid: Option<u32>,
    pub percent: f64,
    pub speed: f64,
    pub remaining: f64,
    pub file_index: usize,
    pub file_total: usize,
    pub current_file: String,
    pub current_out: Option<String>,
    pub queue_names: Vec<String>,
    /// Secondes écoulées depuis le début de la session (`start_time.elapsed()`).
    /// Vaut 0 si aucun encodage n'est en cours.
    pub elapsed_secs: u64,
}

/// Construit un `EncoderSnapshot` en lisant d'abord les atomiques (sans lock),
/// puis en verrouillant brièvement `META` une seule fois.
pub fn snapshot() -> EncoderSnapshot {
    // Lectures atomiques — aucun lock.
    let encoding   = ENCODING.load(Ordering::Acquire);
    let cancel     = CANCEL.load(Ordering::Relaxed);
    let skip       = SKIP.load(Ordering::Relaxed);
    let pause      = PAUSE.load(Ordering::Relaxed);
    let raw_pid    = CHILD_PID.load(Ordering::Acquire);
    let child_pid  = if raw_pid == NO_PID { None } else { Some(raw_pid) };
    let percent    = load_f64(&PERCENT);
    let speed      = load_f64(&SPEED);
    let remaining  = load_f64(&REMAINING);
    let file_index = FILE_INDEX.load(Ordering::Relaxed);
    let file_total = FILE_TOTAL.load(Ordering::Relaxed);

    // Un seul lock court sur les métadonnées stables.
    let meta = lock_meta();
    let elapsed_secs = meta.start_time
        .map(|t: std::time::Instant| t.elapsed().as_secs())
        .unwrap_or(0);
    EncoderSnapshot {
        encoding, cancel, skip, pause, child_pid,
        percent, speed, remaining, file_index, file_total,
        current_file: meta.current_file.clone(),
        current_out:  meta.current_out.clone(),
        queue_names:  meta.queue_names.clone(),
        elapsed_secs,
    }
}

// ─── Réinitialisation complète ────────────────────────────────────────────────

/// Réinitialise tout l'état encodeur à ses valeurs par défaut.
/// À appeler avant de démarrer une nouvelle session d'encodage.
#[allow(dead_code)]
pub fn reset_encoder_state() {
    // Ordre délibéré : on réinitialise ENCODING en dernier pour que les
    // lectures de snapshot voient un état cohérent (pas de ENCODING=true
    // avec des métriques à zéro partiellement écrites).
    CANCEL.store(false, Ordering::Relaxed);
    SKIP.store(false, Ordering::Relaxed);
    PAUSE.store(false, Ordering::Relaxed);
    CHILD_PID.store(NO_PID, Ordering::Relaxed);
    store_f64(&PERCENT, 0.0);
    store_f64(&SPEED, 0.0);
    store_f64(&REMAINING, 0.0);
    FILE_INDEX.store(0, Ordering::Relaxed);
    FILE_TOTAL.store(0, Ordering::Relaxed);
    {
        *lock_meta() = EncoderMeta::default();
    }
    // Release en dernier : garantit que tous les stores précédents sont
    // visibles aux threads qui lisent ENCODING avec Acquire.
    ENCODING.store(false, Ordering::Release);
}

// ─── Contrôle du process FFmpeg ──────────────────────────────────────────────

/// Suspend le process FFmpeg en cours.
///
/// Retourne `true` si la suspension a réussi. Ne fait rien si :
/// - aucun encodage n'est actif (`ENCODING` = false),
/// - déjà en pause (`PAUSE` = true),
/// - annulation en cours (`CANCEL` = true).
pub fn pause_ffmpeg_process() -> bool {
    if !ENCODING.load(Ordering::Acquire)
        || PAUSE.load(Ordering::Relaxed)
        || CANCEL.load(Ordering::Relaxed)
    {
        return false;
    }

    let pid = CHILD_PID.load(Ordering::Acquire);
    if pid == NO_PID { return false; }

    let ok = suspend_process(pid);
    if ok {
        // Release : les lectures ultérieures de PAUSE verront true.
        PAUSE.store(true, Ordering::Release);
    }
    ok
}

/// Reprend le process FFmpeg précédemment suspendu.
///
/// Retourne `true` si la reprise a réussi.
pub fn resume_ffmpeg_process() -> bool {
    if !ENCODING.load(Ordering::Acquire) || !PAUSE.load(Ordering::Relaxed) {
        return false;
    }

    let pid = CHILD_PID.load(Ordering::Acquire);
    if pid == NO_PID { return false; }

    let ok = resume_process(pid);
    if ok {
        PAUSE.store(false, Ordering::Release);
    }
    ok
}

/// Annule l'encodage en cours : marque `CANCEL`, tue le process FFmpeg,
/// et retourne le chemin du fichier de sortie partiel à supprimer.
///
/// Peut être appelé depuis `on-close-requested` ou la commande `cancel_encoding`.
/// Idempotent : un double appel n'a aucun effet négatif.
pub fn kill_ffmpeg_process() -> Option<String> {
    if !ENCODING.load(Ordering::Acquire) {
        return None;
    }

    CANCEL.store(true, Ordering::Release);

    let pid = CHILD_PID.swap(NO_PID, Ordering::AcqRel);

    if pid != NO_PID {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
            use windows::Win32::Foundation::CloseHandle;
            unsafe {
                if let Ok(handle) = OpenProcess(PROCESS_TERMINATE, false, pid) {
                    let _ = TerminateProcess(handle, 1);
                    let _ = CloseHandle(handle);
                } else {
                    eprintln!("[kill] OpenProcess({pid}) échoué — process déjà terminé ?");
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        unsafe {
            libc::kill(pid as i32, libc::SIGKILL);
        }
    }

    lock_meta().current_out.clone()
}

/// Demande l'annulation de l'extraction de sous-titres en cours.
///
/// Positionne `CANCEL_EXTRACTION` à `true`. Le process FFmpeg d'extraction
/// en cours sera tué via `kill_subtitle_ffmpeg_process` si un PID est stocké
/// dans `EXTRACTION_PID`. La boucle JS vérifie également ce flag entre chaque
/// appel `invoke("extract_subtitles")`.
pub fn request_cancel_extraction() {
    CANCEL_EXTRACTION.store(true, Ordering::Release);

    // Tuer le process FFmpeg d'extraction en cours s'il existe.
    let pid = EXTRACTION_PID.swap(NO_PID, Ordering::AcqRel);
    if pid == NO_PID {
        return;
    }

    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
        use windows::Win32::Foundation::CloseHandle;
        unsafe {
            if let Ok(handle) = OpenProcess(PROCESS_TERMINATE, false, pid) {
                let _ = TerminateProcess(handle, 1);
                let _ = CloseHandle(handle);
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    unsafe {
        libc::kill(pid as i32, libc::SIGKILL);
    }
}

/// Remet `CANCEL_EXTRACTION` et `EXTRACTION_PID` à leur état initial.
/// À appeler au début de chaque nouvelle session d'extraction batch.
pub fn reset_extraction_state() {
    EXTRACTION_PID.store(NO_PID, Ordering::Relaxed);
    CANCEL_EXTRACTION.store(false, Ordering::Relaxed);
}

/// PID du process FFmpeg d'extraction en cours. Vaut `NO_PID` si absent.
/// Écrit par `extract_subtitles` juste après le spawn, relu par
/// `request_cancel_extraction` pour kill immédiat.
pub static EXTRACTION_PID: AtomicU32 = AtomicU32::new(NO_PID);

/// Interrompt uniquement le process FFmpeg du fichier courant
/// Retourne `true` si un process a bien été tué.
pub fn skip_ffmpeg_process() -> bool {
    if !ENCODING.load(Ordering::Acquire) {
        return false;
    }
    // On swap le PID pour éviter un double-kill concurrent.
    let pid = CHILD_PID.swap(NO_PID, Ordering::AcqRel);
    if pid == NO_PID {
        return false;
    }

    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
        use windows::Win32::Foundation::CloseHandle;
        unsafe {
            if let Ok(handle) = OpenProcess(PROCESS_TERMINATE, false, pid) {
                let _ = TerminateProcess(handle, 1);
                let _ = CloseHandle(handle);
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    unsafe {
        libc::kill(pid as i32, libc::SIGKILL);
    }

    // Marque le fichier courant comme "ignoré volontairement".
    // Pas de CANCEL.store(true) → la boucle for continue vers le fichier suivant.
    // `media.rs` remet SKIP à false en tête de chaque nouveau fichier.
    //
    // Si l'encodage était en pause, on remet PAUSE à false : le process FFmpeg
    // vient d'être tué, il n'existe plus de process à reprendre. Sans ce reset,
    // le fichier suivant démarrerait avec PAUSE=true (état fantôme), provoquant
    // un affichage UI incohérent et des commandes pause/resume erratiques.
    PAUSE.store(false, Ordering::Release);
    SKIP.store(true, Ordering::Release);

    true
}