<script lang="ts">
  import { theme } from "$lib/stores/theme.svelte";
  import { encoder } from "$lib/stores/encoder.svelte";
  import { formatSize } from "$lib/utils";
  import Settings from "$components/Settings.svelte";
  import LogConsole from "$components/LogConsole.svelte";
  import EncodingSettings from "$components/EncodingSettings.svelte";
  import Dashboard from "$components/Dashboard.svelte";

  let showLogs = $state(false);
  let showSettings = $state(false);
  let showDashboard = $state(false);

  let totalSize  = $derived(encoder.files.reduce((acc, f) => acc + (f.size_mb ?? 0), 0));
  let readyCount = $derived(encoder.files.filter(f => f.status === "ready").length);
  let doneCount  = $derived(encoder.files.filter(f => f.status === "done").length);
  let errorCount = $derived(encoder.logs.filter((e) => e.level === "error").length);
  let warnCount  = $derived(encoder.logs.filter((e) => e.level === "warn").length);

  function toggleLogs() {
    showLogs = !showLogs;
  }
</script>

<header class="flex items-center justify-between px-4 h-10 shrink-0 select-none
               border-b border-[var(--color-border)] bg-[var(--color-panel)]">

  <!-- Left: logo -->
  <div class="flex items-center gap-3">
    <div class="flex items-center gap-2">
      <div class="w-[3px] h-5 rounded-[1px] bg-[var(--color-accent)]"></div>
      <span class="font-semibold text-[13px] text-[var(--color-text)] tracking-tight leading-none">
        RenCodeX
      </span>
    </div>
    <span class="font-mono text-[10px] px-2 py-[3px] border border-[var(--color-border)]
                 text-[var(--color-subtext)] rounded-[2px] tracking-widest leading-none">
      H.265 · NVENC
    </span>
  </div>

  <!-- Center: status -->
  <div class="flex items-center gap-3 text-[11px] font-mono">
    {#if encoder.encoding}
      <span class="flex items-center gap-2 text-[var(--color-accent)]">
        <span class="inline-block w-[7px] h-[7px] rounded-full bg-[var(--color-accent)] animate-pulse"></span>
        ENCODING
      </span>
    {:else if doneCount > 0 && doneCount === encoder.files.length}
      <span class="flex items-center gap-2 text-[var(--color-success)]">
        <span class="inline-block w-[7px] h-[7px] rounded-full bg-[var(--color-success)]"></span>
        DONE — {doneCount} fichier{doneCount > 1 ? "s" : ""}
      </span>
    {:else if encoder.files.length > 0}
      <span class="text-[var(--color-subtext)]">
        {readyCount}/{encoder.files.length} prêt{readyCount > 1 ? "s" : ""}
      </span>
      {#if totalSize > 0}
        <span class="text-[var(--color-subtext2)]">·</span>
        <span class="text-[var(--color-subtext)]">{formatSize(totalSize)}</span>
      {/if}
    {/if}
  </div>

  <!-- Right -->
  <div class="flex items-center gap-1">
    <!-- Bouton Logs -->
    <button type="button" onclick={toggleLogs} class="btn btn-ghost px-2 py-1 text-[11px] gap-1.5 font-mono relative"
            title="Journal des logs">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
        <polyline points="14 2 14 8 20 8"/>
        <line x1="16" y1="13" x2="8" y2="13"/>
        <line x1="16" y1="17" x2="8" y2="17"/>
        <polyline points="10 9 9 9 8 9"/>
      </svg>
      LOGS
      {#if errorCount > 0 || warnCount > 0}
        <span class="absolute -top-1 -right-1 w-2 h-2 bg-[var(--color-danger)] rounded-full animate-pulse"></span>
      {/if}
    </button>

    <!-- Bouton Paramètres encodage (CRF / Preset) -->
    <button type="button" onclick={() => (showSettings = !showSettings)} class="btn btn-ghost px-2 py-1 text-[11px] gap-1.5 font-mono"
            title="Paramètres d'encodage (CRF, Preset)">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3"/>
        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>
      </svg>
      ENCODE
    </button>

    <!-- Bouton Dashboard -->
    <button type="button" onclick={() => (showDashboard = !showDashboard)} class="btn btn-ghost px-2 py-1 text-[11px] gap-1.5 font-mono"
            title="Dashboard des statistiques">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="18" y1="20" x2="18" y2="10"/>
        <line x1="12" y1="20" x2="12" y2="4"/>
        <line x1="6" y1="20" x2="6" y2="14"/>
      </svg>
      DASHBOARD
    </button>

    <!-- Bouton thème -->
    <button type="button" onclick={() => theme.toggle()} class="btn btn-ghost px-2 py-1 text-[11px] gap-1.5 font-mono"
            title="Basculer thème">
      {#if theme.dark}
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="5"/>
          <line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/>
          <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
          <line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/>
          <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
        </svg>
        LIGHT
      {:else}
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
        </svg>
        DARK
      {/if}
    </button>
    
    <div class="sep h-4 mx-1"></div>
    <Settings />
  </div>
</header>

<!-- Overlay Logs -->
{#if showLogs}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60"
    role="dialog"
    aria-modal="true"
    aria-label="Journal des logs"
    onclick={(e) => e.target === e.currentTarget && (showLogs = false)}
    onkeydown={(e) => e.key === 'Escape' && (showLogs = false)}
    tabindex="-1"
  >
    <div class="w-[800px] max-w-[90vw] max-h-[80vh]">
      <LogConsole onClose={() => showLogs = false} />
    </div>
  </div>
{/if}

<!-- Overlay Paramètres encodage -->
{#if showSettings}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60"
    role="dialog"
    aria-modal="true"
    aria-label="Paramètres d'encodage"
    onclick={(e) => e.target === e.currentTarget && (showSettings = false)}
    onkeydown={(e) => e.key === 'Escape' && (showSettings = false)}
    tabindex="-1"
  >
    <div class="w-[600px] max-w-[90vw] max-h-[80vh]">
      <EncodingSettings onClose={() => (showSettings = false)} />
    </div>
  </div>
{/if}

<!-- Overlay Dashboard -->
{#if showDashboard}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60"
    role="dialog"
    aria-modal="true"
    aria-label="Dashboard des statistiques"
    onclick={(e) => e.target === e.currentTarget && (showDashboard = false)}
    onkeydown={(e) => e.key === 'Escape' && (showDashboard = false)}
    tabindex="-1"
  >
    <div class="w-[600px] max-w-[90vw] max-h-[80vh]">
      <Dashboard onClose={() => (showDashboard = false)} />
    </div>
  </div>
{/if}
