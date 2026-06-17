<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import PlayIcon from "@iconify-svelte/lucide/play";
  import XIcon from '@iconify-svelte/lucide/x';

  function handleReset() {
    encoder.resetToDefault();
    setTimeout(() => window.dispatchEvent(new Event("resize")), 10);
  }
</script>

<div class="flex items-center gap-2 flex-wrap px-4 py-5">
  <button
    onclick={() => encoder.startEncoding()}
    disabled={encoder.encoding ||
      encoder.files.filter((f) => f.status === "ready").length === 0}
    class="btn btn-primary gap-2 font-mono text-[12px] px-5 py-2"
  >
    {#if encoder.encoding}
      <span
        class="spinner w-3 h-3 border-2 border-white/30 border-t-white shrink-0 rounded-full animate-spin"
      ></span>
      ENCODAGE EN COURS…
    {:else}
      <PlayIcon height="1em" fill="currentColor" stroke="none" class="shrink-0" />
      LANCER L'ENCODAGE
    {/if}
  </button>

  {#if encoder.encoding}
    <button
      onclick={() => encoder.cancelEncoding()}
      class="btn btn-danger font-mono text-[11px] gap-1.5"
    >
      <XIcon height="1em" aria-hidden="true" />
      ANNULER
    </button>
  {/if}

  <button
    onclick={handleReset}
    disabled={encoder.encoding}
    class="btn font-mono text-[11px] ml-auto"
  >
    RÉINITIALISER
  </button>
</div>