<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";

  let container = $derived(encoder.container);
</script>

<section class="tab">
  <header class="tab-header">
    <h2 class="tab-title">Conteneur</h2>
    <p class="tab-desc">
      Le conteneur détermine quels types de pistes et métadonnées peuvent être inclus.
    </p>
  </header>

  <div class="container-grid">
    <button
      type="button"
      class="container-card {container === 'mkv' ? 'container-card--active' : ''}"
      onclick={() => encoder.setContainer("mkv")}
    >
      <div class="cc-top">
        <span class="cc-format">MKV</span>
        <span class="cc-name">Matroska</span>
        <span class="cc-badge cc-badge--rec">Recommandé</span>
      </div>
      <ul class="cc-features">
        <li>Sous-titres ASS natifs</li>
        <li>Chapitres et métadonnées</li>
        <li>Flux multiples</li>
      </ul>
    </button>

    <button
      type="button"
      class="container-card {container === 'mp4' ? 'container-card--active' : ''}"
      onclick={() => encoder.setContainer("mp4")}
    >
      <div class="cc-top">
        <span class="cc-format">MP4</span>
        <span class="cc-name">MPEG-4</span>
        <span class="cc-badge">Compatible</span>
      </div>
      <ul class="cc-features">
        <li>Apple / mobile</li>
        <li>Streaming HTTP natif</li>
        <li>Sous-titres mov_text</li>
      </ul>
    </button>
  </div>

  {#if container === "mp4"}
    <div class="callout callout--warn">
      Les sous-titres ASS avancés seront convertis en mov_text — certains styles peuvent être perdus.
    </div>
  {/if}
</section>

<style>
  .tab {
    padding: 24px 28px;
    max-width: 520px;
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

  .container-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
    margin-bottom: 14px;
  }

  .container-card {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 16px 18px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    text-align: left;
    transition: border-color 0.12s, background 0.12s;
  }
  .container-card:hover { border-color: var(--color-subtext2); }
  .container-card--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 7%, var(--color-surface));
  }

  .cc-top {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  .cc-format {
    font-family: "Geist Mono", monospace;
    font-size: 20px;
    font-weight: 700;
    color: var(--color-subtext);
    line-height: 1;
    transition: color 0.12s;
  }
  .container-card--active .cc-format { color: var(--color-accent); }
  .cc-name {
    font-size: 11px;
    color: var(--color-subtext2);
    flex: 1;
  }
  .cc-badge {
    font-family: "Geist Mono", monospace;
    font-size: 8px;
    letter-spacing: 0.04em;
    padding: 2px 6px;
    border-radius: var(--radius-full);
    border: 1px solid var(--color-border);
    color: var(--color-subtext2);
    background: var(--color-panel);
  }
  .cc-badge--rec {
    border-color: color-mix(in srgb, var(--color-success) 30%, var(--color-border));
    color: var(--color-success);
    background: color-mix(in srgb, var(--color-success) 8%, var(--color-panel));
  }

  .cc-features {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .cc-features li {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext2);
    padding-left: 10px;
    position: relative;
  }
  .cc-features li::before {
    content: "·";
    position: absolute;
    left: 0;
  }

  .callout {
    padding: 11px 14px;
    border-radius: var(--radius-sm);
    font-size: 12px;
    color: var(--color-subtext);
    line-height: 1.5;
    border: 1px solid var(--color-border);
    background: var(--color-surface);
  }
  .callout--warn {
    background: color-mix(in srgb, var(--color-warning, #fbbf24) 5%, var(--color-surface));
    border-color: color-mix(in srgb, var(--color-warning, #fbbf24) 25%, var(--color-border));
    color: color-mix(in srgb, var(--color-warning, #fbbf24) 65%, var(--color-subtext));
  }
</style>
