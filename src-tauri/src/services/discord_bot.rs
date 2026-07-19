use tauri::{AppHandle, Emitter};
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use std::collections::HashMap;
use std::time::Instant;
use tokio_util::sync::CancellationToken;
use crate::state::{
    ENCODING, snapshot,
    pause_ffmpeg_process, resume_ffmpeg_process, kill_ffmpeg_process, skip_ffmpeg_process,
};
use serenity::async_trait;
use serenity::builder::{CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter, CreateMessage, CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::ButtonStyle;
use serenity::model::application::Interaction;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

// ── Rate limiting ─────────────────────────────────────────────────────────────

/// Fenêtre glissante : max 5 commandes par utilisateur sur 10 secondes.
const RATE_LIMIT_MAX: usize = 2;
const RATE_LIMIT_WINDOW_SECS: u64 = 10;

/// Horodatages des N dernières commandes par user_id.
type RateLimitMap = Mutex<HashMap<u64, Vec<Instant>>>;

/// Vérifie si `user_id` a dépassé la limite de taux.
/// Retourne `true` si la commande est autorisée, `false` si bloquée.
fn check_rate_limit(map: &RateLimitMap, user_id: u64) -> bool {
    let mut guard = map.lock().unwrap();
    let now = Instant::now();
    let window = std::time::Duration::from_secs(RATE_LIMIT_WINDOW_SECS);

    let timestamps = guard.entry(user_id).or_default();

    // Supprimer les entrées expirées (hors fenêtre)
    timestamps.retain(|t| now.duration_since(*t) < window);

    if timestamps.len() >= RATE_LIMIT_MAX {
        return false;
    }

    timestamps.push(now);
    true
}

// ── Handler ───────────────────────────────────────────────────────────────────

struct Handler {
    channel_id: u64,
    app: AppHandle,
    rate_limits: RateLimitMap,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.channel_id.get() != self.channel_id {
            return;
        }
        if msg.author.bot {
            return;
        }

        let content = msg.content.trim().to_lowercase();
        if !content.starts_with('!') {
            return;
        }

        // Rate limiting — vérifié avant tout traitement
        let user_id = msg.author.id.get();
        if !check_rate_limit(&self.rate_limits, user_id) {
            let _ = msg.channel_id.say(
                &ctx.http,
                format!(
                    "⏱️ Trop de commandes. Limite : {} commandes / {}s.",
                    RATE_LIMIT_MAX, RATE_LIMIT_WINDOW_SECS
                ),
            ).await;
            return;
        }

        // Commande !panel — envoi d'un embed avec boutons interactifs
        if content.as_str() == "!panel" {
            send_panel(&ctx, &msg).await;
            return;
        }

        let reply = match content.as_str() {
            "!help"   => format_help(),
            "!status" => format_status(),
            "!queue"  => format_queue(),
            "!skip" => {
                if skip_ffmpeg_process() {
                    "⏭ Fichier actuel ignoré, passage au suivant.".to_string()
                } else {
                    "ℹ️ Aucun encodage en cours.".to_string()
                }
            }
            "!cancel" => {
                if ENCODING.load(Ordering::Acquire) {
                    kill_ffmpeg_process();
                    let _ = self.app.emit("encode-cancelled", ());
                    "⏹ Encodage annulé depuis Discord.".to_string()
                } else {
                    "ℹ️ Aucun encodage en cours.".to_string()
                }
            }
            "!pause" => {
                if pause_ffmpeg_process() {
                    let _ = self.app.emit("encode-paused", true);
                    "⏸ Encodage mis en pause.".to_string()
                } else {
                    "ℹ️ Impossible de mettre en pause (aucun encodage actif ou déjà en pause).".to_string()
                }
            }
            "!resume" => {
                if resume_ffmpeg_process() {
                    let _ = self.app.emit("encode-paused", false);
                    "▶️ Encodage repris.".to_string()
                } else {
                    "ℹ️ Impossible de reprendre (aucune pause active).".to_string()
                }
            }
            _ => return,
        };

        if let Err(e) = msg.channel_id.say(&ctx.http, reply).await {
            eprintln!("[Discord bot] Erreur envoi réponse : {e}");
        }
    }

    // Gestion des clics sur les boutons du panneau
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let Interaction::Component(component) = interaction else { return };

        // Rate limiting sur les interactions boutons (même pool que les commandes texte)
        let user_id = component.user.id.get();
        if !check_rate_limit(&self.rate_limits, user_id) {
            let resp = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!(
                        "⏱️ Trop de commandes. Limite : {} commandes / {}s.",
                        RATE_LIMIT_MAX, RATE_LIMIT_WINDOW_SECS
                    ))
                    .ephemeral(true),
            );
            let _ = component.create_response(&ctx.http, resp).await;
            return;
        }

        let response_text = match component.data.custom_id.as_str() {
            "status" => format_status(),
            "queue"  => format_queue(),
            "pause" => {
                if pause_ffmpeg_process() {
                    let _ = self.app.emit("encode-paused", true);
                    "⏸ Encodage mis en pause.".to_string()
                } else {
                    "ℹ️ Impossible de mettre en pause (aucun encodage actif ou déjà en pause).".to_string()
                }
            }
            "resume" => {
                if resume_ffmpeg_process() {
                    let _ = self.app.emit("encode-paused", false);
                    "▶️ Encodage repris.".to_string()
                } else {
                    "ℹ️ Impossible de reprendre (aucune pause active).".to_string()
                }
            }
            "cancel" => {
                if ENCODING.load(Ordering::Acquire) {
                    kill_ffmpeg_process();
                    let _ = self.app.emit("encode-cancelled", ());
                    "⏹ Encodage annulé depuis Discord.".to_string()
                } else {
                    "ℹ️ Aucun encodage en cours.".to_string()
                }
            }
            _ => return,
        };

        // Réponse éphémère : visible uniquement par l'auteur du clic
        let resp = CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content(response_text)
                .ephemeral(true),
        );
        if let Err(e) = component.create_response(&ctx.http, resp).await {
            eprintln!("[Discord bot] Erreur réponse interaction : {e}");
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("[Discord bot] Connecté en tant que {}", ready.user.name);
        let _ = self.app.emit("discord-bot-connected", serde_json::json!({ "name": ready.user.name }));
    }
}

// Envoie le panneau de contrôle avec embed + boutons
async fn send_panel(ctx: &Context, msg: &Message) {
    let embed = CreateEmbed::new()
        .title("🤖 RenCodeX — Panneau de Contrôle")
        .description("Utilisez les boutons ci-dessous pour gérer vos encodages.")
        .color(0x3498db)
        .field("📊 Status", "Voir la progression en cours", true)
        .field("📋 Queue", "Fichiers en attente", true)
        .footer(CreateEmbedFooter::new("RenCodeX Bot • Rust Edition"));

    let buttons = CreateActionRow::Buttons(vec![
        CreateButton::new("status").style(ButtonStyle::Primary).label("📊 Status"),
        CreateButton::new("queue").style(ButtonStyle::Secondary).label("📋 Queue"),
        CreateButton::new("pause").style(ButtonStyle::Secondary).label("⏸️ Pause"),
        CreateButton::new("resume").style(ButtonStyle::Success).label("▶️ Reprendre"),
        CreateButton::new("cancel").style(ButtonStyle::Danger).label("🛑 Annuler"),
    ]);

    let message = CreateMessage::new()
        .embed(embed)
        .components(vec![buttons]);

    if let Err(e) = msg.channel_id.send_message(&ctx.http, message).await {
        eprintln!("[Discord bot] Erreur envoi panneau : {e}");
    }
}

fn format_help() -> String {
    "**RenCodeX — Commandes disponibles**\n\
     `!status`  — Progression de l'encodage en cours\n\
     `!queue`   — Liste des fichiers en attente\n\
     `!skip`    — Passer au fichier suivant\n\
     `!pause`   — Mettre en pause\n\
     `!resume`  — Reprendre\n\
     `!cancel`  — Annuler l'encodage\n\
     `!panel`   — Ouvrir le panneau de contrôle interactif\n\
     `!help`    — Afficher cette aide"
        .to_string()
}

fn format_status() -> String {
    let s = snapshot();
    if !s.encoding {
        return "💤 Aucun encodage en cours.".to_string();
    }
    let filled = (s.percent / 100.0 * 20.0) as usize;
    let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(20 - filled));
    let elapsed = s.elapsed_secs;
    let rem_m = s.remaining as u64 / 60;
    let rem_s = s.remaining as u64 % 60;
    format!(
        "**📦 Encodage en cours**\n\
         Fichier : `{}` ({}/{})\n\
         {} **{:.1}%**\n\
         Vitesse : `{:.2}x`  |  Restant : `{}m{}s`\n\
         Temps écoulé : `{}m{}s`",
        s.current_file, s.file_index + 1, s.file_total,
        bar, s.percent, s.speed,
        rem_m, rem_s, elapsed / 60, elapsed % 60,
    )
}

fn format_queue() -> String {
    let s = snapshot();
    if s.queue_names.is_empty() {
        return "📭 File vide.".to_string();
    }
    let mut lines = format!(
        "**📋 File d'attente — {}/{} fichiers**\n",
        s.file_index + 1, s.file_total
    );
    for (i, name) in s.queue_names.iter().enumerate() {
        let prefix = if i < s.file_index { "✅" }
            else if i == s.file_index && s.encoding { "⏳" }
            else { "⏸" };
        lines.push_str(&format!("{} `{}`\n", prefix, name));
    }
    lines
}

pub async fn start(
    token: String,
    channel_id: u64,
    app: AppHandle,
    cancel: CancellationToken,
) -> Option<String> {
    let token = token.trim().to_string();
    let token = token.strip_prefix("Bot ").unwrap_or(&token).to_string();
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = match Client::builder(&token, intents)
        .event_handler(Handler {
            channel_id,
            app: app.clone(),
            rate_limits: Mutex::new(HashMap::new()),
        })
        .await
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[Discord bot] Impossible de créer le client : {e}");
            // Erreur à la création = token invalide ou config erronée → fatale
            return Some(e.to_string());
        }
    };

    // Surveillance du token d'annulation : déclenche shutdown_all() sur le
    // shard manager dès que cancel.cancelled() se résout. La tâche se termine
    // d'elle-même une fois client.start() retourné.
    let shard_manager = client.shard_manager.clone();
    tauri::async_runtime::spawn(async move {
        cancel.cancelled().await;
        eprintln!("[Discord bot] Arrêt gracieux : fermeture des shards…");
        shard_manager.shutdown_all().await;
    });

    if let Err(err) = client.start().await {
        let err_str = err.to_string();
        eprintln!("[Discord bot] Erreur de connexion : {err_str}");

        // "invalid authentication" et "401" sont des erreurs fatales
        let is_fatal = err_str.contains("invalid authentication")
            || err_str.contains("401")
            || err_str.contains("Invalid token");

        if is_fatal {
            let _ = app.emit("discord-bot-error", serde_json::json!({ "message": err_str.clone() }));
            return Some(err_str);
        }
    }

    // Déconnexion propre (ou arrêt via shutdown_all) → None = redémarrage autorisé
    let _ = app.emit("discord-bot-disconnected", ());
    None
}