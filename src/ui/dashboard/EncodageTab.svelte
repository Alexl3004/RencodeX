<script lang="ts">
  import {
    TrendingDown,
    HardDrive,
    Clock,
    FileCheck2,
    Timer,
  } from "@lucide/svelte";
  import { formatSize } from "$lib/utils";

  let {
    loaded,
    hasData,
    totalSavedMb,
    totalOriginalMb,
    totalEncodedMb,
    avgInputMb,
    avgOutputMb,
    avgRatioPct,
    totalSecs,
    avgSecs,
    totalFiles,
  }: {
    loaded: boolean;
    hasData: boolean;
    totalSavedMb: number;
    totalOriginalMb: number;
    totalEncodedMb: number;
    avgInputMb: number;
    avgOutputMb: number;
    avgRatioPct: number;
    totalSecs: number;
    avgSecs: number;
    totalFiles: number;
  } = $props();

  function formatDuration(secs: number): string {
    if (secs <= 0) return "--";
    const h = Math.floor(secs / 3600);
    const m = Math.floor((secs % 3600) / 60);
    const s = Math.floor(secs % 60);
    if (h > 0) return `${h}h ${m.toString().padStart(2, "0")}m`;
    if (m > 0) return `${m}m ${s.toString().padStart(2, "0")}s`;
    return `${s}s`;
  }

  let barMaxVal = $derived(Math.max(avgInputMb, 0.001));
  let barInH = $derived(Math.round((avgInputMb / barMaxVal) * 120));
  let barOutH = $derived(Math.round((avgOutputMb / barMaxVal) * 120));
</script>

<section class="content-section">
  <header class="section-header">
    <div>
      <h2 class="section-title">Statistiques d'encodage</h2>
      <p class="section-desc">
        Résultats cumulés de tous les encodages réussis. Les fichiers en erreur
        ou annulés sont exclus des métriques de taille.
      </p>
    </div>
  </header>

  {#if !loaded}
    <div class="empty-state">
      <span class="empty-sub">Chargement…</span>
    </div>
  {:else if !hasData}
    <div class="empty-state">
      <HardDrive class="w-10 h-10" style="color:var(--color-subtext2);" />
      <span class="empty-label">Aucun encodage enregistré</span>
      <span class="empty-sub"
        >Lance un premier encodage pour voir apparaître les statistiques.</span
      >
    </div>
  {:else}
    <!-- Hero row : espace économisé + compression -->
    <div class="hero-row">
      <div class="hero-card hero-card--success">
        <div class="hero-label">
          <TrendingDown class="w-3.5 h-3.5" />
          Espace économisé total
        </div>
        <div class="hero-value">{formatSize(totalSavedMb)}</div>
        <div class="hero-sub">
          {formatSize(totalOriginalMb)} → {formatSize(totalEncodedMb)}
        </div>
      </div>
      <div class="hero-card hero-card--accent">
        <div class="hero-label hero-label--accent">
          <TrendingDown class="w-3.5 h-3.5" />
          Compression moyenne
        </div>
        <div class="hero-value hero-value--accent">
          −{avgRatioPct.toFixed(1)}%
        </div>
        <div class="hero-sub hero-sub--accent">
          {totalFiles} fichier{totalFiles > 1 ? "s" : ""} encodé{totalFiles > 1
            ? "s"
            : ""}
        </div>
      </div>
    </div>

    <!-- Stats grid 4 colonnes -->
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-icon-row">
          <Clock class="w-4 h-4 stat-icon" />
          <span class="stat-label">Durée totale</span>
        </div>
        <div class="stat-value">{formatDuration(totalSecs)}</div>
      </div>
      <div class="stat-card">
        <div class="stat-icon-row">
          <Timer class="w-4 h-4 stat-icon" />
          <span class="stat-label">Temps moyen / fichier</span>
        </div>
        <div class="stat-value">{formatDuration(avgSecs)}</div>
      </div>
      <div class="stat-card">
        <div class="stat-icon-row">
          <FileCheck2 class="w-4 h-4 stat-icon" />
          <span class="stat-label">Fichiers encodés</span>
        </div>
        <div class="stat-value">{totalFiles}</div>
      </div>
      <div class="stat-card">
        <div class="stat-icon-row">
          <TrendingDown class="w-4 h-4 stat-icon" />
          <span class="stat-label">Taille moy. entrée</span>
        </div>
        <div class="stat-value">{formatSize(avgInputMb)}</div>
      </div>
    </div>

    <!-- Bar chart large -->
    <div class="chart-section">
      <div class="chart-label">
        Taille moyenne par fichier — Entrée vs Sortie
      </div>
      <div class="bar-card">
        <div class="bar-chart-custom">
          <div class="bar-y-axis">
            <span class="bar-y-label">{formatSize(barMaxVal)}</span>
            <span class="bar-y-label">{formatSize(barMaxVal * 0.5)}</span>
            <span class="bar-y-label">0</span>
          </div>
          <div class="bar-columns">
            <div class="bar-col">
              <span class="bar-val">{formatSize(avgInputMb)}</span>
              <div class="bar-track">
                <div
                  class="bar-fill bar-fill--input"
                  style="height:{barInH}px;"
                ></div>
              </div>
              <span class="bar-col-label">Entrée</span>
            </div>
            <div class="bar-col">
              <span class="bar-val bar-val--accent"
                >{formatSize(avgOutputMb)}</span
              >
              <div class="bar-track">
                <div
                  class="bar-fill bar-fill--output"
                  style="height:{barOutH}px;"
                ></div>
              </div>
              <span class="bar-col-label">Sortie</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  {/if}
</section>

<style>
  .content-section {
    padding: 32px 40px;
    width: 100%;
  }
  .section-header {
    margin-bottom: 28px;
    padding-bottom: 14px;
    border-bottom: 1px solid var(--color-border);
  }
  .section-title {
    font-size: 18px;
    font-weight: 600;
    color: var(--color-text);
    letter-spacing: -0.02em;
    margin: 0 0 4px;
  }
  .section-desc {
    font-size: 12px;
    color: var(--color-subtext);
    line-height: 1.6;
    margin: 0;
  }

  /* Empty */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 80px 24px;
    gap: 10px;
    text-align: center;
  }
  .empty-label {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-subtext);
    margin-top: 6px;
  }
  .empty-sub {
    font-size: 12px;
    color: var(--color-subtext2);
    max-width: 320px;
    line-height: 1.6;
  }

  /* Hero row */
  .hero-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    margin-bottom: 14px;
  }
  .hero-card {
    padding: 12px;
    border-radius: var(--radius-sm);
    border: 1px solid;
    display: flex;
    flex-direction: column;
    align-items: center;
  }
  .hero-card--success {
    background: color-mix(in srgb, var(--color-success) 8%, transparent);
    border-color: color-mix(in srgb, var(--color-success) 25%, transparent);
  }
  .hero-card--accent {
    background: color-mix(in srgb, var(--color-accent) 8%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 25%, transparent);
  }
  .hero-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 10px;
    font-family: "Geist Mono", monospace;
    color: color-mix(in srgb, var(--color-success) 80%, var(--color-text));
    margin-bottom: 12px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .hero-label--accent {
    color: color-mix(in srgb, var(--color-accent) 80%, var(--color-text));
  }
  .hero-value {
    font-family: "Geist Mono", monospace;
    font-size: 36px;
    font-weight: 700;
    line-height: 1;
    color: var(--color-success);
    margin-bottom: 8px;
  }
  .hero-value--accent {
    color: var(--color-accent);
  }
  .hero-sub {
    font-family: "Geist Mono", monospace;
    font-size: 12px;
    color: color-mix(in srgb, var(--color-success) 55%, var(--color-subtext2));
  }
  .hero-sub--accent {
    color: color-mix(in srgb, var(--color-accent) 55%, var(--color-subtext2));
  }

  /* Stats grid */
  .stats-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 12px;
    margin-bottom: 14px;
  }
  .stat-card {
    padding: 6px 10px;
    display: flex;
    flex-direction: column;
    justify-content: center;
    border-radius: var(--radius-sm);
    background: var(--color-panel);
    border: 1px solid var(--color-border);
  }
  .stat-icon-row {
    display: flex;
    align-items: center;
    gap: 7px;
    margin-bottom: 12px;
  }
  .stat-value {
    font-family: "Geist Mono", monospace;
    font-size: 20px;
    font-weight: 700;
    line-height: 1;
    color: var(--color-text);
  }

  /* Bar chart */
  .chart-section {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .chart-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--color-subtext2);
  }
  .bar-card {
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 24px 28px 20px;
  }
  .bar-chart-custom {
    display: flex;
    align-items: stretch;
    gap: 16px;
  }
  .bar-y-axis {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    align-items: flex-end;
    padding-bottom: 24px;
  }
  .bar-y-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext2);
    white-space: nowrap;
  }
  .bar-columns {
    display: flex;
    gap: 32px;
    align-items: flex-end;
    flex: 1;
  }
  .bar-col {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    flex: 0 0 80px;
  }
  .bar-val {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
    white-space: nowrap;
  }
  .bar-val--accent {
    color: var(--color-accent);
  }
  .bar-track {
    width: 100%;
    height: 120px;
    display: flex;
    align-items: flex-end;
    border-bottom: 1px solid var(--color-border);
  }
  .bar-fill {
    width: 100%;
    border-radius: 4px 4px 0 0;
    min-height: 2px;
  }
  .bar-fill--input {
    background: var(--color-subtext);
    opacity: 0.45;
  }
  .bar-fill--output {
    background: var(--color-accent);
    opacity: 0.85;
  }
  .bar-col-label {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext2);
    padding-top: 4px;
  }
</style>
