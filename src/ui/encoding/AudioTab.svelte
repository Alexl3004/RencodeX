<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";

  let audioMode    = $derived(encoder.audioMode);
  let audioBitrate = $derived(encoder.audioBitrate);

  const BITRATE_HINTS: Record<number, string> = {
    128: "Standard",
    192: "Recommandé",
    256: "Haute qualité",
    320: "Maximum",
  };
</script>

<section class="tab">
  <header class="tab-header">
    <h2 class="tab-title">Audio</h2>
    <p class="tab-desc">
      Réencoder en AAC garantit la compatibilité. La copie conserve la piste d'origine sans dégradation.
    </p>
  </header>

  <!-- Mode -->
  <div class="field">
    <span class="field-label">Mode</span>
    <div class="mode-grid">
      <button
        type="button"
        class="mode-card {audioMode === 'reencode' ? 'mode-card--active' : ''}"
        onclick={() => encoder.setAudioMode("reencode")}
      >
        <span class="mode-name">Réencoder</span>
        <span class="mode-hint">AAC · compatibilité maximale</span>
      </button>
      <button
        type="button"
        class="mode-card {audioMode === 'copy' ? 'mode-card--active' : ''}"
        onclick={() => encoder.setAudioMode("copy")}
      >
        <span class="mode-name">Copier</span>
        <span class="mode-hint">-c:a copy · sans dégradation</span>
      </button>
    </div>
  </div>

  <!-- Débit (visible seulement en mode reencode) -->
  {#if audioMode === "reencode"}
    <div class="field" style="animation: fade-in 0.14s ease">
      <span class="field-label">Débit AAC</span>
      <div class="bitrate-grid">
        {#each [128, 192, 256, 320] as br}
          {@const active = audioBitrate === br}
          <button
            type="button"
            class="bitrate-btn {active ? 'bitrate-btn--active' : ''}"
            onclick={() => encoder.setAudioBitrate(br)}
          >
            <span class="bitrate-val">{br}</span>
            <span class="bitrate-unit">kbps</span>
            <span class="bitrate-hint">{BITRATE_HINTS[br]}</span>
          </button>
        {/each}
      </div>
    </div>
  {:else}
    <div class="callout" style="animation: fade-in 0.14s ease">
      La piste audio source est conservée telle quelle — aucune dégradation supplémentaire.
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

  .field {
    margin-bottom: 20px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .field:last-child { margin-bottom: 0; }

  .field-label {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color-subtext);
  }

  /* ── Mode cards ─────────────────────────────────────────────────────────── */
  .mode-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }
  .mode-card {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 13px 14px;
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
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-surface));
  }
  .mode-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-subtext);
    transition: color 0.12s;
  }
  .mode-card--active .mode-name { color: var(--color-accent); }
  .mode-hint {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext2);
  }

  /* ── Débit ──────────────────────────────────────────────────────────────── */
  .bitrate-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 6px;
  }
  .bitrate-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 12px 6px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    transition: border-color 0.12s, background 0.12s;
  }
  .bitrate-btn:hover { border-color: var(--color-subtext2); }
  .bitrate-btn--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-surface));
  }
  .bitrate-val {
    font-family: "Geist Mono", monospace;
    font-size: 18px;
    font-weight: 700;
    color: var(--color-subtext);
    line-height: 1;
    transition: color 0.12s;
  }
  .bitrate-btn--active .bitrate-val { color: var(--color-accent); }
  .bitrate-unit {
    font-family: "Geist Mono", monospace;
    font-size: 8px;
    color: var(--color-subtext2);
  }
  .bitrate-hint {
    font-size: 9px;
    color: var(--color-subtext2);
    margin-top: 2px;
  }
  .bitrate-btn--active .bitrate-hint { color: var(--color-accent); opacity: 0.8; }

  /* ── Callout ────────────────────────────────────────────────────────────── */
  .callout {
    padding: 11px 14px;
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    font-size: 12px;
    color: var(--color-subtext);
    line-height: 1.5;
  }

  @keyframes fade-in {
    from { opacity: 0; transform: translateY(3px); }
    to   { opacity: 1; transform: translateY(0); }
  }
</style>
