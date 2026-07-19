<script lang="ts">
  import { onMount } from "svelte";
  import { encoder } from "$lib/stores/encoder.svelte";

  import ToastNotif from "$components/ToastNotif.svelte";
  import TopBar from "$components/TopBar.svelte";
  import FileToolbar from "$components/FileToolbar.svelte";
  import FileTable from "$components/FileTable.svelte";
  import ProgressPanel from "$components/ProgressPanel.svelte";
  import ControlBar from "$components/ControlBar.svelte";
  import EncodingSettings from "$components/encoding/EncodingSettings.svelte"
  import RenamingSettings from "$components/renaming/RenamingSettings.svelte";
  import Dashboard from "$components/dashboard/Dashboard.svelte";
  import LogConsole from "$components/Log/LogConsole.svelte";
  import Settings from "$components/setting/Settings.svelte";
  import { stats } from "$lib/stores/stats.svelte";

  let topbar = $state<{ activeTab: string; TABS: any[] } | null>(null);
  let activeTab = $derived(topbar?.activeTab ?? "home");
  let TABS      = $derived(topbar?.TABS ?? []);

  onMount(async () => { await encoder.init(); });

  onMount(() => { stats.init(); });
  // Le listener drag & drop tauri://file-drop est géré exclusivement dans
  // FileTable.svelte (qui pilote aussi l'overlay visuel) — pas de double écoute.
</script>

<svelte:window ondragover={(e) => e.preventDefault()} />

<div class="app-shell">
  <!-- TopBar : ligne unique logo + status + utils -->
  <TopBar bind:this={topbar} />

  <!-- Navbar horizontale (conditionnelle) -->
  {#if encoder.navLayout === "horizontal"}
  <div class="topnav" role="tablist" aria-label="Navigation">

    <!-- Accueil — épinglé à gauche -->
    {#each TABS.filter(t => t.id === "home") as tab}
      {@const isActive = activeTab === tab.id}
      <button type="button" role="tab" aria-selected={isActive}
        aria-label={tab.label}
        class="side-tab" class:side-tab--active={isActive}
        onclick={() => { if (topbar) topbar.activeTab = tab.id; }}>
        <div style="position:relative;display:flex;">
          <tab.icon class="side-icon" aria-hidden="true" />
        </div>
        {#if isActive}<span class="top-label">{tab.label}</span>{/if}
      </button>
    {/each}

    <!-- Séparateur vertical -->
    <div class="top-sep" aria-hidden="true"></div>

    <!-- Onglets centraux -->
    <div class="topnav-middle">
      {#each TABS.filter(t => t.id !== "home" && t.id !== "settings") as tab}
        {@const isActive = activeTab === tab.id}
        {@const badge = tab.badge?.()}
        <button type="button" role="tab" aria-selected={isActive}
          aria-label={tab.label}
          class="side-tab" class:side-tab--active={isActive}
          onclick={() => { if (topbar) topbar.activeTab = tab.id; }}>
          <div style="position:relative;display:flex;">
            <tab.icon class="side-icon" aria-hidden="true" />
            {#if badge}<span class="side-badge" aria-label="{badge} alertes">{badge}</span>{/if}
          </div>
          {#if isActive}<span class="top-label">{tab.label}</span>{/if}
        </button>
      {/each}
    </div>

    <!-- Séparateur vertical -->
    <div class="top-sep" aria-hidden="true"></div>

    <!-- Settings — épinglé à droite -->
    {#each TABS.filter(t => t.id === "settings") as tab}
      {@const isActive = activeTab === tab.id}
      <button type="button" role="tab" aria-selected={isActive}
        aria-label={tab.label}
        class="side-tab" class:side-tab--active={isActive}
        onclick={() => { if (topbar) topbar.activeTab = tab.id; }}>
        <div style="position:relative;display:flex;">
          <tab.icon class="side-icon" aria-hidden="true" />
        </div>
        {#if isActive}<span class="top-label">{tab.label}</span>{/if}
      </button>
    {/each}

  </div>
  {/if}

  <!-- Corps : sidebar + contenu -->
  <div class="body-row">

    <!-- ── Sidebar verticale ──────────────────────────────────────────────── -->
    {#if encoder.navLayout === "vertical"}
    <div class="sidebar" role="tablist" aria-label="Navigation">

      <!-- Accueil — épinglé en haut -->
      {#each TABS.filter(t => t.id === "home") as tab}
        {@const isActive = activeTab === tab.id}
        <button type="button" role="tab" aria-selected={isActive}
          aria-label={tab.label} title={!isActive ? tab.label : undefined}
          class="side-tab" class:side-tab--active={isActive}
          onclick={() => { if (topbar) topbar.activeTab = tab.id; }}>
          <tab.icon class="side-icon" aria-hidden="true" />
          {#if isActive}<span class="side-label">{tab.label}</span>{/if}
        </button>
      {/each}

      <!-- Séparateur -->
      <div class="side-sep"></div>

      <!-- Onglets centraux — poussés vers le centre -->
      <div class="side-middle">
        {#each TABS.filter(t => t.id !== "home" && t.id !== "settings") as tab}
          {@const isActive = activeTab === tab.id}
          {@const badge = tab.badge?.()}
          <button type="button" role="tab" aria-selected={isActive}
            aria-label={tab.label} title={!isActive ? tab.label : undefined}
            class="side-tab" class:side-tab--active={isActive}
            onclick={() => { if (topbar) topbar.activeTab = tab.id; }}>
            <tab.icon class="side-icon" aria-hidden="true" />
            {#if isActive}<span class="side-label">{tab.label}</span>{/if}
            {#if badge}<span class="side-badge" aria-label="{badge} alertes">{badge}</span>{/if}
          </button>
        {/each}
      </div>

      <!-- Séparateur -->
      <div class="side-sep"></div>

      <!-- Settings — épinglé en bas -->
      {#each TABS.filter(t => t.id === "settings") as tab}
        {@const isActive = activeTab === tab.id}
        <button type="button" role="tab" aria-selected={isActive}
          aria-label={tab.label} title={!isActive ? tab.label : undefined}
          class="side-tab" class:side-tab--active={isActive}
          onclick={() => { if (topbar) topbar.activeTab = tab.id; }}>
          <tab.icon class="side-icon" aria-hidden="true" />
          {#if isActive}<span class="side-label">{tab.label}</span>{/if}
        </button>
      {/each}

    </div>
    {/if}

    <!-- ── Contenu principal ──────────────────────────────────────────────── -->
    <div class="view-area">

      {#if activeTab === "home"}
        <div class="main-content" role="tabpanel" aria-label="Accueil">
          <div class="fileToolbar-slot"><FileToolbar /></div>
          <section class="filetable-slot">
            <div class="section-label mb-1">
              Fichiers
              {#if encoder.files.length > 0}
                <span class="text-[var(--color-accent)] ml-1">{encoder.files.length}</span>
              {/if}
            </div>
            <div class="filetable-wrap"><FileTable /></div>
          </section>
          <div class="progress-slot"><ProgressPanel /></div>
        </div>

      {:else if activeTab === "encoding"}
        <div class="tab-page" role="tabpanel" aria-label="Encodage">
          <EncodingSettings />
        </div>

      {:else if activeTab === "renaming"}
        <div class="tab-page" role="tabpanel" aria-label="Nommage">
          <RenamingSettings onClose={() => {}} />
        </div>

      {:else if activeTab === "stats"}
        <div class="tab-page" role="tabpanel" aria-label="Statistiques">
          <Dashboard />
        </div>

      {:else if activeTab === "logs"}
        <div class="tab-page" role="tabpanel" aria-label="Logs">
          <LogConsole />
        </div>

      {:else if activeTab === "settings"}
        <div class="tab-page" role="tabpanel" aria-label="Paramètres">
          <Settings />
        </div>
      {/if}

      <ControlBar />
    </div>
  </div>
</div>

<ToastNotif />

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
    background: var(--color-surface);
  }

  /* ── Corps principal ────────────────────────────────────────────────────── */
  .body-row {
    display: flex;
    flex: 1 1 0;
    min-height: 0;
    overflow: hidden;
  }

  /* ── Sidebar ────────────────────────────────────────────────────────────── */
  .sidebar {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 8px 4px;
    width: 56px;
    flex-shrink: 0;
    background: var(--color-panel);
    border-right: 1px solid var(--color-border);
  }

  /* Zone centrale qui absorbe l'espace restant et centre ses enfants */
  .side-middle {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    width: 100%;
  }

  .side-sep {
    width: 24px;
    height: 1px;
    background: var(--color-border);
    margin: 4px 0;
    flex-shrink: 0;
    opacity: 0.5;
  }

  .side-tab {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 3px;
    width: 48px;
    min-height: 44px;
    padding: 8px 4px;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    overflow: hidden;
    transition: opacity 0.15s, background 0.15s, color 0.15s;
    flex-shrink: 0;
  }
  .side-tab:hover {
    opacity: 0.7;
    background: color-mix(in srgb, var(--color-muted) 12%, transparent);
    color: var(--color-subtext);
  }
  .side-tab--active {
    opacity: 1;
    color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 10%, transparent);
  }
  .side-tab--active:hover {
    opacity: 1;
    background: color-mix(in srgb, var(--color-accent) 14%, transparent);
  }

  :global(.side-icon) {
    width: 14px;
    height: 14px;
    flex-shrink: 0;
  }

  .side-label {
    font-family: "DM Sans", sans-serif;
    font-size: 9px;
    font-weight: 500;
    line-height: 1;
    text-align: center;
    max-width: 48px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    opacity: 0;
    animation: label-in 0.18s ease forwards;
  }
  @keyframes label-in {
    from { opacity: 0; transform: translateY(3px); }
    to   { opacity: 1; transform: translateY(0); }
  }

  .side-badge {
    position: absolute;
    top: 5px;
    right: 6px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 14px;
    height: 14px;
    padding: 0 3px;
    border-radius: 999px;
    background: var(--color-danger);
    color: #fff;
    font-size: 8px;
    font-weight: 700;
    font-family: "Geist Mono", monospace;
    line-height: 1;
  }

  /* ── Zone de contenu ────────────────────────────────────────────────────── */
  .view-area {
    flex: 1 1 0;
    min-width: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* ── Onglet Accueil ─────────────────────────────────────────────────────── */
  .main-content {
    flex: 1 1 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 8px;
    overflow: hidden;
  }
  .fileToolbar-slot  { flex: 0 0 auto; }
  .filetable-slot {
    flex: 3 1 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  .filetable-wrap {
    flex: 1 1 0;
    min-height: 0;
    overflow: hidden;
  }
  .progress-slot {
    flex: 0.5 1 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  /* ── Pages génériques ───────────────────────────────────────────────────── */
  .tab-page {
    flex: 1 1 0;
    min-height: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    animation: tab-in 0.15s cubic-bezier(0.22, 1, 0.36, 1);
  }
  @keyframes tab-in {
    from { opacity: 0; transform: translateX(-6px); }
    to   { opacity: 1; transform: translateX(0); }
  }
  @media (prefers-reduced-motion: reduce) {
    .tab-page { animation: none; }
  }
  /* ── Navbar horizontale ─────────────────────────────────────────────────── */
  .topnav {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 4px 8px;
    background: var(--color-panel);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
    height: 44px;
  }

  /* side-tab en horizontal : icone + label en dessous (comme sidebar) */
  .topnav .side-tab {
    flex-direction: column;
    gap: 3px;
    min-height: 36px;
    width: auto;
    min-width: 44px;
    padding: 4px 10px;
  }

  /* Separateur vertical — miroir du side-sep */
  .top-sep {
    width: 1px;
    height: 20px;
    background: var(--color-border);
    margin: 0 4px;
    flex-shrink: 0;
    opacity: 0.5;
  }

  /* Zone centrale qui pousse settings a droite */
  .topnav-middle {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 2px;
  }

  /* Label visible uniquement sur l'onglet actif, animé comme side-label */
  .top-label {
    font-family: "DM Sans", sans-serif;
    font-size: 9px;
    font-weight: 500;
    line-height: 1;
    white-space: nowrap;
    opacity: 0;
    animation: label-in 0.18s ease forwards;
  }
</style>