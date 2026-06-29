<script lang="ts">
  import {
    encoder,
    type AppFile,
    formatDuration,
  } from "$lib/stores/encoder.svelte";
  import { formatSize } from "$lib/utils";
  import RenameModal from "$components/RenameModal.svelte";  
  import { Info, Pencil, X } from '@lucide/svelte';


  let editingFile = $state<AppFile | null>(null);
  let selectedFile = $state<AppFile | null>(null);
  let showFileInfo = $state(false);

  function openRename(file: AppFile) {
    if (!encoder.encoding) editingFile = file;
  }
  function handleRename(n: string) {
    if (editingFile) encoder.renameFile(editingFile.path, n);
    editingFile = null;
  }
  function showFileDetails(f: AppFile) {
    selectedFile = f;
    showFileInfo = true;
  }
  function closeFileInfo() {
    showFileInfo = false;
    selectedFile = null;
  }

  function isCurrentlyEncoding(file: AppFile): boolean {
    return (
      file.status === "encoding" &&
      encoder.progress?.file_index ===
        encoder.files.findIndex((f) => f.path === file.path)
    );
  }

  function isPending(file: AppFile): boolean {
    if (!encoder.encoding) return false;
    if (file.status !== "ready") return false;
    const currentIndex = encoder.progress?.file_index;
    if (currentIndex === undefined) return false;
    const fileIndex = encoder.files.findIndex((f) => f.path === file.path);
    return fileIndex > currentIndex;
  }

  function getStatusLabel(file: AppFile): string {
    if (file.status === "analysing") return "Analyse";
    if (file.status === "encoding") return "En cours";
    if (file.status === "ready") {
      if (isPending(file)) return "En attente";
      return "Prêt";
    }
    if (file.status === "done") return "Terminé";
    if (file.status === "error") return "Erreur";
    return "—";
  }

  const STATUS_COLOR: Record<string, string> = {
    analysing: "text-[var(--color-accent)] animate-pulse",
    encoding:  "text-[var(--color-accent)]",
    ready:     "text-[var(--color-success)]",
    pending:   "text-[var(--color-subtext)]",
    done:      "text-[var(--color-success)]",
    error:     "text-[var(--color-danger)]",
  };

  function getStatusColor(file: AppFile): string {
    if (file.status === "ready" && isPending(file)) return STATUS_COLOR.pending;
    return STATUS_COLOR[file.status] ?? "";
  }
</script>

{#if editingFile}
  <RenameModal
    file={editingFile}
    onclose={() => (editingFile = null)}
    onrename={handleRename}
  />
{/if}

<!-- File info modal (native overlay) -->
{#if showFileInfo && selectedFile}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    aria-label="Informations fichier"
    tabindex="-1"
    onclick={(e) => { if (e.target === e.currentTarget) closeFileInfo(); }}
    onkeydown={(e) => { if (e.key === 'Escape') closeFileInfo(); }}
  >
    <div class="modal-box" style="width: 480px; max-width: 95vw;">
      <!-- Header -->
      <div class="modal-header">
        <div class="flex items-center gap-2">
          <div class="w-[3px] h-4 rounded-[1px]" style="background: var(--color-accent);"></div>
          <span class="font-mono text-[12px] font-semibold uppercase tracking-wider" style="color: var(--color-text);">
            Infos fichier
          </span>
        </div>
        <button onclick={closeFileInfo} class="modal-close-btn" aria-label="Fermer">✕</button>
      </div>

      <!-- Body -->
      <div class="modal-body space-y-3">
        <div>
          <div class="section-label mb-1">Fichier source</div>
          <div class="px-3 py-2 rounded-[2px] font-mono text-[11px] break-all"
               style="background: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-text);">
            {selectedFile.filename}
          </div>
        </div>

        <div>
          <div class="section-label mb-1">Fichier de sortie</div>
          <div class="px-3 py-2 rounded-[2px] font-mono text-[11px] break-all"
               style="background: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-success);">
            {selectedFile.output_name}{selectedFile.output_ext}
          </div>
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div class="p-3 rounded-[2px]" style="background: var(--color-surface); border: 1px solid var(--color-border);">
            <div class="section-label mb-1">Taille</div>
            <div class="font-mono text-[13px]" style="color: var(--color-text);">{formatSize(selectedFile.size_mb)}</div>
          </div>
          <div class="p-3 rounded-[2px]" style="background: var(--color-surface); border: 1px solid var(--color-border);">
            <div class="section-label mb-1">Durée</div>
            <div class="font-mono text-[13px]" style="color: var(--color-text);">{formatDuration(selectedFile.duration_secs)}</div>
          </div>
          <div class="p-3 rounded-[2px]" style="background: var(--color-surface); border: 1px solid var(--color-border);">
            <div class="section-label mb-1">FPS</div>
            <div class="font-mono text-[13px]" style="color: var(--color-text);">{selectedFile.fps?.toFixed(2) || "—"} fps</div>
          </div>
          <div class="p-3 rounded-[2px]" style="background: var(--color-surface); border: 1px solid var(--color-border);">
            <div class="section-label mb-1">Statut</div>
            <div class="font-mono text-[13px]" style="color: var(--color-text);">{getStatusLabel(selectedFile)}</div>
          </div>
        </div>

        {#if selectedFile.audio_langs?.length > 0}
          <div>
            <div class="section-label mb-1.5">Pistes audio</div>
            <div class="flex flex-wrap gap-1">
              {#each selectedFile.audio_langs as lang}
                <span class="font-mono text-[10px] px-2 py-0.5 rounded-[2px]"
                      style="background: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-subtext);">
                  {lang.toUpperCase()}
                </span>
              {/each}
            </div>
          </div>
        {/if}

        <div>
          <div class="section-label mb-1.5">Sous-titres</div>
          <div class="flex flex-wrap gap-1">
            {#if selectedFile.sub_langs?.length > 0}
              {#each selectedFile.sub_langs as lang}
                <span class="font-mono text-[10px] px-2 py-0.5 rounded-[2px]"
                      style="background: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-subtext);">
                  {lang.toUpperCase()}
                </span>
              {/each}
            {:else}
              <span class="text-[11px] italic" style="color: var(--color-subtext);">Aucun</span>
            {/if}
          </div>
        </div>

        {#if selectedFile.streams?.length > 0}
          <div>
            <div class="section-label mb-1.5">Flux</div>
            <div class="space-y-0.5 max-h-32 overflow-y-auto">
              {#each selectedFile.streams as s}
                <div class="font-mono text-[10px] py-1 flex gap-2"
                     style="border-bottom: 1px solid color-mix(in srgb, var(--color-border) 30%, transparent);">
                  <span style="color: var(--color-accent);">{s.codec_type?.toUpperCase()}</span>
                  <span style="color: var(--color-subtext);">{s.codec_name}</span>
                  {#if s.width && s.height}
                    <span style="color: var(--color-subtext2);">{s.width}×{s.height}</span>
                  {/if}
                  <span style="color: var(--color-subtext2);">[{s.language?.toUpperCase() || "und"}]</span>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="modal-footer">
        <button onclick={closeFileInfo} class="btn btn-secondary font-mono text-[11px]">Fermer</button>
      </div>
    </div>
  </div>
{/if}

<!-- Table -->
<div class="rounded-[2px] overflow-auto h-full"
     style="border: 1px solid var(--color-border);">
  {#if encoder.files.length === 0}
    <div class="py-10 text-center flex items-center justify-center h-full">
      <p class="text-[11px] font-mono uppercase tracking-widest" style="color: var(--color-subtext);">
        Aucun fichier
      </p>
    </div>
  {:else}
    <table class="w-full text-[11px] file-table">
      <thead class="sticky top-0 z-10" style="background: var(--color-surface);">
        <tr>
          <th class="text-left w-[400px]">Fichier de sortie</th>
          <th class="text-right w-20">Taille</th>
          <th class="text-center w-24">Audio</th>
          <th class="text-center w-24">Sous-titres</th>
          <th class="text-right w-24">Temps</th>
          <th class="text-center w-24">Statut</th>
          <th class="w-16"></th>
        </tr>
      </thead>
      <tbody>
        {#each encoder.files as file (file.path)}
          {@const isCurrentEncoding = isCurrentlyEncoding(file)}
          {@const isFilePending = isPending(file)}
          <tr class="group transition-colors {isCurrentEncoding ? 'row-encoding' : ''}">
            <!-- Nom -->
            <td class="max-w-[400px]">
              <div class="flex items-center gap-1.5">
                <button
                  onclick={() => showFileDetails(file)}
                  class="shrink-0 transition-colors"
                  style="color: var(--color-subtext);"
                  title="Infos"
                  aria-label="Informations fichier"
                  onmouseenter={(e) => (e.currentTarget as HTMLElement).style.color = 'var(--color-accent)'}
                  onmouseleave={(e) => (e.currentTarget as HTMLElement).style.color = 'var(--color-subtext)'}
                >
                  <Info class="w-3.5 h-3.5" />
                </button>
                <button
                  onclick={() => openRename(file)}
                  disabled={encoder.encoding}
                  class="shrink-0 transition-colors disabled:opacity-30"
                  style="color: var(--color-subtext);"
                  title="Renommer"
                  aria-label="Renommer le fichier"
                  onmouseenter={(e) => (e.currentTarget as HTMLElement).style.color = 'var(--color-accent)'}
                  onmouseleave={(e) => (e.currentTarget as HTMLElement).style.color = 'var(--color-subtext)'}
                >
                  <Pencil class="w-3.5 h-3.5" />
                </button>
                <span class="truncate font-mono flex-1 min-w-0" style="color: var(--color-text);"
                      title={file.output_name + file.output_ext}>
                  {file.output_name}{file.output_ext}
                </span>
              </div>
            </td>
            <!-- Taille -->
            <td class="text-right font-mono" style="color: var(--color-subtext);">
              {file.size_mb > 0 ? formatSize(file.size_mb) : "—"}
            </td>
            <!-- Audio -->
            <td class="text-center font-mono" style="color: var(--color-subtext);">
              {#if file.status === "analysing"}
                <span class="animate-pulse" style="color: var(--color-accent);">…</span>
              {:else}
                <span
                  class="inline-block max-w-[88px] truncate align-middle"
                  title={file.audio_langs.map((l) => l.toUpperCase()).join(" ") || "—"}
                >
                  {file.audio_langs.map((l) => l.toUpperCase()).join(" ") || "—"}
                </span>
              {/if}
            </td>
            <!-- Subs -->
            <td class="text-center font-mono" style="color: var(--color-subtext);">
              <span
                class="inline-block max-w-[88px] truncate align-middle"
                title={file.sub_langs.map((l) => l.toUpperCase()).join(" ") || "—"}
              >
                {file.sub_langs.map((l) => l.toUpperCase()).join(" ") || "—"}
              </span>
            </td>
            <!-- Temps -->
            <td class="text-right font-mono" style="color: var(--color-subtext);">
              {#if file.status === "done" && file.result?.duration_secs}
                {Math.floor(file.result.duration_secs / 60)}m{Math.floor(file.result.duration_secs % 60)}s
              {:else if (file.status === "encoding" || file.status === "ready") && file.duration_secs > 0}
                {Math.floor(file.duration_secs / 60)}m{Math.floor(file.duration_secs % 60)}s
              {:else}—{/if}
            </td>
            <!-- Statut -->
            <td class="text-center">
              {#if isCurrentEncoding && encoder.progress}
                <div class="flex flex-col items-center gap-0.5">
                  <span class="font-mono text-[10px] font-bold animate-pulse" style="color: var(--color-accent);">
                    {getStatusLabel(file)}
                  </span>
                  <span class="font-mono text-[9px]" style="color: var(--color-subtext);">
                    {Math.round(encoder.progress.percent)}%
                  </span>
                </div>
              {:else}
                <span class="font-mono text-[10px] {getStatusColor(file)}">
                  {getStatusLabel(file)}
                </span>
              {/if}
              {#if file.status === "done" && file.result && !isCurrentEncoding}
                <div class="font-mono text-[10px]" style="color: var(--color-success);">
                  -{(((file.result.original_mb - file.result.encoded_mb) / file.result.original_mb) * 100).toFixed(1)}%
                </div>
              {/if}
            </td>
            <!-- Suppr -->
            <td class="text-center">
              <button
                class="row-delete-btn opacity-0 group-hover:opacity-100 transition-opacity font-mono text-[10px] disabled:hidden"
                onclick={() => encoder.removeFile(file.path)}
                disabled={encoder.encoding}
                title="Supprimer"
                aria-label="Supprimer le fichier"
              >
                <X class="w-3.5 h-3.5" />
              </button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style>
  .row-encoding td {
    background: color-mix(in srgb, var(--color-accent) 5%, transparent);
  }

  /* Hauteur de ligne fixe : évite qu'une ligne avec 20 pistes audio explose */
  tbody tr {
    height: 34px;
    max-height: 34px;
  }
  tbody td {
    overflow: hidden;
    white-space: nowrap;
  }

  .row-delete-btn {
    color: var(--color-subtext);
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 2px;
    border-radius: 3px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: color 0.1s;
  }
  .row-delete-btn:hover {
    color: var(--color-danger);
  }

  /* Modal overlay */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 9999;
    background: rgba(0, 0, 0, 0.65);
    display: flex;
    align-items: center;
    justify-content: center;
    backdrop-filter: blur(2px);
  }

  .modal-box {
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    display: flex;
    flex-direction: column;
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.5);
    overflow: hidden;
    max-height: 85vh;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-panel);
    flex-shrink: 0;
  }

  .modal-body {
    padding: 16px;
    overflow-y: auto;
    flex: 1;
  }

  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 16px;
    border-top: 1px solid var(--color-border);
    background: var(--color-panel);
    flex-shrink: 0;
  }

  .modal-close-btn {
    width: 24px;
    height: 24px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--color-subtext);
    font-size: 12px;
    cursor: pointer;
    transition: background 0.1s, color 0.1s, border-color 0.1s;
  }
  .modal-close-btn:hover {
    background: var(--color-panel2);
    border-color: var(--color-border);
    color: var(--color-text);
  }
</style>