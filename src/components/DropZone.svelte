<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { invoke } from "@tauri-apps/api/core";
  import { Download, Folder, ChevronDown, Loader2, FilePlus2, FolderOpen } from '@lucide/svelte';
  import { Popover, Portal } from "@skeletonlabs/skeleton-svelte";

  let dragging = $state(false);
  let scanning = $state(false);

  const HISTORY_KEY = "rencodex-output-history";
  const MAX_HISTORY = 5;

  function loadHistory(): string[] {
    try {
      return JSON.parse(localStorage.getItem(HISTORY_KEY) ?? "[]");
    } catch {
      return [];
    }
  }
  function saveHistory(dir: string) {
    const h = [dir, ...loadHistory().filter((d) => d !== dir)].slice(0, MAX_HISTORY);
    localStorage.setItem(HISTORY_KEY, JSON.stringify(h));
  }

  let history = $state<string[]>(loadHistory());

  function shortPath(p: string): string {
    if (!p) return "";
    const parts = p.split(/[\\/]/);
    if (p.length <= 52) return p;
    return "…\\" + parts.slice(-2).join("\\");
  }

  async function pickFiles() {
    const result = await open({
      multiple: true,
      filters: [{ name: "Vidéo", extensions: ["mp4", "mkv", "avi", "mov", "flv"] }],
    });
    if (!result) return;
    await encoder.addFiles(Array.isArray(result) ? result : [result]);
  }

  async function pickFolder() {
    const folder = await open({ directory: true });
    if (!folder || typeof folder !== "string") return;
    scanning = true;
    try {
      const paths = await invoke<string[]>("scan_folder", {
        folder,
        extensions: ["mp4", "mkv", "avi", "mov", "flv"],
      });
      if (paths.length === 0) {
        encoder.log(`Aucun fichier vidéo : ${folder}`, "warn");
        return;
      }
      await encoder.addFiles(paths);
    } catch (e) {
      encoder.log(`Erreur scan : ${e}`, "error");
    } finally {
      scanning = false;
    }
  }

  async function pickOutputDir() {
    const dir = await open({ directory: true, defaultPath: encoder.outputDir });
    if (dir && typeof dir === "string") setOutputDir(dir);
  }

  function setOutputDir(dir: string) {
    encoder.outputDir = dir;
    encoder.log(`Dossier de sortie : ${dir}`, "info");
    saveHistory(dir);
    history = loadHistory();
  }
</script>

<div class="relative">
  <div class="border border-[var(--color-border)] rounded-[var(--radius-md)] bg-[var(--color-panel)] overflow-hidden">
    <!-- Source -->
    <div class="flex items-center">
      <div class="flex items-center justify-center w-10 shrink-0 border-r border-[var(--color-border)] text-[var(--color-subtext)]">
        <Download height="1em" />
      </div>

      <div class="flex-1 px-3 py-3">
        <p class="text-[11px] font-medium text-[var(--color-text)]">Source</p>
      </div>

      <div class="flex border-l border-[var(--color-border)]">
        <button
          onclick={pickFiles}
          disabled={encoder.encoding}
          class="btn btn-ghost px-3 py-2"
          title="Ajouter des fichiers"
          aria-label="Ajouter des fichiers"
        >
          <FilePlus2 class="w-4 h-4" />
        </button>
        <button
          onclick={pickFolder}
          disabled={encoder.encoding || scanning}
          class="btn btn-ghost px-3 py-2 border-l border-[var(--color-border)]"
          title="Ajouter un dossier"
          aria-label="Ajouter un dossier"
        >
          {#if scanning}
            <Loader2 class="w-4 h-4 animate-spin" />
          {:else}
            <FolderOpen class="w-4 h-4" />
          {/if}
        </button>
      </div>
    </div>

    <div class="border-t border-[var(--color-border)]"></div>

    <!-- Destination -->
    <div class="flex items-center">
      <div class="flex items-center justify-center w-10 shrink-0 border-r border-[var(--color-border)] text-[var(--color-subtext)]">
        <Folder height="1em" />
      </div>

      <div class="flex-1 min-w-0">
        {#if history.length > 0}
          <Popover positioning={{ placement: "bottom-start" }}>
            <Popover.Trigger
              disabled={encoder.encoding}
              class="w-full text-left px-3 py-2 text-[10px] font-mono text-[var(--color-text)] truncate flex items-center gap-1"
              title={encoder.outputDir || "Cliquer pour choisir"}
            >
              <span class="truncate">{encoder.outputDir ? shortPath(encoder.outputDir) : "Choisir dossier"}</span>
              <ChevronDown class="shrink-0 text-[var(--color-subtext)]" height="1em" />
            </Popover.Trigger>
            <Portal>
              <Popover.Positioner>
                <Popover.Content
                  class="min-w-[260px] z-30 rounded-[var(--radius-sm)] shadow-lg py-1"
                  style="background: var(--color-panel); border: 1px solid var(--color-border);"
                >
                  <p class="px-3 py-1 text-[9px] uppercase font-mono border-b border-[var(--color-border)]"
                     style="color: var(--color-subtext);">Récents</p>
                  {#each history as dir}
                    <Popover.CloseTrigger
                      onclick={() => setOutputDir(dir)}
                      class="w-full text-left px-3 py-2 text-[10px] font-mono truncate block hover:bg-[var(--color-panel2)] transition-colors
                             {dir === encoder.outputDir ? 'text-[var(--color-accent)]' : 'text-[var(--color-text)]'}"
                      title={dir}
                    >
                      {shortPath(dir)}
                    </Popover.CloseTrigger>
                  {/each}
                </Popover.Content>
              </Popover.Positioner>
            </Portal>
          </Popover>
        {:else}
          <button
            onclick={pickOutputDir}
            disabled={encoder.encoding}
            class="w-full text-left px-3 py-2 text-[10px] font-mono truncate"
            style="color: var(--color-subtext);"
          >
            Choisir dossier…
          </button>
        {/if}
      </div>

      <button
        onclick={pickOutputDir}
        disabled={encoder.encoding}
        class="btn btn-ghost px-3 py-2 text-[10px] font-mono border-l border-[var(--color-border)] shrink-0"
      >
        CHOISIR
      </button>
    </div>
  </div>

</div>