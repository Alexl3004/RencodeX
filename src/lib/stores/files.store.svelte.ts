import { invoke } from "@tauri-apps/api/core";
import type { AppFile, FileAnalysis, CleanedName } from "./types";
import {
  computeTag,
  computeAudioTag,
  computeVideoCodecTag,
  buildOutputName,
  formatDuration,
  formatMb,
  langName,
  sortedLangs,
} from "./naming";
import { prefs } from "./prefs.store.svelte";

// ─── Store fichiers ───────────────────────────────────────────────────────────

function createFilesStore() {
  let files      = $state<AppFile[]>([]);
  let audioLangs = $state<Set<string>>(new Set());
  let subLangs   = $state<Set<string>>(new Set());

  // null = suivre les prefs, Set = override manuel de la session
  let selAudioOverride = $state<Set<string> | null>(null);
  let selSubsOverride  = $state<Set<string> | null>(null);

  // Getters calculés — pas de rune $derived pour rester compatible hors composant
  function getSelAudio() { return selAudioOverride ?? new Set(prefs.defaultAudioLangs); }
  function getSelSubs()  { return selSubsOverride  ?? new Set(prefs.defaultSubLangs);  }

  // Surcharges par fichier (chemin → Set de langues sélectionnées)
  let fileSelAudio = $state<Map<string, Set<string>>>(new Map());
  let fileSelSubs  = $state<Map<string, Set<string>>>(new Map());

  // Overrides de langues et codecs sur les streams
  let audioOverrides      = $state<Record<string, Record<string, string>>>({});
  let subOverrides        = $state<Record<string, Record<string, string>>>({});
  let globalCodecOverride = $state<Record<string, string>>({});

  // Sélection extraction
  let selectedForExtraction = $state<Set<string>>(new Set());
  let extractSelectionMode  = $state(false);

  // Sélection encodage
  let encodeSelectionMode  = $state(false);
  let selectedForEncoding  = $state<Set<string>>(new Set());

  // ─── Log callback (injecté par la façade) ────────────────────────────────

  let _log: (msg: string, level?: "info" | "warn" | "error" | "success") => void =
    (msg) => console.log(msg);

  function setLogger(fn: typeof _log) {
    _log = fn;
  }

  // ─── Helpers internes ────────────────────────────────────────────────────

  function _buildOptions() {
    return {
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
    };
  }

  function rebuildLangs() {
    const a = new Set<string>(), s = new Set<string>();
    files.forEach((f) => {
      f.audio_langs.forEach((l) => a.add(l));
      f.sub_langs.forEach((l)   => s.add(l));
    });
    // Trier selon l'ordre configurable (prefs.langOrder) plutôt que l'ordre d'insertion
    audioLangs = new Set(sortedLangs([...a], prefs.langOrder));
    subLangs   = new Set(sortedLangs([...s], prefs.langOrder));
  }

  // ─── Rafraîchissement des noms ────────────────────────────────────────────

  function refreshOutputNames() {
    const _tagOrder     = prefs.tagOrder;
    const _disabled     = prefs.disabledTags;
    const _resCase      = prefs.resolutionCase;
    const _titleCase    = prefs.titleCase;
    const _codecFmt     = prefs.codecFormat;
    const _srcCase      = prefs.sourceCase;
    const _yearParen    = prefs.yearFormat;
    const _webFmt       = prefs.webSourceFormat;
    const _tagSep       = prefs.tagSeparator;
    const _provCase     = prefs.providerCase;
    const _seFormat     = prefs.seasonEpisodeFormat;
    const _team         = prefs.team;
    const _selAudio     = getSelAudio();
    const _selSubs      = getSelSubs();
    const _fileSelAudio = fileSelAudio;
    const _fileSelSubs  = fileSelSubs;
    const _keepJapVer   = prefs.keepJapaneseVer;
    const _audioMode    = prefs.audioMode;
    const _videoMode    = prefs.videoMode;

    files = files.map((f) => {
      if (!f.cleaned || f.status === "encoding" || f.status === "done") return f;
      const effAudio = _fileSelAudio.get(f.path) ?? _selAudio;
      const effSubs  = _fileSelSubs.get(f.path)  ?? _selSubs;
      const tag      = computeTag(f.audio_langs, f.sub_langs, effAudio, effSubs);
      const audioTag = computeAudioTag(f.streams, effAudio, _audioMode);
      const codecTag = computeVideoCodecTag(f.streams, _videoMode, _codecFmt);
      const name     = buildOutputName(
        f.cleaned,
        tag,
        _seFormat,
        audioTag,
        _tagOrder,
        _team,
        {
          disabledTags:    _disabled,
          resolutionCase:  _resCase,
          titleCase:       _titleCase,
          codecFormat:     codecTag as import("./types").CodecFormat,
          sourceCase:      _srcCase,
          yearFormat:      _yearParen,
          webSourceFormat: _webFmt,
          tagSeparator:    _tagSep,
          providerCase:    _provCase,
          keepJapaneseVer: _keepJapVer,
        },
      );
      return { ...f, output_name: name };
    });
  }

  // ─── Ajout de fichiers ────────────────────────────────────────────────────

  async function addFiles(paths: string[]) {
    const ignored = paths.filter((p) => !/\.(mp4|mkv|avi|mov|flv)$/i.test(p));
    ignored.forEach((p) => {
      _log(`Ignoré : ${p.split(/[\\\/]/).pop() ?? p} (format non supporté)`, "warn");
    });
    const valid = paths.filter(
      (p) =>
        /\.(mp4|mkv|avi|mov|flv)$/i.test(p) && !files.find((f) => f.path === p),
    );
    if (valid.length === 0) return;
    if (valid.length === 1)
      _log(`Ajout : ${valid[0].split(/[\\\/]/).pop()}`, "info");
    else
      _log(`Ajout de ${valid.length} fichiers…`, "info");

    // Ajouter tous les placeholders d'un coup
    for (const path of valid) {
      const placeholder: AppFile = {
        path,
        filename: path.split(/[\\\/]/).pop() ?? path,
        size_mb: 0,
        duration_secs: 0,
        fps: 25,
        audio_langs: [],
        sub_langs: [],
        streams: [],
        status: "analysing",
        output_name: path.split(/[\\\/]/).pop()?.replace(/\.[^.]+$/, "") ?? "",
        output_ext: ".mkv",
        sub_extract_status: "none",
      };
      files = [...files, placeholder];
    }

    // Analyser tous les fichiers en parallèle
    await Promise.allSettled(
      valid.map(async (path) => {
        const name = path.split(/[\\\/]/).pop() ?? path;
        try {
          const analysis = await invoke<FileAnalysis>("analyze_file", { path });
          const idx = files.findIndex((f) => f.path === path);
          if (idx < 0) return;

          const cleaned = await invoke<CleanedName>("clean_filename", {
            raw: analysis.filename,
            audioLangs: analysis.audio_langs,
            subLangs: analysis.sub_langs,
          });

          const tag      = computeTag(analysis.audio_langs, analysis.sub_langs, getSelAudio(), getSelSubs());
          const audioTag = computeAudioTag(analysis.streams, getSelAudio(), prefs.audioMode);
          const codecTag = computeVideoCodecTag(analysis.streams, prefs.videoMode, prefs.codecFormat);
          const outName  = buildOutputName(
            cleaned,
            tag,
            prefs.seasonEpisodeFormat,
            audioTag,
            prefs.tagOrder,
            prefs.team,
            { ..._buildOptions(), codecFormat: codecTag as import("./types").CodecFormat },
          );

          const updated: AppFile = {
            ...files[idx],
            ...analysis,
            status: "ready",
            output_name: outName,
            output_ext: ".mkv",
            cleaned,
            sub_extract_status: "none",
          };
          files = files.map((f, i) => (i === idx ? updated : f));

          const mergedAudio = sortedLangs([...new Set([...audioLangs, ...analysis.audio_langs])], prefs.langOrder);
          const mergedSubs  = sortedLangs([...new Set([...subLangs,   ...analysis.sub_langs])],   prefs.langOrder);
          audioLangs = new Set(mergedAudio);
          subLangs   = new Set(mergedSubs);

          const dur   = formatDuration(analysis.duration_secs);
          const size  = formatMb(analysis.size_mb);
          const audio = analysis.audio_langs.map((l) => l.toUpperCase()).join("+") || "—";
          const subs  = analysis.sub_langs.map((l) => l.toUpperCase()).join("+") || "aucun";
          _log(
            `Analysé : ${analysis.filename} — ${size}${dur ? `, ${dur}` : ""}, audio [${audio}], sous-titres [${subs}]`,
            "info",
          );
          const audioMatch = analysis.audio_langs.some((l) => getSelAudio().has(l));
          if (!audioMatch)
            _log(
              `  ⚠ Aucune piste audio sélectionnée présente dans ${name} — toutes les pistes seront conservées`,
              "warn",
            );
        } catch (e) {
          const idx = files.findIndex((f) => f.path === path);
          if (idx >= 0)
            files = files.map((f, i) => (i === idx ? { ...f, status: "error" } : f));
          _log(`Erreur analyse : ${name} — ${e}`, "error");
        }
      })
    );
  }

  // ─── Suppression ─────────────────────────────────────────────────────────

  function removeFile(path: string) {
    const f = files.find((f) => f.path === path);
    if (f) _log(`Retiré : ${f.filename}`, "info");
    files = files.filter((f) => f.path !== path);

    const es = new Set(selectedForExtraction); es.delete(path);
    selectedForExtraction = es;
    const cs = new Set(selectedForEncoding);   cs.delete(path);
    selectedForEncoding = cs;
    rebuildLangs();
  }

  // ─── Sélection de langues globales ───────────────────────────────────────

  function toggleAudioLang(lang: string) {
    const s = new Set(getSelAudio());
    s.has(lang) ? s.delete(lang) : s.add(lang);
    selAudioOverride = s;
    refreshOutputNames();
    _log(`Piste audio : ${langName(lang)} ${s.has(lang) ? "activée" : "désactivée"}`, "info");
  }

  function toggleSubLang(lang: string) {
    const s = new Set(getSelSubs());
    s.has(lang) ? s.delete(lang) : s.add(lang);
    selSubsOverride = s;
    refreshOutputNames();
    _log(`Sous-titres : ${langName(lang)} ${s.has(lang) ? "activés" : "désactivés"}`, "info");
  }

  // ─── Sélection de langues par fichier ─────────────────────────────────────

  function setFileLangSel(path: string, audio: Set<string> | null, subs: Set<string> | null) {
    if (audio !== null) {
      const m = new Map(fileSelAudio);
      if (audio.size === 0) m.delete(path); else m.set(path, audio);
      fileSelAudio = m;
    }
    if (subs !== null) {
      const m = new Map(fileSelSubs);
      if (subs.size === 0) m.delete(path); else m.set(path, subs);
      fileSelSubs = m;
    }
    refreshOutputNames();
  }

  function clearFileLangSel(path: string) {
    const ma = new Map(fileSelAudio); ma.delete(path);
    const ms = new Map(fileSelSubs);  ms.delete(path);
    fileSelAudio = ma;
    fileSelSubs  = ms;
    refreshOutputNames();
  }

  // ─── Overrides streams ────────────────────────────────────────────────────

  function setAudioOverride(filePath: string, streamIndex: number, newLang: string) {
    audioOverrides = {
      ...audioOverrides,
      [filePath]: { ...(audioOverrides[filePath] ?? {}), [streamIndex]: newLang },
    };
    const name = files.find((f) => f.path === filePath)?.filename ?? filePath;
    _log(`Override audio stream ${streamIndex} → ${newLang.toUpperCase()} sur ${name}`, "info");
  }

  function setSubOverride(filePath: string, streamIndex: number, newLang: string) {
    subOverrides = {
      ...subOverrides,
      [filePath]: { ...(subOverrides[filePath] ?? {}), [streamIndex]: newLang },
    };
    const name = files.find((f) => f.path === filePath)?.filename ?? filePath;
    _log(`Override sous-titre stream ${streamIndex} → ${newLang.toUpperCase()} sur ${name}`, "info");
  }

  function setGlobalCodecOverride(sourceCodec: string, targetCodec: string) {
    globalCodecOverride = { ...globalCodecOverride, [sourceCodec]: targetCodec };
    _log(`Codec audio : toutes les pistes ${sourceCodec.toUpperCase()} → ${targetCodec}`, "info");
  }

  // ─── Renommage manuel ─────────────────────────────────────────────────────

  function renameFile(path: string, newName: string) {
    const f = files.find((f) => f.path === path);
    if (f) _log(`Renommé : ${f.output_name} → ${newName}`, "info");
    files = files.map((f) => (f.path === path ? { ...f, output_name: newName } : f));
  }

  // ─── Sélections extraction ────────────────────────────────────────────────

  function setExtractSelectionMode(value: boolean) {
    extractSelectionMode = value;
    if (!value) selectedForExtraction = new Set();
  }
  function toggleExtractSelection(path: string) {
    const s = new Set(selectedForExtraction);
    s.has(path) ? s.delete(path) : s.add(path);
    selectedForExtraction = s;
  }
  function setExtractSelection(paths: string[])  { selectedForExtraction = new Set(paths); }
  function clearExtractSelection()               { selectedForExtraction = new Set(); }

  // ─── Sélections encodage ──────────────────────────────────────────────────

  function setEncodeSelectionMode(value: boolean) {
    encodeSelectionMode = value;
    if (!value) selectedForEncoding = new Set();
  }
  function toggleEncodeSelection(path: string) {
    const s = new Set(selectedForEncoding);
    s.has(path) ? s.delete(path) : s.add(path);
    selectedForEncoding = s;
  }
  function setEncodeSelection(paths: string[])   { selectedForEncoding = new Set(paths); }
  function clearEncodeSelection()                { selectedForEncoding = new Set(); }

  // ─── Reset ────────────────────────────────────────────────────────────────

  function clearAll() {
    files                 = [];
    audioLangs            = new Set();
    subLangs              = new Set();
    selAudioOverride      = null;
    selSubsOverride       = null;
    audioOverrides        = {};
    subOverrides          = {};
    globalCodecOverride   = {};
    selectedForExtraction = new Set();
    selectedForEncoding   = new Set();
  }

  function clearSession() {
    files                 = [];
    audioLangs            = new Set();
    subLangs              = new Set();
    selAudioOverride      = null;
    selSubsOverride       = null;
    audioOverrides        = {};
    subOverrides          = {};
    globalCodecOverride   = {};
    selectedForExtraction = new Set();
    extractSelectionMode  = false;
    selectedForEncoding   = new Set();
    encodeSelectionMode   = false;
  }

  // ─── Exports ──────────────────────────────────────────────────────────────

  return {
    get files()      { return files; },
    set files(v)     { files = v; },  // accès écriture pour encoding.store
    get audioLangs() { return audioLangs; },
    get subLangs()   { return subLangs; },
    get selAudio()   { return getSelAudio(); },
    get selSubs()    { return getSelSubs(); },
    get fileSelAudio()   { return fileSelAudio; },
    get fileSelSubs()    { return fileSelSubs; },
    get audioOverrides() { return audioOverrides; },
    get subOverrides()   { return subOverrides; },
    get globalCodecOverride() { return globalCodecOverride; },

    get selectedForExtraction() { return selectedForExtraction; },
    get extractSelectionMode()  { return extractSelectionMode; },
    get encodeSelectionMode()   { return encodeSelectionMode; },
    get selectedForEncoding()   { return selectedForEncoding; },
    set selectedForEncoding(v)  { selectedForEncoding = v; },
    set encodeSelectionMode(v)  { encodeSelectionMode = v; },

    setLogger,
    addFiles,
    removeFile,
    clearAll,
    clearSession,
    refreshOutputNames,
    toggleAudioLang,
    toggleSubLang,
    setFileLangSel,
    clearFileLangSel,
    setAudioOverride,
    setSubOverride,
    setGlobalCodecOverride,
    renameFile,
    setExtractSelectionMode,
    toggleExtractSelection,
    setExtractSelection,
    clearExtractSelection,
    setEncodeSelectionMode,
    toggleEncodeSelection,
    setEncodeSelection,
    clearEncodeSelection,
  };
}

export const filesStore = createFilesStore();