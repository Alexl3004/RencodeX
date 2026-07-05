import { invoke } from "@tauri-apps/api/core";
import type { EncodeSummary } from "./encoder.svelte";

// ── Record d'un fichier ────────────────────────────────────────────────────
interface FileRecord {
  name:        string;
  original_mb: number;
  encoded_mb:  number;
  ratio_pct:   number; // % de compression (ex: 42.3)
}

// ── Session d'encodage ─────────────────────────────────────────────────────
interface EncodeSession {
  date:      string; // ISO
  files:     number; // fichiers réussis
  ratio_pct: number; // compression moyenne de la session
  saved_mb:  number; // espace économisé dans la session
}

// ── Session d'extraction ───────────────────────────────────────────────────
interface ExtractSession {
  date:   string;
  files:  number; // fichiers extraits avec succès
  tracks: number; // pistes extraites
}

interface RawStats {
  total_files:             number;
  total_launched:          number;
  total_original_mb:       number;
  total_encoded_mb:        number;
  sum_ratio_pct:           number;
  total_secs:              number;
  last_updated:            string | null;
  total_extracted_files:   number;
  total_extract_launched:  number;
  total_tracks_extracted:  number;
  // Nouveaux champs
  record_heaviest:         FileRecord | null; // fichier avec la plus grande taille d'entrée
  record_best_ratio:       FileRecord | null; // fichier avec le meilleur ratio de compression
  encode_sessions:         EncodeSession[];   // dernières N sessions d'encodage
  extract_sessions:        ExtractSession[];  // dernières N sessions d'extraction
}

const MAX_SESSIONS = 10;

function emptyStats(): RawStats {
  return {
    total_files: 0,
    total_launched: 0,
    total_original_mb: 0,
    total_encoded_mb: 0,
    sum_ratio_pct: 0,
    total_secs: 0,
    last_updated: null,
    total_extracted_files: 0,
    total_extract_launched: 0,
    total_tracks_extracted: 0,
    record_heaviest: null,
    record_best_ratio: null,
    encode_sessions: [],
    extract_sessions: [],
  };
}

function createStats() {
  let totalFiles           = $state(0);
  let totalLaunched        = $state(0);
  let totalOriginalMb      = $state(0);
  let totalEncodedMb       = $state(0);
  let sumRatioPct          = $state(0);
  let totalSecs            = $state(0);
  let lastUpdated          = $state<string | null>(null);
  let totalExtractedFiles  = $state(0);
  let totalExtractLaunched = $state(0);
  let totalTracksExtracted = $state(0);
  // Records
  let recordHeaviest       = $state<FileRecord | null>(null);
  let recordBestRatio      = $state<FileRecord | null>(null);
  // Historiques
  let encodeSessions       = $state<EncodeSession[]>([]);
  let extractSessions      = $state<ExtractSession[]>([]);

  async function persist() {
    const raw: RawStats = {
      total_files: totalFiles,
      total_launched: totalLaunched,
      total_original_mb: totalOriginalMb,
      total_encoded_mb: totalEncodedMb,
      sum_ratio_pct: sumRatioPct,
      total_secs: totalSecs,
      last_updated: lastUpdated,
      total_extracted_files: totalExtractedFiles,
      total_extract_launched: totalExtractLaunched,
      total_tracks_extracted: totalTracksExtracted,
      record_heaviest: recordHeaviest,
      record_best_ratio: recordBestRatio,
      encode_sessions: encodeSessions,
      extract_sessions: extractSessions,
    };
    try {
      await invoke("save_stats", { stats: raw });
    } catch (e) {
      console.error("Impossible de sauvegarder les statistiques :", e);
    }
  }

  async function init() {
    try {
      const raw = await invoke<RawStats>("load_stats");
      totalFiles           = raw.total_files            ?? 0;
      totalLaunched        = raw.total_launched         ?? 0;
      totalOriginalMb      = raw.total_original_mb      ?? 0;
      totalEncodedMb       = raw.total_encoded_mb       ?? 0;
      sumRatioPct          = raw.sum_ratio_pct          ?? 0;
      totalSecs            = raw.total_secs             ?? 0;
      lastUpdated          = raw.last_updated           ?? null;
      totalExtractedFiles  = raw.total_extracted_files  ?? 0;
      totalExtractLaunched = raw.total_extract_launched ?? 0;
      totalTracksExtracted = raw.total_tracks_extracted ?? 0;
      recordHeaviest       = raw.record_heaviest        ?? null;
      recordBestRatio      = raw.record_best_ratio      ?? null;
      encodeSessions       = raw.encode_sessions        ?? [];
      extractSessions      = raw.extract_sessions       ?? [];
    } catch (e) {
      console.error("Impossible de charger les statistiques :", e);
    }
  }

  // Enregistre les résultats d'une session d'encodage terminée.
  async function recordSummary(summary: EncodeSummary) {
    const okFiles = summary.files.filter(f => f.status === "ok" && f.original_mb > 0);

    totalLaunched += summary.files.length;

    if (okFiles.length > 0) {
      let sessionOriginalMb = 0;
      let sessionEncodedMb  = 0;

      for (const f of okFiles) {
        totalFiles      += 1;
        totalOriginalMb += f.original_mb;
        totalEncodedMb  += f.encoded_mb;
        const ratio      = ((f.original_mb - f.encoded_mb) / f.original_mb) * 100;
        sumRatioPct     += ratio;

        sessionOriginalMb += f.original_mb;
        sessionEncodedMb  += f.encoded_mb;

        // Record : fichier le plus lourd en entrée
        if (!recordHeaviest || f.original_mb > recordHeaviest.original_mb) {
          recordHeaviest = {
            name:        f.name ?? "—",
            original_mb: f.original_mb,
            encoded_mb:  f.encoded_mb,
            ratio_pct:   ratio,
          };
        }

        // Record : meilleur ratio de compression (plus grand %)
        if (!recordBestRatio || ratio > recordBestRatio.ratio_pct) {
          recordBestRatio = {
            name:        f.name ?? "—",
            original_mb: f.original_mb,
            encoded_mb:  f.encoded_mb,
            ratio_pct:   ratio,
          };
        }
      }

      // Historique sessions — on garde les MAX_SESSIONS dernières (les plus récentes en tête)
      const sessionRatio = sessionOriginalMb > 0
        ? ((sessionOriginalMb - sessionEncodedMb) / sessionOriginalMb) * 100
        : 0;
      const newSession: EncodeSession = {
        date:      new Date().toISOString(),
        files:     okFiles.length,
        ratio_pct: sessionRatio,
        saved_mb:  Math.max(0, sessionOriginalMb - sessionEncodedMb),
      };
      encodeSessions = [newSession, ...encodeSessions].slice(0, MAX_SESSIONS);
    }

    if (summary.total_secs > 0) {
      totalSecs += summary.total_secs;
    }

    lastUpdated = new Date().toISOString();
    await persist();
  }

  // Enregistre les résultats d'une session d'extraction terminée.
  async function recordExtraction(files: Array<{ sub_extract_status: string; streams: Array<{ codec_type: string; language: string }> }>, selSubs: Set<string>) {
    const launched = files.filter(f => f.sub_extract_status !== "none");
    const ok = launched.filter(f => f.sub_extract_status === "done");

    totalExtractLaunched += launched.length;
    totalExtractedFiles  += ok.length;

    let sessionTracks = 0;
    for (const f of ok) {
      const trackCount = f.streams.filter(
        s => s.codec_type === "subtitle" && selSubs.has(s.language)
      ).length;
      const n = Math.max(1, trackCount);
      totalTracksExtracted += n;
      sessionTracks        += n;
    }

    if (ok.length > 0) {
      const newSession: ExtractSession = {
        date:   new Date().toISOString(),
        files:  ok.length,
        tracks: sessionTracks,
      };
      extractSessions = [newSession, ...extractSessions].slice(0, MAX_SESSIONS);
    }

    lastUpdated = new Date().toISOString();
    await persist();
  }

  async function reset() {
    const e = emptyStats();
    totalFiles           = e.total_files;
    totalLaunched        = e.total_launched;
    totalOriginalMb      = e.total_original_mb;
    totalEncodedMb       = e.total_encoded_mb;
    sumRatioPct          = e.sum_ratio_pct;
    totalSecs            = e.total_secs;
    lastUpdated          = e.last_updated;
    totalExtractedFiles  = e.total_extracted_files;
    totalExtractLaunched = e.total_extract_launched;
    totalTracksExtracted = e.total_tracks_extracted;
    recordHeaviest       = e.record_heaviest;
    recordBestRatio      = e.record_best_ratio;
    encodeSessions       = e.encode_sessions;
    extractSessions      = e.extract_sessions;
    await persist();
  }

  return {
    get totalFiles()           { return totalFiles; },
    get totalLaunched()        { return totalLaunched; },
    get totalOriginalMb()      { return totalOriginalMb; },
    get totalEncodedMb()       { return totalEncodedMb; },
    get totalSecs()            { return totalSecs; },
    get totalSavedMb()         { return Math.max(0, totalOriginalMb - totalEncodedMb); },
    get avgInputMb()           { return totalFiles > 0 ? totalOriginalMb / totalFiles : 0; },
    get avgOutputMb()          { return totalFiles > 0 ? totalEncodedMb  / totalFiles : 0; },
    get avgRatioPct()          { return totalFiles > 0 ? sumRatioPct / totalFiles : 0; },
    get avgSecs()              { return totalFiles > 0 ? totalSecs / totalFiles : 0; },
    get lastUpdated()          { return lastUpdated; },
    // Records
    get recordHeaviest()       { return recordHeaviest; },
    get recordBestRatio()      { return recordBestRatio; },
    // Historiques
    get encodeSessions()       { return encodeSessions; },
    get extractSessions()      { return extractSessions; },
    // Extraction
    get totalExtractedFiles()  { return totalExtractedFiles; },
    get totalExtractLaunched() { return totalExtractLaunched; },
    get totalTracksExtracted() { return totalTracksExtracted; },
    get avgTracksPerFile()     { return totalExtractedFiles > 0 ? totalTracksExtracted / totalExtractedFiles : 0; },
    init,
    recordSummary,
    recordExtraction,
    reset,
  };
}

export const stats = createStats();
export type { EncodeSession, ExtractSession, FileRecord };