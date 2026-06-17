// Svelte 5 runes-based store — statistiques globales d'encodage
// Les chiffres sont cumulés au fil des sessions et persistés en localStorage.

import type { EncodeSummary } from "./encoder.svelte";

const STORAGE_KEY = "rencodex-stats";

interface RawStats {
  totalFiles:       number; // nb de fichiers encodés avec succès
  totalOriginalMb:  number; // somme des tailles d'entrée
  totalEncodedMb:   number; // somme des tailles de sortie
  sumRatioPct:      number; // somme des ratios de compression individuels (pour la moyenne)
  lastUpdated:      string | null;
}

function emptyStats(): RawStats {
  return {
    totalFiles: 0,
    totalOriginalMb: 0,
    totalEncodedMb: 0,
    sumRatioPct: 0,
    lastUpdated: null,
  };
}

function createStats() {
  let totalFiles      = $state(0);
  let totalOriginalMb = $state(0);
  let totalEncodedMb  = $state(0);
  let sumRatioPct      = $state(0);
  let lastUpdated      = $state<string | null>(null);

  function persist() {
    if (typeof localStorage === "undefined") return;
    const raw: RawStats = { totalFiles, totalOriginalMb, totalEncodedMb, sumRatioPct, lastUpdated };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(raw));
  }

  function init() {
    if (typeof localStorage === "undefined") return;
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (!saved) return;
      const raw = JSON.parse(saved) as RawStats;
      totalFiles      = raw.totalFiles      ?? 0;
      totalOriginalMb = raw.totalOriginalMb ?? 0;
      totalEncodedMb  = raw.totalEncodedMb  ?? 0;
      sumRatioPct      = raw.sumRatioPct     ?? 0;
      lastUpdated      = raw.lastUpdated     ?? null;
    } catch {
      // ignore corrupted storage
    }
  }

  // Enregistre les résultats d'une session d'encodage terminée.
  // Seuls les fichiers réussis ("ok") comptent dans les stats.
  function recordSummary(summary: EncodeSummary) {
    const okFiles = summary.files.filter(f => f.status === "ok" && f.original_mb > 0);
    if (okFiles.length === 0) return;

    for (const f of okFiles) {
      totalFiles += 1;
      totalOriginalMb += f.original_mb;
      totalEncodedMb  += f.encoded_mb;
      sumRatioPct += ((f.original_mb - f.encoded_mb) / f.original_mb) * 100;
    }
    lastUpdated = new Date().toISOString();
    persist();
  }

  function reset() {
    const e = emptyStats();
    totalFiles      = e.totalFiles;
    totalOriginalMb = e.totalOriginalMb;
    totalEncodedMb  = e.totalEncodedMb;
    sumRatioPct      = e.sumRatioPct;
    lastUpdated      = e.lastUpdated;
    persist();
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
