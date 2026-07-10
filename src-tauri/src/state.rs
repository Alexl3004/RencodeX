//! État global de l'encodeur

use std::sync::{Arc, Mutex};
use std::time::Instant;
use once_cell::sync::Lazy;
use std::os::windows::process::CommandExt;

// ─── Helpers Windows pour suspendre / reprendre un process ───────────────────

/// Ouvre le process par PID avec PROCESS_SUSPEND_RESUME (0x0800) et le suspend.
/// Utilise NtSuspendProcess depuis ntdll — disponible sur toutes les versions
/// de Windows supportées par Tauri.
#[cfg(target_os = "windows")]
fn suspend_process(pid: u32) -> bool {
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_SUSPEND_RESUME};
    use windows::Win32::Foundation::CloseHandle;
    unsafe {
        let Ok(handle) = OpenProcess(PROCESS_SUSPEND_RESUME, false, pid) else {
            return false;
        };
        // NtSuspendProcess n'est pas exposé dans le crate windows, on le charge dynamiquement.
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
    // SIGSTOP sur Unix / macOS
    unsafe { libc::kill(pid as i32, libc::SIGSTOP) == 0 }
}

#[cfg(not(target_os = "windows"))]
fn resume_process(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, libc::SIGCONT) == 0 }
}

pub struct EncoderState {
    pub cancel: bool,
    pub pause: bool,
    pub child_pid: Option<u32>,
    pub current_out: Option<String>,
    pub encoding: bool,
    pub current_file: String,
    pub file_index: usize,
    pub file_total: usize,
    pub percent: f64,
    pub speed: f64,
    pub remaining: f64,
    pub queue_names: Vec<String>,
    pub start_time: Option<Instant>,
}

impl Default for EncoderState {
    fn default() -> Self {
        Self {
            cancel: false,
            pause: false,
            child_pid: None,
            current_out: None,
            encoding: false,
            current_file: String::new(),
            file_index: 0,
            file_total: 0,
            percent: 0.0,
            speed: 0.0,
            remaining: 0.0,
            queue_names: Vec::new(),
            start_time: None,
        }
    }
}

pub static ENCODER: Lazy<Arc<Mutex<EncoderState>>> = Lazy::new(|| {
    Arc::new(Mutex::new(EncoderState::default()))
});

pub fn lock_encoder() -> std::sync::MutexGuard<'static, EncoderState> {
    ENCODER.lock().unwrap_or_else(|e| e.into_inner())
}

/// Suspend le process ffmpeg en cours (si un encodage est actif et pas déjà en pause).
/// Retourne `true` si la suspension a réussi.
pub fn pause_ffmpeg_process() -> bool {
    let mut s = lock_encoder();
    if !s.encoding || s.pause || s.cancel {
        return false;
    }
    let Some(pid) = s.child_pid else { return false };
    let ok = suspend_process(pid);
    if ok {
        s.pause = true;
    }
    ok
}

/// Reprend le process ffmpeg précédemment suspendu.
/// Retourne `true` si la reprise a réussi.
pub fn resume_ffmpeg_process() -> bool {
    let mut s = lock_encoder();
    if !s.encoding || !s.pause {
        return false;
    }
    let Some(pid) = s.child_pid else { return false };
    let ok = resume_process(pid);
    if ok {
        s.pause = false;
    }
    ok
}

/// Tue proprement le process ffmpeg en cours (si existant) et marque l'encodeur
/// comme annulé. Appelé depuis le handler on-close-requested ET depuis cancel_encoding.
/// Retourne le chemin du fichier de sortie partiel à supprimer, si applicable.
pub fn kill_ffmpeg_process() -> Option<String> {
    let (pid_opt, out_opt) = {
        let mut s = lock_encoder();
        if !s.encoding {
            return None;
        }
        s.cancel = true;
        (s.child_pid.take(), s.current_out.clone())
    };

    if let Some(pid) = pid_opt {
        let _ = std::process::Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F", "/T"])
            .creation_flags(0x08000000)
            .spawn();
    }

    out_opt
}