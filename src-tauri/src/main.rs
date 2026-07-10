#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod tests;
mod models;
mod state;
mod regex;
mod utils;
mod filename;
mod media;
mod notify;
mod commands;
mod discord_fields;
mod db;
mod tests_extended;
mod entities;

use crate::utils::resolve_config;
use crate::commands::load_config;
use tauri::Manager;

fn main() {
    tauri::async_runtime::block_on(db::init());

    let cfg = resolve_config(load_config());

    // ── Vérification FFmpeg au démarrage ──────────────────────────────────────
    // On vérifie avant tout que le binaire configuré est accessible.
    // L'erreur apparaîtra dans les logs console et, côté Svelte, dans le panneau
    // de logs via la commande `check_ffmpeg` appelée dans `encoder.init()`.
    if !std::path::Path::new(&cfg.ffmpeg_path).exists() {
        eprintln!(
            "[startup] AVERTISSEMENT : ffmpeg introuvable à « {} ».\n\
             Configurez le chemin dans Paramètres › FFmpeg avant d'encoder.",
            cfg.ffmpeg_path
        );
    }
    // ─────────────────────────────────────────────────────────────────────────

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
            commands::pause_encoding,
            commands::resume_encoding,
            commands::get_paused,
            commands::get_default_output_dir,
            commands::load_config,
            commands::save_config,
            commands::load_encoding_prefs,
            commands::save_encoding_prefs,
            commands::get_effective_config,
            commands::get_discord_field_catalog,
            commands::check_ffmpeg,
            commands::send_discord_notification,
            commands::send_discord_start_notification,
            commands::send_discord_file_done_notification,
            commands::send_discord_error_notification,
            commands::send_discord_stats_notification,
            commands::send_discord_progress_notification,
            commands::scan_folder,
            commands::send_email_report,
            commands::load_stats,
            commands::save_stats,
            commands::list_subtitle_tracks,
            commands::extract_subtitles,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let is_encoding = crate::state::ENCODING.load(std::sync::atomic::Ordering::Acquire);

                if !is_encoding {
                    return;
                }

                api.prevent_close();

                let window = window.clone();
                tauri::async_runtime::spawn(async move {
                    use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

                    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();

                    window
                        .app_handle()
                        .dialog()
                        .message("Un encodage est en cours.\n\nFermer l'application va arrêter FFmpeg et supprimer le fichier en cours de traitement.")
                        .title("Fermeture — encodage actif")
                        .buttons(MessageDialogButtons::OkCancelCustom(
                            "Fermer quand même".to_string(),
                            "Annuler".to_string(),
                        ))
                        .show(move |confirmed| {
                            let _ = tx.send(confirmed);
                        });

                    let confirmed = rx.await.unwrap_or(false);

                    if !confirmed {
                        return;
                    }

                    let out_opt = crate::state::kill_ffmpeg_process();

                    if let Some(out_path) = out_opt {
                        tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
                        let _ = crate::utils::delete_partial_output(&out_path);
                    }

                    window.destroy().ok();
                });
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}