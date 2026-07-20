//! Commandes liées aux notifications Discord

use crate::models::{AppConfig, EncodeSummary};
use crate::services::notify::{
    discord_notify, discord_notify_start, discord_notify_file_done,
    discord_notify_error, discord_notify_stats, discord_notify_progress,
};
use super::settings::load_config;

fn resolve_discord_creds(
    bot_token: String,
    log_channel_id: String,
) -> Result<(String, String), String> {
    let token = if !bot_token.is_empty() { bot_token }
        else { std::env::var("RENCODEX_DISCORD_TOKEN").unwrap_or_default() };
    let chan = if !log_channel_id.is_empty() { log_channel_id }
        else { std::env::var("RENCODEX_DISCORD_LOG_CHANNEL").unwrap_or_default() };
    if token.is_empty() || chan.is_empty() {
        return Err("Token ou log_channel_id manquant".to_string());
    }
    Ok((token, chan))
}

fn fields_for(cfg: &AppConfig, notif_type: &str) -> Vec<String> {
    cfg.discord_fields
        .get(notif_type)
        .cloned()
        .unwrap_or_else(|| crate::services::discord_fields::default_fields(notif_type))
}

fn note_for(cfg: &AppConfig, notif_type: &str) -> String {
    cfg.discord_custom_notes
        .get(notif_type)
        .cloned()
        .unwrap_or_default()
}

#[tauri::command]
pub async fn send_discord_notification(
    bot_token: String,
    log_channel_id: String,
    summary: EncodeSummary,
) -> Result<(), String> {
    let (token, chan) = resolve_discord_creds(bot_token, log_channel_id)?;
    let cfg = load_config();
    discord_notify(&token, &chan, &summary, &fields_for(&cfg, "summary"), &note_for(&cfg, "summary")).await;
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
    audio_label: String,
) -> Result<(), String> {
    let (token, chan) = resolve_discord_creds(bot_token, log_channel_id)?;
    let cfg = load_config();
    discord_notify_start(&token, &chan, total_files, total_size_mb, crf, &preset, &audio_label, &fields_for(&cfg, "start"), &note_for(&cfg, "start")).await;
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
    crf: u32,
    preset: String,
) -> Result<(), String> {
    let (token, chan) = resolve_discord_creds(bot_token, log_channel_id)?;
    let cfg = load_config();
    discord_notify_file_done(
        &token, &chan, &file_name,
        original_mb, encoded_mb, duration_secs,
        crf, &preset, &fields_for(&cfg, "file_done"), &note_for(&cfg, "file_done"),
    ).await;
    Ok(())
}

#[tauri::command]
pub async fn send_discord_error_notification(
    bot_token: String,
    log_channel_id: String,
    file_name: String,
    error_msg: String,
) -> Result<(), String> {
    let (token, chan) = resolve_discord_creds(bot_token, log_channel_id)?;
    let cfg = load_config();
    discord_notify_error(&token, &chan, &file_name, &error_msg, &fields_for(&cfg, "error"), &note_for(&cfg, "error")).await;
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
    let (token, chan) = resolve_discord_creds(bot_token, log_channel_id)?;
    let cfg = load_config();
    discord_notify_progress(
        &token, &chan,
        &file_name, file_index, file_total,
        percent, speed, remaining_secs, elapsed_secs,
        &fields_for(&cfg, "progress"), &note_for(&cfg, "progress"),
    ).await;
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
    let (token, chan) = resolve_discord_creds(bot_token, log_channel_id)?;
    discord_notify_stats(&token, &chan, total_files, total_original_mb, total_encoded_mb, total_duration_secs).await;
    Ok(())
}