<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";

  let container = $derived(encoder.container);
</script>

<section class="content-section">
  <header class="section-header">
    <div>
      <h2 class="section-title">Conteneur de sortie</h2>
      <p class="section-desc">
        Le conteneur détermine quels types de pistes et métadonnées peuvent être
        inclus.
      </p>
    </div>
  </header>

  <div class="field-block">
    <div class="container-cards">
      <button
        type="button"
        class="container-card {container === 'mkv'
          ? 'container-card--active'
          : ''}"
        onclick={() => encoder.setContainer("mkv")}
      >
        <div class="cc-format">MKV</div>
        <div class="cc-name">Matroska</div>
        <ul class="cc-features">
          <li>Sous-titres ASS natifs</li>
          <li>Chapitres et métadonnées</li>
          <li>Flux multiples</li>
        </ul>
        <div class="cc-badge cc-badge--rec">Recommandé</div>
      </button>
      <button
        type="button"
        class="container-card {container === 'mp4'
          ? 'container-card--active'
          : ''}"
        onclick={() => encoder.setContainer("mp4")}
      >
        <div class="cc-format">MP4</div>
        <div class="cc-name">MPEG-4</div>
        <ul class="cc-features">
          <li>Compatible Apple / mobile</li>
          <li>Streaming HTTP natif</li>
          <li>Sous-titres mov_text</li>
        </ul>
        <div class="cc-badge">Compatible</div>
      </button>
    </div>

    {#if container === "mp4"}
      <div class="info-callout info-callout--warn">
        Les sous-titres ASS avancés seront convertis en mov_text — certains
        styles peuvent être perdus.
      </div>
    {/if}
  </div>
</section>

<style>
  .content-section {
    padding: 28px 32px;
    max-width: 680px;
  }

  .section-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 28px;
    padding-bottom: 20px;
    border-bottom: 1px solid var(--color-border);
  }
  .section-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text);
    letter-spacing: -0.02em;
    margin: 0 0 6px;
  }
  .section-desc {
    font-size: 12px;
    color: var(--color-subtext);
    line-height: 1.6;
    max-width: 420px;
    margin: 0;
  }

  .field-block {
    margin-bottom: 24px;
  }
  .field-block:last-child {
    margin-bottom: 0;
  }

  .container-cards {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    margin-bottom: 14px;
  }
  .container-card {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
    padding: 16px 18px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    text-align: left;
    transition:
      border-color 0.15s,
      background 0.15s;
  }
  .container-card:hover {
    border-color: var(--color-subtext2);
  }
  .container-card--active {
    border-color: var(--color-accent);
    background: color-mix(
      in srgb,
      var(--color-accent) 7%,
      var(--color-surface)
    );
  }
  .cc-format {
    font-family: "Geist Mono", monospace;
    font-size: 22px;
    font-weight: 700;
    color: var(--color-subtext);
    line-height: 1;
    transition: color 0.15s;
  }
  .container-card--active .cc-format {
    color: var(--color-accent);
  }
  .cc-name {
    font-size: 11px;
    color: var(--color-subtext2);
    margin-bottom: 4px;
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
    color: var(--color-subtext2);
  }
  .cc-badge {
    margin-top: 4px;
    font-family: "Geist Mono", monospace;
    font-size: 8px;
    letter-spacing: 0.05em;
    padding: 2px 7px;
    border-radius: var(--radius-full);
    border: 1px solid var(--color-border);
    color: var(--color-subtext2);
    background: var(--color-panel);
  }
  .cc-badge--rec {
    border-color: color-mix(
      in srgb,
      var(--color-success) 30%,
      var(--color-border)
    );
    color: var(--color-success);
    background: color-mix(in srgb, var(--color-success) 8%, var(--color-panel));
  }

  .info-callout {
    padding: 11px 14px;
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    font-size: 12px;
    color: var(--color-subtext);
    line-height: 1.5;
  }
  .info-callout--warn {
    background: color-mix(
      in srgb,
      var(--color-warning, #fbbf24) 5%,
      var(--color-surface)
    );
    border-color: color-mix(
      in srgb,
      var(--color-warning, #fbbf24) 25%,
      var(--color-border)
    );
    color: color-mix(
      in srgb,
      var(--color-warning, #fbbf24) 70%,
      var(--color-subtext)
    );
  }
</style>
