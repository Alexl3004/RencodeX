//! Commandes liées à la configuration et aux préférences

use crate::models::{AppConfig, EncodingPrefs, Stats};
use crate::utils::{config_path, resolve_config};
use tauri::Emitter;

fn encoding_prefs_path() -> std::path::PathBuf {
    config_path()
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("encoding_prefs.json")
}

#[tauri::command]
pub fn load_config() -> AppConfig {
    if let Ok(data) = std::fs::read_to_string(config_path()) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        AppConfig::default()
    }
}

#[tauri::command]
pub fn save_config(config: AppConfig, app: tauri::AppHandle) -> Result<(), String> {
    let path = config_path();
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())?;
    let _ = app.emit("config-saved", ());
    Ok(())
}

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
    serde_json::to_value(crate::services::discord_fields::full_catalog()).unwrap_or_default()
}