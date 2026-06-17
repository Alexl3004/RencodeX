import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { stats } from "./stats.svelte";


// ─── Types ────────────────────────────────────────────────────────────────────

export interface StreamInfo {
  index:      number;
  codec_type: string;
  codec_name: string;
  language:   string;
  width?:     number;
  height?:    number;
}

export interface FileAnalysis {
  path:          string;
  filename:      string;
  size_mb:       number;
  duration_secs: number;
  fps:           number;
  streams:       StreamInfo[];
  audio_langs:   string[];
  sub_langs:     string[];
}

export interface EncodeJob {
  input_path:              string;
  output_path:             string;
  audio_langs:             string[];
  sub_langs:               string[];
  audio_overrides:         Record<string, string>;
  sub_overrides:           Record<string, string>;
  audio_codec_overrides:   Record<string, string>;
  audio_bitrate_overrides: Record<string, string>;
  duration_secs:           number;
  fps:                     number;
  crf:                     number;
  preset:                  string;
}

export interface ProgressEvent {
  file_index:      number;
  file_total:      number;
  file_name:       string;
  percent:         number;
  speed:           number;
  remaining_file:  number;
  remaining_total: number;
}

export interface FileResult {
  path:          string;
  name:          string;
  status:        "ok" | "error" | "cancelled";
  original_mb:   number;
  encoded_mb:    number;
  duration_secs: number;
  error_msg?:    string;
}

export interface EncodeSummary {
  files:              FileResult[];
  total_original_mb:  number;
  total_encoded_mb:   number;
  total_secs:         number;
}

export interface CleanedName {
  title:          string;
  season_episode: string;
  resolution:     string;
  source:         string;
  provider:       string;
  audio_tags:     string;
  suggested:      string;
}

export interface AppFile {
  path:          string;
  filename:      string;
  size_mb:       number;
  duration_secs: number;
  fps:           number;
  audio_langs:   string[];
  sub_langs:     string[];
  streams:       StreamInfo[];
  status:        "pending" | "analysing" | "ready" | "encoding" | "done" | "error";
  output_name:   string;
  output_ext:    string;
  result?:       FileResult;
  cleaned?:      CleanedName;
}

// ─── Lang helpers ─────────────────────────────────────────────────────────────

export const LANG_NAMES: Record<string, string> = {
  fre: "Français", eng: "Anglais", jpn: "Japonais", ger: "Allemand",
  spa: "Espagnol", kor: "Coréen",  ita: "Italien",  por: "Portugais",
  rus: "Russe",    chi: "Chinois", und: "Indéfini",
};

export const LANG_ORDER = ["fre","eng","jpn","kor","ger","spa","ita","por","rus","chi","und"];

export function langName(code: string): string {
  return LANG_NAMES[code] ?? code.toUpperCase();
}

// ─── Tag helpers ──────────────────────────────────────────────────────────────

export function computeTag(
  fileAudioLangs: string[],
  fileSubLangs:   string[],
  selAudio:       Set<string>,
  selSubs:        Set<string>,
): string {
  const audio = fileAudioLangs.filter(l => selAudio.has(l));
  const subs  = fileSubLangs.filter(l => selSubs.has(l));

  if (audio.length === 0 || audio.every(l => l === "und")) return "";

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

export function buildOutputName(cleaned: CleanedName, tag: string): string {
  return [
    cleaned.title,
    cleaned.season_episode,
    tag,
    cleaned.resolution,
    cleaned.provider,
    cleaned.source,
    "H265",
    "10bit",
    "AAC",
  ].filter(Boolean).join(" ").replace(/\s+/g, " ").trim();
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

export function formatDuration(secs: number): string {
  if (!secs || secs <= 0) return "";
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = Math.floor(secs % 60);
  if (h > 0) return `${h}h ${String(m).padStart(2, "0")}m ${String(s).padStart(2, "0")}s`;
  if (m > 0) return `${m}m ${String(s).padStart(2, "0")}s`;
  return `${s}s`;
}

function formatDurationShort(secs: number): string {
  if (!secs || secs <= 0) return "";
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = Math.floor(secs % 60);
  if (h > 0) return `${h}h${String(m).padStart(2, "0")}m`;
  if (m > 0) return `${m}m${String(s).padStart(2, "0")}s`;
  return `${s}s`;
}

function formatMb(mb: number): string {
  if (mb >= 1024) return `${(mb / 1024).toFixed(2)} GB`;
  return `${mb.toFixed(0)} MB`;
}

function joinPath(dir: string, name: string): string {
  const d = dir.replace(/[\\/]+$/, "");
  return `${d}\\${name}`;
}

// ─── Codec helper ─────────────────────────────────────────────────────────────

// Codecs déjà lossy/compressés → copy par défaut, pas de re-encodage inutile
const COPY_BY_DEFAULT = ["aac", "opus", "ac3", "eac3", "mp3", "flac"];

export function codecDefault(codecName: string): string {
  return COPY_BY_DEFAULT.includes(codecName.toLowerCase()) ? "copy" : "aac";
}

// ─── Store ────────────────────────────────────────────────────────────────────

function createEncoder() {
  let files          = $state<AppFile[]>([]);
  let audioLangs     = $state<Set<string>>(new Set());
  let subLangs       = $state<Set<string>>(new Set());
  let selAudio       = $state<Set<string>>(new Set(["fre","eng","jpn"]));
  let selSubs        = $state<Set<string>>(new Set(["fre"]));
  let audioOverrides = $state<Record<string, Record<string, string>>>({});
  let subOverrides   = $state<Record<string, Record<string, string>>>({});
  // Override global : clé = codec source (ex: "dts"), valeur = codec cible (ex: "aac")
  let globalCodecOverride = $state<Record<string, string>>({});
  let outputDir      = $state("");
  let encoding       = $state(false);
  let progress       = $state<ProgressEvent | null>(null);
  let summary        = $state<EncodeSummary | null>(null);
  let logs           = $state<{ msg: string; level: "info"|"warn"|"error"|"success" }[]>([]);

  // Paramètres d'encodage
  let crf    = $state(28);
  let preset = $state("p5");

  let _unlisten: UnlistenFn[] = [];

  function setCrf(value: number) {
    crf = value;
    localStorage.setItem("rencodex-crf", value.toString());
  }

  function setPreset(value: string) {
    preset = value;
    localStorage.setItem("rencodex-preset", value);
  }

  function loadEncodingSettings() {
    const savedCrf    = localStorage.getItem("rencodex-crf");
    const savedPreset = localStorage.getItem("rencodex-preset");
    if (savedCrf)    crf    = parseInt(savedCrf);
    if (savedPreset) preset = savedPreset;
  }

  async function init() {
    outputDir = await invoke<string>("get_default_output_dir");
    loadEncodingSettings();
    stats.init();
    log(`Dossier de sortie : ${outputDir}`, "info");
    await listenEvents();
  }

  async function listenEvents() {
    const u1 = await listen<ProgressEvent>("encode-progress", (e) => {
      progress = e.payload;
    });

    const u2 = await listen<any>("encode-file-done", (e) => {
      const p = e.payload;
      const f = files.find(f => f.path === p.path || f.filename === p.name);
      if (f) {
        f.status = p.status === "ok" ? "done" : "error";
        if (p.status === "ok") f.result = p;
      }

      if (p.status === "ok") {
        const gain = p.original_mb > 0
          ? ((p.original_mb - p.encoded_mb) / p.original_mb * 100).toFixed(1)
          : null;
        const dur = formatDuration(p.duration_secs);
        log(
          `✓ ${p.name}` +
          (gain ? ` — gain ${gain}%` : "") +
          ` (${formatMb(p.encoded_mb)}` +
          (dur ? `, ${dur}` : "") +
          `)`,
          "success"
        );
      } else {
        log(`✗ ${p.name} — échec${p.error_msg ? ` : ${p.error_msg}` : ""}`, "error");
      }
    });

    _unlisten = [u1, u2];
  }

  async function addFiles(paths: string[]) {
    const ignored = paths.filter(p => !/\.(mp4|mkv|avi|mov|flv)$/i.test(p));
    ignored.forEach(p => {
      const name = p.split(/[\\/]/).pop() ?? p;
      log(`Ignoré : ${name} (format non supporté)`, "warn");
    });

    const valid = paths.filter(p =>
      /\.(mp4|mkv|avi|mov|flv)$/i.test(p) && !files.find(f => f.path === p)
    );

    if (valid.length === 0) return;

    if (valid.length === 1) {
      log(`Ajout : ${valid[0].split(/[\\/]/).pop()}`, "info");
    } else {
      log(`Ajout de ${valid.length} fichiers…`, "info");
    }

    for (const path of valid) {
      const placeholder: AppFile = {
        path, filename: path.split(/[\\/]/).pop() ?? path,
        size_mb: 0, duration_secs: 0, fps: 25,
        audio_langs: [], sub_langs: [], streams: [],
        status: "analysing",
        output_name: path.split(/[\\/]/).pop()?.replace(/\.[^.]+$/, "") ?? "",
        output_ext: ".mkv",
      };
      files = [...files, placeholder];
    }

    for (const path of valid) {
      const name = path.split(/[\\/]/).pop() ?? path;
      try {
        const analysis = await invoke<FileAnalysis>("analyze_file", { path });
        const idx = files.findIndex(f => f.path === path);
        if (idx < 0) continue;

        const cleaned = await invoke<CleanedName>("clean_filename", {
          raw:        analysis.filename,
          audioLangs: analysis.audio_langs,
          subLangs:   analysis.sub_langs,
        });

        const tag     = computeTag(analysis.audio_langs, analysis.sub_langs, selAudio, selSubs);
        const outName = buildOutputName(cleaned, tag);

        const updated: AppFile = {
          ...files[idx],
          ...analysis,
          status:      "ready",
          output_name: outName,
          output_ext:  ".mkv",
          cleaned,
        };
        files = files.map((f, i) => i === idx ? updated : f);

        analysis.audio_langs.forEach(l => audioLangs = new Set([...audioLangs, l]));
        analysis.sub_langs.forEach(l   => subLangs   = new Set([...subLangs, l]));

        const dur   = formatDuration(analysis.duration_secs);
        const size  = formatMb(analysis.size_mb);
        const audio = analysis.audio_langs.map(l => l.toUpperCase()).join("+") || "—";
        const subs  = analysis.sub_langs.map(l => l.toUpperCase()).join("+") || "aucun";
        log(
          `Analysé : ${analysis.filename} — ${size}${dur ? `, ${dur}` : ""}, audio [${audio}], sous-titres [${subs}]`,
          "info"
        );

        const audioMatch = analysis.audio_langs.some(l => selAudio.has(l));
        if (!audioMatch) {
          log(
            `  ⚠ Aucune piste audio sélectionnée présente dans ${name} — toutes les pistes seront conservées`,
            "warn"
          );
        }

      } catch (e) {
        const idx = files.findIndex(f => f.path === path);
        if (idx >= 0) files = files.map((f, i) => i === idx ? {...f, status: "error"} : f);
        log(`Erreur analyse : ${name} — ${e}`, "error");
      }
    }
  }

  function removeFile(path: string) {
    const f = files.find(f => f.path === path);
    if (f) log(`Retiré : ${f.filename}`, "info");
    files = files.filter(f => f.path !== path);
    rebuildLangs();
  }

  function rebuildLangs() {
    const a = new Set<string>(), s = new Set<string>();
    files.forEach(f => {
      f.audio_langs.forEach(l => a.add(l));
      f.sub_langs.forEach(l => s.add(l));
    });
    audioLangs = a;
    subLangs   = s;
  }

  function clearAll() {
    files               = [];
    audioLangs          = new Set();
    subLangs            = new Set();
    selAudio            = new Set(["fre","eng","jpn"]);
    selSubs             = new Set(["fre"]);
    audioOverrides      = {};
    subOverrides        = {};
    globalCodecOverride = {};
    summary             = null;
    progress            = null;
    logs                = [];
  }

  function clearLogs() {
    logs = [];
  }

  async function startEncoding() {
    if (!files.length || encoding) return;
    encoding = true;
    summary  = null;

    const jobs: EncodeJob[] = files
      .filter(f => f.status === "ready")
      .map(f => {
        // Construire les overrides codec par stream à partir de globalCodecOverride
        const audioCodecOvr: Record<string, string> = {};
        f.streams
          .filter(s => s.codec_type === "audio")
          .forEach(s => {
            const target = globalCodecOverride[s.codec_name.toLowerCase()];
            // N'ajouter l'override que s'il diffère du comportement par défaut
            if (target && target !== codecDefault(s.codec_name)) {
              audioCodecOvr[s.index.toString()] = target;
            }
          });

        return {
          input_path:              f.path,
          output_path:             joinPath(outputDir, `${f.output_name}${f.output_ext}`),
          audio_langs:             [...selAudio],
          sub_langs:               [...selSubs],
          audio_overrides:         audioOverrides[f.path] ?? {},
          sub_overrides:           subOverrides[f.path]   ?? {},
          audio_codec_overrides:   audioCodecOvr,
          audio_bitrate_overrides: {},
          duration_secs:           f.duration_secs,
          fps:                     f.fps ?? 25,
          crf,
          preset,
        };
      });

    const totalSize = formatMb(files.filter(f => f.status === "ready").reduce((s, f) => s + f.size_mb, 0));
    log(`── Démarrage de l'encodage — ${jobs.length} fichier${jobs.length > 1 ? "s" : ""} (${totalSize}) ──`, "info");

    files = files.map(f => f.status === "ready" ? {...f, status: "encoding"} : f);

    jobs.forEach((j, i) => {
      const name = j.input_path.split(/[\\/]/).pop() ?? j.input_path;
      log(`  [${i+1}/${jobs.length}] En attente : ${name}`, "info");
    });

    try {
      summary = await invoke<EncodeSummary>("start_encoding", { jobs });
      stats.recordSummary(summary);

      const ok        = summary.files.filter(f => f.status === "ok").length;
      const errors    = summary.files.filter(f => f.status === "error").length;
      const cancelled = summary.files.filter(f => f.status === "cancelled").length;
      const gain = summary.total_original_mb > 0
        ? ((summary.total_original_mb - summary.total_encoded_mb) / summary.total_original_mb * 100).toFixed(1)
        : null;
      const dur = formatDuration(summary.total_secs);

      log(
        `── Encodage terminé — ${ok}/${jobs.length} réussi${ok > 1 ? "s" : ""}` +
        (errors    > 0 ? `, ${errors} erreur${errors > 1 ? "s" : ""}` : "") +
        (cancelled > 0 ? `, ${cancelled} annulé${cancelled > 1 ? "s" : ""}` : "") +
        (gain ? ` — gain global ${gain}%` : "") +
        (dur  ? ` — durée ${dur}` : "") +
        ` ──`,
        errors > 0 && ok === 0 ? "error" : "success"
      );
    } catch (e) {
      log(`Erreur fatale encodage : ${e}`, "error");
    } finally {
      encoding = false;
      progress = null;
    }
  }

  async function cancelEncoding() {
    await invoke("cancel_encoding");
    encoding = false;
    log("Encodage annulé par l'utilisateur", "warn");
  }

  function refreshOutputNames() {
    files = files.map(f => {
      if (!f.cleaned || f.status === "encoding" || f.status === "done") return f;
      const tag  = computeTag(f.audio_langs, f.sub_langs, selAudio, selSubs);
      const name = buildOutputName(f.cleaned, tag);
      return { ...f, output_name: name };
    });
  }

  function toggleAudioLang(lang: string) {
    const s = new Set(selAudio);
    s.has(lang) ? s.delete(lang) : s.add(lang);
    selAudio = s;
    refreshOutputNames();
    log(`Piste audio : ${langName(lang)} ${s.has(lang) ? "activée" : "désactivée"}`, "info");
  }

  function toggleSubLang(lang: string) {
    const s = new Set(selSubs);
    s.has(lang) ? s.delete(lang) : s.add(lang);
    selSubs = s;
    refreshOutputNames();
    log(`Sous-titres : ${langName(lang)} ${s.has(lang) ? "activés" : "désactivés"}`, "info");
  }

  function setAudioOverride(filePath: string, streamIndex: number, newLang: string) {
    audioOverrides = {
      ...audioOverrides,
      [filePath]: { ...(audioOverrides[filePath] ?? {}), [streamIndex]: newLang }
    };
    const name = files.find(f => f.path === filePath)?.filename ?? filePath;
    log(`Override audio stream ${streamIndex} → ${newLang.toUpperCase()} sur ${name}`, "info");
  }

  function setSubOverride(filePath: string, streamIndex: number, newLang: string) {
    subOverrides = {
      ...subOverrides,
      [filePath]: { ...(subOverrides[filePath] ?? {}), [streamIndex]: newLang }
    };
    const name = files.find(f => f.path === filePath)?.filename ?? filePath;
    log(`Override sous-titre stream ${streamIndex} → ${newLang.toUpperCase()} sur ${name}`, "info");
  }

  // Override global : s'applique à tous les fichiers qui ont ce codec source
  function setGlobalCodecOverride(sourceCodec: string, targetCodec: string) {
    globalCodecOverride = { ...globalCodecOverride, [sourceCodec]: targetCodec };
    log(`Codec audio : toutes les pistes ${sourceCodec.toUpperCase()} → ${targetCodec}`, "info");
  }

  function renameFile(path: string, newName: string) {
    const f = files.find(f => f.path === path);
    if (f) log(`Renommé : ${f.output_name} → ${newName}`, "info");
    files = files.map(f => f.path === path ? { ...f, output_name: newName } : f);
  }

  function log(msg: string, level: "info"|"warn"|"error"|"success" = "info") {
    const ts = new Date().toLocaleTimeString("fr-FR");
    logs = [...logs, { msg: `[${ts}] ${msg}`, level }];
    if (logs.length > 500) logs = logs.slice(-400);
  }

  let forceUpdateCounter = $state(0);

  function forceUpdate() {
    forceUpdateCounter++;
  }

  function resetToDefault() {
    console.log("=== resetToDefault appelée ===");

    files               = [];
    audioLangs          = new Set();
    subLangs            = new Set();
    selAudio            = new Set(["fre", "eng", "jpn"]);
    selSubs             = new Set(["fre"]);
    crf                 = 28;
    preset              = "p5";
    audioOverrides      = {};
    subOverrides        = {};
    globalCodecOverride = {};
    summary             = null;
    progress            = null;

    localStorage.setItem("rencodex-crf",    "28");
    localStorage.setItem("rencodex-preset", "p5");

    logs = [];
    log("Interface réinitialisée aux paramètres par défaut", "info");

    setTimeout(() => {
      window.dispatchEvent(new Event('resize'));
    }, 10);

    forceUpdate();
    console.log("=== resetToDefault terminée ===");
  }

  return {
    get files()               { return files; },
    get audioLangs()          { return audioLangs; },
    get subLangs()            { return subLangs; },
    get selAudio()            { return selAudio; },
    get selSubs()             { return selSubs; },
    get outputDir()           { return outputDir; },
    set outputDir(v)          { outputDir = v; },
    get encoding()            { return encoding; },
    get progress()            { return progress; },
    get summary()             { return summary; },
    get logs()                { return logs; },
    get crf()                 { return crf; },
    get preset()              { return preset; },
    get forceUpdateCounter()  { return forceUpdateCounter; },
    get globalCodecOverride() { return globalCodecOverride; },
    setCrf,
    setPreset,
    init,
    addFiles,
    removeFile,
    clearAll,
    clearLogs,
    startEncoding,
    cancelEncoding,
    toggleAudioLang,
    toggleSubLang,
    setAudioOverride,
    setSubOverride,
    setGlobalCodecOverride,
    renameFile,
    log,
    resetToDefault,
  };
}

export const encoder = createEncoder();