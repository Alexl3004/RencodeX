//! Commandes Tauri (wrappers minces)

use tauri::AppHandle;
use tauri::Emitter;
use tauri_plugin_notification::NotificationExt;
use std::os::windows::process::CommandExt;
use std::time::Duration;

use crate::models::{
    AppConfig, CleanedName, EncodeJob, EncodeSummary, EmailConfig, EncodingPrefs, FileAnalysis, Stats,
};
use crate::state::lock_encoder;
use crate::utils::{
    config_path, delete_partial_output, encoding_prefs_path, filename_of, resolve_config, stats_path,
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

// Configuration
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

// Préférences d'encodage (CRF, preset, ordre des tags, team…)
// Stockées dans un fichier séparé du reste de la config pour ne pas être
// écrasées par les sauvegardes faites depuis la page Settings.
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
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&prefs).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn load_stats() -> Stats {
    match std::fs::read_to_string(stats_path()) {
        Ok(data) => match serde_json::from_str(&data) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[stats] Erreur parsing stats.json : {}", e);
                Stats::default()
            }
        },
        Err(e) => {
            eprintln!("[stats] Erreur lecture stats.json : {}", e);
            Stats::default()
        }
    }
}

#[tauri::command]
pub fn save_stats(stats: Stats) -> Result<(), String> {
    let path = stats_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(&stats).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

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

// Analyse média
#[tauri::command]
pub async fn analyze_file(path: String) -> Result<FileAnalysis, String> {
    analyze_file_impl(path).await
}

// Nettoyage de nom
#[tauri::command]
pub fn clean_filename(
    raw: String,
    audio_langs: Vec<String>,
    sub_langs: Vec<String>,
) -> CleanedName {
    clean_filename_impl(raw, audio_langs, sub_langs)
}

// Encodage
#[tauri::command]
pub async fn start_encoding(
    app: AppHandle,
    jobs: Vec<EncodeJob>,
) -> Result<EncodeSummary, String> {
    start_encoding_impl(app, jobs).await
}

#[tauri::command]
pub fn cancel_encoding(app: AppHandle) {
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

    if let Some(out_path) = &out_opt {
        { lock_encoder().current_out = None; }
        std::thread::sleep(Duration::from_millis(500));
        let deleted = delete_partial_output(out_path);
        let out_name = filename_of(out_path);
        if deleted {
            let _ = app.emit("partial-file-deleted", serde_json::json!({
                "path": out_path,
                "name": out_name,
            }));
        }
    }

    let _ = app.notification()
        .builder()
        .title("⏹ Encodage annulé")
        .body("La file d'attente a été interrompue · fichiers partiels supprimés")
        .show();
}

// Utilitaires
#[tauri::command]
pub fn get_default_output_dir() -> String {
    dirs::download_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

// Scan de dossier
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

// Notifications Discord améliorées
#[tauri::command]
pub async fn send_discord_notification(
    bot_token: String,
    log_channel_id: String,
    summary: EncodeSummary,
) -> Result<(), String> {
    let token = if !bot_token.is_empty() { bot_token }
        else { std::env::var("RENCODEX_DISCORD_TOKEN").unwrap_or_default() };
    let chan = if !log_channel_id.is_empty() { log_channel_id }
        else { std::env::var("RENCODEX_DISCORD_LOG_CHANNEL").unwrap_or_default() };

    if token.is_empty() || chan.is_empty() {
        return Err("Token ou log_channel_id manquant".to_string());
    }
    discord_notify(&token, &chan, &summary).await;
    Ok(())
}

#[tauri::command]
pub async fn send_discord_start_notification(
    bot_token: String,
    log_channel_id: String,
    total_files: usize,
    total_size_mb: f64,
    crf: u32,
    preset: String,
) -> Result<(), String> {
    let token = if !bot_token.is_empty() { bot_token }
        else { std::env::var("RENCODEX_DISCORD_TOKEN").unwrap_or_default() };
    let chan = if !log_channel_id.is_empty() { log_channel_id }
        else { std::env::var("RENCODEX_DISCORD_LOG_CHANNEL").unwrap_or_default() };

    if token.is_empty() || chan.is_empty() {
        return Err("Token ou log_channel_id manquant".to_string());
    }
    discord_notify_start(&token, &chan, total_files, total_size_mb, crf, &preset).await;
    Ok(())
}

#[tauri::command]
pub async fn send_discord_file_done_notification(
    bot_token: String,
    log_channel_id: String,
    file_name: String,
    original_mb: f64,
    encoded_mb: f64,
    duration_secs: f64,
) -> Result<(), String> {
    let token = if !bot_token.is_empty() { bot_token }
        else { std::env::var("RENCODEX_DISCORD_TOKEN").unwrap_or_default() };
    let chan = if !log_channel_id.is_empty() { log_channel_id }
        else { std::env::var("RENCODEX_DISCORD_LOG_CHANNEL").unwrap_or_default() };

    if token.is_empty() || chan.is_empty() {
        return Err("Token ou log_channel_id manquant".to_string());
    }
    discord_notify_file_done(&token, &chan, &file_name, original_mb, encoded_mb, duration_secs).await;
    Ok(())
}

#[tauri::command]
pub async fn send_discord_error_notification(
    bot_token: String,
    log_channel_id: String,
    file_name: String,
    error_msg: String,
) -> Result<(), String> {
    let token = if !bot_token.is_empty() { bot_token }
        else { std::env::var("RENCODEX_DISCORD_TOKEN").unwrap_or_default() };
    let chan = if !log_channel_id.is_empty() { log_channel_id }
        else { std::env::var("RENCODEX_DISCORD_LOG_CHANNEL").unwrap_or_default() };

    if token.is_empty() || chan.is_empty() {
        return Err("Token ou log_channel_id manquant".to_string());
    }
    discord_notify_error(&token, &chan, &file_name, &error_msg).await;
    Ok(())
}

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
    let token = if !bot_token.is_empty() { bot_token }
        else { std::env::var("RENCODEX_DISCORD_TOKEN").unwrap_or_default() };
    let chan = if !log_channel_id.is_empty() { log_channel_id }
        else { std::env::var("RENCODEX_DISCORD_LOG_CHANNEL").unwrap_or_default() };

    if token.is_empty() || chan.is_empty() {
        return Err("Token ou log_channel_id manquant".to_string());
    }
    discord_notify_progress(&token, &chan, &file_name, file_index, file_total, percent, speed, remaining_secs, elapsed_secs).await;
    Ok(())
}

#[tauri::command]
pub async fn send_discord_stats_notification(
    bot_token: String,
    log_channel_id: String,
    total_files: usize,
    total_original_mb: f64,
    total_encoded_mb: f64,
    total_duration_secs: f64,
) -> Result<(), String> {
    let token = if !bot_token.is_empty() { bot_token }
        else { std::env::var("RENCODEX_DISCORD_TOKEN").unwrap_or_default() };
    let chan = if !log_channel_id.is_empty() { log_channel_id }
        else { std::env::var("RENCODEX_DISCORD_LOG_CHANNEL").unwrap_or_default() };

    if token.is_empty() || chan.is_empty() {
        return Err("Token ou log_channel_id manquant".to_string());
    }
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