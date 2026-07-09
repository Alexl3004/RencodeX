<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import LangSelector from "$components/LangSelector.svelte";
  import { X, Captions, Headphones, MessageSquare, Users } from '@lucide/svelte';

  let open = $state(false);

  let audioActive = $derived([...encoder.selAudio].filter(l => encoder.audioLangs.has(l)).length);
  let subActive   = $derived([...encoder.selSubs].filter(l => encoder.subLangs.has(l)).length);
  let totalActive = $derived(audioActive + subActive);
  let totalLangs  = $derived(encoder.audioLangs.size + encoder.subLangs.size);

  let hasOverrides = $derived(
    encoder.files.some(f => encoder.fileSelAudio.has(f.path) || encoder.fileSelSubs.has(f.path))
  );

  function applyToAll() {
    for (const f of encoder.files) {
      encoder.setFileLangSel(
        f.path,
        new Set([...encoder.selAudio].filter(l => f.audio_langs.includes(l))),
        new Set([...encoder.selSubs].filter(l => f.sub_langs.includes(l))),
      );
    }
  }

  function resetAll() {
    for (const f of encoder.files) {
      encoder.clearFileLangSel(f.path);
    }
  }

  // ── Positionnement automatique ──
  let triggerEl = $state<HTMLButtonElement | null>(null);
  let popoverEl = $state<HTMLDivElement | null>(null);

  let popoverStyle = $derived.by(() => {
    if (!triggerEl || !popoverEl) return { left: 0, top: 0 };
    const rect = triggerEl.getBoundingClientRect();
    const popoverRect = popoverEl.getBoundingClientRect();
    const viewWidth = window.innerWidth;
    const viewHeight = window.innerHeight;

    let left = rect.left;
    let top = rect.bottom + 6;

    if (left + popoverRect.width > viewWidth - 12) {
      left = viewWidth - popoverRect.width - 12;
    }
    if (left < 12) left = 12;

    if (top + popoverRect.height > viewHeight - 12) {
      top = rect.top - popoverRect.height - 6;
      if (top < 12) top = 12;
    }

    return { left, top };
  });
</script>

<div class="relative inline-flex">
  <!-- Bouton déclencheur -->
  <button
    bind:this={triggerEl}
    type="button"
    onclick={() => (open = !open)}
    class="trigger"
    class:trigger--active={open}
    aria-label="Pistes audio et sous-titres"
    aria-pressed={open}
    title="Pistes audio et sous-titres"
  >
    <Captions class="w-[18px] h-[18px]" />
    {#if totalLangs > 0}
      <span class="trigger-badge">{totalActive}</span>
    {/if}
  </button>

  {#if open}
    <!-- Overlay transparent pour fermer au clic en dehors -->
    <div
      class="fixed inset-0 z-[9970]"
      role="presentation"
      onclick={() => (open = false)}
      oncontextmenu={(e) => { e.preventDefault(); open = false; }}
    ></div>

    <!-- Popover -->
    <div
      bind:this={popoverEl}
      class="popover"
      style="left:{popoverStyle.left}px; top:{popoverStyle.top}px;"
      role="dialog"
      aria-modal="true"
      aria-label="Pistes &amp; sous-titres globaux"
    >
      <!-- Header -->
      <header class="popover-header">
        <div class="flex items-center gap-2.5">
          <div class="header-accent"></div>
          <div class="flex flex-col gap-0.5">
            <span class="header-title">Pistes &amp; sous-titres</span>
            <span class="header-sub">Sélection globale — tous les fichiers</span>
          </div>
        </div>
        <button
          onclick={() => (open = false)}
          class="close-btn"
          title="Fermer"
          aria-label="Fermer"
        >
          <X class="w-4 h-4" />
        </button>
      </header>

      <!-- Corps -->
      <div class="popover-body">

        <!-- Section Audio -->
        <section class="lang-section">
          <div class="section-header">
            <span class="section-title">
              <Headphones class="w-3.5 h-3.5 opacity-70" />
              Pistes audio
            </span>
            <span class="section-count section-count--audio">{audioActive} actif{audioActive > 1 ? 's' : ''}</span>
          </div>
          <LangSelector mode="audio" />
        </section>

        <div class="divider" role="separator"></div>

        <!-- Section Sous-titres -->
        <section class="lang-section">
          <div class="section-header">
            <span class="section-title">
              <MessageSquare class="w-3.5 h-3.5 opacity-70" />
              Sous-titres
            </span>
            <span class="section-count section-count--sub">{subActive} actif{subActive > 1 ? 's' : ''}</span>
          </div>
          <LangSelector mode="sub" />
        </section>

      </div>

      <!-- Footer -->
      {#if encoder.files.length > 0}
        <footer class="popover-footer">
          <div class="footer-info">
            <Users class="w-3.5 h-3.5 shrink-0 opacity-60" />
            <span>{encoder.files.length} fichier{encoder.files.length > 1 ? 's' : ''}</span>
          </div>
          <div class="footer-actions">
            {#if hasOverrides}
              <button
                type="button"
                class="footer-btn footer-btn--reset"
                onclick={resetAll}
                title="Réinitialiser les overrides par fichier"
              >
                Reset overrides
              </button>
            {/if}
            <button
              type="button"
              class="footer-btn footer-btn--apply"
              onclick={applyToAll}
              disabled={encoder.files.length === 0}
              title="Appliquer cette sélection à tous les fichiers"
            >
              <Captions class="w-3.5 h-3.5 shrink-0" />
              Appliquer à tous
            </button>
          </div>
        </footer>
      {/if}
    </div>
  {/if}
</div>

<style>
  /* ── Bouton déclencheur ────────────────────────────────────────────────── */
  .trigger {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px; height: 28px;
    border-radius: var(--radius-xs);
    border: 1px solid transparent;
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    transition: background 0.1s, border-color 0.1s, color 0.1s;
    flex-shrink: 0;
  }
  .trigger:hover {
    background: var(--color-panel2);
    border-color: var(--color-border);
    color: var(--color-text);
  }
  .trigger--active {
    background: color-mix(in srgb, var(--color-accent) 10%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 25%, var(--color-border));
    color: var(--color-accent);
  }
  .trigger-badge {
    position: absolute;
    top: -4px; right: -4px;
    min-width: 16px; height: 16px;
    padding: 0 3px;
    border-radius: 999px;
    background: var(--color-accent);
    color: #fff;
    font-family: "Geist Mono", monospace;
    font-size: 8px;
    font-weight: 700;
    line-height: 16px;
    text-align: center;
    box-shadow: 0 2px 6px rgba(0,0,0,0.3);
  }

  /* ── Popover ───────────────────────────────────────────────────────────── */
  .popover {
    position: fixed;
    z-index: 9971;
    width: 420px;
    max-width: calc(100vw - 24px);
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    box-shadow: 0 12px 40px rgba(0,0,0,0.5);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    max-height: 80vh;
  }

  /* ── Header ────────────────────────────────────────────────────────────── */
  .popover-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 14px 10px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
    background: var(--color-surface);
  }
  .header-accent {
    width: 3px; height: 18px;
    border-radius: 2px;
    background: var(--color-accent);
    flex-shrink: 0;
  }
  .header-title {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--color-text);
    line-height: 1.2;
  }
  .header-sub {
    font-family: "Geist Mono", monospace;
    font-size: 8px;
    color: var(--color-subtext);
    opacity: 0.7;
  }
  .close-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 26px; height: 26px;
    border-radius: var(--radius-xs);
    border: 1px solid transparent;
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    transition: background 0.1s, color 0.1s, border-color 0.1s;
    flex-shrink: 0;
  }
  .close-btn:hover {
    background: var(--color-panel2);
    border-color: var(--color-border);
    color: var(--color-text);
  }

  /* ── Corps ─────────────────────────────────────────────────────────────── */
  .popover-body {
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-height: 55vh;
    overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: var(--color-border) transparent;
  }

  /* ── Sections ──────────────────────────────────────────────────────────── */
  .lang-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .section-title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color-subtext);
  }
  .section-count {
    font-family: "Geist Mono", monospace;
    font-size: 8px;
    font-weight: 700;
    padding: 1px 7px;
    border-radius: 999px;
  }
  .section-count--audio {
    background: color-mix(in srgb, var(--color-accent) 12%, transparent);
    color: var(--color-accent);
    border: 1px solid color-mix(in srgb, var(--color-accent) 22%, transparent);
  }
  .section-count--sub {
    background: color-mix(in srgb, var(--color-success) 10%, transparent);
    color: var(--color-success);
    border: 1px solid color-mix(in srgb, var(--color-success) 20%, transparent);
  }
  .divider {
    height: 1px;
    background: var(--color-border);
    opacity: 0.4;
    margin: 2px 0;
  }

  /* ── Footer ────────────────────────────────────────────────────────────── */
  .popover-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 8px 14px;
    border-top: 1px solid var(--color-border);
    flex-shrink: 0;
    background: var(--color-surface);
  }
  .footer-info {
    display: flex;
    align-items: center;
    gap: 5px;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext);
    opacity: 0.7;
  }
  .footer-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .footer-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.04em;
    padding: 5px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    transition: all 0.12s;
    white-space: nowrap;
  }
  .footer-btn--reset:hover {
    border-color: color-mix(in srgb, var(--color-danger) 60%, var(--color-border));
    color: var(--color-danger);
    background: color-mix(in srgb, var(--color-danger) 8%, transparent);
  }
  .footer-btn--apply {
    background: var(--color-accent);
    border-color: var(--color-accent);
    color: #fff;
  }
  .footer-btn--apply:hover:not(:disabled) { opacity: 0.88; }
  .footer-btn--apply:disabled { opacity: 0.4; cursor: not-allowed; }
</style>