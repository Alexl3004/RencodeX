<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import type { VideoMode } from "$lib/stores/types";

  let videoMode = $derived(encoder.videoMode);
</script>

<section class="tab">
  <header class="tab-header">
    <h2 class="tab-title">Vidéo</h2>
    <p class="tab-desc">
      Choisissez entre ré-encoder le flux vidéo ou le copier tel quel.
    </p>
  </header>

  <div class="mode-grid">
    <!-- Encoder -->
    <button
      type="button"
      class="mode-card {videoMode === 'encode' ? 'mode-card--active' : ''}"
      onclick={() => encoder.setVideoMode("encode")}
    >
      <div class="mode-icon" aria-hidden="true">
        <svg width="18" height="18" viewBox="0 0 18 18" fill="none">
          <rect x="1" y="3" width="16" height="12" rx="2" stroke="currentColor" stroke-width="1.5"/>
          <path d="M7 6.5L12 9L7 11.5V6.5Z" fill="currentColor"/>
        </svg>
      </div>
      <div class="mode-body">
        <span class="mode-name">Encoder</span>
        <p class="mode-desc">
          Ré-encode en H.265 via NVENC. Réduit la taille avec un contrôle fin sur la qualité.
        </p>
        <div class="mode-tags">
          <span class="tag">hevc_nvenc</span>
          <span class="tag">CRF actif</span>
          <span class="tag">GPU</span>
        </div>
      </div>
    </button>

    <!-- Copier -->
    <button
      type="button"
      class="mode-card {videoMode === 'copy' ? 'mode-card--active' : ''}"
      onclick={() => encoder.setVideoMode("copy")}
    >
      <div class="mode-icon" aria-hidden="true">
        <svg width="18" height="18" viewBox="0 0 18 18" fill="none">
          <rect x="1" y="3" width="16" height="12" rx="2" stroke="currentColor" stroke-width="1.5"/>
          <path d="M5 9H13M10 6L13 9L10 12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </div>
      <div class="mode-body">
        <span class="mode-name">Copier</span>
        <p class="mode-desc">
          Copie le flux vidéo sans ré-encoder. Remuxage instantané, qualité préservée à l'identique.
        </p>
        <div class="mode-tags">
          <span class="tag">-c:v copy</span>
          <span class="tag">Sans perte</span>
          <span class="tag">Instantané</span>
        </div>
      </div>
    </button>
  </div>

  <!-- Callout contextuel -->
  {#if videoMode === "copy"}
    <div class="callout callout--info">
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none" class="callout-icon" aria-hidden="true">
        <circle cx="7" cy="7" r="6" stroke="currentColor" stroke-width="1.4"/>
        <path d="M7 6V10M7 4.5V4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
      </svg>
      <div>
        <strong>Mode copie actif</strong> — CRF, preset, AQ et multipass sont ignorés.
        L'audio et les sous-titres restent traités normalement.
      </div>
    </div>
  {:else}
    <div class="callout">
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none" class="callout-icon" aria-hidden="true">
        <circle cx="7" cy="7" r="6" stroke="currentColor" stroke-width="1.4"/>
        <path d="M5 7.5L6.5 9L9 5.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
      <div>
        <strong>Mode encodage actif</strong> — Configurez CRF, preset et AQ dans l'onglet
        <strong>Préréglages</strong>.
      </div>
    </div>
  {/if}
</section>

<style>
  .tab {
    padding: 24px 28px;
    max-width: 580px;
  }

  .tab-header {
    margin-bottom: 20px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--color-border);
  }
  .tab-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--color-text);
    letter-spacing: -0.02em;
    margin: 0 0 4px;
  }
  .tab-desc {
    font-size: 12px;
    color: var(--color-subtext);
    margin: 0;
  }

  /* ── Cartes de mode ─────────────────────────────────────────────────────── */
  .mode-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
    margin-bottom: 16px;
  }

  .mode-card {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    text-align: left;
    transition: border-color 0.12s, background 0.12s;
  }
  .mode-card:hover { border-color: var(--color-subtext2); }
  .mode-card--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 7%, var(--color-surface));
  }

  .mode-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: var(--radius-xs);
    border: 1px solid var(--color-border);
    background: var(--color-panel);
    color: var(--color-subtext);
    flex-shrink: 0;
    transition: color 0.12s, border-color 0.12s, background 0.12s;
  }
  .mode-card--active .mode-icon {
    color: var(--color-accent);
    border-color: color-mix(in srgb, var(--color-accent) 30%, var(--color-border));
    background: color-mix(in srgb, var(--color-accent) 10%, var(--color-panel));
  }

  .mode-body {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .mode-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text);
    letter-spacing: -0.01em;
    transition: color 0.12s;
  }
  .mode-card--active .mode-name { color: var(--color-accent); }
  .mode-desc {
    font-size: 11px;
    color: var(--color-subtext);
    line-height: 1.55;
    margin: 0;
  }
  .mode-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .tag {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    padding: 2px 6px;
    border-radius: 3px;
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    color: var(--color-subtext2);
    transition: color 0.12s, border-color 0.12s, background 0.12s;
  }
  .mode-card--active .tag {
    border-color: color-mix(in srgb, var(--color-accent) 22%, var(--color-border));
    color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-panel));
  }

  /* ── Callout ────────────────────────────────────────────────────────────── */
  .callout {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 12px 14px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    font-size: 11px;
    color: var(--color-subtext);
    line-height: 1.55;
    animation: fade-in 0.14s ease;
  }
  .callout strong { color: var(--color-text); font-weight: 600; }
  .callout--info {
    background: color-mix(in srgb, var(--color-accent) 6%, var(--color-surface));
    border-color: color-mix(in srgb, var(--color-accent) 22%, var(--color-border));
    color: color-mix(in srgb, var(--color-accent) 60%, var(--color-subtext));
  }
  .callout--info strong { color: var(--color-accent); }
  .callout-icon {
    flex-shrink: 0;
    margin-top: 1px;
    color: var(--color-subtext2);
  }
  .callout--info .callout-icon { color: var(--color-accent); }

  @keyframes fade-in {
    from { opacity: 0; transform: translateY(3px); }
    to   { opacity: 1; transform: translateY(0); }
  }
</style>
