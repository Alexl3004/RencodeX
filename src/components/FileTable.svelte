<script lang="ts">
  import {
    encoder,
    type AppFile,
    formatDuration,
  } from "$lib/stores/encoder.svelte";
  import { formatSize } from "$lib/utils";
  import FileModal from "$components/FileModal.svelte";
  import {
    X,
    Search,
    CircleCheck,
    AlertTriangle,
    LoaderCircle,
    Subtitles,
    Play,
    Trash2,
  } from "@lucide/svelte";

  let filterText = $state("");
  let filterStatus = $state<
    "all" | "ready" | "queued" | "encoding" | "done" | "error"
  >("all");
  let selectedFile = $state<AppFile | null>(null);

  type SortKey = "name" | "size" | "duration" | "status";
  type SortDir = "asc" | "desc";
  let sortKey = $state<SortKey | null>(null);
  let sortDir = $state<SortDir>("asc");

  function toggleSort(key: SortKey) {
    if (sortKey === key) {
      if (sortDir === "asc") sortDir = "desc";
      else { sortKey = null; sortDir = "asc"; }
    } else {
      sortKey = key;
      sortDir = "asc";
    }
  }

  const STATUS_ORDER: Record<string, number> = {
    encoding: 0, analysing: 1, queued: 2, ready: 3, done: 4, error: 5,
  };

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

  function isPending(file: AppFile): boolean {
    if (!encoder.encoding || file.status !== "ready") return false;
    const currentIndex = encoder.progress?.file_index;
    if (currentIndex === undefined) return false;
    const fileIndex = encoder.files.findIndex((f) => f.path === file.path);
    return fileIndex > currentIndex;
  }

  let statusCounts = $derived.by(() => {
    const counts = {
      all: encoder.files.length,
      ready: 0,
      queued: 0,
      encoding: 0,
      done: 0,
      error: 0,
    };
    for (const f of encoder.files) {
      if (f.status === "queued" || (f.status === "ready" && isPending(f)))
        counts.queued++;
      else if (f.status === "encoding" && !isPending(f)) counts.encoding++;
      else if (f.status === "ready") counts.ready++;
      else if (f.status === "done") counts.done++;
      else if (f.status === "error") counts.error++;
    }
    return counts;
  });

  let filteredFiles = $derived.by(() => {
    let result = encoder.files;
    if (filterStatus !== "all") {
      result = result.filter((f) => {
        if (filterStatus === "queued")
          return f.status === "queued" || isPending(f);
        if (filterStatus === "encoding")
          return f.status === "encoding" && !isPending(f);
        return f.status === filterStatus;
      });
    }
    if (filterText.trim()) {
      const q = filterText.trim().toLowerCase();
      result = result.filter(
        (f) =>
          f.output_name.toLowerCase().includes(q) ||
          f.filename.toLowerCase().includes(q),
      );
    }
    if (sortKey) {
      const dir = sortDir === "asc" ? 1 : -1;
      result = [...result].sort((a, b) => {
        if (sortKey === "name") {
          return dir * (a.output_name + a.output_ext).localeCompare(b.output_name + b.output_ext);
        }
        if (sortKey === "size") {
          return dir * (a.size_mb - b.size_mb);
        }
        if (sortKey === "duration") {
          const da = a.result?.duration_secs ?? a.duration_secs ?? 0;
          const db = b.result?.duration_secs ?? b.duration_secs ?? 0;
          return dir * (da - db);
        }
        if (sortKey === "status") {
          const sa = STATUS_ORDER[isPending(a) ? "queued" : a.status] ?? 99;
          const sb = STATUS_ORDER[isPending(b) ? "queued" : b.status] ?? 99;
          return dir * (sa - sb);
        }
        return 0;
      });
    }
    return result;
  });

  function getStatusLabel(file: AppFile): string {
    if (file.status === "analysing") return "Analyse";
    if (file.status === "encoding") return "En cours";
    if (file.status === "queued") return "En file";
    if (file.status === "ready") return isPending(file) ? "En file" : "Prêt";
    if (file.status === "done") return "Terminé";
    if (file.status === "error") return "Erreur";
    return "—";
  }

  const STATUS_COLOR: Record<string, string> = {
    analysing: "text-[var(--color-accent)] animate-pulse",
    encoding: "text-[var(--color-accent)]",
    queued: "text-[var(--color-subtext)]",
    ready: "text-[var(--color-success)]",
    pending: "text-[var(--color-subtext)]",
    done: "text-[var(--color-success)]",
    error: "text-[var(--color-danger)]",
  };

  function getStatusColor(file: AppFile): string {
    if (file.status === "queued") return STATUS_COLOR.queued;
    if (file.status === "ready" && isPending(file)) return STATUS_COLOR.pending;
    return STATUS_COLOR[file.status] ?? "";
  }

  function getSubExtractStatus(file: AppFile) {
    switch (file.sub_extract_status) {
      case "done":
        return {
          label: "Extraits",
          color: "text-[var(--color-success)]",
          icon: CircleCheck,
        };
      case "error":
        return {
          label: "Erreur extraction",
          color: "text-[var(--color-danger)]",
          icon: AlertTriangle,
        };
      default:
        return { label: "", color: "", icon: undefined };
    }
  }
</script>

{#if selectedFile}
  <FileModal file={selectedFile} onclose={closeModal} onrename={handleRename} />
{/if}

<div
  class="flex flex-col rounded-[var(--radius-md)] h-full overflow-hidden"
  style="border: 1px solid var(--color-border);"
>
  <!-- Barre de filtres -->
    <div
      class="flex items-center gap-1.5 px-2 py-1.5 shrink-0 flex-wrap"
      style="border-bottom: 1px solid var(--color-border); background: var(--color-surface);"
    >
      <!-- Vider le tableau -->
      <button
        onclick={() => encoder.clearAll()}
        title="Vider la liste"
        aria-label="Vider la liste des fichiers"
        class="clear-all-btn"
      >
        <X class="w-3 h-3" />
      </button>

      <!-- Supprimer la sélection -->
      <button
        onclick={() => {
          for (const path of encoder.selectedForEncoding) {
            encoder.removeFile(path);
          }
          encoder.clearEncodeSelection();
        }}
        title={encoder.selectedForEncoding.size > 0 ? `Supprimer les ${encoder.selectedForEncoding.size} fichiers sélectionnés` : "Aucun fichier sélectionné"}
        aria-label="Supprimer les fichiers sélectionnés"
        class="delete-sel-btn"
        disabled={encoder.selectedForEncoding.size === 0 || encoder.encoding || encoder.extractingSubs}
      >
        <Trash2 class="w-3 h-3" />
        <span class="delete-sel-count">{encoder.selectedForEncoding.size > 0 ? encoder.selectedForEncoding.size : ""}</span>
      </button>

      <!-- Recherche -->
      <div class="relative flex items-center">
        <Search
          class="absolute left-2 w-3 h-3 pointer-events-none"
          style="color: var(--color-subtext);"
        />
        <input
          type="text"
          bind:value={filterText}
          placeholder="Rechercher…"
          class="font-mono text-[10px] pl-6 pr-2 py-1 rounded-[var(--radius-sm)] w-36 outline-none"
          style="background: var(--color-panel); border: 1px solid var(--color-border); color: var(--color-text);"
        />
        {#if filterText}
          <button
            onclick={() => (filterText = "")}
            class="absolute right-1.5"
            style="color: var(--color-subtext);"
            aria-label="Effacer la recherche"
          >
            <X class="w-3 h-3" />
          </button>
        {/if}
      </div>

      <!-- Filtres par statut -->
      <!-- Badge fusionné Tous + Prêt -->
      <button
        onclick={() => (filterStatus = "all")}
        class="font-mono text-[9px] px-1.5 py-0.5 rounded-[var(--radius-sm)] flex items-center gap-1 transition-colors"
        style={filterStatus === "all"
          ? "background: var(--color-accent); color: #fff;"
          : "background: var(--color-panel); border: 1px solid var(--color-border); color: var(--color-subtext);"}
        aria-pressed={filterStatus === "all"}
      >
        Prêt <span class="opacity-75"
          >{statusCounts.ready}/{statusCounts.all}</span
        >
      </button>

      {#each [{ key: "queued", label: "En file", count: statusCounts.queued }, { key: "encoding", label: "En cours", count: statusCounts.encoding }, { key: "done", label: "Terminé", count: statusCounts.done }, { key: "error", label: "Erreur", count: statusCounts.error }] as btn}
        {#if btn.count > 0}
          <button
            onclick={() => (filterStatus = btn.key as typeof filterStatus)}
            class="font-mono text-[9px] px-1.5 py-0.5 rounded-[var(--radius-sm)] flex items-center gap-1 transition-colors"
            style={filterStatus === btn.key
              ? "background: var(--color-accent); color: #fff;"
              : "background: var(--color-panel); border: 1px solid var(--color-border); color: var(--color-subtext);"}
            aria-pressed={filterStatus === btn.key}
          >
            {btn.label}
            <span class="opacity-75">{btn.count}</span>
          </button>
        {/if}
      {/each}

      {#if filterText || filterStatus !== "all"}
        <span class="font-mono text-[9px]" style="color: var(--color-subtext);">
          {filteredFiles.length}/{encoder.files.length}
        </span>
      {/if}

      <!-- Spacer -->
      <div class="flex-1"></div>

      <!-- ── Groupe sélection unifié (encodage + extraction) ────────────── -->
      {#if encoder.files.some((f) => f.status === "ready")}
        {@const selMode =
          encoder.encodeSelectionMode || encoder.extractSelectionMode}
        {@const selEncCount = encoder.selectedForEncoding.size}
        {@const selExtCount = encoder.selectedForExtraction.size}
        {@const hasExtract =
          encoder.showExtractButton &&
          encoder.files.some((f) => f.sub_langs.length > 0)}

        <div class="sel-group">
          <label
            class="sel-toggle"
            title="Mode sélection : cliquer sur les lignes pour les sélectionner"
          >
            <span class="sel-toggle-label">
              <span class="sel-count">
                {#if selMode && selEncCount > 0}
                  ({selEncCount})
                {/if}
              </span>
              Sélection
            </span>
            <button
              type="button"
              role="switch"
              aria-checked={selMode}
              aria-label={selMode
                ? "Désactiver le mode sélection"
                : "Activer le mode sélection"}
              onclick={() => {
                const next = !selMode;
                encoder.setEncodeSelectionMode(next);
                if (hasExtract) encoder.setExtractSelectionMode(next);
              }}
              class="toggle {selMode ? 'on' : ''}"
            ></button>
          </label>
          <button
            type="button"
            onclick={() => {
              encoder.clearEncodeSelection();
              if (hasExtract) encoder.clearExtractSelection();
            }}
            disabled={!selMode || (selEncCount === 0 && selExtCount === 0)}
            class="micro-btn">Aucun</button
          >
          <button
            type="button"
            onclick={() => {
              encoder.setEncodeSelection(
                encoder.files
                  .filter((f) => f.status === "ready")
                  .map((f) => f.path),
              );
              if (hasExtract)
                encoder.setExtractSelection(
                  encoder.files
                    .filter(
                      (f) => f.status === "ready" && f.sub_langs.length > 0,
                    )
                    .map((f) => f.path),
                );
            }}
            disabled={!selMode}
            class="micro-btn">Tout</button
          >
        </div>
      {/if}
    </div>

  <!-- Table -->
  <div class="flex-1 overflow-auto">
    <table class="w-full text-[11px] file-table">
      <colgroup>
        <col class="col-name" />
        <col style="width: 76px;" />
        <col style="width: 72px;" />
        <col style="width: 90px;" />
        <col style="width: 72px;" />
        <col style="width: 110px;" />
        <col style="width: 36px;" />
      </colgroup>
      <thead
        class="sticky top-0 z-10"
        style="background: var(--color-surface);"
      >
        <tr>
          <th class="text-left">
            <button class="th-sort-btn" onclick={() => toggleSort("name")} aria-label="Trier par nom">
              Fichier de sortie
              <span class="sort-icon" aria-hidden="true">
                {#if sortKey === "name"}{sortDir === "asc" ? "↑" : "↓"}{:else}↕{/if}
              </span>
            </button>
          </th>
          <th class="text-right">
            <button class="th-sort-btn th-sort-right" onclick={() => toggleSort("size")} aria-label="Trier par taille">
              Taille
              <span class="sort-icon" aria-hidden="true">
                {#if sortKey === "size"}{sortDir === "asc" ? "↑" : "↓"}{:else}↕{/if}
              </span>
            </button>
          </th>
          <th class="text-center">Audio</th>
          <th class="text-center">Sous-titres</th>
          <th class="text-right">
            <button class="th-sort-btn th-sort-right" onclick={() => toggleSort("duration")} aria-label="Trier par durée">
              Temps
              <span class="sort-icon" aria-hidden="true">
                {#if sortKey === "duration"}{sortDir === "asc" ? "↑" : "↓"}{:else}↕{/if}
              </span>
            </button>
          </th>
          <th class="text-center">
            <button class="th-sort-btn th-sort-center" onclick={() => toggleSort("status")} aria-label="Trier par statut">
              Statut
              <span class="sort-icon" aria-hidden="true">
                {#if sortKey === "status"}{sortDir === "asc" ? "↑" : "↓"}{:else}↕{/if}
              </span>
            </button>
          </th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {#if encoder.files.length === 0}
          <tr>
            <td colspan="7" class="py-10 text-center">
              <p class="text-[11px] font-mono uppercase tracking-widest" style="color: var(--color-subtext);">
                Aucun fichier
              </p>
            </td>
          </tr>
        {:else if filteredFiles.length === 0}
          <tr>
            <td colspan="7" class="py-10 text-center">
              <p
                class="text-[11px] font-mono uppercase tracking-widest"
                style="color: var(--color-subtext);"
              >
                Aucun résultat
              </p>
            </td>
          </tr>
        {:else}
          {#each filteredFiles as file (file.path)}
            {@const isCurrentEncoding = isCurrentlyEncoding(file)}
            {@const subStatus = getSubExtractStatus(file)}

            {@const isEncodeEligible = file.status === "ready"}
            {@const isEncodeSelected = encoder.selectedForEncoding.has(
              file.path,
            )}
            {@const inEncodeMode = encoder.encodeSelectionMode}

            {@const isExtractEligible =
              file.status === "ready" && file.sub_langs.length > 0}
            {@const isExtractSelected = encoder.selectedForExtraction.has(
              file.path,
            )}
            {@const inExtractMode = encoder.extractSelectionMode}

            <!-- En mode sélection unifié les deux modes sont actifs en même temps -->
            {@const inSelMode = inEncodeMode || inExtractMode}
            {@const isSelected = isEncodeSelected || isExtractSelected}
            {@const isEligible = isEncodeEligible}

            <tr
              class="group transition-colors
                {isCurrentEncoding ? 'row-encoding' : ''}
                {isSelected ? 'row-selected' : ''}
                {inSelMode && isEligible ? 'row-sel-mode' : ''}
                {!inSelMode ? 'row-clickable' : ''}
                {inSelMode && !isEligible ? 'row-sel-disabled' : ''}"
              onclick={(e) => {
                if ((e.target as HTMLElement).closest("button")) return;
                if (inSelMode && isEncodeEligible) {
                  encoder.toggleEncodeSelection(file.path);
                  if (inExtractMode && isExtractEligible) {
                    encoder.toggleExtractSelection(file.path);
                  }
                } else if (!inSelMode) {
                  openModal(file);
                }
              }}
              title={inSelMode
                ? isEncodeEligible
                  ? isEncodeSelected
                    ? "Désélectionner"
                    : "Sélectionner"
                  : "Fichier non prêt"
                : "Cliquer pour voir les infos / renommer"}
            >
              <!-- Indicateur de sélection et couleur de fond selon le mode -->
              <td class="td-name">
                <span
                  class="truncate font-mono block"
                  style="color: var(--color-text);"
                  title={file.output_name + file.output_ext}
                >
                  {#if inSelMode && isEncodeSelected}
                    <span class="sel-dot encode-dot" aria-hidden="true"></span>
                  {:else if inSelMode && isExtractSelected}
                    <span class="sel-dot extract-dot" aria-hidden="true"></span>
                  {/if}
                  {file.output_name}{file.output_ext}
                </span>
              </td>

              <td
                class="text-right font-mono"
                style="color: var(--color-subtext);"
              >
                {file.size_mb > 0 ? formatSize(file.size_mb) : "—"}
              </td>
              <td
                class="text-center font-mono"
                style="color: var(--color-subtext);"
              >
                {#if file.status === "analysing"}
                  <span
                    class="animate-pulse"
                    style="color: var(--color-accent);">…</span
                  >
                {:else}
                  <span
                    class="inline-block max-w-[88px] truncate align-middle"
                    title={file.audio_langs
                      .map((l) => l.toUpperCase())
                      .join(" ") || "—"}
                  >
                    {file.audio_langs.map((l) => l.toUpperCase()).join(" ") ||
                      "—"}
                  </span>
                {/if}
              </td>
              <td
                class="text-center font-mono"
                style="color: var(--color-subtext);"
              >
                <span
                  class="inline-block max-w-[88px] truncate align-middle"
                  title={file.sub_langs.map((l) => l.toUpperCase()).join(" ") ||
                    "—"}
                >
                  {file.sub_langs.map((l) => l.toUpperCase()).join(" ") || "—"}
                </span>
              </td>
              <td
                class="text-right font-mono"
                style="color: var(--color-subtext);"
              >
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
              <td class="text-center">
                {#if isCurrentEncoding && encoder.progress}
                  <div class="flex flex-col items-center gap-0.5">
                    <span
                      class="font-mono text-[10px] font-bold animate-pulse"
                      style="color: var(--color-accent);"
                      >{getStatusLabel(file)}</span
                    >
                    <span
                      class="font-mono text-[9px]"
                      style="color: var(--color-subtext);"
                      >{Math.round(encoder.progress.percent)}%</span
                    >
                  </div>
                {:else if file.sub_extract_status === "extracting"}
                  <div class="flex flex-col items-center gap-0.5">
                    <span
                      class="font-mono text-[10px] animate-pulse inline-flex items-center gap-0.5"
                      style="color: var(--color-accent);"
                    >
                      <LoaderCircle class="w-2.5 h-2.5 animate-spin" />
                      Extraction…
                    </span>
                  </div>
                {:else if file.status === "done" && file.result}
                  <div class="flex flex-col items-center gap-0.5">
                    <span class="font-mono text-[10px] {getStatusColor(file)}"
                      >{getStatusLabel(file)}</span
                    >
                    <span
                      class="font-mono text-[10px]"
                      style="color: var(--color-success);"
                    >
                      -{(
                        ((file.result.original_mb - file.result.encoded_mb) /
                          file.result.original_mb) *
                        100
                      ).toFixed(1)}%
                    </span>
                  </div>
                {:else}
                  <span class="font-mono text-[10px] {getStatusColor(file)}"
                    >{getStatusLabel(file)}</span
                  >
                {/if}

                {#if subStatus.label}
                  <div class="flex flex-col items-center gap-0.5 mt-0.5">
                    <span
                      class="font-mono text-[9px] {subStatus.color} inline-flex items-center gap-0.5"
                    >
                      {#if file.sub_extract_status === "done"}
                        <CircleCheck class="w-2.5 h-2.5" />
                      {:else if file.sub_extract_status === "error"}
                        <AlertTriangle class="w-2.5 h-2.5" />
                      {/if}
                      {subStatus.label}
                    </span>
                  </div>
                {/if}
              </td>

              <td class="col-delete">
                <button
                  class="row-delete-btn opacity-0 group-hover:opacity-100 transition-opacity font-mono text-[10px] disabled:hidden"
                  onclick={() => encoder.removeFile(file.path)}
                  disabled={encoder.encoding || encoder.extractingSubs}
                  title="Supprimer"
                  aria-label="Supprimer le fichier"
                >
                  <X class="w-3.5 h-3.5" />
                </button>
              </td>
            </tr>
          {/each}
        {/if}
        </tbody>
      </table>
  </div>
</div>

<style>
  .clear-all-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 4px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-panel);
    color: var(--color-subtext);
    cursor: pointer;
    transition:
      background 0.1s,
      color 0.1s,
      border-color 0.1s;
    flex-shrink: 0;
  }
  .clear-all-btn:hover {
    background: color-mix(in srgb, var(--color-danger) 12%, var(--color-panel));
    border-color: var(--color-danger);
    color: var(--color-danger);
  }

  .delete-sel-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 7px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-panel);
    color: var(--color-subtext);
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    font-weight: 600;
    cursor: pointer;
    flex-shrink: 0;
    transition: background 0.1s, color 0.1s, border-color 0.1s, opacity 0.1s;
  }
  .delete-sel-btn:not(:disabled) {
    border-color: var(--color-danger);
    background: color-mix(in srgb, var(--color-danger) 10%, var(--color-panel));
    color: var(--color-danger);
  }
  .delete-sel-btn:hover:not(:disabled) {
    background: color-mix(in srgb, var(--color-danger) 22%, var(--color-panel));
  }
  .delete-sel-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .delete-sel-count {
    display: inline-block;
    min-width: 12px;
    text-align: left;
  }

  .file-table {
    table-layout: fixed;
    width: 100%;
  }
  .col-name {
    width: auto;
    min-width: 160px;
  }
  .td-name {
    max-width: 0;
    overflow: hidden;
    position: relative;
  }

  /* Fond selon le mode sélection actif */
  .row-encoding td {
    background: color-mix(in srgb, var(--color-accent) 5%, transparent);
  }
  /* Sélectionné encodage → teinte verte/primaire */
  .row-selected td {
    background: color-mix(in srgb, var(--color-accent) 7%, transparent);
  }

  .row-clickable {
    cursor: pointer;
  }
  .row-clickable:hover td {
    background: color-mix(in srgb, var(--color-accent) 6%, transparent);
  }
  .row-sel-mode {
    cursor: pointer;
  }
  .row-sel-mode:hover td {
    background: color-mix(in srgb, var(--color-accent) 10%, transparent);
  }
  .row-sel-disabled {
    cursor: default;
    opacity: 0.45;
  }

  /* Point coloré indiquant le type de sélection */
  .sel-dot {
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    margin-right: 5px;
    vertical-align: middle;
    flex-shrink: 0;
  }
  .encode-dot {
    background: var(--color-accent);
  }
  .extract-dot {
    background: var(--color-success);
  }

  .col-delete {
    width: 36px;
    text-align: center;
    padding: 0 4px;
  }
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

  /* Groupes de sélection dans la barre de filtres */
  .sel-group {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .sel-sep {
    width: 1px;
    height: 16px;
    background: var(--color-border);
    flex-shrink: 0;
    margin: 0 2px;
  }
  .sel-toggle {
    display: flex;
    align-items: center;
    gap: 5px;
    cursor: pointer;
    user-select: none;
  }
  .sel-toggle-label {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext);
    white-space: nowrap;
  }
  .sel-count {
    display: inline-block;
    min-width: 26px;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 700;
    color: var(--color-accent);
  }
  .micro-btn {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    padding: 2px 7px;
    border-radius: var(--radius-xs, 4px);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-subtext);
    cursor: pointer;
    transition:
      background 0.1s,
      color 0.1s;
    width: 38px;
    text-align: center;
  }
  .micro-btn:hover:not(:disabled) {
    background: var(--color-border);
    color: var(--color-text);
  }
  .micro-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  /* Sortable column headers */
  .th-sort-btn {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    background: transparent;
    border: none;
    cursor: pointer;
    font-family: "Geist Mono", monospace;
    font-size: inherit;
    font-weight: inherit;
    color: var(--color-subtext);
    padding: 0;
    white-space: nowrap;
    transition: color 0.1s;
  }
  .th-sort-btn:hover {
    color: var(--color-text);
  }
  .th-sort-right {
    margin-left: auto;
  }
  .th-sort-center {
    margin: 0 auto;
  }
  .sort-icon {
    font-size: 9px;
    opacity: 0.45;
    line-height: 1;
  }
  thead th {
    padding: 4px 6px;
  }
</style>