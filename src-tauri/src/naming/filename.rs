//! Nettoyage et analyse de noms de fichiers vidéo.
//!
//! Le pipeline de `clean_filename` suit cet ordre :
//!   1. Détections précoces (année, résolution, source, HDR, édition, provider)
//!      sur le nom brut — avant tout remplacement.
//!   2. Nettoyage du titre : points/tirets bas → espaces, crochets, groupes
//!      de release, sigles techniques.
//!   3. Détection saison/épisode.
//!   4. Construction du nom suggéré.

use crate::models::CleanedName;
use crate::naming::regex;
use crate::utils::{normalize_lang, normalize_resolution, format_title_case, sanitize_filename};

// ── Détection de source ───────────────────────────────────────────────────

/// Détecte la meilleure source vidéo dans `s` par ordre de priorité décroissant.
/// Le score évite qu'une source générique (WEB) écrase une source précise (WEB-DL).
fn detect_source(s: &str) -> String {
    const SOURCE_MAP: &[(&once_cell::sync::Lazy<regex::Regex>, &str, i32)] = &[
        (&regex::SRC_WEBDL,  "WEB-DL",  10),
        (&regex::SRC_WEBRIP, "WEBRip",   9),
        (&regex::SRC_BLURAY, "BluRay",  10),
        (&regex::SRC_BDRIP,  "BDRip",    8),
        (&regex::SRC_BRRIP,  "BRRip",    7),
        (&regex::SRC_HDTV,   "HDTV",     6),
        (&regex::SRC_DVDRIP, "DVDRip",   5),
        (&regex::SRC_HDRIP,  "HDRip",    5),
        (&regex::SRC_BD,     "BluRay",   4),  // "BD" seul = alias BluRay
        (&regex::SRC_WEB,    "WEB",      4),  // Toujours en dernier
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

/// Supprime toutes les occurrences de source de `base`.
/// L'ordre est important : les patterns spécifiques d'abord,
/// SRC_WEB en dernier pour éviter de consommer "WEB" dans des titres.
fn strip_sources(base: &mut String) {
    for re in &[
        &regex::SRC_WEBDL, &regex::SRC_WEBRIP,
        &regex::SRC_BLURAY, &regex::SRC_BDRIP, &regex::SRC_BRRIP,
        &regex::SRC_HDTV, &regex::SRC_DVDRIP, &regex::SRC_HDRIP,
        &regex::SRC_BD,    // alias BluRay, avant SRC_WEB
        &regex::SRC_WEB,   // En dernier
    ] {
        *base = re.replace_all(base, " ").to_string();
    }
}

// ── Détection du tag audio/langue ────────────────────────────────────────

/// Calcule le tag audio (VF, VO, MULTI, VOSTFR…) à partir :
/// - des langues audio normalisées,
/// - des langues de sous-titres normalisées,
/// - de la présence du tag MULTI dans le nom original.
///
/// Règle VOSTFR : audio japonais + sous-titres français → VOSTFR
/// (et non pas uniquement d'après la piste audio).
fn compute_audio_tag(
    audio_langs: &[String],
    sub_langs: &[String],
    has_multi_tag: bool,
) -> String {
    let only_und = audio_langs.is_empty() || audio_langs.iter().all(|l| l == "und");
    if only_und {
        return String::new();
    }

    let has_multi_audio = has_multi_tag || audio_langs.len() > 1;
    let has_french_subs = sub_langs.iter().any(|l| l == "fre");
    let has_multi_subs  = sub_langs.len() > 1;

    // MULTI : plusieurs pistes audio OU tag MULTI dans le nom
    if has_multi_audio || has_multi_subs {
        return "MULTI".to_string();
    }

    // Piste unique
    match audio_langs.first().map(|s| s.as_str()) {
        Some("fre") => "VF".to_string(),
        Some("eng") => "VO".to_string(),

        // VOSTFR : audio JP + sous-titres FR
        Some("jpn") if has_french_subs => "VOSTFR".to_string(),
        // VOSTA (sous-titres anglais) : audio JP + sous-titres EN
        Some("jpn") if sub_langs.iter().any(|l| l == "eng") => "VOSTA".to_string(),
        // Audio JP sans sous-titres reconnus → tag neutre
        Some("jpn") => "VOSTFR".to_string(),

        Some("kor") if has_french_subs => "VOSTKR".to_string(),
        Some("kor") => "VO-KR".to_string(),

        Some("chi") if has_french_subs => "VOSTCH".to_string(),
        Some("chi") => "VO-ZH".to_string(),

        Some("ger") => "GER".to_string(),
        Some("spa") => "SPA".to_string(),
        Some("ita") => "ITA".to_string(),
        Some("por") => "POR".to_string(),
        Some("rus") => "RUS".to_string(),
        Some("ara") => "ARA".to_string(),
        Some("dut") => "NL".to_string(),
        Some("pol") => "POL".to_string(),
        _ => String::new(),
    }
}

// ── Fonction principale ───────────────────────────────────────────────────

/// Analyse un nom de fichier vidéo brut et retourne ses composantes structurées
/// ainsi qu'un nom suggéré normalisé.
///
/// # Arguments
/// * `raw`         – Nom de fichier brut (avec ou sans extension).
/// * `audio_langs` – Codes de langue des pistes audio (ISO 639-1 ou 639-2).
/// * `sub_langs`   – Codes de langue des pistes de sous-titres.
pub fn clean_filename(
    raw: String,
    audio_langs: Vec<String>,
    sub_langs: Vec<String>,
) -> CleanedName {
    // ── Normalisation des langues ────────────────────────────────────────
    let audio_langs: Vec<String> = audio_langs.iter().map(|l| normalize_lang(l)).collect();
    let sub_langs:   Vec<String> = sub_langs.iter().map(|l| normalize_lang(l)).collect();

    let has_multi_tag = regex::MULTI_TAG.is_match(&raw);

    // Extraire le nom sans extension
    let base_raw = std::path::Path::new(&raw)
        .file_stem()
        .and_then(|f| f.to_str())
        .unwrap_or(&raw)
        .to_string();

    // ── Phase 1 : Détections précoces (sur le nom brut) ─────────────────
    // On détecte avant tout remplacement pour ne pas rater des patterns
    // qui disparaîtraient lors du nettoyage.

    let provider = regex::PROVIDER.captures(&base_raw)
        .map(|c| c[1].to_uppercase())
        .unwrap_or_default();

    // L'année est dans le groupe 1 du nouveau pattern (le contexte avant n'est pas capturé)
    let year = regex::YEAR.captures(&base_raw)
        .map(|c| c[1].to_string())
        .unwrap_or_default();

    // Détection explicite année entre parenthèses (film) — priorité sur l'heuristique
    let year_in_paren = regex::YEAR_PAREN.is_match(&base_raw);

    let hdr = regex::HDR.captures(&base_raw)
        .map(|c| c[1].to_string())
        .unwrap_or_default();

    let edition = regex::EDITION.captures(&base_raw)
        .map(|c| c[1].to_string())
        .unwrap_or_default();

    let resolution_early = regex::RESOLUTION
        .captures(&base_raw)
        .map(|c| normalize_resolution(&c[1]))
        .or_else(|| {
            regex::RESOLUTION_NUM.captures(&base_raw).and_then(|c| {
                let h: u32 = c[2].parse().unwrap_or(0);
                match h {
                    2160 => Some("2160P".to_string()),
                    1080 => Some("1080P".to_string()),
                    720  => Some("720P".to_string()),
                    480  => Some("480P".to_string()),
                    _    => None,
                }
            })
        })
        .unwrap_or_default();

    let source_early = detect_source(&base_raw);

    // ── Phase 2 : Nettoyage du titre ────────────────────────────────────

    // 2a. Points et tirets bas → espaces
    let mut base = base_raw.replace(['.', '_'], " ");

    // 2b. Contenu entre crochets/parenthèses (groupes, tags, CRC…)
    //     On boucle car plusieurs groupes peuvent être imbriqués ou adjacents.
    for _ in 0..10 {
        let new = regex::BRACKETS.replace_all(&base, " ").to_string();
        if new == base { break; }
        base = new;
    }
    

    // 2c. Groupe de release en fin de titre (ex: " - SubsPlease")
    if let Some(cap) = regex::TRAIL_GROUP.captures(&base) {
        let matched = &cap[1];
        // Ne pas supprimer si c'est en fait un identifiant saison/épisode
        let is_se = regex::S_E.is_match(matched)
            || regex::S_E_MULTI.is_match(matched)
            || regex::E_LONG.is_match(matched);
        if !is_se {
            base = regex::TRAIL_GROUP.replace(&base, "").to_string();
        }
    }

    // 2d. Groupes connus — premier passage sur le nom brut nettoyé
    // (attrape les formes " - GroupName" avant suppression des tech tags)
    for re in regex::KNOWN_GROUPS.iter() {
        base = re.replace_all(&base, " ").to_string();
    }

    // ── Phase 3 : Résolution ────────────────────────────────────────────

    let resolution = regex::RESOLUTION
        .captures(&base)
        .map(|c| normalize_resolution(&c[1]))
        .or_else(|| {
            regex::RESOLUTION_NUM.captures(&base).and_then(|c| {
                let h: u32 = c[2].parse().unwrap_or(0);
                match h {
                    2160 => Some("2160P".to_string()),
                    1080 => Some("1080P".to_string()),
                    720  => Some("720P".to_string()),
                    480  => Some("480P".to_string()),
                    _    => None,
                }
            })
        })
        .unwrap_or(resolution_early);

    base = regex::RESOLUTION.replace_all(&base, " ").to_string();
    base = regex::RESOLUTION_NUM.replace_all(&base, " ").to_string();

    // ── Phase 4 : Année ──────────────────────────────────────────────────────
    // Suppression de l'année uniquement si c'est clairement un film.
    // Si ce qui suit l'année dans le nom brut contient des tags techniques,
    // c'est un millésime de série (ex: "HEROINES 2026 SUBFRENCH") — on garde.
    if !year.is_empty() {
        let after_year_raw = regex::YEAR.find(&base_raw)
            .map(|m| base_raw[m.end()..].trim())
            .unwrap_or("");
        let looks_technical = regex::RESOLUTION.is_match(after_year_raw)
            || regex::SRC_WEBDL.is_match(after_year_raw)
            || regex::SRC_WEBRIP.is_match(after_year_raw)
            || regex::SRC_BLURAY.is_match(after_year_raw)
            || regex::SRC_WEB.is_match(after_year_raw)
            || {
                let up = after_year_raw.to_uppercase();
                up.contains("SUBFRENCH") || up.contains("VOSTFR")
                || up.contains("FRENCH") || up.contains("MULTI")
            };
        if year_in_paren || !looks_technical {
            base = regex::YEAR_CTX.replace_all(&base, "$1 $3").to_string();
            base = regex::YEAR.replace_all(&base, " ").to_string();
        }
    }

    // ── Phase 5 : Saison / Épisode ──────────────────────────────────────

    let mut season_episode = String::new();

    // 5a. Multi-épisodes (S01E01E02)
    if let Some(cap) = regex::S_E_MULTI.captures(&base) {
        let s:  u32 = cap[1].parse().unwrap_or(1);
        let e1: u32 = cap[2].parse().unwrap_or(1);
        let e2: u32 = cap[3].parse().unwrap_or(1);
        if s <= 50 && e1 <= 9999 && e2 <= 9999 {
            season_episode = format!("S{:02}E{:02}-E{:02}", s, e1, e2);
            let m = cap.get(0).unwrap();
            base = base[..m.start()].to_string();
        }
    }

    // 5b. Format S01E01 et variantes
    if season_episode.is_empty() {
        let season_pats: &[(&once_cell::sync::Lazy<regex::Regex>, bool)] = &[
            (&regex::S_DASH_EP, true),  // "S3 - 01" / "S3 – 01" (anime fansub)
            (&regex::S_E,      true),
            (&regex::NUM_X,    true),
            (&regex::SAISON_EP, true),
            (&regex::E_LONG,   false),
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
                    let m = cap.get(0).unwrap();
                    base = base[..m.start()].to_string();
                    break;
                }
            }
        }
    }

    base = regex::TRAIL_DASH.replace(&base, "").to_string();

    // 5c. Numéro d'épisode trailing isolé (ex: "Show - 12")
    if season_episode.is_empty() {
        let has_ova_context = regex::OVA_CONTEXT.is_match(&base);
        if !has_ova_context {
            if let Some(cap) = regex::EP_TRAIL.captures(&base) {
                let num: u32 = cap[1].parse().unwrap_or(0);
                // Exclure les années et les 0
                if !(1900..=2099).contains(&num) && num >= 1 && num <= 999 {
                    season_episode = format!("E{:02}", num);
                    base = regex::EP_TRAIL.replace(&base, " ").to_string();
                }
            }
        }
    }

    // 5d. OVA / Spécial / ONA / OAD
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

    // ── Phase 6 : Source ────────────────────────────────────────────────
    // Re-détecter sur la base nettoyée ; sinon garder la détection précoce.
    let source = {
        let s = detect_source(&base);
        if s.is_empty() { source_early } else { s }
    };
    strip_sources(&mut base);

    // ── Phase 7 : Suppression des tags techniques résiduels ──────────────

    base = regex::HDR.replace_all(&base, " ").to_string();
    base = regex::EDITION.replace_all(&base, " ").to_string();

    for r in regex::TECH_TAGS.iter() {
        base = r.replace_all(&base, " ").to_string();
    }

    base = regex::AUDIO_CODEC_NUM.replace_all(&base, " ").to_string();
    base = regex::AUDIO_DD.replace_all(&base, " ").to_string();
    base = regex::AUDIO_DTS.replace_all(&base, " ").to_string();

    base = regex::H264_SPACE.replace_all(&base, " ").to_string();
    base = regex::H265_SPACE.replace_all(&base, " ").to_string();

    base = regex::PROVIDER.replace_all(&base, " ").to_string();

    // Second passage KNOWN_GROUPS : après suppression des tech tags,
    // les formes collées type "x264-Tsundere-Raws" sont maintenant accessibles.
    for re in regex::KNOWN_GROUPS.iter() {
        base = re.replace_all(&base, " ").to_string();
    }
    // TRAIL_GROUP second passage : cas résiduels après nettoyage technique
    if let Some(cap) = regex::TRAIL_GROUP.captures(&base) {
        let matched = &cap[1];
        let is_se = regex::S_E.is_match(matched)
            || regex::S_E_MULTI.is_match(matched)
            || regex::E_LONG.is_match(matched);
        if !is_se {
            base = regex::TRAIL_GROUP.replace(&base, "").to_string();
        }
    }
    // TRAIL_GROUP_STUCK : groupe collé après un tag (ex: résidu "x264-Tsundere-Raws")
    base = regex::TRAIL_GROUP_STUCK.replace(&base, "").to_string();

    // ── Phase 8 : Nettoyage final du titre ───────────────────────────────

    base = regex::TRAIL_DASH.replace(&base, "").to_string();
    base = regex::MULTI_SP.replace_all(&base, " ").to_string();
    let base = format_title_case(base.trim());

    // ── Phase 9 : Tag audio / langue ─────────────────────────────────────

    let audio_tag = compute_audio_tag(&audio_langs, &sub_langs, has_multi_tag);

    // ── Phase 10 : Construction du nom suggéré ───────────────────────────

    // Un fichier est considéré comme un film si :
    // - pas de saison/épisode détecté
    // - l'année est entre parenthèses dans le nom original (ex: "Show (2026)")
    //   OU l'année apparaît seule juste après le titre (pas entourée de tags techniques)
    // Heuristique : on vérifie que l'année est dans YEAR_PAREN ou
    // que le nom original contient "Titre Année " (pas "Titre Année SUBFRENCH").
    let is_movie = season_episode.is_empty() && !year.is_empty() && {
        if year_in_paren {
            true
        } else {
            // L'année est suivie de tags techniques → c'est une série/anime avec millésime
            let after_year = regex::YEAR.find(&base_raw)
                .map(|m| base_raw[m.end()..].trim())
                .unwrap_or("");
            // Si ce qui suit l'année ressemble à des tags (résolution, source, codec…)
            let looks_technical = regex::RESOLUTION.is_match(after_year)
                || regex::SRC_WEBDL.is_match(after_year)
                || regex::SRC_WEBRIP.is_match(after_year)
                || regex::SRC_BLURAY.is_match(after_year)
                || regex::SRC_WEB.is_match(after_year)
                || after_year.to_uppercase().contains("SUBFRENCH")
                || after_year.to_uppercase().contains("VOSTFR")
                || after_year.to_uppercase().contains("FRENCH")
                || after_year.to_uppercase().contains("MULTI");
            !looks_technical
        }
    };

    let mut parts: Vec<String> = Vec::new();
    if !base.is_empty()  { parts.push(base.clone()); }
    if is_movie {
        if !year.is_empty() { parts.push(format!("({})", year)); }
    } else if !season_episode.is_empty() {
        parts.push(season_episode.clone());
    }
    if !audio_tag.is_empty()  { parts.push(audio_tag.clone()); }
    if !resolution.is_empty() { parts.push(resolution.clone()); }
    if !hdr.is_empty()        { parts.push(hdr.clone()); }
    if !provider.is_empty()   { parts.push(provider.clone()); }
    if !source.is_empty()     { parts.push(source.clone()); }
    if !edition.is_empty()    { parts.push(edition.clone()); }
    parts.push("H265".to_string());
    parts.push("10bit".to_string());
    parts.push("AAC".to_string());

    let suggested_raw = regex::MULTI_SP
        .replace_all(&parts.join(" "), " ")
        .trim()
        .to_string();

    // Sanitize le nom final pour le système de fichiers Windows
    let suggested = sanitize_filename(&suggested_raw);

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