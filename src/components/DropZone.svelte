<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { invoke } from "@tauri-apps/api/core";

  let dragging = $state(false);
  let scanning = $state(false);
  let showHistory = $state(false);

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
    showHistory = false;
    const dir = await open({ directory: true, defaultPath: encoder.outputDir });
    if (dir && typeof dir === "string") setOutputDir(dir);
  }

  function setOutputDir(dir: string) {
    encoder.outputDir = dir;
    encoder.log(`Dossier de sortie : ${dir}`, "info");
    saveHistory(dir);
    history = loadHistory();
    showHistory = false;
  }
</script>

<svelte:window
  onclick={() => {
    if (showHistory) showHistory = false;
  }}
  onkeydown={(e) => {
    if (e.key === "Escape") showHistory = false;
  }}
/>

<div class="border border-[var(--color-border)] rounded-[2px] bg-[var(--color-panel)] overflow-visible">
  <!-- Source -->
  <div class="flex items-center">
    <div class="flex items-center justify-center w-10 shrink-0 border-r border-[var(--color-border)] text-[var(--color-subtext)]">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
        <polyline points="17 8 12 3 7 8" />
        <line x1="12" y1="3" x2="12" y2="15" />
      </svg>
    </div>
    
    <div class="flex-1 px-3 py-3">
      <p class="text-[11px] font-medium text-[var(--color-text)]">Source</p>
    </div>

    <div class="flex border-l border-[var(--color-border)]">
      <button onclick={pickFiles} disabled={encoder.encoding} class="btn btn-ghost px-3 py-2 text-[10px] font-mono">
        FICHIERS
      </button>
      <button onclick={pickFolder} disabled={encoder.encoding || scanning} class="btn btn-ghost px-3 py-2 text-[10px] font-mono border-l border-[var(--color-border)]">
        {#if scanning}⏳{:else}DOSSIER{/if}
      </button>
    </div>
  </div>

  <div class="border-t border-[var(--color-border)]"></div>

  <!-- Destination avec historique -->
  <div class="flex items-center relative">
    <div class="flex items-center justify-center w-10 shrink-0 border-r border-[var(--color-border)] text-[var(--color-subtext)]">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
      </svg>
    </div>

    <div class="flex-1 relative">
      <button
        onclick={(e) => {
          e.stopPropagation();
          if (history.length > 0) showHistory = !showHistory;
          else pickOutputDir();
        }}
        disabled={encoder.encoding}
        class="w-full text-left px-3 py-2 text-[10px] font-mono text-[var(--color-text)] truncate"
        title={encoder.outputDir || "Cliquer pour choisir"}
      >
        {encoder.outputDir ? shortPath(encoder.outputDir) : "Choisir dossier"}
        {#if history.length > 0}
          <svg class="inline-block ml-1 text-[var(--color-subtext)]" width="8" height="8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="6 9 12 15 18 9" />
          </svg>
        {/if}
      </button>

      {#if showHistory && history.length > 0}
        <ul class="absolute top-full left-0 w-full min-w-[260px] z-30 bg-[var(--color-panel)] border border-[var(--color-border)] shadow-lg rounded-[2px] py-1">
          <li class="px-3 py-1 text-[9px] text-[var(--color-subtext)] uppercase font-mono border-b border-[var(--color-border)]">
            Récents
          </li>
          {#each history as dir}
            <li>
              <button
                onclick={() => setOutputDir(dir)}
                class="w-full text-left px-3 py-2 text-[10px] font-mono truncate hover:bg-[var(--color-panel2)] transition-colors
                       {dir === encoder.outputDir ? 'text-[var(--color-accent)]' : 'text-[var(--color-text)]'}"
                title={dir}
              >
                {shortPath(dir)}
              </button>
            </li>
          {/each}
        </ul>
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