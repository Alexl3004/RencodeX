<script lang="ts">
  import {
    encoder,
    type AppFile,
    formatDuration,
  } from "$lib/stores/encoder.svelte";
  import { formatSize } from "$lib/utils";
  import RenameModal from "$components/RenameModal.svelte";
  import BadgeInfoIcon from "@iconify-svelte/lucide/badge-info";
  import Edit2Icon from "@iconify-svelte/lucide/edit-2";
  import XIcon from "@iconify-svelte/lucide/x";

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

  // Déterminer si un fichier est en cours d'encodage
  function isCurrentlyEncoding(file: AppFile): boolean {
    return (
      file.status === "encoding" &&
      encoder.progress?.file_index ===
        encoder.files.findIndex((f) => f.path === file.path)
    );
  }

  // Déterminer si un fichier est en attente (dans la file)
  function isPending(file: AppFile): boolean {
    if (!encoder.encoding) return false;
    if (file.status !== "ready") return false;

    const currentIndex = encoder.progress?.file_index;
    if (currentIndex === undefined) return false;

    const fileIndex = encoder.files.findIndex((f) => f.path === file.path);
    return fileIndex > currentIndex;
  }

  // Nouveaux statuts simplifiés
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
    encoding: "text-[var(--color-accent)]",
    ready: "text-[var(--color-success)]",
    pending: "text-[var(--color-subtext)]",
    done: "text-[var(--color-success)]",
    error: "text-[var(--color-danger)]",
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

<div
  class="border border-[var(--color-border)] rounded-[2px] overflow-auto h-[30vh]"
>
  {#if encoder.files.length === 0}
    <div class="py-10 text-center flex items-center justify-center h-full">
      <p
        class="text-[11px] font-mono text-[var(--color-subtext)] uppercase tracking-widest"
      >
        Aucun fichier
      </p>
    </div>
  {:else}
    <table class="w-full text-[11px] file-table">
      <thead class="sticky top-0 bg-[var(--color-surface)] z-10">
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
        {#each encoder.files as file, index (file.path)}
          {@const isCurrentEncoding = isCurrentlyEncoding(file)}
          {@const isFilePending = isPending(file)}
          <tr
            class="group transition-colors {isCurrentEncoding
              ? 'bg-[var(--color-accent)]/5'
              : ''}"
          >
            <!-- Nom de sortie -->
            <td class="max-w-[400px]">
              <div class="flex items-center gap-1.5">
                <button
                  onclick={() => showFileDetails(file)}
                  class="shrink-0 text-[var(--color-subtext)] hover:text-[var(--color-accent)] transition-colors"
                  title="Infos"
                  aria-label="Informations fichier"
                >
                  <BadgeInfoIcon height="1em" />
                </button>
                <button
                  onclick={() => openRename(file)}
                  disabled={encoder.encoding}
                  class="shrink-0 text-[var(--color-subtext)] hover:text-[var(--color-accent)] transition-colors disabled:opacity-30"
                  title="Renommer"
                  aria-label="Renommer le fichier"
                >
                  <Edit2Icon height="1em" />
                </button>
                <span
                  class="truncate text-[var(--color-text)] font-mono flex-1 min-w-0"
                  title={file.output_name + file.output_ext}
                >
                  {file.output_name}{file.output_ext}
                </span>
              </div>
            </td>
            <!-- Taille -->
            <td class="text-right font-mono text-[var(--color-subtext)]">
              {file.size_mb > 0 ? formatSize(file.size_mb) : "—"}
            </td>
            <!-- Audio -->
            <td class="text-center font-mono text-[var(--color-subtext)]">
              {#if file.status === "analysing"}
                <span class="text-[var(--color-accent)] animate-pulse">…</span>
              {:else}
                {file.audio_langs.map((l) => l.toUpperCase()).join(" ") || "—"}
              {/if}
            </td>
            <!-- Subs -->
            <td class="text-center font-mono text-[var(--color-subtext)]">
              {file.sub_langs.map((l) => l.toUpperCase()).join(" ") || "—"}
            </td>
            <!-- Temps -->
            <td class="text-right font-mono text-[var(--color-subtext)]">
              {#if file.status === "done" && file.result?.duration_secs}
                {Math.floor(file.result.duration_secs / 60)}m{Math.floor(
                  file.result.duration_secs % 60,
                )}s
              {:else if (file.status === "encoding" || file.status === "ready") && file.duration_secs > 0}
                {Math.floor(file.duration_secs / 60)}m{Math.floor(
                  file.duration_secs % 60,
                )}s
              {:else}—{/if}
            </td>
            <!-- Statut -->
            <td class="text-center">
              {#if isCurrentEncoding && encoder.progress}
                <div class="flex flex-col items-center gap-0.5">
                  <span
                    class="font-mono text-[10px] text-[var(--color-accent)] font-bold animate-pulse"
                  >
                    {getStatusLabel(file)}
                  </span>
                  <span
                    class="font-mono text-[9px] text-[var(--color-subtext)]"
                  >
                    {Math.round(encoder.progress.percent)}%
                  </span>
                </div>
              {:else}
                <span class="font-mono text-[10px] {getStatusColor(file)}">
                  {getStatusLabel(file)}
                </span>
              {/if}
              {#if file.status === "done" && file.result && !isCurrentEncoding}
                <div class="text-[var(--color-success)] font-mono text-[10px]">
                  -{(
                    ((file.result.original_mb - file.result.encoded_mb) /
                      file.result.original_mb) *
                    100
                  ).toFixed(1)}%
                </div>
              {/if}
            </td>
            <!-- Suppr -->
            <td class="text-center">
              <button
                class="opacity-0 group-hover:opacity-100 transition-opacity font-mono text-[10px]
                             text-[var(--color-subtext)] hover:text-[var(--color-danger)] disabled:hidden"
                onclick={() => encoder.removeFile(file.path)}
                disabled={encoder.encoding}
                title="Supprimer"
                aria-label="Supprimer le fichier"
              >
                ✕
              </button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<!-- Modal info fichier -->
{#if showFileInfo && selectedFile}
  <div class="fixed inset-0 z-50 flex items-center justify-center">
    <button
      type="button"
      class="absolute inset-0 bg-black/60 cursor-default w-full h-full"
      onclick={closeFileInfo}
      tabindex="-1"
      aria-label="Fermer"
    ></button>

    <div
      class="relative w-[480px] max-w-[92vw] bg-[var(--color-panel)] border border-[var(--color-border)]
                rounded-[2px] shadow-[0_20px_60px_rgba(0,0,0,0.7)] flex flex-col overflow-hidden"
    >
      <!-- Header -->
      <div
        class="flex items-center justify-between px-5 py-3 border-b border-[var(--color-border)] shrink-0"
      >
        <div class="flex items-center gap-2">
          <div class="w-[3px] h-4 rounded-[1px] bg-[var(--color-accent)]"></div>
          <span class="text-[12px] font-semibold text-[var(--color-text)]"
            >Infos fichier</span
          >
        </div>
        <button
          onclick={closeFileInfo}
          class="text-[var(--color-subtext)] hover:text-[var(--color-text)] transition-colors p-1"
          aria-label="Fermer"
        >
          <XIcon height="1em" aria-hidden="true" />
        </button>
      </div>
      <!-- Body -->
      <div class="px-5 py-4 space-y-3 max-h-[70vh] overflow-y-auto">
        <div>
          <div class="section-label mb-1">Fichier source</div>
          <div
            class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[2px] px-3 py-2 font-mono text-[11px] break-all text-[var(--color-text)]"
          >
            {selectedFile.filename}
          </div>
        </div>

        <div>
          <div class="section-label mb-1">Fichier de sortie</div>
          <div
            class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[2px] px-3 py-2 font-mono text-[11px] break-all text-[var(--color-success)]"
          >
            {selectedFile.output_name}{selectedFile.output_ext}
          </div>
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div
            class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[2px] p-3"
          >
            <div class="section-label mb-1">Taille</div>
            <div class="font-mono text-[13px] text-[var(--color-text)]">
              {formatSize(selectedFile.size_mb)}
            </div>
          </div>
          <div
            class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[2px] p-3"
          >
            <div class="section-label mb-1">Durée</div>
            <div class="font-mono text-[13px] text-[var(--color-text)]">
              {formatDuration(selectedFile.duration_secs)}
            </div>
          </div>
          <div
            class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[2px] p-3"
          >
            <div class="section-label mb-1">FPS</div>
            <div class="font-mono text-[13px] text-[var(--color-text)]">
              {selectedFile.fps?.toFixed(2) || "—"} fps
            </div>
          </div>
          <div
            class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[2px] p-3"
          >
            <div class="section-label mb-1">Statut</div>
            <div class="font-mono text-[13px] text-[var(--color-text)]">
              {getStatusLabel(selectedFile)}
            </div>
          </div>
        </div>

        {#if selectedFile.audio_langs?.length > 0}
          <div>
            <div class="section-label mb-1.5">Pistes audio</div>
            <div class="flex flex-wrap gap-1">
              {#each selectedFile.audio_langs as lang}
                <span
                  class="font-mono text-[10px] border border-[var(--color-border)] px-2 py-0.5 rounded-[2px] bg-[var(--color-surface)] text-[var(--color-subtext)]"
                  >{lang.toUpperCase()}</span
                >
              {/each}
            </div>
          </div>
        {/if}

        <div>
          <div class="section-label mb-1.5">Sous-titres</div>
          <div class="flex flex-wrap gap-1">
            {#if selectedFile.sub_langs?.length > 0}
              {#each selectedFile.sub_langs as lang}
                <span
                  class="font-mono text-[10px] border border-[var(--color-border)] px-2 py-0.5 rounded-[2px] bg-[var(--color-surface)] text-[var(--color-subtext)]"
                  >{lang.toUpperCase()}</span
                >
              {/each}
            {:else}
              <span class="text-[11px] text-[var(--color-subtext)] italic"
                >Aucun</span
              >
            {/if}
          </div>
        </div>

        {#if selectedFile.streams?.length > 0}
          <div>
            <div class="section-label mb-1.5">Flux</div>
            <div class="space-y-0.5 max-h-32 overflow-y-auto">
              {#each selectedFile.streams as s}
                <div
                  class="font-mono text-[10px] py-1 border-b border-[var(--color-border)]/30 flex gap-2"
                >
                  <span class="text-[var(--color-accent)]"
                    >{s.codec_type?.toUpperCase()}</span
                  >
                  <span class="text-[var(--color-subtext)]">{s.codec_name}</span
                  >
                  {#if s.width && s.height}<span
                      class="text-[var(--color-subtext2)]"
                      >{s.width}×{s.height}</span
                    >{/if}
                  <span class="text-[var(--color-subtext2)]"
                    >[{s.language?.toUpperCase() || "und"}]</span
                  >
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
      <div
        class="flex justify-end px-5 py-3 border-t border-[var(--color-border)] shrink-0"
      >
        <button onclick={closeFileInfo} class="btn text-[11px] font-mono"
          >FERMER</button
        >
      </div>
    </div>
  </div>
{/if}
