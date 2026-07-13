<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { TAG_LABELS } from "$lib/stores/naming";
  import type { TagId, YearFormat } from "$lib/stores/types";
  import { ArrowUp, ArrowDown, GripVertical, RotateCcw } from "@lucide/svelte";

  let {
    tagOrder,
    disabledTags,
    yearFormat,
  }: {
    tagOrder: TagId[];
    disabledTags: Set<TagId>;
    yearFormat: YearFormat;
  } = $props();

  // ── Drag & drop ─────────────────────────────────────────────────────────────

  let localOrder = $state<TagId[]>([]);
  let dragIndex = $state<number | null>(null);
  let dragOverIndex = $state<number | null>(null);
  let listEl = $state<HTMLElement | null>(null);

  $effect(() => {
    localOrder = [...tagOrder];
  });

  function getRowIndex(y: number): number | null {
    if (!listEl) return null;
    const rows = Array.from(listEl.children) as HTMLElement[];
    for (let i = 0; i < rows.length; i++) {
      const r = rows[i].getBoundingClientRect();
      if (y >= r.top && y <= r.bottom) return i;
    }
    if (!rows.length) return null;
    if (y < rows[0].getBoundingClientRect().top) return 0;
    if (y > rows[rows.length - 1].getBoundingClientRect().bottom)
      return rows.length - 1;
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
      const dir = steps > 0 ? 1 : -1;
      for (let s = 0; s < Math.abs(steps); s++) encoder.moveTag(id, dir);
    }
    dragIndex = null;
    dragOverIndex = null;
  }
</script>

<section class="content-section">
  <header class="section-header">
    <div>
      <h2 class="section-title">Ordre des tags</h2>
      <p class="section-desc">
        Glissez les tags pour réorganiser leur ordre dans le nom de fichier.
        Désactivez ceux que vous ne souhaitez pas inclure.
      </p>
    </div>
    <button
      type="button"
      class="reset-btn"
      onclick={() => encoder.resetTagOrder()}
      title="Remettre l'ordre par défaut"
    >
      <RotateCcw class="w-3 h-3" />
      Réinitialiser
    </button>
  </header>

  <div
    class="tag-list"
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
          {dragOverIndex === i && dragIndex !== null && dragIndex !== i
          ? 'tag-row--over'
          : ''}
          {disabled ? 'tag-row--disabled' : ''}
          {id === 'year' ? 'tag-row--with-sub' : ''}"
        role="listitem"
      >
        <!-- Ligne principale -->
        <div class="tag-row-main">
          <!-- Position badge -->
          <div class="tag-pos">
            {disabled
              ? "—"
              : i +
                1 -
                [...localOrder.slice(0, i)].filter((t) => disabledTags.has(t))
                  .length}
          </div>

          <!-- Toggle -->
          <button
            type="button"
            class="toggle-switch {disabled ? '' : 'toggle-switch--on'}"
            onclick={() => encoder.toggleTag(id)}
            aria-pressed={!disabled}
            aria-label={disabled
              ? `Activer ${TAG_LABELS[id]}`
              : `Désactiver ${TAG_LABELS[id]}`}
          >
            <span class="toggle-thumb"></span>
          </button>

          <!-- Label -->
          <span class="tag-label {disabled ? 'tag-label--off' : ''}">
            {TAG_LABELS[id]}
            {#if id === "team" && !encoder.team}
              <span class="tag-empty">vide</span>
            {/if}
          </span>

          <!-- Actions -->
          <div class="tag-actions">
            <button
              type="button"
              class="icon-btn"
              disabled={i === 0}
              onclick={() => encoder.moveTag(id, -1)}
              aria-label="Monter"
            >
              <ArrowUp class="w-3 h-3" />
            </button>
            <button
              type="button"
              class="icon-btn"
              disabled={i === localOrder.length - 1}
              onclick={() => encoder.moveTag(id, 1)}
              aria-label="Descendre"
            >
              <ArrowDown class="w-3 h-3" />
            </button>
            <button
              type="button"
              class="drag-handle"
              aria-label="Déplacer"
              onpointerdown={(e) => onHandlePointerDown(e, i)}
            >
              <GripVertical class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      </div>
    {/each}
  </div>
</section>

<style>
  .content-section {
    padding: 28px 24px;
  }
  .section-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 24px;
    padding-bottom: 20px;
    border-bottom: 1px solid var(--color-border);
  }
  .section-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text);
    letter-spacing: -0.02em;
    margin: 0 0 6px;
  }
  .section-desc {
    font-size: 12px;
    color: var(--color-subtext);
    line-height: 1.6;
    max-width: 380px;
    margin: 0;
  }

  .reset-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 5px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-panel);
    color: var(--color-subtext);
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    cursor: pointer;
    white-space: nowrap;
    flex-shrink: 0;
    transition:
      border-color 0.12s,
      color 0.12s;
  }
  .reset-btn:hover {
    border-color: var(--color-subtext);
    color: var(--color-text);
  }

  .tag-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    user-select: none;
    touch-action: none;
  }

  .tag-row {
    display: flex;
    flex-direction: column;
    gap: 0;
    padding: 6px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-panel);
    transition:
      background 0.12s,
      border-color 0.12s,
      opacity 0.12s;
  }

  .tag-row-main {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
  }
  .tag-row--dragging {
    opacity: 0.5;
    border-style: dashed;
  }
  .tag-row--over {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 6%, var(--color-panel));
  }
  .tag-row--disabled {
    opacity: 0.45;
  }

  .tag-pos {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext2);
    width: 16px;
    text-align: center;
    flex-shrink: 0;
  }

  .toggle-switch {
    position: relative;
    width: 28px;
    height: 16px;
    border-radius: 999px;
    background: var(--color-border);
    border: none;
    cursor: pointer;
    transition: background 0.15s;
    flex-shrink: 0;
    padding: 0;
  }
  .toggle-switch--on {
    background: var(--color-accent);
  }
  .toggle-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: white;
    transition: transform 0.15s;
  }
  .toggle-switch--on .toggle-thumb {
    transform: translateX(12px);
  }

  .tag-label {
    flex: 1;
    font-size: 13px;
    color: var(--color-text);
  }
  .tag-label--off {
    color: var(--color-subtext2);
  }
  .tag-empty {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext2);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 3px;
    padding: 1px 4px;
    margin-left: 4px;
  }

  .tag-actions {
    display: flex;
    gap: 2px;
    align-items: center;
    flex-shrink: 0;
  }
  .icon-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: var(--radius-xs);
    border: 1px solid transparent;
    background: transparent;
    color: var(--color-subtext2);
    cursor: pointer;
    transition:
      background 0.1s,
      color 0.1s,
      border-color 0.1s;
  }
  .icon-btn:hover:not(:disabled) {
    background: var(--color-surface);
    border-color: var(--color-border);
    color: var(--color-text);
  }
  .icon-btn:disabled {
    opacity: 0.25;
    cursor: default;
  }

  .drag-handle {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: var(--radius-xs);
    border: 1px solid transparent;
    background: transparent;
    color: var(--color-subtext2);
    cursor: grab;
    touch-action: none;
    transition:
      background 0.1s,
      color 0.1s;
  }
  .drag-handle:hover {
    background: var(--color-surface);
    border-color: var(--color-border);
    color: var(--color-text);
  }
  .drag-handle:active {
    cursor: grabbing;
  }</style>