<script lang="ts">
  import { onMount } from "svelte";
  import { encoder } from "$lib/stores/encoder.svelte";
  import { theme } from "$lib/stores/theme.svelte";
  import { listen } from "@tauri-apps/api/event";
  import type { UnlistenFn } from "@tauri-apps/api/event";

  import TopBar from "$components/TopBar.svelte";
  import DropZone from "$components/DropZone.svelte";
  import FileTable from "$components/FileTable.svelte";
  import LangSelector from "$components/LangSelector.svelte";
  import ProgressPanel from "$components/ProgressPanel.svelte";
  import ControlBar from "$components/ControlBar.svelte";

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

<svelte:window ondragover={(e) => e.preventDefault()} />

<div class="flex flex-col h-screen bg-[var(--color-surface)]">
  <TopBar />

  <main class="flex-1 overflow-y-auto p-4 space-y-3 w-full max-w-[1200px] mx-auto">
    <DropZone />

    <!-- Fichiers -->
    <section>
      <div class="section-label mb-1.5">
        Fichiers
        {#if encoder.files.length > 0}
          <span class="text-[var(--color-accent)] ml-1">{encoder.files.length}</span>
        {/if}
      </div>
      <FileTable />
    </section>

    <!-- Pistes -->
    <div class="grid grid-cols-2 gap-3">
      <section class="border border-[var(--color-border)] rounded-[2px] bg-[var(--color-panel)] p-3">
        <div class="section-label mb-2">Pistes audio</div>
        <LangSelector mode="audio" />
      </section>
      
      <section class="border border-[var(--color-border)] rounded-[2px] bg-[var(--color-panel)] p-3">
        <div class="section-label mb-2">Sous-titres</div>
        <LangSelector mode="sub" />
      </section>
    </div>

    <!-- Progression -->
    <ProgressPanel />
  </main>

  <ControlBar />
</div>