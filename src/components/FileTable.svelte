<script lang="ts">
  import {
    encoder,
    type AppFile,
    formatDuration,
  } from "$lib/stores/encoder.svelte";
  import { formatSize } from "$lib/utils";
  import FileModal from "$components/FileModal.svelte";
  import { X, Search } from '@lucide/svelte';


  let filterText   = $state("");
  let filterStatus = $state<"all" | "ready" | "queued" | "encoding" | "done" | "error">("all");
  let selectedFile = $state<AppFile | null>(null);

  function openModal(f: AppFile) {
    selectedFile = f;
  }
  function handleRename(n: string) {
    if (selectedFile) encoder.renameFile(selectedFile.path, n);
    selectedFile = null;
  }
  function closeModal() {
    selectedFile = null;
  }

  function isCurrentlyEncoding(file: AppFile): boolean {
    return (
      file.status === "encoding" &&
      encoder.progress?.file_index ===
        encoder.files.findIndex((f) => f.path === file.path)
    );
  }

  // Compte les fichiers par statut pour les badges de filtres
  let statusCounts = $derived.by(() => {
    const counts = { all: encoder.files.length, ready: 0, queued: 0, encoding: 0, done: 0, error: 0 };
    for (const f of encoder.files) {
      if (f.status === "queued" || (f.status === "ready" && isPending(f))) counts.queued++;
      else if (f.status === "encoding" && !isPending(f)) counts.encoding++;
      else if (f.status === "ready") counts.ready++;
      else if (f.status === "done") counts.done++;
      else if (f.status === "error") counts.error++;
    }
    return counts;
  });

  function isPending(file: AppFile): boolean {
    if (!encoder.encoding) return false;
    if (file.status !== "ready") return false;
    const currentIndex = encoder.progress?.file_index;
    if (currentIndex === undefined) return false;
    const fileIndex = encoder.files.findIndex((f) => f.path === file.path);
    return fileIndex > currentIndex;
  }

  let filteredFiles = $derived.by(() => {
    let result = encoder.files;
    if (filterStatus !== "all") {
      result = result.filter(f => {
        if (filterStatus === "queued") return f.status === "queued" || isPending(f);
        if (filterStatus === "encoding") return f.status === "encoding" && !isPending(f);
        return f.status === filterStatus;
      });
    }
    if (filterText.trim()) {
      const q = filterText.trim().toLowerCase();
      result = result.filter(f =>
        f.output_name.toLowerCase().includes(q) ||
        f.filename.toLowerCase().includes(q)
      );
    }
    return result;
  });

  function getStatusLabel(file: AppFile): string {
    if (file.status === "analysing") return "Analyse";
    if (file.status === "encoding") return "En cours";
    if (file.status === "queued")   return "En file";
    if (file.status === "ready") {
      if (isPending(file)) return "En file";
      return "Prêt";
    }
    if (file.status === "done") return "Terminé";
    if (file.status === "error") return "Erreur";
    return "—";
  }

  const STATUS_COLOR: Record<string, string> = {
    analysing: "text-[var(--color-accent)] animate-pulse",
    encoding:  "text-[var(--color-accent)]",
    queued:    "text-[var(--color-subtext)]",
    ready:     "text-[var(--color-success)]",
    pending:   "text-[var(--color-subtext)]",
    done:      "text-[var(--color-success)]",
    error:     "text-[var(--color-danger)]",
  };

  function getStatusColor(file: AppFile): string {
    if (file.status === "queued") return STATUS_COLOR.queued;
    if (file.status === "ready" && isPending(file)) return STATUS_COLOR.pending;
    return STATUS_COLOR[file.status] ?? "";
  }
</script>

{#if selectedFile}
  <FileModal
    file={selectedFile}
    onclose={closeModal}
    onrename={handleRename}
  />
{/if}

<!-- Table -->
<div class="flex flex-col rounded-[var(--radius-md)] h-full overflow-hidden"
     style="border: 1px solid var(--color-border);">

  <!-- Barre de filtres -->
  {#if encoder.files.length > 0}
    <div class="flex items-center gap-1.5 px-2 py-1.5 shrink-0"
         style="border-bottom: 1px solid var(--color-border); background: var(--color-surface);">
      <!-- Recherche texte -->
      <div class="relative flex items-center">
        <Search class="absolute left-2 w-3 h-3 pointer-events-none" style="color: var(--color-subtext);" />
        <input
          type="text"
          bind:value={filterText}
          placeholder="Rechercher…"
          class="font-mono text-[10px] pl-6 pr-2 py-1 rounded-[var(--radius-sm)] w-36 outline-none"
          style="background: var(--color-panel); border: 1px solid var(--color-border); color: var(--color-text);"
        />
        {#if filterText}
          <button
            onclick={() => filterText = ""}
            class="absolute right-1.5"
            style="color: var(--color-subtext);"
            aria-label="Effacer la recherche"
          >
            <X class="w-3 h-3" />
          </button>
        {/if}
      </div>

      <!-- Filtres statut -->
      {#each [
        { key: "all",      label: "Tous",      count: statusCounts.all },
        { key: "ready",    label: "Prêt",      count: statusCounts.ready },
        { key: "queued",   label: "En file",   count: statusCounts.queued },
        { key: "encoding", label: "En cours",  count: statusCounts.encoding },
        { key: "done",     label: "Terminé",   count: statusCounts.done },
        { key: "error",    label: "Erreur",    count: statusCounts.error },
      ] as btn}
        {#if btn.count > 0 || btn.key === "all"}
          <button
            onclick={() => filterStatus = btn.key as typeof filterStatus}
            class="font-mono text-[9px] px-1.5 py-0.5 rounded-[var(--radius-sm)] flex items-center gap-1 transition-colors"
            style={filterStatus === btn.key
              ? "background: var(--color-accent); color: #fff;"
              : "background: var(--color-panel); border: 1px solid var(--color-border); color: var(--color-subtext);"}
            aria-pressed={filterStatus === btn.key}
          >
            {btn.label}
            {#if btn.count > 0}
              <span class="opacity-75">{btn.count}</span>
            {/if}
          </button>
        {/if}
      {/each}

      {#if filterText || filterStatus !== "all"}
        <span class="ml-auto font-mono text-[9px]" style="color: var(--color-subtext);">
          {filteredFiles.length}/{encoder.files.length}
        </span>
      {/if}
    </div>
  {/if}

  <div class="flex-1 overflow-auto">
  {#if encoder.files.length === 0}
    <div class="py-10 text-center flex items-center justify-center h-full">
      <p class="text-[11px] font-mono uppercase tracking-widest" style="color: var(--color-subtext);">
        Aucun fichier
      </p>
    </div>
  {:else if filteredFiles.length === 0}
    <div class="py-10 text-center flex items-center justify-center h-full">
      <p class="text-[11px] font-mono uppercase tracking-widest" style="color: var(--color-subtext);">
        Aucun résultat
      </p>
    </div>
  {:else}
    <table class="w-full text-[11px] file-table">
      <colgroup>
        <col class="col-name" />
        <col style="width: 76px;" />
        <col style="width: 72px;" />
        <col style="width: 90px;" />
        <col style="width: 72px;" />
        <col style="width: 80px;" />
        <col style="width: 36px;" />
      </colgroup>
      <thead class="sticky top-0 z-10" style="background: var(--color-surface);">
        <tr>
          <th class="text-left">Fichier de sortie</th>
          <th class="text-right">Taille</th>
          <th class="text-center">Audio</th>
          <th class="text-center">Sous-titres</th>
          <th class="text-right">Temps</th>
          <th class="text-center">Statut</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {#each filteredFiles as file (file.path)}
          {@const isCurrentEncoding = isCurrentlyEncoding(file)}
          {@const isFilePending = isPending(file)}
          <tr
            class="group transition-colors {isCurrentEncoding ? 'row-encoding' : ''} row-clickable"
            onclick={(e) => {
              if ((e.target as HTMLElement).closest('button')) return;
              openModal(file);
            }}
            title="Cliquer pour voir les infos / renommer"
          >
            <!-- Nom -->
            <td class="td-name">
              <span class="truncate font-mono block" style="color: var(--color-text);"
                    title={file.output_name + file.output_ext}>
                {file.output_name}{file.output_ext}
              </span>
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
            <td class="col-delete">
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
</div>

<style>
  .file-table {
    table-layout: fixed;
    width: 100%;
  }

  .col-name {
    /* Prend tout l'espace restant après les colonnes fixes */
    width: auto;
    min-width: 160px;
  }

  .td-name {
    max-width: 0; /* force le td à respecter la largeur du colgroup en table-fixed */
    overflow: hidden;
  }

  .row-encoding td {
    background: color-mix(in srgb, var(--color-accent) 5%, transparent);
  }

  .row-clickable {
    cursor: pointer;
  }
  .row-clickable:hover td {
    background: color-mix(in srgb, var(--color-accent) 6%, transparent);
  }

  /* Colonne suppression : toujours réservée, bouton visible au hover */
  .col-delete {
    width: 36px;
    text-align: center;
    padding: 0 4px;
  }

  /* Hauteur de ligne fixe */
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
    border-radius: var(--radius-lg);
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
    border-radius: var(--radius-xs);
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