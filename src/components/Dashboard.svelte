<script lang="ts">
  import { stats } from "$lib/stores/stats.svelte";
  import { encoder } from "$lib/stores/encoder.svelte";
  import { formatSize } from "$lib/utils";
  import {
    HardDrive,
    TrendingDown,
    Clock,
    FileCheck2,
    Subtitles,
    BarChart3,
    Timer,
    Trophy,
    Weight,
    History,
    Zap,
  } from "@lucide/svelte";
  import { Popover, Portal } from "@skeletonlabs/skeleton-svelte";
  import { Chart, Svg, Spline } from "layerchart";
  import { onMount } from "svelte";


  onMount(() => {
    stats.init(); 
  });
  
  let { onClose }: { onClose?: () => void } = $props();

  // ── Stats dérivées ─────────────────────────────────────────────────────
  let totalFiles            = $derived(stats.totalFiles);
  let totalSavedMb          = $derived(stats.totalSavedMb);
  let totalOriginalMb       = $derived(stats.totalOriginalMb);
  let totalEncodedMb        = $derived(stats.totalEncodedMb);
  let avgInputMb            = $derived(stats.avgInputMb);
  let avgOutputMb           = $derived(stats.avgOutputMb);
  let avgRatioPct           = $derived(stats.avgRatioPct);
  let avgSecs               = $derived(stats.avgSecs);
  let totalSecs             = $derived(stats.totalSecs);
  let lastUpdated           = $derived(stats.lastUpdated);
  let totalExtractedFiles   = $derived(stats.totalExtractedFiles);
  let totalExtractLaunched  = $derived(stats.totalExtractLaunched);
  let totalTracksExtracted  = $derived(stats.totalTracksExtracted);
  let avgTracksPerFile      = $derived(stats.avgTracksPerFile);
  let recordHeaviest        = $derived(stats.recordHeaviest);
  let recordBestRatio       = $derived(stats.recordBestRatio);
  let encodeSessions        = $derived(stats.encodeSessions);
  let extractSessions       = $derived(stats.extractSessions);
  let hasData               = $derived(totalFiles > 0);
  let lastSession           = $derived(encodeSessions.length > 0 ? encodeSessions[0] : null);
  let hasExtractData        = $derived(totalExtractLaunched > 0);

  // ── Navigation ─────────────────────────────────────────────────────────
  type SectionId = "encodage" | "historique" | "extraction";
  let activeSection = $state<SectionId>("encodage");

  const SECTIONS: { id: SectionId; label: string; icon: any; desc: string }[] = [
    { id: "encodage",   label: "Encodage",   icon: BarChart3,  desc: "Compression · succès" },
    { id: "historique", label: "Historique", icon: History,    desc: "Sessions · records" },
    { id: "extraction", label: "Extraction", icon: Subtitles,  desc: "Sous-titres extraits" },
  ];

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

  // ── Bar chart custom : hauteurs des barres ────────────────────────────
  let barMaxVal  = $derived(Math.max(avgInputMb, 0.001));
  let barInH     = $derived(Math.round((avgInputMb  / barMaxVal) * 80));
  let barOutH    = $derived(Math.round((avgOutputMb / barMaxVal) * 80));
  let encodeSparkData = $derived(
    [...encodeSessions].reverse().map((s, i) => ({ x: i, y: s.ratio_pct }))
  );

  // ── LayerChart : sparkline sessions extraction (tracks) ────────────────
  let extractSparkData = $derived(
    [...extractSessions].reverse().map((s, i) => ({ x: i, y: s.tracks }))
  );

  // ── Helpers ────────────────────────────────────────────────────────────
  function truncName(name: string, max = 32): string {
    if (!name) return "—";
    if (name.length <= max) return name;
    const ext = name.lastIndexOf(".");
    if (ext > 0) {
      const base = name.slice(0, ext);
      const e    = name.slice(ext);
      return base.slice(0, max - e.length - 1) + "…" + e;
    }
    return name.slice(0, max - 1) + "…";
  }

  function shortDate(iso: string): string {
    const d = new Date(iso);
    return d.toLocaleDateString("fr-FR", { day: "2-digit", month: "2-digit" })
      + " " + d.toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" });
  }
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
        {#if !stats.loaded}
          <div class="empty-state">
            <span class="empty-sub">Chargement…</span>
          </div>
        {:else if !hasData}
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

            <!-- Bar chart LayerChart : Entrée vs Sortie -->
            <div class="field-block">
              <div class="field-label">Taille moyenne par fichier</div>
              <div class="bar-card">
                <div class="bar-chart-custom">
                  <!-- Axe Y labels -->
                  <div class="bar-y-axis">
                    <span class="bar-y-label">{formatSize(barMaxVal)}</span>
                    <span class="bar-y-label">{formatSize(barMaxVal * 0.5)}</span>
                    <span class="bar-y-label">0</span>
                  </div>
                  <!-- Barres -->
                  <div class="bar-columns">
                    <div class="bar-col">
                      <span class="bar-val">{formatSize(avgInputMb)}</span>
                      <div class="bar-track">
                        <div class="bar-fill bar-fill--input" style="height:{barInH}px;"></div>
                      </div>
                      <span class="bar-col-label">Entrée</span>
                    </div>
                    <div class="bar-col">
                      <span class="bar-val bar-val--accent">{formatSize(avgOutputMb)}</span>
                      <div class="bar-track">
                        <div class="bar-fill bar-fill--output" style="height:{barOutH}px;"></div>
                      </div>
                      <span class="bar-col-label">Sortie</span>
                    </div>
                  </div>
                </div>
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
                <Timer class="w-3.5 h-3.5 stat-icon" />
                <span class="stat-label">Temps moyen / fichier</span>
              </div>
              <div class="stat-value">{formatDuration(avgSecs)}</div>
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

    <!-- ════ HISTORIQUE ════ -->
    {:else if activeSection === "historique"}
      <section class="content-section">
        <header class="section-header">
          <div>
            <h2 class="section-title">Historique</h2>
            <p class="section-desc">
              Dernière session d'encodage et records personnels.
            </p>
          </div>
        </header>

        {#if !hasData}
          <div class="empty-state">
            <History class="w-8 h-8" style="color:var(--color-subtext2);" />
            <span class="empty-label">Aucun historique disponible</span>
            <span class="empty-sub">Lance un premier encodage pour voir apparaître l'historique.</span>
          </div>
        {:else}

          <!-- ── Dernière session ───────────────────────────────────────────── -->
          {#if lastSession}
            <div class="subsection-title">
              <Zap class="w-3 h-3" />
              Dernière session
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

          <!-- ── Records ───────────────────────────────────────────────────── -->
          {#if recordHeaviest || recordBestRatio}
            <div class="subsection-title" style="margin-top: 24px;">
              <Trophy class="w-3 h-3" />
              Records
            </div>
            <div class="records-grid">
              {#if recordHeaviest}
                <div class="record-card">
                  <div class="record-header">
                    <Weight class="w-3 h-3 shrink-0" style="color:var(--color-subtext2);" />
                    <span class="record-label">Fichier le plus lourd</span>
                  </div>
                  <div class="record-name" title={recordHeaviest.name}>{truncName(recordHeaviest.name)}</div>
                  <div class="record-values">
                    <span class="record-val">{formatSize(recordHeaviest.original_mb)}</span>
                    <span class="record-arrow">→</span>
                    <span class="record-val record-val--accent">{formatSize(recordHeaviest.encoded_mb)}</span>
                    <span class="record-ratio">−{recordHeaviest.ratio_pct.toFixed(1)}%</span>
                  </div>
                </div>
              {/if}
              {#if recordBestRatio}
                <div class="record-card record-card--gold">
                  <div class="record-header">
                    <Trophy class="w-3 h-3 shrink-0" style="color:var(--color-warning);" />
                    <span class="record-label">Meilleure compression</span>
                  </div>
                  <div class="record-name" title={recordBestRatio.name}>{truncName(recordBestRatio.name)}</div>
                  <div class="record-values">
                    <span class="record-val">{formatSize(recordBestRatio.original_mb)}</span>
                    <span class="record-arrow">→</span>
                    <span class="record-val record-val--accent">{formatSize(recordBestRatio.encoded_mb)}</span>
                    <span class="record-ratio record-ratio--gold">−{recordBestRatio.ratio_pct.toFixed(1)}%</span>
                  </div>
                </div>
              {/if}
            </div>
          {/if}

          <!-- ── Toutes les sessions ────────────────────────────────────────── -->
          {#if encodeSessions.length > 0}
            <div class="subsection-title" style="margin-top: 24px;">
              <History class="w-3 h-3" />
              Sessions
            </div>
            <div class="sessions-block">
              {#if encodeSessions.length >= 2}
                <div class="spark-wrap">
                  <div style="height: 44px;">
                    <Chart
                      data={encodeSparkData}
                      x="x"
                      y="y"
                      yDomain={[0, null]}
                      padding={{ top: 4, bottom: 4, left: 0, right: 0 }}
                    >
                      <Svg>
                        <Spline stroke="var(--color-accent)" strokeWidth={2} />
                      </Svg>
                    </Chart>
                  </div>
                  <span class="spark-label">Compression % / session</span>
                </div>
              {/if}
              <div class="sessions-list">
                {#each encodeSessions as s, i}
                  <div class="session-row {i === 0 ? 'session-row--latest' : ''}">
                    <span class="session-date">{shortDate(s.date)}</span>
                    <span class="session-files">{s.files} fichier{s.files > 1 ? "s" : ""}</span>
                    <span class="session-saved">{formatSize(s.saved_mb)} éco.</span>
                    <span class="session-ratio">−{s.ratio_pct.toFixed(1)}%</span>
                  </div>
                {/each}
              </div>
            </div>
          {/if}

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
                <Subtitles class="w-3.5 h-3.5" style="color:var(--color-accent);" />
                <span class="stat-label">Moy. pistes / fichier</span>
              </div>
              <div class="stat-value" style="color:var(--color-accent);">
                {avgTracksPerFile.toFixed(1)}
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

          <!-- ── Historique sessions extraction ─────────────────────────── -->
          {#if extractSessions.length > 0}
            <div class="subsection-title">
              <History class="w-3 h-3" />
              Dernières sessions
            </div>
            <div class="sessions-block">
              {#if extractSessions.length >= 2}
                <div class="spark-wrap">
                  <div style="height: 44px;">
                    <Chart
                      data={extractSparkData}
                      x="x"
                      y="y"
                      yDomain={[0, null]}
                      padding={{ top: 4, bottom: 4, left: 0, right: 0 }}
                    >
                      <Svg>
                        <Spline stroke="var(--color-accent)" strokeWidth={2} />
                      </Svg>
                    </Chart>
                  </div>
                  <span class="spark-label">Pistes extraites / session</span>
                </div>
              {/if}
              <div class="sessions-list">
                {#each extractSessions as s, i}
                  <div class="session-row {i === 0 ? 'session-row--latest' : ''}">
                    <span class="session-date">{shortDate(s.date)}</span>
                    <span class="session-files">{s.files} fichier{s.files > 1 ? "s" : ""}</span>
                    <span class="session-ratio">{s.tracks} piste{s.tracks > 1 ? "s" : ""}</span>
                  </div>
                {/each}
              </div>
            </div>
          {/if}

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
    padding: 14px 16px 12px;
  }

  /* Custom bar chart (CSS-based, no SVG) */
  .bar-chart-custom {
    display: flex;
    align-items: stretch;
    gap: 12px;
  }
  .bar-y-axis {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    align-items: flex-end;
    padding-bottom: 20px; /* align with bar-col-label height */
  }
  .bar-y-label {
    font-family: "Geist Mono", monospace;
    font-size: 8px;
    color: var(--color-subtext2);
    white-space: nowrap;
  }
  .bar-columns {
    display: flex;
    gap: 20px;
    align-items: flex-end;
    flex: 1;
  }
  .bar-col {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    flex: 1;
  }
  .bar-val {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext);
    white-space: nowrap;
  }
  .bar-val--accent { color: var(--color-accent); }
  .bar-track {
    width: 100%;
    height: 80px;
    display: flex;
    align-items: flex-end;
    border-bottom: 1px solid var(--color-border);
  }
  .bar-fill {
    width: 100%;
    border-radius: 3px 3px 0 0;
    min-height: 2px;
  }
  .bar-fill--input  { background: var(--color-subtext); opacity: 0.5; }
  .bar-fill--output { background: var(--color-accent); opacity: 0.85; }
  .bar-col-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext2);
    padding-top: 2px;
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

  /* ── Sous-titre de section ── */
  .subsection-title {
    display: flex;
    align-items: center;
    gap: 5px;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--color-subtext2);
    margin: 24px 0 10px;
  }

  /* ── Records ── */
  .records-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
    margin-bottom: 4px;
  }
  .record-card {
    padding: 12px 14px;
    border-radius: var(--radius-sm);
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .record-card--gold {
    border-color: color-mix(in srgb, var(--color-warning) 30%, var(--color-border));
    background: color-mix(in srgb, var(--color-warning) 5%, var(--color-panel));
  }
  .record-header {
    display: flex;
    align-items: center;
    gap: 5px;
  }
  .record-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--color-subtext);
  }
  .record-name {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-weight: 500;
  }
  .record-values {
    display: flex;
    align-items: center;
    gap: 5px;
    flex-wrap: wrap;
  }
  .record-val {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
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
    margin-left: auto;
    font-family: "Geist Mono", monospace;
    font-size: 13px;
    font-weight: 700;
    color: var(--color-success);
  }
  .record-ratio--gold {
    color: var(--color-warning);
  }

  /* ── Sessions ── */
  .sessions-block {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .spark-wrap {
    display: flex;
    flex-direction: column;
    gap: 4px;
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 10px 14px 8px;
  }
  .spark-label {
    font-family: "Geist Mono", monospace;
    font-size: 8px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-subtext2);
  }
  .sessions-list {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }
  .session-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 12px;
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
    border-bottom: 1px solid var(--color-border);
    transition: background 0.08s;
  }
  .session-row:last-child { border-bottom: none; }
  .session-row--latest {
    background: color-mix(in srgb, var(--color-accent) 5%, var(--color-panel));
    color: var(--color-text);
  }
  .session-date  { min-width: 80px; color: var(--color-subtext2); }
  .session-files { min-width: 60px; }
  .session-saved { flex: 1; color: var(--color-success); }
  .session-ratio {
    font-weight: 700;
    color: var(--color-accent);
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
  /* ── Dernière session ── */
  .last-session-card {
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 16px 20px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .ls-date {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext2);
    letter-spacing: 0.03em;
  }
  .ls-stats-row {
    display: flex;
    align-items: center;
    gap: 0;
  }
  .ls-stat {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
    align-items: center;
  }
  .ls-divider {
    width: 1px;
    height: 32px;
    background: var(--color-border);
    flex-shrink: 0;
  }
  .ls-stat-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-subtext2);
  }
  .ls-stat-value {
    font-family: "Geist Mono", monospace;
    font-size: 18px;
    font-weight: 700;
    color: var(--color-text);
    line-height: 1;
  }
  .ls-stat-value--success { color: var(--color-success); }
  .ls-stat-value--accent  { color: var(--color-accent); }


</style>