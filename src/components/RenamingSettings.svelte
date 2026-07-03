<script lang="ts">
  import {
    encoder,
    TAG_LABELS,
    SEASON_EPISODE_FORMATS,
    formatSeasonEpisode,
    type TagId,
    type ResolutionCase,
    type TitleCaseMode,
    type CodecFormat,
    type SourceCase,
  } from "$lib/stores/encoder.svelte";
  import { X, ArrowUp, ArrowDown, GripVertical, RotateCcw } from "@lucide/svelte";

  let { onClose }: { onClose: () => void } = $props();

  let tagOrder     = $derived(encoder.tagOrder);
  let disabledTags = $derived(encoder.disabledTags);
  let resCase      = $derived(encoder.resolutionCase);
  let titleCase    = $derived(encoder.titleCase);
  let codecFmt     = $derived(encoder.codecFormat);
  let sourceCase   = $derived(encoder.sourceCase);
  let seFormat     = $derived(encoder.seasonEpisodeFormat);
  let team         = $derived(encoder.team);

  let activeTab = $state<"tags" | "format" | "team">("tags");

  // ── Copie locale pour le drag ──────────────────────────────────────────────
  let localOrder = $state<TagId[]>([]);
  $effect(() => { localOrder = [...tagOrder]; });

  // ── Drag & drop (pointer events) ──────────────────────────────────────────
  let dragIndex     = $state<number | null>(null);
  let dragOverIndex = $state<number | null>(null);
  let listEl        = $state<HTMLElement | null>(null);

  function getRowIndex(y: number): number | null {
    if (!listEl) return null;
    const rows = Array.from(listEl.children) as HTMLElement[];
    for (let i = 0; i < rows.length; i++) {
      const rect = rows[i].getBoundingClientRect();
      if (y >= rect.top && y <= rect.bottom) return i;
    }
    if (rows.length === 0) return null;
    const first = rows[0].getBoundingClientRect();
    const last  = rows[rows.length - 1].getBoundingClientRect();
    if (y < first.top) return 0;
    if (y > last.bottom) return rows.length - 1;
    return null;
  }

  function onHandlePointerDown(e: PointerEvent, i: number) {
    if (e.button !== 0 && e.pointerType === "mouse") return;
    e.preventDefault();
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    dragIndex = i;
    dragOverIndex = i;
  }

  function onHandlePointerMove(e: PointerEvent) {
    if (dragIndex === null) return;
    e.preventDefault();
    const over = getRowIndex(e.clientY);
    if (over !== null) dragOverIndex = over;
  }

  function onHandlePointerUp() {
    if (dragIndex === null) return;
    const toIndex = dragOverIndex ?? dragIndex;
    if (toIndex !== dragIndex) {
      const id = localOrder[dragIndex];
      const newOrder = [...localOrder];
      newOrder.splice(dragIndex, 1);
      newOrder.splice(toIndex, 0, id);
      localOrder = newOrder;
      const steps = toIndex - dragIndex;
      const dir   = steps > 0 ? 1 : -1;
      for (let s = 0; s < Math.abs(steps); s++) encoder.moveTag(id, dir);
    }
    dragIndex = null;
    dragOverIndex = null;
  }

  // ── Aperçu ────────────────────────────────────────────────────────────────
  function previewName() {
    return encoder.getDisplayName({
      cleaned: {
        title: "Jujutsu Kaisen",
        season_episode: "S03E01",
        resolution: "1080P",
        source: "BluRay",
        provider: "",
        audio_tags: "VOSTFR",
        suggested: "",
      },
      output_name: "VOSTFR AAC",
      path: "", filename: "", size_mb: 0, duration_secs: 0,
      fps: 0, audio_langs: [], sub_langs: [], streams: [],
      status: "ready", output_ext: ".mkv", sub_extract_status: "none",
    });
  }

  const TITLE_CASE_OPTIONS: { value: TitleCaseMode; label: string; preview: string }[] = [
    { value: "original", label: "Original",    preview: "Jujutsu Kaisen" },
    { value: "upper",    label: "MAJUSCULES",   preview: "JUJUTSU KAISEN" },
    { value: "lower",    label: "minuscules",   preview: "jujutsu kaisen" },
    { value: "title",    label: "Titre",        preview: "Jujutsu Kaisen" },
  ];
</script>

<div class="panel-root">
  <!-- En-tête -->
  <div class="panel-header">
    <div class="flex items-center gap-2">
      <div class="w-[3px] h-4 rounded-[1px]" style="background: var(--color-accent);"></div>
      <span class="section-label">Renommage — Tags &amp; format</span>
    </div>
    <button onclick={onClose} class="icon-btn" aria-label="Fermer">
      <X class="w-4 h-4" />
    </button>
  </div>

  <!-- Onglets -->
  <div class="tab-bar-outer">
    <div class="tab-bar">
      <button type="button"
        class="tab-btn {activeTab === 'tags'   ? 'tab-btn--active' : ''}"
        onclick={() => (activeTab = "tags")}>Ordre</button>
      <button type="button"
        class="tab-btn {activeTab === 'format' ? 'tab-btn--active' : ''}"
        onclick={() => (activeTab = "format")}>Format</button>
      <button type="button"
        class="tab-btn {activeTab === 'team'   ? 'tab-btn--active' : ''}"
        onclick={() => (activeTab = "team")}>Team{team ? ` · ${team}` : ""}</button>
    </div>
  </div>

  <div class="panel-body space-y-3">

    <!-- ── Onglet Ordre ──────────────────────────────────────────────────── -->
    {#if activeTab === "tags"}
      <p class="hint">Glissez ⠿ ou utilisez les flèches. Décochez pour masquer un tag.</p>

      <div
        class="space-y-1"
        role="list"
        bind:this={listEl}
        onpointermove={onHandlePointerMove}
        onpointerup={onHandlePointerUp}
        onpointercancel={onHandlePointerUp}
      >
        {#each localOrder as id, i (id)}
          {@const disabled = disabledTags.has(id)}
          <div
            class="tag-row
              {dragIndex === i ? 'tag-row--dragging' : ''}
              {dragOverIndex === i && dragIndex !== null && dragIndex !== i ? 'tag-row--over' : ''}
              {disabled ? 'tag-row--disabled' : ''}"
            role="listitem"
          >
            <!-- Toggle actif/inactif -->
            <button
              type="button"
              class="toggle-btn {disabled ? '' : 'toggle-btn--on'}"
              onclick={() => encoder.toggleTag(id)}
              aria-label={disabled ? `Activer ${TAG_LABELS[id]}` : `Désactiver ${TAG_LABELS[id]}`}
              aria-pressed={!disabled}
              title={disabled ? "Tag masqué" : "Tag actif"}
            >
              <span class="toggle-dot"></span>
            </button>

            <!-- Poignée drag -->
            <button
              type="button"
              class="drag-handle"
              aria-label="Déplacer {TAG_LABELS[id]}"
              onpointerdown={(e) => onHandlePointerDown(e, i)}
            >
              <GripVertical class="w-3.5 h-3.5" />
            </button>

            <span
              class="font-mono text-[11px] flex-1 {disabled ? 'line-through' : ''}"
              style="color: {disabled
                ? 'var(--color-subtext2)'
                : id === 'team' && !team
                ? 'var(--color-subtext2)'
                : 'var(--color-text)'};"
            >
              {TAG_LABELS[id]}{#if id === "team" && !team}
                <span style="color:var(--color-subtext2);font-size:9px;"> (vide)</span>
              {/if}
            </span>

            <div class="tag-row-actions">
              <button type="button" class="icon-btn" disabled={i === 0}
                onclick={() => encoder.moveTag(id, -1)} aria-label={`Monter ${TAG_LABELS[id]}`}>
                <ArrowUp class="w-3.5 h-3.5" />
              </button>
              <button type="button" class="icon-btn" disabled={i === localOrder.length - 1}
                onclick={() => encoder.moveTag(id, 1)} aria-label={`Descendre ${TAG_LABELS[id]}`}>
                <ArrowDown class="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
        {/each}
      </div>

      <div class="info-box">
        <span class="font-mono text-[9px]" style="color:var(--color-subtext);">Exemple :</span>
        <span class="font-mono text-[11px] ml-1.5" style="color:var(--color-text);">{previewName()}</span>
      </div>

      <button type="button" class="reset-btn" onclick={() => encoder.resetTagOrder()}
        title="Remettre l'ordre et les tags par défaut">
        <RotateCcw class="w-3 h-3" />
        Réinitialiser
      </button>
    {/if}

    <!-- ── Onglet Format ─────────────────────────────────────────────────── -->
    {#if activeTab === "format"}

      <!-- Résolution -->
      <div class="section-block">
        <span class="block-label">Casse de la résolution</span>
        <div class="seg-group">
          {#each ([["upper", "1080P"], ["lower", "1080p"]] as const) as [val, preview]}
            <button
              type="button"
              class="seg-btn {resCase === val ? 'seg-btn--active' : ''}"
              onclick={() => encoder.setResolutionCase(val as ResolutionCase)}
            >{preview}</button>
          {/each}
        </div>
      </div>

      <!-- Casse du titre -->
      <div class="section-block">
        <span class="block-label">Casse du titre</span>
        <div class="seg-group seg-group--col">
          {#each TITLE_CASE_OPTIONS as opt}
            <button
              type="button"
              class="seg-btn seg-btn--wide {titleCase === opt.value ? 'seg-btn--active' : ''}"
              onclick={() => encoder.setTitleCase(opt.value)}
            >
              <span class="seg-label">{opt.label}</span>
              <span class="seg-preview">{opt.preview}</span>
            </button>
          {/each}
        </div>
      </div>

      <!-- Saison / Épisode -->
      <div class="section-block">
        <span class="block-label">Format saison/épisode</span>
        <div class="seg-group seg-group--col">
          {#each SEASON_EPISODE_FORMATS as f}
            <button
              type="button"
              class="seg-btn seg-btn--wide {seFormat === f.value ? 'seg-btn--active' : ''}"
              onclick={() => encoder.setSeasonEpisodeFormat(f.value)}
            >
              <span class="seg-label">{f.label}</span>
              <span class="seg-preview">{formatSeasonEpisode("S03E01", f.value)}</span>
            </button>
          {/each}
        </div>
      </div>

      <!-- Codec vidéo -->
      <div class="section-block">
        <span class="block-label">Format du codec vidéo</span>
        <div class="seg-group">
          {#each (["H265", "H.265", "HEVC"] as const) as val}
            <button
              type="button"
              class="seg-btn {codecFmt === val ? 'seg-btn--active' : ''}"
              onclick={() => encoder.setCodecFormat(val as CodecFormat)}
            >{val}</button>
          {/each}
        </div>
      </div>

      <!-- Source -->
      <div class="section-block">
        <span class="block-label">Casse de la source</span>
        <div class="seg-group">
          {#each ([["original", "BluRay"], ["upper", "BLURAY"], ["lower", "bluray"]] as const) as [val, preview]}
            <button
              type="button"
              class="seg-btn {sourceCase === val ? 'seg-btn--active' : ''}"
              onclick={() => encoder.setSourceCase(val as SourceCase)}
            >{preview}</button>
          {/each}
        </div>
      </div>

      <div class="info-box">
        <span class="font-mono text-[9px]" style="color:var(--color-subtext);">Exemple :</span>
        <span class="font-mono text-[11px] ml-1.5" style="color:var(--color-text);">{previewName()}</span>
      </div>
    {/if}

    <!-- ── Onglet Team ───────────────────────────────────────────────────── -->
    {#if activeTab === "team"}
      <p class="hint">Inséré dans le nom à la position définie dans l'onglet « Ordre ».</p>

      <input
        type="text"
        value={team}
        oninput={(e: Event) => encoder.setTeam((e.target as HTMLInputElement).value)}
        placeholder="Nom de la team (optionnel)"
        class="w-full font-mono text-[11px] px-2 py-1.5 rounded-[var(--radius-sm)]"
        style="background:var(--color-surface);border:1px solid var(--color-border);color:var(--color-text);"
      />

      {#if team}
        <div class="info-box">
          <span class="font-mono text-[9px]" style="color:var(--color-subtext);">Aperçu :</span>
          <span class="font-mono text-[11px] ml-1.5" style="color:var(--color-text);">{previewName()}</span>
        </div>
      {:else}
        <div class="info-box">
          <span class="font-mono text-[10px]" style="color:var(--color-subtext2);">
            Aucune team — le tag sera ignoré.
          </span>
        </div>
      {/if}
    {/if}

  </div>
</div>

<style>
  .panel-root {
    display: flex;
    flex-direction: column;
    background: var(--color-panel);
    height: 100%;
  }
  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }
  .tab-bar-outer {
    padding: 8px 12px 0;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }
  .tab-bar {
    display: flex;
    gap: 2px;
  }
  .tab-btn {
    flex: 1;
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    padding: 6px 8px;
    border: none;
    border-bottom: 2px solid transparent;
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    transition: color 0.12s, border-color 0.12s;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: -1px;
  }
  .tab-btn:hover { color: var(--color-text); }
  .tab-btn--active {
    color: var(--color-accent);
    border-bottom-color: var(--color-accent);
    font-weight: 600;
  }

  .panel-body { padding: 12px 16px 16px; overflow-y: auto; }

  .hint {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
  }

  /* ── Toggle bouton actif/inactif ── */
  .toggle-btn {
    width: 26px;
    height: 14px;
    border-radius: 7px;
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    padding: 0 2px;
    flex-shrink: 0;
    transition: background 0.15s, border-color 0.15s;
  }
  .toggle-btn--on {
    background: var(--color-accent);
    border-color: var(--color-accent);
  }
  .toggle-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-subtext2);
    transition: transform 0.15s, background 0.15s;
  }
  .toggle-btn--on .toggle-dot {
    transform: translateX(12px);
    background: #fff;
  }

  /* ── Drag handle ── */
  .drag-handle {
    display: inline-flex;
    align-items: center;
    color: var(--color-subtext2);
    cursor: grab;
    flex-shrink: 0;
    padding: 2px 4px;
    border-radius: var(--radius-xs);
    touch-action: none;
    transition: color 0.1s, background 0.1s;
  }
  .drag-handle:hover  { color: var(--color-subtext); background: var(--color-panel2); }
  .drag-handle:active { cursor: grabbing; }

  /* ── Lignes de tag ── */
  .tag-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 8px;
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    transition: opacity 0.12s, border-color 0.12s, background 0.12s;
    user-select: none;
  }
  .tag-row--dragging { opacity: 0.35; }
  .tag-row--over {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-surface));
  }
  .tag-row--disabled { opacity: 0.55; }

  .tag-row-actions { display: flex; align-items: center; gap: 4px; flex-shrink: 0; }

  /* ── Icône bouton ── */
  .icon-btn {
    width: 24px; height: 24px;
    display: inline-flex; align-items: center; justify-content: center;
    border-radius: var(--radius-xs);
    border: 1px solid transparent;
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    transition: background 0.1s, color 0.1s, border-color 0.1s;
  }
  .icon-btn:hover  { background: var(--color-panel2); border-color: var(--color-border); color: var(--color-text); }
  .icon-btn:disabled { opacity: 0.35; cursor: not-allowed; }

  /* ── Reset ── */
  .reset-btn {
    display: inline-flex; align-items: center; gap: 5px;
    font-family: "Geist Mono", monospace; font-size: 10px;
    padding: 5px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    transition: background 0.1s, color 0.1s, border-color 0.1s;
  }
  .reset-btn:hover {
    background: var(--color-panel2);
    border-color: color-mix(in srgb, var(--color-danger) 40%, var(--color-border));
    color: var(--color-danger);
  }

  /* ── Info box ── */
  .info-box {
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
  }

  /* ── Section blocs (onglet Format) ── */
  .section-block { display: flex; flex-direction: column; gap: 6px; }
  .block-label {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
  }

  /* Segmented control */
  .seg-group {
    display: flex;
    gap: 4px;
    padding: 3px;
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
  }
  .seg-group--col { flex-direction: column; }

  .seg-btn {
    flex: 1;
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    padding: 5px 10px;
    border-radius: calc(var(--radius-sm) - 2px);
    border: 1px solid transparent;
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    transition: background 0.12s, color 0.12s, border-color 0.12s;
    text-align: center;
  }
  .seg-btn:hover { color: var(--color-text); background: var(--color-panel2); }
  .seg-btn--active {
    background: var(--color-panel);
    border-color: var(--color-border);
    color: var(--color-accent);
    font-weight: 600;
  }

  /* Variante wide avec preview à droite */
  .seg-btn--wide {
    display: flex;
    align-items: center;
    justify-content: space-between;
    text-align: left;
    padding: 6px 10px;
  }
  .seg-label  { font-size: 10px; }
  .seg-preview {
    font-size: 10px;
    opacity: 0.6;
    font-style: italic;
  }
  .seg-btn--active .seg-preview { opacity: 0.85; }
</style>