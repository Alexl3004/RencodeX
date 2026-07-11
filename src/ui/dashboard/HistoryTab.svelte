<script lang="ts">
  import { History, Zap, Trophy, Weight } from "@lucide/svelte";
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
      d.toLocaleDateString("fr-FR", { day: "2-digit", month: "2-digit", year: "numeric" }) +
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
</script>

<section class="content-section">
  <header class="section-header">
    <div>
      <h2 class="section-title">Historique</h2>
      <p class="section-desc">Dernière session d'encodage et records personnels.</p>
    </div>
  </header>

  {#if !hasData}
    <div class="empty-state">
      <History class="w-10 h-10" style="color:var(--color-subtext2);" />
      <span class="empty-label">Aucun historique disponible</span>
      <span class="empty-sub">Lance un premier encodage pour voir apparaître l'historique.</span>
    </div>
  {:else}
    <div class="main-grid">
      <!-- Colonne gauche : dernière session + sessions list -->
      <div class="col-main">
        {#if lastSession}
          <div class="block-header">
            <Zap class="w-3.5 h-3.5" style="color:var(--color-accent);" />
            <span>Dernière session</span>
          </div>
          <div class="last-session-card">
            <div class="ls-date">{formattedDate(lastSession.date)}</div>
            <div class="ls-stats-row">
              <div class="ls-stat">
                <span class="ls-stat-label">Fichiers</span>
                <span class="ls-stat-value">{lastSession.files}</span>
              </div>
              <div class="ls-divider"></div>
              <div class="ls-stat">
                <span class="ls-stat-label">Économisé</span>
                <span class="ls-stat-value ls-stat-value--success">{formatSize(lastSession.saved_mb)}</span>
              </div>
              <div class="ls-divider"></div>
              <div class="ls-stat">
                <span class="ls-stat-label">Compression</span>
                <span class="ls-stat-value ls-stat-value--accent">−{lastSession.ratio_pct.toFixed(1)}%</span>
              </div>
            </div>
          </div>
        {/if}

        {#if encodeSessions.length > 0}
          <div class="block-header" style="margin-top: 28px;">
            <History class="w-3.5 h-3.5" style="color:var(--color-subtext2);" />
            <span>Toutes les sessions</span>
          </div>

          {#if encodeSessions.length >= 2}
            <div class="spark-wrap">
              <div style="height: 56px;">
                <Chart
                  data={encodeSparkData}
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
              <span class="spark-label">Compression % par session</span>
            </div>
          {/if}

          <div class="sessions-list">
            <div class="sessions-list-header">
              <span>Date</span>
              <span>Fichiers</span>
              <span>Économisé</span>
              <span>Compression</span>
            </div>
            {#each encodeSessions as s, i}
              <div class="session-row {i === 0 ? 'session-row--latest' : ''}">
                <span class="session-date">{shortDate(s.date)}</span>
                <span class="session-files">{s.files} fichier{s.files > 1 ? "s" : ""}</span>
                <span class="session-saved">{formatSize(s.saved_mb)}</span>
                <span class="session-ratio">−{s.ratio_pct.toFixed(1)}%</span>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Colonne droite : records -->
      {#if recordHeaviest || recordBestRatio}
        <div class="col-side">
          <div class="block-header">
            <Trophy class="w-3.5 h-3.5" style="color:var(--color-warning);" />
            <span>Records</span>
          </div>
          <div class="records-stack">
            {#if recordHeaviest}
              <div class="record-card">
                <div class="record-header">
                  <Weight class="w-3.5 h-3.5 shrink-0" style="color:var(--color-subtext2);" />
                  <span class="record-label">Fichier le plus lourd</span>
                </div>
                <div class="record-name" title={recordHeaviest.name}>{truncName(recordHeaviest.name)}</div>
                <div class="record-values">
                  <span class="record-val">{formatSize(recordHeaviest.original_mb)}</span>
                  <span class="record-arrow">→</span>
                  <span class="record-val record-val--accent">{formatSize(recordHeaviest.encoded_mb)}</span>
                </div>
                <div class="record-ratio">−{recordHeaviest.ratio_pct.toFixed(1)}%</div>
              </div>
            {/if}
            {#if recordBestRatio}
              <div class="record-card record-card--gold">
                <div class="record-header">
                  <Trophy class="w-3.5 h-3.5 shrink-0" style="color:var(--color-warning);" />
                  <span class="record-label">Meilleure compression</span>
                </div>
                <div class="record-name" title={recordBestRatio.name}>{truncName(recordBestRatio.name)}</div>
                <div class="record-values">
                  <span class="record-val">{formatSize(recordBestRatio.original_mb)}</span>
                  <span class="record-arrow">→</span>
                  <span class="record-val record-val--accent">{formatSize(recordBestRatio.encoded_mb)}</span>
                </div>
                <div class="record-ratio record-ratio--gold">−{recordBestRatio.ratio_pct.toFixed(1)}%</div>
              </div>
            {/if}
          </div>
        </div>
      {/if}
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

  /* Layout 2 colonnes */
  .main-grid {
    display: grid;
    grid-template-columns: 1fr 280px;
    gap: 32px;
    align-items: start;
  }
  .col-main { display: flex; flex-direction: column; }
  .col-side { display: flex; flex-direction: column; }

  /* Titres de bloc */
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

  /* Dernière session */
  .last-session-card {
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 20px 24px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
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
    gap: 6px;
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
    font-size: 22px;
    font-weight: 700;
    color: var(--color-text);
    line-height: 1;
  }
  .ls-stat-value--success { color: var(--color-success); }
  .ls-stat-value--accent { color: var(--color-accent); }

  /* Sparkline */
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

  /* Sessions list */
  .sessions-list {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }
  .sessions-list-header {
    display: grid;
    grid-template-columns: 1fr 80px 80px 80px;
    padding: 8px 16px;
    background: color-mix(in srgb, var(--color-muted) 40%, var(--color-panel));
    border-bottom: 1px solid var(--color-border);
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-subtext2);
  }
  .session-row {
    display: grid;
    grid-template-columns: 1fr 80px 80px 80px;
    align-items: center;
    padding: 10px 16px;
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    color: var(--color-subtext);
    border-bottom: 1px solid var(--color-border);
    transition: background 0.08s;
  }
  .session-row:last-child { border-bottom: none; }
  .session-row--latest {
    background: color-mix(in srgb, var(--color-accent) 5%, var(--color-panel));
    color: var(--color-text);
  }
  .session-date { color: var(--color-subtext2); }
  .session-saved { color: var(--color-success); }
  .session-ratio { font-weight: 700; color: var(--color-accent); }

  /* Records */
  .records-stack {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .record-card {
    padding: 18px 20px;
    border-radius: var(--radius-sm);
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .record-card--gold {
    border-color: color-mix(in srgb, var(--color-warning) 30%, var(--color-border));
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
  }
  .record-val {
    font-family: "Geist Mono", monospace;
    font-size: 12px;
    color: var(--color-subtext);
  }
  .record-val--accent { color: var(--color-accent); font-weight: 600; }
  .record-arrow { font-size: 10px; color: var(--color-subtext2); }
  .record-ratio {
    font-family: "Geist Mono", monospace;
    font-size: 20px;
    font-weight: 700;
    color: var(--color-success);
    margin-top: 4px;
  }
  .record-ratio--gold { color: var(--color-warning); }
</style>
