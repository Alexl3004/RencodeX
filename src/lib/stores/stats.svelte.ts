import { invoke } from "@tauri-apps/api/core";
import type { EncodeSummary } from "./encoder.svelte";

interface RawStats {
  total_files:             number; // nb de fichiers encodés avec succès
  total_launched:          number; // nb total de fichiers lancés (ok + erreurs + annulés)
  total_original_mb:       number; // somme des tailles d'entrée
  total_encoded_mb:        number; // somme des tailles de sortie
  sum_ratio_pct:           number; // somme des ratios de compression individuels (pour la moyenne)
  total_secs:              number; // durée totale cumulée de tous les encodages
  last_updated:            string | null;
  total_extracted_files:   number; // nb de fichiers dont l'extraction a réussi
  total_extract_launched:  number; // nb total de fichiers soumis à l'extraction
  total_tracks_extracted:  number; // nb total de pistes extraites avec succès
}

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
    } catch (e) {
      console.error("Impossible de charger les statistiques :", e);
    }
  }

  // Enregistre les résultats d'une session d'encodage terminée.
  // Seuls les fichiers réussis ("ok") comptent dans les stats de taille/ratio.
  async function recordSummary(summary: EncodeSummary) {
    const okFiles = summary.files.filter(f => f.status === "ok" && f.original_mb > 0);

    totalLaunched += summary.files.length;

    if (okFiles.length > 0) {
      for (const f of okFiles) {
        totalFiles      += 1;
        totalOriginalMb += f.original_mb;
        totalEncodedMb  += f.encoded_mb;
        sumRatioPct     += ((f.original_mb - f.encoded_mb) / f.original_mb) * 100;
      }
    }

    if (summary.total_secs > 0) {
      totalSecs += summary.total_secs;
    }

    lastUpdated = new Date().toISOString();
    await persist();
  }

  // Enregistre les résultats d'une session d'extraction terminée.
  // files : liste des AppFile après extraction (on lit sub_extract_status et streams)
  async function recordExtraction(files: Array<{ sub_extract_status: string; streams: Array<{ codec_type: string; language: string }> }>, selSubs: Set<string>) {
    const launched = files.filter(f => f.sub_extract_status !== "none");
    const ok = launched.filter(f => f.sub_extract_status === "done");

    totalExtractLaunched += launched.length;
    totalExtractedFiles  += ok.length;

    // Compter les pistes réellement extraites (audio subs sélectionnées × fichiers ok)
    for (const f of ok) {
      const trackCount = f.streams.filter(
        s => s.codec_type === "subtitle" && selSubs.has(s.language)
      ).length;
      totalTracksExtracted += Math.max(1, trackCount);
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
    get avgThroughputMbps()    { return totalSecs > 0 ? totalOriginalMb / totalSecs : 0; },
    get successRatePct()       { return totalLaunched > 0 ? (totalFiles / totalLaunched) * 100 : 0; },
    get lastUpdated()          { return lastUpdated; },
    // Extraction
    get totalExtractedFiles()  { return totalExtractedFiles; },
    get totalExtractLaunched() { return totalExtractLaunched; },
    get totalTracksExtracted() { return totalTracksExtracted; },
    get extractSuccessRatePct() { return totalExtractLaunched > 0 ? (totalExtractedFiles / totalExtractLaunched) * 100 : 0; },
    init,
    recordSummary,
    recordExtraction,
    reset,
  };
}

export const stats = createStats();