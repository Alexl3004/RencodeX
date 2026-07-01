import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { stats } from "./stats.svelte";
import { toasts } from "$lib/stores/toasts.svelte";


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

export type AudioMode = "reencode" | "copy";
export type MultipassMode = "disabled" | "qres" | "fullres";
export type ContainerFormat = "mkv" | "mp4";

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
  audio_mode:              AudioMode;
  audio_bitrate:           number;
  spatial_aq:              boolean;
  temporal_aq:             boolean;
  aq_strength:             number;
  multipass:               MultipassMode;
  container:               ContainerFormat;
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
  status:        "pending" | "analysing" | "ready" | "queued" | "encoding" | "done" | "error";
  output_name:   string;
  output_ext:    string;
  result?:       FileResult;
  cleaned?:      CleanedName;
}

// ─── Format Saison/Épisode ──────────────────────────────────────────────────
// Le backend renvoie toujours `season_episode` au format canonique
// "S01E01" (ou "S01E01-E02" pour le multi-épisodes, ou "E07" pour un épisode
// seul sans saison, ou des tags type "OVA 02" / "SPECIAL"). On reformate
// côté client selon la préférence choisie par l'utilisateur, sans toucher
// au backend.

export type SeasonEpisodeFormat = "S01E01" | "S1E01" | "S1 E01" | "1x01";

export const SEASON_EPISODE_FORMATS: { value: SeasonEpisodeFormat; label: string; example: string }[] = [
  { value: "S01E01", label: "S01E01",  example: "S01E01" },
  { value: "S1E01",  label: "S1E01",   example: "S1E01" },
  { value: "S1 E01", label: "S1 E01",  example: "S1 E01" },
  { value: "1x01",   label: "1x01",    example: "1x01" },
];

const SE_RANGE_RE = /^S(\d{2,})E(\d{2,})(?:-E(\d{2,}))?$/;

function pad2(n: number): string {
  return n.toString().padStart(2, "0");
}

/**
 * Reformate une chaîne `season_episode` déjà normalisée par le backend
 * (ex: "S01E01", "S02E03-E04") selon le format d'affichage choisi.
 * Les valeurs qui ne correspondent pas au motif SxxExx (ex: "E07", "OVA 02",
 * "SPECIAL") sont laissées inchangées, car elles ne contiennent pas de
 * numéro de saison à reformater.
 */
export function formatSeasonEpisode(raw: string, format: SeasonEpisodeFormat): string {
  if (!raw) return raw;

  const m = raw.match(SE_RANGE_RE);
  if (!m) return raw;

  const season = parseInt(m[1], 10);
  const ep1    = parseInt(m[2], 10);
  const ep2    = m[3] !== undefined ? parseInt(m[3], 10) : null;

  switch (format) {
    case "S1E01":
      return ep2 !== null ? `S${season}E${pad2(ep1)}-E${pad2(ep2)}` : `S${season}E${pad2(ep1)}`;
    case "S1 E01":
      return ep2 !== null ? `S${season} E${pad2(ep1)}-E${pad2(ep2)}` : `S${season} E${pad2(ep1)}`;
    case "1x01":
      return ep2 !== null ? `${season}x${pad2(ep1)}-${pad2(ep2)}` : `${season}x${pad2(ep1)}`;
    case "S01E01":
    default:
      return raw;
  }
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

// ─── Ordre des tags (personnalisable) ──────────────────────────────────────
// Chaque tag du nom de sortie est identifié par un id stable. L'utilisateur
// peut réordonner ces ids (et insérer "team" où il veut) depuis les
// Paramètres d'encodage ; `buildOutputName` se contente de mapper chaque id
// vers sa valeur courante puis de filtrer les vides.

export type TagId =
  | "title" | "se" | "audio" | "resolution" | "provider"
  | "source" | "codec" | "bitdepth" | "audioCodec" | "team";

export const DEFAULT_TAG_ORDER: TagId[] = [
  "title", "se", "audio", "resolution", "provider", "source",
  "codec", "bitdepth", "audioCodec", "team",
];

export const TAG_LABELS: Record<TagId, string> = {
  title:      "Titre",
  se:         "Saison/Épisode",
  audio:      "Tag audio (VOSTFR, VF…)",
  resolution: "Résolution",
  provider:   "Provider",
  source:     "Source (BluRay, WEB-DL…)",
  codec:      "Codec vidéo (H265)",
  bitdepth:   "Profondeur (10bit)",
  audioCodec: "Codec audio (AAC…)",
  team:       "Team",
};

export function buildOutputName(
  cleaned: CleanedName,
  tag: string,
  seFormat: SeasonEpisodeFormat = "S01E01",
  audioTag: string = "AAC",
  tagOrder: TagId[] = DEFAULT_TAG_ORDER,
  team: string = "",
): string {
  const values: Record<TagId, string> = {
    title:      cleaned.title,
    se:         formatSeasonEpisode(cleaned.season_episode, seFormat),
    audio:      tag,
    resolution: cleaned.resolution,
    provider:   cleaned.provider,
    source:     cleaned.source,
    codec:      "H265",
    bitdepth:   "10bit",
    audioCodec: audioTag,
    team:       team.trim(),
  };

  return tagOrder
    .map((id) => values[id])
    .filter(Boolean)
    .join(" ")
    .replace(/\s+/g, " ")
    .trim();
}

/**
 * Détermine le tag audio à afficher dans le nom de sortie :
 * - "AAC" si l'audio est réencodé (mode "reencode")
 * - le(s) codec(s) source réel(s) si l'audio est copié sans réencodage
 *   (ex: "EAC3", ou "AAC-FLAC" si plusieurs pistes sélectionnées avec
 *   des codecs différents)
 */
export function computeAudioTag(
  streams: StreamInfo[],
  selAudio: Set<string>,
  audioMode: AudioMode,
): string {
  if (audioMode === "reencode") return "AAC";
  const codecs = new Set(
    streams
      .filter(s => s.codec_type === "audio" && selAudio.has(s.language))
      .map(s => s.codec_name.toUpperCase())
  );
  if (codecs.size === 0) return "AAC";
  return [...codecs].sort().join("-");
}

/**
 * Recalcule le nom d'affichage d'un fichier en appliquant le format
 * saison/épisode courant. À utiliser dans les templates à la place de
 * `file.output_name` pour que le rendu soit toujours réactif au format choisi.
 *
 * Si le fichier a été renommé manuellement (pas de `cleaned`), on applique
 * quand même une substitution de pattern sur le nom stocké pour couvrir
 * les cas courants.
 */
export function applySeFormat(
  file: AppFile,
  seFormat: SeasonEpisodeFormat,
  tagOrder: TagId[] = DEFAULT_TAG_ORDER,
  team: string = "",
): string {
  if (file.cleaned) {
    const tag = file.output_name.match(/\b(VF|VO|VOSTFR|VOSTA|VOSTKR|VOSTCH|MULTI|GER|SPA|ITA|POR|RUS)\b/)?.[1] ?? "";
    // Le tag audio (AAC, EAC3, etc.) est toujours le dernier token du nom déjà
    // construit — on le préserve tel quel ici plutôt que de le re-hardcoder,
    // car cette fonction n'a pas accès au mode audio courant du store.
    const audioTag = file.output_name.trim().split(/\s+/).pop() ?? "AAC";
    return buildOutputName(file.cleaned, tag, seFormat, audioTag, tagOrder, team);
  }
  // Renommage manuel : substitution directe du pattern SxxExx dans le nom stocké
  return file.output_name.replace(
    /S(\d{1,2})E(\d{2,})(?:-E(\d{2,}))?/gi,
    (_, s, e1, e2) => {
      const raw = e2 ? `S${s.padStart(2,"0")}E${e1}-E${e2}` : `S${s.padStart(2,"0")}E${e1}`;
      return formatSeasonEpisode(raw, seFormat);
    }
  );
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
  let globalCodecOverride = $state<Record<string, string>>({});
  let outputDir      = $state("");
  let encoding       = $state(false);
  let progress       = $state<ProgressEvent | null>(null);
  let summary        = $state<EncodeSummary | null>(null);
  let logs           = $state<{ msg: string; level: "info"|"warn"|"error"|"success" }[]>([]);

  let crf    = $state(28);
  let preset = $state("p5");
  let seasonEpisodeFormat = $state<SeasonEpisodeFormat>("S01E01");
  let tagOrder = $state<TagId[]>([...DEFAULT_TAG_ORDER]);
  let team     = $state("");

  // ─── Réglages avancés (audio / NVENC / conteneur) ──────────────────────
  let audioMode    = $state<AudioMode>("reencode");
  let audioBitrate = $state(192);
  let spatialAq    = $state(false);
  let temporalAq   = $state(false);
  let aqStrength   = $state(8);
  let multipass    = $state<MultipassMode>("disabled");
  let container    = $state<ContainerFormat>("mkv");

  let _unlisten: UnlistenFn[] = [];

  function setCrf(value: number) {
    crf = value;
    persistPrefs();
  }

  function setPreset(value: string) {
    preset = value;
    persistPrefs();
  }

  function setSeasonEpisodeFormat(value: SeasonEpisodeFormat) {
    seasonEpisodeFormat = value;
    persistPrefs();
    refreshOutputNames();
  }

  function setTagOrder(value: TagId[]) {
    tagOrder = value;
    persistPrefs();
    refreshOutputNames();
  }

  /** Déplace le tag `id` d'une position vers le haut (-1) ou le bas (+1). */
  function moveTag(id: TagId, dir: -1 | 1) {
    const idx = tagOrder.indexOf(id);
    if (idx < 0) return;
    const newIdx = idx + dir;
    if (newIdx < 0 || newIdx >= tagOrder.length) return;
    const next = [...tagOrder];
    [next[idx], next[newIdx]] = [next[newIdx], next[idx]];
    setTagOrder(next);
  }

  function setTeam(value: string) {
    team = value;
    persistPrefs();
    refreshOutputNames();
  }

  function setAudioMode(value: AudioMode) {
    audioMode = value;
    persistPrefs();
    refreshOutputNames();
  }

  function setAudioBitrate(value: number) {
    audioBitrate = value;
    persistPrefs();
  }

  function setSpatialAq(value: boolean) {
    spatialAq = value;
    persistPrefs();
  }

  function setTemporalAq(value: boolean) {
    temporalAq = value;
    persistPrefs();
  }

  function setAqStrength(value: number) {
    aqStrength = value;
    persistPrefs();
  }

  function setMultipass(value: MultipassMode) {
    multipass = value;
    persistPrefs();
  }

  function setContainer(value: ContainerFormat) {
    container = value;
    persistPrefs();
  }

  // ─── Persistance des préférences d'encodage (fichier disque, via Rust) ───
  // On utilise un petit débounce pour éviter une écriture disque à chaque
  // pixel glissé sur un slider, tout en garantissant que la dernière valeur
  // est bien écrite avant la fermeture de l'app (cf. flushPrefs côté init).
  let _persistTimer: ReturnType<typeof setTimeout> | null = null;

  function currentPrefs() {
    return {
      crf,
      preset,
      se_format: seasonEpisodeFormat,
      tag_order: tagOrder,
      team,
      audio_mode: audioMode,
      audio_bitrate: audioBitrate,
      spatial_aq: spatialAq,
      temporal_aq: temporalAq,
      aq_strength: aqStrength,
      multipass,
      container,
    };
  }

  async function flushPrefs() {
    if (_persistTimer) {
      clearTimeout(_persistTimer);
      _persistTimer = null;
    }
    try {
      await invoke("save_encoding_prefs", { prefs: currentPrefs() });
    } catch (e) {
      log(`Erreur sauvegarde préférences d'encodage : ${e}`, "error");
    }
  }

  function persistPrefs() {
    if (_persistTimer) clearTimeout(_persistTimer);
    _persistTimer = setTimeout(() => { flushPrefs(); }, 300);
  }

  async function loadEncodingSettings() {
    try {
      const prefs = await invoke<{
        crf: number; preset: string; se_format: string; tag_order: string[]; team: string;
        audio_mode: string; audio_bitrate: number; spatial_aq: boolean; temporal_aq: boolean;
        aq_strength: number; multipass: string; container: string;
      }>("load_encoding_prefs");

      if (typeof prefs.crf === "number") crf = prefs.crf;
      if (prefs.preset) preset = prefs.preset;
      if (prefs.se_format && SEASON_EPISODE_FORMATS.some(f => f.value === prefs.se_format)) {
        seasonEpisodeFormat = prefs.se_format as SeasonEpisodeFormat;
      }

      const parsedOrder = prefs.tag_order as TagId[] | undefined;
      const isValidOrder = Array.isArray(parsedOrder)
        && parsedOrder.length === DEFAULT_TAG_ORDER.length
        && DEFAULT_TAG_ORDER.every(id => parsedOrder.includes(id));
      if (isValidOrder) tagOrder = parsedOrder!;

      if (typeof prefs.team === "string") team = prefs.team;

      if (prefs.audio_mode === "reencode" || prefs.audio_mode === "copy") audioMode = prefs.audio_mode;
      if (typeof prefs.audio_bitrate === "number") audioBitrate = prefs.audio_bitrate;
      if (typeof prefs.spatial_aq === "boolean") spatialAq = prefs.spatial_aq;
      if (typeof prefs.temporal_aq === "boolean") temporalAq = prefs.temporal_aq;
      if (typeof prefs.aq_strength === "number") aqStrength = prefs.aq_strength;
      if (prefs.multipass === "disabled" || prefs.multipass === "qres" || prefs.multipass === "fullres") {
        multipass = prefs.multipass;
      }
      if (prefs.container === "mkv" || prefs.container === "mp4") container = prefs.container;
    } catch (e) {
      log(`Erreur chargement préférences d'encodage : ${e}`, "error");
    }
  }

  async function init() {
    outputDir = await invoke<string>("get_default_output_dir");
    await loadEncodingSettings();
    await stats.init();
    log(`Dossier de sortie : ${outputDir}`, "info");
    await listenEvents();

    // S'assure que les préférences en attente (debounce) sont bien écrites
    // sur disque avant que l'application ne se ferme.
    window.addEventListener("beforeunload", () => { flushPrefs(); });
  }

  async function listenEvents() {
    const u1 = await listen<ProgressEvent>("encode-progress", (e) => {
      progress = e.payload;
      // Seul le fichier actif bascule en "encoding", les autres restent "queued"
      const activeIdx = e.payload.file_index;
      files = files.map((f, i) => {
        if (f.status === "queued" || f.status === "encoding") {
          return { ...f, status: i === activeIdx ? "encoding" : "queued" };
        }
        return f;
      });
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

        const tag      = computeTag(analysis.audio_langs, analysis.sub_langs, selAudio, selSubs);
        const audioTag = computeAudioTag(analysis.streams, selAudio, audioMode);
        const outName  = buildOutputName(cleaned, tag, seasonEpisodeFormat, audioTag, tagOrder, team);

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
        const audioCodecOvr: Record<string, string> = {};
        f.streams
          .filter(s => s.codec_type === "audio")
          .forEach(s => {
            const target = globalCodecOverride[s.codec_name.toLowerCase()];
            if (target && target !== codecDefault(s.codec_name)) {
              audioCodecOvr[s.index.toString()] = target;
            }
          });

        const outExt = container === "mp4" ? ".mp4" : ".mkv";

        return {
          input_path:              f.path,
          output_path:             joinPath(outputDir, `${applySeFormat(f, seasonEpisodeFormat)}${outExt}`),
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
          audio_mode:              audioMode,
          audio_bitrate:           audioBitrate,
          spatial_aq:              spatialAq,
          temporal_aq:             temporalAq,
          aq_strength:             aqStrength,
          multipass,
          container,
        };
      });

    const totalSize = formatMb(files.filter(f => f.status === "ready").reduce((s, f) => s + f.size_mb, 0));
    log(`── Démarrage de l'encodage — ${jobs.length} fichier${jobs.length > 1 ? "s" : ""} (${totalSize}) ──`, "info");

    // Tous les fichiers prêts passent en file d'attente.
    // Le statut "encoding" est mis à jour fichier par fichier via encode-progress.
    files = files.map(f => f.status === "ready" ? {...f, status: "queued"} : f);

    jobs.forEach((j, i) => {
      const name = j.input_path.split(/[\\/]/).pop() ?? j.input_path;
      log(`  [${i+1}/${jobs.length}] En file : ${name}`, "info");
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
      const savedMb = summary.total_original_mb - summary.total_encoded_mb;
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

      // ── Toasts in-app ────────────────────────────────────────────────
      if (errors > 0 && ok === 0) {
        toasts.error(`Batch échoué — ${errors}/${jobs.length} erreurs`);
      } else if (errors > 0 || cancelled > 0) {
        toasts.warn(
          `${ok}/${jobs.length} réussis` +
          (gain ? ` · −${gain}%` : "") +
          (errors > 0 ? ` · ${errors} erreur${errors > 1 ? "s" : ""}` : "")
        );
      } else {
        toasts.success(
          `${ok} fichier${ok > 1 ? "s" : ""} encodé${ok > 1 ? "s" : ""}` +
          (gain ? ` · −${gain}% (${formatMb(savedMb)})` : "") +
          (dur  ? ` · ${dur}` : ""),
        );
      }
      // ─────────────────────────────────────────────────────────────────

    } catch (e) {
      log(`Erreur fatale encodage : ${e}`, "error");
      toasts.error(`Erreur fatale : ${e}`);
    } finally {
      encoding = false;
      progress = null;
    }
  }

  async function cancelEncoding() {
    await invoke("cancel_encoding");
    encoding = false;
    log("Encodage annulé par l'utilisateur", "warn");
    toasts.warn("Encodage annulé");
  }

  function refreshOutputNames() {
    files = files.map(f => {
      if (!f.cleaned || f.status === "encoding" || f.status === "done") return f;
      const tag      = computeTag(f.audio_langs, f.sub_langs, selAudio, selSubs);
      const audioTag = computeAudioTag(f.streams, selAudio, audioMode);
      const name     = buildOutputName(f.cleaned, tag, seasonEpisodeFormat, audioTag, tagOrder, team);
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
    files               = [];
    audioLangs          = new Set();
    subLangs            = new Set();
    selAudio            = new Set(["fre", "eng", "jpn"]);
    selSubs             = new Set(["fre"]);
    crf                 = 28;
    preset              = "p5";
    seasonEpisodeFormat = "S01E01";
    tagOrder            = [...DEFAULT_TAG_ORDER];
    team                = "";
    audioOverrides      = {};
    subOverrides        = {};
    globalCodecOverride = {};
    summary             = null;
    progress            = null;
    audioMode           = "reencode";
    audioBitrate        = 192;
    spatialAq           = false;
    temporalAq          = false;
    aqStrength          = 8;
    multipass           = "disabled";
    container           = "mkv";

    flushPrefs();

    logs = [];
    log("Interface réinitialisée aux paramètres par défaut", "info");

    setTimeout(() => {
      window.dispatchEvent(new Event('resize'));
    }, 10);

    forceUpdate();
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
    get seasonEpisodeFormat() { return seasonEpisodeFormat; },
    get tagOrder()            { return tagOrder; },
    get team()                { return team; },
    get audioMode()           { return audioMode; },
    get audioBitrate()        { return audioBitrate; },
    get spatialAq()           { return spatialAq; },
    get temporalAq()          { return temporalAq; },
    get aqStrength()          { return aqStrength; },
    get multipass()           { return multipass; },
    get container()           { return container; },
    get forceUpdateCounter()  { return forceUpdateCounter; },
    get globalCodecOverride() { return globalCodecOverride; },
    /** Retourne le nom d'affichage en appliquant le format SE courant en temps réel. */
    getDisplayName(file: AppFile): string {
      return applySeFormat(file, seasonEpisodeFormat, tagOrder, team);
    },
    setCrf,
    setPreset,
    setSeasonEpisodeFormat,
    setTagOrder,
    moveTag,
    setTeam,
    setAudioMode,
    setAudioBitrate,
    setSpatialAq,
    setTemporalAq,
    setAqStrength,
    setMultipass,
    setContainer,
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