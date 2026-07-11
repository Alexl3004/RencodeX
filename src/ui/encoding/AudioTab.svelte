<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";

  let audioMode = $derived(encoder.audioMode);
  let audioBitrate = $derived(encoder.audioBitrate);
</script>

<section class="content-section">
  <header class="section-header">
    <div>
      <h2 class="section-title">Audio</h2>
      <p class="section-desc">
        Réencoder en AAC garantit la compatibilité maximale. La copie conserve
        la piste originale sans dégradation supplémentaire.
      </p>
    </div>
  </header>

  <div class="field-block">
    <div class="field-label">Mode</div>
    <div class="toggle-row">
      <button
        type="button"
        class="toggle-opt {audioMode === 'reencode'
          ? 'toggle-opt--active'
          : ''}"
        onclick={() => encoder.setAudioMode("reencode")}
      >
        <span class="toggle-opt-title">Réencoder</span>
        <span class="toggle-opt-sub">AAC · compatibilité maximale</span>
      </button>
      <button
        type="button"
        class="toggle-opt {audioMode === 'copy' ? 'toggle-opt--active' : ''}"
        onclick={() => encoder.setAudioMode("copy")}
      >
        <span class="toggle-opt-title">Copier</span>
        <span class="toggle-opt-sub">-c:a copy · sans perte</span>
      </button>
    </div>
  </div>

  {#if audioMode === "reencode"}
    <div class="field-block">
      <div class="field-label">Débit AAC</div>
      <div class="bitrate-row">
        {#each [128, 192, 256, 320] as br}
          <button
            type="button"
            class="bitrate-btn {audioBitrate === br
              ? 'bitrate-btn--active'
              : ''}"
            onclick={() => encoder.setAudioBitrate(br)}
          >
            <span class="bitrate-val">{br}</span>
            <span class="bitrate-unit">kbps</span>
          </button>
        {/each}
      </div>
    </div>
  {:else}
    <div class="info-callout">
      La piste audio source est conservée telle quelle — aucune dégradation
      supplémentaire.
    </div>
  {/if}
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

  .field-label {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color-subtext);
    margin-bottom: 10px;
  }

  .toggle-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }
  .toggle-opt {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 3px;
    padding: 12px 14px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    text-align: left;
    transition:
      border-color 0.15s,
      background 0.15s;
  }
  .toggle-opt:hover {
    border-color: var(--color-subtext2);
  }
  .toggle-opt--active {
    border-color: var(--color-accent);
    background: color-mix(
      in srgb,
      var(--color-accent) 8%,
      var(--color-surface)
    );
  }
  .toggle-opt-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-subtext);
    transition: color 0.15s;
  }
  .toggle-opt--active .toggle-opt-title {
    color: var(--color-accent);
  }
  .toggle-opt-sub {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext2);
  }

  .bitrate-row {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
  }
  .bitrate-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 10px 6px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    transition:
      border-color 0.15s,
      background 0.15s;
  }
  .bitrate-btn:hover {
    border-color: var(--color-subtext2);
  }
  .bitrate-btn--active {
    border-color: var(--color-accent);
    background: color-mix(
      in srgb,
      var(--color-accent) 8%,
      var(--color-surface)
    );
  }
  .bitrate-val {
    font-family: "Geist Mono", monospace;
    font-size: 16px;
    font-weight: 700;
    color: var(--color-subtext);
    line-height: 1;
    transition: color 0.15s;
  }
  .bitrate-btn--active .bitrate-val {
    color: var(--color-accent);
  }
  .bitrate-unit {
    font-family: "Geist Mono", monospace;
    font-size: 8px;
    color: var(--color-subtext2);
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
</style>
