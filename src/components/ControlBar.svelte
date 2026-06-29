<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { X, Play } from '@lucide/svelte';

  function handleReset() {
    encoder.resetToDefault();
    setTimeout(() => window.dispatchEvent(new Event("resize")), 10);
  }
</script>

<div class="flex items-center gap-2 flex-wrap px-4 py-2.5 border-t border-[var(--color-border)]" style="background: var(--color-panel);">
  <button
    onclick={() => encoder.startEncoding()}
    disabled={encoder.encoding ||
      encoder.files.filter((f) => f.status === "ready").length === 0}
    class="btn btn-primary gap-2 font-mono text-[11px] px-4 py-1.5"
  >
    {#if encoder.encoding}
      <span class="spinner w-3 h-3 border-2 border-white/30 border-t-white shrink-0 rounded-full animate-spin"></span>
      EN COURS…
    {:else}
      <Play height="0.5em" fill="currentColor" stroke="none" class="shrink-0" />
      LANCER
    {/if}
  </button>

  {#if encoder.encoding}
    <button
      onclick={() => encoder.cancelEncoding()}
      class="btn btn-danger font-mono text-[11px] px-4 py-1.5 gap-1.5"
    >
      <X height="1em" aria-hidden="true" />
      ANNULER
    </button>
  {/if}

  <button
    onclick={handleReset}
    disabled={encoder.encoding}
    class="btn font-mono text-[11px] px-4 py-1.5 ml-auto"
  >
    RÉINITIALISER
  </button>
</div>