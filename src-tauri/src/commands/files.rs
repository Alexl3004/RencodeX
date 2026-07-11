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
        .parent()
        .unwrap_or_else(|| std::path::Path::new(""))
        .join("ffprobe");

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
    let cfg = resolve_config(load_config());

    let status = tokio::process::Command::new(&cfg.ffmpeg_path)
        .args([
            "-y",
            "-i", &source_path,
            "-map", &format!("0:s:{}", track_index),
            &output_path,
        ])
        .creation_flags(0x08000000)
        .status()
        .await
        .map_err(|e| format!("Erreur lancement ffmpeg : {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "ffmpeg a échoué (code {})",
            status.code().unwrap_or(-1)
        ))
    }
}