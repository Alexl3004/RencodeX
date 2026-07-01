import { invoke } from "@tauri-apps/api/core";
import type { EncodeSummary } from "./encoder.svelte";

interface RawStats {
  total_files:        number; // nb de fichiers encodés avec succès
  total_launched:     number; // nb total de fichiers lancés (ok + erreurs + annulés)
  total_original_mb:  number; // somme des tailles d'entrée
  total_encoded_mb:   number; // somme des tailles de sortie
  sum_ratio_pct:      number; // somme des ratios de compression individuels (pour la moyenne)
  total_secs:         number; // durée totale cumulée de tous les encodages
  last_updated:       string | null;
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
  };
}

function createStats() {
  let totalFiles      = $state(0);
  let totalLaunched   = $state(0);
  let totalOriginalMb = $state(0);
  let totalEncodedMb  = $state(0);
  let sumRatioPct     = $state(0);
  let totalSecs       = $state(0);
  let lastUpdated     = $state<string | null>(null);

  async function persist() {
    const raw: RawStats = {
      total_files: totalFiles,
      total_launched: totalLaunched,
      total_original_mb: totalOriginalMb,
      total_encoded_mb: totalEncodedMb,
      sum_ratio_pct: sumRatioPct,
      total_secs: totalSecs,
      last_updated: lastUpdated,
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
      totalFiles      = raw.total_files       ?? 0;
      totalLaunched   = raw.total_launched    ?? 0;
      totalOriginalMb = raw.total_original_mb ?? 0;
      totalEncodedMb  = raw.total_encoded_mb  ?? 0;
      sumRatioPct     = raw.sum_ratio_pct     ?? 0;
      totalSecs       = raw.total_secs        ?? 0;
      lastUpdated     = raw.last_updated      ?? null;
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

  async function reset() {
    const e = emptyStats();
    totalFiles      = e.total_files;
    totalLaunched   = e.total_launched;
    totalOriginalMb = e.total_original_mb;
    totalEncodedMb  = e.total_encoded_mb;
    sumRatioPct     = e.sum_ratio_pct;
    totalSecs       = e.total_secs;
    lastUpdated     = e.last_updated;
    await persist();
  }

  return {
    get totalFiles()      { return totalFiles; },
    get totalLaunched()   { return totalLaunched; },
    get totalOriginalMb() { return totalOriginalMb; },
    get totalEncodedMb()  { return totalEncodedMb; },
    get totalSecs()       { return totalSecs; },
    get totalSavedMb()    { return Math.max(0, totalOriginalMb - totalEncodedMb); },
    get avgInputMb()      { return totalFiles > 0 ? totalOriginalMb / totalFiles : 0; },
    get avgOutputMb()     { return totalFiles > 0 ? totalEncodedMb  / totalFiles : 0; },
    get avgRatioPct()     { return totalFiles > 0 ? sumRatioPct / totalFiles : 0; },
    // Débit moyen : Mo d'entrée traités par seconde sur l'ensemble des sessions
    get avgThroughputMbps() { return totalSecs > 0 ? totalOriginalMb / totalSecs : 0; },
    // Taux de succès en % sur tous les fichiers lancés
    get successRatePct()  { return totalLaunched > 0 ? (totalFiles / totalLaunched) * 100 : 0; },
    get lastUpdated()     { return lastUpdated; },
    init,
    recordSummary,
    reset,
  };
}

export const stats = createStats();