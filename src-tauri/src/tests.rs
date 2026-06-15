//! Tests unitaires

#[cfg(test)]
mod tests {
    use crate::filename::clean_filename;
    use crate::utils::normalize_lang;

    #[test]
    fn test_normalize_lang() {
        assert_eq!(normalize_lang("fr"), "fre");
        assert_eq!(normalize_lang("French"), "fre");
        assert_eq!(normalize_lang("en"), "eng");
        assert_eq!(normalize_lang("jpn"), "jpn");
        assert_eq!(normalize_lang(""), "und");
    }

    #[test]
    fn test_clean_filename_series() {
        let result = clean_filename(
            "Show.S01E01.Episode.Title.1080p.WEB-DL.x264.mkv".to_string(),
            vec!["eng".to_string()],
            vec![],
        );
        assert_eq!(result.title, "Show");
        assert_eq!(result.season_episode, "S01E01");
        assert_eq!(result.resolution, "1080P");
    }

    #[test]
    fn test_clean_filename_movie() {
        let result = clean_filename(
            "Movie.Title.2022.1080p.BluRay.x264.mkv".to_string(),
            vec!["eng".to_string()],
            vec![],
        );
        assert_eq!(result.title, "Movie Title");
        assert_eq!(result.year, "2022");
        assert!(result.suggested.contains("H265"));
    }

    #[test]
    fn test_clean_filename_with_audio_lang() {
        let result = clean_filename(
            "Anime.Show.S01E01.VOSTFR.1080p.WEB-DL.x264.mkv".to_string(),
            vec!["jpn".to_string()],
            vec!["fre".to_string()],
        );
        assert!(result.suggested.contains("VOSTFR"));
    }
}