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
use tauri::Listener;
use std::sync::{Mutex, OnceLock};
use tokio_util::sync::CancellationToken;

// ── Singleton de supervision du bot Discord ───────────────────────────────────

/// Token d'annulation de la tâche de supervision en cours.
/// `cancel()` déclenche un shutdown gracieux ; `None` si aucune tâche active.
static BOT_TOKEN: OnceLock<Mutex<Option<CancellationToken>>> = OnceLock::new();

fn bot_token_slot() -> &'static Mutex<Option<CancellationToken>> {
    BOT_TOKEN.get_or_init(|| Mutex::new(None))
}

/// Annule la tâche de supervision précédente (shutdown gracieux) puis en lance
/// une nouvelle avec un token frais.
fn spawn_bot_supervisor(token: String, channel_id: u64, app: tauri::AppHandle) {
    let cancel = CancellationToken::new();
    let mut guard = bot_token_slot().lock().unwrap();
    if let Some(old) = guard.take() {
        old.cancel();
    }
    *guard = Some(cancel.clone());
    drop(guard);

    tauri::async_runtime::spawn(supervise_discord_bot(token, channel_id, app, cancel));
}

// ── Supervision du bot Discord ────────────────────────────────────────────────

/// Lance le bot Discord avec redémarrage automatique sur erreur.
///
/// Avec twilight, `start()` gère lui-même l'annulation via `session_cancel` :
/// dès que `cancel` est déclenché, `session_cancel` l'est aussi (via le watcher
/// ci-dessous), ce qui provoque la fermeture propre du shard dans `start()`.
/// On attend simplement que `start()` revienne, puis on sort sans redémarrer.
///
/// Backoff exponentiel plafonné à 5 min entre chaque tentative ratée.
async fn supervise_discord_bot(
    token: String,
    channel_id: u64,
    app: tauri::AppHandle,
    cancel: CancellationToken,
) {
    const MAX_BACKOFF_SECS: u64 = 300;
    let mut backoff_secs: u64 = 5;
    let mut current_token = token;
    let mut current_channel = channel_id;
    let mut consecutive_auth_failures: u32 = 0;

    // Délai initial court : laisse le réseau s'initialiser.
    // Twilight établit la connexion TLS + WebSocket lui-même ; 3s suffisent
    // dans la grande majorité des cas (vs 10s nécessaires avec Serenity).
    tokio::select! {
        _ = cancel.cancelled() => return,
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(3)) => {}
    }

    loop {
        eprintln!("[Discord bot] Démarrage…");

        // session_cancel est un token indépendant propagé à start().
        // Un watcher le déclenche dès que le parent (cancel) est annulé,
        // ce qui provoque la fermeture propre du shard dans start().
        let session_cancel = CancellationToken::new();
        {
            let parent = cancel.clone();
            let child  = session_cancel.clone();
            tauri::async_runtime::spawn(async move {
                parent.cancelled().await;
                child.cancel();
            });
        }

        // Appel unique à start() — pas de select! double.
        // start() revient dès que le shard se déconnecte (proprement ou sur erreur).
        let result = services::discord_bot::start(
            current_token.clone(),
            current_channel,
            app.clone(),
            session_cancel,
        ).await;

        // Si cancel a été déclenché pendant start(), on sort sans redémarrer.
        if cancel.is_cancelled() {
            eprintln!("[Discord bot] Supervision arrêtée (arrêt gracieux).");
            return;
        }

        if let Some(fatal_msg) = result {
            consecutive_auth_failures += 1;
            eprintln!(
                "[Discord bot] Échec connexion ({}) : {}",
                consecutive_auth_failures, fatal_msg
            );
            // Fatale après 5 échecs consécutifs (token invalide, config erronée…)
            if consecutive_auth_failures >= 5 {
                eprintln!(
                    "[Discord bot] Supervision arrêtée après {} échecs consécutifs.",
                    consecutive_auth_failures
                );
                return;
            }
        } else {
            // Connexion réussie puis perdue → réinitialiser le compteur
            consecutive_auth_failures = 0;
        }

        eprintln!("[Discord bot] Déconnecté. Nouvelle tentative dans {}s.", backoff_secs);
        tokio::select! {
            _ = cancel.cancelled() => {
                eprintln!("[Discord bot] Arrêt gracieux pendant backoff, supervision arrêtée.");
                return;
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(backoff_secs)) => {}
        }
        backoff_secs = (backoff_secs * 2).min(MAX_BACKOFF_SECS);

        // Relire la config à chaque redémarrage — prend en compte les mises à jour.
        let cfg = resolve_config(load_config());
        if !cfg.discord_bot_token.is_empty() {
            current_token = cfg.discord_bot_token;
        }
        if let Ok(id) = cfg.discord_cmd_channel_id.parse::<u64>() {
            current_channel = id;
        }
    }
}

// ── Point d'entrée ────────────────────────────────────────────────────────────

fn main() {
    // Installer le crypto provider rustls (ring) avant toute connexion TLS.
    // Sans ça, rustls panic avec "No default crypto provider installed".
    // Doit être fait une seule fois, avant db::init() et avant le runtime Tauri.
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Impossible d'installer le crypto provider rustls");

    tauri::async_runtime::block_on(db::init());
    let cfg = resolve_config(load_config());



    // Plus de set_var pour RENCODEX_DISCORD_TOKEN : le fichier config est
    // désormais prioritaire sur l'env var (voir resolve_config dans utils.rs).
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
            commands::settings::get_bot_status,
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
            commands::encoding::skip_encoding,
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
        ])
        .setup(move |app| {
            if !cfg.discord_bot_token.is_empty()
                && !cfg.discord_cmd_channel_id.is_empty()
            {
                if let Ok(cmd_channel_id) = cfg.discord_cmd_channel_id.parse::<u64>() {
                    let token = cfg.discord_bot_token.clone();
                    let app_handle = app.handle().clone();
                    spawn_bot_supervisor(token, cmd_channel_id, app_handle);
                }
            }

            // Redémarrer (ou arrêter) le bot quand la config est sauvegardée
            let app_handle = app.handle().clone();
            app.listen("config-saved", move |_| {
                let app2 = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    let cfg = resolve_config(load_config());
                    if !cfg.discord_bot_token.is_empty()
                        && !cfg.discord_cmd_channel_id.is_empty()
                    {
                        if let Ok(id) = cfg.discord_cmd_channel_id.parse::<u64>() {
                            eprintln!("[Discord bot] Config mise à jour, redémarrage…");
                            spawn_bot_supervisor(cfg.discord_bot_token, id, app2);
                        }
                    } else {
                        let mut guard = bot_token_slot().lock().unwrap();
                        if let Some(old) = guard.take() {
                            eprintln!("[Discord bot] Config mise à jour, arrêt du bot…");
                            old.cancel();
                        }
                    }
                });
            });

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