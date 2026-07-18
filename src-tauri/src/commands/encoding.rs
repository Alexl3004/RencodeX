//! Commandes liées à l'encodage et à FFmpeg

use std::time::Duration;
use tauri::AppHandle;
use tauri::Emitter;
use tauri_plugin_notification::NotificationExt;

use crate::models::{CleanedName, EncodeJob, EncodeSummary, FileAnalysis};
use crate::state::PAUSE;
use std::sync::atomic::Ordering;
use crate::utils::{delete_partial_output, filename_of, resolve_config};
use crate::naming::clean_filename as clean_filename_impl;
use crate::services::media::{analyze_file as analyze_file_impl, start_encoding as start_encoding_impl};
use super::settings::load_config;

#[derive(serde::Serialize, Debug)]
pub struct FfmpegCheckResult {
    pub path: String,
    pub exists: bool,
    pub executable: bool,
    pub version: Option<String>,
}

#[tauri::command]
pub async fn check_ffmpeg(path: String) -> FfmpegCheckResult {
    use std::path::Path;

    let resolved_path = if path.is_empty() {
        resolve_config(load_config()).ffmpeg_path
    } else {
        path
    };

    let p = Path::new(&resolved_path);

    if !p.exists() || !p.is_file() {
        return FfmpegCheckResult {
            path: resolved_path,
            exists: false,
            executable: false,
            version: None,
        };
    }

    let result = tokio::process::Command::new(&resolved_path)
        .arg("-version")
        .creation_flags(0x08000000)
        .output()
        .await;

    match result {
        Err(_) => FfmpegCheckResult {
            path: resolved_path,
            exists: true,
            executable: false,
            version: None,
        },
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let version = stdout
                .lines()
                .next()
                .and_then(|l| l.split("version").nth(1))
                .map(|v| v.split_whitespace().next().unwrap_or("").to_string())
                .filter(|v| !v.is_empty());

            let executable = out.status.success() || !out.stdout.is_empty();

            FfmpegCheckResult {
                path: resolved_path,
                exists: true,
                executable,
                version,
            }
        }
    }
}

#[tauri::command]
pub async fn analyze_file(path: String) -> Result<FileAnalysis, String> {
    analyze_file_impl(path).await
}

#[tauri::command]
pub fn clean_filename(
    raw: String,
    audio_langs: Vec<String>,
    sub_langs: Vec<String>,
) -> CleanedName {
    clean_filename_impl(raw, audio_langs, sub_langs)
}

#[tauri::command]
pub async fn start_encoding(
    app: AppHandle,
    jobs: Vec<EncodeJob>,
) -> Result<EncodeSummary, String> {
    start_encoding_impl(app, jobs).await
}

#[tauri::command]
pub async fn cancel_encoding(app: AppHandle) {
    let out_opt = crate::state::kill_ffmpeg_process();

    if let Some(out_path) = out_opt {
        let app2 = app.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(500)).await;
            let deleted = delete_partial_output(&out_path);
            let out_name = filename_of(&out_path);
            if deleted {
                let _ = app2.emit("partial-file-deleted", serde_json::json!({
                    "path": out_path,
                    "name": out_name,
                }));
            }
        });
    }

    let _ = app.notification()
        .builder()
        .title("⏹ Encodage annulé")
        .body("La file d'attente a été interrompue · fichiers partiels supprimés")
        .show();
}

#[tauri::command]
pub fn pause_encoding() -> bool {
    crate::state::pause_ffmpeg_process()
}

#[tauri::command]
pub fn resume_encoding() -> bool {
    crate::state::resume_ffmpeg_process()
}

#[tauri::command]
pub fn skip_encoding() {
    crate::state::skip_ffmpeg_process();
}

#[tauri::command]
pub fn get_paused() -> bool {
    PAUSE.load(Ordering::Relaxed)
}