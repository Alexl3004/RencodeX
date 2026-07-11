<script lang="ts">
  import { formatSeasonEpisode } from "$lib/stores/naming";
  import type {
    TagId,
    TitleCaseMode,
    ResolutionCase,
    ProviderCase,
    SeasonEpisodeFormat,
  } from "$lib/stores/types";
  import { TAG_LABELS } from "$lib/stores/naming";

  const TITLE_CASE_OPTIONS: { value: TitleCaseMode; label: string }[] = [
    { value: "original", label: "Original" },
    { value: "upper", label: "MAJUSCULES" },
    { value: "lower", label: "minuscules" },
    { value: "title", label: "Titre" },
  ];

  let {
    previewSeries,
    previewMovie,
    tagOrder,
    disabledTags,
    team,
    resCase,
    titleCase,
    seFormat,
    japVer,
    codecFmt,
    sourceCase,
    webSourceFmt,
    tagSep,
    provCase,
    yearParentheses,
  }: {
    previewSeries: string;
    previewMovie: string;
    tagOrder: TagId[];
    disabledTags: Set<TagId>;
    team: string;
    resCase: ResolutionCase;
    titleCase: TitleCaseMode;
    seFormat: SeasonEpisodeFormat;
    japVer: boolean;
    codecFmt: string;
    sourceCase: string;
    webSourceFmt: string;
    tagSep: string;
    provCase: ProviderCase;
    yearParentheses: boolean;
  } = $props();
</script>

<section class="content-section">
  <header class="section-header">
    <div>
      <h2 class="section-title">Aperçu du nom</h2>
      <p class="section-desc">
        Nom de fichier généré avec les paramètres actuels sur un exemple fictif.
      </p>
    </div>
  </header>

  <!-- Noms générés -->
  <div class="preview-name-box">
    <div class="pnb-label">SÉRIE · NOM GÉNÉRÉ</div>
    <div class="pnb-name">{previewSeries}</div>
    <div class="pnb-label" style="margin-top:14px;">FILM · AVEC ANNÉE</div>
    <div class="pnb-name">{previewMovie}</div>
  </div>

  <!-- Tags actifs/inactifs -->
  <div class="preview-tags-grid">
    {#each tagOrder as id}
      {@const disabled = disabledTags.has(id)}
      <div class="ptg-item {disabled ? 'ptg-item--off' : ''}">
        <span class="ptg-dot {disabled ? 'ptg-dot--off' : ''}"></span>
        <span class="ptg-name">{TAG_LABELS[id]}</span>
        {#if id === "team" && team}
          <span class="ptg-val">{team}</span>
        {/if}
      </div>
    {/each}
  </div>

  <!-- Récap des paramètres -->
  <div class="preview-params">
    <div class="pp-row">
      <span class="pp-key">Résolution</span>
      <span class="pp-val">{resCase === "upper" ? "1080P" : "1080p"}</span>
    </div>
    <div class="pp-row">
      <span class="pp-key">Casse titre</span>
      <span class="pp-val"
        >{TITLE_CASE_OPTIONS.find((o) => o.value === titleCase)?.label ??
          titleCase}</span
      >
    </div>
    <div class="pp-row">
      <span class="pp-key">S/E format</span>
      <span class="pp-val">{formatSeasonEpisode("S03E01", seFormat)}</span>
    </div>
    <div class="pp-row">
      <span class="pp-key">(Japanese ver.)</span>
      <span class="pp-val">{japVer ? "Inclus" : "Exclus"}</span>
    </div>
    <div class="pp-row">
      <span class="pp-key">Codec</span>
      <span class="pp-val">{codecFmt}</span>
    </div>
    <div class="pp-row">
      <span class="pp-key">Source BluRay</span>
      <span class="pp-val"
        >{{ original: "BluRay", upper: "BLURAY", lower: "bluray" }[
          sourceCase
        ]}</span
      >
    </div>
    <div class="pp-row">
      <span class="pp-key">Source WEB</span>
      <span class="pp-val"
        >{{ "WEB-DL": "WEB-DL", WEBDL: "WEBDL", "Web-DL": "Web-DL" }[
          webSourceFmt
        ]}</span
      >
    </div>
    <div class="pp-row">
      <span class="pp-key">Séparateur</span>
      <span class="pp-val"
        >{{ " ": "Espace", ".": "Point", _: "Underscore" }[tagSep]}</span
      >
    </div>
    <div class="pp-row">
      <span class="pp-key">Provider</span>
      <span class="pp-val"
        >{{ upper: "AMZN", lower: "amzn", hidden: "Masqué" }[provCase]}</span
      >
    </div>
    <div class="pp-row">
      <span class="pp-key">Année film</span>
      <span class="pp-val">{yearParentheses ? "(2024)" : "2024"}</span>
    </div>
    {#if team}
      <div class="pp-row">
        <span class="pp-key">Team</span>
        <span class="pp-val">{team}</span>
      </div>
    {/if}
  </div>
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

  .preview-name-box {
    padding: 18px 20px;
    border-radius: var(--radius-sm);
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    margin-bottom: 20px;
  }
  .pnb-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    letter-spacing: 0.07em;
    color: var(--color-subtext2);
    text-transform: uppercase;
    margin-bottom: 8px;
  }
  .pnb-name {
    font-family: "Geist Mono", monospace;
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text);
    line-height: 1.5;
    word-break: break-all;
  }

  .preview-tags-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 20px;
  }
  .ptg-item {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 9px;
    border-radius: var(--radius-full);
    border: 1px solid var(--color-border);
    background: var(--color-panel);
    font-size: 11px;
    color: var(--color-subtext);
    transition: opacity 0.15s;
  }
  .ptg-item--off {
    opacity: 0.35;
    text-decoration: line-through;
  }
  .ptg-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--color-accent);
    flex-shrink: 0;
  }
  .ptg-dot--off {
    background: var(--color-subtext2);
  }
  .ptg-name {
    font-size: 11px;
  }
  .ptg-val {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 10%, transparent);
    padding: 1px 5px;
    border-radius: 3px;
  }

  .preview-params {
    display: flex;
    flex-direction: column;
    gap: 0;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }
  .pp-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 9px 14px;
    border-bottom: 1px solid var(--color-border);
  }
  .pp-row:last-child {
    border-bottom: none;
  }
  .pp-key {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext2);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .pp-val {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    font-weight: 600;
    color: var(--color-text);
  }
</style>
