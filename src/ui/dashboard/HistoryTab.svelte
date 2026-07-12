<script lang="ts">
  import {
    History,
    Zap,
    Trophy,
    Weight,
    ArrowUpDown,
    ArrowUp,
    ArrowDown,
  } from "@lucide/svelte";
  import { Chart, Svg, Spline } from "layerchart";
  import { formatSize } from "$lib/utils";

  let {
    hasData,
    lastSession,
    recordHeaviest,
    recordBestRatio,
    encodeSessions,
  }: {
    hasData: boolean;
    lastSession: any | null;
    recordHeaviest: any | null;
    recordBestRatio: any | null;
    encodeSessions: any[];
  } = $props();

  function formattedDate(iso: string | null): string {
    if (!iso) return "--";
    const d = new Date(iso);
    return (
      d.toLocaleDateString("fr-FR", {
        day: "2-digit",
        month: "2-digit",
        year: "numeric",
      }) +
      " " +
      d.toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" })
    );
  }

  function shortDate(iso: string): string {
    const d = new Date(iso);
    return (
      d.toLocaleDateString("fr-FR", { day: "2-digit", month: "2-digit" }) +
      " " +
      d.toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" })
    );
  }

  function truncName(name: string, max = 40): string {
    if (!name) return "—";
    if (name.length <= max) return name;
    const ext = name.lastIndexOf(".");
    if (ext > 0) {
      const base = name.slice(0, ext);
      const e = name.slice(ext);
      return base.slice(0, max - e.length - 1) + "…" + e;
    }
    return name.slice(0, max - 1) + "…";
  }

  let encodeSparkData = $derived(
    [...encodeSessions].reverse().map((s, i) => ({ x: i, y: s.ratio_pct })),
  );

  // Sorting
  type SortCol = "date" | "files" | "saved_mb" | "ratio_pct";
  type SortDir = "asc" | "desc";

  let sortCol = $state<SortCol>("date");
  let sortDir = $state<SortDir>("desc");

  function toggleSort(col: SortCol) {
    if (sortCol === col) {
      sortDir = sortDir === "asc" ? "desc" : "asc";
    } else {
      sortCol = col;
      sortDir = col === "date" ? "desc" : "desc";
    }
  }

  let sortedSessions = $derived(
    [...encodeSessions].sort((a, b) => {
      let av: any, bv: any;
      if (sortCol === "date") {
        av = new Date(a.date).getTime();
        bv = new Date(b.date).getTime();
      } else {
        av = a[sortCol];
        bv = b[sortCol];
      }
      return sortDir === "asc" ? av - bv : bv - av;
    }),
  );

  let totalSessions = $derived(encodeSessions.length);
</script>

<section class="content-section">
  <header class="section-header">
    <div>
      <h2 class="section-title">Historique</h2>
      <p class="section-desc">Sessions d'encodage et records personnels.</p>
    </div>
  </header>

  {#if !hasData}
    <div class="empty-state">
      <History class="w-10 h-10" style="color:var(--color-subtext2);" />
      <span class="empty-label">Aucun historique disponible</span>
      <span class="empty-sub"
        >Lance un premier encodage pour voir apparaître l'historique.</span
      >
    </div>
  {:else}
    <!-- Rangée 1 : Dernière session + Total sessions -->
    <div class="row-top">
      {#if lastSession}
        <div class="top-card top-card--last">
          <div class="block-header">
            <Zap class="w-3.5 h-3.5" style="color:var(--color-accent);" />
            <span>Dernière session</span>
          </div>
          <div class="ls-date">{formattedDate(lastSession.date)}</div>
          <div class="ls-stats-row">
            <div class="ls-stat">
              <span class="ls-stat-label">Fichiers</span>
              <span class="ls-stat-value">{lastSession.files}</span>
            </div>
            <div class="ls-divider"></div>
            <div class="ls-stat">
              <span class="ls-stat-label">Économisé</span>
              <span class="ls-stat-value ls-stat-value--success"
                >{formatSize(lastSession.saved_mb)}</span
              >
            </div>
            <div class="ls-divider"></div>
            <div class="ls-stat">
              <span class="ls-stat-label">Compression</span>
              <span class="ls-stat-value ls-stat-value--accent"
                >−{lastSession.ratio_pct.toFixed(1)}%</span
              >
            </div>
          </div>
        </div>
      {/if}

      <div class="top-card top-card--total">
        <div class="block-header">
          <History class="w-3.5 h-3.5" style="color:var(--color-subtext2);" />
          <span>Total sessions</span>
        </div>
        <div class="total-number">{totalSessions}</div>
        <div class="total-sub">
          session{totalSessions > 1 ? "s" : ""} enregistrée{totalSessions > 1
            ? "s"
            : ""}
        </div>

        {#if encodeSessions.length >= 2}
          <div class="spark-inner">
            <div style="height: 44px; width: 100%;">
              <Chart
                data={encodeSparkData}
                x="x"
                y="y"
                yDomain={[0, null]}
                padding={{ top: 4, bottom: 4, left: 0, right: 0 }}
              >
                <Svg>
                  <Spline stroke="var(--color-accent)" strokeWidth={1.5} />
                </Svg>
              </Chart>
            </div>
            <span class="spark-label">Compression % / session</span>
          </div>
        {/if}
      </div>
    </div>

    <!-- Rangée 2 : Records -->
    {#if recordHeaviest || recordBestRatio}
      <div class="row-records">
        <div class="block-header" style="margin-bottom: 12px;">
          <Trophy class="w-3.5 h-3.5" style="color:var(--color-warning);" />
          <span>Records</span>
        </div>
        <div class="records-row">
          {#if recordHeaviest}
            <div class="record-card">
              <div class="record-header">
                <Weight
                  class="w-3.5 h-3.5 shrink-0"
                  style="color:var(--color-subtext2);"
                />
                <span class="record-label">Fichier le plus lourd</span>
              </div>
              <div class="record-name" title={recordHeaviest.name}>
                {truncName(recordHeaviest.name)}
              </div>
              <div class="record-values">
                <span class="record-val"
                  >{formatSize(recordHeaviest.original_mb)}</span
                >
                <span class="record-arrow">→</span>
                <span class="record-val record-val--accent"
                  >{formatSize(recordHeaviest.encoded_mb)}</span
                >
                <span class="record-ratio"
                  >−{recordHeaviest.ratio_pct.toFixed(1)}%</span
                >
              </div>
            </div>
          {/if}
          {#if recordBestRatio}
            <div class="record-card record-card--gold">
              <div class="record-header">
                <Trophy
                  class="w-3.5 h-3.5 shrink-0"
                  style="color:var(--color-warning);"
                />
                <span class="record-label">Meilleure compression</span>
              </div>
              <div class="record-name" title={recordBestRatio.name}>
                {truncName(recordBestRatio.name)}
              </div>
              <div class="record-values">
                <span class="record-val"
                  >{formatSize(recordBestRatio.original_mb)}</span
                >
                <span class="record-arrow">→</span>
                <span class="record-val record-val--accent"
                  >{formatSize(recordBestRatio.encoded_mb)}</span
                >
                <span class="record-ratio record-ratio--gold"
                  >−{recordBestRatio.ratio_pct.toFixed(1)}%</span
                >
              </div>
            </div>
          {/if}
        </div>
      </div>
    {/if}

    <!-- Rangée 3 : Tableau des sessions -->
    {#if encodeSessions.length > 0}
      <div class="row-table">
        <div class="block-header" style="margin-bottom: 12px;">
          <History class="w-3.5 h-3.5" style="color:var(--color-subtext2);" />
          <span>Toutes les sessions</span>
        </div>

        <div class="table-wrap">
          <table class="sess-table">
            <colgroup>
              <col style="width: 34%;" />
              <col style="width: 22%;" />
              <col style="width: 22%;" />
              <col style="width: 22%;" />
            </colgroup>
            <thead>
              <tr>
                <th class="th-left">
                  <button
                    class="th-sort-btn"
                    onclick={() => toggleSort("date")}
                  >
                    Date
                    <span class="sort-icon"
                      >{sortCol === "date"
                        ? sortDir === "asc"
                          ? "↑"
                          : "↓"
                        : "↕"}</span
                    >
                  </button>
                </th>
                <th class="th-right">
                  <button
                    class="th-sort-btn th-sort-right"
                    onclick={() => toggleSort("files")}
                  >
                    Fichiers
                    <span class="sort-icon"
                      >{sortCol === "files"
                        ? sortDir === "asc"
                          ? "↑"
                          : "↓"
                        : "↕"}</span
                    >
                  </button>
                </th>
                <th class="th-right">
                  <button
                    class="th-sort-btn th-sort-right"
                    onclick={() => toggleSort("saved_mb")}
                  >
                    Économisé
                    <span class="sort-icon"
                      >{sortCol === "saved_mb"
                        ? sortDir === "asc"
                          ? "↑"
                          : "↓"
                        : "↕"}</span
                    >
                  </button>
                </th>
                <th class="th-right">
                  <button
                    class="th-sort-btn th-sort-right"
                    onclick={() => toggleSort("ratio_pct")}
                  >
                    Compression
                    <span class="sort-icon"
                      >{sortCol === "ratio_pct"
                        ? sortDir === "asc"
                          ? "↑"
                          : "↓"
                        : "↕"}</span
                    >
                  </button>
                </th>
              </tr>
            </thead>
            <tbody>
              {#each sortedSessions as s}
                <tr
                  class="sess-row {s === encodeSessions[0]
                    ? 'sess-row--latest'
                    : ''}"
                >
                  <td class="td-date">{shortDate(s.date)}</td>
                  <td class="td-right"
                    >{s.files} fichier{s.files > 1 ? "s" : ""}</td
                  >
                  <td class="td-right td-saved">{formatSize(s.saved_mb)}</td>
                  <td class="td-right td-ratio">−{s.ratio_pct.toFixed(1)}%</td>
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
    box-sizing: border-box;
    min-width: 0;
  }
  .section-header {
    margin-bottom: 24px;
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

  /* Bloc header générique */
  .block-header {
    display: flex;
    align-items: center;
    gap: 7px;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--color-subtext2);
  }

  /* ── Rangée 1 : Dernière session + Total ── */
  .row-top {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 16px;
    margin-bottom: 20px;
    min-width: 0;
  }
  .top-card {
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 18px 22px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-width: 0;
  }
  .top-card--last {
    flex: 1;
    min-width: 0;
  }
  .top-card--total {
    width: 200px;
    flex-shrink: 0;
  }

  /* Dernière session */
  .ls-date {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    color: var(--color-subtext2);
    letter-spacing: 0.03em;
  }
  .ls-stats-row {
    display: flex;
    align-items: center;
  }
  .ls-stat {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 5px;
    align-items: center;
  }
  .ls-divider {
    width: 1px;
    height: 36px;
    background: var(--color-border);
    flex-shrink: 0;
  }
  .ls-stat-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-subtext2);
  }
  .ls-stat-value {
    font-family: "Geist Mono", monospace;
    font-size: 20px;
    font-weight: 700;
    color: var(--color-text);
    line-height: 1;
  }
  .ls-stat-value--success {
    color: var(--color-success);
  }
  .ls-stat-value--accent {
    color: var(--color-accent);
  }

  /* Total sessions */
  .total-number {
    font-family: "Geist Mono", monospace;
    font-size: 36px;
    font-weight: 700;
    color: var(--color-text);
    line-height: 1;
  }
  .total-sub {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext2);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .spark-inner {
    display: flex;
    flex-direction: column;
    gap: 5px;
    margin-top: 4px;
  }
  .spark-label {
    font-family: "Geist Mono", monospace;
    font-size: 8px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-subtext2);
  }

  /* ── Rangée 2 : Records ── */
  .row-records {
    margin-bottom: 20px;
  }
  .records-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }
  .record-card {
    padding: 16px 18px;
    border-radius: var(--radius-sm);
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }
  .record-card--gold {
    border-color: color-mix(
      in srgb,
      var(--color-warning) 30%,
      var(--color-border)
    );
    background: color-mix(in srgb, var(--color-warning) 5%, var(--color-panel));
  }
  .record-header {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .record-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-subtext);
  }
  .record-name {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    color: var(--color-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-weight: 500;
  }
  .record-values {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }
  .record-val {
    font-family: "Geist Mono", monospace;
    font-size: 12px;
    color: var(--color-subtext);
  }
  .record-val--accent {
    color: var(--color-accent);
    font-weight: 600;
  }
  .record-arrow {
    font-size: 10px;
    color: var(--color-subtext2);
  }
  .record-ratio {
    font-family: "Geist Mono", monospace;
    font-size: 16px;
    font-weight: 700;
    color: var(--color-success);
    margin-left: auto;
  }
  .record-ratio--gold {
    color: var(--color-warning);
  }

  /* ── Rangée 3 : Tableau ── */

  .table-wrap {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    max-height: 300px;
    overflow-y: overlay;
    position: relative;
  }
  .table-wrap::-webkit-scrollbar {
    width: 6px;
  }
  .table-wrap::-webkit-scrollbar-track {
    background: transparent;
  }
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
    text-align: left;
    font-size: 11px;
    border-collapse: collapse;
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
  .th-left {
    text-align: left;
  }
  .th-right {
    text-align: right;
  }

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
  .th-sort-btn:hover {
    color: var(--color-text);
  }
  .th-sort-right {
    margin-left: auto;
  }

  .sort-icon {
    font-size: 9px;
    opacity: 0.45;
    line-height: 1;
  }

  .sess-table tbody tr {
    height: 38px;
    border-bottom: 1px solid var(--color-border);
    transition: background 0.08s;
  }
  .sess-table tbody tr:last-child {
    border-bottom: none;
  }

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
  .td-right {
    text-align: right;
  }
  .td-date {
    color: var(--color-subtext2);
  }
  .td-saved {
    color: var(--color-success);
  }
  .td-ratio {
    font-weight: 700;
    color: var(--color-accent);
  }
</style>
