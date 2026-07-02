<script lang="ts">
  import { encoder, TAG_LABELS, type TagId } from "$lib/stores/encoder.svelte";
  import {
    X,
    ArrowUp,
    ArrowDown,
    GripVertical,
    RotateCcw,
  } from "@lucide/svelte";

  let { onClose }: { onClose: () => void } = $props();

  let tagOrder = $derived(encoder.tagOrder);
  let team = $derived(encoder.team);

  let activeTab = $state<"tags" | "team">("tags");

  // Copie locale pour affichage immédiat pendant le drag
  let localOrder = $state<TagId[]>([]);
  $effect(() => {
    localOrder = [...tagOrder];
  });

  // --- Pointer-based drag & drop ---
  let dragIndex = $state<number | null>(null);
  let dragOverIndex = $state<number | null>(null);
  let listEl = $state<HTMLElement | null>(null);

  function getRowIndex(y: number): number | null {
    if (!listEl) return null;
    const rows = Array.from(listEl.children) as HTMLElement[];
    for (let i = 0; i < rows.length; i++) {
      const rect = rows[i].getBoundingClientRect();
      if (y >= rect.top && y <= rect.bottom) return i;
    }
    // Clamp aux extrémités
    if (rows.length === 0) return null;
    const first = rows[0].getBoundingClientRect();
    const last = rows[rows.length - 1].getBoundingClientRect();
    if (y < first.top) return 0;
    if (y > last.bottom) return rows.length - 1;
    return null;
  }

  function onHandlePointerDown(e: PointerEvent, i: number) {
    // Uniquement le bouton principal (clic gauche / touch)
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

  function onHandlePointerUp(e: PointerEvent) {
    if (dragIndex === null) return;
    const toIndex = dragOverIndex ?? dragIndex;

    if (toIndex !== dragIndex) {
      const id = localOrder[dragIndex];

      // Mise à jour visuelle immédiate
      const newOrder = [...localOrder];
      newOrder.splice(dragIndex, 1);
      newOrder.splice(toIndex, 0, id);
      localOrder = newOrder;

      // Synchronisation encoder
      const steps = toIndex - dragIndex;
      const dir = steps > 0 ? 1 : -1;
      for (let s = 0; s < Math.abs(steps); s++) {
        encoder.moveTag(id, dir);
      }
    }

    dragIndex = null;
    dragOverIndex = null;
  }

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
      path: "",
      filename: "",
      size_mb: 0,
      duration_secs: 0,
      fps: 0,
      audio_langs: [],
      sub_langs: [],
      streams: [],
      status: "ready",
      output_ext: ".mkv",
      sub_extract_status: "none",
    });
  }
</script>

<div class="panel-root">
  <div class="panel-header">
    <div class="flex items-center gap-2">
      <div
        class="w-[3px] h-4 rounded-[1px]"
        style="background: var(--color-accent);"
      ></div>
      <span class="section-label">Renommage — Ordre des tags &amp; team</span>
    </div>
    <button onclick={onClose} class="icon-btn" aria-label="Fermer">
      <X class="w-4 h-4" />
    </button>
  </div>

  <div class="panel-body space-y-3">
    <div class="tab-bar">
      <button
        type="button"
        class="tab-btn {activeTab === 'tags' ? 'tab-btn--active' : ''}"
        onclick={() => (activeTab = "tags")}>Ordre des tags</button
      >
      <button
        type="button"
        class="tab-btn {activeTab === 'team' ? 'tab-btn--active' : ''}"
        onclick={() => (activeTab = "team")}
        >Team{team ? ` · ${team}` : ""}</button
      >
    </div>

    {#if activeTab === "tags"}
      <p class="font-mono text-[10px]" style="color: var(--color-subtext);">
        Glissez ⠿ ou utilisez les flèches pour réorganiser.
      </p>

      <div
        class="space-y-1"
        role="list"
        bind:this={listEl}
        onpointermove={onHandlePointerMove}
        onpointerup={onHandlePointerUp}
        onpointercancel={onHandlePointerUp}
      >
        {#each localOrder as id, i (id)}
          <div
            class="tag-row
              {dragIndex === i ? 'tag-row--dragging' : ''}
              {dragOverIndex === i && dragIndex !== null && dragIndex !== i
              ? 'tag-row--over'
              : ''}"
            role="listitem"
          >
            <!-- Poignée : seul point d'initiation du drag -->
            <span
              class="drag-handle"
              role="button"
              tabindex="0"
              aria-label="Déplacer {TAG_LABELS[id]}"
              onpointerdown={(e) => onHandlePointerDown(e, i)}
            >
              <GripVertical class="w-3.5 h-3.5" />
            </span>

            <span
              class="font-mono text-[11px] flex-1"
              style="color: {id === 'team' && !team
                ? 'var(--color-subtext2)'
                : 'var(--color-text)'};"
            >
              {TAG_LABELS[id]}{#if id === "team" && !team}
                <span
                  class="font-mono text-[9px]"
                  style="color: var(--color-subtext2);"
                >
                  (vide — ignoré)</span
                >
              {/if}
            </span>

            <div class="tag-row-actions">
              <button
                type="button"
                class="icon-btn"
                disabled={i === 0}
                onclick={() => encoder.moveTag(id, -1)}
                aria-label={`Monter ${TAG_LABELS[id]}`}
                ><ArrowUp class="w-3.5 h-3.5" /></button
              >
              <button
                type="button"
                class="icon-btn"
                disabled={i === localOrder.length - 1}
                onclick={() => encoder.moveTag(id, 1)}
                aria-label={`Descendre ${TAG_LABELS[id]}`}
                ><ArrowDown class="w-3.5 h-3.5" /></button
              >
            </div>
          </div>
        {/each}
      </div>

      <div class="info-box">
        <span class="font-mono text-[9px]" style="color: var(--color-subtext);"
          >Exemple :</span
        >
        <span
          class="font-mono text-[11px] ml-1.5"
          style="color: var(--color-text);">{previewName()}</span
        >
      </div>

      <button
        type="button"
        class="reset-btn"
        onclick={() => encoder.resetTagOrder()}
        title="Remettre l'ordre par défaut"
      >
        <RotateCcw class="w-3 h-3" />
        Réinitialiser l'ordre
      </button>
    {/if}

    {#if activeTab === "team"}
      <p class="font-mono text-[10px]" style="color: var(--color-subtext);">
        Renseignez un nom de team pour l'insérer dans le nom de sortie à la
        position définie dans l'onglet « Ordre des tags ».
      </p>

      <input
        type="text"
        value={team}
        oninput={(e: Event) =>
          encoder.setTeam((e.target as HTMLInputElement).value)}
        placeholder="Nom de la team (optionnel)"
        class="w-full font-mono text-[11px] px-2 py-1.5 rounded-[var(--radius-sm)]"
        style="background: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-text);"
      />

      {#if team}
        <div class="info-box">
          <span
            class="font-mono text-[9px]"
            style="color: var(--color-subtext);">Aperçu :</span
          >
          <span
            class="font-mono text-[11px] ml-1.5"
            style="color: var(--color-text);">{previewName()}</span
          >
        </div>
      {:else}
        <div class="info-box">
          <span
            class="font-mono text-[10px]"
            style="color: var(--color-subtext2);"
          >
            Aucune team — le tag sera ignoré dans le nom de sortie.
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
    background: var(--color-panel);
    flex-shrink: 0;
  }

  .panel-body {
    padding: 12px 16px 16px;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
  }

  .icon-btn {
    width: 24px;
    height: 24px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-xs);
    border: 1px solid transparent;
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    transition:
      background 0.1s,
      color 0.1s,
      border-color 0.1s;
  }
  .icon-btn:hover {
    background: var(--color-panel2);
    border-color: var(--color-border);
    color: var(--color-text);
  }
  .icon-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .drag-handle {
    display: inline-flex;
    align-items: center;
    color: var(--color-subtext2);
    cursor: grab;
    flex-shrink: 0;
    padding: 2px 4px;
    border-radius: var(--radius-xs);
    touch-action: none; /* indispensable pour pointer events sur mobile */
    transition:
      color 0.1s,
      background 0.1s;
  }
  .drag-handle:hover {
    color: var(--color-subtext);
    background: var(--color-panel2);
  }
  .drag-handle:active {
    cursor: grabbing;
  }

  .tag-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    transition:
      opacity 0.12s,
      border-color 0.12s,
      background 0.12s;
    user-select: none;
  }

  .tag-row--dragging {
    opacity: 0.35;
  }

  .tag-row--over {
    border-color: var(--color-accent);
    background: color-mix(
      in srgb,
      var(--color-accent) 8%,
      var(--color-surface)
    );
  }

  .tag-row-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .info-box {
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
  }

  .reset-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    padding: 5px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    transition:
      background 0.1s,
      color 0.1s,
      border-color 0.1s;
  }
  .reset-btn:hover {
    background: var(--color-panel2);
    border-color: color-mix(
      in srgb,
      var(--color-danger) 40%,
      var(--color-border)
    );
    color: var(--color-danger);
  }

  .tab-bar {
    display: flex;
    gap: 4px;
    padding: 3px;
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
  }

  .tab-btn {
    flex: 1;
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    padding: 5px 8px;
    border-radius: calc(var(--radius-sm) - 2px);
    border: 1px solid transparent;
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    transition:
      background 0.12s,
      color 0.12s,
      border-color 0.12s;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .tab-btn:hover {
    color: var(--color-text);
    background: var(--color-panel2);
  }
  .tab-btn--active {
    background: var(--color-panel);
    border-color: var(--color-border);
    color: var(--color-accent);
    font-weight: 600;
  }
</style>
