<script lang="ts">
  import { Subtitles, FileCheck2, History } from "@lucide/svelte";
  import { Chart, Svg, Spline } from "layerchart";

  let {
    hasExtractData,
    totalTracksExtracted,
    totalExtractedFiles,
    totalExtractLaunched,
    avgTracksPerFile,
    extractSessions,
  }: {
    hasExtractData: boolean;
    totalTracksExtracted: number;
    totalExtractedFiles: number;
    totalExtractLaunched: number;
    avgTracksPerFile: number;
    extractSessions: any[];
  } = $props();

  function shortDate(iso: string): string {
    const d = new Date(iso);
    return (
      d.toLocaleDateString("fr-FR", { day: "2-digit", month: "2-digit" }) +
      " " +
      d.toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" })
    );
  }

  let extractSparkData = $derived(
    [...extractSessions].reverse().map((s, i) => ({ x: i, y: s.tracks })),
  );

  let successRate = $derived(
    totalExtractLaunched > 0
      ? Math.round((totalExtractedFiles / totalExtractLaunched) * 100)
      : 0
  );

  type SortCol = "date" | "files" | "tracks";
  type SortDir = "asc" | "desc";
  let sortCol = $state<SortCol>("date");
  let sortDir = $state<SortDir>("desc");

  function toggleSort(col: SortCol) {
    if (sortCol === col) {
      sortDir = sortDir === "asc" ? "desc" : "asc";
    } else {
      sortCol = col;
      sortDir = "desc";
    }
  }

  let sortedSessions = $derived(
    [...extractSessions].sort((a, b) => {
      let av: any, bv: any;
      if (sortCol === "date") {
        av = new Date(a.date).getTime();
        bv = new Date(b.date).getTime();
      } else {
        av = a[sortCol];
        bv = b[sortCol];
      }
      return sortDir === "asc" ? av - bv : bv - av;
    })
  );
</script>

<section class="content-section">
  <header class="section-header">
    <div>
      <h2 class="section-title">Statistiques d'extraction</h2>
      <p class="section-desc">Résultats cumulés des extractions de pistes de sous-titres.</p>
    </div>
  </header>

  {#if !hasExtractData}
    <div class="empty-state">
      <Subtitles class="w-10 h-10" style="color:var(--color-subtext2);" />
      <span class="empty-label">Aucune extraction enregistrée</span>
      <span class="empty-sub">Utilise le bouton "Extraire" pour lancer une extraction de sous-titres.</span>
    </div>
  {:else}
    <!-- Hero -->
    <div class="hero-row">
      <div class="hero-card">
        <div class="hero-label">
          <Subtitles class="w-3.5 h-3.5" />
          Pistes extraites au total
        </div>
        <div class="hero-value">{totalTracksExtracted}</div>
        <div class="hero-sub">
          {totalExtractedFiles} fichier{totalExtractedFiles > 1 ? "s" : ""} traité{totalExtractedFiles > 1 ? "s" : ""} avec succès
        </div>
      </div>
      <div class="hero-card hero-card--muted">
        <div class="hero-label hero-label--muted">
          <FileCheck2 class="w-3.5 h-3.5" />
          Taux de succès
        </div>
        <div class="hero-value hero-value--muted">{successRate}%</div>
        <div class="hero-sub hero-sub--muted">{totalExtractedFiles} / {totalExtractLaunched} fichiers</div>
      </div>
    </div>

    <!-- Stats grid -->
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-icon-row">
          <FileCheck2 class="w-4 h-4 stat-icon" />
          <span class="stat-label">Fichiers traités</span>
        </div>
        <div class="stat-value">
          {totalExtractedFiles}
          <span class="stat-value-sub">/ {totalExtractLaunched}</span>
        </div>
      </div>
      <div class="stat-card accent-card">
        <div class="stat-icon-row">
          <Subtitles class="w-4 h-4" style="color:var(--color-accent);" />
          <span class="stat-label">Moy. pistes / fichier</span>
        </div>
        <div class="stat-value" style="color:var(--color-accent);">
          {avgTracksPerFile.toFixed(1)}
        </div>
      </div>
      <div class="stat-card stat-card--wide">
        <div class="stat-icon-row">
          <Subtitles class="w-4 h-4 stat-icon" />
          <span class="stat-label">Total pistes extraites</span>
        </div>
        <div class="stat-value">{totalTracksExtracted}</div>
      </div>
    </div>

    <!-- Historique sessions -->
    {#if extractSessions.length > 0}
      <div class="sessions-section">
        <div class="block-header">
          <History class="w-3.5 h-3.5" style="color:var(--color-subtext2);" />
          <span>Dernières sessions</span>
        </div>

        {#if extractSessions.length >= 2}
          <div class="spark-wrap">
            <div style="height: 56px;">
              <Chart
                data={extractSparkData}
                x="x"
                y="y"
                yDomain={[0, null]}
                padding={{ top: 6, bottom: 6, left: 0, right: 0 }}
              >
                <Svg>
                  <Spline stroke="var(--color-accent)" strokeWidth={2} />
                </Svg>
              </Chart>
            </div>
            <span class="spark-label">Pistes extraites par session</span>
          </div>
        {/if}

        <div class="table-wrap">
          <table class="sess-table">
            <colgroup>
              <col style="width: 42%;" />
              <col style="width: 29%;" />
              <col style="width: 29%;" />
            </colgroup>
            <thead>
              <tr>
                <th class="th-left">
                  <button class="th-sort-btn" onclick={() => toggleSort('date')}>
                    Date
                    <span class="sort-icon">{sortCol === 'date' ? (sortDir === 'asc' ? '↑' : '↓') : '↕'}</span>
                  </button>
                </th>
                <th class="th-right">
                  <button class="th-sort-btn th-sort-right" onclick={() => toggleSort('files')}>
                    Fichiers
                    <span class="sort-icon">{sortCol === 'files' ? (sortDir === 'asc' ? '↑' : '↓') : '↕'}</span>
                  </button>
                </th>
                <th class="th-right">
                  <button class="th-sort-btn th-sort-right" onclick={() => toggleSort('tracks')}>
                    Pistes extraites
                    <span class="sort-icon">{sortCol === 'tracks' ? (sortDir === 'asc' ? '↑' : '↓') : '↕'}</span>
                  </button>
                </th>
              </tr>
            </thead>
            <tbody>
              {#each sortedSessions as s, i}
                <tr class="sess-row {i === 0 && sortCol === 'date' && sortDir === 'desc' ? 'sess-row--latest' : ''}">
                  <td class="td-date">{shortDate(s.date)}</td>
                  <td class="td-right">{s.files} fichier{s.files > 1 ? "s" : ""}</td>
                  <td class="td-right td-tracks">{s.tracks} piste{s.tracks > 1 ? "s" : ""}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>
    {/if}
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
    margin: 0 0 8px;
  }
  .section-desc {
    font-size: 12px;
    color: var(--color-subtext);
    line-height: 1.6;
    margin: 0;
  }
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

  /* Hero */
  .hero-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    margin-bottom: 14px;
  }
  .hero-card {
    padding: 12px;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--color-accent) 8%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-accent) 25%, transparent);
  }
  .hero-card--muted {
    background: var(--color-panel);
    border-color: var(--color-border);
  }
  .hero-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 10px;
    font-family: "Geist Mono", monospace;
    color: color-mix(in srgb, var(--color-accent) 80%, var(--color-text));
    margin-bottom: 12px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .hero-label--muted { color: var(--color-subtext2); }
  .hero-value {
    font-family: "Geist Mono", monospace;
    font-size: 36px;
    font-weight: 700;
    line-height: 1;
    color: var(--color-accent);
    margin-bottom: 8px;
  }
  .hero-value--muted { color: var(--color-text); }
  .hero-sub {
    font-family: "Geist Mono", monospace;
    font-size: 12px;
    color: color-mix(in srgb, var(--color-accent) 55%, var(--color-subtext2));
  }
  .hero-sub--muted { color: var(--color-subtext2); }

  /* Stats grid */
  .stats-grid {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    gap: 12px;
    margin-bottom: 28px;
  }
  .stat-card {
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    background: var(--color-panel);
    border: 1px solid var(--color-border);
  }
  .stat-card--wide { grid-column: span 1; }
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
  .stat-value-sub {
    font-size: 12px;
    font-weight: 400;
    color: var(--color-subtext2);
    margin-left: 3px;
  }
  .accent-card {
    border-color: color-mix(in srgb, var(--color-accent) 30%, var(--color-border));
    background: color-mix(in srgb, var(--color-accent) 5%, var(--color-panel));
  }

  /* Sessions */
  .sessions-section { display: flex; flex-direction: column; }
  .block-header {
    display: flex;
    align-items: center;
    gap: 7px;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--color-subtext2);
    margin-bottom: 12px;
  }
  .spark-wrap {
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 14px 18px 10px;
    margin-bottom: 12px;
  }
  .spark-label {
    font-family: "Geist Mono", monospace;
    font-size: 8px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-subtext2);
  }
  .table-wrap {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    max-height: 300px;
    overflow-y: overlay;
    position: relative;
  }
  .table-wrap::-webkit-scrollbar { width: 6px; }
  .table-wrap::-webkit-scrollbar-track { background: transparent; }
  .table-wrap::-webkit-scrollbar-thumb {
    background: color-mix(in srgb, var(--color-subtext2) 40%, transparent);
    border-radius: 3px;
  }
  .table-wrap::-webkit-scrollbar-thumb:hover {
    background: color-mix(in srgb, var(--color-subtext2) 70%, transparent);
  }

  .sess-table {
    table-layout: fixed;
    width: 100%;
    border-collapse: collapse;
    font-size: 11px;
  }

  .sess-table thead {
    position: sticky;
    top: 0;
    z-index: 1;
    background: var(--color-surface);
  }

  .sess-table thead tr {
    border-bottom: 1px solid var(--color-border);
  }

  .sess-table th {
    padding: 5px 6px;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-subtext2);
    white-space: nowrap;
  }
  .th-left  { text-align: left; }
  .th-right { text-align: right; }

  .sess-table tbody tr {
    height: 38px;
    border-bottom: 1px solid var(--color-border);
    transition: background 0.08s;
  }
  .sess-table tbody tr:last-child { border-bottom: none; }

  .sess-row--latest td {
    background: color-mix(in srgb, var(--color-accent) 5%, transparent);
  }

  .sess-table td {
    padding: 0 6px;
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    color: var(--color-subtext);
    overflow: hidden;
    white-space: nowrap;
  }
  .td-right  { text-align: right; }
  .td-date   { color: var(--color-subtext2); }
  .td-tracks { font-weight: 700; color: var(--color-accent); }

  .th-sort-btn {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    background: transparent;
    border: none;
    cursor: pointer;
    font-family: "Geist Mono", monospace;
    font-size: inherit;
    font-weight: inherit;
    color: var(--color-subtext2);
    padding: 0;
    white-space: nowrap;
    transition: color 0.1s;
    letter-spacing: inherit;
    text-transform: inherit;
  }
  .th-sort-btn:hover { color: var(--color-text); }
  .th-sort-right { margin-left: auto; }

  .sort-icon {
    font-size: 9px;
    opacity: 0.45;
    line-height: 1;
  }
</style>