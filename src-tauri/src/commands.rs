//! Commandes Tauri (wrappers minces)

use tauri::AppHandle;
use tauri::Emitter;
use tauri_plugin_notification::NotificationExt;
use std::os::windows::process::CommandExt;
use std::time::Duration;

use crate::models::{
    AppConfig, CleanedName, EncodeJob, EncodeSummary, EncodingPrefs, EmailConfig, FileAnalysis, Stats,
};
use crate::state::lock_encoder;
use crate::utils::{
    config_path, delete_partial_output, filename_of, resolve_config,
};
use crate::filename::clean_filename as clean_filename_impl;
use crate::media::{analyze_file as analyze_file_impl, start_encoding as start_encoding_impl};
use crate::notify::{
    discord_notify,
    discord_notify_start,
    discord_notify_file_done,
    discord_notify_error,
    discord_notify_stats,
    discord_notify_progress,
    send_email_report as send_email_report_impl
};

// ── Chemin du fichier encoding_prefs.json ────────────────────────────────────

fn encoding_prefs_path() -> std::path::PathBuf {
    config_path()
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("encoding_prefs.json")
}

// ── Configuration générale ────────────────────────────────────────────────────

#[tauri::command]
pub fn load_config() -> AppConfig {
    if let Ok(data) = std::fs::read_to_string(config_path()) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        AppConfig::default()
    }
}

#[tauri::command]
pub fn save_config(config: AppConfig) -> Result<(), String> {
    let path = config_path();
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

// ── Préférences d'encodage ────────────────────────────────────────────────────

#[tauri::command]
pub fn load_encoding_prefs() -> EncodingPrefs {
    if let Ok(data) = std::fs::read_to_string(encoding_prefs_path()) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        EncodingPrefs::default()
    }
}

#[tauri::command]
pub fn save_encoding_prefs(prefs: EncodingPrefs) -> Result<(), String> {
    let path = encoding_prefs_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(&prefs).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

// ── Stats ─────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn load_stats() -> Stats {
    match crate::db::load_stats_from_db().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[stats] Erreur lecture SQLite : {}", e);
            Stats::default()
        }
    }
}

#[tauri::command]
pub async fn save_stats(stats: Stats) -> Result<(), String> {
    crate::db::save_stats_to_db(&stats).await.map_err(|e| e.to_string())
}

// ── Config effective (résumé pour le frontend) ────────────────────────────────

#[tauri::command]
pub fn get_effective_config() -> serde_json::Value {
    let cfg = resolve_config(load_config());
    serde_json::json!({
        "ffmpeg_path": cfg.ffmpeg_path,
        "dark_mode": cfg.dark_mode,
        "send_email": cfg.send_email,
        "email_to": cfg.email_to,
        "discord_token_set": !cfg.discord_bot_token.is_empty(),
        "discord_log_channel_id": cfg.discord_log_channel_id,
        "discord_cmd_channel_id": cfg.discord_cmd_channel_id,
        "discord_enabled": cfg.discord_enabled,
        "discord_notify_start": cfg.discord_notify_start,
        "discord_notify_file_done": cfg.discord_notify_file_done,
        "discord_notify_summary": cfg.discord_notify_summary,
        "discord_notify_error": cfg.discord_notify_error,
        "discord_notify_progress": cfg.discord_notify_progress,
        "discord_progress_interval": cfg.discord_progress_interval,
    })
}

#[tauri::command]
pub fn get_discord_field_catalog() -> serde_json::Value {
    serde_json::to_value(crate::discord_fields::full_catalog()).unwrap_or_default()
}

// ── Analyse média ─────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn analyze_file(path: String) -> Result<FileAnalysis, String> {
    analyze_file_impl(path).await
}

// ── Nettoyage de nom ──────────────────────────────────────────────────────────

#[tauri::command]
pub fn clean_filename(
    raw: String,
    audio_langs: Vec<String>,
    sub_langs: Vec<String>,
) -> CleanedName {
    clean_filename_impl(raw, audio_langs, sub_langs)
}

// ── Encodage ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn start_encoding(
    app: AppHandle,
    jobs: Vec<EncodeJob>,
) -> Result<EncodeSummary, String> {
    start_encoding_impl(app, jobs).await
}

#[tauri::command]
pub async fn cancel_encoding(app: AppHandle) {
    let (pid_opt, out_opt) = {
        let mut s = lock_encoder();
        s.cancel = true;
        (s.child_pid, s.current_out.clone())
    };

    if let Some(pid) = pid_opt {
        let _ = std::process::Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F", "/T"])
            .creation_flags(0x08000000)
            .spawn();
    }

    // Nettoyage en arrière-plan — ne bloque plus l'UI
    if let Some(out_path) = out_opt {
        let app2 = app.clone();
        tokio::spawn(async move {
            { lock_encoder().current_out = None; }
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

// ── Utilitaires ───────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_default_output_dir() -> String {
    dirs::download_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

// ── Scan de dossier ───────────────────────────────────────────────────────────

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

// ── Helpers internes ──────────────────────────────────────────────────────────

/// Résout token + channel depuis les paramètres ou les variables d'env.
fn resolve_discord_creds(bot_token: String, log_channel_id: String) -> Result<(String, String), String> {
    let token = if !bot_token.is_empty() { bot_token }
        else { std::env::var("RENCODEX_DISCORD_TOKEN").unwrap_or_default() };
    let chan = if !log_channel_id.is_empty() { log_channel_id }
        else { std::env::var("RENCODEX_DISCORD_LOG_CHANNEL").unwrap_or_default() };
    if token.is_empty() || chan.is_empty() {
        return Err("Token ou log_channel_id manquant".to_string());
    }
    Ok((token, chan))
}

fn fields_for(notif_type: &str) -> Vec<String> {
    let cfg = load_config();
    cfg.discord_fields
        .get(notif_type)
        .cloned()
        .unwrap_or_else(|| crate::discord_fields::default_fields(notif_type))
}

fn note_for(notif_type: &str) -> String {
    let cfg = load_config();
    cfg.discord_custom_notes
        .get(notif_type)
        .cloned()
        .unwrap_or_default()
}

// ── Commandes de notification Discord ────────────────────────────────────────

/// Notification de résumé de session (fin de file complète).
#[tauri::command]
pub async fn send_discord_notification(
    bot_token: String,
    log_channel_id: String,
    summary: EncodeSummary,
) -> Result<(), String> {
    let (token, chan) = resolve_discord_creds(bot_token, log_channel_id)?;
    let fields = fields_for("summary");
    let note   = note_for("summary");
    discord_notify(&token, &chan, &summary, &fields, &note).await;
    Ok(())
}

/// Notification de démarrage d'encodage.
#[tauri::command]
pub async fn send_discord_start_notification(
    bot_token: String,
    log_channel_id: String,
    total_files: usize,
    total_size_mb: f64,
    crf: u32,
    preset: String,
) -> Result<(), String> {
    let (token, chan) = resolve_discord_creds(bot_token, log_channel_id)?;
    let fields = fields_for("start");
    let note   = note_for("start");
    discord_notify_start(&token, &chan, total_files, total_size_mb, crf, &preset, &fields, &note).await;
    Ok(())
}

/// Notification de fin de fichier individuel.
#[tauri::command]
pub async fn send_discord_file_done_notification(
    bot_token: String,
    log_channel_id: String,
    file_name: String,
    original_mb: f64,
    encoded_mb: f64,
    duration_secs: f64,
    crf: u32,
    preset: String,
) -> Result<(), String> {
    let (token, chan) = resolve_discord_creds(bot_token, log_channel_id)?;
    let fields = fields_for("file_done");
    let note   = note_for("file_done");
    discord_notify_file_done(
        &token, &chan, &file_name,
        original_mb, encoded_mb, duration_secs,
        crf, &preset, &fields, &note,
    ).await;
    Ok(())
}

/// Notification d'erreur d'encodage.
#[tauri::command]
pub async fn send_discord_error_notification(
    bot_token: String,
    log_channel_id: String,
    file_name: String,
    error_msg: String,
) -> Result<(), String> {
    let (token, chan) = resolve_discord_creds(bot_token, log_channel_id)?;
    let fields = fields_for("error");
    let note   = note_for("error");
    discord_notify_error(&token, &chan, &file_name, &error_msg, &fields, &note).await;
    Ok(())
}

/// Notification de progression (mise à jour périodique).
#[tauri::command]
pub async fn send_discord_progress_notification(
    bot_token: String,
    log_channel_id: String,
    file_name: String,
    file_index: usize,
    file_total: usize,
    percent: f64,
    speed: f64,
    remaining_secs: f64,
    elapsed_secs: f64,
) -> Result<(), String> {
    let (token, chan) = resolve_discord_creds(bot_token, log_channel_id)?;
    let fields = fields_for("progress");
    let note   = note_for("progress");
    discord_notify_progress(
        &token, &chan,
        &file_name, file_index, file_total,
        percent, speed, remaining_secs, elapsed_secs,
        &fields, &note,
    ).await;
    Ok(())
}

/// Notification de statistiques globales cumulées.
#[tauri::command]
pub async fn send_discord_stats_notification(
    bot_token: String,
    log_channel_id: String,
    total_files: usize,
    total_original_mb: f64,
    total_encoded_mb: f64,
    total_duration_secs: f64,
) -> Result<(), String> {
    let (token, chan) = resolve_discord_creds(bot_token, log_channel_id)?;
    discord_notify_stats(&token, &chan, total_files, total_original_mb, total_encoded_mb, total_duration_secs).await;
    Ok(())
}

#[tauri::command]
pub async fn send_email_report(
    summary: EncodeSummary,
    email_cfg: EmailConfig,
) -> Result<(), String> {
    send_email_report_impl(summary, email_cfg).await
}

// ── Extraction de sous-titres ─────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct SubtitleTrack {
    pub index: u32,       // index de flux ffmpeg (ex: 0:s:0)
    pub stream_index: u32, // index global dans le fichier
    pub language: String,
    pub title: String,
    pub codec: String,    // "subrip", "ass", "hdmv_pgs_subtitle", etc.
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