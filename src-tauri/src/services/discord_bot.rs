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

use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt as _};
use twilight_http::Client as HttpClient;
use twilight_model::application::interaction::{
    InteractionData, InteractionType,
};
use twilight_model::channel::message::component::ButtonStyle;
use twilight_model::channel::message::MessageFlags;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseType,
};
use twilight_model::id::marker::ChannelMarker;
use twilight_model::id::Id;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder, EmbedFooterBuilder};
use twilight_util::builder::InteractionResponseDataBuilder;

// ── Rate limiting ─────────────────────────────────────────────────────────────

/// Fenêtre glissante : max 2 commandes par utilisateur sur 10 secondes.
const RATE_LIMIT_MAX: usize = 2;
const RATE_LIMIT_WINDOW_SECS: u64 = 10;

type RateLimitMap = Mutex<HashMap<u64, Vec<Instant>>>;

fn check_rate_limit(map: &RateLimitMap, user_id: u64) -> bool {
    let mut guard = map.lock().unwrap();
    let now = Instant::now();
    let window = std::time::Duration::from_secs(RATE_LIMIT_WINDOW_SECS);
    let timestamps = guard.entry(user_id).or_default();
    timestamps.retain(|t| now.duration_since(*t) < window);
    if timestamps.len() >= RATE_LIMIT_MAX {
        return false;
    }
    timestamps.push(now);
    true
}

// ── Helpers ───────────────────────────────────────────────────────────────────

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

// ── Envoi de messages / interactions via HTTP ─────────────────────────────────

async fn say(http: &HttpClient, channel_id: Id<ChannelMarker>, text: &str) {
    if let Err(e) = http
        .create_message(channel_id)
        .content(text)
        .await
    {
        eprintln!("[Discord bot] Erreur envoi message : {e}");
    }
}


/// Envoie le panneau de contrôle avec embed + boutons.
async fn send_panel(http: &HttpClient, channel_id: Id<ChannelMarker>) {
    use twilight_model::channel::message::component::{
        ActionRow, Button, Component,
    };

    let embed = EmbedBuilder::new()
        .title("🤖 RenCodeX — Panneau de Contrôle")
        .description("Utilisez les boutons ci-dessous pour gérer vos encodages.")
        .color(0x3498db)
        .field(EmbedFieldBuilder::new("📊 Status", "Voir la progression en cours").inline())
        .field(EmbedFieldBuilder::new("📋 Queue", "Fichiers en attente").inline())
        .footer(EmbedFooterBuilder::new("RenCodeX Bot • Rust Edition"))
        .build();

    fn btn(id: &str, label: &str, style: ButtonStyle) -> Component {
        Component::Button(Button {
            custom_id: Some(id.to_string()),
            disabled: false,
            emoji: None,
            label: Some(label.to_string()),
            style,
            url: None,
            sku_id: None,
        })
    }

    let row1 = Component::ActionRow(ActionRow {
        components: vec![
            btn("status",  "📊 Status",    ButtonStyle::Primary),
            btn("queue",   "📋 Queue",     ButtonStyle::Secondary),
            btn("pause",   "⏸️ Pause",    ButtonStyle::Secondary),
            btn("resume",  "▶️ Reprendre", ButtonStyle::Success),
        ],
    });

    let row2 = Component::ActionRow(ActionRow {
        components: vec![
            btn("skip",   "⏭️ Skip",    ButtonStyle::Primary),
            btn("cancel", "🛑 Annuler", ButtonStyle::Danger),
        ],
    });

    if let Err(e) = http
        .create_message(channel_id)
        .embeds(&[embed])
        .components(&[row1, row2])
        .await
    {
        eprintln!("[Discord bot] Erreur envoi panneau : {e}");
    }
}

// ── Boucle principale ─────────────────────────────────────────────────────────

pub async fn start(
    token: String,
    channel_id: u64,
    app: AppHandle,
    cancel: CancellationToken,
) -> Option<String> {
    // Nettoyage défensif du token : supprime espaces, guillemets, retours
    // à la ligne et le préfixe "Bot " éventuel, dans n'importe quel ordre.
    let token = token
        .trim()
        .trim_matches('"')   // guillemets JSON parasites
        .trim_matches('\'')  // guillemets simples
        .trim()
        .to_string();
    let token = token.strip_prefix("Bot ").unwrap_or(&token).trim().to_string();



    let intents = Intents::GUILD_MESSAGES
        | Intents::DIRECT_MESSAGES
        | Intents::MESSAGE_CONTENT;

    // Construire le client HTTP
    let http = std::sync::Arc::new(HttpClient::new(format!("Bot {}", token)));

    // Valider le token et récupérer l'app_id via /users/@me.
    // On utilise current_user() (pas current_user_application() qui exige OAuth2).
    // Pour les bots, user.id == application_id.
    let app_id = match http.current_user().await {
        Ok(resp) => match resp.model().await {
            Ok(user) => {
                // Convertir UserId → ApplicationId (même valeur numérique)
                twilight_model::id::Id::new(user.id.get())
            }
            Err(e) => {
                eprintln!("[Discord bot] Impossible de lire l'utilisateur : {e}");
                return Some(e.to_string());
            }
        },
        Err(e) => {
            eprintln!("[Discord bot] Erreur auth (token invalide ?) : {e}");
            return Some(e.to_string());
        }
    };

    // Construire le shard (infaillible dans twilight 0.16+)
    let mut shard = Shard::new(ShardId::ONE, format!("Bot {}", token), intents);

    let channel = Id::<ChannelMarker>::new(channel_id);
    let rate_limits: std::sync::Arc<RateLimitMap> =
        std::sync::Arc::new(Mutex::new(HashMap::new()));


    loop {
        let event = tokio::select! {
            _ = cancel.cancelled() => {
                eprintln!("[Discord bot] Arrêt gracieux demandé.");
                shard.close(twilight_gateway::CloseFrame::NORMAL);
                break;
            }
            ev = shard.next_event(EventTypeFlags::all()) => {
                match ev {
                    Some(Ok(ev)) => ev,
                    Some(Err(e)) => {
                        let err_str = e.to_string();
                        eprintln!("[Discord bot] Erreur gateway : {err_str}");
                        // twilight 0.16 : pas de is_fatal(), on détecte via les
                        // close codes Discord (4004 = auth failed, 4010/4011 = shard)
                        let is_fatal = err_str.contains("4004")
                            || err_str.contains("4010")
                            || err_str.contains("4011")
                            || err_str.contains("invalid token")
                            || err_str.contains("authentication failed");
                        if is_fatal {
                            eprintln!("[Discord bot] Erreur fatale, arrêt.");
                            crate::state::BOT_CONNECTED.store(false, Ordering::Release);
                            *crate::state::BOT_NAME.lock().unwrap_or_else(|e| e.into_inner()) = String::new();
                            let _ = app.emit("discord-bot-error", serde_json::json!({ "message": err_str }));
                            return None;
                        }
                        continue;
                    }
                    None => {
                        eprintln!("[Discord bot] Stream fermé.");
                        break;
                    }
                }
            }
        };

        match event {
            Event::Ready(ready) => {
                let name = ready.user.name.clone();
                eprintln!("[Discord bot] Connecté en tant que {name}");
                crate::state::BOT_CONNECTED.store(true, Ordering::Release);
                *crate::state::BOT_NAME.lock().unwrap_or_else(|e| e.into_inner()) = name.clone();
                let _ = app.emit("discord-bot-connected", serde_json::json!({ "name": name }));
            }

            Event::MessageCreate(msg) => {
                if msg.channel_id != channel { continue; }
                if msg.author.bot { continue; }

                let content = msg.content.trim().to_lowercase();
                if !content.starts_with('!') { continue; }

                let user_id = msg.author.id.get();
                if !check_rate_limit(&rate_limits, user_id) {
                    let text = format!(
                        "⏱️ Trop de commandes. Limite : {} commandes / {}s.",
                        RATE_LIMIT_MAX, RATE_LIMIT_WINDOW_SECS
                    );
                    say(&http, channel, &text).await;
                    continue;
                }

                if content.as_str() == "!panel" {
                    send_panel(&http, channel).await;
                    continue;
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
                            let _ = app.emit("encode-cancelled", ());
                            "⏹ Encodage annulé depuis Discord.".to_string()
                        } else {
                            "ℹ️ Aucun encodage en cours.".to_string()
                        }
                    }
                    "!pause" => {
                        if pause_ffmpeg_process() {
                            let _ = app.emit("encode-paused", true);
                            "⏸ Encodage mis en pause.".to_string()
                        } else {
                            "ℹ️ Impossible de mettre en pause (aucun encodage actif ou déjà en pause).".to_string()
                        }
                    }
                    "!resume" => {
                        if resume_ffmpeg_process() {
                            let _ = app.emit("encode-paused", false);
                            "▶️ Encodage repris.".to_string()
                        } else {
                            "ℹ️ Impossible de reprendre (aucune pause active).".to_string()
                        }
                    }
                    _ => continue,
                };

                say(&http, channel, &reply).await;
            }

            Event::InteractionCreate(interaction) => {
                if interaction.kind != InteractionType::MessageComponent { continue; }

                let user_id = interaction
                    .author_id()
                    .map(|id| id.get())
                    .unwrap_or(0);

                if !check_rate_limit(&rate_limits, user_id) {
                    let data = InteractionResponseDataBuilder::new()
                        .content(format!(
                            "⏱️ Trop de commandes. Limite : {} commandes / {}s.",
                            RATE_LIMIT_MAX, RATE_LIMIT_WINDOW_SECS
                        ))
                        .flags(MessageFlags::EPHEMERAL)
                        .build();
                    let resp = InteractionResponse {
                        kind: InteractionResponseType::ChannelMessageWithSource,
                        data: Some(data),
                    };
                    let _ = http
                        .interaction(app_id)
                        .create_response(interaction.id, &interaction.token, &resp)
                        .await;
                    continue;
                }

                let custom_id = match &interaction.data {
                    Some(InteractionData::MessageComponent(d)) => d.custom_id.clone(),
                    _ => continue,
                };

                let response_text = match custom_id.as_str() {
                    "status" => format_status(),
                    "queue"  => format_queue(),
                    "skip" => {
                        if skip_ffmpeg_process() {
                            "⏭ Fichier actuel ignoré, passage au suivant.".to_string()
                        } else {
                            "ℹ️ Aucun encodage en cours.".to_string()
                        }
                    }
                    "pause" => {
                        if pause_ffmpeg_process() {
                            let _ = app.emit("encode-paused", true);
                            "⏸ Encodage mis en pause.".to_string()
                        } else {
                            "ℹ️ Impossible de mettre en pause (aucun encodage actif ou déjà en pause).".to_string()
                        }
                    }
                    "resume" => {
                        if resume_ffmpeg_process() {
                            let _ = app.emit("encode-paused", false);
                            "▶️ Encodage repris.".to_string()
                        } else {
                            "ℹ️ Impossible de reprendre (aucune pause active).".to_string()
                        }
                    }
                    "cancel" => {
                        if ENCODING.load(Ordering::Acquire) {
                            kill_ffmpeg_process();
                            let _ = app.emit("encode-cancelled", ());
                            "⏹ Encodage annulé depuis Discord.".to_string()
                        } else {
                            "ℹ️ Aucun encodage en cours.".to_string()
                        }
                    }
                    _ => continue,
                };

                let data = InteractionResponseDataBuilder::new()
                    .content(response_text)
                    .flags(MessageFlags::EPHEMERAL)
                    .build();
                let resp = InteractionResponse {
                    kind: InteractionResponseType::ChannelMessageWithSource,
                    data: Some(data),
                };
                if let Err(e) = http
                    .interaction(app_id)
                    .create_response(interaction.id, &interaction.token, &resp)
                    .await
                {
                    eprintln!("[Discord bot] Erreur réponse interaction : {e}");
                }
            }

            Event::GatewayClose(_) => {
                eprintln!("[Discord bot] Gateway fermée.");
                break;
            }

            _ => {}
        }
    }

    crate::state::BOT_CONNECTED.store(false, Ordering::Release);
    *crate::state::BOT_NAME.lock().unwrap_or_else(|e| e.into_inner()) = String::new();
    let _ = app.emit("discord-bot-disconnected", ());
    None
}