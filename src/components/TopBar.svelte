<script lang="ts">
  import { theme } from "$lib/stores/theme.svelte";
  import { encoder } from "$lib/stores/encoder.svelte";
  import { formatSize } from "$lib/utils";
  import Settings from "$components/Settings.svelte";
  import LangPopover from "$components/LangPopover.svelte";
  import LogConsole from "$components/LogConsole.svelte";
  import Dashboard from "$components/Dashboard.svelte";
  import EncodingSettings from "$components/EncodingSettings.svelte";
  import RenamingSettings from "$components/RenamingSettings.svelte";
  import {
    ChartColumnDecreasing,
    Sun,
    Moon,
    Wrench,
    SlidersHorizontal,
    Tags,
    Logs,
    RefreshCw,
  } from "@lucide/svelte";

  let { showAppSettings = $bindable(false) } = $props();

  let openPanel = $state<"logs" | "dashboard" | "settings" | "renaming" | null>(null);

  function toggle(panel: "logs" | "dashboard" | "settings" | "renaming") {
    openPanel = openPanel === panel ? null : panel;
  }
  function close() {
    openPanel = null;
  }

  function handleRefresh() {
    encoder.clearSession();
  }

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
  class="flex items-center justify-between px-4 h-10 shrink-0 select-none"
  style="border-bottom: 1px solid var(--color-border); background: var(--color-panel);"
>
  <!-- Left: logo -->
  <div class="flex items-center gap-3">
    <div class="flex items-center gap-2">
      <div
        class="w-[3px] h-5 rounded-[1px]"
        style="background: var(--color-accent);"
      ></div>
      <span
        class="font-semibold text-[13px] tracking-tight leading-none"
        style="color: var(--color-text);"
      >
        RenCodeX
      </span>
    </div>
    <span
      class="font-mono text-[10px] px-2 py-[3px] rounded-[var(--radius-full)] tracking-widest leading-none"
      style="border: 1px solid var(--color-border); color: var(--color-subtext);"
    >
      H.265 · NVENC
    </span>
  </div>

  <!-- Center: status pill -->
  <div class="flex items-center h-full">
    {#if encoder.encoding}
      <span
        class="flex items-center gap-2 font-mono text-[11px]"
        style="color: var(--color-accent);"
      >
        <span
          class="inline-block w-[6px] h-[6px] rounded-full animate-pulse"
          style="background: var(--color-accent);"
        ></span>
        ENCODING
      </span>
    {:else if doneCount > 0 && doneCount === encoder.files.length}
      <span
        class="flex items-center gap-2 font-mono text-[11px]"
        style="color: var(--color-success);"
      >
        <span
          class="inline-block w-[6px] h-[6px] rounded-full"
          style="background: var(--color-success);"
        ></span>
        DONE — {doneCount} fichier{doneCount > 1 ? "s" : ""}
      </span>
    {:else if encoder.files.length > 0}
      <span class="font-mono text-[11px]" style="color: var(--color-subtext);">
        {readyCount}/{encoder.files.length} prêt{readyCount > 1 ? "s" : ""}
        {#if totalSize > 0}
          <span style="color: var(--color-subtext2);" class="mx-1">·</span
          >{formatSize(totalSize)}
        {/if}
      </span>
    {/if}
  </div>

  <!-- Right: icon buttons -->
  <div class="flex items-center gap-0.5">
    <div class="flex items-center">
      <!-- Logs -->
      <div class="relative inline-flex">
        <button
          type="button"
          onclick={() => toggle("logs")}
          class="topbar-btn relative {openPanel === 'logs'
            ? 'topbar-btn--active'
            : ''}"
          aria-label="Journal des événements"
          aria-pressed={openPanel === "logs"}
          title="Logs{hasAlert
            ? ` · ${errorCount} erreur${errorCount > 1 ? 's' : ''}, ${warnCount} avert.`
            : ''}"
        >
          <Logs class="w-4 h-4" />
          {#if hasAlert}
            <span
              class="absolute top-[5px] right-[5px] w-[5px] h-[5px] rounded-full animate-pulse"
              style="background: var(--color-danger);"
              aria-hidden="true"
            ></span>
          {/if}
        </button>

        {#if openPanel === "logs"}
          <div
            class="popover-backdrop"
            role="presentation"
            onclick={close}
          ></div>
          <div
            class="popover-panel w-[520px] h-[340px]"
            role="dialog"
            aria-modal="true"
            aria-label="Journal"
          >
            <LogConsole onClose={close} />
          </div>
        {/if}
      </div>

      <!-- Dashboard -->
      <div class="relative inline-flex">
        <button
          type="button"
          onclick={() => toggle("dashboard")}
          class="topbar-btn {openPanel === 'dashboard'
            ? 'topbar-btn--active'
            : ''}"
          aria-label="Dashboard des statistiques"
          aria-pressed={openPanel === "dashboard"}
          title="Statistiques"
        >
          <ChartColumnDecreasing class="w-4 h-4" />
        </button>

        {#if openPanel === "dashboard"}
          <div
            class="popover-backdrop"
            role="presentation"
            onclick={close}
          ></div>
          <div
            class="popover-panel w-[340px]"
            role="dialog"
            aria-modal="true"
            aria-label="Dashboard"
          >
            <Dashboard onClose={close} />
          </div>
        {/if}
      </div>
    </div>

    <div class="sep h-4 mx-1.5"></div>

    <!-- Config -->
    <div class="flex items-center">
    <LangPopover />
      <!-- Paramètres encodage -->
      <div class="relative inline-flex">
        <button
          type="button"
          onclick={() => toggle("settings")}
          class="topbar-btn {openPanel === 'settings'
            ? 'topbar-btn--active'
            : ''}"
          aria-label="Paramètres d'encodage"
          aria-pressed={openPanel === "settings"}
          title="Paramètres d'encodage (CRF, Preset)"
        >
          <SlidersHorizontal class="w-4 h-4" />
        </button>

        {#if openPanel === "settings"}
          <div
            class="popover-backdrop"
            role="presentation"
            onclick={close}
          ></div>
          <div
            class="popover-panel w-[340px]"
            role="dialog"
            aria-modal="true"
            aria-label="Paramètres d'encodage"
          >
            <EncodingSettings onClose={close} />
          </div>
        {/if}
      </div>

      <!-- Renommage (ordre des tags & team) -->
      <div class="relative inline-flex">
        <button
          type="button"
          onclick={() => toggle("renaming")}
          class="topbar-btn {openPanel === 'renaming' ? 'topbar-btn--active' : ''}"
          aria-label="Paramètres de renommage"
          aria-pressed={openPanel === "renaming"}
          title="Renommage (ordre des tags, team)"
        >
          <Tags class="w-4 h-4" />
        </button>

        {#if openPanel === "renaming"}
          <div
            class="popover-backdrop"
            role="presentation"
            onclick={close}
          ></div>
          <div
            class="popover-panel w-[340px]"
            role="dialog"
            aria-modal="true"
            aria-label="Paramètres de renommage"
          >
            <RenamingSettings onClose={close} />
          </div>
        {/if}
      </div>
    </div>

    <div class="sep h-4 mx-1.5"></div>
    <!-- Paramètres app -->
    <button
      type="button"
      onclick={() => (showAppSettings = !showAppSettings)}
      class="topbar-btn {showAppSettings ? 'topbar-btn--active' : ''}"
      aria-label="Paramètres de l'application"
      aria-pressed={showAppSettings}
      title="Paramètres (FFmpeg, Discord…)"
    >
      <Wrench class="w-4 h-4" />
    </button>
    <Settings bind:open={showAppSettings} />
    <!-- Rafraîchir -->
    <button
      type="button"
      onclick={handleRefresh}
      class="topbar-btn"
      aria-label="Rafraîchir l'interface"
      title="Rafraîchir l'interface"
    >
      <RefreshCw class="w-4 h-4" />
    </button>
    <!-- Thème -->
    <button
      type="button"
      onclick={() => theme.toggle()}
      class="topbar-btn"
      aria-label={theme.dark
        ? "Activer le thème clair"
        : "Activer le thème sombre"}
      title={theme.dark ? "Passer en mode clair" : "Passer en mode sombre"}
    >
      {#if theme.dark}
        <Sun class="w-4 h-4" />
      {:else}
        <Moon class="w-4 h-4" />
      {/if}
    </button>
  </div>
</header>

<style>
  .topbar-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-xs);
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

  /* Popover shared */
  .popover-backdrop {
    position: fixed;
    inset: 0;
    z-index: 9970;
    background: rgba(0, 0, 0, 0.1);
    animation: fade-in 0.15s ease;
  }

  .popover-panel {
    position: absolute;
    top: calc(100% + 6px);
    right: 0;
    z-index: 9971;
    max-width: calc(100vw - 24px);
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.45);
    animation: slide-down 0.2s cubic-bezier(0.22, 1, 0.36, 1);
    transform-origin: top right;
    overflow: hidden;
    border-radius: var(--radius-lg);
  }

  @keyframes fade-in {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
  @keyframes slide-down {
    from {
      opacity: 0;
      transform: translateY(-10px) scale(0.97);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }
</style>