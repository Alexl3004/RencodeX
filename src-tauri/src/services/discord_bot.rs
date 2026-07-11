use tauri::{AppHandle, Emitter};
use std::sync::atomic::Ordering;
use crate::state::{
    ENCODING, snapshot,
    pause_ffmpeg_process, resume_ffmpeg_process, kill_ffmpeg_process,
};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler {
    channel_id: u64,
    app: AppHandle,
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
        let reply = match content.as_str() {
            "!help"   => format_help(),
            "!status" => format_status(),
            "!queue"  => format_queue(),
            "!skip" => {
                if ENCODING.load(Ordering::Acquire) {
                    kill_ffmpeg_process();
                    let _ = self.app.emit("encode-cancelled", ());
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

    async fn ready(&self, _: Context, ready: Ready) {
        println!("[Discord bot] Connecté en tant que {}", ready.user.name);
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

pub async fn start(token: String, channel_id: u64, app: AppHandle) {
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = match Client::builder(&token, intents)
        .event_handler(Handler { channel_id, app })
        .await
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[Discord bot] Impossible de créer le client : {e}");
            return;
        }
    };

    if let Err(e) = client.start().await {
        eprintln!("[Discord bot] Erreur de connexion : {e}");
    }
}