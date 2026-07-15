import type {
  CleanedName,
  NamingOptions,
  SeasonEpisodeFormat,
  StreamInfo,
  AppFile,
  AudioMode,
  VideoMode,
  TagId,
} from "./types";

// ─── Constantes ───────────────────────────────────────────────────────────────

export const SEASON_EPISODE_FORMATS: {
  value: SeasonEpisodeFormat;
  label: string;
  example: string;
}[] = [
  { value: "S01E01", label: "S01E01", example: "S01E01" },
  { value: "S1E01",  label: "S1E01",  example: "S1E01"  },
  { value: "S1 E01", label: "S1 E01", example: "S1 E01" },
  { value: "1x01",   label: "1x01",   example: "1x01"   },
];

export const DEFAULT_TAG_ORDER: TagId[] = [
  "title",
  "year",
  "se",
  "audio",
  "resolution",
  "provider",
  "source",
  "hdr", 
  "codec",
  "bitdepth",
  "audioCodec",
  "team",
  "japver",
];

export const TAG_LABELS: Record<TagId, string> = {
  title:      "Titre",
  year:       "Année",
  se:         "Saison/Épisode",
  audio:      "Tag audio (VOSTFR, VF…)",
  resolution: "Résolution",
  provider:   "Provider",
  source:     "Source (BluRay, WEB-DL…)",
  hdr:        "HDR (HDR10, HLG, DV…)",
  codec:      "Codec vidéo (H265)",
  bitdepth:   "Profondeur (10bit)",
  audioCodec: "Codec audio (AAC…)",
  team:       "Team",
  japver:     "(Japanese ver.)",
};

export const LANG_NAMES: Record<string, string> = {
  fre: "Français",
  eng: "Anglais",
  jpn: "Japonais",
  ger: "Allemand",
  spa: "Espagnol",
  kor: "Coréen",
  ita: "Italien",
  por: "Portugais",
  rus: "Russe",
  chi: "Chinois",
  und: "Indéfini",
};

/** Ordre par défaut — peut être remplacé par `prefs.langOrder`. */
export const LANG_ORDER = [
  "fre", "eng", "jpn", "kor", "ger", "spa", "ita", "por", "rus", "chi", "und",
];

/**
 * Trie un tableau de codes de langue selon un ordre de priorité configurable.
 * Les langues absentes de `order` sont placées à la fin, triées alphabétiquement.
 */
export function sortedLangs(langs: string[], order: string[] = LANG_ORDER): string[] {
  return [...langs].sort((a, b) => {
    const ia = order.indexOf(a);
    const ib = order.indexOf(b);
    if (ia === -1 && ib === -1) return a.localeCompare(b);
    if (ia === -1) return 1;
    if (ib === -1) return -1;
    return ia - ib;
  });
}

const COPY_BY_DEFAULT = ["aac", "opus", "ac3", "eac3", "mp3", "flac"];

const SE_RANGE_RE = /^S(\d{2,})E(\d{2,})(?:-E(\d{2,}))?$/;

// ─── Helpers internes ─────────────────────────────────────────────────────────

function pad2(n: number): string {
  return n.toString().padStart(2, "0");
}

export function formatDuration(secs: number): string {
  if (!secs || secs <= 0) return "";
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = Math.floor(secs % 60);
  if (h > 0)
    return `${h}h ${String(m).padStart(2, "0")}m ${String(s).padStart(2, "0")}s`;
  if (m > 0) return `${m}m ${String(s).padStart(2, "0")}s`;
  return `${s}s`;
}

export function formatMb(mb: number): string {
  if (mb >= 1024) return `${(mb / 1024).toFixed(2)} GB`;
  return `${mb.toFixed(0)} MB`;
}

export function joinPath(dir: string, name: string): string {
  const d = dir.replace(/[\\\/]+$/, "");
  return `${d}\\${name}`;
}

// ─── Lang helpers ─────────────────────────────────────────────────────────────

export function langName(code: string): string {
  return LANG_NAMES[code] ?? code.toUpperCase();
}

export function codecDefault(codecName: string): string {
  return COPY_BY_DEFAULT.includes(codecName.toLowerCase()) ? "copy" : "aac";
}

// ─── Formatage saison/épisode ─────────────────────────────────────────────────

export function formatSeasonEpisode(
  raw: string,
  format: SeasonEpisodeFormat,
): string {
  if (!raw) return raw;
  const m = raw.match(SE_RANGE_RE);
  if (!m) return raw;
  const season = parseInt(m[1], 10);
  const ep1 = parseInt(m[2], 10);
  const ep2 = m[3] !== undefined ? parseInt(m[3], 10) : null;
  switch (format) {
    case "S1E01":
      return ep2 !== null
        ? `S${season}E${pad2(ep1)}-E${pad2(ep2)}`
        : `S${season}E${pad2(ep1)}`;
    case "S1 E01":
      return ep2 !== null
        ? `S${season} E${pad2(ep1)}-E${pad2(ep2)}`
        : `S${season} E${pad2(ep1)}`;
    case "1x01":
      return ep2 !== null
        ? `${season}x${pad2(ep1)}-${pad2(ep2)}`
        : `${season}x${pad2(ep1)}`;
    case "S01E01":
    default:
      return raw;
  }
}

// ─── Tag audio ────────────────────────────────────────────────────────────────

export function computeTag(
  fileAudioLangs: string[],
  fileSubLangs: string[],
  selAudio: Set<string>,
  selSubs: Set<string>,
): string {
  const audio = fileAudioLangs.filter((l) => selAudio.has(l));
  const subs  = fileSubLangs.filter((l) => selSubs.has(l));
  if (audio.length === 0 || audio.every((l) => l === "und")) return "";
  if (audio.length > 1 || subs.length > 1) return "MULTI";
  switch (audio[0]) {
    case "fre": return "VF";
    case "eng": return "VO";
    case "jpn": return "VOSTFR";
    case "kor": return "VOSTKR";
    case "chi": return "VOSTCH";
    case "ger": return "GER";
    case "spa": return "SPA";
    case "ita": return "ITA";
    case "por": return "POR";
    case "rus": return "RUS";
    default:    return audio[0].toUpperCase();
  }
}

export function computeAudioTag(
  streams: StreamInfo[],
  selAudio: Set<string>,
  audioMode: AudioMode,
): string {
  if (audioMode === "reencode") return "AAC";
  const codecs = new Set(
    streams
      .filter((s) => s.codec_type === "audio" && selAudio.has(s.language))
      .map((s) => s.codec_name.toUpperCase()),
  );
  if (codecs.size === 0) return "AAC";
  return [...codecs].sort().join("-");
}

export function computeVideoCodecTag(
  streams: StreamInfo[],
  videoMode: VideoMode,
  codecFormat: string,
): string {
  if (videoMode === "encode") return codecFormat;
  const video = streams.find((s) => s.codec_type === "video");
  if (!video) return codecFormat;
  const MAP: Record<string, string> = {
    hevc:        "H265",
    h265:        "H265",
    h264:        "H264",
    avc:         "H264",
    av1:         "AV1",
    vp9:         "VP9",
    vp8:         "VP8",
    mpeg2video:  "MPEG2",
    mpeg4:       "MPEG4",
    xvid:        "XviD",
    divx:        "DivX",
    theora:      "Theora",
    vc1:         "VC-1",
    wmv3:        "WMV",
  };
  return MAP[video.codec_name.toLowerCase()] ?? video.codec_name.toUpperCase();
}

// ─── Construction du nom de sortie ───────────────────────────────────────────

export function buildOutputName(
  cleaned: CleanedName,
  tag: string,
  seFormat: SeasonEpisodeFormat = "S01E01",
  audioTag: string = "AAC",
  tagOrder: TagId[] = DEFAULT_TAG_ORDER,
  team: string = "",
  options: NamingOptions = {},
  hdrOverride?: string,
): string {
  const {
    disabledTags    = new Set<TagId>(),
    resolutionCase  = "upper",
    titleCase       = "original",
    codecFormat     = "H265",
    sourceCase      = "original",
    yearFormat      = "parentheses",
    webSourceFormat = "WEB-DL",
    tagSeparator    = " ",
    providerCase    = "upper",
    keepJapaneseVer = false,
  } = options;

  const applyTitleCase = (t: string): string => {
    switch (titleCase) {
      case "upper": return t.toUpperCase();
      case "lower": return t.toLowerCase();
      case "title": return t.replace(/\b\w/g, (c) => c.toUpperCase());
      default:      return t;
    }
  };

  const applyResCase = (r: string): string =>
    resolutionCase === "lower" ? r.toLowerCase() : r.toUpperCase();

  const applyWebFormat = (s: string): string => {
    switch (webSourceFormat) {
      case "WEBDL":
        return s.replace(/[-\s]/g, "").toUpperCase();
      case "Web-DL": {
        const map: Record<string, string> = {
          "WEB-DL": "Web-DL",
          "WEBRip": "Web-Rip",
          "WEB":    "Web",
        };
        return map[s] ?? s;
      }
      default:
        return s;
    }
  };

  const applySourceCase = (s: string): string => {
    if (/^WEB/i.test(s)) return applyWebFormat(s);
    switch (sourceCase) {
      case "upper": return s.toUpperCase();
      case "lower": return s.toLowerCase();
      default:      return s;
    }
  };

  const yearValue = cleaned.year
    ? (yearFormat === "parentheses" ? `(${cleaned.year})` : cleaned.year)
    : "";

  const stripDuplicateYear = (t: string): string => {
    if (!cleaned.year) return t;
    const escapedYear = cleaned.year.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    return t
      .replace(new RegExp(`[\\s.,_-]*\\(?${escapedYear}\\)?\\s*$`), "")
      .trim();
  };

  const titleBase = stripDuplicateYear(applyTitleCase(cleaned.title));

  const values: Record<TagId, string> = {
    title:      titleBase,
    year:       yearValue,
    se:         formatSeasonEpisode(cleaned.season_episode, seFormat),
    audio:      tag,
    resolution: applyResCase(cleaned.resolution),
    provider:   providerCase === "hidden" ? "" :
                providerCase === "lower"  ? cleaned.provider.toLowerCase() :
                cleaned.provider.toUpperCase(),
    source:     applySourceCase(cleaned.source),
    hdr: hdrOverride ?? cleaned.hdr ?? "",
    codec:      codecFormat,
    bitdepth:   "10bit",
    audioCodec: audioTag,
    team:       team.trim(),
    japver:     keepJapaneseVer ? "(Japanese ver.)" : "",
  };

  const sep = tagSeparator;
  const escapedSep = sep.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return tagOrder
    .filter((id) => !disabledTags.has(id))
    .map((id) => values[id])
    .filter(Boolean)
    .map((v) => (sep === " " ? v : v.replace(/\s+/g, sep)))
    .join(sep)
    .replace(new RegExp(`${escapedSep}{2,}`, "g"), sep)
    .trim();
}

export function applySeFormat(
  file: AppFile,
  seFormat: SeasonEpisodeFormat,
  tagOrder: TagId[] = DEFAULT_TAG_ORDER,
  team: string = "",
  options: NamingOptions = {},
): string {
  if (file.cleaned) {
    const tag =
      file.output_name.match(
        /\b(VF|VO|VOSTFR|VOSTA|VOSTKR|VOSTCH|MULTI|GER|SPA|ITA|POR|RUS)\b/,
      )?.[1] ?? "";
    const AUDIO_CODECS = ["AAC", "AC3", "EAC3", "DTS", "FLAC", "OPUS", "MP3", "TrueHD"];
    const audioTag =
      AUDIO_CODECS.find((c) =>
        new RegExp(`\\b${c}\\b`, "i").test(file.output_name),
      ) ?? "AAC";
    return buildOutputName(
      file.cleaned,
      tag,
      seFormat,
      audioTag,
      tagOrder,
      team,
      options,
      file.hdr_format,
    );
  }
  return file.output_name.replace(
    /S(\d{1,2})E(\d{2,})(?:-E(\d{2,}))?/gi,
    (_, s, e1, e2) => {
      const raw = e2
        ? `S${s.padStart(2, "0")}E${e1}-E${e2}`
        : `S${s.padStart(2, "0")}E${e1}`;
      return formatSeasonEpisode(raw, seFormat);
    },
  );
}