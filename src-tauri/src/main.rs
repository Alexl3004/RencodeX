#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod state;
mod regex;
mod utils;
mod filename;
mod media;
mod notify;
mod commands;

use crate::utils::resolve_config;
use crate::commands::load_config;

fn main() {
    let cfg = resolve_config(load_config());

    if cfg.discord_enabled
        && !cfg.discord_bot_token.is_empty()
        && !cfg.discord_cmd_channel_id.is_empty()
    {
        if let Ok(cmd_channel_id) = cfg.discord_cmd_channel_id.parse::<u64>() {
            let token = cfg.discord_bot_token.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(notify::discord_bot::start(token, cmd_channel_id));
            });
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::analyze_file,
            commands::clean_filename,
            commands::start_encoding,
            commands::cancel_encoding,
            commands::get_default_output_dir,
            commands::load_config,
            commands::save_config,
            commands::get_effective_config,
            commands::send_discord_notification,
            commands::send_discord_start_notification,
            commands::send_discord_file_done_notification,
            commands::send_discord_error_notification,
            commands::send_discord_stats_notification,
            commands::send_discord_progress_notification,
            commands::scan_folder,
            commands::send_email_report,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}