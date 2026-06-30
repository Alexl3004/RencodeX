<script lang="ts">
  import { onMount } from "svelte";
  import { encoder } from "$lib/stores/encoder.svelte";
  import { theme } from "$lib/stores/theme.svelte";
  import { listen } from "@tauri-apps/api/event";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import ToastNotif from "$components/ToastNotif.svelte";

  import TopBar from "$components/TopBar.svelte";
  import DropZone from "$components/DropZone.svelte";
  import FileTable from "$components/FileTable.svelte";
  import ProgressPanel from "$components/ProgressPanel.svelte";
  import ControlBar from "$components/ControlBar.svelte";
  import LogConsole from "$components/LogConsole.svelte";
  import EncodingSettings from "$components/EncodingSettings.svelte";
  import Dashboard from "$components/Dashboard.svelte";

  let showLogs      = $state(false);
  let showSettings  = $state(false);
  let showDashboard = $state(false);
  let showAppSettings = $state(false);

  function closeOnEsc(e: KeyboardEvent) {
    if (e.key === "Escape") {
      showLogs = false;
      showSettings = false;
      showDashboard = false;
      showAppSettings = false;
    }
  }

  onMount(async () => {
    theme.init();
    await encoder.init();
  });

  onMount(() => {
    let unlisten: UnlistenFn | undefined;
    listen<string[]>("tauri://file-drop", async (e) => {
      await encoder.addFiles(e.payload);
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  });
</script>

<svelte:window ondragover={(e) => e.preventDefault()} onkeydown={closeOnEsc} />

<div class="app-shell">
  <TopBar bind:showAppSettings />

  <main class="main-content">
    <!-- DropZone : hauteur fixe -->
    <div class="dropzone-slot">
      <DropZone />
    </div>

    <!-- FileTable : flex-grow 3 (≈55% de l'espace restant) -->
    <section class="filetable-slot">
      <div class="section-label mb-1">
        Fichiers
        {#if encoder.files.length > 0}
          <span class="text-[var(--color-accent)] ml-1">{encoder.files.length}</span>
        {/if}
      </div>
      <div class="filetable-wrap">
        <FileTable />
      </div>
    </section>

    <!-- ProgressPanel : flex-grow 2 (≈45% de l'espace restant) -->
    <div class="progress-slot">
      <ProgressPanel />
    </div>
  </main>

  <ControlBar />
</div>

<!-- ── Overlays — rendus à la racine, hors du flux flex ── -->

{#if showLogs}
  <div
    class="overlay-backdrop"
    role="dialog"
    aria-modal="true"
    aria-label="Journal des événements"
    tabindex="-1"
    onclick={(e) => { if (e.target === e.currentTarget) showLogs = false; }}
    onkeydown={(e) => { if (e.key === 'Escape') showLogs = false; }}
  >
    <div class="overlay-panel" style="width: 700px; max-width: 95vw; height: 70vh;">
      <LogConsole onClose={() => (showLogs = false)} />
    </div>
  </div>
{/if}

{#if showSettings}
  <div
    class="overlay-backdrop"
    role="dialog"
    aria-modal="true"
    aria-label="Paramètres d'encodage"
    tabindex="-1"
    onclick={(e) => { if (e.target === e.currentTarget) showSettings = false; }}
    onkeydown={(e) => { if (e.key === 'Escape') showSettings = false; }}
  >
    <div class="overlay-panel" style="width: 520px; max-width: 95vw; max-height: 85vh; overflow-y: auto;">
      <EncodingSettings onClose={() => (showSettings = false)} />
    </div>
  </div>
{/if}

{#if showDashboard}
  <div
    class="overlay-backdrop"
    role="dialog"
    aria-modal="true"
    aria-label="Dashboard"
    tabindex="-1"
    onclick={(e) => { if (e.target === e.currentTarget) showDashboard = false; }}
    onkeydown={(e) => { if (e.key === 'Escape') showDashboard = false; }}
  >
    <div class="overlay-panel" style="width: 520px; max-width: 95vw; max-height: 85vh; overflow-y: auto;">
      <Dashboard onClose={() => (showDashboard = false)} />
    </div>
  </div>
{/if}

<ToastNotif />

<style>
  /* ── Shell principal : remplit exactement la fenêtre sans scroll ── */
  .app-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
    background: var(--color-surface);
  }

  /* ── Zone centrale : flex column, prend tout l'espace entre TopBar et ControlBar ── */
  .main-content {
    flex: 1 1 0;
    min-height: 0;          /* ← essentiel pour que flex-shrink fonctionne */
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px 14px 8px;
    overflow: hidden;
  }

  /* ── DropZone : taille naturelle, ne grandit pas ── */
  .dropzone-slot {
    flex: 0 0 auto;
  }

  /* ── Section FileTable : grandit (ratio 3) ── */
  .filetable-slot {
    flex: 3 1 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  /* ── Le wrapper qui donne sa hauteur à FileTable ── */
  .filetable-wrap {
    flex: 1 1 0;
    min-height: 0;
    overflow: hidden;
  }

  /* ── ProgressPanel : grandit (ratio 2) ── */
  .progress-slot {
    flex: 2 1 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  /* ── Overlays ────────────────────────────────────────────────────── */
  .overlay-backdrop {
    position: fixed;
    inset: 0;
    z-index: 9990;
    background: color-mix(in srgb, var(--color-surface) 40%, rgba(0,0,0,0.7));
    display: flex;
    align-items: center;
    justify-content: center;
    backdrop-filter: blur(2px);
  }

  .overlay-panel {
    border-radius: var(--radius-lg);
    overflow: hidden;
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.5);
  }
</style>