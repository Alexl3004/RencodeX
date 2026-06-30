<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import LangSelector from "$components/LangSelector.svelte";
  import { X, Captions, Headphones, MessageSquare } from '@lucide/svelte';

  let open = $state(false);

  let totalLangs  = $derived(encoder.audioLangs.size + encoder.subLangs.size);
  let activeLangs = $derived(
  [...encoder.selAudio].filter(l => encoder.audioLangs.has(l)).length +
  [...encoder.selSubs].filter(l => encoder.subLangs.has(l)).length
);


  function toggle() {
    open = !open;
  }
  function close() {
    open = false;
  }
</script>

<div class="relative inline-flex">
  <!-- Bouton déclencheur -->
  <button
    type="button"
    onclick={toggle}
    class="relative inline-flex items-center justify-center w-7 h-7 rounded-[var(--radius-xs)] border border-transparent bg-transparent text-[var(--color-subtext)] cursor-pointer transition-colors duration-100 flex-shrink-0
           hover:bg-[var(--color-panel2)] hover:border-[var(--color-border)] hover:text-[var(--color-text)]
           active:translate-y-px
           {open ? 'bg-[color-mix(in_srgb,var(--color-accent)_10%,transparent)] border-[color-mix(in_srgb,var(--color-accent)_25%,var(--color-border))] text-[var(--color-accent)]' : ''}"
    aria-label="Pistes audio et sous-titres"
    aria-pressed={open}
    title="Pistes audio et sous-titres"
  >
    <Captions class="w-[18px] h-[18px]" />
    {#if totalLangs > 0}
      <span class="absolute -top-1 -right-1 min-w-4 h-4 px-1 rounded-full bg-[var(--color-accent)] text-white font-mono text-[8px] font-semibold leading-4 text-center tracking-[0.3px] shadow-[0_2px_6px_rgba(0,0,0,0.25)]">
        {activeLangs}
      </span>
    {/if}
  </button>

  {#if open}
    <!-- Backdrop (sans flou) -->
    <div
      class="fixed inset-0 z-[9970] bg-black/10 animate-fade-in"
      role="presentation"
      onclick={close}
    ></div>

    <!-- Popover -->
    <div
      class="absolute top-full right-0 z-[9971] w-[400px] max-w-[calc(100vw-24px)] bg-[var(--color-panel)] border border-[var(--color-border)] rounded-[var(--radius-lg)] shadow-[0_16px_40px_rgba(0,0,0,0.45)] flex flex-col overflow-hidden animate-slide-down origin-top-right"
      role="dialog"
      aria-modal="true"
      aria-label="Pistes audio et sous-titres"
      tabindex="-1"
    >
      <!-- En‑tête -->
      <header class="flex items-center justify-between px-3 py-2 border-b border-[var(--color-border)] bg-[var(--color-panel)] select-none">
        <div class="flex items-center gap-2.5">
          <div class="w-[3px] h-4 rounded-sm bg-[var(--color-accent)] shrink-0"></div>
          <span class="text-[11px] font-semibold uppercase tracking-[0.4px] text-[var(--color-subtext)]">Pistes &amp; sous-titres</span>
        </div>
        <div class="flex items-center gap-1">
          <button onclick={close} class="flex items-center justify-center w-6 h-6 border-none bg-transparent text-[var(--color-subtext)] cursor-pointer rounded-[var(--radius-xs)] hover:bg-[var(--color-panel2)] hover:text-[var(--color-text)] transition-colors" title="Fermer" aria-label="Fermer">
            <X size={16} />
          </button>
        </div>
      </header>

      <!-- Corps -->
      <div class="px-3.5 py-3 flex flex-col gap-1.5 max-h-[55vh] overflow-y-auto scroll-smooth
                  [&::-webkit-scrollbar]:w-1 [&::-webkit-scrollbar-track]:bg-transparent [&::-webkit-scrollbar-thumb]:bg-[var(--color-border)] [&::-webkit-scrollbar-thumb]:rounded-full">
        <!-- Section Audio -->
        <section class="flex flex-col gap-1">
          <div class="flex items-center justify-between py-0.5">
            <span class="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-[0.4px] text-[var(--color-subtext)]">
              <Headphones class="w-3.5 h-3.5 opacity-80" />
              Pistes audio
              <span class="inline-flex items-center justify-center min-w-[18px] h-[18px] px-1.5 rounded-full bg-[color-mix(in_srgb,var(--color-accent)_12%,transparent)] text-[var(--color-accent)] text-[9px] font-mono font-semibold leading-[18px] ml-0.5">
                {[...encoder.selAudio].filter(l => encoder.audioLangs.has(l)).length}
              </span>
            </span>
          </div>
          <div class="py-0.5">
            <LangSelector mode="audio" />
          </div>
        </section>

        <div class="h-px bg-[var(--color-border)] my-1 opacity-50" role="separator"></div>

        <!-- Section Sous‑titres -->
        <section class="flex flex-col gap-1">
          <div class="flex items-center justify-between py-0.5">
            <span class="flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-[0.4px] text-[var(--color-subtext)]">
              <MessageSquare class="w-3.5 h-3.5 opacity-80" />
              Sous-titres
              <span class="inline-flex items-center justify-center min-w-[18px] h-[18px] px-1.5 rounded-full bg-[color-mix(in_srgb,var(--color-accent)_12%,transparent)] text-[var(--color-accent)] text-[9px] font-mono font-semibold leading-[18px] ml-0.5">
                {[...encoder.selSubs].filter(l => encoder.subLangs.has(l)).length}
              </span>
            </span>
          </div>
          <div class="py-0.5">
            <LangSelector mode="sub" />
          </div>
        </section>
      </div>
    </div>
  {/if}
</div>

<style>
  @keyframes fade-in {
    from { opacity: 0; }
    to   { opacity: 1; }
  }
  @keyframes slide-down {
    from {
      opacity: 0;
      transform: translateY(-12px) scale(0.96);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }
  .animate-fade-in {
    animation: fade-in 0.2s ease;
  }
  .animate-slide-down {
    animation: slide-down 0.2s cubic-bezier(0.22, 1, 0.36, 1);
  }
</style>