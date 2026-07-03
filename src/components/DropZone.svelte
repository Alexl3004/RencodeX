<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { invoke } from "@tauri-apps/api/core";
  import {
    File,
    FolderOpen,
    FolderInput,
    ChevronDown,
    Loader2,
    Check,
  } from "@lucide/svelte";
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
    if (p.length <= 48) return p;
    return "…/" + parts.slice(-2).join("/");
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

<div class="toolbar">
  <!-- Source label -->
  <span class="section-label">Source</span>

  <!-- Source actions -->
  <div class="action-group">
    <button
      onclick={pickFiles}
      disabled={encoder.encoding}
      class="action-btn"
      title="Ajouter des fichiers vidéo"
      aria-label="Ajouter des fichiers"
    >
      <File class="btn-icon" />
      <span>Fichiers</span>
    </button>

    <div class="action-sep"></div>

    <button
      onclick={pickFolder}
      disabled={encoder.encoding || scanning}
      class="action-btn"
      title="Scanner un dossier"
      aria-label="Scanner un dossier"
    >
      {#if scanning}
        <Loader2 class="btn-icon animate-spin" />
        <span>Scan…</span>
      {:else}
        <FolderOpen class="btn-icon" />
        <span>Dossier</span>
      {/if}
    </button>
  </div>

  <!-- Spacer -->
  <div class="flex-1"></div>

  <div class="toolbar-divider"></div>

  <!-- Sortie label -->
  <span class="section-label">Sortie</span>

  <!-- Output path -->
  <div class="output-path-wrap">
    {#if history.length > 0}
      <Popover positioning={{ placement: "bottom-end" }}>
        <Popover.Trigger
          disabled={encoder.encoding}
          class="output-trigger"
          title={encoder.outputDir || "Choisir un dossier de sortie"}
          style="display:flex; flex-direction:row; align-items:center; flex-wrap:nowrap; width:100%; overflow:hidden;"
        >
          <span class="output-path-text">{encoder.outputDir ? encoder.outputDir : "Choisir…"}</span>
          <ChevronDown class="chevron-icon" />
        </Popover.Trigger>
        <Portal>
          <Popover.Positioner>
            <Popover.Content class="history-popover">
              <p class="history-header">Récents</p>
              {#each history as dir}
                <Popover.CloseTrigger
                  onclick={() => setOutputDir(dir)}
                  class="history-item {dir === encoder.outputDir ? 'history-item--active' : ''}"
                  title={dir}
                >
                  <span class="history-item-path">{shortPath(dir)}</span>
                  {#if dir === encoder.outputDir}
                    <Check class="history-check" />
                  {/if}
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
        class="output-trigger output-trigger--empty"
        title="Choisir le dossier de sortie"
      >
        <span class="output-path-text output-path-text--placeholder">Choisir un dossier…</span>
      </button>
    {/if}
  </div>

  <!-- Browse button -->
  <button
    onclick={pickOutputDir}
    disabled={encoder.encoding}
    class="output-browse-btn"
    title="Parcourir"
    aria-label="Choisir le dossier de sortie"
  >
    <FolderInput class="btn-icon" />
  </button>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 0;
    height: 36px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-surface);
    overflow: hidden;
    padding: 0 4px;
  }

  .section-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--color-subtext);
    white-space: nowrap;
    opacity: 0.6;
    padding: 0 8px;
    flex-shrink: 0;
  }

  .toolbar-divider {
    width: 1px;
    height: 18px;
    background: var(--color-border);
    flex-shrink: 0;
    margin: 0 2px;
  }

  /* ── Action group ────────────────────────────── */
  .action-group {
    display: flex;
    align-items: center;
    gap: 2px;
    flex-shrink: 0;
  }

  .action-sep {
    width: 1px;
    height: 14px;
    background: var(--color-border);
    flex-shrink: 0;
    margin: 0 2px;
  }

  .action-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    border: none;
    background: transparent;
    color: var(--color-subtext);
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    font-weight: 500;
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.12s, color 0.12s;
  }

  .action-btn:hover:not(:disabled) {
    background: var(--color-panel);
    color: var(--color-text);
  }

  .action-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  /* ── Output path ─────────────────────────────── */
  .output-path-wrap {
    min-width: 0;
    flex: 1;
    overflow: hidden;
  }

  .output-trigger {
    display: flex;
    align-items: center;
    flex-direction: row;
    gap: 3px;
    width: 100%;
    padding: 4px 6px;
    border-radius: var(--radius-sm);
    border: none;
    background: transparent;
    cursor: pointer;
    transition: background 0.12s;
    min-width: 0;
    overflow: hidden;
  }

  .output-trigger:hover:not(:disabled) {
    background: var(--color-panel);
  }

  .output-trigger:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .output-path-text {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    direction: rtl;
    text-align: left;
    flex: 1;
    min-width: 0;
  }

  .output-path-text--placeholder {
    color: var(--color-subtext);
    opacity: 0.6;
    font-style: italic;
    direction: ltr;
  }

  .output-browse-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 4px 6px;
    border-radius: var(--radius-sm);
    border: none;
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    transition: background 0.12s, color 0.12s;
    flex-shrink: 0;
  }

  .output-browse-btn:hover:not(:disabled) {
    background: var(--color-panel);
    color: var(--color-text);
  }

  .output-browse-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  /* ── History popover ─────────────────────────── */
  :global(.history-popover) {
    min-width: 240px;
    z-index: 30;
    border-radius: var(--radius-sm);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.18);
    padding: 4px 0;
    background: var(--color-panel);
    border: 1px solid var(--color-border);
  }

  :global(.history-header) {
    padding: 4px 12px 6px;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--color-subtext);
    opacity: 0.6;
    border-bottom: 1px solid var(--color-border);
    margin-bottom: 2px;
  }

  :global(.history-item) {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 6px 12px;
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-text);
    text-align: left;
    background: transparent;
    border: none;
    cursor: pointer;
    gap: 8px;
    transition: background 0.1s;
  }

  :global(.history-item:hover) {
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-panel));
  }

  :global(.history-item--active) {
    color: var(--color-accent);
  }

  :global(.history-item-path) {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  :global(.history-check) {
    width: 11px;
    height: 11px;
    flex-shrink: 0;
    color: var(--color-accent);
  }
</style>