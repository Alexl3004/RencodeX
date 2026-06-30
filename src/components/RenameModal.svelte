<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { encoder, buildOutputName, computeTag } from "$lib/stores/encoder.svelte";
  import type { AppFile, CleanedName} from "$lib/stores/encoder.svelte";
  import { Loader, Pencil, RefreshCw, CircleCheck, X } from '@lucide/svelte';


  // ── Props ────────────────────────────────────────────────────────────────────
  type Props = {
    file:     AppFile;
    onclose:  () => void;
    onrename: (newName: string) => void;
  };

  let { file, onclose, onrename }: Props = $props();

  let initFilename  = $derived(file.filename);
  let initOutputExt = $derived(file.output_ext);
  let audio_langs   = $derived(file.audio_langs);
  let sub_langs     = $derived(file.sub_langs);

  let editValue = $state('');
  let loading   = $state(true);
  let cleaned   = $state<CleanedName | null>(null);

  let initAudioLangs = $derived(audio_langs.filter((l: string) => encoder.selAudio.has(l)));
  let initSubLangs   = $derived(sub_langs.filter((l: string) => encoder.selSubs.has(l)));

  // Nom suggéré recalculé localement (comme le reste de l'app) pour respecter
  // le format saison/épisode choisi par l'utilisateur — `cleaned.suggested`
  // (renvoyé par le backend) utilise toujours le format "S01E01" figé.
  let suggestedName = $derived.by(() => {
    if (!cleaned) return "";
    const tag = computeTag(audio_langs, sub_langs, encoder.selAudio, encoder.selSubs);
    return buildOutputName(cleaned, tag, encoder.seasonEpisodeFormat);
  });

  $effect(() => {
    editValue = file.output_name;
  });

  $effect(() => {
    loading = true;
    invoke<CleanedName>("clean_filename", {
      raw:        initFilename,
      audioLangs: initAudioLangs,
      subLangs:   initSubLangs,
    })
      .then((r) => {
        cleaned = r;
        const tag = computeTag(audio_langs, sub_langs, encoder.selAudio, encoder.selSubs);
        if (r) editValue = buildOutputName(r, tag, encoder.seasonEpisodeFormat);
      })
      .catch(() => {})
      .finally(() => { loading = false; });
  });

  function confirm() {
    const v = editValue.trim();
    if (v) onrename(v);
    onclose();
  }

  function applyTag(tag: string) {
    editValue = tag;
  }

  function focusAndSelect(node: HTMLInputElement) {
    node.focus();
    node.select();
  }
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === "Enter")  confirm();
    if (e.key === "Escape") onclose();
  }}
/>

<!-- Overlay backdrop -->
<div
  class="modal-backdrop"
  role="dialog"
  aria-modal="true"
  aria-label="Renommer le fichier"
  tabindex="-1"
  onclick={(e) => { if (e.target === e.currentTarget) onclose(); }}
  onkeydown={(e) => { if (e.key === 'Escape') onclose(); }}
>
  <!-- Dialog box -->
  <div class="modal-box" style="width: 540px; max-width: 95vw;">

    <!-- Header -->
    <div class="modal-header">
      <div class="flex items-center gap-2">
        <Pencil class="w-4 h-4" style="color: var(--color-accent);" />
        <span class="font-mono text-[12px] font-semibold" style="color: var(--color-text); text-transform: uppercase; letter-spacing: 0.08em;">
          Renommer le fichier
        </span>
      </div>
      <button onclick={onclose} class="modal-close-btn" aria-label="Fermer"><X class="w-4 h-4" /></button>
    </div>

    <!-- Body -->
    <div class="modal-body space-y-4">
      <!-- Fichier source -->
      <div class="space-y-1">
        <div class="section-label">Fichier source</div>
        <div
          class="font-mono text-[11px] truncate px-3 py-2 rounded-[var(--radius-sm)]"
          style="background: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-subtext);"
          title={initFilename}
        >
          {initFilename}
        </div>
      </div>

      <!-- Nouveau nom -->
      <div class="space-y-1.5">
        <div class="flex items-center justify-between">
          <div class="section-label">Nouveau nom</div>
          {#if suggestedName && editValue !== suggestedName}
            <button
              onclick={() => applyTag(suggestedName)}
              class="flex items-center gap-1 text-[10px] transition-colors"
              style="color: var(--color-accent);"
            >
              <RefreshCw class="w-3 h-3" />
              Réinitialiser au nom suggéré
            </button>
          {/if}
        </div>
        <input
          type="text"
          bind:value={editValue}
          placeholder="Nom du fichier de sortie…"
          use:focusAndSelect
          class="w-full px-3 py-2 text-[11px] rounded-[var(--radius-sm)]"
          style="background: var(--color-surface); border: 1px solid var(--color-accent); color: var(--color-text); font-family: 'Geist Mono', monospace; outline: none;"
        />
      </div>

      <!-- Suggestion -->
      {#if loading}
        <div class="flex items-center gap-2 text-[11px]" style="color: var(--color-subtext);">
          <Loader size="4" />
          Analyse du nom en cours…
        </div>
      {:else if suggestedName}
        <button
          onclick={() => applyTag(suggestedName)}
          class="w-full text-left text-[11px] px-3 py-2 rounded-[var(--radius-sm)] font-mono transition-colors"
          style="border: 1px solid {editValue === suggestedName ? 'color-mix(in srgb, var(--color-accent) 40%, transparent)' : 'var(--color-border)'}; background: {editValue === suggestedName ? 'color-mix(in srgb, var(--color-accent) 10%, transparent)' : 'var(--color-surface)'}; color: {editValue === suggestedName ? 'var(--color-accent)' : 'var(--color-subtext)'};"
          title="Appliquer le nom suggéré"
        >
          <span class="block mb-0.5 text-[9px]" style="color: var(--color-subtext2);">Nom suggéré</span>
          {suggestedName}
          {#if editValue === suggestedName}
            <span class="ml-2 inline-flex items-center gap-0.5 text-[9px]" style="color: var(--color-success);"><CircleCheck class="w-3 h-3" />appliqué</span>
          {/if}
        </button>
      {/if}

      <!-- Aperçu -->
      <div class="space-y-1">
        <div class="section-label">Aperçu</div>
        <div
          class="text-[11px] font-mono px-3 py-2 rounded-[var(--radius-sm)] truncate"
          style="background: var(--color-surface); border: 1px solid var(--color-border);"
          title="{editValue}{initOutputExt}"
        >
          <span style="color: var(--color-text);">{editValue || "…"}</span><span style="color: var(--color-subtext);">{initOutputExt}</span>
        </div>
      </div>
    </div>

    <!-- Footer -->
    <div class="modal-footer">
      <button onclick={onclose} class="btn btn-secondary font-mono text-[11px]">Annuler</button>
      <button
        onclick={confirm}
        disabled={!editValue.trim()}
        class="btn btn-primary font-mono text-[11px] flex items-center gap-1.5"
      >
        <CircleCheck class="w-3.5 h-3.5" />
        Renommer
      </button>
    </div>
  </div>
</div>

<style>
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