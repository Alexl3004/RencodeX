//! Expressions régulières pré-compilées pour l'analyse de noms de fichiers

pub use regex::Regex;
use once_cell::sync::Lazy;
use regex::Regex as InnerRegex;

// ── Patterns saison/épisode ───────────────────────────────────────────────

/// S01E01 ou S01E01E02 (multi-épisodes)
pub static S_E: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)[Ss](\d{1,2})[\.\- ]?[Ee](\d{2,})").unwrap()
);
pub static S_E_MULTI: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)[Ss](\d{1,2})[\.\- ]?[Ee](\d{2,})[\.\- ]?[Ee](\d{2,})").unwrap()
);
pub static NUM_X: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)(?:^|\s)(\d{1,2})[xX](\d{2,})").unwrap()
);
pub static SAISON_EP: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)Saison[ ._](\d+)[ ._]Episode[ ._](\d+)").unwrap()
);
pub static E_LONG: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\bE(\d{2,})\b").unwrap()
);
/// Format "S3 - 01" ou "S3 – 01" (saison sans padding + tiret + épisode)
/// Typique des releases anime fansub : "[Group] Show S3 - 01 [...]"
pub static S_DASH_EP: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\bS(\d{1,2})\s*[-–]\s*(\d{2,3})\b").unwrap()
);
/// Numéro isolé en fin de titre (ex: " - 12" pour l'épisode 12)
pub static EP_TRAIL: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)[-\s](\d{1,4})\s*$").unwrap()
);
pub static OVA_SPECIAL: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\s*[-–]?\s*\b(OVA|OAD|ONA)[\s._-]?(\d*)\b|\bSpecial[\s._-](\d+)\b").unwrap()
);

// ── Résolution ────────────────────────────────────────────────────────────

/// Mots-clés standards de résolution (insensible à la casse)
pub static RESOLUTION: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\b(2160p|4K|UHD|1080p|FHD|720p|HD|480p|SD)\b").unwrap()
);
/// Dimensions numériques ex: 1920x1080
pub static RESOLUTION_NUM: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)(\d{3,4})x(\d{3,4})").unwrap()
);

// ── Sources vidéo ─────────────────────────────────────────────────────────
// IMPORTANT : SRC_WEB doit être testé EN DERNIER (score le plus bas) car
// "\bWEB\b" est très générique et pourrait consommer des titres contenant "Web".

pub static SRC_WEBDL: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\bWEB[-\s]?DL\b").unwrap()     // Couvre WEB-DL et WEBDL
);
pub static SRC_WEBRIP: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\bWEB[-\s]?Rip\b").unwrap()    // Couvre WEBRip et WEB-Rip
);
pub static SRC_BLURAY: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\bBlu[-\s]?Ray\b").unwrap()    // Couvre BluRay, Blu-Ray, Blu Ray
);
pub static SRC_BDRIP: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\bBD[-\s]?Rip\b").unwrap()
);
pub static SRC_BRRIP: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\bBR[-\s]?Rip\b").unwrap()
);
/// "BD" seul en tant que source (= BluRay) — Score faible, testé après
/// SRC_BDRIP/SRC_BLURAY pour éviter de tronquer "BDRip" ou de matcher
/// dans des tokens type "BDxxx". On exige des bornes espace/début/fin.
pub static SRC_BD: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)(?:^|[\s._\-\[\(])BD(?:[\s._\-\]\)]|$)").unwrap()
);
pub static SRC_HDTV: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\bHDTV\b").unwrap()
);
pub static SRC_DVDRIP: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\bDVD[-\s]?Rip\b").unwrap()
);
pub static SRC_HDRIP: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\bHD[-\s]?Rip\b").unwrap()
);
/// "WEB" seul en tant que source — Score faible, testé en dernier dans detect_source.
/// On matche " WEB " ou "^WEB " ou " WEB$" pour éviter "Webmaster", "WEBDL", etc.
/// La crate `regex` de Rust ne supporte pas les lookahead/lookbehind, donc on
/// contrôle le contexte via des espaces/début-fin de chaîne.
pub static SRC_WEB: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)(?:^|\s)WEB(?:\s|$)").unwrap()
);

// ── HDR ───────────────────────────────────────────────────────────────────

/// Regex de détection individuelle des tokens HDR.
///
/// Stratégie : on utilise `find_iter` pour collecter TOUS les tokens HDR
/// présents dans le nom de fichier, puis `normalize_hdr_tags` les combine
/// en un tag canonique unique (ex: ["DV", "HDR10"] → "DV HDR10").
///
/// Pourquoi pas une alternance combinée ?
/// Un seul `find` retourne le *premier* match — sur "DV HDR10" il renvoie
/// "DV" et s'arrête. `find_iter` collecte les deux indépendamment de leur
/// ordre dans la chaîne, ce qui gère aussi "HDR10 DV" ou "HLG DV".
///
/// Tokens reconnus (du plus spécifique au plus générique) :
///   HDR10+, HDR10, HLG, DV, DoVi, Dolby Vision, HDR, SDR
///
/// Le suffix `(?:[^A-Za-z0-9]|$)` empêche de matcher "HDR" dans "HDRip".
pub static HDR: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(
        r"(?i)\b(HDR10\+|HDR10|HLG|DV|DoVi|Dolby[\s.]?Vision|HDR|SDR)(?:[^A-Za-z0-9]|$)"
    ).unwrap()
);

/// Collecte tous les tokens HDR d'un nom de fichier et retourne un tag
/// canonique normalisé.
///
/// Exemples :
/// ```
/// "DV HDR10"        → "DV HDR10"
/// "HDR10 DV"        → "DV HDR10"   (ordre normalisé : DV toujours en premier)
/// "Dolby Vision"    → "DV"
/// "DoVi HDR10+"     → "DV HDR10+"
/// "HLG"             → "HLG"
/// "HDR10"           → "HDR10"
/// "SDR"             → ""           (SDR = pas de tag)
/// ""                → ""
/// ```
pub fn normalize_hdr_tags(name: &str) -> String {
    // Collecte tous les tokens bruts trouvés par find_iter
    let raw_tags: Vec<String> = HDR
        .captures_iter(name)
        .filter_map(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .collect();

    if raw_tags.is_empty() {
        return String::new();
    }

    // Normalise chaque token vers une forme canonique
    let mut has_dv      = false;
    let mut has_hdr10p  = false;
    let mut has_hdr10   = false;
    let mut has_hlg     = false;
    let mut has_hdr     = false;
    // SDR et tokens inconnus sont ignorés (pas de tag dans le nom de sortie)

    for raw in &raw_tags {
        let up = raw.to_uppercase().replace([' ', '.'], "");
        match up.as_str() {
            "DV" | "DOVI" | "DOLBYVISION" => has_dv     = true,
            "HDR10+"                       => has_hdr10p = true,
            "HDR10"                        => has_hdr10  = true,
            "HLG"                          => has_hlg    = true,
            "HDR"                          => has_hdr    = true,
            _                              => {}   // SDR ou inconnu → ignoré
        }
    }

    // Combinaisons canoniques — ordre : DV d'abord, puis le format de base
    match (has_dv, has_hdr10p, has_hdr10, has_hlg, has_hdr) {
        (true,  true,  _,     _,     _    ) => "DV HDR10+".to_string(),
        (true,  _,     true,  _,     _    ) => "DV HDR10".to_string(),
        (true,  _,     _,     true,  _    ) => "DV HLG".to_string(),
        (true,  _,     _,     _,     true ) => "DV HDR".to_string(),
        (true,  _,     _,     _,     _    ) => "DV".to_string(),
        (_,     true,  _,     _,     _    ) => "HDR10+".to_string(),
        (_,     _,     true,  _,     _    ) => "HDR10".to_string(),
        (_,     _,     _,     true,  _    ) => "HLG".to_string(),
        (_,     _,     _,     _,     true ) => "HDR".to_string(),
        _                                   => String::new(),
    }
}

// ── Édition spéciale ─────────────────────────────────────────────────────

pub static EDITION: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(
        r"(?i)\b(Extended(?:[.\s]Cut)?|Director[s']?(?:[.\s]Cut)?|IMAX|Theatrical|Unrated|REMASTERED|Criterion|Anniversary(?:[.\s]Edition)?)\b"
    ).unwrap()
);

// ── Année ────────────────────────────────────────────────────────────────
// FIX : couverture étendue jusqu'à 2099 (au lieu de 2029 dans l'original)
// Exige des word-boundaries pour éviter de matcher dans 1920x1080 etc.
/// Année de production/sortie — word-boundary simple.
/// La logique film/série est gérée dans filename.rs.
pub static YEAR: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"\b(19[0-9]{2}|20[0-9]{2})\b").unwrap()
);

/// Année entre parenthèses — signal fort que c'est un film.
pub static YEAR_PAREN: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"\((19[0-9]{2}|20[0-9]{2})\)").unwrap()
);

/// Année dans un contexte séparateur (ctx_avant)(année)(ctx_après) — sans lookahead.
/// groupe 1 = séparateur avant, groupe 2 = année, groupe 3 = séparateur après.
/// Permet replace_all en restituant "$1$3" pour supprimer l'année sans le contexte.
pub static YEAR_CTX: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"([\s._\(\[\-])(19[0-9]{2}|20[0-9]{2})([\s._\)\]\-])").unwrap()
);

// ── Fournisseurs de streaming ─────────────────────────────────────────────

/// Identifiants connus des plateformes de streaming (sigles)
pub static PROVIDER: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(
        r"(?i)\b(ADN|WKN|AMZN|NF|CR|DSNP|DNSP|HMAX|PCOK|ATVP|SHO|STAN|BCORE|HULU|PMTP|TVNZ|CRAV|TVING|WOWOW|ABEMA|HIDIVE|DRTV|RTBF|ARTE)\b"
    ).unwrap()
);

// ── Codecs audio ─────────────────────────────────────────────────────────

/// AAC avec éventuellement canal (ex: AAC 2.0, AAC5.1)
pub static AUDIO_CODEC_NUM: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\bAAC[\s._]?\d+(?:[.\s]\d+)?").unwrap()
);
/// Dolby Digital (DD/DDP/DD+) avec éventuel canal
pub static AUDIO_DD: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\bDDP?[\s._]?\d+(?:[.\s]\d+)?|\bDDP?\+").unwrap()
);
/// DTS et ses variantes (DTS-HD, DTS-MA, DTS:X)
pub static AUDIO_DTS: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\bDTS(?:[-:\s]?(?:HD|MA|X))?\b").unwrap()
);

// ── H.264 / H.265 — espaces parasites ────────────────────────────────────

pub static H264_SPACE: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\bH\s+264\b").unwrap()
);
pub static H265_SPACE: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\bH\s+265\b").unwrap()
);

// ── Nettoyage titre ───────────────────────────────────────────────────────

/// Suppression contenu entre crochets/parenthèses (non-greedy pour éviter
/// de supprimer deux groupes distincts en un seul match).
pub static BRACKETS: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"[\[\(][^\]\)]*?[\]\)]").unwrap()
);
/// Groupe de release en fin de nom (ex: " - SubsPlease" ou " -RARBG")
/// Groupe de release classique en fin de nom : " - SubsPlease" ou " -RARBG"
pub static TRAIL_GROUP: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\s*-\s*([A-Za-z0-9][A-Za-z0-9_.]{2,})\s*$").unwrap()
);
/// Groupe de release collé à un tag technique en fin de nom : "x264-Tsundere-Raws$"
/// Capture : [A-Za-z0-9]+ tiret groupe en fin de chaîne.
pub static TRAIL_GROUP_STUCK: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)[A-Za-z0-9]+-([A-Za-z][A-Za-z0-9_.-]{2,})\s*$").unwrap()
);
/// Tirets/tirets cadratins résiduels en fin de chaîne
pub static TRAIL_DASH: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"\s*[-–]+\s*$").unwrap()
);
/// Espaces multiples consécutifs
pub static MULTI_SP: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"\s{2,}").unwrap()
);
/// Présence du tag MULTI dans le nom original
pub static MULTI_TAG: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\bMULTI\b").unwrap()
);
/// Contexte OVA/Spécial pour ne pas confondre un numéro d'épisode trailing
pub static OVA_CONTEXT: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\b(Special|OVA|OAD|ONA)\s+\d{1,4}\s*$").unwrap()
);
/// Apostrophes typographiques normalisées
pub static APOSTROPHE: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new("['\u{2018}\u{2019}`\u{00B4}]").unwrap()
);
/// Caractères interdits dans les noms de fichiers Windows
pub static INVALID_CHARS: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r#"[<>:"/\\|?*\x00-\x1F]"#).unwrap()
);

// ── Progression ffmpeg (progress pipe) ───────────────────────────────────

/// Numéro de frame courant dans la sortie progress de ffmpeg (`frame=<n>`)
pub static FFMPEG_FRAME: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"^frame=(\d+)$").unwrap()
);
/// Vitesse d'encodage (`speed=1.23x`)
pub static FFMPEG_SPEED: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"speed=\s*([\d.]+)x").unwrap()
);
/// Temps de sortie en microsecondes (`out_time_us=<n>`)
pub static FFMPEG_OUT_TIME: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"^out_time_us=(\d+)$").unwrap()
);

// ── Tags techniques à supprimer ───────────────────────────────────────────
// FIX : les doublons (sources, providers) ont été retirés — ils sont déjà
// gérés par leurs propres regex. Cela évite des suppressions prématurées.

pub static TECH_TAGS: Lazy<Vec<InnerRegex>> = Lazy::new(|| {
    const TAGS: &[&str] = &[
        // Langue / multi
        "MULTI", "MULTi", "MULTi4", "MULTi9", "Multi4", "Multi9",
        "VOSTFR", "VOSTKR", "VOSTCH", "VFF", "FRENCH", "TRUEFRENCH",
        "SUBFRENCH", "SUBFR", "VO", "VF", "VF2", "VFi",
        // Codecs vidéo
        "x264", "x265", "H265", "H264", "H.264", "H.265",
        "HEVC", "AVC", "AV1", "VP9", "VC-1", "XviD", "DivX",
        "H 264", "H 265",
        // Profondeur colorimétrique
        "10BIT", "10bit", "10bits", "8bit",
        // Codecs audio
        "E-AC-3", "EAC3", "TrueHD", "Atmos", "Dolby Atmos", "Dolby",
        "FLAC", "OPUS", "MP3", "MP2", "AC3", "AAC",
        // Canaux audio (isolés après suppression des codecs)
        "5 1", "7 1", "2 0",
        // Méta release
        "REPACK", "PROPER", "RERIP", "REMUX", "INTERNAL",
        "FANSUB", "END", "Special",
        // SDR (HDR géré séparément)
        "SDR",
    ];
    TAGS.iter()
        .map(|tag| InnerRegex::new(&format!(r"(?i)\b{}\b", regex::escape(tag))).unwrap())
        .collect()
});

// ── Groupes de release connus ─────────────────────────────────────────────

pub static KNOWN_GROUPS: Lazy<Vec<InnerRegex>> = Lazy::new(|| {
    const GROUPS: &[&str] = &[
        // Groupes fansub anime
        "Tsundere-Raws", "TsundereRaws", "SubsPlease", "HorribleSubs",
        "Erai-raws", "EraiRaws", "Judas", "GundamGuy", "LostYears",
        "Aoi-Project", "DameDesuYo", "Asenshi", "Commie", "FFF",
        "Punisher694", "YuiSubs", "Anime-Koi", "DeadFish", "GJM",
        "Reaktor", "Hakata-Ramen", "Nep-Blanc", "Cleo", "Doki",
        "Underwater", "Vivid", "UTW", "Chihiro", "Coalgirls",
        "Beatrice-Raws", "SCY", "Reinforce", "Setsugen", "Nyanpasu",
        "Kaleido-Subs", "Aesenshi", "FLan", "ANE", "Licca",
        "PAS", "Pog42", "GST", "Drag", "Crow",
        // Groupes P2P généralistes
        "YIFY", "YTS", "RARBG", "FGT", "EVO", "SPARKS", "GECKOS",
        "ROVERS", "FLEET", "DRONES", "PSYCHD", "DEFLATE", "CMRG",
        "iFT", "TEPES", "TOMMY", "NTb", "playWEB", "APEX",
        "SMURF", "SUGAR", "KiNGS", "TBS", "FLUX", "GOSSIP",
        "MZABI", "WATCHER", "JETIX", "HONE", "WELP", "GNOME",
        "BONSAI", "HORIZON", "CAKES", "NAISU", "LAZY",
        "TURG", "KOGi", "BLUDV", "YELLO", "VARYG", "KOGI",
        "EDPH", "MIXED", "BLUEFOX", "XZVCD", "BUYMORE", "PLZPROPER",
        "Slay3R", "Slay3r", "EXTREME", "PREM", "ACOOL", "SYNAPSE",
        "BLURAY-GROUP", "FRENSHiCK", "PiMPAND",
        "QCF", "WARPZONE", "STVFRENCH", "TFSERIES", "FRENCH-RELEASE",
        "HARIBO", "STVFR", "FREEFR", "MiXED", "VF2HD",
        "NOTEAM", "TEAM-W4F", "W4F", "ZT", "SH",
        "HDForever", "HDFever", "K7", "PHD", "ATeam", "ATB",
        "THREESOME", "NEMO", "NOTAG",
        "AMIABLE", "BATV", "BiPOLAR", "BRMP",
        "CADAVER", "CHD", "CRiMSON", "CTRLweb", "D-Z0N3",
        "DON", "EbP", "ESiR", "FiHTV", "FLAME",
        "FoV", "FTW-HD", "GHOULS", "GiMCHi", "GROUP",
        "HiDt", "HiSD", "HDS", "INFLATE", "iNK",
        "JYK", "KASHMiR", "KiMCHI", "LCHD", "LEGi0N",
        "LiBRARiANS", "LPCM", "MA", "MaNiAcs", "MBLURAYFR",
        "MCR", "MkvCage", "MLRips", "MTeam", "NTG",
        "NTR", "OMFUG", "ORENJI", "OZL", "PENN",
        "PSA", "QOQ", "RCDiVX", "REGRET", "RENCODE",
        "RightSizeMe", "SADPANDA", "SHITBOX", "SiMPLE",
        "SiNNERS", "SKiDROW", "SLOMO", "SNOW",
        "SPEC", "STRIFE", "STUTTERSHIT", "SUNSCREEN", "SURCODE",
        "SWTYBLZ", "TERMINAL", "THORA", "THRONE",
        "TM", "TRiToN", "TRUSTNO1", "URANiME", "USAViD",
        "VETO", "VietHD", "ViSUM", "VORE", "W4NK3R",
        "WAF", "WiKi", "WMD", "WOMO",
        "ZQ", "dAV1nci", "lvl99",
    ];
    GROUPS.iter()
        .flat_map(|g| {
            let escaped = regex::escape(g);
            // Pattern 1 : séparateur optionnel + groupe + séparateur optionnel en fin
            let p1 = InnerRegex::new(&format!(r"(?i)\s*[-–_]?\s*{}\s*[-–]?\s*$", escaped)).unwrap();
            // Pattern 2 : collé après un token alphanumérique (ex: "x264-Tsundere-Raws")
            // Sans lookbehind : on matche [alnum]-groupe en fin de chaîne
            let p2 = InnerRegex::new(&format!(r"(?i)[A-Za-z0-9]+-{}\s*$", escaped)).unwrap();
            vec![p1, p2]
        })
        .collect()
});