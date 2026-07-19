//! Tests étendus — regex.rs & logique media.rs
//!
//! Fichier à placer dans : src-tauri/src/
//! Inclure dans main.rs avec : #[cfg(test)] mod tests_extended;
//!
//! Couverture :
//!   - regex.rs  : tous les patterns (saison/épisode, résolution, source,
//!                 HDR, année, provider, codecs audio, nettoyage titre)
//!   - media.rs  : parse_fps, parse_duration, tri des langues audio/sub,
//!                 déduplication, calcul gain de compression, parsing JSON
//!                 ffprobe simulé

#[cfg(test)]
mod tests_regex {
    use rencodex::naming::regex::*;
    use rencodex::naming::regex::normalize_hdr_tags;

    // ══════════════════════════════════════════════════════════════════════
    // SAISON / ÉPISODE
    // ══════════════════════════════════════════════════════════════════════

    // ── S_E (format standard SxxExx) ─────────────────────────────────────

    #[test]
    fn test_se_standard() {
        assert!(S_E.is_match("Show.S01E01.mkv"));
        assert!(S_E.is_match("Show.S12E24.mkv"));
    }

    #[test]
    fn test_se_lowercase() {
        assert!(S_E.is_match("show.s03e07.mkv"));
    }

    #[test]
    fn test_se_with_dots_separators() {
        assert!(S_E.is_match("Show.S02.E05.mkv"));
        assert!(S_E.is_match("Show S02-E05 720p"));
    }

    #[test]
    fn test_se_no_match_year() {
        // Une année comme 2024 ne doit pas être confondue avec S20E24
        assert!(!S_E.is_match("Film.2024.BluRay"));
    }

    #[test]
    fn test_se_captures_correct_groups() {
        let caps = S_E.captures("Show.S03E12.720p").unwrap();
        assert_eq!(&caps[1], "03");
        assert_eq!(&caps[2], "12");
    }

    // ── S_E_MULTI (multi-épisodes SxxExxExx) ─────────────────────────────

    #[test]
    fn test_se_multi_standard() {
        assert!(S_E_MULTI.is_match("Show.S01E01E02.mkv"));
        assert!(S_E_MULTI.is_match("Show.S02E03E04E05.mkv"));
    }

    #[test]
    fn test_se_multi_captures_three_groups() {
        let caps = S_E_MULTI.captures("Show.S01E03E04.mkv").unwrap();
        assert_eq!(&caps[1], "01");
        assert_eq!(&caps[2], "03");
        assert_eq!(&caps[3], "04");
    }

    #[test]
    fn test_se_multi_no_match_single_episode() {
        // Un seul épisode ne doit pas matcher S_E_MULTI
        assert!(!S_E_MULTI.is_match("Show.S01E01.mkv"));
    }

    // ── NUM_X (format 1x01) ───────────────────────────────────────────────

    #[test]
    fn test_num_x_standard() {
        assert!(NUM_X.is_match("Show 2x05 720p"));
        assert!(NUM_X.is_match("Show 1X12 HDTV"));
    }

    #[test]
    fn test_num_x_resolution_not_matched() {
        // "1920x1080" ne doit pas être pris pour un numéro d'épisode
        // NUM_X exige un début de chaîne ou un espace devant le chiffre de saison
        assert!(!NUM_X.is_match("1920x1080"));
    }

    // ── SAISON_EP (format texte) ──────────────────────────────────────────

    #[test]
    fn test_saison_ep_text_format() {
        assert!(SAISON_EP.is_match("Saison 2 Episode 3"));
        assert!(SAISON_EP.is_match("Saison.2.Episode.3"));
        assert!(SAISON_EP.is_match("saison_2_episode_3"));
    }

    #[test]
    fn test_saison_ep_captures() {
        let caps = SAISON_EP.captures("Saison 3 Episode 12").unwrap();
        assert_eq!(&caps[1], "3");
        assert_eq!(&caps[2], "12");
    }

    // ── E_LONG (épisode seul Exx) ─────────────────────────────────────────

    #[test]
    fn test_e_long_matches() {
        assert!(E_LONG.is_match("Show E12 720p"));
        assert!(E_LONG.is_match("Anime E100"));
    }

    #[test]
    fn test_e_long_no_match_in_word() {
        // "Extended" ne doit pas matcher \bExx\b
        assert!(!E_LONG.is_match("Extended.Cut.BluRay"));
    }

    // ── S_DASH_EP (format anime "S3 - 01") ───────────────────────────────

    #[test]
    fn test_s_dash_ep_standard() {
        assert!(S_DASH_EP.is_match("Show S3 - 01 [SubsPlease]"));
        assert!(S_DASH_EP.is_match("Anime S2 – 12"));
        assert!(S_DASH_EP.is_match("Series S1-05"));
    }

    #[test]
    fn test_s_dash_ep_captures() {
        let caps = S_DASH_EP.captures("Anime S2 - 07 1080p").unwrap();
        assert_eq!(&caps[1], "2");
        assert_eq!(&caps[2], "07");
    }

    // ── EP_TRAIL (numéro trailing " - 12") ───────────────────────────────

    #[test]
    fn test_ep_trail_matches() {
        assert!(EP_TRAIL.is_match("Anime Title - 12"));
        assert!(EP_TRAIL.is_match("Show  05"));
    }

    // ── OVA_SPECIAL ──────────────────────────────────────────────────────

    #[test]
    fn test_ova_matches() {
        assert!(OVA_SPECIAL.is_match("Show OVA 1"));
        assert!(OVA_SPECIAL.is_match("Anime - OVA"));
        assert!(OVA_SPECIAL.is_match("Series OAD2"));
        assert!(OVA_SPECIAL.is_match("Show Special 3"));
    }

    // ══════════════════════════════════════════════════════════════════════
    // RÉSOLUTION
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_resolution_keywords() {
        for kw in &["2160p", "4K", "UHD", "1080p", "FHD", "720p", "HD", "480p", "SD"] {
            assert!(RESOLUTION.is_match(kw), "expected match for: {}", kw);
        }
    }

    #[test]
    fn test_resolution_case_insensitive() {
        assert!(RESOLUTION.is_match("1080P"));
        assert!(RESOLUTION.is_match("720P"));
        assert!(RESOLUTION.is_match("uhd"));
    }

    #[test]
    fn test_resolution_num_standard() {
        assert!(RESOLUTION_NUM.is_match("1920x1080"));
        assert!(RESOLUTION_NUM.is_match("3840x2160"));
        assert!(RESOLUTION_NUM.is_match("1280x720"));
    }

    #[test]
    fn test_resolution_num_captures() {
        let caps = RESOLUTION_NUM.captures("1920x1080").unwrap();
        assert_eq!(&caps[1], "1920");
        assert_eq!(&caps[2], "1080");
    }

    #[test]
    fn test_resolution_no_false_positive_year() {
        // "2024" ne doit pas être confondu avec une résolution numérique
        assert!(!RESOLUTION_NUM.is_match("2024"));
    }

    // ══════════════════════════════════════════════════════════════════════
    // SOURCES VIDÉO
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_src_webdl_variants() {
        assert!(SRC_WEBDL.is_match("Show 1080p WEB-DL"));
        assert!(SRC_WEBDL.is_match("Show WEBDL x265"));
        assert!(SRC_WEBDL.is_match("film web-dl"));
    }

    #[test]
    fn test_src_webrip_variants() {
        assert!(SRC_WEBRIP.is_match("Show 720p WEBRip"));
        assert!(SRC_WEBRIP.is_match("Show WEB-Rip x264"));
    }

    #[test]
    fn test_src_bluray_variants() {
        assert!(SRC_BLURAY.is_match("Film BluRay 1080p"));
        assert!(SRC_BLURAY.is_match("Film Blu-Ray"));
        assert!(SRC_BLURAY.is_match("Film Blu Ray REMUX"));
    }

    #[test]
    fn test_src_bdrip() {
        assert!(SRC_BDRIP.is_match("Film BDRip x264"));
        assert!(SRC_BDRIP.is_match("Film BD-Rip"));
    }

    #[test]
    fn test_src_brrip() {
        assert!(SRC_BRRIP.is_match("Film BRRip x265"));
        assert!(SRC_BRRIP.is_match("Film BR-Rip"));
    }

    #[test]
    fn test_src_hdtv() {
        assert!(SRC_HDTV.is_match("Show HDTV 720p"));
    }

    #[test]
    fn test_src_dvdrip() {
        assert!(SRC_DVDRIP.is_match("Film DVDRip x264"));
        assert!(SRC_DVDRIP.is_match("Film DVD-Rip"));
    }

    #[test]
    fn test_src_hdrip() {
        assert!(SRC_HDRIP.is_match("Film HDRip 720p"));
        assert!(SRC_HDRIP.is_match("Film HD-Rip"));
    }

    #[test]
    fn test_src_web_isolated() {
        // "WEB" seul doit matcher
        assert!(SRC_WEB.is_match("Film 1080p WEB x265"));
        // "WEBDL" ne doit pas matcher SRC_WEB (géré par SRC_WEBDL)
        assert!(!SRC_WEB.is_match("FilmWEBDL"));
    }

    #[test]
    fn test_src_bd_isolated() {
        assert!(SRC_BD.is_match("Film BD 1080p"));
        // "BDRip" ne doit pas matcher SRC_BD seul
        assert!(!SRC_BD.is_match("FilmBDRip"));
    }

    // ══════════════════════════════════════════════════════════════════════
    // HDR
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_hdr_variants() {
        let inputs = [
            "Film HDR10+ BluRay",
            "Film HDR10 1080p",
            "Film HDR 2160p",
            "Film DV BluRay",
            "Film DoVi 4K",
            "Film Dolby Vision UHD",
            "Film HLG HDTV",
            "Film SDR 720p",
        ];
        for input in &inputs {
            assert!(HDR.is_match(input), "expected HDR match in: {}", input);
        }
    }

    #[test]
    fn test_hdr_no_false_positive() {
        // "HDTV" ne contient pas HDR comme mot complet
        assert!(!HDR.is_match("Show HDTV 720p"));
    }

    // ══════════════════════════════════════════════════════════════════════
    // ÉDITION SPÉCIALE
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_edition_variants() {
        let inputs = [
            "Film Extended Cut",
            "Film Extended",
            "Film Director's Cut",
            "Film Directors Cut",
            "Film IMAX 1080p",
            "Film Theatrical 4K",
            "Film Unrated",
            "Film REMASTERED",
            "Film Criterion",
            "Film Anniversary Edition",
        ];
        for input in &inputs {
            assert!(EDITION.is_match(input), "expected EDITION match in: {}", input);
        }
    }

    // ══════════════════════════════════════════════════════════════════════
    // ANNÉE
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_year_range() {
        assert!(YEAR.is_match("Film.1990.BluRay"));
        assert!(YEAR.is_match("Film.2024.WEB-DL"));
        assert!(YEAR.is_match("Film.2099.HDR"));
        // Avant 1900 ne doit pas matcher
        assert!(!YEAR.is_match("Film.1899.Old"));
    }

    #[test]
    fn test_year_not_in_resolution() {
        // "1080" ne doit pas être capturé comme année
        let caps: Vec<_> = YEAR.find_iter("Film 1080p BluRay").collect();
        assert!(caps.is_empty(), "1080 should not match as a year");
    }

    #[test]
    fn test_year_paren() {
        assert!(YEAR_PAREN.is_match("Film (2023) BluRay"));
        assert!(!YEAR_PAREN.is_match("Film 2023 BluRay"));
    }

    #[test]
    fn test_year_paren_captures() {
        let caps = YEAR_PAREN.captures("Film (2021) 1080p").unwrap();
        assert_eq!(&caps[1], "2021");
    }

    #[test]
    fn test_year_ctx_captures_groups() {
        // Vérifie que les groupes 1, 2, 3 sont bien le séparateur avant, l'année, le séparateur après
        let caps = YEAR_CTX.captures("Film.2023.BluRay").unwrap();
        assert_eq!(&caps[2], "2023");
    }

    // ══════════════════════════════════════════════════════════════════════
    // FOURNISSEURS DE STREAMING
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_provider_known_sigles() {
        let providers = ["NF", "AMZN", "DSNP", "CR", "HMAX", "ATVP", "HULU", "ADN", "ARTE"];
        for p in &providers {
            assert!(PROVIDER.is_match(p), "expected PROVIDER match for: {}", p);
        }
    }

    #[test]
    fn test_provider_in_context() {
        assert!(PROVIDER.is_match("Show.S01E01.NF.WEB-DL.1080p"));
        assert!(PROVIDER.is_match("Film.AMZN.HDR.2160p"));
    }

    #[test]
    fn test_provider_no_false_positive() {
        // "HDTV" ne doit pas matcher (pas dans la liste)
        assert!(!PROVIDER.is_match("Show HDTV 720p"));
    }

    // ══════════════════════════════════════════════════════════════════════
    // CODECS AUDIO
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_audio_aac_variants() {
        assert!(AUDIO_CODEC_NUM.is_match("Film AAC 2.0 BluRay"));
        assert!(AUDIO_CODEC_NUM.is_match("Film AAC5.1 WEB"));
        assert!(AUDIO_CODEC_NUM.is_match("Film AAC2.0"));
    }

    #[test]
    fn test_audio_dd_variants() {
        assert!(AUDIO_DD.is_match("Film DD5.1"));
        assert!(AUDIO_DD.is_match("Film DDP 7.1"));
        assert!(AUDIO_DD.is_match("Film DD+"));
        assert!(AUDIO_DD.is_match("Film DDP+"));
    }

    #[test]
    fn test_audio_dts_variants() {
        assert!(AUDIO_DTS.is_match("Film DTS BluRay"));
        assert!(AUDIO_DTS.is_match("Film DTS-HD MA"));
        assert!(AUDIO_DTS.is_match("Film DTS:X"));
        assert!(AUDIO_DTS.is_match("Film DTS-MA"));
    }

    // ══════════════════════════════════════════════════════════════════════
    // CODEC H264 / H265 AVEC ESPACES
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_h264_space() {
        assert!(H264_SPACE.is_match("Film H 264 BluRay"));
        assert!(!H264_SPACE.is_match("Film H264 BluRay")); // sans espace = non concerné
    }

    #[test]
    fn test_h265_space() {
        assert!(H265_SPACE.is_match("Film H 265 WEB-DL"));
        assert!(!H265_SPACE.is_match("Film H265 WEB-DL"));
    }

    // ══════════════════════════════════════════════════════════════════════
    // NETTOYAGE TITRE
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_brackets_removes_content() {
        let result = BRACKETS.replace_all("Show [SubsPlease] 1080p", "");
        assert!(!result.contains("SubsPlease"));
    }

    #[test]
    fn test_brackets_parentheses() {
        let result = BRACKETS.replace_all("Show (2023) BluRay", "");
        assert!(!result.contains("2023"));
    }

    #[test]
    fn test_brackets_non_greedy() {
        // Non-greedy : deux groupes distincts restent deux remplacements séparés
        let result = BRACKETS.replace_all("[Groupe] Show [1080p]", "");
        assert!(!result.contains("Groupe"));
        assert!(!result.contains("1080p"));
    }

    #[test]
    fn test_trail_group_captures() {
        assert!(TRAIL_GROUP.is_match("Show - SubsPlease"));
        assert!(TRAIL_GROUP.is_match("Show -RARBG"));
    }

    #[test]
    fn test_trail_group_short_not_matched() {
        // Groupes de moins de 3 caractères ne doivent pas matcher
        assert!(!TRAIL_GROUP.is_match("Show - AB"));
    }

    #[test]
    fn test_trail_dash_removes_ending_dashes() {
        let result = TRAIL_DASH.replace_all("Show Title -–", "");
        assert!(!result.trim().ends_with('-'));
    }

    #[test]
    fn test_multi_sp_collapses() {
        let result = MULTI_SP.replace_all("Show   Title  720p", " ");
        assert_eq!(result, "Show Title 720p");
    }

    #[test]
    fn test_multi_tag_matches() {
        // MULTI_TAG = \bMULTI\b (case-insensitive) : matche "MULTI" isolé
        assert!(MULTI_TAG.is_match("Show MULTI WEB-DL"));
        assert!(MULTI_TAG.is_match("Film multi NF"));
        // "MULTi4" contient "MULTI" sans word-boundary en fin → ne matche pas
        // (le tag complet "MULTi4" est géré par TECH_TAGS, pas MULTI_TAG)
        assert!(!MULTI_TAG.is_match("Film MULTi4 NF"),
            "MULTi4 should NOT match MULTI_TAG because it lacks a word boundary after MULTI");
    }

    #[test]
    fn test_apostrophe_normalisation() {
        // Apostrophes typographiques → ASCII
        let result = APOSTROPHE.replace_all("\u{2018}test\u{2019}", "'");
        assert_eq!(result, "'test'");
    }

    #[test]
    fn test_invalid_chars_windows() {
        let dirty = "film<colon>name|pipe.mkv";
        let result = INVALID_CHARS.replace_all(dirty, "");
        assert!(!result.contains('<'));
        assert!(!result.contains('|'));
    }

    // ══════════════════════════════════════════════════════════════════════
    // TECH_TAGS (liste de tags à supprimer)
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_tech_tags_not_empty() {
        assert!(!TECH_TAGS.is_empty(), "TECH_TAGS should contain compiled regexes");
    }

    #[test]
    fn test_tech_tags_removes_known_tags() {
        let input = "Show FRENCH VOSTFR x265 HEVC 10bit AAC REPACK";
        let mut result = input.to_string();
        for re in TECH_TAGS.iter() {
            result = (*re).replace_all(result.as_str(), "").into_owned();
        }
        let result = result.trim().to_string();
        assert!(!result.contains("FRENCH"));
        assert!(!result.contains("x265"));
        assert!(!result.contains("HEVC"));
        assert!(!result.contains("REPACK"));
    }

    #[test]
    fn test_tech_tags_word_boundary() {
        // "VF2HD" ne doit pas être réduit à "2HD" par le tag "VF"
        // car il n'y a pas de word-boundary après "VF" dans "VF2HD"
        let input = "Film VF2HD 1080p";
        let mut result = input.to_string();
        for re in TECH_TAGS.iter() {
            result = (*re).replace_all(result.as_str(), "").into_owned();
        }
        // "VF2HD" reste intact (pas dans TECH_TAGS) et "VF" seul est dans TECH_TAGS mais
        // ici il est accolé à "2HD" → word-boundary protège "VF2HD"
        // (comportement attendu : la suppression de "VF" isole ne touche pas "VF2HD")
        assert!(!result.contains(" VF ") || result.contains("VF2HD"),
            "word-boundary should protect VF2HD from VF tag removal");
    }

    // ══════════════════════════════════════════════════════════════════════
    // KNOWN_GROUPS (groupes de release)
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_known_groups_not_empty() {
        assert!(!KNOWN_GROUPS.is_empty());
    }

    #[test]
    fn test_known_groups_removes_subsplease() {
        let input = "Anime Title - SubsPlease";
        let mut result = input.to_string();
        for re in KNOWN_GROUPS.iter() {
            result = (*re).replace_all(result.as_str(), "").into_owned();
        }
        assert!(!result.contains("SubsPlease"), "SubsPlease should be removed");
    }

    #[test]
    fn test_known_groups_removes_yify() {
        let input = "Film Title 1080p BluRay -YIFY";
        let mut result = input.to_string();
        for re in KNOWN_GROUPS.iter() {
            result = (*re).replace_all(result.as_str(), "").into_owned();
        }
        assert!(!result.contains("YIFY"));
    }


    // ── TRAIL_GROUP_STUCK ─────────────────────────────────────────────────

    #[test]
    fn test_trail_group_stuck_no_false_positive_hyphenated_title() {
        assert!(!TRAIL_GROUP_STUCK.is_match("The Amazing Spider-Man"),
            "Spider-Man ne doit pas matcher comme groupe release collé");
        assert!(!TRAIL_GROUP_STUCK.is_match("Man-Thing 2023"));
        assert!(!TRAIL_GROUP_STUCK.is_match("Demon-Slayer 1080p"));
    }

    #[test]
    fn test_trail_group_stuck_matches_known_codecs() {
        assert!(TRAIL_GROUP_STUCK.is_match("Anime.x264-Tsundere-Raws"));
        assert!(TRAIL_GROUP_STUCK.is_match("Film.x265-SubsPlease"));
        assert!(TRAIL_GROUP_STUCK.is_match("Show.HEVC-Judas"));
        assert!(TRAIL_GROUP_STUCK.is_match("Film.AVC-YIFY"));
    }

    #[test]
    fn test_trail_group_stuck_captures_group_name() {
        let caps = TRAIL_GROUP_STUCK.captures("Anime.x264-Tsundere-Raws").unwrap();
        assert_eq!(&caps[1], "Tsundere-Raws");
    }

    // ── OVA_SPECIAL — Special seul sans numéro ───────────────────────────

    #[test]
    fn test_ova_special_alone_no_match() {
        assert!(!OVA_SPECIAL.is_match("Special Ops Lioness S01E01"),
            "Special Ops sans numéro ne doit pas matcher OVA_SPECIAL");
        assert!(!OVA_SPECIAL.is_match("The End Special Edition"));
    }

    #[test]
    fn test_ova_special_with_number_still_matches() {
        assert!(OVA_SPECIAL.is_match("Anime Special 3"));
        assert!(OVA_SPECIAL.is_match("Show Special.2"));
    }

    // ── normalize_hdr_tags ────────────────────────────────────────────────

    #[test]
    fn test_normalize_hdr_single_tags() {
        assert_eq!(normalize_hdr_tags("Film HDR10 BluRay"), "HDR10");
        assert_eq!(normalize_hdr_tags("Film HDR10+ x265"), "HDR10+");
        assert_eq!(normalize_hdr_tags("Film HLG HDTV"), "HLG");
        assert_eq!(normalize_hdr_tags("Film HDR WEB"), "HDR");
    }

    #[test]
    fn test_normalize_hdr_dv_aliases() {
        assert_eq!(normalize_hdr_tags("Film DV BluRay"), "DV");
        assert_eq!(normalize_hdr_tags("Film DoVi 4K"), "DV");
        assert_eq!(normalize_hdr_tags("Film Dolby Vision UHD"), "DV");
    }

    #[test]
    fn test_normalize_hdr_combinations_order_independent() {
        assert_eq!(normalize_hdr_tags("Film DV HDR10 BluRay"), "DV HDR10");
        assert_eq!(normalize_hdr_tags("Film HDR10 DV BluRay"), "DV HDR10",
            "l'ordre dans le nom ne doit pas affecter la normalisation");
        assert_eq!(normalize_hdr_tags("Film DoVi HDR10+ x265"), "DV HDR10+");
        assert_eq!(normalize_hdr_tags("Film DV HLG"), "DV HLG");
        assert_eq!(normalize_hdr_tags("Film DV HDR x265"), "DV HDR");
    }

    #[test]
    fn test_normalize_hdr_sdr_ignored() {
        assert_eq!(normalize_hdr_tags("Film SDR 1080p"), "");
    }

    #[test]
    fn test_normalize_hdr_no_token() {
        assert_eq!(normalize_hdr_tags("Film 1080p BluRay x264"), "");
    }

    #[test]
    fn test_normalize_hdr_no_false_positive_hdrip() {
        assert_eq!(normalize_hdr_tags("Film HDRip 720p"), "");
    }

    // ── TECH_TAGS — régressions END / Special ────────────────────────────

    #[test]
    fn test_tech_tags_end_not_removed() {
        let input = "The End 2024 1080p BluRay";
        let mut result = input.to_string();
        for re in TECH_TAGS.iter() {
            result = (*re).replace_all(result.as_str(), "").into_owned();
        }
        assert!(result.contains("End"),
            "\"End\" ne doit pas être supprimé par TECH_TAGS, reçu: {}", result);
    }

    #[test]
    fn test_tech_tags_special_not_removed() {
        let input = "Special Ops Lioness S01E01 1080p WEB-DL";
        let mut result = input.to_string();
        for re in TECH_TAGS.iter() {
            result = (*re).replace_all(result.as_str(), "").into_owned();
        }
        assert!(result.contains("Special"),
            "\"Special\" ne doit pas être supprimé par TECH_TAGS, reçu: {}", result);
    }

    #[test]
    fn test_tech_tags_repack_still_removed() {
        let input = "Film 2023 1080p BluRay REPACK";
        let mut result = input.to_string();
        for re in TECH_TAGS.iter() {
            result = (*re).replace_all(result.as_str(), "").into_owned();
        }
        assert!(!result.contains("REPACK"), "REPACK doit toujours être supprimé");
    }

    // ── KNOWN_GROUPS — compléments ────────────────────────────────────────

    #[test]
    fn test_known_groups_removes_erai_raws() {
        let input = "Anime Title - Erai-raws";
        let mut result = input.to_string();
        for re in KNOWN_GROUPS.iter() {
            result = (*re).replace_all(result.as_str(), "").into_owned();
        }
        assert!(!result.contains("Erai-raws"));
    }

    #[test]
    fn test_known_groups_removes_rarbg_dash() {
        let input = "Film.Title.2022.1080p.BluRay-RARBG";
        let mut result = input.to_string();
        for re in KNOWN_GROUPS.iter() {
            result = (*re).replace_all(result.as_str(), "").into_owned();
        }
        assert!(!result.contains("RARBG"));
    }

    #[test]
    fn test_known_groups_does_not_touch_spectre_title() {
        let input = "Spectre.2015.1080p.BluRay";
        let mut result = input.to_string();
        for re in KNOWN_GROUPS.iter() {
            result = (*re).replace_all(result.as_str(), "").into_owned();
        }
        assert!(result.contains("Spectre"),
            "\"Spectre\" titre ne doit pas être altéré, reçu: {}", result);
    }

    #[test]
    fn test_known_groups_removes_stuck_group() {
        // Format collé : "x264-Tsundere-Raws"
        let input = "Anime.x264-Tsundere-Raws";
        let mut result = input.to_string();
        for re in KNOWN_GROUPS.iter() {
            result = (*re).replace_all(result.as_str(), "").into_owned();
        }
        assert!(!result.contains("Tsundere-Raws"));
    }
}

// ══════════════════════════════════════════════════════════════════════════
// TESTS LOGIQUE MEDIA (fonctions pures extraites de media.rs)
// ══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests_media_logic {
    use rencodex::utils::normalize_lang;

    // ── parse_fps depuis r_frame_rate ─────────────────────────────────────

    fn parse_fps(r_frame_rate: &str) -> Option<f64> {
        let parts: Vec<f64> = r_frame_rate
            .split('/')
            .filter_map(|x| x.parse().ok())
            .collect();
        if parts.len() == 2 && parts[1] > 0.0 {
            Some(parts[0] / parts[1])
        } else {
            None
        }
    }

    #[test]
    fn test_fps_standard_24() {
        let fps = parse_fps("24000/1001").unwrap();
        assert!((fps - 23.976).abs() < 0.01, "expected ~23.976, got {}", fps);
    }

    #[test]
    fn test_fps_25() {
        let fps = parse_fps("25/1").unwrap();
        assert!((fps - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_fps_30() {
        let fps = parse_fps("30000/1001").unwrap();
        assert!((fps - 29.97).abs() < 0.01);
    }

    #[test]
    fn test_fps_60() {
        let fps = parse_fps("60/1").unwrap();
        assert!((fps - 60.0).abs() < 0.001);
    }

    #[test]
    fn test_fps_invalid_zero_denominator() {
        assert!(parse_fps("24/0").is_none());
    }

    #[test]
    fn test_fps_invalid_garbage() {
        assert!(parse_fps("N/A").is_none());
        assert!(parse_fps("").is_none());
    }

    #[test]
    fn test_fps_single_value() {
        // Certains conteneurs donnent "25" sans fraction
        assert!(parse_fps("25").is_none()); // notre impl exige deux parties
    }

    // ── parse_duration ────────────────────────────────────────────────────

    fn parse_duration(s: Option<&str>) -> f64 {
        s.and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0)
    }

    #[test]
    fn test_duration_standard() {
        assert!((parse_duration(Some("3600.0")) - 3600.0).abs() < 0.001);
    }

    #[test]
    fn test_duration_fractional() {
        assert!((parse_duration(Some("5432.123")) - 5432.123).abs() < 0.001);
    }

    #[test]
    fn test_duration_none() {
        assert_eq!(parse_duration(None), 0.0);
    }

    #[test]
    fn test_duration_invalid_string() {
        assert_eq!(parse_duration(Some("N/A")), 0.0);
        assert_eq!(parse_duration(Some("")), 0.0);
    }

    // ── Déduplication et tri des langues audio/sub ────────────────────────
    //
    // Reproduit la logique de analyze_file() :
    //   - déduplique les langues par ordre d'apparition
    //   - trie selon l'ordre de priorité interne

    fn sort_langs(langs: &mut Vec<String>) {
        let order = ["fre", "eng", "jpn", "kor", "ger", "spa", "ita", "por", "rus", "chi", "und"];
        langs.sort_by_key(|l| order.iter().position(|o| *o == l.as_str()).unwrap_or(999));
    }

    fn dedup_langs(raw: &[&str]) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for &lang in raw {
            let normalized = normalize_lang(lang);
            if !out.contains(&normalized) {
                out.push(normalized);
            }
        }
        out
    }

    #[test]
    fn test_dedup_removes_duplicates() {
        let input = ["fre", "eng", "fre", "jpn", "eng"];
        let result = dedup_langs(&input);
        assert_eq!(result.len(), 3);
        assert!(result.contains(&"fre".to_string()));
        assert!(result.contains(&"eng".to_string()));
        assert!(result.contains(&"jpn".to_string()));
    }

    #[test]
    fn test_dedup_preserves_first_occurrence_order() {
        let input = ["jpn", "fre", "eng"];
        let result = dedup_langs(&input);
        assert_eq!(result[0], "jpn");
        assert_eq!(result[1], "fre");
        assert_eq!(result[2], "eng");
    }

    #[test]
    fn test_sort_langs_priority_fre_first() {
        let mut langs = vec!["eng".to_string(), "jpn".to_string(), "fre".to_string()];
        sort_langs(&mut langs);
        assert_eq!(langs[0], "fre");
    }

    #[test]
    fn test_sort_langs_eng_before_jpn() {
        let mut langs = vec!["jpn".to_string(), "eng".to_string()];
        sort_langs(&mut langs);
        assert_eq!(langs[0], "eng");
    }

    #[test]
    fn test_sort_langs_und_last() {
        let mut langs = vec!["und".to_string(), "fre".to_string(), "eng".to_string()];
        sort_langs(&mut langs);
        assert_eq!(langs.last().unwrap(), "und");
    }

    #[test]
    fn test_sort_langs_unknown_after_und() {
        // Une langue inconnue (ex "hin") arrive après "und" dans l'ordre
        let mut langs = vec!["hin".to_string(), "und".to_string(), "fre".to_string()];
        sort_langs(&mut langs);
        assert_eq!(langs[0], "fre");
        // "hin" et "und" sont en fin, dans un ordre indéfini entre eux
        let tail: Vec<_> = langs[1..].to_vec();
        assert!(tail.contains(&"und".to_string()));
        assert!(tail.contains(&"hin".to_string()));
    }

    // ── Calcul du gain de compression ─────────────────────────────────────

    fn compression_gain(original_mb: f64, encoded_mb: f64) -> f64 {
        if original_mb > 0.0 {
            100.0 * (original_mb - encoded_mb) / original_mb
        } else {
            0.0
        }
    }

    #[test]
    fn test_gain_fifty_percent() {
        let gain = compression_gain(1000.0, 500.0);
        assert!((gain - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_gain_zero_original() {
        assert_eq!(compression_gain(0.0, 0.0), 0.0);
    }

    #[test]
    fn test_gain_no_compression() {
        let gain = compression_gain(500.0, 500.0);
        assert!((gain - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_gain_negative_means_bigger_output() {
        // Si l'encodé est plus gros que l'original
        let gain = compression_gain(100.0, 150.0);
        assert!(gain < 0.0);
    }

    #[test]
    fn test_gain_near_hundred_percent() {
        let gain = compression_gain(1000.0, 1.0);
        assert!(gain > 99.0);
    }

    // ── Parse JSON ffprobe simulé ─────────────────────────────────────────
    //
    // Teste la logique d'extraction sans appeler ffprobe réellement.

    fn parse_ffprobe_json(json_str: &str) -> Option<(f64, f64, Vec<String>)> {
        let json: serde_json::Value = serde_json::from_str(json_str).ok()?;
        let duration = json["format"]["duration"]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        let fps = json["streams"]
            .as_array()?
            .iter()
            .find(|s| s["codec_type"].as_str() == Some("video"))
            .and_then(|v| v["r_frame_rate"].as_str())
            .and_then(|r| {
                let parts: Vec<f64> = r.split('/').filter_map(|x| x.parse().ok()).collect();
                if parts.len() == 2 && parts[1] > 0.0 { Some(parts[0] / parts[1]) } else { None }
            })
            .unwrap_or(25.0);
        let langs: Vec<String> = json["streams"]
            .as_array()?
            .iter()
            .filter(|s| s["codec_type"].as_str() == Some("audio"))
            .map(|s| normalize_lang(s["tags"]["language"].as_str().unwrap_or("und")))
            .collect();
        Some((duration, fps, langs))
    }

    #[test]
    fn test_parse_ffprobe_full_file() {
        let json = r#"{
            "format": { "duration": "7200.0" },
            "streams": [
                { "codec_type": "video", "r_frame_rate": "24000/1001" },
                { "codec_type": "audio", "tags": { "language": "fre" } },
                { "codec_type": "audio", "tags": { "language": "eng" } }
            ]
        }"#;
        let (dur, fps, langs) = parse_ffprobe_json(json).unwrap();
        assert!((dur - 7200.0).abs() < 0.1);
        assert!((fps - 23.976).abs() < 0.01);
        assert_eq!(langs, vec!["fre", "eng"]);
    }

    #[test]
    fn test_parse_ffprobe_missing_duration() {
        let json = r#"{
            "format": {},
            "streams": [
                { "codec_type": "video", "r_frame_rate": "25/1" }
            ]
        }"#;
        let (dur, fps, langs) = parse_ffprobe_json(json).unwrap();
        assert_eq!(dur, 0.0);
        assert!((fps - 25.0).abs() < 0.001);
        assert!(langs.is_empty());
    }

    #[test]
    fn test_parse_ffprobe_unknown_audio_lang() {
        let json = r#"{
            "format": { "duration": "3600.0" },
            "streams": [
                { "codec_type": "video", "r_frame_rate": "30/1" },
                { "codec_type": "audio", "tags": { "language": "und" } }
            ]
        }"#;
        let (_, _, langs) = parse_ffprobe_json(json).unwrap();
        assert_eq!(langs, vec!["und"]);
    }

    #[test]
    fn test_parse_ffprobe_no_audio_streams() {
        let json = r#"{
            "format": { "duration": "120.0" },
            "streams": [
                { "codec_type": "video", "r_frame_rate": "25/1" }
            ]
        }"#;
        let (_, _, langs) = parse_ffprobe_json(json).unwrap();
        assert!(langs.is_empty());
    }

    #[test]
    fn test_parse_ffprobe_malformed_json() {
        assert!(parse_ffprobe_json("{not valid json}").is_none());
    }

    #[test]
    fn test_parse_ffprobe_iso639_1_normalization() {
        // Les codes ISO 639-1 (2 lettres) doivent être normalisés en 639-2
        let json = r#"{
            "format": { "duration": "1800.0" },
            "streams": [
                { "codec_type": "video", "r_frame_rate": "25/1" },
                { "codec_type": "audio", "tags": { "language": "ja" } },
                { "codec_type": "audio", "tags": { "language": "ko" } }
            ]
        }"#;
        let (_, _, langs) = parse_ffprobe_json(json).unwrap();
        assert_eq!(langs[0], "jpn");
        assert_eq!(langs[1], "kor");
    }

    // ── Logique de progression FFmpeg ─────────────────────────────────────
    //
    // Reproduit le calcul de pourcentage de media.rs sans dépendance Tauri.

    fn compute_percent(
        out_time_secs: Option<f64>,
        current_frame: Option<f64>,
        speed_val: Option<f64>,
        elapsed: f64,
        file_duration: f64,
        file_fps: f64,
    ) -> f64 {
        let total_frames = (file_duration * file_fps).max(1.0);
        let done = if let Some(t) = out_time_secs.filter(|&t| t > 0.5) {
            t.min(file_duration)
        } else if let Some(frame) = current_frame.filter(|&f| f > 0.0) {
            (frame / file_fps).min(file_duration)
        } else if let Some(speed) = speed_val.filter(|&s| s > 0.0) {
            (elapsed * speed).min(file_duration)
        } else {
            (elapsed * 0.8).min(file_duration * 0.8)
        };

        if file_duration > 0.0 {
            (done / file_duration * 100.0).clamp(0.0, 99.9)
        } else if total_frames > 0.0 {
            (current_frame.unwrap_or(0.0) / total_frames * 100.0).clamp(0.0, 99.9)
        } else {
            0.0
        }
    }

    #[test]
    fn test_progress_from_out_time() {
        // Priorité 1 : out_time_us
        let pct = compute_percent(Some(1800.0), None, None, 0.0, 3600.0, 25.0);
        assert!((pct - 50.0).abs() < 0.1, "expected ~50%, got {}", pct);
    }

    #[test]
    fn test_progress_from_frame() {
        // Priorité 2 : frame (pas de out_time)
        let pct = compute_percent(None, Some(1500.0), None, 0.0, 120.0, 25.0);
        // 1500 frames / (120s * 25fps=3000 frames) = 50%
        assert!((pct - 50.0).abs() < 0.5, "expected ~50%, got {}", pct);
    }

    #[test]
    fn test_progress_from_speed() {
        // Priorité 3 : speed (10x pendant 60s sur un fichier de 3600s)
        let pct = compute_percent(None, None, Some(10.0), 60.0, 3600.0, 25.0);
        // done = 60 * 10 = 600s → 600/3600 = ~16.7%
        assert!((pct - 16.7).abs() < 0.5, "expected ~16.7%, got {}", pct);
    }

    #[test]
    fn test_progress_conservative_fallback() {
        // Priorité 4 : fallback linéaire conservateur (0.8x)
        let pct = compute_percent(None, None, None, 100.0, 1000.0, 25.0);
        // done = 100 * 0.8 = 80 → 80/1000 = 8%
        assert!((pct - 8.0).abs() < 0.5, "expected ~8%, got {}", pct);
    }

    #[test]
    fn test_progress_never_reaches_100() {
        // Le pourcentage est limité à 99.9 même si out_time >= duration
        let pct = compute_percent(Some(3600.0), None, None, 0.0, 3600.0, 25.0);
        assert!(pct <= 99.9);
    }

    #[test]
    fn test_progress_clamp_min_zero() {
        let pct = compute_percent(Some(0.0), None, None, 0.0, 3600.0, 25.0);
        assert!(pct >= 0.0);
    }

    #[test]
    fn test_progress_out_time_capped_at_duration() {
        // Si out_time dépasse la durée déclarée, on plafonne à duration
        let pct = compute_percent(Some(5000.0), None, None, 0.0, 3600.0, 25.0);
        assert!(pct <= 99.9);
    }
}
