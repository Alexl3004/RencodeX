<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { encoder } from "$lib/stores/encoder.svelte";
  import type { AppFile, CleanedName } from "$lib/stores/encoder.svelte";

  // ── Props ────────────────────────────────────────────────────────────────────
  type Props = {
    file:     AppFile;
    onclose:  () => void;
    onrename: (newName: string) => void;
  };

  // 1. Réception de la prop réactive à plat
  let { file, onclose, onrename }: Props = $props();

  // 2. Propriétés réactives dérivées de l'objet "file"
  let initFilename  = $derived(file.filename);
  let initOutputExt = $derived(file.output_ext);
  let audio_langs   = $derived(file.audio_langs);
  let sub_langs     = $derived(file.sub_langs);
  
  // 3. State local modifiable de la Modal
  // Utiliser une valeur dérivée pour l'initialisation
  let editValue = $state('');
  let loading   = $state(true);
  let cleaned   = $state<CleanedName | null>(null);

  // 4. Filtres réactifs basés sur les changements globaux de sélection (stores)
  let initAudioLangs = $derived(audio_langs.filter((l: string) => encoder.selAudio.has(l)));
  let initSubLangs   = $derived(sub_langs.filter((l: string) => encoder.selSubs.has(l)));

  // ── Initialiser editValue quand le composant s'ouvre ─────────────────────────
  $effect(() => {
    editValue = file.output_name;
  });

  // ── Charger le nom suggéré de manière réactive ───────────────────────────────
  $effect(() => {
    loading = true;
    invoke<CleanedName>("clean_filename", {
      raw:        initFilename,
      audioLangs: initAudioLangs,
      subLangs:   initSubLangs,
    })
      .then((r) => {
        cleaned = r;
        if (r?.suggested) editValue = r.suggested;
      })
      .catch(() => {})
      .finally(() => {
        loading = false;
      });
  });

  // ── Actions ──────────────────────────────────────────────────────────────────
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
    if (e.key === "Enter")   confirm();
    if (e.key === "Escape") onclose();
  }}
/>

<div class="fixed inset-0 z-50 flex items-center justify-center">
  <button
    type="button"
    class="absolute inset-0 bg-black/50 backdrop-blur-[2px] cursor-default w-full h-full"
    onclick={onclose}
    tabindex="-1"
    aria-label="Fermer le popup"
  ></button>

  <div
    class="relative w-[560px] max-w-[92vw] bg-[var(--color-panel)] border border-[var(--color-border)] rounded-[2px]
           shadow-2xl flex flex-col overflow-hidden"
    tabindex="-1"
  >
    <div class="flex items-center justify-between px-5 py-3.5 border-b border-[var(--color-border)] shrink-0">
      <div class="flex items-center gap-2">
        <svg
          width="13"
          height="13"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          class="text-[var(--color-accent)]"
        >
          <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
          <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
        </svg>
        <span class="text-sm font-medium text-[var(--color-text)]">Renommer le fichier</span>
      </div>
      <button
        onclick={onclose}
        class="text-[var(--color-subtext)] hover:text-[var(--color-text)] transition-colors p-1"
        title="Fermer (Échap)"
      >
        <svg
          width="13"
          height="13"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
        >
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </div>

    <div class="px-5 py-4 space-y-4">
      <div class="space-y-1">
        <div class="text-[10px] text-[var(--color-subtext)] uppercase tracking-wider">
          Fichier source
        </div>
        <div
          class="text-xs text-[var(--color-subtext)] font-mono bg-[var(--color-surface)] border border-[var(--color-border)]
                 rounded-[2px] px-3 py-2 truncate"
          title={initFilename}
        >
          {initFilename}
        </div>
      </div>

      <div class="space-y-1.5">
        <div class="flex items-center justify-between">
          <div class="text-[10px] text-[var(--color-subtext)] uppercase tracking-wider">
            Nouveau nom
          </div>
          {#if cleaned?.suggested && editValue !== cleaned.suggested}
            <button
              onclick={() => applyTag(cleaned!.suggested)}
              class="text-[10px] text-[var(--color-accent)] hover:text-[var(--color-accent)]/70 transition-colors
                     flex items-center gap-1"
            >
              <svg
                width="9"
                height="9"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.5"
              >
                <polyline points="1 4 1 10 7 10" />
                <path d="M3.51 15a9 9 0 1 0 .49-3.47" />
              </svg>
              Réinitialiser au nom suggéré
            </button>
          {/if}
        </div>
        <input
          type="text"
          bind:value={editValue}
          class="w-full bg-[var(--color-surface)] border border-[var(--color-accent)] rounded-[2px] px-3 py-2 text-xs
                 text-[var(--color-text)] outline-none font-mono transition-colors
                 focus:ring-1 focus:ring-[var(--color-accent)]/20"
          placeholder="Nom du fichier de sortie…"
          use:focusAndSelect
        />
      </div>

      {#if loading}
        <div class="flex items-center gap-2 text-xs text-[var(--color-subtext)]">
          <span class="inline-block w-3 h-3 border-2 border-subtext/30 border-t-subtext
                       rounded-full animate-spin"></span>
          Analyse du nom en cours…
        </div>
      {:else if cleaned?.suggested}
        <button
          onclick={() => applyTag(cleaned!.suggested)}
          class="w-full text-left text-[11px] px-3 py-2 border rounded-[2px] font-mono
                 transition-colors hover:border-[var(--color-accent)]
                 {editValue === cleaned.suggested
            ? 'border-[var(--color-accent)]/40 bg-[var(--color-accent)]/10 text-[var(--color-accent)]'
            : 'border-[var(--color-border)] bg-[var(--color-surface)] text-[var(--color-subtext)] hover:text-[var(--color-accent)]'}"
          title="Appliquer le nom suggéré"
        >
          <span class="text-[9px] text-[var(--color-subtext2)] block mb-0.5">Nom suggéré</span>
          {cleaned.suggested}
          {#if editValue === cleaned.suggested}
            <span class="ml-2 text-[9px] text-[var(--color-success)]">✓ appliqué</span>
          {/if}
        </button>
      {/if}

      <div class="space-y-1">
        <div class="text-[10px] text-[var(--color-subtext)] uppercase tracking-wider">
          Aperçu
        </div>
        <div
          class="text-xs font-mono bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[2px] px-3 py-2 truncate"
          title="{editValue}{initOutputExt}"
        >
          <span class="text-[var(--color-text)]">{editValue || "…"}</span><span class="text-[var(--color-subtext)]">{initOutputExt}</span>
        </div>
      </div>
    </div>

    <div class="flex items-center justify-end gap-2 px-5 py-3.5 border-t border-[var(--color-border)] shrink-0">
      <button onclick={onclose} class="btn text-xs"> Annuler </button>
      <button
        onclick={confirm}
        disabled={!editValue.trim()}
        class="btn btn-primary text-xs disabled:opacity-40"
      >
        <svg
          width="11"
          height="11"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
        >
          <polyline points="20 6 9 17 4 12" />
        </svg>
        Renommer
      </button>
    </div>
  </div>
</div>