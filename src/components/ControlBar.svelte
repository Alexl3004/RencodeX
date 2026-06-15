<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";

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
      <svg
        width="12"
        height="12"
        viewBox="0 0 24 24"
        fill="currentColor"
        stroke="none"
        class="shrink-0"
      >
        <polygon points="5 3 19 12 5 21 5 3" />
      </svg>
      LANCER L'ENCODAGE
    {/if}
  </button>

  {#if encoder.encoding}
    <button
      onclick={() => encoder.cancelEncoding()}
      class="btn btn-danger font-mono text-[11px] gap-1.5"
    >
      <svg
        width="10"
        height="10"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2.5"
        aria-hidden="true"
      >
        <rect x="3" y="3" width="18" height="18" rx="2" />
      </svg>
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