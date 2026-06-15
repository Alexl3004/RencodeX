//! Expressions régulières pré-compilées pour l'analyse de noms de fichiers

pub use regex::Regex;  // Ajout important - exporter Regex publiquement

use once_cell::sync::Lazy;
use regex::Regex as InnerRegex;  // Renommer pour éviter les conflits

// ── Patterns saison/épisode ───────────────────────────────────────────────
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
pub static EP_TRAIL: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)[-\s](\d{1,4})\s*$").unwrap()
);
pub static OVA_SPECIAL: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\s*[-–]?\s*\b(OVA|OAD|ONA)[\s._-]?(\d*)\b|\bSpecial[\s._-](\d+)\b").unwrap()
);

// ── Résolution ────────────────────────────────────────────────────────────
pub static RESOLUTION: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\b(2160p|4K|UHD|1080p|FHD|720p|HD|480p|SD)\b").unwrap()
);
pub static RESOLUTION_NUM: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)(\d{3,4})x(\d{3,4})").unwrap()
);

// ── Sources vidéo ─────────────────────────────────────────────────────────
pub static SRC_WEBDL: Lazy<InnerRegex> = Lazy::new(|| InnerRegex::new(r"(?i)\bWEB-DL\b").unwrap());
pub static SRC_WEBRIP: Lazy<InnerRegex> = Lazy::new(|| InnerRegex::new(r"(?i)\bWEBRip\b").unwrap());
pub static SRC_BLURAY: Lazy<InnerRegex> = Lazy::new(|| InnerRegex::new(r"(?i)\bBluRay\b").unwrap());
pub static SRC_BDRIP: Lazy<InnerRegex> = Lazy::new(|| InnerRegex::new(r"(?i)\bBDRip\b").unwrap());
pub static SRC_BRRIP: Lazy<InnerRegex> = Lazy::new(|| InnerRegex::new(r"(?i)\bBRRip\b").unwrap());
pub static SRC_HDTV: Lazy<InnerRegex> = Lazy::new(|| InnerRegex::new(r"(?i)\bHDTV\b").unwrap());
pub static SRC_DVDRIP: Lazy<InnerRegex> = Lazy::new(|| InnerRegex::new(r"(?i)\bDVDRip\b").unwrap());
pub static SRC_HDRIP: Lazy<InnerRegex> = Lazy::new(|| InnerRegex::new(r"(?i)\bHDRip\b").unwrap());
pub static SRC_WEB: Lazy<InnerRegex> = Lazy::new(|| InnerRegex::new(r"(?i)\bWEB\b").unwrap());

// ── HDR ───────────────────────────────────────────────────────────────────
pub static HDR: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\b(HDR10\+|HDR10|HDR|DV|DoVi|Dolby[.\s]?Vision|HLG|SDR)\b").unwrap()
);

// ── Édition spéciale ─────────────────────────────────────────────────────
pub static EDITION: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\b(Extended(?:[.\s]Cut)?|Director[s']?(?:[.\s]Cut)?|IMAX|Theatrical|Unrated|REMASTERED)\b").unwrap()
);

// ── Année ────────────────────────────────────────────────────────────────
pub static YEAR: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"\b(19[0-9]{2}|20[0-2][0-9])\b").unwrap()
);

// ── Fournisseurs de streaming ─────────────────────────────────────────────
pub static PROVIDER: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\b(ADN|WKN|AMZN|NF|CR|DSNP|DNSP|HMAX|PCOK|ATVP|SHO|STAN|BCORE|HULU|PMTP|TVNZ|CRAV|TVING|WOWOW|ABEMA|HIDIVE)\b").unwrap()
);

// ── Codecs audio ─────────────────────────────────────────────────────────
pub static AUDIO_CODEC_NUM: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\bAAC[\s._]?\d+(?:[.\s]\d+)?").unwrap()
);
pub static AUDIO_DD: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\bDDP?[\s._]?\d+(?:[.\s]\d+)?|\bDDP?\+").unwrap()
);
pub static AUDIO_DTS: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\bDTS(?:[-\s]?(?:HD|MA|X))?\b").unwrap()
);

// ── H.264 / H.265 ────────────────────────────────────────────────────────
pub static H264_SPACE: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\bH\s+264\b").unwrap()
);
pub static H265_SPACE: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\bH\s+265\b").unwrap()
);

// ── Nettoyage titre ───────────────────────────────────────────────────────
pub static BRACKETS: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"[\[\(][^\]\)]*[\]\)]").unwrap()
);
pub static TRAIL_GROUP: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\s+-\s*([A-Za-z0-9][A-Za-z0-9_.]{3,})$").unwrap()
);
pub static TRAIL_DASH: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"\s*[-–]+\s*$").unwrap()
);
pub static MULTI_SP: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"\s{2,}").unwrap()
);
pub static MULTI_TAG: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\bMULTI\b").unwrap()
);
pub static OVA_CONTEXT: Lazy<InnerRegex> = Lazy::new(||
    InnerRegex::new(r"(?i)\b(Special|OVA|OAD|ONA)\s+\d{1,4}\s*$").unwrap()
);

// ── Tags techniques ───────────────────────────────────────────────────────
pub static TECH_TAGS: Lazy<Vec<InnerRegex>> = Lazy::new(|| {
    const TAGS: &[&str] = &[
        "MULTI", "MULTi", "MULTi4", "MULTi9", "Multi4", "Multi9",
        "VOSTFR", "VOSTKR", "VOSTCH", "VFF", "FRENCH", "TRUEFRENCH",
        "SUBFRENCH", "SUBFR", "VO", "VF", "VF2", "VFi",
        "x264", "x265", "H265", "H264", "H.264", "H.265",
        "HEVC", "AVC", "AV1", "VP9", "VC-1", "XviD", "DivX",
        "H 264", "H 265", "10BIT",
        "E-AC-3", "EAC3", "TrueHD", "Atmos", "Dolby Atmos", "Dolby",
        "FLAC", "OPUS", "MP3", "MP2", "AC3", "AAC", "5 1", "7 1", "2 0",
        "WEB-DL", "WEBRip", "BluRay", "BDRip", "HDRip", "HDTV", "DVDRip", "BRRip",
        "NF", "AMZN", "CR", "ADN",
        "10bit", "10bits", "8bit",
        "REPACK", "PROPER", "RERIP", "REMUX", "INTERNAL",
        "FANSUB", "SDR", "END", "Special",
    ];
    TAGS.iter()
        .map(|tag| InnerRegex::new(&format!(r"(?i)\b{}\b", regex::escape(tag))).unwrap())
        .collect()
});

pub static KNOWN_GROUPS: Lazy<Vec<InnerRegex>> = Lazy::new(|| {
    const GROUPS: &[&str] = &[
        "Tsundere-Raws", "TsundereRaws", "SubsPlease", "HorribleSubs",
        "Erai-raws", "EraiRaws", "Judas", "GundamGuy", "LostYears",
        "Aoi-Project", "DameDesuYo", "Asenshi", "Commie", "FFF",
        "Punisher694", "YuiSubs", "Anime-Koi", "DeadFish", "GJM",
        "Reaktor", "Hakata-Ramen", "Nep-Blanc", "Cleo", "Doki",
        "Underwater", "Vivid", "UTW", "Chihiro", "Coalgirls",
        "Beatrice-Raws", "SCY", "Reinforce", "Setsugen", "Nyanpasu",
        "Kaleido-Subs", "Aesenshi", "FLan", "ANE", "Licca",
        "PAS", "Pog42", "GST", "Drag", "Crow",
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
        .map(|g| InnerRegex::new(&format!(r"(?i)\s*[-–_]?\s*{}\s*[-–]?\s*$", regex::escape(g))).unwrap())
        .collect()
});