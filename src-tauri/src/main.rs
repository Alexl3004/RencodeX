#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod models;
mod state;
mod utils;
mod naming;
mod services;
mod commands;
mod db;
mod entities;

use crate::utils::resolve_config;
use crate::commands::settings::load_config;
use tauri::Manager;

// ── Supervision du bot Discord ────────────────────────────────────────────────

/// Lance le bot Discord avec redémarrage automatique sur erreur.
/// Chaque tentative attend `backoff` avant de redémarrer, avec un plafond
/// de 5 minutes. Le bot tourne sur le runtime Tauri (pas de runtime dédié).
async fn supervise_discord_bot(token: String, channel_id: u64, app: tauri::AppHandle) {
    const MAX_BACKOFF_SECS: u64 = 300;
    let mut backoff_secs: u64 = 5;

    loop {
        eprintln!("[Discord bot] Démarrage…");

        if let Some(fatal) = services::discord_bot::start(token.clone(), channel_id, app.clone()).await {
            eprintln!("[Discord bot] Erreur fatale, supervision arrêtée : {fatal}");
            return;
        }

        eprintln!("[Discord bot] Déconnecté. Nouvelle tentative dans {}s.", backoff_secs);
        tokio::time::sleep(tokio::time::Duration::from_secs(backoff_secs)).await;
        backoff_secs = (backoff_secs * 2).min(MAX_BACKOFF_SECS);
    }
}

// ── Point d'entrée ────────────────────────────────────────────────────────────

fn main() {
    tauri::async_runtime::block_on(db::init());

    let cfg = resolve_config(load_config());

    if !std::path::Path::new(&cfg.ffmpeg_path).exists() {
        eprintln!(
            "[startup] AVERTISSEMENT : ffmpeg introuvable à « {} ».\n\
             Configurez le chemin dans Paramètres › FFmpeg avant d'encoder.",
            cfg.ffmpeg_path
        );
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // settings
            commands::settings::load_config,
            commands::settings::save_config,
            commands::settings::load_encoding_prefs,
            commands::settings::save_encoding_prefs,
            commands::settings::get_effective_config,
            commands::settings::get_discord_field_catalog,
            commands::settings::load_stats,
            commands::settings::save_stats,
            // encoding
            commands::encoding::check_ffmpeg,
            commands::encoding::analyze_file,
            commands::encoding::clean_filename,
            commands::encoding::start_encoding,
            commands::encoding::cancel_encoding,
            commands::encoding::pause_encoding,
            commands::encoding::resume_encoding,
            commands::encoding::get_paused,
            // files
            commands::files::get_default_output_dir,
            commands::files::list_subtitle_tracks,
            commands::files::extract_subtitles,
            commands::files::scan_folder,
            // discord
            commands::discord::send_discord_notification,
            commands::discord::send_discord_start_notification,
            commands::discord::send_discord_file_done_notification,
            commands::discord::send_discord_error_notification,
            commands::discord::send_discord_stats_notification,
            commands::discord::send_discord_progress_notification,
            commands::discord::send_email_report,
        ])
        .setup(move |app| {
            if cfg.discord_enabled
                && !cfg.discord_bot_token.is_empty()
                && !cfg.discord_cmd_channel_id.is_empty()
            {
                if let Ok(cmd_channel_id) = cfg.discord_cmd_channel_id.parse::<u64>() {
                    let token = cfg.discord_bot_token.clone();
                    let app_handle = app.handle().clone();

                    tauri::async_runtime::spawn(supervise_discord_bot(
                        token,
                        cmd_channel_id,
                        app_handle,
                    ));
                }
            }
            Ok(())
        })
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