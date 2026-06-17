import { invoke } from "@tauri-apps/api/core";
import type { EncodeSummary } from "./encoder.svelte";

interface RawStats {
  total_files:        number; // nb de fichiers encodés avec succès
  total_original_mb:  number; // somme des tailles d'entrée
  total_encoded_mb:   number; // somme des tailles de sortie
  sum_ratio_pct:       number; // somme des ratios de compression individuels (pour la moyenne)
  last_updated:        string | null;
}

function emptyStats(): RawStats {
  return {
    total_files: 0,
    total_original_mb: 0,
    total_encoded_mb: 0,
    sum_ratio_pct: 0,
    last_updated: null,
  };
}

function createStats() {
  let totalFiles      = $state(0);
  let totalOriginalMb = $state(0);
  let totalEncodedMb  = $state(0);
  let sumRatioPct      = $state(0);
  let lastUpdated      = $state<string | null>(null);

  async function persist() {
    const raw: RawStats = {
      total_files: totalFiles,
      total_original_mb: totalOriginalMb,
      total_encoded_mb: totalEncodedMb,
      sum_ratio_pct: sumRatioPct,
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
      totalOriginalMb = raw.total_original_mb ?? 0;
      totalEncodedMb  = raw.total_encoded_mb  ?? 0;
      sumRatioPct      = raw.sum_ratio_pct     ?? 0;
      lastUpdated      = raw.last_updated      ?? null;
    } catch (e) {
      console.error("Impossible de charger les statistiques :", e);
    }
  }

  // Enregistre les résultats d'une session d'encodage terminée.
  // Seuls les fichiers réussis ("ok") comptent dans les stats.
  async function recordSummary(summary: EncodeSummary) {
    const okFiles = summary.files.filter(f => f.status === "ok" && f.original_mb > 0);
    if (okFiles.length === 0) return;

    for (const f of okFiles) {
      totalFiles += 1;
      totalOriginalMb += f.original_mb;
      totalEncodedMb  += f.encoded_mb;
      sumRatioPct += ((f.original_mb - f.encoded_mb) / f.original_mb) * 100;
    }
    lastUpdated = new Date().toISOString();
    await persist();
  }

  async function reset() {
    const e = emptyStats();
    totalFiles      = e.total_files;
    totalOriginalMb = e.total_original_mb;
    totalEncodedMb  = e.total_encoded_mb;
    sumRatioPct      = e.sum_ratio_pct;
    lastUpdated      = e.last_updated;
    await persist();
  }

  return {
    get totalFiles()      { return totalFiles; },
    get totalOriginalMb() { return totalOriginalMb; },
    get totalEncodedMb()  { return totalEncodedMb; },
    get totalSavedMb()    { return Math.max(0, totalOriginalMb - totalEncodedMb); },
    get avgInputMb()      { return totalFiles > 0 ? totalOriginalMb / totalFiles : 0; },
    get avgOutputMb()     { return totalFiles > 0 ? totalEncodedMb  / totalFiles : 0; },
    get avgRatioPct()     { return totalFiles > 0 ? sumRatioPct / totalFiles : 0; },
    get lastUpdated()     { return lastUpdated; },
    init,
    recordSummary,
    reset,
  };
}

export const stats = createStats();