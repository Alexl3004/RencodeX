<script lang="ts">
  import { encoder, LANG_ORDER, langName } from "$lib/stores/encoder.svelte";
  import CheckIcon from "@iconify-svelte/lucide/check";

  const { mode }: { mode: "audio" | "sub" } = $props();

  // État dérivé simple
  let langs = $derived(mode === "audio" ? encoder.audioLangs : encoder.subLangs);
  let selected = $derived(mode === "audio" ? encoder.selAudio : encoder.selSubs);
  
  // Langues triées
  let sortedLangs = $derived(
    [...langs].sort((a, b) => {
      const ai = LANG_ORDER.indexOf(a);
      const bi = LANG_ORDER.indexOf(b);
      return (ai === -1 ? 999 : ai) - (bi === -1 ? 999 : bi);
    })
  );

  // États vides
  let isEmpty = $derived(sortedLangs.length === 0);

  // Fonctions utilitaires
  function toggleLanguage(lang: string) {
    if (mode === "audio") {
      encoder.toggleAudioLang(lang);
    } else {
      encoder.toggleSubLang(lang);
    }
  }
</script>

<div class="min-h-[5vh] max-h-[12vh] overflow-y-auto">
  {#if isEmpty}
    <div class="h-full flex items-center justify-center">
      <span class="text-[10px] font-mono text-[var(--color-subtext)] italic">
        {mode === "audio"
          ? "Ajoutez des fichiers pour détecter les pistes"
          : "Aucun sous-titre détecté"}
      </span>
    </div>
  {:else}
    <div class="flex flex-wrap gap-1.5">
      {#each sortedLangs as lang}
        {@const isActive = selected.has(lang)}
        <button
          onclick={() => toggleLanguage(lang)}
          disabled={encoder.encoding}
          class="btn font-mono text-[11px] px-3 py-1.5 gap-1.5
                 {isActive ? 'btn-lang-active' : 'btn-secondary'}"
          aria-label={isActive ? `Désactiver ${langName(lang)}` : `Activer ${langName(lang)}`}
        >
          {#if isActive}
            <CheckIcon height="1em" class="shrink-0" />
          {/if}
          <span class="opacity-70 text-[9px] tracking-wider">{lang.toUpperCase()}</span>
          <span>{langName(lang)}</span>
        </button>
      {/each}
    </div>
  {/if}
</div>