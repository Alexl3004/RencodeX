//! Utilitaires généraux

use std::path::{Path, PathBuf};

const SMALL_WORDS: &[&str] = &[
    "and", "or", "of", "the", "a", "an", "in", "on", "at",
    "to", "for", "by", "with", "from", "but", "nor", "yet",
    "so", "as", "if", "vs",
    "et", "de", "du", "le", "la", "les", "un", "une",
    "en", "au", "aux", "sur", "par", "ou", "ni",
];

#[allow(dead_code)]
pub fn filename_of(path: &str) -> String {
    Path::new(path).file_name()
        .and_then(|f| f.to_str())
        .unwrap_or(path)
        .to_string()
}

#[allow(dead_code)]
pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("RenCodeX")
        .join("config.json")
}

#[allow(dead_code)]
pub fn delete_partial_output(path: &str) -> bool {
    let p = Path::new(path);
    if p.exists() { std::fs::remove_file(p).is_ok() } else { false }
}

#[allow(dead_code)]
pub fn normalize_lang(raw: &str) -> String {
    let l = raw.to_lowercase();
    if l.starts_with("fr") { return "fre".to_string(); }
    if l.starts_with("en") { return "eng".to_string(); }
    if l.starts_with("ja") { return "jpn".to_string(); }
    if l.starts_with("de") { return "ger".to_string(); }
    if l.starts_with("es") { return "spa".to_string(); }
    if l.starts_with("ko") || l.starts_with("kr") { return "kor".to_string(); }
    if l.starts_with("it") { return "ita".to_string(); }
    if l.starts_with("pt") { return "por".to_string(); }
    if l.starts_with("ru") { return "rus".to_string(); }
    if l.starts_with("zh") { return "chi".to_string(); }
    if l.is_empty() || l == "und" { return "und".to_string(); }
    l.chars().take(3).collect()
}

#[allow(dead_code)]
pub fn lang_display_name(code: &str) -> &'static str {
    match code {
        "fre" => "Français",  "eng" => "Anglais",
        "jpn" => "Japonais",  "ger" => "Allemand",
        "spa" => "Espagnol",  "kor" => "Coréen",
        "ita" => "Italien",   "por" => "Portugais",
        "rus" => "Russe",     "chi" => "Chinois",
        _     => "Inconnu",
    }
}

#[allow(dead_code)]
pub fn normalize_resolution(r: &str) -> String {
    match r.to_uppercase().as_str() {
        "2160P" | "4K" | "UHD" => "2160P".to_string(),
        "1080P" | "FHD"        => "1080P".to_string(),
        "720P"  | "HD"         => "720P".to_string(),
        "480P"  | "SD"         => "480P".to_string(),
        other                  => other.to_string(),
    }
}

#[allow(dead_code)]
pub fn ffmpeg_path() -> PathBuf {
    if let Ok(p) = std::env::var("RENCODEX_FFMPEG_PATH") {
        let pb = PathBuf::from(&p);
        if pb.exists() { return pb; }
    }
    let default = PathBuf::from(r"C:\Outil\ffmpeg\bin\ffmpeg.exe");
    if default.exists() { return default; }
    PathBuf::from("ffmpeg")
}

#[allow(dead_code)]
pub fn ffprobe_path() -> PathBuf {
    let ffmpeg = ffmpeg_path();
    let probe = ffmpeg.with_file_name("ffprobe.exe");
    if probe.exists() { return probe; }
    PathBuf::from("ffprobe")
}

#[allow(dead_code)]
pub fn resolve_config(mut cfg: crate::models::AppConfig) -> crate::models::AppConfig {
    if let Ok(v) = std::env::var("RENCODEX_FFMPEG_PATH") {
        if !v.is_empty() { cfg.ffmpeg_path = v; }
    }
    if let Ok(v) = std::env::var("RENCODEX_DISCORD_TOKEN") {
        if !v.is_empty() {
            cfg.discord_bot_token = v;
            cfg.discord_enabled = true;
        }
    }
    if let Ok(v) = std::env::var("RENCODEX_DISCORD_LOG_CHANNEL") {
        if !v.is_empty() { cfg.discord_log_channel_id = v; }
    }
    if let Ok(v) = std::env::var("RENCODEX_DISCORD_CMD_CHANNEL") {
        if !v.is_empty() { cfg.discord_cmd_channel_id = v; }
    }
    cfg
}

#[allow(dead_code)]
pub fn format_title_case(base: &str) -> String {
    base.split_whitespace()
        .enumerate()
        .map(|(i, word)| {
            let lower = word.to_lowercase();
            let alpha: String = word.chars().filter(|c| c.is_alphabetic()).collect();

            if alpha.len() >= 2 && alpha.chars().all(|c| c.is_uppercase()) {
                return word.to_string();
            }
            if i > 0 && SMALL_WORDS.contains(&lower.as_str()) {
                return lower;
            }
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase()
                    .chain(chars.flat_map(|c| c.to_lowercase()))
                    .collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}