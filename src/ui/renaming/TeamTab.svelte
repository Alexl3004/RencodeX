<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";

  let {
    team,
    previewSeries,
  }: {
    team: string;
    previewSeries: string;
  } = $props();
</script>

<section class="content-section">
  <header class="section-header">
    <div>
      <h2 class="section-title">Team</h2>
      <p class="section-desc">
        Le nom de la team est inséré dans le nom de fichier à la position
        définie dans l'onglet « Ordre des tags ». Laissez vide pour l'exclure.
      </p>
    </div>
  </header>

  <div class="field-block">
    <div class="field-label">Nom de la team</div>
    <input
      type="text"
      value={team}
      oninput={(e: Event) =>
        encoder.setTeam((e.target as HTMLInputElement).value)}
      placeholder="ex : SubsPlease, Erai-raws…"
      class="text-input"
    />
  </div>

  {#if team}
    <div class="team-preview-box">
      <div class="tpb-label">TAG INSÉRÉ</div>
      <div class="tpb-value">{team}</div>
      <div class="tpb-example">
        <span class="tpb-ex-label">Exemple ·</span>
        <span class="tpb-ex-name">{previewSeries}</span>
      </div>
    </div>
  {:else}
    <div class="info-callout">
      Aucune team définie — le tag sera ignoré même s'il est activé dans
      l'ordre.
    </div>
  {/if}
</section>

<style>
  .content-section {
    padding: 28px 24px;
  }
  .section-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 24px;
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
    max-width: 380px;
    margin: 0;
  }

  .field-block {
    margin-bottom: 20px;
  }
  .field-label {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color-subtext);
    margin-bottom: 10px;
  }

  .text-input {
    width: 100%;
    font-family: "Geist Mono", monospace;
    font-size: 12px;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    color: var(--color-text);
    outline: none;
    transition: border-color 0.15s;
    box-sizing: border-box;
  }
  .text-input:focus {
    border-color: var(--color-accent);
  }
  .text-input::placeholder {
    color: var(--color-subtext2);
  }

  .team-preview-box {
    padding: 16px 18px;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--color-accent) 6%, var(--color-panel));
    border: 1px solid
      color-mix(in srgb, var(--color-accent) 22%, var(--color-border));
  }
  .tpb-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    letter-spacing: 0.06em;
    color: var(--color-subtext2);
    text-transform: uppercase;
    margin-bottom: 5px;
  }
  .tpb-value {
    font-family: "Geist Mono", monospace;
    font-size: 18px;
    font-weight: 700;
    color: var(--color-accent);
    margin-bottom: 10px;
  }
  .tpb-example {
    display: flex;
    align-items: baseline;
    gap: 6px;
    flex-wrap: wrap;
  }
  .tpb-ex-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext2);
    white-space: nowrap;
  }
  .tpb-ex-name {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
    word-break: break-all;
  }

  .info-callout {
    padding: 12px 14px;
    border-radius: var(--radius-sm);
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    font-size: 12px;
    color: var(--color-subtext);
    line-height: 1.5;
  }
</style>
