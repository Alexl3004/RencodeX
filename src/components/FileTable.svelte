<script lang="ts">
  import { encoder, type AppFile} from "$lib/stores/encoder.svelte";
  import { formatDuration } from "$lib/stores/naming";
  import { formatSize } from "$lib/utils";
  import FileModal from "$components/FileModal.svelte";
  import FileRenameModal from "$components/FileRenameModal.svelte";
  import FileLangModal from "$components/FileLangModal.svelte";
  import LangPopover from "$components/LangPopover.svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import {
    X,
    Search,
    CircleCheck,
    AlertTriangle,
    LoaderCircle,
    Trash2,
    Captions,
    Tag,
    Pencil,
    ChevronRight,
    Headphones,
    MessageSquare,
    RotateCcw,
    FileDown,
    Upload,
  } from "@lucide/svelte";

  let filterText = $state("");
  let filterStatus = $state<
    "all" | "ready" | "queued" | "encoding" | "done" | "error"
  >("all");

  // ── Drag & drop natif Tauri ────────────────────────────────────────────────
  let isDragOver = $state(false);
  let dragEnterCount = 0; // compteur pour ignorer les events des enfants
  let tableContainer = $state<HTMLDivElement | null>(null);

  $effect(() => {
    const unlisteners: UnlistenFn[] = [];

    async function setup() {

      const eventNames = [
        "tauri://drag-drop",   // Tauri v2 probable
        "tauri://file-drop",   // Tauri v1 / compat
        "tauri://drop",        // Alias possible
      ];

      for (const name of eventNames) {
        try {
          const u = await listen<{ paths: string[]; position: { x: number; y: number } }>(
            name,
            (event) => {

              const paths: string[] =
                Array.isArray(event.payload)
                  ? (event.payload as unknown as string[])  // v1 payload = string[]
                  : event.payload?.paths ?? [];

              // isDragOver est déjà false ici (ondragleave DOM fire avant Tauri)
              // → on check uniquement encoding et paths
              if (!encoder.encoding && paths.length > 0) {
                encoder.addFiles(paths);
              }

              isDragOver = false;
              dragEnterCount = 0;
            }
          );
          unlisteners.push(u);
        } catch (err) {
        }
      }

    }

    setup().catch(() => {});

    return () => {
      unlisteners.forEach((u) => u());
    };
  });
  // Sélection modale renommage — dérivée de encoder.files pour rester à jour
  // Modal info
  let infoPath = $state<string | null>(null);
  let infoPosition = $state<{ x: number; y: number } | null>(null);
  let infoFile = $derived(
    infoPath ? (encoder.files.find((f) => f.path === infoPath) ?? null) : null,
  );

  // Modal renommage
  let renamePath = $state<string | null>(null);
  let renameFile = $derived(
    renamePath ? (encoder.files.find((f) => f.path === renamePath) ?? null) : null,
  );
  let renamePosition = $state<{ x: number; y: number } | null>(null);

  // Modal pistes par fichier
  let langModalFile = $state<AppFile | null>(null);

  // Context menu clic droit
  let ctxMenu = $state<{ x: number; y: number; file: AppFile } | null>(null);
  let ctxSubmenu = $state<"lang" | null>(null);

  // Sélection rapide de langue dans le context menu (sans modal)
  let ctxAudio = $state<Set<string>>(new Set());
  let ctxSubs  = $state<Set<string>>(new Set());

  function openCtx(e: MouseEvent, file: AppFile) {
    e.preventDefault();
    const x = Math.min(e.clientX, window.innerWidth  - 220);
    const y = Math.min(e.clientY, window.innerHeight - 320);
    ctxMenu    = { x, y, file };
    ctxSubmenu = null;
    // Pré-remplir avec la sélection actuelle du fichier
    ctxAudio = encoder.fileSelAudio.has(file.path)
      ? new Set(encoder.fileSelAudio.get(file.path))
      : new Set([...encoder.selAudio].filter(l => file.audio_langs.includes(l)));
    ctxSubs = encoder.fileSelSubs.has(file.path)
      ? new Set(encoder.fileSelSubs.get(file.path))
      : new Set([...encoder.selSubs].filter(l => file.sub_langs.includes(l)));
  }

  function closeCtx() { ctxMenu = null; ctxSubmenu = null; }

  function ctxApplyLang() {
    if (!ctxMenu) return;
    encoder.setFileLangSel(ctxMenu.file.path, ctxAudio, ctxSubs);
    closeCtx();
  }

  function ctxToggleAudio(lang: string) {
    const s = new Set(ctxAudio);
    s.has(lang) ? s.delete(lang) : s.add(lang);
    ctxAudio = s;
  }
  function ctxToggleSub(lang: string) {
    const s = new Set(ctxSubs);
    s.has(lang) ? s.delete(lang) : s.add(lang);
    ctxSubs = s;
  }

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

  function openInfoModal(f: AppFile) { 
    // On utilise désormais la position du menu contextuel, donc on ne l'appelle plus directement.
    // Cette fonction est conservée pour compatibilité mais on utilisera l'ouverture depuis le menu.
  }
  function openRenameModal(f: AppFile) { renamePath = f.path; }
  function handleRename(n: string) {
    if (renamePath) encoder.renameFile(renamePath, n);
    renamePath = null;
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
  // Extraction depuis le context menu — délègue au store (ProgressPanel + toasts)
  function ctxExtractSubs(file: AppFile) {
    if (!file.sub_langs.length || encoder.extractingSubs) return;
    closeCtx();
    // Force la sélection sur ce fichier uniquement puis lance
    encoder.setExtractSelection([file.path]);
    encoder.startSubtitleExtraction();
  }
</script>

{#if infoFile && infoPosition}
  <FileModal
    file={infoFile}
    position={infoPosition}
    onclose={() => { infoPath = null; infoPosition = null; }}
  />
{/if}

{#if renameFile && renamePosition}
  <FileRenameModal
    file={renameFile}
    position={renamePosition}
    onclose={() => { renamePath = null; renamePosition = null; }}
    onrename={handleRename}
  />
{/if}

{#if langModalFile}
  <FileLangModal file={langModalFile} onclose={() => (langModalFile = null)} />
{/if}

{#if ctxMenu}
  <!-- Backdrop invisible pour fermer -->
  <div
    class="fixed inset-0 z-[9960]"
    role="presentation"
    onclick={closeCtx}
    oncontextmenu={(e) => { e.preventDefault(); closeCtx(); }}
  ></div>

  <!-- Context menu -->
  <div
    class="ctx-menu fixed z-[9961] flex flex-col overflow-hidden"
    style="left:{ctxMenu.x}px; top:{ctxMenu.y}px;"
    role="menu"
  >
    <!-- Nom du fichier -->
    <div class="ctx-filename">
      <span class="truncate">{ctxMenu.file.filename}</span>
      {#if encoder.fileSelAudio.has(ctxMenu.file.path) || encoder.fileSelSubs.has(ctxMenu.file.path)}
        <span class="ctx-badge">override</span>
      {/if}
    </div>

    <div class="ctx-sep"></div>

    <!-- Infos -->
    <button
      type="button"
      class="ctx-item"
      role="menuitem"
      onclick={() => {
        infoPath = ctxMenu!.file.path;
        infoPosition = { x: ctxMenu!.x, y: ctxMenu!.y + 8 };
        closeCtx();
      }}
    >
      <Tag class="w-3.5 h-3.5 shrink-0" />
      Infos
    </button>

    <!-- Renommer -->
    {#if !encoder.encoding && ctxMenu.file.status !== "analysing"}
    <button
      type="button"
      class="ctx-item"
      role="menuitem"
      onclick={() => {
        renamePath = ctxMenu!.file.path;
        renamePosition = { x: ctxMenu!.x, y: ctxMenu!.y + 8 };
        closeCtx();
      }}
    >
      <Pencil class="w-3.5 h-3.5 shrink-0" />
      Renommer
    </button>
    {/if}

    <div class="ctx-sep"></div>

    <!-- Changer langue — header cliquable -->
    <button
      type="button"
      class="ctx-item ctx-item--section"
      role="menuitem"
      aria-expanded={ctxSubmenu === "lang"}
      onclick={() => (ctxSubmenu = ctxSubmenu === "lang" ? null : "lang")}
    >
      <Captions class="w-3.5 h-3.5 shrink-0" />
      Changer les pistes
      <ChevronRight class="w-3 h-3 shrink-0 ml-auto transition-transform {ctxSubmenu === 'lang' ? 'rotate-90' : ''}" />
    </button>

    <!-- Sous-menu langues inline -->
    {#if ctxSubmenu === "lang"}
      <div class="ctx-lang-panel">

        <!-- Audio -->
        {#if ctxMenu.file.audio_langs.length > 0}
          <div class="ctx-lang-group">
            <span class="ctx-lang-label">
              <Headphones class="w-3 h-3 opacity-60" />
              Audio
            </span>
            <div class="ctx-lang-chips">
              {#each ctxMenu.file.audio_langs as lang}
                {@const on = ctxAudio.has(lang)}
                <button
                  type="button"
                  class="ctx-chip {on ? 'ctx-chip--audio-on' : ''}"
                  onclick={() => ctxToggleAudio(lang)}
                  aria-pressed={on}
                  title={lang}
                >
                  {#if on}<CircleCheck class="w-2.5 h-2.5 shrink-0" />{/if}
                  {lang.toUpperCase()}
                </button>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Sous-titres -->
        {#if ctxMenu.file.sub_langs.length > 0}
          <div class="ctx-lang-group">
            <span class="ctx-lang-label">
              <MessageSquare class="w-3 h-3 opacity-60" />
              Sous-titres
            </span>
            <div class="ctx-lang-chips">
              {#each ctxMenu.file.sub_langs as lang}
                {@const on = ctxSubs.has(lang)}
                <button
                  type="button"
                  class="ctx-chip {on ? 'ctx-chip--sub-on' : ''}"
                  onclick={() => ctxToggleSub(lang)}
                  aria-pressed={on}
                  title={lang}
                >
                  {#if on}<CircleCheck class="w-2.5 h-2.5 shrink-0" />{/if}
                  {lang.toUpperCase()}
                </button>
              {/each}
            </div>
          </div>
        {/if}

        {#if ctxMenu.file.audio_langs.length === 0 && ctxMenu.file.sub_langs.length === 0}
          <p class="ctx-lang-empty">Aucune piste détectée</p>
        {/if}

        <!-- Actions -->
        <div class="ctx-lang-actions">
          {#if encoder.fileSelAudio.has(ctxMenu.file.path) || encoder.fileSelSubs.has(ctxMenu.file.path)}
            <button type="button" class="ctx-action-btn ctx-action-btn--reset"
              onclick={() => { encoder.clearFileLangSel(ctxMenu!.file.path); closeCtx(); }}>
              <RotateCcw class="w-3 h-3" />
              Reset
            </button>
          {/if}
          <button type="button" class="ctx-action-btn ctx-action-btn--apply ml-auto"
            onclick={ctxApplyLang}>
            <CircleCheck class="w-3 h-3" />
            Appliquer
          </button>
        </div>
      </div>
    {/if}

    <!-- Extraire les sous-titres -->
    {#if ctxMenu.file.sub_langs.length > 0 && !encoder.encoding}
      <div class="ctx-sep"></div>
      <button
        type="button"
        class="ctx-item"
        role="menuitem"
        disabled={encoder.extractingSubs}
        onclick={() => ctxExtractSubs(ctxMenu!.file)}
      >
        {#if encoder.extractingSubs}
          <LoaderCircle class="w-3.5 h-3.5 shrink-0 animate-spin" />
        {:else}
          <FileDown class="w-3.5 h-3.5 shrink-0" />
        {/if}
        Extraire les sous-titres
        <span class="ctx-extract-fmt">{encoder.subExtractFormat.toUpperCase()}</span>
      </button>
    {/if}

    <div class="ctx-sep"></div>

    <!-- Supprimer -->
    <button
      type="button"
      class="ctx-item ctx-item--danger"
      role="menuitem"
      disabled={encoder.encoding || encoder.extractingSubs}
      onclick={() => { encoder.removeFile(ctxMenu!.file.path); closeCtx(); }}
    >
      <Trash2 class="w-3.5 h-3.5 shrink-0" />
      Supprimer
    </button>
  </div>
{/if}

<div
  bind:this={tableContainer}
  role="region"
  aria-label="Liste des fichiers — zone de dépôt"
  class="flex flex-col rounded-[var(--radius-md)] h-full overflow-hidden relative"
  style="border: 1px solid {isDragOver ? 'var(--color-accent)' : 'var(--color-border)'}; transition: border-color 0.15s;"
  ondragenter={(e) => {
    e.preventDefault();
    dragEnterCount++;
    if (!encoder.encoding) { isDragOver = true; }
  }}
  ondragover={(e) => { e.preventDefault(); }}
  ondragleave={() => {
    dragEnterCount = Math.max(0, dragEnterCount - 1);
    if (dragEnterCount === 0) { isDragOver = false; }
  }}
  ondrop={(e) => {
    e.preventDefault();
    // ⚠️ Ne pas remettre isDragOver = false ici : Tauri lit isDragOver juste après
  }}
>
  <!-- Overlay drop ─────────────────────────────────────────── -->
  {#if isDragOver}
    <div class="drop-overlay" aria-hidden="true">
      <div class="drop-overlay-inner">
        <Upload class="drop-icon" />
        <span class="drop-label">Déposer les fichiers vidéo</span>
        <span class="drop-hint">mp4 · mkv · avi · mov · flv</span>
      </div>
    </div>
  {/if}
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

        <!-- Sélection globale des pistes -->
        <LangPopover />
      {/if}
    </div>

  <!-- Table -->
  <div class="flex-1 overflow-auto">
    <table class="w-full text-[11px] file-table">
      <colgroup>
        <col style="width: 42%;" />   <!-- Fichier de sortie -->
        <col style="width: 10%;" />   <!-- Taille -->
        <col style="width: 12%;" />   <!-- Audio -->
        <col style="width: 12%;" />   <!-- Sous-titres -->
        <col style="width: 12%;" />   <!-- Temps -->
        <col style="width: 12%;" />   <!-- Statut -->
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
        </tr>
      </thead>
      <tbody>
        {#if encoder.files.length === 0}
          <tr>
            <td colspan="7" class="py-10 text-center">
              <div class="empty-state">
                <Upload class="empty-icon" />
                <p class="empty-title">Aucun fichier</p>
                <p class="empty-hint">Glissez des fichiers vidéo ici</p>
              </div>
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
              oncontextmenu={(e) => openCtx(e, file)}
              onclick={(e) => {
                if ((e.target as HTMLElement).closest("button")) return;
                if (inSelMode && isEncodeEligible) {
                  encoder.toggleEncodeSelection(file.path);
                  if (inExtractMode && isExtractEligible) {
                    encoder.toggleExtractSelection(file.path);
                  }
                }
              }}
              title={inSelMode
                ? isEncodeEligible
                  ? isEncodeSelected
                    ? "Désélectionner"
                    : "Sélectionner"
                  : "Fichier non prêt"
                : ""}
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
                {#if encoder.fileSelAudio.has(file.path) || encoder.fileSelSubs.has(file.path)}
                  <span class="override-dot" title="Pistes personnalisées"></span>
                {/if}
                </span>
              </td>

              <td
                class="text-right font-mono"
                style="color: var(--color-subtext);"
              >
                {file.size_mb > 0 ? formatSize(file.size_mb) : "—"}
              </td>
              <td class="text-center font-mono" style="color: var(--color-subtext);">
                {#if file.status === "analysing"}
                  <span class="animate-pulse" style="color: var(--color-accent);">…</span>
                {:else}
                  {@const fileAudioSel = encoder.fileSelAudio.get(file.path) ?? encoder.selAudio}
                  {@const hasAudioOverride = encoder.fileSelAudio.has(file.path)}
                  <span class="cell-lang-grid" title={hasAudioOverride ? "Pistes personnalisées" : ""}>
                    {#each file.audio_langs as lang}
                      <span class="cell-lang-tag {fileAudioSel.has(lang) ? 'cell-lang-tag--on' : 'cell-lang-tag--off'}">{lang.toUpperCase()}</span>
                    {/each}
                    {#if file.audio_langs.length === 0}<span>—</span>{/if}
                    {#if hasAudioOverride}<span class="cell-override-dot" title="Override actif">●</span>{/if}
                  </span>
                {/if}
              </td>
              <td class="text-center font-mono" style="color: var(--color-subtext);">
                {#if true}
                  {@const fileSubSel = encoder.fileSelSubs.get(file.path) ?? encoder.selSubs}
                  {@const hasSubOverride = encoder.fileSelSubs.has(file.path)}
                  <span class="cell-lang-grid" title={hasSubOverride ? "Pistes personnalisées" : ""}>
                    {#each file.sub_langs as lang}
                      <span class="cell-lang-tag {fileSubSel.has(lang) ? 'cell-lang-tag--sub' : 'cell-lang-tag--off'}">{lang.toUpperCase()}</span>
                    {/each}
                    {#if file.sub_langs.length === 0}<span>—</span>{/if}
                    {#if hasSubOverride}<span class="cell-override-dot" title="Override actif">●</span>{/if}
                  </span>
                {/if}
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
            </tr>
          {/each}
        {/if}
        </tbody>
      </table>
  </div>
</div>

<style>
  /* ── Context menu ─────────────────────────────────────────────────────── */
  .ctx-menu {
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    box-shadow: 0 12px 40px rgba(0,0,0,0.5);
    min-width: 200px;
    max-width: 260px;
    overflow: hidden;
  }
  .ctx-filename {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px 6px;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext);
    overflow: hidden;
  }
  .ctx-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 12px;
    font-family: "DM Sans", sans-serif;
    font-size: 12px;
    background: transparent;
    border: none;
    color: var(--color-text);
    cursor: pointer;
    text-align: left;
    width: 100%;
    transition: background 0.08s, color 0.08s;
  }
  .ctx-item:hover {
    background: var(--color-panel2);
    color: var(--color-accent);
  }
  .ctx-item--section {
    font-weight: 500;
  }
  .ctx-item--danger {
    color: var(--color-danger);
  }
  .ctx-item--danger:hover {
    background: color-mix(in srgb, var(--color-danger) 10%, var(--color-panel2));
    color: var(--color-danger);
  }
  .ctx-item--danger:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }
  .ctx-sep {
    height: 1px;
    background: var(--color-border);
    margin: 2px 0;
    opacity: 0.6;
  }
  .ctx-badge {
    margin-left: auto;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    padding: 1px 5px;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--color-accent) 12%, transparent);
    color: var(--color-accent);
    border: 1px solid color-mix(in srgb, var(--color-accent) 25%, var(--color-border));
    flex-shrink: 0;
  }

  /* ── Sous-menu langues ────────────────────────────────────────────────── */
  .ctx-lang-panel {
    padding: 8px 10px;
    border-top: 1px solid var(--color-border);
    background: color-mix(in srgb, var(--color-surface) 60%, var(--color-panel));
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .ctx-lang-group {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .ctx-lang-label {
    display: flex;
    align-items: center;
    gap: 4px;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--color-subtext);
  }
  .ctx-lang-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .ctx-chip {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    font-weight: 700;
    padding: 3px 7px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-panel);
    color: var(--color-subtext);
    cursor: pointer;
    transition: all 0.1s;
    white-space: nowrap;
  }
  .ctx-chip:hover {
    border-color: var(--color-subtext);
    color: var(--color-text);
  }
  .ctx-chip--audio-on {
    background: color-mix(in srgb, var(--color-accent) 15%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 45%, transparent);
    color: var(--color-accent);
  }
  .ctx-chip--sub-on {
    background: color-mix(in srgb, var(--color-success) 12%, transparent);
    border-color: color-mix(in srgb, var(--color-success) 40%, transparent);
    color: var(--color-success);
  }
  .ctx-lang-empty {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
    opacity: 0.5;
    text-align: center;
    padding: 4px 0;
  }
  .ctx-lang-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    padding-top: 4px;
    border-top: 1px solid color-mix(in srgb, var(--color-border) 60%, transparent);
  }
  .ctx-action-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 600;
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    transition: all 0.1s;
  }
  .ctx-action-btn--reset:hover {
    border-color: var(--color-danger);
    color: var(--color-danger);
    background: color-mix(in srgb, var(--color-danger) 8%, transparent);
  }
  .ctx-action-btn--apply {
    background: var(--color-accent);
    border-color: var(--color-accent);
    color: #fff;
  }
  .ctx-action-btn--apply:hover {
    opacity: 0.85;
  }

  /* ── Override dot ─────────────────────────────────────────────────────── */
  .override-dot {
    display: inline-block;
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--color-accent);
    margin-left: 4px;
    vertical-align: middle;
    flex-shrink: 0;
    opacity: 0.85;
  }

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
  tbody tr {
    height: 44px;
    max-height: 44px;
  }
  tbody td {
    overflow: hidden;
    white-space: nowrap;
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

  /* ── Tags langue dans cellules ── */
  .cell-lang-grid {
    display: grid;
    grid-template-columns: repeat(2, auto);
    gap: 2px;
    justify-content: center;
    justify-items: center;
    max-height: calc(2 * (1.4em + 4px) + 2px); /* 2 lignes de tags */
    overflow: hidden;
  }
  .cell-lang-tag {
    font-family: "Geist Mono", monospace;
    font-size: 8px;
    font-weight: 700;
    padding: 1px 4px;
    border-radius: 3px;
    border: 1px solid transparent;
    line-height: 1.4;
    white-space: nowrap;
  }
  .cell-lang-tag--off {
    color: var(--color-subtext2);
    opacity: 0.45;
  }
  .cell-lang-tag--on {
    background: color-mix(in srgb, var(--color-accent) 12%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 30%, transparent);
    color: var(--color-accent);
  }
  .cell-lang-tag--sub {
    background: color-mix(in srgb, var(--color-success) 10%, transparent);
    border-color: color-mix(in srgb, var(--color-success) 28%, transparent);
    color: var(--color-success);
  }
  .cell-override-dot {
    font-size: 6px;
    color: var(--color-warning);
    line-height: 1;
    align-self: flex-start;
    margin-top: 2px;
  }

  /* ── Bouton reset pistes rapide ── */
  .apply-all-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    padding: 2px 8px;
    border-radius: var(--radius-xs);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-subtext);
    cursor: pointer;
    transition: background 0.1s, color 0.1s, border-color 0.1s;
    white-space: nowrap;
  }
  .apply-all-btn:hover:not(:disabled) {
    background: color-mix(in srgb, var(--color-warning) 10%, transparent);
    border-color: var(--color-warning);
    color: var(--color-warning);
  }
  .apply-all-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .ctx-extract-fmt {
    margin-left: auto;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 700;
    padding: 1px 5px;
    border-radius: var(--radius-full);
    background: color-mix(in srgb, var(--color-success) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-success) 25%, transparent);
    color: var(--color-success);
  }

  /* ── Empty state ──────────────────────────────────────────────────────── */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }
  .empty-icon {
    width: 20px;
    height: 20px;
    color: var(--color-subtext);
    opacity: 0.3;
    margin-bottom: 4px;
  }
  .empty-title {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--color-subtext);
    opacity: 0.6;
  }
  .empty-hint {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext);
    opacity: 0.4;
    letter-spacing: 0.05em;
  }

  /* ── Drop overlay ─────────────────────────────────────────────────────── */
  .drop-overlay {
    position: absolute;
    inset: 0;
    z-index: 50;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-surface));
    backdrop-filter: blur(2px);
    pointer-events: none;
  }
  .drop-overlay-inner {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }
  .drop-icon {
    width: 36px;
    height: 36px;
    color: var(--color-accent);
    opacity: 0.85;
    animation: drop-bounce 0.6s ease-in-out infinite alternate;
  }
  @keyframes drop-bounce {
    from { transform: translateY(0px); }
    to   { transform: translateY(-6px); }
  }
  .drop-label {
    font-family: "Geist Mono", monospace;
    font-size: 13px;
    font-weight: 700;
    letter-spacing: 0.04em;
    color: var(--color-accent);
  }
  .drop-hint {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
    opacity: 0.7;
    letter-spacing: 0.06em;
  }
</style>