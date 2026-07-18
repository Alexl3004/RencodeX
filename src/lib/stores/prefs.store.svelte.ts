import { invoke } from "@tauri-apps/api/core";
import type { VideoMode } from "./types";
import type {
  YearFormat,
  SeasonEpisodeFormat,
  TagId,
  ResolutionCase,
  TitleCaseMode,
  CodecFormat,
  SourceCase,
  WebSourceFormat,
  TagSeparator,
  ProviderCase,
  AudioCodec,
  AudioCodecRules,
  MultipassMode,
  ContainerFormat,
  SubExtractFormat,
  SubExtractNaming,
  SubExtractPathMode,
  EncodePreset,
  AudioPreset,
} from "./types";
import { DEFAULT_TAG_ORDER, LANG_ORDER, SEASON_EPISODE_FORMATS } from "./naming";

// ─── Persistance localStorage ─────────────────────────────────────────────────

const RENAMING_KEY        = "rencodeX:renaming";
const ACTIVE_PRESET_KEY   = "rencodeX:activePresetId";
const AUDIO_PRESETS_KEY   = "rencodeX:audioPresets";

// ─── Règles audio réutilisables ───────────────────────────────────────────────

/**
 * Tout copier sans réencodage — idéal pour une passe rapide ou une archive
 * lorsque tous les codecs sources sont déjà compatibles avec le conteneur cible.
 */
export const AUDIO_RULES_COPY_ALL: AudioCodecRules = {
  __default__: { action: "copy", targetCodec: "aac" },
};

/**
 * Mode intelligent : copie les codecs natifs (AAC, AC3, E-AC3, OPUS) et
 * réencode en AAC les formats lourds ou incompatibles (DTS, TrueHD, FLAC…).
 * C'est le comportement recommandé pour la majorité des fichiers.
 */
export const AUDIO_RULES_SMART: AudioCodecRules = {
  __default__: { action: "reencode", targetCodec: "aac" },
  aac:         { action: "copy",     targetCodec: "aac" },
  ac3:         { action: "copy",     targetCodec: "ac3" },
  eac3:        { action: "copy",     targetCodec: "ac3" },
  opus:        { action: "copy",     targetCodec: "opus" },
  dts:         { action: "reencode", targetCodec: "aac" },
  truehd:      { action: "reencode", targetCodec: "aac" },
  flac:        { action: "reencode", targetCodec: "aac" },
  mp3:         { action: "reencode", targetCodec: "aac" },
  vorbis:      { action: "reencode", targetCodec: "aac" },
  pcm_s16le:   { action: "reencode", targetCodec: "aac" },
  pcm_s24le:   { action: "reencode", targetCodec: "aac" },
};

/**
 * Tout réencoder en AAC — utile pour normaliser l'audio de fichiers hétérogènes
 * ou forcer une compatibilité maximale avec les lecteurs.
 */
export const AUDIO_RULES_REENCODE_ALL: AudioCodecRules = {
  __default__: { action: "reencode", targetCodec: "aac" },
  aac:         { action: "reencode", targetCodec: "aac" },
  ac3:         { action: "reencode", targetCodec: "aac" },
  eac3:        { action: "reencode", targetCodec: "aac" },
  opus:        { action: "reencode", targetCodec: "aac" },
  dts:         { action: "reencode", targetCodec: "aac" },
  truehd:      { action: "reencode", targetCodec: "aac" },
  flac:        { action: "reencode", targetCodec: "aac" },
  mp3:         { action: "reencode", targetCodec: "aac" },
  vorbis:      { action: "reencode", targetCodec: "aac" },
  pcm_s16le:   { action: "reencode", targetCodec: "aac" },
  pcm_s24le:   { action: "reencode", targetCodec: "aac" },
};

// ─── Préréglages d'encodage intégrés ─────────────────────────────────────────

export const BUILTIN_PRESETS: EncodePreset[] = [
  {
    id:              "fast",
    label:           "Rapide",
    crf:             30,
    preset:          "p3",
    videoMode:       "encode",
    audioCodecRules: AUDIO_RULES_COPY_ALL,   // tout copier = passe rapide
    audioBitrate:    192,
    spatialAq:       false,
    temporalAq:      false,
    aqStrength:      8,
    multipass:       "disabled",
  },
  {
    id:              "balanced",
    label:           "Équilibré",
    crf:             28,
    preset:          "p5",
    videoMode:       "encode",
    audioCodecRules: AUDIO_RULES_SMART,      // copie AAC/AC3, réencode DTS/FLAC
    audioBitrate:    192,
    spatialAq:       false,
    temporalAq:      false,
    aqStrength:      8,
    multipass:       "disabled",
  },
  {
    id:              "quality",
    label:           "Haute qualité",
    crf:             24,
    preset:          "p7",
    videoMode:       "encode",
    audioCodecRules: AUDIO_RULES_SMART,
    audioBitrate:    320,
    spatialAq:       true,
    temporalAq:      true,
    aqStrength:      8,
    multipass:       "qres",
  },
  {
    id:              "archive",
    label:           "Archive",
    crf:             20,
    preset:          "p7",
    videoMode:       "encode",
    audioCodecRules: AUDIO_RULES_COPY_ALL,   // archive = copie sans perte
    audioBitrate:    192,
    spatialAq:       true,
    temporalAq:      true,
    aqStrength:      8,
    multipass:       "fullres",
  },
];

// ─── Préréglages audio intégrés (AudioPreset) ─────────────────────────────────

export const BUILTIN_AUDIO_PRESETS: AudioPreset[] = [
  {
    id:      "audio_copy_all",
    label:   "Tout copier",
    builtin: true,
    rules:   AUDIO_RULES_COPY_ALL,
  },
  {
    id:      "audio_smart",
    label:   "Intelligent",
    builtin: true,
    rules:   AUDIO_RULES_SMART,
  },
  {
    id:      "audio_reencode_all",
    label:   "Tout réencoder",
    builtin: true,
    rules:   AUDIO_RULES_REENCODE_ALL,
  },
];

// ─── Helpers de persistance préréglages audio ─────────────────────────────────

function loadCustomAudioPresets(): AudioPreset[] {
  try {
    const raw = localStorage.getItem(AUDIO_PRESETS_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (p): p is AudioPreset =>
        p &&
        typeof p.id === "string" &&
        typeof p.label === "string" &&
        typeof p.rules === "object",
    );
  } catch { return []; }
}

function saveCustomAudioPresets(presets: AudioPreset[]): void {
  try {
    localStorage.setItem(AUDIO_PRESETS_KEY, JSON.stringify(presets));
  } catch { /* storage indisponible */ }
}

// ─── Store préférences ────────────────────────────────────────────────────────

function createPrefsStore() {
  // Encodage
  let crf    = $state(28);
  let preset = $state("p5");

  // Préréglage actif (null = paramètres personnalisés)
  let activePresetId = $state<string | null>(
    typeof localStorage !== "undefined"
      ? localStorage.getItem(ACTIVE_PRESET_KEY)
      : null,
  );

  // Nommage
  let seasonEpisodeFormat = $state<SeasonEpisodeFormat>("S01E01");
  let tagOrder            = $state<TagId[]>([...DEFAULT_TAG_ORDER]);
  let disabledTags        = $state<Set<TagId>>(new Set());
  let resolutionCase      = $state<ResolutionCase>("upper");
  let titleCase           = $state<TitleCaseMode>("original");
  let codecFormat         = $state<CodecFormat>("H265");
  let sourceCase          = $state<SourceCase>("original");
  let yearFormat          = $state<YearFormat>("parentheses");
  let webSourceFormat     = $state<WebSourceFormat>("WEB-DL");
  let tagSeparator        = $state<TagSeparator>(" ");
  let providerCase        = $state<ProviderCase>("upper");
  let team                = $state("");
  let keepJapaneseVer     = $state<boolean>(false);
  let preserveDv          = $state<boolean>(false);
  let langOrder           = $state<string[]>([...LANG_ORDER]);
  let defaultAudioLangs   = $state<string[]>(["fre", "eng", "jpn"]);
  let defaultSubLangs     = $state<string[]>(["fre"]);

  // Audio / vidéo
  let videoMode       = $state<VideoMode>("encode");
  let audioCodecRules = $state<AudioCodecRules>({ ...AUDIO_RULES_SMART });
  let audioBitrate    = $state(192);
  let spatialAq       = $state(false);
  let temporalAq      = $state(false);
  let aqStrength      = $state(8);
  let multipass       = $state<MultipassMode>("disabled");
  let container       = $state<ContainerFormat>("mkv");

  // Préréglages audio custom (chargés depuis localStorage)
  let customAudioPresets = $state<AudioPreset[]>(
    typeof localStorage !== "undefined" ? loadCustomAudioPresets() : [],
  );

  // Extraction sous-titres
  let subExtractFormat     = $state<SubExtractFormat>("srt");
  let subExtractNaming     = $state<SubExtractNaming>("source");
  let subExtractPathMode   = $state<SubExtractPathMode>("source");
  let subExtractCustomPath = $state<string>("");
  let showExtractButton    = $state(true);

  // ─── Timer de debounce ───────────────────────────────────────────────────

  let _persistTimer: ReturnType<typeof setTimeout> | null = null;

  // ─── Snapshot / flush ────────────────────────────────────────────────────

  function snapshot() {
    return {
      crf,
      preset,
      se_format:               seasonEpisodeFormat,
      tag_order:               tagOrder,
      disabled_tags:           [...disabledTags],
      resolution_case:         resolutionCase,
      title_case:              titleCase,
      codec_format:            codecFormat,
      source_case:             sourceCase,
      year_format:             yearFormat,
      web_source_format:       webSourceFormat,
      tag_separator:           tagSeparator,
      provider_case:           providerCase,
      team,
      keep_japanese_ver:       keepJapaneseVer,
      preserve_dv:             preserveDv,
      lang_order:              langOrder,
      default_audio_langs:     defaultAudioLangs,
      default_sub_langs:       defaultSubLangs,
      video_mode:              videoMode,
      audio_codec_rules:       audioCodecRules,
      audio_bitrate:           audioBitrate,
      spatial_aq:              spatialAq,
      temporal_aq:             temporalAq,
      aq_strength:             aqStrength,
      multipass,
      container,
      sub_extract_format:      subExtractFormat,
      sub_extract_naming:      subExtractNaming,
      sub_extract_path_mode:   subExtractPathMode,
      sub_extract_custom_path: subExtractCustomPath,
      show_extract_button:     showExtractButton,
    };
  }

  async function flush() {
    if (_persistTimer) {
      clearTimeout(_persistTimer);
      _persistTimer = null;
    }
    try {
      await invoke("save_encoding_prefs", { prefs: snapshot() });
    } catch (e) {
      console.error(`Erreur sauvegarde préférences : ${e}`);
    }
  }

  function schedule() {
    if (_persistTimer) clearTimeout(_persistTimer);
    _persistTimer = setTimeout(() => flush(), 300);
  }

  // ─── localStorage (renommage, synchrone) ─────────────────────────────────

  function saveRenamingPrefs(): void {
    try {
      localStorage.setItem(RENAMING_KEY, JSON.stringify({
        tag_order:           tagOrder,
        disabled_tags:       [...disabledTags],
        resolution_case:     resolutionCase,
        title_case:          titleCase,
        codec_format:        codecFormat,
        source_case:         sourceCase,
        year_format:         yearFormat,
        web_source_format:   webSourceFormat,
        tag_separator:       tagSeparator,
        provider_case:       providerCase,
        team,
        keepJapaneseVer,
        lang_order:          langOrder,
        default_audio_langs: defaultAudioLangs,
        default_sub_langs:   defaultSubLangs,
      }));
    } catch { /* storage indisponible */ }
  }

  function loadRenamingPrefs(): void {
    try {
      const raw = localStorage.getItem(RENAMING_KEY);
      if (!raw) return;
      const p = JSON.parse(raw) as Record<string, unknown>;

      if (Array.isArray(p.tag_order)) {
        const saved = (p.tag_order as string[]).filter((id) =>
          DEFAULT_TAG_ORDER.includes(id as TagId),
        ) as TagId[];
        const missing = DEFAULT_TAG_ORDER.filter((id) => !saved.includes(id));
        tagOrder = [...saved, ...missing];
      }

      if (Array.isArray(p.disabled_tags))
        disabledTags = new Set(
          (p.disabled_tags as string[]).filter((id) =>
            DEFAULT_TAG_ORDER.includes(id as TagId),
          ) as TagId[],
        );

      if (p.resolution_case === "upper" || p.resolution_case === "lower")
        resolutionCase = p.resolution_case;
      if (["original", "upper", "lower", "title"].includes(p.title_case as string))
        titleCase = p.title_case as TitleCaseMode;
      if (["H265", "H.265", "HEVC"].includes(p.codec_format as string))
        codecFormat = p.codec_format as CodecFormat;
      if (["original", "upper", "lower"].includes(p.source_case as string))
        sourceCase = p.source_case as SourceCase;
      if (p.year_format === "parentheses" || p.year_format === "plain")
        yearFormat = p.year_format;
      else if (typeof p.year_parentheses === "boolean")
        yearFormat = p.year_parentheses ? "parentheses" : "plain";
      if (["WEB-DL", "WEBDL", "Web-DL"].includes(p.web_source_format as string))
        webSourceFormat = p.web_source_format as WebSourceFormat;
      if ([" ", ".", "_"].includes(p.tag_separator as string))
        tagSeparator = p.tag_separator as TagSeparator;
      if (["upper", "lower", "hidden"].includes(p.provider_case as string))
        providerCase = p.provider_case as ProviderCase;
      if (typeof p.team === "string") team = p.team;
      if (typeof p.keepJapaneseVer === "boolean") keepJapaneseVer = p.keepJapaneseVer;
      if (
        Array.isArray(p.lang_order) &&
        (p.lang_order as string[]).every((c) => typeof c === "string" && c.length > 0)
      )
        langOrder = p.lang_order as string[];
      if (
        Array.isArray(p.default_audio_langs) &&
        (p.default_audio_langs as string[]).length > 0
      )
        defaultAudioLangs = p.default_audio_langs as string[];
      if (Array.isArray(p.default_sub_langs))
        defaultSubLangs = p.default_sub_langs as string[];
    } catch { /* JSON invalide */ }
  }

  // ─── Chargement depuis Tauri ──────────────────────────────────────────────

  async function load() {
    try {
      const p = await invoke<{
        crf: number;
        preset: string;
        se_format: string;
        tag_order: string[];
        team: string;
        video_mode?: string;
        audio_codec_rules?: Record<string, { action: string; targetCodec: string }>;
        audio_bitrate: number;
        spatial_aq: boolean;
        temporal_aq: boolean;
        aq_strength: number;
        multipass: string;
        container: string;
        sub_extract_format?: string;
        sub_extract_naming?: string;
        sub_extract_path_mode?: string;
        sub_extract_custom_path?: string;
        show_extract_button?: boolean;
        disabled_tags?: string[];
        resolution_case?: string;
        title_case?: string;
        codec_format?: string;
        source_case?: string;
        year_format?: string;
        year_parentheses?: boolean;
        web_source_format?: string;
        tag_separator?: string;
        provider_case?: string;
        keep_japanese_ver?: boolean;
        preserve_dv?: boolean;
        lang_order?: string[];
        default_audio_langs?: string[];
        default_sub_langs?: string[];
      }>("load_encoding_prefs");

      if (typeof p.crf === "number") crf = p.crf;
      if (p.preset) preset = p.preset;
      if (p.se_format && SEASON_EPISODE_FORMATS.some((f) => f.value === p.se_format))
        seasonEpisodeFormat = p.se_format as SeasonEpisodeFormat;

      if (Array.isArray(p.tag_order)) {
        const saved = (p.tag_order as string[]).filter((id) =>
          DEFAULT_TAG_ORDER.includes(id as TagId),
        ) as TagId[];
        const missing = DEFAULT_TAG_ORDER.filter((id) => !saved.includes(id));
        tagOrder = [...saved, ...missing];
      }
      if (Array.isArray(p.disabled_tags)) {
        const valid = (p.disabled_tags as string[]).filter((id) =>
          DEFAULT_TAG_ORDER.includes(id as TagId),
        );
        disabledTags = new Set(valid as TagId[]);
      }
      if (p.resolution_case === "upper" || p.resolution_case === "lower")
        resolutionCase = p.resolution_case;
      if (["original", "upper", "lower", "title"].includes(p.title_case ?? ""))
        titleCase = p.title_case as TitleCaseMode;
      if (["H265", "H.265", "HEVC"].includes(p.codec_format ?? ""))
        codecFormat = p.codec_format as CodecFormat;
      if (["original", "upper", "lower"].includes(p.source_case ?? ""))
        sourceCase = p.source_case as SourceCase;
      if (p.year_format === "parentheses" || p.year_format === "plain")
        yearFormat = p.year_format;
      else if (typeof p.year_parentheses === "boolean")
        yearFormat = p.year_parentheses ? "parentheses" : "plain";
      if (["WEB-DL", "WEBDL", "Web-DL"].includes(p.web_source_format ?? ""))
        webSourceFormat = p.web_source_format as WebSourceFormat;
      if ([" ", ".", "_"].includes(p.tag_separator ?? ""))
        tagSeparator = p.tag_separator as TagSeparator;
      if (["upper", "lower", "hidden"].includes(p.provider_case ?? ""))
        providerCase = p.provider_case as ProviderCase;
      if (typeof p.team === "string") team = p.team;
      if (typeof p.keep_japanese_ver === "boolean") keepJapaneseVer = p.keep_japanese_ver;
      if (typeof p.preserve_dv === "boolean") preserveDv = p.preserve_dv;
      if (
        Array.isArray(p.lang_order) &&
        (p.lang_order as string[]).every((c) => typeof c === "string" && c.length > 0)
      )
        langOrder = p.lang_order as string[];
      if (Array.isArray(p.default_audio_langs) && (p.default_audio_langs as string[]).length > 0)
        defaultAudioLangs = p.default_audio_langs as string[];
      if (Array.isArray(p.default_sub_langs))
        defaultSubLangs = p.default_sub_langs as string[];
      if (p.video_mode === "encode" || p.video_mode === "copy")
        videoMode = p.video_mode;

      if (p.audio_codec_rules && typeof p.audio_codec_rules === "object") {
        const validActions = new Set(["copy", "reencode"]);
        const validCodecs  = new Set(["aac", "ac3", "opus"]);
        const sanitized: AudioCodecRules = {};
        for (const [k, v] of Object.entries(p.audio_codec_rules)) {
          if (validActions.has(v.action) && validCodecs.has(v.targetCodec)) {
            sanitized[k] = {
              action:      v.action as "copy" | "reencode",
              targetCodec: v.targetCodec as AudioCodec,
            };
          }
        }
        if (Object.keys(sanitized).length > 0) audioCodecRules = sanitized;
      }

      if (typeof p.audio_bitrate === "number") audioBitrate = p.audio_bitrate;
      if (typeof p.spatial_aq === "boolean") spatialAq = p.spatial_aq;
      if (typeof p.temporal_aq === "boolean") temporalAq = p.temporal_aq;
      if (typeof p.aq_strength === "number") aqStrength = p.aq_strength;
      if (
        p.multipass === "disabled" ||
        p.multipass === "qres" ||
        p.multipass === "fullres"
      )
        multipass = p.multipass;
      if (p.container === "mkv" || p.container === "mp4") container = p.container;
      if (p.sub_extract_format === "srt" || p.sub_extract_format === "ass")
        subExtractFormat = p.sub_extract_format;
      if (p.sub_extract_naming === "source" || p.sub_extract_naming === "cleaned")
        subExtractNaming = p.sub_extract_naming;
      if (
        p.sub_extract_path_mode === "source" ||
        p.sub_extract_path_mode === "downloads" ||
        p.sub_extract_path_mode === "custom"
      )
        subExtractPathMode = p.sub_extract_path_mode;
      if (typeof p.sub_extract_custom_path === "string")
        subExtractCustomPath = p.sub_extract_custom_path;
      if (typeof p.show_extract_button === "boolean")
        showExtractButton = p.show_extract_button;
    } catch (e) {
      console.error(`Erreur chargement préférences : ${e}`);
    }
  }

  // ─── Préréglages d'encodage ───────────────────────────────────────────────

  function _markCustom() {
    activePresetId = null;
    try { localStorage.removeItem(ACTIVE_PRESET_KEY); } catch { /* */ }
  }

  function applyPreset(id: string) {
    const p = BUILTIN_PRESETS.find((p) => p.id === id);
    if (!p) return;

    crf             = p.crf;
    preset          = p.preset;
    // Appliquer les règles audio du préréglage (copie profonde)
    audioCodecRules = { ...p.audioCodecRules };
    audioBitrate    = p.audioBitrate;
    spatialAq       = p.spatialAq;
    temporalAq      = p.temporalAq;
    aqStrength      = p.aqStrength;
    multipass       = p.multipass;

    activePresetId = id;
    try { localStorage.setItem(ACTIVE_PRESET_KEY, id); } catch { /* */ }
    schedule();
  }

  // ─── Préréglages audio custom ─────────────────────────────────────────────

  /** Retourne tous les préréglages audio (intégrés + custom). */
  function allAudioPresets(): import("./types").AudioPreset[] {
    return [
      ...BUILTIN_AUDIO_PRESETS,
      ...customAudioPresets,
    ];
  }

  /** Applique les règles d'un préréglage audio (intégré ou custom). */
  function applyAudioPreset(id: string) {
    const p = allAudioPresets().find((x) => x.id === id);
    if (!p) return;
    audioCodecRules = { ...p.rules };
    _markCustom();
    schedule();
  }

  /** Sauvegarde les règles actuelles sous un nouveau préréglage audio custom. */
  function saveAudioPreset(label: string): AudioPreset {
    const id = `audio_custom_${Date.now()}`;
    const newPreset: AudioPreset = {
      id,
      label: label.trim(),
      builtin: false,
      rules: { ...audioCodecRules },
    };
    customAudioPresets = [...customAudioPresets, newPreset];
    saveCustomAudioPresets(customAudioPresets);
    return newPreset;
  }

  /** Renomme un préréglage audio custom. */
  function renameAudioPreset(id: string, newLabel: string) {
    customAudioPresets = customAudioPresets.map((p) =>
      p.id === id ? { ...p, label: newLabel.trim() } : p,
    );
    saveCustomAudioPresets(customAudioPresets);
  }

  /** Supprime un préréglage audio custom (les intégrés sont protégés). */
  function deleteAudioPreset(id: string) {
    customAudioPresets = customAudioPresets.filter((p) => p.id !== id);
    saveCustomAudioPresets(customAudioPresets);
  }

  // ─── Setters ──────────────────────────────────────────────────────────────

  function setCrf(value: number)            { crf = value;          _markCustom(); schedule(); }
  function setPreset(value: string)         { preset = value;       _markCustom(); schedule(); }
  function setVideoMode(v: VideoMode)       { videoMode = v;        _markCustom(); schedule(); }
  function setAudioCodecRule(sourceCodec: string, rule: AudioCodecRules[string]) {
    audioCodecRules = { ...audioCodecRules, [sourceCodec]: rule };
    _markCustom(); schedule();
  }
  function setAudioBitrate(v: number)       { audioBitrate = v;     _markCustom(); schedule(); }
  function setSpatialAq(v: boolean)         { spatialAq = v;        _markCustom(); schedule(); }
  function setTemporalAq(v: boolean)        { temporalAq = v;       _markCustom(); schedule(); }
  function setAqStrength(v: number)         { aqStrength = v;       _markCustom(); schedule(); }
  function setMultipass(v: MultipassMode)   { multipass = v;        _markCustom(); schedule(); }
  function setContainer(v: ContainerFormat) { container = v;        schedule(); }

  function setSubExtractFormat(v: SubExtractFormat)    { subExtractFormat = v;     schedule(); }
  function setSubExtractNaming(v: SubExtractNaming)    { subExtractNaming = v;     schedule(); }
  function setSubExtractPathMode(v: SubExtractPathMode){ subExtractPathMode = v;   schedule(); }
  function setSubExtractCustomPath(v: string)          { subExtractCustomPath = v; schedule(); }
  function setShowExtractButton(v: boolean)            { showExtractButton = v;    schedule(); }

  function _namingChanged() { saveRenamingPrefs(); schedule(); }

  function setSeasonEpisodeFormat(v: SeasonEpisodeFormat) {
    seasonEpisodeFormat = v; _namingChanged();
  }
  function setTagOrder(v: TagId[])               { tagOrder = v;         _namingChanged(); }
  function setDisabledTags(v: Set<TagId>)        { disabledTags = v;     _namingChanged(); }
  function setResolutionCase(v: ResolutionCase)  { resolutionCase = v;   _namingChanged(); }
  function setTitleCase(v: TitleCaseMode)        { titleCase = v;        _namingChanged(); }
  function setCodecFormat(v: CodecFormat)        { codecFormat = v;      _namingChanged(); }
  function setSourceCase(v: SourceCase)          { sourceCase = v;       _namingChanged(); }
  function setYearFormat(v: YearFormat)          { yearFormat = v;       _namingChanged(); }
  function setWebSourceFormat(v: WebSourceFormat){ webSourceFormat = v;  _namingChanged(); }
  function setTagSeparator(v: TagSeparator)      { tagSeparator = v;     _namingChanged(); }
  function setProviderCase(v: ProviderCase)      { providerCase = v;     _namingChanged(); }
  function setTeam(v: string)                    { team = v;             _namingChanged(); }
  function setKeepJapaneseVer(v: boolean)        { keepJapaneseVer = v;  _namingChanged(); }
  function setPreserveDv(v: boolean)             { preserveDv = v;       schedule(); }
  function setDefaultAudioLangs(v: string[])     { defaultAudioLangs = v; _namingChanged(); }
  function setDefaultSubLangs(v: string[])       { defaultSubLangs = v;   _namingChanged(); }

  function setLangOrder(v: string[]) {
    const seen = new Set<string>();
    const cleaned = v.filter((c) => c && !seen.has(c) && seen.add(c));
    langOrder = cleaned.length > 0 ? cleaned : [...LANG_ORDER];
    _namingChanged();
  }

  function resetFormatOptions() {
    resolutionCase      = "upper";
    titleCase           = "original";
    codecFormat         = "H265";
    sourceCase          = "original";
    yearFormat          = "parentheses";
    webSourceFormat     = "WEB-DL";
    tagSeparator        = " ";
    providerCase        = "upper";
    seasonEpisodeFormat = "S01E01";
    _namingChanged();
  }

  function resetToDefault() {
    // Applique le préréglage "Équilibré" comme point de départ
    crf             = 28;
    preset          = "p5";
    audioCodecRules = { ...AUDIO_RULES_SMART };
    audioBitrate    = 192;
    spatialAq       = false;
    temporalAq      = false;
    aqStrength      = 8;
    multipass       = "disabled";
    container       = "mkv";

    seasonEpisodeFormat  = "S01E01";
    tagOrder             = [...DEFAULT_TAG_ORDER];
    disabledTags         = new Set<TagId>(["japver"]);
    team                 = "";
    keepJapaneseVer      = false;
    preserveDv           = false;
    videoMode            = "encode";
    subExtractFormat     = "srt";
    subExtractNaming     = "source";
    subExtractPathMode   = "source";
    subExtractCustomPath = "";
    showExtractButton    = true;
    resolutionCase       = "upper";
    titleCase            = "original";
    codecFormat          = "H265";
    sourceCase           = "original";
    yearFormat           = "parentheses";
    webSourceFormat      = "WEB-DL";
    tagSeparator         = " ";
    providerCase         = "upper";
    activePresetId       = "balanced";
    langOrder            = [...LANG_ORDER];
    defaultAudioLangs    = ["fre", "eng", "jpn"];
    defaultSubLangs      = ["fre"];
    try { localStorage.setItem(ACTIVE_PRESET_KEY, "balanced"); } catch { /* */ }
    flush();
  }

  // ─── Exports ──────────────────────────────────────────────────────────────

  return {
    get crf()                  { return crf; },
    get preset()               { return preset; },
    get activePresetId()       { return activePresetId; },
    get seasonEpisodeFormat()  { return seasonEpisodeFormat; },
    get tagOrder()             { return tagOrder; },
    get disabledTags()         { return disabledTags; },
    get resolutionCase()       { return resolutionCase; },
    get titleCase()            { return titleCase; },
    get codecFormat()          { return codecFormat; },
    get sourceCase()           { return sourceCase; },
    get yearFormat()           { return yearFormat; },
    get webSourceFormat()      { return webSourceFormat; },
    get tagSeparator()         { return tagSeparator; },
    get providerCase()         { return providerCase; },
    get team()                 { return team; },
    get keepJapaneseVer()      { return keepJapaneseVer; },
    get preserveDv()           { return preserveDv; },
    get langOrder()            { return langOrder; },
    get defaultAudioLangs()    { return defaultAudioLangs; },
    get defaultSubLangs()      { return defaultSubLangs; },
    get videoMode()            { return videoMode; },
    get audioCodecRules()      { return audioCodecRules; },
    get audioBitrate()         { return audioBitrate; },
    get spatialAq()            { return spatialAq; },
    get temporalAq()           { return temporalAq; },
    get aqStrength()           { return aqStrength; },
    get multipass()            { return multipass; },
    get container()            { return container; },
    get subExtractFormat()     { return subExtractFormat; },
    get subExtractNaming()     { return subExtractNaming; },
    get subExtractPathMode()   { return subExtractPathMode; },
    get subExtractCustomPath() { return subExtractCustomPath; },
    get showExtractButton()    { return showExtractButton; },
    get customAudioPresets()   { return customAudioPresets; },

    load,
    loadRenamingPrefs,
    flush,
    saveRenamingPrefs,
    applyPreset,
    allAudioPresets,
    applyAudioPreset,
    saveAudioPreset,
    renameAudioPreset,
    deleteAudioPreset,

    setCrf,
    setPreset,
    setSeasonEpisodeFormat,
    setTagOrder,
    setDisabledTags,
    setResolutionCase,
    setTitleCase,
    setCodecFormat,
    setSourceCase,
    setYearFormat,
    setWebSourceFormat,
    setTagSeparator,
    setProviderCase,
    setTeam,
    setKeepJapaneseVer,
    setPreserveDv,
    setLangOrder,
    setDefaultAudioLangs,
    setDefaultSubLangs,
    setVideoMode,
    setAudioCodecRule,
    setAudioBitrate,
    setSpatialAq,
    setTemporalAq,
    setAqStrength,
    setMultipass,
    setContainer,
    setSubExtractFormat,
    setSubExtractNaming,
    setSubExtractPathMode,
    setSubExtractCustomPath,
    setShowExtractButton,
    resetFormatOptions,
    resetToDefault,
  };
}

export const prefs = createPrefsStore();