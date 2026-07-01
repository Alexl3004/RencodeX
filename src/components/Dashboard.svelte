<script lang="ts">
  import { stats } from "$lib/stores/stats.svelte";
  import { formatSize } from "$lib/utils";
  import {
    X,
    HardDrive,
    FileCheck2,
    Clock,
    Gauge,
    TrendingDown,
    CheckCircle2,
  } from "@lucide/svelte";
  import { Progress, Popover, Portal } from "@skeletonlabs/skeleton-svelte";

  let { onClose }: { onClose?: () => void } = $props();

  let totalFiles = $derived(stats.totalFiles);
  let totalSavedMb = $derived(stats.totalSavedMb);
  let totalOriginalMb = $derived(stats.totalOriginalMb);
  let totalEncodedMb = $derived(stats.totalEncodedMb);
  let avgInputMb = $derived(stats.avgInputMb);
  let avgOutputMb = $derived(stats.avgOutputMb);
  let avgRatioPct = $derived(stats.avgRatioPct);
  let avgThroughputMbps = $derived(stats.avgThroughputMbps);
  let successRatePct = $derived(stats.successRatePct);
  let totalSecs = $derived(stats.totalSecs);
  let lastUpdated = $derived(stats.lastUpdated);

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
    return mbps < 1
      ? `${(mbps * 1024).toFixed(0)} KB/s`
      : `${mbps.toFixed(1)} MB/s`;
  }

  let progressVal = $derived(Math.max(0, 100 - avgRatioPct));
  let hasData = $derived(totalFiles > 0);
</script>

<div class="panel-root">
  <!-- Header -->
  <div class="panel-header">
    <div class="flex items-center gap-2">
      <div class="accent-bar"></div>
      <span class="section-label">Dashboard</span>
    </div>
    {#if onClose}
      <button onclick={onClose} class="icon-btn" aria-label="Fermer">
        <X class="w-4 h-4" />
      </button>
    {/if}
  </div>

  <!-- Body -->
  <div class="panel-body space-y-4">
    <p class="font-mono text-[10px] subtext">
      Statistiques cumulées · encodages réussis uniquement
    </p>

    {#if !hasData}
      <!-- Empty state -->
      <div class="empty-state">
        <HardDrive class="w-8 h-8 empty-icon" />
        <span class="font-mono text-[11px] subtext mt-2"
          >Aucun encodage enregistré</span
        >
      </div>
    {:else}
      <!-- Hero : espace économisé -->
      <div class="hero-card">
        <div class="hero-label">
          <TrendingDown class="w-3 h-3" />
          Espace économisé total
        </div>
        <div class="hero-value">{formatSize(totalSavedMb)}</div>
        <div class="hero-sub font-mono">
          {formatSize(totalOriginalMb)} → {formatSize(totalEncodedMb)}
        </div>
      </div>

      <!-- Barre de compression -->
      <div class="compress-bar-wrap">
        <Progress value={progressVal} max={100} class="relative block">
          <Progress.Track class="compress-track">
            <Progress.Range class="compress-range" />
          </Progress.Track>
          <div class="compress-label font-mono">
            {formatSize(avgOutputMb)} / {formatSize(avgInputMb)} en moyenne · −{avgRatioPct.toFixed(
              1,
            )}%
          </div>
        </Progress>
      </div>

      <!-- Grille 2×2 principale -->
      <div class="grid grid-cols-2 gap-2">
        <div class="stat-card">
          <div class="stat-icon-row">
            <FileCheck2 class="w-3.5 h-3.5 stat-icon" />
            <span class="stat-label">Fichiers encodés</span>
          </div>
          <div class="stat-value">{totalFiles}</div>
        </div>

        <div class="stat-card accent-card">
          <div class="stat-icon-row">
            <TrendingDown
              class="w-3.5 h-3.5"
              style="color: var(--color-accent);"
            />
            <span class="stat-label">Compression moy.</span>
          </div>
          <div class="stat-value" style="color: var(--color-accent);">
            −{avgRatioPct.toFixed(1)}%
          </div>
        </div>

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
      </div>

      <!-- Taux de succès + volumes : ligne complète -->
      <div class="grid grid-cols-3 gap-2">
        <div class="stat-card success-card col-span-1">
          <div class="stat-icon-row">
            <CheckCircle2
              class="w-3.5 h-3.5"
              style="color: var(--color-success);"
            />
            <span class="stat-label">Succès</span>
          </div>
          <div class="stat-value" style="color: var(--color-success);">
            {successRatePct.toFixed(0)}%
          </div>
        </div>

        <div class="stat-card col-span-1">
          <div class="stat-label mb-1.5">Entrée moy.</div>
          <div class="stat-value">{formatSize(avgInputMb)}</div>
        </div>

        <div class="stat-card col-span-1">
          <div class="stat-label mb-1.5">Sortie moy.</div>
          <div class="stat-value">{formatSize(avgOutputMb)}</div>
        </div>
      </div>
    {/if}

    <!-- Footer -->
    <div class="footer-row">
      <span class="font-mono text-[9px] subtext2">
        Màj : {formattedDate(lastUpdated)}
      </span>
      <Popover positioning={{ placement: "top-end" }}>
        <Popover.Trigger class="btn font-mono text-[9px] px-2 py-1">
          Réinitialiser
        </Popover.Trigger>
        <Portal>
          <Popover.Positioner>
            <Popover.Content
              class="w-56 p-3 rounded-[var(--radius-md)] shadow-xl"
              style="background: var(--color-panel); border: 1px solid var(--color-border);"
            >
              <Popover.Description
                class="text-[11px] leading-snug"
                style="color: var(--color-text);"
              >
                Réinitialiser toutes les statistiques cumulées ?
              </Popover.Description>
              <div class="flex gap-2 mt-3">
                <Popover.CloseTrigger
                  onclick={() => stats.reset()}
                  class="btn btn-danger flex-1 justify-center font-mono text-[9px]"
                >
                  Confirmer
                </Popover.CloseTrigger>
                <Popover.CloseTrigger
                  class="btn btn-secondary flex-1 justify-center font-mono text-[9px]"
                >
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
  .panel-root {
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-panel);
    flex-shrink: 0;
  }

  .panel-body {
    padding: 14px 16px 16px;
    overflow-y: auto;
  }

  /* Accent bar in header */
  .accent-bar {
    width: 3px;
    height: 14px;
    border-radius: 1px;
    background: var(--color-accent);
  }

  /* Subtext helpers */
  .subtext {
    color: var(--color-subtext);
  }
  .subtext2 {
    color: var(--color-subtext2);
  }

  /* Empty state */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 32px 0;
    gap: 4px;
  }

  /* Hero card */
  .hero-card {
    padding: 14px 16px;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--color-success) 8%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-success) 25%, transparent);
  }
  .hero-label {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 10px;
    font-family: monospace;
    color: color-mix(in srgb, var(--color-success) 80%, var(--color-text));
    margin-bottom: 6px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .hero-value {
    font-family: monospace;
    font-size: 28px;
    font-weight: 700;
    line-height: 1;
    color: var(--color-success);
  }
  .hero-sub {
    font-size: 10px;
    color: color-mix(in srgb, var(--color-success) 60%, transparent);
    margin-top: 5px;
  }

  /* Compression bar */
  .compress-bar-wrap {
    position: relative;
  }
  :global(.compress-track) {
    height: 18px;
    border-radius: var(--radius-sm) !important;
    overflow: hidden;
    border: 1px solid var(--color-border);
    background: var(--color-surface);
  }
  :global(.compress-range) {
    border-radius: 1px;
    background: var(--color-success);
    transition: width 0.3s ease;
    opacity: 0.7;
  }
  .compress-label {
    pointer-events: none;
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 9px;
    color: var(--color-text);
  }

  /* Stat cards */
  .stat-card {
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
  }
  .stat-icon-row {
    display: flex;
    align-items: center;
    gap: 5px;
    margin-bottom: 6px;
  }

  .stat-label {
    font-size: 9px;
    font-family: monospace;
    color: var(--color-subtext);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .stat-value {
    font-family: monospace;
    font-size: 16px;
    font-weight: 700;
    line-height: 1;
    color: var(--color-text);
  }

  /* Accent card (compression) */
  .accent-card {
    border-color: color-mix(
      in srgb,
      var(--color-accent) 30%,
      var(--color-border)
    );
    background: color-mix(
      in srgb,
      var(--color-accent) 5%,
      var(--color-surface)
    );
  }

  /* Success card */
  .success-card {
    border-color: color-mix(
      in srgb,
      var(--color-success) 30%,
      var(--color-border)
    );
    background: color-mix(
      in srgb,
      var(--color-success) 5%,
      var(--color-surface)
    );
  }

  /* Footer */
  .footer-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-top: 10px;
    border-top: 1px solid var(--color-border);
  }

  /* Icon button */
  .icon-btn {
    width: 24px;
    height: 24px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-xs);
    border: 1px solid transparent;
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    transition:
      background 0.1s,
      color 0.1s,
      border-color 0.1s;
  }
  .icon-btn:hover {
    background: var(--color-panel2);
    border-color: var(--color-border);
    color: var(--color-text);
  }
</style>