import { invoke } from "@tauri-apps/api/core";
import { prefs } from "./prefs.store.svelte";
import { filesStore } from "./files.store.svelte";
import { encodingStore } from "./encoding.store.svelte";
import { applySeFormat, DEFAULT_TAG_ORDER } from "./naming";
import type { AppFile, NamingOptions } from "./types";
import { stats } from "./stats.svelte";

// ─── Façade encoder ───────────────────────────────────────────────────────────
// Colle les trois stores ensemble et expose l'API publique utilisée par les
// composants Svelte. Gère aussi init, outputDir, navLayout, reset, clearSession.

type FfmpegCheckResult = {
  path: string;
  exists: boolean;
  executable: boolean;
  version: string | null;
};

function createEncoder() {
  let outputDir      = $state("");
  let navLayout      = $state<"vertical" | "horizontal">("vertical");
  let innerNavLayout = $state<"vertical" | "horizontal">("vertical");
  let outputDirPresets = $state<string[]>([]);
  let forceUpdateCounter = $state(0);

  // Câble le logger des fichiers sur le log de l'encodage
  filesStore.setLogger((msg, level) => encodingStore.log(msg, level));

  // ─── Init ─────────────────────────────────────────────────────────────────

  async function init() {
    outputDir = await invoke<string>("get_default_output_dir");
    await loadDirConfig();
    await prefs.load();
    prefs.loadRenamingPrefs(); // override Tauri avec localStorage si plus récent
    encodingStore.log(`Dossier de sortie : ${outputDir}`, "info");
    await encodingStore.listenEvents();
    stats.init();

    // ── Vérification FFmpeg au démarrage ──────────────────────────────────
    // On passe path="" pour que le backend utilise le chemin résolu depuis
    // la config + les variables d'environnement (resolve_config).
    try {
      const ffCheck = await invoke<FfmpegCheckResult>("check_ffmpeg", { path: "" });

      if (!ffCheck.exists) {
        encodingStore.log(
          `⚠️ FFmpeg introuvable : « ${ffCheck.path} » — configurez le chemin dans Paramètres › FFmpeg`,
          "warn"
        );
      } else if (!ffCheck.executable) {
        encodingStore.log(
          `⚠️ FFmpeg présent mais non exécutable : « ${ffCheck.path} » — vérifiez les permissions`,
          "warn"
        );
      } else {
        encodingStore.log(
          `FFmpeg ${ffCheck.version ? ffCheck.version + "  " : ""}${ffCheck.path}`,
          "info"
        );
      }
    } catch (e) {
      encodingStore.log(`Impossible de vérifier FFmpeg : ${e}`, "warn");
    }
    // ──────────────────────────────────────────────────────────────────────

    window.addEventListener("beforeunload", () => {
      prefs.flush();
      prefs.saveRenamingPrefs();
    });
  }

  async function loadDirConfig() {
    try {
      const cfg = await invoke<Record<string, any>>("load_config");
      if (cfg.nav_layout)       navLayout      = cfg.nav_layout as "vertical" | "horizontal";
      if (cfg.inner_nav_layout) innerNavLayout = cfg.inner_nav_layout as "vertical" | "horizontal";
      const g = (snake: string, camel: string) => cfg[snake] ?? cfg[camel];
      outputDirPresets =
        (g("output_dir_presets", "outputDirPresets") as string[] | undefined) ?? [];
    } catch {
      outputDirPresets = [];
    }
  }

  // ─── Helpers nommage (délégués à prefs + files) ───────────────────────────

  function getDisplayName(file: AppFile): string {
    return applySeFormat(file, prefs.seasonEpisodeFormat, prefs.tagOrder, prefs.team, {
      disabledTags:    prefs.disabledTags,
      resolutionCase:  prefs.resolutionCase,
      titleCase:       prefs.titleCase,
      codecFormat:     prefs.codecFormat,
      sourceCase:      prefs.sourceCase,
      yearFormat:      prefs.yearFormat,
      webSourceFormat: prefs.webSourceFormat,
      tagSeparator:    prefs.tagSeparator,
      providerCase:    prefs.providerCase,
      keepJapaneseVer: prefs.keepJapaneseVer,
    } satisfies NamingOptions);
  }

  // ─── Tag order helpers ────────────────────────────────────────────────────

  function moveTag(id: import("./types").TagId, dir: -1 | 1) {
    const order  = prefs.tagOrder;
    const idx    = order.indexOf(id);
    if (idx < 0) return;
    const newIdx = idx + dir;
    if (newIdx < 0 || newIdx >= order.length) return;
    const next = [...order];
    [next[idx], next[newIdx]] = [next[newIdx], next[idx]];
    prefs.setTagOrder(next);
    filesStore.refreshOutputNames();
  }

  function toggleTag(id: import("./types").TagId) {
    const next = new Set(prefs.disabledTags);
    next.has(id) ? next.delete(id) : next.add(id);
    prefs.setDisabledTags(next);
    filesStore.refreshOutputNames();
  }

  function resetTagOrder() {
    prefs.setTagOrder([...DEFAULT_TAG_ORDER]);
    prefs.setDisabledTags(new Set());
    filesStore.refreshOutputNames();
  }

  // ─── Wrappers setters prefs qui déclenchent refreshOutputNames ────────────

  function setSeasonEpisodeFormat(v: import("./types").SeasonEpisodeFormat) {
    prefs.setSeasonEpisodeFormat(v);
    filesStore.refreshOutputNames();
  }
  function setTagOrder(v: import("./types").TagId[]) {
    prefs.setTagOrder(v);
    filesStore.refreshOutputNames();
  }
  function setResolutionCase(v: import("./types").ResolutionCase) {
    prefs.setResolutionCase(v);
    filesStore.refreshOutputNames();
  }
  function setTitleCase(v: import("./types").TitleCaseMode) {
    prefs.setTitleCase(v);
    filesStore.refreshOutputNames();
  }
  function setCodecFormat(v: import("./types").CodecFormat) {
    prefs.setCodecFormat(v);
    filesStore.refreshOutputNames();
  }
  function setSourceCase(v: import("./types").SourceCase) {
    prefs.setSourceCase(v);
    filesStore.refreshOutputNames();
  }
  function setYearFormat(v: import("./types").YearFormat) {
    prefs.setYearFormat(v);
    filesStore.refreshOutputNames();
  }
  function setWebSourceFormat(v: import("./types").WebSourceFormat) {
    prefs.setWebSourceFormat(v);
    filesStore.refreshOutputNames();
  }
  function setTagSeparator(v: import("./types").TagSeparator) {
    prefs.setTagSeparator(v);
    filesStore.refreshOutputNames();
  }
  function setProviderCase(v: import("./types").ProviderCase) {
    prefs.setProviderCase(v);
    filesStore.refreshOutputNames();
  }
  function setTeam(v: string) {
    prefs.setTeam(v);
    filesStore.refreshOutputNames();
  }
  function setKeepJapaneseVer(v: boolean) {
    prefs.setKeepJapaneseVer(v);
    filesStore.refreshOutputNames();
  }
  function setAudioMode(v: import("./types").AudioMode) {
    prefs.setAudioMode(v);
    filesStore.refreshOutputNames();
  }
  function resetFormatOptions() {
    prefs.resetFormatOptions();
    filesStore.refreshOutputNames();
  }

  // ─── Reset global ─────────────────────────────────────────────────────────

  function forceUpdate() { forceUpdateCounter++; }

  function resetToDefault() {
    filesStore.clearAll();
    encodingStore.clearSummary();
    encodingStore.clearLogs();
    prefs.resetToDefault();
    filesStore.refreshOutputNames();
    encodingStore.log("Interface réinitialisée aux paramètres par défaut", "info");
    setTimeout(() => window.dispatchEvent(new Event("resize")), 10);
    forceUpdate();
  }

  function clearSession() {
    filesStore.clearSession();
    encodingStore.clearSummary();
    encodingStore.clearLogs();
    encodingStore.log("Session réinitialisée", "info");
    setTimeout(() => window.dispatchEvent(new Event("resize")), 10);
    forceUpdate();
  }

  // ─── Exports ──────────────────────────────────────────────────────────────

  return {
    // État layout / répertoire
    get outputDir()        { return outputDir; },
    set outputDir(v)       { outputDir = v; },
    get outputDirPresets() { return outputDirPresets; },
    get navLayout()        { return navLayout; },
    set navLayout(v: "vertical" | "horizontal") { navLayout = v; },
    get innerNavLayout()   { return innerNavLayout; },
    set innerNavLayout(v: "vertical" | "horizontal") { innerNavLayout = v; },
    get forceUpdateCounter() { return forceUpdateCounter; },
    loadDirConfig,

    // Délégation fichiers
    get files()      { return filesStore.files; },
    get audioLangs() { return filesStore.audioLangs; },
    get subLangs()   { return filesStore.subLangs; },
    get selAudio()   { return filesStore.selAudio; },
    get selSubs()    { return filesStore.selSubs; },
    get fileSelAudio()          { return filesStore.fileSelAudio; },
    get fileSelSubs()           { return filesStore.fileSelSubs; },
    get audioOverrides()        { return filesStore.audioOverrides; },
    get subOverrides()          { return filesStore.subOverrides; },
    get globalCodecOverride()   { return filesStore.globalCodecOverride; },
    get selectedForExtraction() { return filesStore.selectedForExtraction; },
    get extractSelectionMode()  { return filesStore.extractSelectionMode; },
    get encodeSelectionMode()   { return filesStore.encodeSelectionMode; },
    get selectedForEncoding()   { return filesStore.selectedForEncoding; },
    addFiles:                  (paths: string[]) => filesStore.addFiles(paths),
    removeFile:                (path: string)    => filesStore.removeFile(path),
    clearAll:                  ()               => filesStore.clearAll(),
    toggleAudioLang:           (l: string)      => filesStore.toggleAudioLang(l),
    toggleSubLang:             (l: string)      => filesStore.toggleSubLang(l),
    setFileLangSel:            filesStore.setFileLangSel,
    clearFileLangSel:          filesStore.clearFileLangSel,
    setAudioOverride:          filesStore.setAudioOverride,
    setSubOverride:            filesStore.setSubOverride,
    setGlobalCodecOverride:    filesStore.setGlobalCodecOverride,
    renameFile:                filesStore.renameFile,
    setExtractSelectionMode:   filesStore.setExtractSelectionMode,
    toggleExtractSelection:    filesStore.toggleExtractSelection,
    setExtractSelection:       filesStore.setExtractSelection,
    clearExtractSelection:     filesStore.clearExtractSelection,
    setEncodeSelectionMode:    filesStore.setEncodeSelectionMode,
    toggleEncodeSelection:     filesStore.toggleEncodeSelection,
    setEncodeSelection:        filesStore.setEncodeSelection,
    clearEncodeSelection:      filesStore.clearEncodeSelection,

    // Délégation encodage
    get encoding()           { return encodingStore.encoding; },
    get paused()             { return encodingStore.paused; },
    get progress()           { return encodingStore.progress; },
    get summary()            { return encodingStore.summary; },
    get logs()               { return encodingStore.logs; },
    get extractingSubs()     { return encodingStore.extractingSubs; },
    get subExtractProgress() { return encodingStore.subExtractProgress; },
    startEncoding:            () => encodingStore.startEncoding(outputDir),
    cancelEncoding:           () => encodingStore.cancelEncoding(),
    pauseEncoding:            () => encodingStore.pauseEncoding(),
    resumeEncoding:           () => encodingStore.resumeEncoding(),
    startSubtitleExtraction:  () => encodingStore.startSubtitleExtraction(outputDir),
    cancelSubtitleExtraction: () => encodingStore.cancelSubtitleExtraction(),
    clearLogs:                () => encodingStore.clearLogs(),
    log:                      (m: string, l?: "info"|"warn"|"error"|"success") =>
                                encodingStore.log(m, l),

    // Délégation prefs
    get crf()                  { return prefs.crf; },
    get preset()               { return prefs.preset; },
    get seasonEpisodeFormat()  { return prefs.seasonEpisodeFormat; },
    get tagOrder()             { return prefs.tagOrder; },
    get disabledTags()         { return prefs.disabledTags; },
    get resolutionCase()       { return prefs.resolutionCase; },
    get titleCase()            { return prefs.titleCase; },
    get codecFormat()          { return prefs.codecFormat; },
    get sourceCase()           { return prefs.sourceCase; },
    get yearFormat()           { return prefs.yearFormat; },
    get webSourceFormat()      { return prefs.webSourceFormat; },
    get tagSeparator()         { return prefs.tagSeparator; },
    get providerCase()         { return prefs.providerCase; },
    get team()                 { return prefs.team; },
    get keepJapaneseVer()      { return prefs.keepJapaneseVer; },
    get audioMode()            { return prefs.audioMode; },
    get audioBitrate()         { return prefs.audioBitrate; },
    get spatialAq()            { return prefs.spatialAq; },
    get temporalAq()           { return prefs.temporalAq; },
    get aqStrength()           { return prefs.aqStrength; },
    get multipass()            { return prefs.multipass; },
    get container()            { return prefs.container; },
    get subExtractFormat()     { return prefs.subExtractFormat; },
    get subExtractNaming()     { return prefs.subExtractNaming; },
    get subExtractPathMode()   { return prefs.subExtractPathMode; },
    get subExtractCustomPath() { return prefs.subExtractCustomPath; },
    get showExtractButton()    { return prefs.showExtractButton; },
    get activePresetId() { return prefs.activePresetId; },
    applyPreset:           (id: string) => { prefs.applyPreset(id); filesStore.refreshOutputNames(); },
    setCrf:                prefs.setCrf,
    setPreset:             prefs.setPreset,
    setSeasonEpisodeFormat,
    setTagOrder,
    moveTag,
    toggleTag,
    resetTagOrder,
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
    setAudioMode,
    setAudioBitrate:       prefs.setAudioBitrate,
    setSpatialAq:          prefs.setSpatialAq,
    setTemporalAq:         prefs.setTemporalAq,
    setAqStrength:         prefs.setAqStrength,
    setMultipass:          prefs.setMultipass,
    setContainer:          prefs.setContainer,
    setSubExtractFormat:   prefs.setSubExtractFormat,
    setSubExtractNaming:   prefs.setSubExtractNaming,
    setSubExtractPathMode: prefs.setSubExtractPathMode,
    setSubExtractCustomPath: prefs.setSubExtractCustomPath,
    setShowExtractButton:  prefs.setShowExtractButton,
    resetFormatOptions,

    // Nommage
    getDisplayName,

    // Lifecycle
    init,
    resetToDefault,
    clearSession,
  };
}

export const encoder = createEncoder();