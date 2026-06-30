<script lang="ts">
  import { encoder, LANG_ORDER, langName } from "$lib/stores/encoder.svelte";
  import { CircleCheck } from '@lucide/svelte';

  const { mode }: { mode: "audio" | "sub" } = $props();

  let langs    = $derived(mode === "audio" ? encoder.audioLangs : encoder.subLangs);
  let selected = $derived(mode === "audio" ? encoder.selAudio : encoder.selSubs);

  let sortedLangs = $derived(
    [...langs].sort((a, b) => {
      const ai = LANG_ORDER.indexOf(a);
      const bi = LANG_ORDER.indexOf(b);
      return (ai === -1 ? 999 : ai) - (bi === -1 ? 999 : bi);
    })
  );

  let isEmpty = $derived(sortedLangs.length === 0);

  function toggleLanguage(lang: string) {
    if (mode === "audio") encoder.toggleAudioLang(lang);
    else encoder.toggleSubLang(lang);
  }
</script>

<div class="min-h-[5vh] max-h-[12vh] overflow-y-auto">
  {#if isEmpty}
    <div class="h-full flex items-center justify-center">
      <span class="font-mono text-[10px] italic" style="color: var(--color-subtext);">
        {mode === "audio" ? "Ajoutez des fichiers pour détecter les pistes" : "Aucun sous-titre détecté"}
      </span>
    </div>
  {:else}
    <div class="flex flex-wrap gap-1.5">
      {#each sortedLangs as lang}
        {@const isActive = selected.has(lang)}
        <button
          type="button"
          onclick={() => toggleLanguage(lang)}
          disabled={encoder.encoding}
          class="lang-btn {isActive ? 'lang-btn--active' : ''}"
          aria-label={isActive ? `Désactiver ${langName(lang)}` : `Activer ${langName(lang)}`}
          aria-pressed={isActive}
        >
          {#if isActive}
            <CircleCheck class="w-3 h-3 shrink-0" />
          {/if}
          <span class="text-[9px] tracking-wider" style="opacity: 0.7;">{lang.toUpperCase()}</span>
          <span class="font-mono text-[11px]">{langName(lang)}</span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
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
    transition: background 0.1s, border-color 0.12s, color 0.1s;
    user-select: none;
  }

  .lang-btn:hover:not(:disabled) {
    background: var(--color-muted);
    border-color: var(--color-subtext2);
    color: var(--color-text);
  }

  .lang-btn:disabled {
    opacity: 0.38;
    cursor: not-allowed;
  }

  /* État actif — utilise les variables du CSS (btn-lang-active dans app.css) */
  .lang-btn--active {
    background: color-mix(in srgb, var(--color-accent) 15%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 30%, var(--color-border));
    color: var(--color-accent);
    font-weight: 500;
  }

  .lang-btn--active:hover:not(:disabled) {
    background: color-mix(in srgb, var(--color-accent) 22%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 45%, var(--color-border));
    color: var(--color-accent);
  }
</style>