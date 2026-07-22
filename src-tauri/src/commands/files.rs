//! Commandes liées aux fichiers et sous-titres

use crate::utils::resolve_config;
use super::settings::load_config;

#[tauri::command]
pub fn get_default_output_dir() -> String {
    dirs::download_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

const MAX_SCAN_DEPTH: usize = 32;

fn scan_dir_recursive(dir: &str, exts: &[String], results: &mut Vec<String>, depth: usize) {
    if depth > MAX_SCAN_DEPTH {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if meta.is_dir() {
            if let Some(p) = path.to_str() {
                scan_dir_recursive(p, exts, results, depth + 1);
            }
        } else if meta.is_file() {
            let ext = path.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .unwrap_or_default();
            if exts.contains(&ext) {
                if let Some(p) = path.to_str() {
                    results.push(p.to_string());
                }
            }
        }
    }
}

#[tauri::command]
pub fn scan_folder(folder: String, extensions: Vec<String>) -> Vec<String> {
    let exts: Vec<String> = extensions.iter().map(|e| e.to_lowercase()).collect();
    let mut results = Vec::new();
    scan_dir_recursive(&folder, &exts, &mut results, 0);
    results.sort();
    results
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct SubtitleTrack {
    pub index: u32,
    pub stream_index: u32,
    pub language: String,
    pub title: String,
    pub codec: String,
}

#[tauri::command]
pub async fn list_subtitle_tracks(path: String) -> Result<Vec<SubtitleTrack>, String> {
    let cfg = resolve_config(load_config());
    let ffprobe = std::path::Path::new(&cfg.ffmpeg_path)
        .with_file_name(if cfg!(windows) { "ffprobe.exe" } else { "ffprobe" });

    let output = tokio::process::Command::new(&ffprobe)
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_streams",
            "-select_streams", "s",
            &path,
        ])
        .creation_flags(0x08000000)
        .output()
        .await
        .map_err(|e| format!("ffprobe introuvable : {}", e))?;

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Erreur parsing ffprobe : {}", e))?;

    let streams = json["streams"].as_array().cloned().unwrap_or_default();
    let mut tracks = Vec::new();
    for (i, s) in streams.iter().enumerate() {
        let codec = s["codec_name"].as_str().unwrap_or("unknown").to_string();
        let lang = s["tags"]["language"].as_str().unwrap_or("und").to_string();
        let title = s["tags"]["title"].as_str().unwrap_or("").to_string();
        let stream_index = s["index"].as_u64().unwrap_or(0) as u32;
        tracks.push(SubtitleTrack {
            index: i as u32,
            stream_index,
            language: lang,
            title,
            codec,
        });
    }
    Ok(tracks)
}

#[tauri::command]
pub async fn extract_subtitles(
    source_path: String,
    track_index: u32,
    output_path: String,
) -> Result<(), String> {
    use std::sync::atomic::Ordering;

    // Vérifier le flag d'annulation AVANT de lancer FFmpeg.
    if crate::state::CANCEL_EXTRACTION.load(Ordering::Acquire) {
        return Err("cancelled".to_string());
    }

    let cfg = resolve_config(load_config());

    let mut child = tokio::process::Command::new(&cfg.ffmpeg_path)
        .args([
            "-y",
            "-i", &source_path,
            "-map", &format!("0:s:{}", track_index),
            &output_path,
        ])
        .creation_flags(0x08000000)
        .spawn()
        .map_err(|e| format!("Erreur lancement ffmpeg : {}", e))?;

    // Stocker le PID pour permettre un kill immédiat depuis cancel_subtitle_extraction.
    if let Some(pid) = child.id() {
        crate::state::EXTRACTION_PID.store(pid, Ordering::Release);
    }

    let status = child
        .wait()
        .await
        .map_err(|e| format!("Erreur attente ffmpeg : {}", e))?;

    // Remettre le PID à NO_PID une fois terminé.
    crate::state::EXTRACTION_PID.store(u32::MAX, Ordering::Relaxed);

    // Si le process a été tué par cancel, retourner "cancelled" proprement.
    if crate::state::CANCEL_EXTRACTION.load(Ordering::Acquire) {
        return Err("cancelled".to_string());
    }

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "ffmpeg a échoué (code {})",
            status.code().unwrap_or(-1)
        ))
    }
}

/// Annule immédiatement l'extraction de sous-titres en cours.
/// Positionne le flag `CANCEL_EXTRACTION` et tue le process FFmpeg actif.
#[tauri::command]
pub fn cancel_subtitle_extraction() {
    crate::state::request_cancel_extraction();
}

/// Remet l'état d'extraction à zéro avant de démarrer un nouveau batch.
/// Doit être appelé au début de chaque `startSubtitleExtraction()` côté front.
#[tauri::command]
pub fn reset_subtitle_extraction() {
    crate::state::reset_extraction_state();
}