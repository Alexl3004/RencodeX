<script lang="ts">
  import { stats } from "$lib/stores/stats.svelte";
  import { encoder } from "$lib/stores/encoder.svelte";
  import { formatSize } from "$lib/utils";
  import {
    X,
    HardDrive,
    TrendingDown,
    CheckCircle2,
    Clock,
    Gauge,
    FileCheck2,
    Subtitles,
    BarChart3,
  } from "@lucide/svelte";
  import { Popover, Portal } from "@skeletonlabs/skeleton-svelte";

  let { onClose }: { onClose?: () => void } = $props();

  // ── Stats dérivées ─────────────────────────────────────────────────────
  let totalFiles            = $derived(stats.totalFiles);
  let totalSavedMb          = $derived(stats.totalSavedMb);
  let totalOriginalMb       = $derived(stats.totalOriginalMb);
  let totalEncodedMb        = $derived(stats.totalEncodedMb);
  let avgInputMb            = $derived(stats.avgInputMb);
  let avgOutputMb           = $derived(stats.avgOutputMb);
  let avgRatioPct           = $derived(stats.avgRatioPct);
  let avgThroughputMbps     = $derived(stats.avgThroughputMbps);
  let successRatePct        = $derived(stats.successRatePct);
  let totalSecs             = $derived(stats.totalSecs);
  let lastUpdated           = $derived(stats.lastUpdated);
  let totalExtractedFiles   = $derived(stats.totalExtractedFiles);
  let totalExtractLaunched  = $derived(stats.totalExtractLaunched);
  let totalTracksExtracted  = $derived(stats.totalTracksExtracted);
  let extractSuccessRatePct = $derived(stats.extractSuccessRatePct);
  let hasData               = $derived(totalFiles > 0);
  let hasExtractData        = $derived(totalExtractLaunched > 0);

  // ── Navigation ─────────────────────────────────────────────────────────
  type SectionId = "encodage" | "extraction";
  let activeSection = $state<SectionId>("encodage");

  const SECTIONS: { id: SectionId; label: string; icon: any; desc: string }[] = [
    { id: "encodage",   label: "Encodage",   icon: BarChart3,  desc: "Compression · succès" },
    { id: "extraction", label: "Extraction", icon: Subtitles,  desc: "Sous-titres extraits" },
  ];

  // ── Donut SVG ──────────────────────────────────────────────────────────
  const R    = 32;
  const SW   = 8;
  const CX   = 42;
  const CY   = 42;
  const CIRC = 2 * Math.PI * R;
  function arcDash(pct: number) {
    const f = Math.max(0, Math.min(pct, 100)) / 100 * CIRC;
    return `${f} ${CIRC - f}`;
  }

  // ── Formatters ─────────────────────────────────────────────────────────
  function formattedDate(iso: string | null): string {
    if (!iso) return "--";
    const d = new Date(iso);
    return (
      d.toLocaleDateString("fr-FR", { day: "2-digit", month: "2-digit", year: "numeric" }) +
      " " + d.toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" })
    );
  }
  function formatDuration(secs: number): string {
    if (secs <= 0) return "--";
    const h = Math.floor(secs / 3600);
    const m = Math.floor((secs % 3600) / 60);
    const s = Math.floor(secs % 60);
    if (h > 0) return `${h}h ${m.toString().padStart(2, "0")}m`;
    if (m > 0) return `${m}m ${s.toString().padStart(2, "0")}s`;
    return `${s}s`;
  }
  function formatThroughput(mbps: number): string {
    if (mbps <= 0) return "--";
    return mbps < 1 ? `${(mbps * 1024).toFixed(0)} KB/s` : `${mbps.toFixed(1)} MB/s`;
  }

  // ── Bar chart SVG simple ───────────────────────────────────────────────
  // Hauteur max des barres en px
  const BAR_H  = 72;
  const BAR_W  = 44;
  const GAP    = 20;
  const SVG_W  = BAR_W * 2 + GAP + 48; // espace pour labels gauche
  const SVG_H  = BAR_H + 28;            // + espace label bas

  let barScale = $derived(avgInputMb > 0 ? BAR_H / (avgInputMb * 1.1) : 0);
  let inH      = $derived(Math.round(avgInputMb  * barScale));
  let outH     = $derived(Math.round(avgOutputMb * barScale));
  // positions x des barres (laisser 40px à gauche pour axe Y)
  const X_OFF  = 40;
  let x1       = X_OFF;
  let x2       = X_OFF + BAR_W + GAP;
</script>

<div class="page-root" class:page-root--horizontal={encoder.innerNavLayout === "horizontal"}>

  <!-- ── Sidebar ─────────────────────────────────────────────────────────── -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <span class="sidebar-title">Dashboard</span>
      <span class="sidebar-sub">Statistiques cumulées</span>
    </div>

    <nav class="sidebar-nav" aria-label="Sections">
      {#each SECTIONS as sec}
        <button
          type="button"
          class="nav-item {activeSection === sec.id ? 'nav-item--active' : ''}"
          onclick={() => (activeSection = sec.id)}
          aria-current={activeSection === sec.id ? "page" : undefined}
        >
          <div class="nav-item-icon">
            <sec.icon class="w-3.5 h-3.5" />
          </div>
          <div class="nav-item-text">
            <span class="nav-item-label">{sec.label}</span>
            <span class="nav-item-desc">{sec.desc}</span>
          </div>
          {#if activeSection === sec.id}
            <div class="nav-item-indicator" aria-hidden="true"></div>
          {/if}
        </button>
      {/each}
    </nav>

    <!-- Footer sidebar -->
    <div class="sidebar-footer">
    </div>
  </aside>

  <!-- ── Content ─────────────────────────────────────────────────────────── -->
  <div class="content-panel">

    <!-- ════ ENCODAGE ════ -->
    {#if activeSection === "encodage"}
      <section class="content-section">
        <header class="section-header">
          <div>
            <h2 class="section-title">Statistiques d'encodage</h2>
            <p class="section-desc">
              Résultats cumulés de tous les encodages réussis. Les fichiers en erreur ou annulés sont exclus des métriques de taille.
            </p>
          </div>
        </header>

        {#if !hasData}
          <div class="empty-state">
            <HardDrive class="w-8 h-8" style="color:var(--color-subtext2);" />
            <span class="empty-label">Aucun encodage enregistré</span>
            <span class="empty-sub">Lance un premier encodage pour voir apparaître les statistiques.</span>
          </div>
        {:else}

          <!-- Hero espace économisé -->
          <div class="hero-card">
            <div class="hero-label">
              <TrendingDown class="w-3 h-3" />
              Espace économisé total
            </div>
            <div class="hero-value">{formatSize(totalSavedMb)}</div>
            <div class="hero-sub">{formatSize(totalOriginalMb)} → {formatSize(totalEncodedMb)}</div>
          </div>

          <!-- Charts row -->
          <div class="charts-row">

            <!-- Bar chart SVG : Entrée vs Sortie -->
            <div class="field-block">
              <div class="field-label">Taille moyenne par fichier</div>
              <div class="bar-card">
                <svg width={SVG_W} height={SVG_H} class="bar-svg">
                  <!-- Ligne de base -->
                  <line x1={X_OFF} y1={BAR_H} x2={SVG_W} y2={BAR_H}
                    stroke="var(--color-border)" stroke-width="1" />

                  <!-- Barre Entrée -->
                  <rect
                    x={x1} y={BAR_H - inH}
                    width={BAR_W} height={inH}
                    rx="3"
                    fill="var(--color-subtext)"
                    opacity="0.5"
                  />
                  <!-- Barre Sortie -->
                  <rect
                    x={x2} y={BAR_H - outH}
                    width={BAR_W} height={outH}
                    rx="3"
                    fill="var(--color-accent)"
                    opacity="0.85"
                  />

                  <!-- Labels valeurs au-dessus -->
                  <text x={x1 + BAR_W / 2} y={BAR_H - inH - 5}
                    text-anchor="middle"
                    style="font-size:9px; font-family:monospace; fill:var(--color-subtext); font-weight:400;">
                    {formatSize(avgInputMb)}
                  </text>
                  <text x={x2 + BAR_W / 2} y={BAR_H - outH - 5}
                    text-anchor="middle"
                    style="font-size:9px; font-family:monospace; fill:var(--color-accent); font-weight:400;">
                    {formatSize(avgOutputMb)}
                  </text>

                  <!-- Labels catégories en bas -->
                  <text x={x1 + BAR_W / 2} y={BAR_H + 16}
                    text-anchor="middle"
                    style="font-size:9px; font-family:monospace; fill:var(--color-subtext2); font-weight:400;">
                    Entrée
                  </text>
                  <text x={x2 + BAR_W / 2} y={BAR_H + 16}
                    text-anchor="middle"
                    style="font-size:9px; font-family:monospace; fill:var(--color-subtext2); font-weight:400;">
                    Sortie
                  </text>

                  <!-- Axe Y : quelques repères -->
                  {#each [0.25, 0.5, 0.75, 1.0] as t}
                    {@const yy = BAR_H - BAR_H * t}
                    {@const val = avgInputMb * 1.1 * t}
                    <line x1={X_OFF - 3} y1={yy} x2={SVG_W} y2={yy}
                      stroke="var(--color-border)" stroke-width="0.5" stroke-dasharray="2 3" />
                    <text x={X_OFF - 5} y={yy + 3}
                      text-anchor="end"
                      style="font-size:7px; font-family:monospace; fill:var(--color-subtext2);">
                      {formatSize(val)}
                    </text>
                  {/each}
                </svg>
              </div>
            </div>

            <!-- Donut taux de succès -->
            <div class="field-block">
              <div class="field-label">Taux de succès</div>
              <div class="donut-card">
                <svg width="84" height="84" viewBox="0 0 84 84">
                  <circle cx={CX} cy={CY} r={R} fill="none"
                    stroke="var(--color-surface)" stroke-width={SW} />
                  <circle cx={CX} cy={CY} r={R} fill="none"
                    stroke="var(--color-success)"
                    stroke-width={SW}
                    stroke-dasharray={arcDash(successRatePct)}
                    stroke-dashoffset={CIRC * 0.25}
                    stroke-linecap="round" />
                  <text x={CX} y={CY - 2} text-anchor="middle" dominant-baseline="middle"
                    style="font-size:13px; font-weight:700; font-family:monospace; fill:var(--color-text);">
                    {successRatePct.toFixed(0)}%
                  </text>
                  <text x={CX} y={CY + 13} text-anchor="middle"
                    style="font-size:8px; font-family:monospace; fill:var(--color-subtext2);">
                    {totalFiles}/{stats.totalLaunched}
                  </text>
                </svg>
                <span class="donut-label">Encodage</span>
              </div>
            </div>

          </div>

          <!-- Grille stats secondaires -->
          <div class="stats-grid">
            <div class="stat-card">
              <div class="stat-icon-row">
                <Clock class="w-3.5 h-3.5 stat-icon" />
                <span class="stat-label">Durée totale</span>
              </div>
              <div class="stat-value">{formatDuration(totalSecs)}</div>
            </div>
            <div class="stat-card">
              <div class="stat-icon-row">
                <Gauge class="w-3.5 h-3.5 stat-icon" />
                <span class="stat-label">Débit moyen</span>
              </div>
              <div class="stat-value">{formatThroughput(avgThroughputMbps)}</div>
            </div>
            <div class="stat-card">
              <div class="stat-icon-row">
                <FileCheck2 class="w-3.5 h-3.5 stat-icon" />
                <span class="stat-label">Fichiers encodés</span>
              </div>
              <div class="stat-value">{totalFiles}</div>
            </div>
            <div class="stat-card accent-card">
              <div class="stat-icon-row">
                <TrendingDown class="w-3.5 h-3.5" style="color:var(--color-accent);" />
                <span class="stat-label">Compression moy.</span>
              </div>
              <div class="stat-value" style="color:var(--color-accent);">−{avgRatioPct.toFixed(1)}%</div>
            </div>
          </div>

        {/if}
      </section>

    <!-- ════ EXTRACTION ════ -->
    {:else if activeSection === "extraction"}
      <section class="content-section">
        <header class="section-header">
          <div>
            <h2 class="section-title">Statistiques d'extraction</h2>
            <p class="section-desc">
              Résultats cumulés des extractions de pistes de sous-titres.
            </p>
          </div>
        </header>

        {#if !hasExtractData}
          <div class="empty-state">
            <Subtitles class="w-8 h-8" style="color:var(--color-subtext2);" />
            <span class="empty-label">Aucune extraction enregistrée</span>
            <span class="empty-sub">Utilise le bouton "Extraire" pour lancer une extraction de sous-titres.</span>
          </div>
        {:else}

          <!-- Hero pistes extraites -->
          <div class="hero-card hero-card--accent">
            <div class="hero-label hero-label--accent">
              <Subtitles class="w-3 h-3" />
              Pistes extraites au total
            </div>
            <div class="hero-value hero-value--accent">{totalTracksExtracted}</div>
            <div class="hero-sub">{totalExtractedFiles} fichier{totalExtractedFiles > 1 ? "s" : ""} traité{totalExtractedFiles > 1 ? "s" : ""} avec succès</div>
          </div>

          <!-- Charts row extraction -->
          <div class="charts-row">
            <!-- Donut taux extraction -->
            <div class="field-block">
              <div class="field-label">Taux de succès</div>
              <div class="donut-card">
                <svg width="84" height="84" viewBox="0 0 84 84">
                  <circle cx={CX} cy={CY} r={R} fill="none"
                    stroke="var(--color-surface)" stroke-width={SW} />
                  <circle cx={CX} cy={CY} r={R} fill="none"
                    stroke="var(--color-accent)"
                    stroke-width={SW}
                    stroke-dasharray={arcDash(extractSuccessRatePct)}
                    stroke-dashoffset={CIRC * 0.25}
                    stroke-linecap="round" />
                  <text x={CX} y={CY - 2} text-anchor="middle" dominant-baseline="middle"
                    style="font-size:13px; font-weight:700; font-family:monospace; fill:var(--color-text);">
                    {extractSuccessRatePct.toFixed(0)}%
                  </text>
                  <text x={CX} y={CY + 13} text-anchor="middle"
                    style="font-size:8px; font-family:monospace; fill:var(--color-subtext2);">
                    {totalExtractedFiles}/{totalExtractLaunched}
                  </text>
                </svg>
                <span class="donut-label">Extraction</span>
              </div>
            </div>
          </div>

          <!-- Stats extraction -->
          <div class="stats-grid">
            <div class="stat-card">
              <div class="stat-icon-row">
                <FileCheck2 class="w-3.5 h-3.5 stat-icon" />
                <span class="stat-label">Fichiers traités</span>
              </div>
              <div class="stat-value">
                {totalExtractedFiles}
                <span class="stat-value-sub">/ {totalExtractLaunched}</span>
              </div>
            </div>
            <div class="stat-card accent-card">
              <div class="stat-icon-row">
                <CheckCircle2 class="w-3.5 h-3.5" style="color:var(--color-accent);" />
                <span class="stat-label">Succès</span>
              </div>
              <div class="stat-value" style="color:var(--color-accent);">
                {extractSuccessRatePct.toFixed(0)}%
              </div>
            </div>
            <div class="stat-card col-span-2">
              <div class="stat-icon-row">
                <Subtitles class="w-3.5 h-3.5 stat-icon" />
                <span class="stat-label">Pistes extraites (total)</span>
              </div>
              <div class="stat-value">{totalTracksExtracted}</div>
            </div>
          </div>

        {/if}
      </section>
    {/if}

    <!-- Réinitialiser les stats -->
    <div class="content-reset-bar">
      <Popover positioning={{ placement: "top-start" }}>
        <Popover.Trigger class="reset-btn">
          Réinitialiser les statistiques
        </Popover.Trigger>
        <Portal>
          <Popover.Positioner>
            <Popover.Content
              class="w-56 p-3 rounded-[var(--radius-md)] shadow-xl"
              style="background:var(--color-panel); border:1px solid var(--color-border);"
            >
              <Popover.Description class="text-[11px] leading-snug" style="color:var(--color-text);">
                Réinitialiser toutes les statistiques cumulées ?
              </Popover.Description>
              <div class="flex gap-2 mt-3">
                <Popover.CloseTrigger
                  onclick={() => stats.reset()}
                  class="btn btn-danger flex-1 justify-center font-mono text-[9px]"
                >Confirmer</Popover.CloseTrigger>
                <Popover.CloseTrigger class="btn btn-secondary flex-1 justify-center font-mono text-[9px]">
                  Annuler
                </Popover.CloseTrigger>
              </div>
            </Popover.Content>
          </Popover.Positioner>
        </Portal>
      </Popover>
    </div>

  </div>
</div>

<style>
  .page-root {
    display: flex;
    height: 100%;
    overflow: hidden;
    background: var(--color-surface);
  }

  /* ── Sidebar ── */
  .sidebar {
    width: 200px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    background: var(--color-panel);
    border-right: 1px solid var(--color-border);
    overflow: hidden;
  }

  .sidebar-header {
    padding: 20px 16px 14px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }
  .sidebar-title {
    display: block;
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text);
    letter-spacing: -0.01em;
  }
  .sidebar-sub {
    display: block;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    letter-spacing: 0.06em;
    color: var(--color-subtext2);
    margin-top: 3px;
    text-transform: uppercase;
  }

  .sidebar-nav {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .nav-item {
    position: relative;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid transparent;
    background: transparent;
    cursor: pointer;
    text-align: left;
    transition: background 0.1s, border-color 0.1s;
    width: 100%;
  }
  .nav-item:hover { background: color-mix(in srgb, var(--color-muted) 30%, transparent); }
  .nav-item--active {
    background: color-mix(in srgb, var(--color-accent) 9%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 22%, var(--color-border));
  }
  .nav-item-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-xs);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    color: var(--color-subtext);
    flex-shrink: 0;
    transition: background 0.1s, color 0.1s, border-color 0.1s;
  }
  .nav-item--active .nav-item-icon {
    background: color-mix(in srgb, var(--color-accent) 12%, var(--color-surface));
    border-color: color-mix(in srgb, var(--color-accent) 30%, var(--color-border));
    color: var(--color-accent);
  }
  .nav-item-text {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }
  .nav-item-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--color-subtext);
    line-height: 1.2;
    transition: color 0.1s;
  }
  .nav-item--active .nav-item-label {
    color: var(--color-accent);
    font-weight: 600;
  }
  .nav-item-desc {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext2);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .nav-item-indicator {
    position: absolute;
    right: 0;
    top: 50%;
    transform: translateY(-50%);
    width: 3px;
    height: 18px;
    border-radius: 2px 0 0 2px;
    background: var(--color-accent);
  }

  .sidebar-footer {
    padding: 12px 16px;
    border-top: 1px solid var(--color-border);
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  /* ── Content ── */
  .content-panel {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .content-reset-bar {
    margin-top: auto;
    padding: 14px 32px;
    border-top: 1px solid var(--color-border);
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }

  :global(.reset-btn) {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 5px 12px;
    font-size: 11px;
    font-family: "Geist Mono", monospace;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: transparent;
    color: var(--color-subtext2);
    cursor: pointer;
    transition: border-color 0.12s, color 0.12s;
  }
  :global(.reset-btn:hover) {
    border-color: var(--color-danger);
    color: var(--color-danger);
  }

  .content-section {
    padding: 28px 32px;
    max-width: 600px;
  }

  .section-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 28px;
    padding-bottom: 20px;
    border-bottom: 1px solid var(--color-border);
  }
  .section-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text);
    letter-spacing: -0.02em;
    margin: 0 0 6px;
  }
  .section-desc {
    font-size: 12px;
    color: var(--color-subtext);
    line-height: 1.6;
    max-width: 380px;
    margin: 0;
  }

  /* Empty state */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px 24px;
    gap: 8px;
    text-align: center;
  }
  .empty-label {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-subtext);
    margin-top: 4px;
  }
  .empty-sub {
    font-size: 11px;
    color: var(--color-subtext2);
    max-width: 280px;
    line-height: 1.5;
  }

  /* Hero card */
  .hero-card {
    padding: 16px 20px;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--color-success) 8%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-success) 25%, transparent);
    margin-bottom: 24px;
  }
  .hero-card--accent {
    background: color-mix(in srgb, var(--color-accent) 8%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 25%, transparent);
  }
  .hero-label {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 10px;
    font-family: "Geist Mono", monospace;
    color: color-mix(in srgb, var(--color-success) 80%, var(--color-text));
    margin-bottom: 8px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .hero-label--accent { color: color-mix(in srgb, var(--color-accent) 80%, var(--color-text)); }
  .hero-value {
    font-family: "Geist Mono", monospace;
    font-size: 32px;
    font-weight: 700;
    line-height: 1;
    color: var(--color-success);
    margin-bottom: 6px;
  }
  .hero-value--accent { color: var(--color-accent); }
  .hero-sub {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    color: color-mix(in srgb, var(--color-success) 55%, var(--color-subtext2));
  }

  /* Charts row */
  .charts-row {
    display: flex;
    gap: 24px;
    align-items: flex-start;
    margin-bottom: 24px;
  }

  /* Field block (label + contenu) */
  .field-block {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .field-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-subtext2);
  }

  /* Bar chart SVG wrapper */
  .bar-card {
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 14px 14px 10px;
    display: inline-flex;
  }
  .bar-svg { display: block; overflow: visible; }

  /* Donut */
  .donut-card {
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 14px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }
  .donut-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-subtext2);
  }

  /* Stats grid */
  .stats-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }
  .stat-card {
    padding: 12px 14px;
    border-radius: var(--radius-sm);
    background: var(--color-panel);
    border: 1px solid var(--color-border);
  }
  .col-span-2 { grid-column: span 2; }
  .stat-icon-row {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 8px;
  }
  .stat-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .stat-value {
    font-family: "Geist Mono", monospace;
    font-size: 18px;
    font-weight: 700;
    line-height: 1;
    color: var(--color-text);
  }
  .stat-value-sub {
    font-size: 11px;
    font-weight: 400;
    color: var(--color-subtext2);
    margin-left: 2px;
  }
  .accent-card {
    border-color: color-mix(in srgb, var(--color-accent) 30%, var(--color-border));
    background: color-mix(in srgb, var(--color-accent) 5%, var(--color-panel));
  }

  /* ── Layout horizontal ──────────────────────────────────────────────────── */
  .page-root--horizontal {
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  .page-root--horizontal .sidebar {
    width: 100%;
    height: auto;
    flex-direction: row;
    align-items: center;
    border-right: none;
    border-bottom: 1px solid var(--color-border);
    padding: 0 12px;
    gap: 0;
    overflow-x: auto;
    overflow-y: visible;
    flex-shrink: 0;
  }
  .page-root--horizontal .sidebar-header {
    display: none;
  }
  .page-root--horizontal .sidebar-nav {
    flex-direction: row;
    padding: 0;
    gap: 2px;
    overflow: visible;
    flex: 1;
    justify-content: center;
  }
  .page-root--horizontal .sidebar-footer {
    display: none;
  }
  .page-root--horizontal .nav-item {
    flex-direction: row;
    min-height: 36px;
    width: auto;
    padding: 6px 14px;
    gap: 6px;
    border-left: none;
    border-bottom: none;
    border-radius: var(--radius-sm);
    white-space: nowrap;
  }
  .page-root--horizontal .nav-item--active {
    border-left-color: transparent;
  }
  .page-root--horizontal .nav-item-indicator {
    display: none;
  }
  .page-root--horizontal .nav-item-text {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 6px;
  }
  .page-root--horizontal .nav-item-desc {
    display: none;
  }
  .page-root--horizontal .content-panel {
    flex: 1 1 0;
    min-height: 0;
    min-width: 0;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    align-items: center;
  }
  .page-root--horizontal .content-section {
    width: 100%;
    max-width: 720px;
  }
</style>