//! Nettoyage et analyse de noms de fichiers

use crate::models::CleanedName;
use crate::regex;
use crate::utils::{normalize_lang, normalize_resolution, format_title_case};

fn detect_source(s: &str) -> String {
    const SOURCE_MAP: &[(&once_cell::sync::Lazy<regex::Regex>, &str, i32)] = &[
        (&regex::SRC_WEBDL, "WEB-DL", 10),
        (&regex::SRC_WEBRIP, "WEBRip", 9),
        (&regex::SRC_BLURAY, "BluRay", 10),
        (&regex::SRC_BDRIP, "BDRip", 8),
        (&regex::SRC_BRRIP, "BRRip", 7),
        (&regex::SRC_HDTV, "HDTV", 6),
        (&regex::SRC_DVDRIP, "DVDRip", 5),
        (&regex::SRC_HDRIP, "HDRip", 5),
        (&regex::SRC_WEB, "WEB", 4),
    ];
    let mut best = String::new();
    let mut best_score = 0i32;
    for (re, label, score) in SOURCE_MAP {
        if re.is_match(s) && *score > best_score {
            best = label.to_string();
            best_score = *score;
        }
    }
    best
}

fn strip_sources(base: &mut String) {
    for re in &[
        &regex::SRC_WEBDL, &regex::SRC_WEBRIP, &regex::SRC_BLURAY,
        &regex::SRC_BDRIP, &regex::SRC_BRRIP, &regex::SRC_HDTV,
        &regex::SRC_DVDRIP, &regex::SRC_HDRIP, &regex::SRC_WEB,
    ] {
        *base = re.replace_all(base, " ").to_string();
    }
}

pub fn clean_filename(
    raw: String,
    audio_langs: Vec<String>,
    sub_langs: Vec<String>,
) -> CleanedName {
    let audio_langs: Vec<String> = audio_langs.iter().map(|l| normalize_lang(l)).collect();
    let sub_langs: Vec<String> = sub_langs.iter().map(|l| normalize_lang(l)).collect();

    let has_multi_tag = regex::MULTI_TAG.is_match(&raw);

    let base_raw = std::path::Path::new(&raw)
        .file_stem()
        .and_then(|f| f.to_str())
        .unwrap_or(&raw)
        .to_string();

    // Détections précoces
    let provider = regex::PROVIDER.captures(&base_raw)
        .map(|c| c[1].to_uppercase())
        .unwrap_or_default();

    let year = regex::YEAR.captures(&base_raw)
        .map(|c| c[1].to_string())
        .unwrap_or_default();

    let hdr = regex::HDR.captures(&base_raw)
        .map(|c| c[1].to_string())
        .unwrap_or_default();

    let edition = regex::EDITION.captures(&base_raw)
        .map(|c| c[1].to_string())
        .unwrap_or_default();

    let resolution_early = regex::RESOLUTION
        .captures(&base_raw)
        .map(|c| normalize_resolution(&c[1]))
        .unwrap_or_default();

    let source_early = detect_source(&base_raw);

    let mut base = base_raw.replace(['.', '_'], " ");

    // Suppression des crochets/parenthèses - Version simplifiée sans fermeture
    // On supprime d'abord tous les crochets/parenthèses
    base = regex::BRACKETS.replace_all(&base, " ").to_string();

    // Suppression groupe de release en fin - Version simplifiée
    // On vérifie si le pattern correspond et on supprime
    if let Some(cap) = regex::TRAIL_GROUP.captures(&base) {
        let matched = &cap[1];
        let is_se = regex::S_E.is_match(matched)
            || regex::S_E_MULTI.is_match(matched)
            || regex::E_LONG.is_match(matched);
        if !is_se {
            base = regex::TRAIL_GROUP.replace(&base, "").to_string();
        }
    }

    // Groupes connus
    for re in regex::KNOWN_GROUPS.iter() {
        base = re.replace_all(&base, " ").to_string();
    }

    // Résolution
    let resolution = regex::RESOLUTION
        .captures(&base)
        .map(|c| normalize_resolution(&c[1]))
        .or_else(|| {
            regex::RESOLUTION_NUM.captures(&base).and_then(|c| {
                let h: u32 = c[2].parse().unwrap_or(0);
                match h {
                    2160 => Some("2160P".to_string()),
                    1080 => Some("1080P".to_string()),
                    720 => Some("720P".to_string()),
                    480 => Some("480P".to_string()),
                    _ => None,
                }
            })
        })
        .unwrap_or(resolution_early);

    base = regex::RESOLUTION.replace_all(&base, " ").to_string();
    base = regex::RESOLUTION_NUM.replace_all(&base, " ").to_string();

    // Suppression année
    if !year.is_empty() {
        base = regex::YEAR.replace_all(&base, " ").to_string();
    }

    // Détection saison/épisode
    let mut season_episode = String::new();

    if let Some(cap) = regex::S_E_MULTI.captures(&base) {
        let s: u32 = cap[1].parse().unwrap_or(1);
        let e1: u32 = cap[2].parse().unwrap_or(1);
        let e2: u32 = cap[3].parse().unwrap_or(1);
        if s <= 50 && e1 <= 9999 && e2 <= 9999 {
            season_episode = format!("S{:02}E{:02}-E{:02}", s, e1, e2);
            base = regex::S_E_MULTI.replace(&base, " ").to_string();
        }
    }

    if season_episode.is_empty() {
        let season_pats: &[(&once_cell::sync::Lazy<regex::Regex>, bool)] = &[
            (&regex::S_E, true),
            (&regex::NUM_X, true),
            (&regex::SAISON_EP, true),
            (&regex::E_LONG, false),
        ];
        for (r, two_groups) in season_pats {
            if let Some(cap) = r.captures(&base) {
                let se = if *two_groups {
                    let s: u32 = cap[1].parse().unwrap_or(1);
                    let e: u32 = cap[2].parse().unwrap_or(1);
                    if s <= 50 && e <= 9999 {
                        format!("S{:02}E{:02}", s, e)
                    } else { String::new() }
                } else {
                    let e: u32 = cap[1].parse().unwrap_or(1);
                    if e <= 9999 { format!("E{:02}", e) } else { String::new() }
                };
                if !se.is_empty() {
                    season_episode = se;
                    base = r.replace(&base, " ").to_string();
                    break;
                }
            }
        }
    }
    base = regex::TRAIL_DASH.replace(&base, "").to_string();

    if season_episode.is_empty() {
        let has_ova_context = regex::OVA_CONTEXT.is_match(&base);
        if !has_ova_context {
            if let Some(cap) = regex::EP_TRAIL.captures(&base) {
                let num: u32 = cap[1].parse().unwrap_or(0);
                if !(1900..=2099).contains(&num) && num >= 1 && num <= 999 {
                    season_episode = format!("E{:02}", num);
                    base = regex::EP_TRAIL.replace(&base, " ").to_string();
                }
            }
        }
    }

    if season_episode.is_empty() {
        if let Some(cap) = regex::OVA_SPECIAL.captures(&base) {
            let type_tag = cap.get(1)
                .map(|m| m.as_str().to_uppercase())
                .unwrap_or_else(|| "SPECIAL".to_string());

            let num_str = cap.get(2)
                .or_else(|| cap.get(3))
                .map(|m| m.as_str().trim())
                .unwrap_or("")
                .to_string();

            season_episode = if num_str.is_empty() {
                type_tag
            } else if let Ok(n) = num_str.parse::<u32>() {
                format!("{} {:02}", type_tag, n)
            } else {
                type_tag
            };
            base = regex::OVA_SPECIAL.replace(&base, " ").to_string();
        }
    }

    // Source
    let source = {
        let s = detect_source(&base);
        if s.is_empty() { source_early } else { s }
    };
    strip_sources(&mut base);

    // Tags HDR, édition, techniques
    base = regex::HDR.replace_all(&base, " ").to_string();
    base = regex::EDITION.replace_all(&base, " ").to_string();

    for r in regex::TECH_TAGS.iter() {
        base = r.replace_all(&base, " ").to_string();
    }
    base = regex::TRAIL_DASH.replace(&base, "").to_string();

    // Codecs audio
    base = regex::AUDIO_CODEC_NUM.replace_all(&base, " ").to_string();
    base = regex::AUDIO_DD.replace_all(&base, " ").to_string();
    base = regex::AUDIO_DTS.replace_all(&base, " ").to_string();

    // H.264/H.265
    base = regex::H264_SPACE.replace_all(&base, " ").to_string();
    base = regex::H265_SPACE.replace_all(&base, " ").to_string();

    // Provider
    base = regex::PROVIDER.replace_all(&base, " ").to_string();

    // Nettoyage final
    base = regex::TRAIL_DASH.replace(&base, "").to_string();
    base = regex::MULTI_SP.replace_all(&base, " ").to_string();

    let base = format_title_case(&base.trim().to_string());

    // Tag audio/langue
    let only_und = audio_langs.is_empty() || audio_langs.iter().all(|l| l == "und");
    let has_multi_subs = sub_langs.len() > 1;

    let audio_tag = if only_und {
        String::new()
    } else if has_multi_tag || audio_langs.len() > 1 || has_multi_subs {
        "MULTI".to_string()
    } else {
        match audio_langs.first().map(|s| s.as_str()) {
            Some("fre") => "VF".to_string(),
            Some("eng") => "VO".to_string(),
            Some("jpn") => "VOSTFR".to_string(),
            Some("kor") => "VOSTKR".to_string(),
            Some("chi") => "VOSTCH".to_string(),
            Some("ger") => "GER".to_string(),
            Some("spa") => "SPA".to_string(),
            Some("ita") => "ITA".to_string(),
            Some("por") => "POR".to_string(),
            Some("rus") => "RUS".to_string(),
            _ => String::new(),
        }
    };

    // Nom suggéré
    let is_movie = season_episode.is_empty() && !year.is_empty();

    let mut parts: Vec<String> = Vec::new();
    if !base.is_empty() { parts.push(base.clone()); }
    if is_movie {
        if !year.is_empty() { parts.push(format!("({})", year)); }
    } else {
        if !season_episode.is_empty() { parts.push(season_episode.clone()); }
    }
    if !audio_tag.is_empty() { parts.push(audio_tag.clone()); }
    if !resolution.is_empty() { parts.push(resolution.clone()); }
    if !hdr.is_empty() { parts.push(hdr.clone()); }
    if !provider.is_empty() { parts.push(provider.clone()); }
    if !source.is_empty() { parts.push(source.clone()); }
    if !edition.is_empty() { parts.push(edition.clone()); }
    parts.push("H265".to_string());
    parts.push("10bit".to_string());
    parts.push("AAC".to_string());

    let suggested = regex::MULTI_SP
        .replace_all(&parts.join(" "), " ")
        .trim()
        .to_string();

    CleanedName {
        title: base,
        year,
        season_episode,
        resolution,
        source,
        provider,
        hdr,
        edition,
        audio_tags: audio_tag,
        suggested,
    }
}