use crate::models::{EncodeSummary, EmailConfig};
use once_cell::sync::Lazy;
use std::time::Duration;

static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("RenCodeX/2.0")
        .build()
        .expect("Failed to create reqwest::Client")
});

async fn send_discord_message(token: &str, channel_id: &str, payload: &serde_json::Value) {
    let url = format!("https://discord.com/api/v10/channels/{}/messages", channel_id);
    for attempt in 0..3 {
        let resp = HTTP_CLIENT
            .post(&url)
            .header("Authorization", format!("Bot {}", token))
            .header("Content-Type", "application/json")
            .json(payload)
            .send()
            .await;
        match resp {
            Ok(r) if r.status().as_u16() == 429 => {
                let retry_after = r
                    .json::<serde_json::Value>()
                    .await
                    .ok()
                    .and_then(|v| v["retry_after"].as_f64())
                    .unwrap_or(1.0);
                tokio::time::sleep(Duration::from_secs_f64(retry_after)).await;
            }
            Ok(_) => return,
            Err(_) if attempt < 2 => {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Err(e) => {
                eprintln!("[Discord] Erreur envoi: {e}");
                return;
            }
        }
    }
}

fn unix_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

fn duration_to_str(seconds: u64) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;
    if h > 0 { format!("{}h {}m {}s", h, m, s) }
    else if m > 0 { format!("{}m {}s", m, s) }
    else { format!("{}s", s) }
}

fn thumbnail_for(color: u32) -> &'static str {
    match color {
        0x4CAF6E => "https://i.imgur.com/tTlFcKX.png",
        0xE74C3C => "https://i.imgur.com/9q0s8cM.png",
        0xC8882A => "https://i.imgur.com/0qLzKxH.png",
        0x3498DB => "https://i.imgur.com/8TfG5dR.png",
        0x9B59B6 => "https://i.imgur.com/S1Jhq0p.png",
        0x2ECC71 => "https://i.imgur.com/3wLcGqN.png",
        _ => "",
    }
}

pub async fn discord_notify(token: &str, channel_id: &str, summary: &EncodeSummary) {
    if token.is_empty() || channel_id.is_empty() { return; }

    let success = summary.files.iter().filter(|f| f.status == "ok").count();
    let cancelled = summary.files.iter().filter(|f| f.status == "cancelled").count();
    let errors = summary.files.iter().filter(|f| f.status == "error").count();
    let total = summary.files.len();

    let gain_pct = if summary.total_original_mb > 0.0 {
        100.0 * (summary.total_original_mb - summary.total_encoded_mb) / summary.total_original_mb
    } else { 0.0 };

    let elapsed = summary.total_secs as u64;

    let color: u32 = if errors > 0 && success == 0 { 0xCC3C3C }
        else if errors > 0 || cancelled > 0 { 0xC8882A }
        else { 0x4CAF6E };

    let saved_mb = summary.total_original_mb - summary.total_encoded_mb;
    let saved_gb = saved_mb / 1024.0;

    let mut detail = String::new();
    for f in summary.files.iter().take(10) {
        let icon = match f.status.as_str() {
            "ok" => "✅",
            "error" => "❌",
            "cancelled" => "⏹",
            _ => "•",
        };
        let gain = if f.status == "ok" && f.original_mb > 0.0 {
            format!(" `−{:.1}%`", 100.0 * (f.original_mb - f.encoded_mb) / f.original_mb)
        } else { String::new() };
        let size_info = if f.status == "ok" {
            format!(" `{:.1}MB`", f.encoded_mb)
        } else { String::new() };
        detail.push_str(&format!("{} `{}`{}{}\n", icon, f.name, gain, size_info));
    }
    if total > 10 {
        detail.push_str(&format!("… et {} autres fichiers\n", total - 10));
    }

    let mut embed = serde_json::json!({
        "title": if errors > 0 && success == 0 { "❌ Encodage échoué" } else if errors > 0 { "⚠️ Encodage partiel" } else { "✅ Encodage terminé" },
        "color": color,
        "thumbnail": { "url": thumbnail_for(color) },
        "fields": [
            { "name": "📁 Fichiers", "value": format!("`{}/{}` réussis", success, total), "inline": true },
            { "name": "📊 Gain total", "value": format!("`{:.1}%`", gain_pct), "inline": true },
            { "name": "⏱️ Durée", "value": format!("`{}`", duration_to_str(elapsed)), "inline": true },
        ],
        "footer": { "text": "RenCodeX · H.265 NVENC", "icon_url": "https://i.imgur.com/xxx.png" },
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    if saved_mb > 0.0 {
        let gain_str = if saved_gb >= 1.0 {
            format!("{:.2} GB", saved_gb)
        } else {
            format!("{:.0} MB", saved_mb)
        };
        embed["fields"].as_array_mut().unwrap().push(
            serde_json::json!({ "name": "💾 Espace libéré", "value": format!("`{}`", gain_str), "inline": true })
        );
    }

    if !detail.is_empty() {
        let detail_value = if detail.len() > 1000 {
            format!("{}…", &detail[..997])
        } else { detail };
        embed["fields"].as_array_mut().unwrap().push(
            serde_json::json!({ "name": "📋 Détail", "value": detail_value, "inline": false })
        );
    }

    let content = if errors > 0 && success > 0 {
        Some("⚠️ Certains fichiers ont échoué. Consultez les logs pour plus d'informations.")
    } else if errors > 0 && success == 0 {
        Some("❌ Aucun fichier n'a pu être encodé. Vérifiez votre configuration.")
    } else if cancelled > 0 {
        Some("⏹ L'encodage a été interrompu.")
    } else {
        None
    };

    let mut payload = serde_json::json!({
        "username": "RenCodeX",
        "embeds": [embed],
        "allowed_mentions": { "parse": [] }
    });

    if let Some(msg) = content {
        payload["content"] = serde_json::json!(msg);
    }

    send_discord_message(token, channel_id, &payload).await;
}

pub async fn discord_notify_start(token: &str, channel_id: &str, total_files: usize, total_size_mb: f64, crf: u32, preset: &str) {
    if token.is_empty() || channel_id.is_empty() { return; }

    let total_size_str = if total_size_mb >= 1024.0 {
        format!("{:.2} GB", total_size_mb / 1024.0)
    } else {
        format!("{:.0} MB", total_size_mb)
    };

    let payload = serde_json::json!({
        "username": "RenCodeX",
        "embeds": [{
            "title": "🎬 Encodage démarré",
            "color": 0x3498DB,
            "thumbnail": { "url": thumbnail_for(0x3498DB) },
            "fields": [
                { "name": "📁 Fichiers", "value": format!("`{}` fichiers", total_files), "inline": true },
                { "name": "💾 Taille totale", "value": format!("`{}`", total_size_str), "inline": true },
                { "name": "🎚️ CRF", "value": format!("`{}`", crf), "inline": true },
                { "name": "⚡ Preset", "value": format!("`{}`", preset.to_uppercase()), "inline": true },
                { "name": "🎨 Codec", "value": "`H.265 NVENC`", "inline": true },
                { "name": "🔊 Audio", "value": "`AAC 192k`", "inline": true },
            ],
            "footer": { "text": "RenCodeX · Encodage en cours" },
            "timestamp": chrono::Utc::now().to_rfc3339()
        }]
    });

    send_discord_message(token, channel_id, &payload).await;
}

pub async fn discord_notify_error(token: &str, channel_id: &str, file_name: &str, error_msg: &str) {
    if token.is_empty() || channel_id.is_empty() { return; }

    let truncated_error = if error_msg.len() > 500 {
        format!("{}…", &error_msg[..497])
    } else {
        error_msg.to_string()
    };

    let payload = serde_json::json!({
        "username": "RenCodeX",
        "embeds": [{
            "title": "❌ Erreur d'encodage",
            "color": 0xE74C3C,
            "thumbnail": { "url": thumbnail_for(0xE74C3C) },
            "fields": [
                { "name": "📁 Fichier", "value": format!("`{}`", file_name), "inline": false },
                { "name": "🐛 Erreur", "value": format!("```{}```", truncated_error), "inline": false }
            ],
            "footer": { "text": "RenCodeX · Action requise" },
            "timestamp": chrono::Utc::now().to_rfc3339()
        }],
        "allowed_mentions": { "parse": [] }
    });

    send_discord_message(token, channel_id, &payload).await;
}

pub async fn discord_notify_file_done(
    token: &str,
    channel_id: &str,
    file_name: &str,
    original_mb: f64,
    encoded_mb: f64,
    duration_secs: f64,
) {
    if token.is_empty() || channel_id.is_empty() { return; }

    let gain_pct = if original_mb > 0.0 {
        ((original_mb - encoded_mb) / original_mb) * 100.0
    } else { 0.0 };

    let saved_mb = original_mb - encoded_mb;
    let saved_str = if saved_mb >= 1024.0 {
        format!("{:.2} GB", saved_mb / 1024.0)
    } else {
        format!("{:.0} MB", saved_mb)
    };

    let emoji = if gain_pct > 50.0 { "🚀" } else if gain_pct > 30.0 { "📦" } else { "✅" };
    let duration_str = duration_to_str(duration_secs as u64);

    let payload = serde_json::json!({
        "username": "RenCodeX",
        "embeds": [{
            "title": format!("{} Fichier encodé", emoji),
            "color": 0x2ECC71,
            "thumbnail": { "url": thumbnail_for(0x2ECC71) },
            "fields": [
                { "name": "📁 Fichier", "value": format!("`{}`", file_name), "inline": false },
                { "name": "💾 Taille", "value": format!("`{:.1} MB` → `{:.1} MB`", original_mb, encoded_mb), "inline": true },
                { "name": "📊 Gain", "value": format!("`{:.1}%` (`{}`)", gain_pct, saved_str), "inline": true },
                { "name": "⏱️ Temps", "value": format!("`{}`", duration_str), "inline": true }
            ],
            "footer": { "text": "RenCodeX · Fichier terminé" },
            "timestamp": chrono::Utc::now().to_rfc3339()
        }]
    });

    send_discord_message(token, channel_id, &payload).await;
}

pub async fn discord_notify_stats(
    token: &str,
    channel_id: &str,
    total_files: usize,
    total_original_mb: f64,
    total_encoded_mb: f64,
    total_duration_secs: f64,
) {
    if token.is_empty() || channel_id.is_empty() { return; }

    let saved_mb = total_original_mb - total_encoded_mb;
    let gain_pct = if total_original_mb > 0.0 {
        (saved_mb / total_original_mb) * 100.0
    } else { 0.0 };

    let saved_str = if saved_mb >= 1024.0 {
        format!("{:.2} GB", saved_mb / 1024.0)
    } else {
        format!("{:.0} MB", saved_mb)
    };

    let total_duration_str = duration_to_str(total_duration_secs as u64);
    let avg_file_duration = if total_files > 0 {
        duration_to_str((total_duration_secs / total_files as f64) as u64)
    } else { "—".to_string() };

    let payload = serde_json::json!({
        "username": "RenCodeX",
        "embeds": [{
            "title": "📊 Statistiques d'encodage",
            "color": 0x9B59B6,
            "thumbnail": { "url": thumbnail_for(0x9B59B6) },
            "fields": [
                { "name": "📁 Fichiers encodés", "value": format!("`{}`", total_files), "inline": true },
                { "name": "💾 Espace libéré", "value": format!("`{}` (`{:.1}%`)", saved_str, gain_pct), "inline": true },
                { "name": "⏱️ Temps total", "value": format!("`{}`", total_duration_str), "inline": true },
                { "name": "⚡ Temps moyen/fichier", "value": format!("`{}`", avg_file_duration), "inline": true },
            ],
            "footer": { "text": "RenCodeX · Période" },
            "timestamp": chrono::Utc::now().to_rfc3339()
        }]
    });

    send_discord_message(token, channel_id, &payload).await;
}

pub async fn discord_notify_progress(
    token: &str,
    channel_id: &str,
    file_name: &str,
    file_index: usize,
    file_total: usize,
    percent: f64,
    speed: f64,
    remaining_secs: f64,
    _elapsed_secs: f64,
) {
    if token.is_empty() || channel_id.is_empty() { return; }

    let filled = (percent / 100.0 * 15.0) as usize;
    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(15usize.saturating_sub(filled)));

    let remaining_str = duration_to_str(remaining_secs as u64);
    let ts = unix_timestamp();

    let payload = serde_json::json!({
        "username": "RenCodeX",
        "embeds": [{
            "title": "📈 Encodage en cours",
            "color": 0x3498DB,
            "thumbnail": { "url": thumbnail_for(0x3498DB) },
            "fields": [
                { "name": "📁 Fichier", "value": format!("`{}` ({}/{})", file_name, file_index + 1, file_total), "inline": false },
                { "name": "📊 Progression", "value": format!("{} **{:.1}%**", bar, percent), "inline": false },
                { "name": "⚡ Vitesse", "value": format!("`{:.2}x`", speed), "inline": true },
                { "name": "⏱️ Restant", "value": format!("`{}`", remaining_str), "inline": true },
                { "name": "⌛ Démarré", "value": format!("<t:{}:R>", ts), "inline": true },
            ],
            "footer": { "text": "RenCodeX · Mise à jour progression" },
            "timestamp": chrono::Utc::now().to_rfc3339()
        }]
    });

    send_discord_message(token, channel_id, &payload).await;
}

pub async fn send_email_report(
    summary: EncodeSummary,
    email_cfg: EmailConfig,
) -> Result<(), String> {
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{Message, SmtpTransport, Transport};

    let total = summary.files.len();
    let success = summary.files.iter().filter(|f| f.status == "ok").count();
    let gain_pct = if summary.total_original_mb > 0.0 {
        100.0 * (summary.total_original_mb - summary.total_encoded_mb)
            / summary.total_original_mb
    } else { 0.0 };

    let mut body = format!(
        "Encodage terminé ✅\n\nRÉSUMÉ:\n\
         - Fichiers traités : {total}\n\
         - Réussis          : {success}\n\
         - Taille avant     : {:.2} MB\n\
         - Taille après     : {:.2} MB\n\
         - Gain             : {gain_pct:.1}%\n\n",
        summary.total_original_mb, summary.total_encoded_mb,
    );
    for f in &summary.files {
        body.push_str(&format!(
            "{} - {} ({:.1}% gain)\n", f.name, f.status,
            if f.original_mb > 0.0 {
                100.0 * (f.original_mb - f.encoded_mb) / f.original_mb
            } else { 0.0 }
        ));
    }

    let email = Message::builder()
        .from(email_cfg.username.parse()
            .map_err(|e: lettre::address::AddressError| e.to_string())?)
        .to(email_cfg.to.parse()
            .map_err(|e: lettre::address::AddressError| e.to_string())?)
        .subject("📦 RenCodeX - Encodage terminé")
        .body(body)
        .map_err(|e| e.to_string())?;

    let creds = Credentials::new(email_cfg.username, email_cfg.password);
    let mailer = SmtpTransport::relay(&email_cfg.smtp_host)
        .map_err(|e| e.to_string())?
        .credentials(creds)
        .port(email_cfg.smtp_port)
        .build();

    mailer.send(&email).map_err(|e| e.to_string())?;
    Ok(())
}

pub mod discord_bot {
    use std::os::windows::process::CommandExt;
    use crate::state::lock_encoder;
    use serenity::async_trait;
    use serenity::model::channel::Message;
    use serenity::model::gateway::Ready;
    use serenity::prelude::*;

    struct Handler {
        channel_id: u64,
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
            let reply = match content.as_str() {
                "!help" => format_help(),
                "!status" => format_status(),
                "!queue" => format_queue(),
                "!skip" => {
                    let (encoding, pid_opt) = {
                        let s = lock_encoder();
                        (s.encoding, s.child_pid)
                    };
                    if encoding {
                        if let Some(pid) = pid_opt {
                            lock_encoder().cancel = true;
                            let _ = std::process::Command::new("taskkill")
                                .args(["/PID", &pid.to_string(), "/F", "/T"])
                                .creation_flags(0x08000000)
                                .spawn();
                            "⏭ Fichier actuel ignoré, passage au suivant.".to_string()
                        } else {
                            "ℹ️ Aucun processus en cours à interrompre.".to_string()
                        }
                    } else {
                        "ℹ️ Aucun encodage en cours.".to_string()
                    }
                }
                "!cancel" => {
                    let (encoding, pid_opt) = {
                        let s = lock_encoder();
                        (s.encoding, s.child_pid)
                    };
                    if encoding {
                        lock_encoder().cancel = true;
                        if let Some(pid) = pid_opt {
                            let _ = std::process::Command::new("taskkill")
                                .args(["/PID", &pid.to_string(), "/F", "/T"])
                                .creation_flags(0x08000000)
                                .spawn();
                        }
                        "⏹ Encodage annulé depuis Discord.".to_string()
                    } else {
                        "ℹ️ Aucun encodage en cours.".to_string()
                    }
                }
                "!pause" => {
                    let encoding = lock_encoder().encoding;
                    if encoding {
                        lock_encoder().pause = true;
                        "⏸ Pause demandée (prendra effet au prochain fichier).".to_string()
                    } else {
                        "ℹ️ Aucun encodage en cours.".to_string()
                    }
                }
                "!resume" => {
                    lock_encoder().pause = false;
                    "▶️ Reprise demandée.".to_string()
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
        let s = lock_encoder();
        if !s.encoding {
            return "💤 Aucun encodage en cours.".to_string();
        }
        let filled = (s.percent / 100.0 * 20.0) as usize;
        let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(20 - filled));
        let elapsed = s.start_time.map(|t| t.elapsed().as_secs()).unwrap_or(0);
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
        let s = lock_encoder();
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

    pub async fn start(token: String, channel_id: u64) {
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;

        let mut client = match Client::builder(&token, intents)
            .event_handler(Handler { channel_id })
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
}
