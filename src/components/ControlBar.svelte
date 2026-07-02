<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { X, Play, FileDown } from "@lucide/svelte";

  // Fichiers prêts (hors sélection)
  let readyCount = $derived(encoder.files.filter((f) => f.status === "ready").length);

  // Fichiers cibles pour l'encodage selon le mode
  let encodeTargetCount = $derived(
    encoder.encodeSelectionMode && encoder.selectedForEncoding.size > 0
      ? encoder.selectedForEncoding.size
      : readyCount,
  );

  let canEncode = $derived(!encoder.encoding && encodeTargetCount > 0);
</script>

<div
  class="flex items-center gap-2 flex-wrap px-4 py-2.5 border-t border-[var(--color-border)]"
  style="background: var(--color-panel);"
>
  <!-- LANCER -->
  <button
    onclick={() => encoder.startEncoding()}
    disabled={!canEncode}
    class="btn btn-primary gap-2 font-mono text-[11px] px-4 py-1.5"
  >
    {#if encoder.encoding}
      <span class="spinner w-3 h-3 border-2 border-white/30 border-t-white shrink-0 rounded-full animate-spin"></span>
      EN COURS…
    {:else if encoder.encodeSelectionMode && encoder.selectedForEncoding.size > 0}
      <Play height="0.5em" fill="currentColor" stroke="none" class="shrink-0" />
      LANCER ({encoder.selectedForEncoding.size})
    {:else}
      <Play height="0.5em" fill="currentColor" stroke="none" class="shrink-0" />
      LANCER
    {/if}
  </button>

  <!-- ANNULER (encodage ou extraction) -->
  {#if encoder.encoding || encoder.extractingSubs}
    <button
      onclick={() =>
        encoder.encoding
          ? encoder.cancelEncoding()
          : encoder.cancelSubtitleExtraction()}
      class="btn btn-danger font-mono text-[11px] px-4 py-1.5 gap-1.5"
    >
      <X height="1em" /> ANNULER
    </button>
  {/if}

  <!-- ── Zone extraction sous-titres ───────────────────────────────────── -->
  {#if encoder.showExtractButton &&
    !encoder.encoding &&
    encoder.files.some((f) => f.status === "ready" && f.sub_langs.length > 0)}

    <!-- Séparateur visuel -->
    <div class="sep" aria-hidden="true"></div>

    <!-- Bouton EXTRAIRE -->
    <button
      onclick={() => encoder.startSubtitleExtraction()}
      disabled={encoder.extractingSubs ||
        encoder.selSubs.size === 0 ||
        (encoder.extractSelectionMode && encoder.selectedForExtraction.size === 0)}
      class="btn font-mono text-[11px] px-4 py-1.5 gap-1.5"
      class:btn-primary={!encoder.extractingSubs}
      title="Extraire les pistes de sous-titres"
    >
      {#if encoder.extractingSubs}
        <span class="spinner w-3 h-3 border-2 border-white/30 border-t-white shrink-0 rounded-full animate-spin"></span>
        EXTRACTION…
      {:else}
        <FileDown height="0.9em" class="shrink-0" />
        EXTRAIRE{#if encoder.extractSelectionMode && encoder.selectedForExtraction.size > 0}&nbsp;({encoder.selectedForExtraction.size}){/if}
      {/if}
    </button>
  {/if}
</div>

<style>
  .sep {
    width: 1px;
    height: 18px;
    background: var(--color-border);
    flex-shrink: 0;
  }
</style>