import { invoke } from "@tauri-apps/api/core";
import type {
  SeasonEpisodeFormat,
  TagId,
  ResolutionCase,
  TitleCaseMode,
  CodecFormat,
  SourceCase,
  WebSourceFormat,
  TagSeparator,
  ProviderCase,
  AudioMode,
  MultipassMode,
  ContainerFormat,
  SubExtractFormat,
  SubExtractNaming,
  SubExtractPathMode,
  EncodePreset,
} from "./types";
import { DEFAULT_TAG_ORDER, LANG_ORDER, SEASON_EPISODE_FORMATS } from "./naming";

// ─── Persistance localStorage (renommage) ─────────────────────────────────────

const RENAMING_KEY      = "rencodeX:renaming";
const ACTIVE_PRESET_KEY = "rencodeX:activePresetId";

// ─── Préréglages d'encodage intégrés ─────────────────────────────────────────
// Extensible plus tard : ajouter des presets custom dans localStorage/Tauri.

export const BUILTIN_PRESETS: EncodePreset[] = [
  {
    id:           "fast",
    label:        "Rapide",
    crf:          30,
    preset:       "p3",
    audioMode:    "copy",
    audioBitrate: 192,
    spatialAq:    false,
    temporalAq:   false,
    aqStrength:   8,
    multipass:    "disabled",
  },
  {
    id:           "balanced",
    label:        "Équilibré",
    crf:          28,
    preset:       "p5",
    audioMode:    "reencode",
    audioBitrate: 192,
    spatialAq:    false,
    temporalAq:   false,
    aqStrength:   8,
    multipass:    "disabled",
  },
  {
    id:           "quality",
    label:        "Haute qualité",
    crf:          24,
    preset:       "p7",
    audioMode:    "reencode",
    audioBitrate: 320,
    spatialAq:    true,
    temporalAq:   true,
    aqStrength:   8,
    multipass:    "qres",
  },
  {
    id:           "archive",
    label:        "Archive",
    crf:          20,
    preset:       "p7",
    audioMode:    "copy",
    audioBitrate: 192,
    spatialAq:    true,
    temporalAq:   true,
    aqStrength:   8,
    multipass:    "fullres",
  },
];

// ─── Store préférences ────────────────────────────────────────────────────────

function createPrefsStore() {
  // Encodage
  let crf     = $state(28);
  let preset  = $state("p5");

  // Préréglage actif (null = paramètres personnalisés)
  let activePresetId = $state<string | null>(
    typeof localStorage !== "undefined"
      ? localStorage.getItem(ACTIVE_PRESET_KEY)
      : null
  );

  // Nommage
  let seasonEpisodeFormat = $state<SeasonEpisodeFormat>("S01E01");
  let tagOrder            = $state<TagId[]>([...DEFAULT_TAG_ORDER]);
  let disabledTags        = $state<Set<TagId>>(new Set());
  let resolutionCase      = $state<ResolutionCase>("upper");
  let titleCase           = $state<TitleCaseMode>("original");
  let codecFormat         = $state<CodecFormat>("H265");
  let sourceCase          = $state<SourceCase>("original");
  let yearParentheses     = $state<boolean>(true);
  let webSourceFormat     = $state<WebSourceFormat>("WEB-DL");
  let tagSeparator        = $state<TagSeparator>(" ");
  let providerCase        = $state<ProviderCase>("upper");
  let team                = $state("");
  let keepJapaneseVer     = $state<boolean>(false);
  let langOrder           = $state<string[]>([...LANG_ORDER]);
  let defaultAudioLangs   = $state<string[]>(["fre", "eng", "jpn"]);
  let defaultSubLangs     = $state<string[]>(["fre"]);

  // Audio / vidéo
  let audioMode    = $state<AudioMode>("reencode");
  let audioBitrate = $state(192);
  let spatialAq    = $state(false);
  let temporalAq   = $state(false);
  let aqStrength   = $state(8);
  let multipass    = $state<MultipassMode>("disabled");
  let container    = $state<ContainerFormat>("mkv");

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
      year_parentheses:        yearParentheses,
      web_source_format:       webSourceFormat,
      tag_separator:           tagSeparator,
      provider_case:           providerCase,
      team,
      keep_japanese_ver:       keepJapaneseVer,
      lang_order:              langOrder,
      default_audio_langs:     defaultAudioLangs,
      default_sub_langs:       defaultSubLangs,
      audio_mode:              audioMode,
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
        tag_order:         tagOrder,
        disabled_tags:     [...disabledTags],
        resolution_case:   resolutionCase,
        title_case:        titleCase,
        codec_format:      codecFormat,
        source_case:       sourceCase,
        year_parentheses:  yearParentheses,
        web_source_format: webSourceFormat,
        tag_separator:     tagSeparator,
        provider_case:     providerCase,
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

      if (
        Array.isArray(p.tag_order) &&
        (p.tag_order as string[]).every((id) => DEFAULT_TAG_ORDER.includes(id as TagId))
      ) tagOrder = p.tag_order as TagId[];

      if (Array.isArray(p.disabled_tags))
        disabledTags = new Set(
          (p.disabled_tags as string[]).filter((id) =>
            DEFAULT_TAG_ORDER.includes(id as TagId)
          ) as TagId[]
        );

      if (p.resolution_case === "upper" || p.resolution_case === "lower")
        resolutionCase = p.resolution_case;
      if (["original","upper","lower","title"].includes(p.title_case as string))
        titleCase = p.title_case as TitleCaseMode;
      if (["H265","H.265","HEVC"].includes(p.codec_format as string))
        codecFormat = p.codec_format as CodecFormat;
      if (["original","upper","lower"].includes(p.source_case as string))
        sourceCase = p.source_case as SourceCase;
      if (typeof p.year_parentheses === "boolean")
        yearParentheses = p.year_parentheses;
      if (["WEB-DL","WEBDL","Web-DL"].includes(p.web_source_format as string))
        webSourceFormat = p.web_source_format as WebSourceFormat;
      if ([" ",".", "_"].includes(p.tag_separator as string))
        tagSeparator = p.tag_separator as TagSeparator;
      if (["upper","lower","hidden"].includes(p.provider_case as string))
        providerCase = p.provider_case as ProviderCase;
      if (typeof p.team === "string") team = p.team;
      if (typeof p.keepJapaneseVer === "boolean")
        keepJapaneseVer = p.keepJapaneseVer;
      if (Array.isArray(p.lang_order) && (p.lang_order as string[]).every((c) => typeof c === "string" && c.length > 0))
        langOrder = p.lang_order as string[];
      if (Array.isArray(p.default_audio_langs) && (p.default_audio_langs as string[]).length > 0)
        defaultAudioLangs = p.default_audio_langs as string[];
      if (Array.isArray(p.default_sub_langs))
        defaultSubLangs = p.default_sub_langs as string[];
    } catch { /* JSON invalide */ }
  }

  // ─── Chargement depuis Tauri ──────────────────────────────────────────────

  async function load() {
    try {
      const prefs = await invoke<{
        crf: number;
        preset: string;
        se_format: string;
        tag_order: string[];
        team: string;
        audio_mode: string;
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
        year_parentheses?: boolean;
        web_source_format?: string;
        tag_separator?: string;
        provider_case?: string;
        keep_japanese_ver?: boolean;
        lang_order?: string[];
        default_audio_langs?: string[];
        default_sub_langs?: string[];
      }>("load_encoding_prefs");

      if (typeof prefs.crf === "number") crf = prefs.crf;
      if (prefs.preset) preset = prefs.preset;
      if (
        prefs.se_format &&
        SEASON_EPISODE_FORMATS.some((f) => f.value === prefs.se_format)
      ) {
        seasonEpisodeFormat = prefs.se_format as SeasonEpisodeFormat;
      }
      const parsedOrder = prefs.tag_order as TagId[] | undefined;
      const isValidOrder =
        Array.isArray(parsedOrder) &&
        parsedOrder.length === DEFAULT_TAG_ORDER.length &&
        DEFAULT_TAG_ORDER.every((id) => parsedOrder.includes(id));
      if (isValidOrder) tagOrder = parsedOrder!;

      if (Array.isArray(prefs.disabled_tags)) {
        const valid = (prefs.disabled_tags as string[]).filter((id) =>
          DEFAULT_TAG_ORDER.includes(id as TagId)
        );
        disabledTags = new Set(valid as TagId[]);
      }
      if (prefs.resolution_case === "upper" || prefs.resolution_case === "lower")
        resolutionCase = prefs.resolution_case;
      if (["original","upper","lower","title"].includes(prefs.title_case ?? ""))
        titleCase = prefs.title_case as TitleCaseMode;
      if (["H265","H.265","HEVC"].includes(prefs.codec_format ?? ""))
        codecFormat = prefs.codec_format as CodecFormat;
      if (["original","upper","lower"].includes(prefs.source_case ?? ""))
        sourceCase = prefs.source_case as SourceCase;
      if (typeof prefs.year_parentheses === "boolean")
        yearParentheses = prefs.year_parentheses;
      if (["WEB-DL","WEBDL","Web-DL"].includes(prefs.web_source_format ?? ""))
        webSourceFormat = prefs.web_source_format as WebSourceFormat;
      if ([" ", ".", "_"].includes(prefs.tag_separator ?? ""))
        tagSeparator = prefs.tag_separator as TagSeparator;
      if (["upper","lower","hidden"].includes(prefs.provider_case ?? ""))
        providerCase = prefs.provider_case as ProviderCase;
      if (typeof prefs.team === "string") team = prefs.team;
      if (typeof prefs.keep_japanese_ver === "boolean")
        keepJapaneseVer = prefs.keep_japanese_ver;
      if (
        Array.isArray(prefs.lang_order) &&
        (prefs.lang_order as string[]).every((c) => typeof c === "string" && c.length > 0)
      ) langOrder = prefs.lang_order as string[];
      if (Array.isArray(prefs.default_audio_langs) && (prefs.default_audio_langs as string[]).length > 0)
        defaultAudioLangs = prefs.default_audio_langs as string[];
      if (Array.isArray(prefs.default_sub_langs))
        defaultSubLangs = prefs.default_sub_langs as string[];
      if (prefs.audio_mode === "reencode" || prefs.audio_mode === "copy")
        audioMode = prefs.audio_mode;
      if (typeof prefs.audio_bitrate === "number") audioBitrate = prefs.audio_bitrate;
      if (typeof prefs.spatial_aq === "boolean") spatialAq = prefs.spatial_aq;
      if (typeof prefs.temporal_aq === "boolean") temporalAq = prefs.temporal_aq;
      if (typeof prefs.aq_strength === "number") aqStrength = prefs.aq_strength;
      if (
        prefs.multipass === "disabled" ||
        prefs.multipass === "qres" ||
        prefs.multipass === "fullres"
      ) multipass = prefs.multipass;
      if (prefs.container === "mkv" || prefs.container === "mp4")
        container = prefs.container;
      if (prefs.sub_extract_format === "srt" || prefs.sub_extract_format === "ass")
        subExtractFormat = prefs.sub_extract_format;
      if (prefs.sub_extract_naming === "source" || prefs.sub_extract_naming === "cleaned")
        subExtractNaming = prefs.sub_extract_naming;
      if (
        prefs.sub_extract_path_mode === "source" ||
        prefs.sub_extract_path_mode === "downloads" ||
        prefs.sub_extract_path_mode === "custom"
      ) subExtractPathMode = prefs.sub_extract_path_mode;
      if (typeof prefs.sub_extract_custom_path === "string")
        subExtractCustomPath = prefs.sub_extract_custom_path;
      if (typeof prefs.show_extract_button === "boolean")
        showExtractButton = prefs.show_extract_button;
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

    // Appliquer tous les paramètres sans passer par _markCustom
    crf          = p.crf;
    preset       = p.preset;
    audioMode    = p.audioMode;
    audioBitrate = p.audioBitrate;
    spatialAq    = p.spatialAq;
    temporalAq   = p.temporalAq;
    aqStrength   = p.aqStrength;
    multipass    = p.multipass;

    activePresetId = id;
    try { localStorage.setItem(ACTIVE_PRESET_KEY, id); } catch { /* */ }
    schedule();
  }

  // ─── Setters ──────────────────────────────────────────────────────────────

  function setCrf(value: number)            { crf = value;          _markCustom(); schedule(); }
  function setPreset(value: string)         { preset = value;       _markCustom(); schedule(); }
  function setAudioMode(v: AudioMode)       { audioMode = v;        _markCustom(); schedule(); }
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
  function setTagOrder(v: TagId[])              { tagOrder = v;         _namingChanged(); }
  function setDisabledTags(v: Set<TagId>)       { disabledTags = v;     _namingChanged(); }
  function setResolutionCase(v: ResolutionCase) { resolutionCase = v;   _namingChanged(); }
  function setTitleCase(v: TitleCaseMode)       { titleCase = v;        _namingChanged(); }
  function setCodecFormat(v: CodecFormat)       { codecFormat = v;      _namingChanged(); }
  function setSourceCase(v: SourceCase)         { sourceCase = v;       _namingChanged(); }
  function setYearParentheses(v: boolean)       { yearParentheses = v;  _namingChanged(); }
  function setWebSourceFormat(v: WebSourceFormat){ webSourceFormat = v; _namingChanged(); }
  function setTagSeparator(v: TagSeparator)     { tagSeparator = v;     _namingChanged(); }
  function setProviderCase(v: ProviderCase)     { providerCase = v;     _namingChanged(); }
  function setTeam(v: string)                   { team = v;             _namingChanged(); }
  function setKeepJapaneseVer(v: boolean)       { keepJapaneseVer = v;  _namingChanged(); }
  function setDefaultAudioLangs(v: string[]) { defaultAudioLangs = v; _namingChanged(); }
  function setDefaultSubLangs(v: string[])   { defaultSubLangs   = v; _namingChanged(); }

  function setLangOrder(v: string[]) {
    // Validation : codes non vides et uniques
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
    yearParentheses     = true;
    webSourceFormat     = "WEB-DL";
    tagSeparator        = " ";
    providerCase        = "upper";
    seasonEpisodeFormat = "S01E01";
    _namingChanged();
  }

  function resetToDefault() {
    crf                  = 28;
    preset               = "p5";
    seasonEpisodeFormat  = "S01E01";
    tagOrder             = [...DEFAULT_TAG_ORDER];
    disabledTags         = new Set<TagId>(["japver"]);
    team                 = "";
    keepJapaneseVer      = false;
    audioMode            = "reencode";
    audioBitrate         = 192;
    spatialAq            = false;
    temporalAq           = false;
    aqStrength           = 8;
    multipass            = "disabled";
    container            = "mkv";
    subExtractFormat     = "srt";
    subExtractNaming     = "source";
    subExtractPathMode   = "source";
    subExtractCustomPath = "";
    showExtractButton    = true;
    resolutionCase       = "upper";
    titleCase            = "original";
    codecFormat          = "H265";
    sourceCase           = "original";
    yearParentheses      = true;
    webSourceFormat      = "WEB-DL";
    tagSeparator         = " ";
    providerCase         = "upper";
    activePresetId       = "balanced"; // reset → préréglage Équilibré
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
    get yearParentheses()      { return yearParentheses; },
    get webSourceFormat()      { return webSourceFormat; },
    get tagSeparator()         { return tagSeparator; },
    get providerCase()         { return providerCase; },
    get team()                 { return team; },
    get keepJapaneseVer()      { return keepJapaneseVer; },
    get langOrder()            { return langOrder; },
    get defaultAudioLangs()    { return defaultAudioLangs; },
    get defaultSubLangs()      { return defaultSubLangs; },
    get audioMode()            { return audioMode; },
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

    load,
    loadRenamingPrefs,
    flush,
    saveRenamingPrefs,
    applyPreset,

    setCrf,
    setPreset,
    setSeasonEpisodeFormat,
    setTagOrder,
    setDisabledTags,
    setResolutionCase,
    setTitleCase,
    setCodecFormat,
    setSourceCase,
    setYearParentheses,
    setWebSourceFormat,
    setTagSeparator,
    setProviderCase,
    setTeam,
    setKeepJapaneseVer,
    setLangOrder,
    setDefaultAudioLangs,
    setDefaultSubLangs,
    setAudioMode,
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