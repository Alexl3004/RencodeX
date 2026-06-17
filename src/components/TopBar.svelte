<script lang="ts">
  import { theme } from "$lib/stores/theme.svelte";
  import { encoder } from "$lib/stores/encoder.svelte";
  import { formatSize } from "$lib/utils";
  import Settings from "$components/Settings.svelte";
  import LogConsole from "$components/LogConsole.svelte";
  import EncodingSettings from "$components/EncodingSettings.svelte";
  import Dashboard from "$components/Dashboard.svelte";
  import Settings2Icon from "@iconify-svelte/lucide/settings-2";
  import LogsIcon from "@iconify-svelte/lucide/logs";
  import ChartColumnIcon from "@iconify-svelte/lucide/chart-column";
  import SunIcon from "@iconify-svelte/lucide/sun";
  import MoonIcon from "@iconify-svelte/lucide/moon";

  let showLogs = $state(false);
  let showSettings = $state(false);
  let showDashboard = $state(false);

  let totalSize = $derived(
    encoder.files.reduce((acc, f) => acc + (f.size_mb ?? 0), 0),
  );
  let readyCount = $derived(
    encoder.files.filter((f) => f.status === "ready").length,
  );
  let doneCount = $derived(
    encoder.files.filter((f) => f.status === "done").length,
  );
  let errorCount = $derived(
    encoder.logs.filter((e) => e.level === "error").length,
  );
  let warnCount = $derived(
    encoder.logs.filter((e) => e.level === "warn").length,
  );
  let hasAlert = $derived(errorCount > 0 || warnCount > 0);
</script>

<header
  class="flex items-center justify-between px-4 h-10 shrink-0 select-none
               border-b border-[var(--color-border)] bg-[var(--color-panel)]"
>
  <!-- Left: logo -->
  <div class="flex items-center gap-3">
    <div class="flex items-center gap-2">
      <div class="w-[3px] h-5 rounded-[1px] bg-[var(--color-accent)]"></div>
      <span
        class="font-semibold text-[13px] text-[var(--color-text)] tracking-tight leading-none"
      >
        RenCodeX
      </span>
    </div>
    <span
      class="font-mono text-[10px] px-2 py-[3px] border border-[var(--color-border)]
                 text-[var(--color-subtext)] rounded-[2px] tracking-widest leading-none"
    >
      H.265 · NVENC
    </span>
  </div>

  <!-- Center: status pill -->
  <div class="flex items-center h-full">
    {#if encoder.encoding}
      <span
        class="flex items-center gap-2 font-mono text-[11px] text-[var(--color-accent)]"
      >
        <span
          class="inline-block w-[6px] h-[6px] rounded-full bg-[var(--color-accent)] animate-pulse"
        ></span>
        ENCODING
      </span>
    {:else if doneCount > 0 && doneCount === encoder.files.length}
      <span
        class="flex items-center gap-2 font-mono text-[11px] text-[var(--color-success)]"
      >
        <span
          class="inline-block w-[6px] h-[6px] rounded-full bg-[var(--color-success)]"
        ></span>
        DONE — {doneCount} fichier{doneCount > 1 ? "s" : ""}
      </span>
    {:else if encoder.files.length > 0}
      <span class="font-mono text-[11px] text-[var(--color-subtext)]">
        {readyCount}/{encoder.files.length} prêt{readyCount > 1 ? "s" : ""}
        {#if totalSize > 0}
          <span class="text-[var(--color-subtext2)] mx-1">·</span>{formatSize(
            totalSize,
          )}
        {/if}
      </span>
    {/if}
  </div>

  <!-- Right: icon buttons, grouped by role -->
  <div class="flex items-center gap-0.5">
    <!-- Groupe "vues" : Logs + Dashboard -->
    <div class="flex items-center">
      <!-- Logs -->
      <button
        type="button"
        onclick={() => (showLogs = !showLogs)}
        class="topbar-btn relative {showLogs ? 'topbar-btn--active' : ''}"
        title="Logs{hasAlert
          ? ` · ${errorCount} erreur${errorCount > 1 ? 's' : ''}, ${warnCount} avert.`
          : ''}"
        aria-label="Journal des événements"
        aria-pressed={showLogs}
      >
        <LogsIcon height="1em" />
        {#if hasAlert}
          <span
            class="absolute top-[5px] right-[5px] w-[5px] h-[5px] rounded-full
                       bg-[var(--color-danger)] animate-pulse"
            aria-hidden="true"
          ></span>
        {/if}
      </button>

      <!-- Dashboard -->
      <button
        type="button"
        onclick={() => (showDashboard = !showDashboard)}
        class="topbar-btn {showDashboard ? 'topbar-btn--active' : ''}"
        title="Statistiques"
        aria-label="Dashboard des statistiques"
        aria-pressed={showDashboard}
      >
        <ChartColumnIcon height="1em" />
      </button>
    </div>

    <div class="sep h-4 mx-1.5"></div>

    <!-- Groupe "config" : Encodage + Paramètres app -->
    <div class="flex items-center">
      <!-- Paramètres d'encodage (CRF / Preset) -->
      <button
        type="button"
        onclick={() => (showSettings = !showSettings)}
        class="topbar-btn {showSettings ? 'topbar-btn--active' : ''}"
        title="Paramètres d'encodage (CRF, Preset)"
        aria-label="Paramètres d'encodage"
        aria-pressed={showSettings}
      >
        <Settings2Icon height="1em" />
      </button>

      <!-- Paramètres app (Settings component) -->
      <Settings />
    </div>

    <div class="sep h-4 mx-1.5"></div>

    <!-- Thème -->
    <button
      type="button"
      onclick={() => theme.toggle()}
      class="topbar-btn"
      title={theme.dark ? "Passer en mode clair" : "Passer en mode sombre"}
      aria-label={theme.dark
        ? "Activer le thème clair"
        : "Activer le thème sombre"}
    >
      {#if theme.dark}
        <SunIcon height="1em" />
      {:else}
        <MoonIcon height="1em" />
      {/if}
    </button>
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
    onkeydown={(e) => e.key === "Escape" && (showLogs = false)}
    tabindex="-1"
  >
    <div class="w-[800px] max-w-[90vw] max-h-[80vh]">
      <LogConsole onClose={() => (showLogs = false)} />
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
    onkeydown={(e) => e.key === "Escape" && (showSettings = false)}
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
    onkeydown={(e) => e.key === "Escape" && (showDashboard = false)}
    tabindex="-1"
  >
    <div class="w-[600px] max-w-[90vw] max-h-[80vh]">
      <Dashboard onClose={() => (showDashboard = false)} />
    </div>
  </div>
{/if}

<style>
  /* Bouton icône de la topbar */
  .topbar-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 4px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    transition:
      background 0.1s,
      border-color 0.1s,
      color 0.1s;
    position: relative;
    flex-shrink: 0;
  }

  .topbar-btn:hover {
    background: var(--color-panel2);
    border-color: var(--color-border);
    color: var(--color-text);
  }

  .topbar-btn:active {
    transform: translateY(1px);
  }

  /* État actif : overlay ouvert */
  .topbar-btn--active {
    background: color-mix(in srgb, var(--color-accent) 10%, transparent);
    border-color: color-mix(
      in srgb,
      var(--color-accent) 25%,
      var(--color-border)
    );
    color: var(--color-accent);
  }

  .topbar-btn--active:hover {
    background: color-mix(in srgb, var(--color-accent) 16%, transparent);
    color: var(--color-accent);
  }
</style>
