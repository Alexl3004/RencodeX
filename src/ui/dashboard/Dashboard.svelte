<script lang="ts">
  import { stats } from "$lib/stores/stats.svelte";
  import { encoder } from "$lib/stores/encoder.svelte";
  import { BarChart3, Subtitles, History } from "@lucide/svelte";
  import { Popover, Portal } from "@skeletonlabs/skeleton-svelte";
  import { onMount } from "svelte";

  import EncodageTab from "./EncodageTab.svelte";
  import HistoryTab from "./HistoryTab.svelte";
  import ExtractionTab from "./ExtractionTab.svelte";

  onMount(() => {
    stats.init();
  });

  let { onClose }: { onClose?: () => void } = $props();

  // ── Stats dérivées ──────────────────────────────────────────────────────
  let totalFiles = $derived(stats.totalFiles);
  let totalSavedMb = $derived(stats.totalSavedMb);
  let totalOriginalMb = $derived(stats.totalOriginalMb);
  let totalEncodedMb = $derived(stats.totalEncodedMb);
  let avgInputMb = $derived(stats.avgInputMb);
  let avgOutputMb = $derived(stats.avgOutputMb);
  let avgRatioPct = $derived(stats.avgRatioPct);
  let avgSecs = $derived(stats.avgSecs);
  let totalSecs = $derived(stats.totalSecs);
  let totalExtractedFiles = $derived(stats.totalExtractedFiles);
  let totalExtractLaunched = $derived(stats.totalExtractLaunched);
  let totalTracksExtracted = $derived(stats.totalTracksExtracted);
  let avgTracksPerFile = $derived(stats.avgTracksPerFile);
  let recordHeaviest = $derived(stats.recordHeaviest);
  let recordBestRatio = $derived(stats.recordBestRatio);
  let encodeSessions = $derived(stats.encodeSessions);
  let extractSessions = $derived(stats.extractSessions);
  let hasData = $derived(totalFiles > 0);
  let lastSession = $derived(
    encodeSessions.length > 0 ? encodeSessions[0] : null,
  );
  let hasExtractData = $derived(totalExtractLaunched > 0);

  // ── Navigation ──────────────────────────────────────────────────────────
  type SectionId = "encodage" | "historique" | "extraction";
  let activeSection = $state<SectionId>("encodage");

  const SECTIONS: { id: SectionId; label: string; icon: any; desc: string }[] =
    [
      {
        id: "encodage",
        label: "Encodage",
        icon: BarChart3,
        desc: "Compression · succès",
      },
      {
        id: "historique",
        label: "Historique",
        icon: History,
        desc: "Sessions · records",
      },
      {
        id: "extraction",
        label: "Extraction",
        icon: Subtitles,
        desc: "Sous-titres extraits",
      },
    ];
</script>

<div
  class="page-root"
  class:page-root--horizontal={encoder.innerNavLayout === "horizontal"}
>
  <!-- ── Sidebar ──────────────────────────────────────────────────────────── -->
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

    <div class="sidebar-footer"></div>
  </aside>

  <!-- ── Content ─────────────────────────────────────────────────────────── -->
  <div class="content-panel">
    {#if activeSection === "encodage"}
      <EncodageTab
        loaded={stats.loaded}
        {hasData}
        {totalSavedMb}
        {totalOriginalMb}
        {totalEncodedMb}
        {avgInputMb}
        {avgOutputMb}
        {avgRatioPct}
        {totalSecs}
        {avgSecs}
        {totalFiles}
      />
    {:else if activeSection === "historique"}
      <HistoryTab
        {hasData}
        {lastSession}
        {recordHeaviest}
        {recordBestRatio}
        {encodeSessions}
      />
    {:else if activeSection === "extraction"}
      <ExtractionTab
        {hasExtractData}
        {totalTracksExtracted}
        {totalExtractedFiles}
        {totalExtractLaunched}
        {avgTracksPerFile}
        {extractSessions}
      />
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
              <Popover.Description
                class="text-[11px] leading-snug"
                style="color:var(--color-text);"
              >
                Réinitialiser toutes les statistiques cumulées ?
              </Popover.Description>
              <div class="flex gap-2 mt-3">
                <Popover.CloseTrigger
                  onclick={() => stats.reset()}
                  class="btn btn-danger flex-1 justify-center font-mono text-[9px]"
                  >Confirmer</Popover.CloseTrigger
                >
                <Popover.CloseTrigger
                  class="btn btn-secondary flex-1 justify-center font-mono text-[9px]"
                  >Annuler</Popover.CloseTrigger
                >
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
    transition:
      background 0.1s,
      border-color 0.1s;
    width: 100%;
  }
  .nav-item:hover {
    background: color-mix(in srgb, var(--color-muted) 30%, transparent);
  }
  .nav-item--active {
    background: color-mix(in srgb, var(--color-accent) 9%, transparent);
    border-color: color-mix(
      in srgb,
      var(--color-accent) 22%,
      var(--color-border)
    );
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
    transition:
      background 0.1s,
      color 0.1s,
      border-color 0.1s;
  }
  .nav-item--active .nav-item-icon {
    background: color-mix(
      in srgb,
      var(--color-accent) 12%,
      var(--color-surface)
    );
    border-color: color-mix(
      in srgb,
      var(--color-accent) 30%,
      var(--color-border)
    );
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
    transition:
      border-color 0.12s,
      color 0.12s;
  }
  :global(.reset-btn:hover) {
    border-color: var(--color-danger);
    color: var(--color-danger);
  }

  /* ── Layout horizontal ── */
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
</style>
