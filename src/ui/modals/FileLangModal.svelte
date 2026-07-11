<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { LANG_ORDER, langName } from "$lib/stores/naming";
  import type { AppFile } from "$lib/stores/types";
  import { untrack } from "svelte";
  import {
    X,
    Headphones,
    MessageSquare,
    RotateCcw,
    CircleCheck,
  } from "@lucide/svelte";

  let { file, onclose }: { file: AppFile; onclose: () => void } = $props();

  // untrack() : on veut intentionnellement la valeur initiale de file au montage,
  // pas une dérivation réactive (la modal est recréée si le fichier change).
  let selAudio = $state<Set<string>>(
    untrack(() =>
      encoder.fileSelAudio.get(file.path)
        ? new Set(encoder.fileSelAudio.get(file.path))
        : new Set(
            [...encoder.selAudio].filter((l) => file.audio_langs.includes(l)),
          ),
    ),
  );
  let selSubs = $state<Set<string>>(
    untrack(() =>
      encoder.fileSelSubs.get(file.path)
        ? new Set(encoder.fileSelSubs.get(file.path))
        : new Set(
            [...encoder.selSubs].filter((l) => file.sub_langs.includes(l)),
          ),
    ),
  );

  let hasOverride = $derived(
    encoder.fileSelAudio.has(file.path) || encoder.fileSelSubs.has(file.path),
  );

  let sortedAudio = $derived(
    [...file.audio_langs].sort((a, b) => {
      const ai = LANG_ORDER.indexOf(a),
        bi = LANG_ORDER.indexOf(b);
      return (ai === -1 ? 999 : ai) - (bi === -1 ? 999 : bi);
    }),
  );
  let sortedSubs = $derived(
    [...file.sub_langs].sort((a, b) => {
      const ai = LANG_ORDER.indexOf(a),
        bi = LANG_ORDER.indexOf(b);
      return (ai === -1 ? 999 : ai) - (bi === -1 ? 999 : bi);
    }),
  );

  function toggleAudio(lang: string) {
    const s = new Set(selAudio);
    s.has(lang) ? s.delete(lang) : s.add(lang);
    selAudio = s;
  }
  function toggleSub(lang: string) {
    const s = new Set(selSubs);
    s.has(lang) ? s.delete(lang) : s.add(lang);
    selSubs = s;
  }

  function apply() {
    encoder.setFileLangSel(file.path, selAudio, selSubs);
    onclose();
  }

  function reset() {
    encoder.clearFileLangSel(file.path);
    selAudio = new Set(
      [...encoder.selAudio].filter((l) => file.audio_langs.includes(l)),
    );
    selSubs = new Set(
      [...encoder.selSubs].filter((l) => file.sub_langs.includes(l)),
    );
  }
</script>

<!-- Backdrop -->
<div
  class="fixed inset-0 z-[9980] bg-black/40"
  role="presentation"
  onclick={onclose}
></div>

<!-- Modal -->
<div
  class="fixed left-1/2 top-1/2 z-[9981] w-[400px] max-w-[calc(100vw-32px)]
         -translate-x-1/2 -translate-y-1/2
         rounded-[var(--radius-lg)] shadow-[0_20px_60px_rgba(0,0,0,0.5)]
         flex flex-col overflow-hidden"
  style="background: var(--color-panel); border: 1px solid var(--color-border);"
  role="dialog"
  aria-modal="true"
  aria-label="Sélection des pistes — {file.filename}"
>
  <!-- En-tête -->
  <div
    class="flex items-center justify-between px-4 py-2.5 shrink-0"
    style="border-bottom: 1px solid var(--color-border);"
  >
    <div class="flex items-center gap-2 min-w-0">
      <div
        class="w-[3px] h-4 rounded-sm shrink-0"
        style="background: var(--color-accent);"
      ></div>
      <div class="flex flex-col min-w-0">
        <span class="section-label">Pistes &amp; sous-titres</span>
        <span
          class="font-mono text-[10px] truncate max-w-[260px]"
          style="color: var(--color-subtext);"
          title={file.filename}>{file.filename}</span
        >
      </div>
      {#if hasOverride}
        <span
          class="font-mono text-[9px] px-1.5 py-0.5 rounded-[var(--radius-sm)] shrink-0"
          style="background: color-mix(in srgb, var(--color-accent) 12%, transparent);
                 color: var(--color-accent); border: 1px solid color-mix(in srgb, var(--color-accent) 25%, var(--color-border));"
          >override</span
        >
      {/if}
    </div>
    <div class="flex items-center gap-1 shrink-0">
      {#if hasOverride}
        <button
          class="icon-btn"
          onclick={reset}
          title="Réinitialiser aux paramètres globaux"
          aria-label="Réinitialiser"
        >
          <RotateCcw class="w-3.5 h-3.5" />
        </button>
      {/if}
      <button class="icon-btn" onclick={onclose} aria-label="Fermer">
        <X class="w-4 h-4" />
      </button>
    </div>
  </div>

  <!-- Corps -->
  <div class="px-4 py-3 flex flex-col gap-4">
    <!-- Pistes audio -->
    <section class="flex flex-col gap-2">
      <span class="flex items-center gap-1.5 section-label">
        <Headphones class="w-3.5 h-3.5 opacity-70" />
        Pistes audio
      </span>

      {#if sortedAudio.length === 0}
        <p
          class="font-mono text-[10px] italic"
          style="color: var(--color-subtext2);"
        >
          Aucune piste audio détectée
        </p>
      {:else}
        <div class="flex flex-wrap gap-1.5">
          {#each sortedAudio as lang}
            {@const active = selAudio.has(lang)}
            <button
              type="button"
              class="lang-btn {active ? 'lang-btn--active' : ''}"
              onclick={() => toggleAudio(lang)}
              aria-pressed={active}
            >
              {#if active}<CircleCheck class="w-3 h-3 shrink-0" />{/if}
              <span class="text-[9px] tracking-wider" style="opacity:.7"
                >{lang.toUpperCase()}</span
              >
              <span class="font-mono text-[11px]">{langName(lang)}</span>
            </button>
          {/each}
        </div>
      {/if}
    </section>

    <div
      class="h-px"
      style="background: var(--color-border); opacity: .5;"
    ></div>

    <!-- Sous-titres -->
    <section class="flex flex-col gap-2">
      <span class="flex items-center gap-1.5 section-label">
        <MessageSquare class="w-3.5 h-3.5 opacity-70" />
        Sous-titres
      </span>

      {#if sortedSubs.length === 0}
        <p
          class="font-mono text-[10px] italic"
          style="color: var(--color-subtext2);"
        >
          Aucun sous-titre détecté
        </p>
      {:else}
        <div class="flex flex-wrap gap-1.5">
          {#each sortedSubs as lang}
            {@const active = selSubs.has(lang)}
            <button
              type="button"
              class="lang-btn {active ? 'lang-btn--active' : ''}"
              onclick={() => toggleSub(lang)}
              aria-pressed={active}
            >
              {#if active}<CircleCheck class="w-3 h-3 shrink-0" />{/if}
              <span class="text-[9px] tracking-wider" style="opacity:.7"
                >{lang.toUpperCase()}</span
              >
              <span class="font-mono text-[11px]">{langName(lang)}</span>
            </button>
          {/each}
        </div>
      {/if}
    </section>
  </div>

  <!-- Pied -->
  <div
    class="flex items-center justify-end gap-2 px-4 py-2.5 shrink-0"
    style="border-top: 1px solid var(--color-border);"
  >
    <button type="button" class="btn btn-secondary" onclick={onclose}
      >Annuler</button
    >
    <button type="button" class="btn btn-primary" onclick={apply}
      >Appliquer</button
    >
  </div>
</div>

<style>
  .icon-btn {
    width: 26px;
    height: 26px;
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

  .lang-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 10px;
    border-radius: var(--radius-full);
    border: 1px solid var(--color-border);
    background: var(--color-panel2);
    color: var(--color-subtext);
    font-family: "DM Sans", sans-serif;
    font-size: 12px;
    cursor: pointer;
    transition:
      background 0.1s,
      border-color 0.12s,
      color 0.1s;
    user-select: none;
  }
  .lang-btn:hover {
    background: var(--color-muted);
    border-color: var(--color-subtext2);
    color: var(--color-text);
  }
  .lang-btn--active {
    background: color-mix(in srgb, var(--color-accent) 15%, transparent);
    border-color: color-mix(
      in srgb,
      var(--color-accent) 30%,
      var(--color-border)
    );
    color: var(--color-accent);
    font-weight: 500;
  }
  .lang-btn--active:hover {
    background: color-mix(in srgb, var(--color-accent) 22%, transparent);
    border-color: color-mix(
      in srgb,
      var(--color-accent) 45%,
      var(--color-border)
    );
    color: var(--color-accent);
  }
</style>
