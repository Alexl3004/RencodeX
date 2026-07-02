//! Utilitaires généraux

use std::path::{Path, PathBuf};

/// Mots courts qui ne prennent pas de majuscule en position non-initiale.
/// Inclut l'anglais et le français.
const SMALL_WORDS: &[&str] = &[
    // Anglais
    "and", "or", "of", "the", "a", "an", "in", "on", "at",
    "to", "for", "by", "with", "from", "but", "nor", "yet",
    "so", "as", "if", "vs",
    // Français
    "et", "de", "du", "le", "la", "les", "un", "une",
    "en", "au", "aux", "sur", "par", "ou", "ni", "des",
];

/// Noms de fichiers réservés sous Windows (à éviter comme nom de sortie).
const WINDOWS_RESERVED: &[&str] = &[
    "CON", "PRN", "AUX", "NUL",
    "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
    "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
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

pub fn stats_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("RenCodeX")
        .join("stats.json")
}



#[allow(dead_code)]
pub fn delete_partial_output(path: &str) -> bool {
    let p = Path::new(path);
    if p.exists() { std::fs::remove_file(p).is_ok() } else { false }
}

/// Normalise un code de langue brut (ISO 639-1 ou 639-2) vers un code
/// interne à 3 lettres. Retourne `"und"` pour les langues inconnues.
#[allow(dead_code)]
pub fn normalize_lang(raw: &str) -> String {
    let l = raw.to_lowercase();
    if l.starts_with("fr") || l == "fre" { return "fre".to_string(); }
    if l.starts_with("en") || l == "eng" { return "eng".to_string(); }
    if l.starts_with("ja") || l == "jpn" { return "jpn".to_string(); }
    if l.starts_with("de") || l == "ger" { return "ger".to_string(); }
    if l.starts_with("es") || l == "spa" { return "spa".to_string(); }
    if l.starts_with("ko") || l.starts_with("kr") || l == "kor" { return "kor".to_string(); }
    if l.starts_with("it") || l == "ita" { return "ita".to_string(); }
    if l.starts_with("pt") || l == "por" { return "por".to_string(); }
    if l.starts_with("ru") || l == "rus" { return "rus".to_string(); }
    if l.starts_with("zh") || l == "chi" { return "chi".to_string(); }
    if l.starts_with("ar") || l == "ara" { return "ara".to_string(); }
    if l.starts_with("nl") || l == "dut" { return "dut".to_string(); }
    if l.starts_with("pl") || l == "pol" { return "pol".to_string(); }
    if l.is_empty() || l == "und" { return "und".to_string(); }
    // Retourne les 3 premiers caractères pour les codes inconnus
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
        "ara" => "Arabe",     "dut" => "Néerlandais",
        "pol" => "Polonais",
        _     => "Inconnu",
    }
}

/// Normalise les alias courants de résolution vers un format canonique
/// (toujours en majuscules, ex: "1080P").
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

/// Applique un Title Case intelligent :
/// - Les sigles (≥2 lettres toutes en majuscules) sont conservés tels quels.
/// - Les petits mots (articles, conjonctions) restent en minuscule, sauf
///   en première position ou après une apostrophe.
/// - Les apostrophes typographiques sont normalisées vers `'` ASCII.
/// - Les tirets composés (`word-word`) : chaque segment est capitalisé.
#[allow(dead_code)]
pub fn format_title_case(base: &str) -> String {
    // Normaliser les apostrophes avant de traiter
    let normalized = crate::regex::APOSTROPHE.replace_all(base, "'");

    normalized
        .split_whitespace()
        .enumerate()
        .map(|(i, word)| capitalize_word(word, i == 0))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Capitalise un mot unique, en gérant les tirets internes et les sigles.
fn capitalize_word(word: &str, is_first: bool) -> String {
    // Mot avec tiret interne (ex: "Blu-Ray", "Spider-Man") :
    // chaque segment est traité indépendamment.
    if word.contains('-') {
        return word
            .split('-')
            .enumerate()
            .map(|(j, seg)| capitalize_segment(seg, is_first || j == 0))
            .collect::<Vec<_>>()
            .join("-");
    }
    capitalize_segment(word, is_first)
}

/// Capitalise un segment simple (sans tiret).
fn capitalize_segment(segment: &str, is_first: bool) -> String {
    if segment.is_empty() { return String::new(); }

    let lower = segment.to_lowercase();

    // Sigle / tag technique : toutes les lettres du token sont en majuscules.
    // Exemples préservés : "VF", "NF", "AAC", "1080P", "H265", "HDR10".
    // On extrait uniquement les lettres pour tester (les chiffres sont ignorés).
    let alpha: String = segment.chars().filter(|c| c.is_alphabetic()).collect();
    if !alpha.is_empty() && alpha.chars().all(|c| c.is_uppercase()) {
        return segment.to_string();
    }

    // Petit mot en position non-initiale → minuscule
    if !is_first && SMALL_WORDS.contains(&lower.as_str()) {
        return lower;
    }

    // Cas général : première lettre en majuscule, reste en minuscule
    let mut chars = segment.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase()
            .chain(chars.flat_map(|c| c.to_lowercase()))
            .collect(),
    }
}

/// Vérifie si un nom de fichier (sans extension) est un nom réservé Windows.
#[allow(dead_code)]
pub fn is_windows_reserved(name: &str) -> bool {
    let upper = name.to_uppercase();
    // Enlever l'extension éventuelle pour la comparaison
    let stem = upper.split('.').next().unwrap_or(&upper);
    WINDOWS_RESERVED.contains(&stem)
}

/// Sanitize un nom de fichier : supprime les caractères interdits sous Windows,
/// tronque à 200 caractères, et rejette les noms réservés.
/// Retourne le nom nettoyé, ou `"output"` en dernier recours.
#[allow(dead_code)]
pub fn sanitize_filename(name: &str) -> String {
    // Supprimer les caractères interdits
    let clean = crate::regex::INVALID_CHARS.replace_all(name, "").to_string();
    // Supprimer les points et espaces en fin de nom (interdit sous Windows)
    let clean = clean.trim_end_matches(['.', ' ']).to_string();
    // Tronquer à 200 caractères (garde de la marge pour extension + suffixe)
    let clean: String = clean.chars().take(200).collect();
    // Vérifier les noms réservés
    if clean.is_empty() || is_windows_reserved(&clean) {
        return "output".to_string();
    }
    clean
}