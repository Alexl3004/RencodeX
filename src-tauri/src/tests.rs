//! Tests unitaires — formatage des noms de fichiers

#[cfg(test)]
mod tests {
    use crate::filename::clean_filename;
    use crate::utils::{normalize_lang, format_title_case, sanitize_filename, is_windows_reserved};

    // ── normalize_lang ────────────────────────────────────────────────────

    #[test]
    fn test_normalize_lang_basic() {
        assert_eq!(normalize_lang("fr"),      "fre");
        assert_eq!(normalize_lang("French"),  "fre");
        assert_eq!(normalize_lang("fre"),     "fre");
        assert_eq!(normalize_lang("en"),      "eng");
        assert_eq!(normalize_lang("jpn"),     "jpn");
        assert_eq!(normalize_lang("ja"),      "jpn");
        assert_eq!(normalize_lang("ko"),      "kor");
        assert_eq!(normalize_lang("zh"),      "chi");
        assert_eq!(normalize_lang(""),        "und");
        assert_eq!(normalize_lang("und"),     "und");
    }

    // ── format_title_case ─────────────────────────────────────────────────

    #[test]
    fn test_title_case_basic() {
        assert_eq!(format_title_case("the dark knight"), "The Dark Knight");
    }

    #[test]
    fn test_title_case_small_words() {
        // Petits mots en position non-initiale restent en minuscule
        assert_eq!(format_title_case("lord of the rings"), "Lord of the Rings");
        assert_eq!(format_title_case("le seigneur des anneaux"), "Le Seigneur des Anneaux");
    }

    #[test]
    fn test_title_case_preserves_sigles() {
        // Sigles purement alphabétiques
        assert_eq!(format_title_case("NF AMZN"), "NF AMZN");
        assert_eq!(format_title_case("show VF AAC"), "Show VF AAC");
        // Tokens alphanumériques dont les lettres sont toutes en majuscules
        assert_eq!(format_title_case("show VF 1080P"), "Show VF 1080P");
        assert_eq!(format_title_case("film HDR10 H265"), "Film HDR10 H265");
    }

    #[test]
    fn test_title_case_hyphenated() {
        // Mots avec tiret : chaque segment capitalisé
        assert_eq!(format_title_case("spider-man into the spider-verse"), "Spider-Man Into the Spider-Verse");
    }

    #[test]
    fn test_title_case_apostrophe() {
        // L'apostrophe typographique doit être normalisée vers ASCII '
        // "l'" → "L'" (premier mot), "homme" → "Homme" (nom propre, pas petit mot)
        let result = format_title_case("l\u{2019}homme qui voulait vivre sa vie");
        // L'apostrophe doit être présente sous forme ASCII
        assert!(result.contains('\''), "apostrophe ASCII absente dans: {}", result);
        // Le premier mot doit commencer par L majuscule
        assert!(result.starts_with('L'), "doit commencer par L, reçu: {}", result);
    }

    // ── sanitize_filename ─────────────────────────────────────────────────

    #[test]
    fn test_sanitize_removes_invalid_chars() {
        assert_eq!(sanitize_filename("Movie: The Sequel"), "Movie The Sequel");
        assert_eq!(sanitize_filename("File/Name"), "FileName");
        assert_eq!(sanitize_filename("Test<>:\"/\\|?*Name"), "TestName");
    }

    #[test]
    fn test_sanitize_trailing_dots_spaces() {
        assert_eq!(sanitize_filename("Movie..."), "Movie");
        assert_eq!(sanitize_filename("Movie   "), "Movie");
    }

    #[test]
    fn test_sanitize_windows_reserved() {
        assert_eq!(sanitize_filename("CON"), "output");
        assert_eq!(sanitize_filename("NUL"), "output");
        assert_eq!(sanitize_filename("COM1"), "output");
    }

    #[test]
    fn test_is_windows_reserved() {
        assert!(is_windows_reserved("CON"));
        assert!(is_windows_reserved("NUL"));
        assert!(!is_windows_reserved("Concert"));
        assert!(!is_windows_reserved("Movie"));
    }

    // ── clean_filename — séries ───────────────────────────────────────────

    #[test]
    fn test_clean_series_standard() {
        let r = clean_filename(
            "Show.S01E01.Episode.Title.1080p.WEB-DL.x264.mkv".to_string(),
            vec!["eng".to_string()],
            vec![],
        );
        assert_eq!(r.title, "Show");
        assert_eq!(r.season_episode, "S01E01");
        assert_eq!(r.resolution, "1080P");
        assert_eq!(r.source, "WEB-DL");
        assert!(r.suggested.contains("H265"));
    }

    #[test]
    fn test_clean_series_multi_episode() {
        let r = clean_filename(
            "My.Show.S02E03E04.1080p.BluRay.mkv".to_string(),
            vec!["fre".to_string()],
            vec![],
        );
        assert_eq!(r.season_episode, "S02E03-E04");
        assert_eq!(r.source, "BluRay");
    }

    #[test]
    fn test_clean_series_episode_number_trailing() {
        // Anime style : numéro d'épisode seul en fin
        let r = clean_filename(
            "My Anime Show - 07 [1080p].mkv".to_string(),
            vec!["jpn".to_string()],
            vec!["fre".to_string()],
        );
        assert_eq!(r.season_episode, "E07");
        assert_eq!(r.audio_tags, "VOSTFR");
    }

    // ── clean_filename — films ────────────────────────────────────────────

    #[test]
    fn test_clean_movie_standard() {
        let r = clean_filename(
            "Movie.Title.2022.1080p.BluRay.x264.mkv".to_string(),
            vec!["eng".to_string()],
            vec![],
        );
        assert_eq!(r.title, "Movie Title");
        assert_eq!(r.year, "2022");
        assert_eq!(r.resolution, "1080P");
        assert_eq!(r.source, "BluRay");
        assert!(r.suggested.contains("H265"));
        assert!(r.suggested.contains("(2022)"));
    }

    #[test]
    fn test_clean_movie_4k_hdr() {
        let r = clean_filename(
            "Dune.Part.Two.2024.2160p.UHD.BluRay.HDR10.x265.mkv".to_string(),
            vec!["eng".to_string()],
            vec!["fre".to_string()],
        );
        assert_eq!(r.title, "Dune Part Two");
        assert_eq!(r.year, "2024");
        assert_eq!(r.resolution, "2160P");
        assert_eq!(r.hdr, "HDR10");
        // Sous-titres FR avec audio EN → pas de VOSTFR
        assert_ne!(r.audio_tags, "VOSTFR");
    }

    #[test]
    fn test_clean_movie_year_2030_plus() {
        // Régression : YEAR était limité à 2029 dans l'original
        let r = clean_filename(
            "Future.Movie.2031.1080p.WEB-DL.mkv".to_string(),
            vec![],
            vec![],
        );
        assert_eq!(r.year, "2031");
    }

    // ── clean_filename — langues / audio ─────────────────────────────────

    #[test]
    fn test_audio_tag_vostfr_correct() {
        // VOSTFR : audio japonais + sous-titres français
        let r = clean_filename(
            "Anime.Show.S01E01.1080p.WEB-DL.mkv".to_string(),
            vec!["jpn".to_string()],
            vec!["fre".to_string()],
        );
        assert_eq!(r.audio_tags, "VOSTFR");
        assert!(r.suggested.contains("VOSTFR"));
    }

    #[test]
    fn test_audio_tag_vosta() {
        // VOSTA : audio japonais + sous-titres anglais
        let r = clean_filename(
            "Anime.S01E01.mkv".to_string(),
            vec!["jpn".to_string()],
            vec!["eng".to_string()],
        );
        assert_eq!(r.audio_tags, "VOSTA");
    }

    #[test]
    fn test_audio_tag_multi() {
        // MULTI : plusieurs pistes audio
        let r = clean_filename(
            "Movie.2022.1080p.mkv".to_string(),
            vec!["fre".to_string(), "eng".to_string()],
            vec![],
        );
        assert_eq!(r.audio_tags, "MULTI");
    }

    #[test]
    fn test_audio_tag_multi_from_name() {
        // MULTI dans le nom original → MULTI tag même avec une seule piste
        let r = clean_filename(
            "Movie.2022.MULTI.1080p.mkv".to_string(),
            vec!["fre".to_string()],
            vec![],
        );
        assert_eq!(r.audio_tags, "MULTI");
    }

    #[test]
    fn test_audio_tag_vf() {
        let r = clean_filename(
            "Film.2021.1080p.WEB-DL.mkv".to_string(),
            vec!["fre".to_string()],
            vec![],
        );
        assert_eq!(r.audio_tags, "VF");
    }

    // ── clean_filename — cas particuliers ────────────────────────────────

    #[test]
    fn test_web_not_stripped_in_title() {
        // "WEB" seul ne doit pas supprimer "Web" dans un titre
        let r = clean_filename(
            "The.WebMaster.2021.1080p.WEB-DL.mkv".to_string(),
            vec!["eng".to_string()],
            vec![],
        );
        // La source doit être WEB-DL, pas WEB
        assert_eq!(r.source, "WEB-DL");
        // Le titre ne doit pas contenir "DL" ou être vide
        assert!(!r.title.is_empty());
    }

    #[test]
    fn test_provider_extracted() {
        let r = clean_filename(
            "Show.S01E01.NF.1080p.WEB-DL.mkv".to_string(),
            vec!["eng".to_string()],
            vec![],
        );
        assert_eq!(r.provider, "NF");
    }

    #[test]
    fn test_release_group_stripped() {
        let r = clean_filename(
            "Movie.Title.2022.1080p.BluRay-YIFY.mkv".to_string(),
            vec!["eng".to_string()],
            vec![],
        );
        // Le groupe YIFY ne doit pas se retrouver dans le titre
        assert!(!r.title.contains("YIFY"));
        assert_eq!(r.title, "Movie Title");
    }

    #[test]
    fn test_suggested_sanitized() {
        // Les caractères interdits Windows ne doivent pas apparaître
        let r = clean_filename(
            "Movie: The Sequel.2022.1080p.mkv".to_string(),
            vec!["eng".to_string()],
            vec![],
        );
        assert!(!r.suggested.contains(':'));
    }

    #[test]
    fn test_hdr_extracted() {
        let r = clean_filename(
            "Movie.2023.2160p.BluRay.HDR10+.x265.mkv".to_string(),
            vec!["eng".to_string()],
            vec![],
        );
        assert_eq!(r.hdr, "HDR10+");
        assert!(r.suggested.contains("HDR10+"));
    }

    #[test]
    fn test_edition_extracted() {
        let r = clean_filename(
            "Movie.IMAX.2022.1080p.WEB-DL.mkv".to_string(),
            vec!["eng".to_string()],
            vec![],
        );
        assert_eq!(r.edition, "IMAX");
    }

    #[test]
    fn test_ova_special() {
        let r = clean_filename(
            "My Anime - OVA 2 [1080p].mkv".to_string(),
            vec!["jpn".to_string()],
            vec!["fre".to_string()],
        );
        assert!(r.season_episode.contains("OVA"));
    }
}