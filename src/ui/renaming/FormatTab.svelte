<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { SEASON_EPISODE_FORMATS } from "$lib/stores/naming";
  import type {
    TitleCaseMode,
    ResolutionCase,
    CodecFormat,
    SourceCase,
    WebSourceFormat,
    TagSeparator,
    ProviderCase,
    YearFormat,
  } from "$lib/stores/types";
  import {
    RotateCcw,
    Monitor,
    Type,
    Calendar,
    Film,
    Disc,
    Globe,
    Minus,
    Building,
    Languages,
    CalendarDays,
  } from "@lucide/svelte";

  let {
    previewSeries,
    previewMovie,
    resCase,
    titleCase,
    codecFmt,
    sourceCase,
    yearFormat,
    webSourceFmt,
    tagSep,
    provCase,
    seFormat,
    japVer,
  }: {
    previewSeries: string;
    previewMovie: string;
    resCase: ResolutionCase;
    titleCase: TitleCaseMode;
    codecFmt: CodecFormat;
    sourceCase: SourceCase;
    yearFormat: YearFormat;
    webSourceFmt: WebSourceFormat;
    tagSep: TagSeparator;
    provCase: ProviderCase;
    seFormat: string;
    japVer: boolean;
  } = $props();

  const TITLE_CASE_OPTIONS: { value: TitleCaseMode; label: string }[] = [
    { value: "original", label: "Original" },
    { value: "upper", label: "MAJUSCULES" },
    { value: "lower", label: "minuscules" },
    { value: "title", label: "Titre" },
  ];
</script>

<section class="content-section">
  <header class="section-header">
    <div>
      <h2 class="section-title">Format</h2>
      <p class="section-desc">
        Personnalisez la casse et le format de chaque composant du nom de
        fichier.
      </p>
    </div>
    <button
      type="button"
      class="reset-btn"
      onclick={() => encoder.resetFormatOptions()}
      title="Remettre les options par défaut"
    >
      <RotateCcw class="w-3 h-3" />
      Réinitialiser
    </button>
  </header>

  <!-- Aperçu compact -->
  <div class="format-preview-box">
    <div class="fp-row">
      <span class="fp-label">Série</span>
      <span class="fp-name">{previewSeries}</span>
    </div>
    <div class="fp-row">
      <span class="fp-label">Film</span>
      <span class="fp-name">{previewMovie}</span>
    </div>
  </div>

  <!-- Grille des paramètres -->
  <div class="format-grid">
    <!-- Résolution -->
    <div class="format-card">
      <div class="format-card-header">
        <Monitor
          class="format-card-icon w-[14px] h-[14px]"
          aria-hidden="true"
        />
        <span class="format-card-label">Résolution</span>
      </div>
      <div class="format-card-options">
        {#each [["upper", "1080P"], ["lower", "1080p"]] as [val, preview]}
          <button
            type="button"
            class="opt-pill {resCase === val ? 'opt-pill--active' : ''}"
            onclick={() => encoder.setResolutionCase(val as ResolutionCase)}
            >{preview}</button
          >
        {/each}
      </div>
    </div>

    <!-- Titre -->
    <div class="format-card">
      <div class="format-card-header">
        <Type class="format-card-icon w-[14px] h-[14px]" aria-hidden="true" />
        <span class="format-card-label">Titre</span>
      </div>
      <div class="format-card-options">
        {#each TITLE_CASE_OPTIONS as opt}
          <button
            type="button"
            class="opt-pill {titleCase === opt.value ? 'opt-pill--active' : ''}"
            onclick={() => encoder.setTitleCase(opt.value)}>{opt.label}</button
          >
        {/each}
      </div>
    </div>

    <!-- Saison / Épisode -->
    <div class="format-card">
      <div class="format-card-header">
        <Calendar
          class="format-card-icon w-[14px] h-[14px]"
          aria-hidden="true"
        />
        <span class="format-card-label">S/E format</span>
      </div>
      <div class="format-card-options">
        {#each SEASON_EPISODE_FORMATS as f}
          <button
            type="button"
            class="opt-pill {seFormat === f.value ? 'opt-pill--active' : ''}"
            onclick={() => encoder.setSeasonEpisodeFormat(f.value)}
            >{f.label}</button
          >
        {/each}
      </div>
    </div>

    <!-- Codec vidéo -->
    <div class="format-card">
      <div class="format-card-header">
        <Film class="format-card-icon w-[14px] h-[14px]" aria-hidden="true" />
        <span class="format-card-label">Codec vidéo</span>
      </div>
      <div class="format-card-options">
        {#each ["H265", "H.265", "HEVC"] as const as val}
          <button
            type="button"
            class="opt-pill {codecFmt === val ? 'opt-pill--active' : ''}"
            onclick={() => encoder.setCodecFormat(val as CodecFormat)}
            >{val}</button
          >
        {/each}
      </div>
    </div>

    <!-- Source BluRay -->
    <div class="format-card">
      <div class="format-card-header">
        <Disc class="format-card-icon w-[14px] h-[14px]" aria-hidden="true" />
        <span class="format-card-label">Source BluRay</span>
      </div>
      <div class="format-card-options">
        {#each [["original", "BluRay"], ["upper", "BLURAY"], ["lower", "bluray"]] as [val, preview]}
          <button
            type="button"
            class="opt-pill {sourceCase === val ? 'opt-pill--active' : ''}"
            onclick={() => encoder.setSourceCase(val as SourceCase)}
            >{preview}</button
          >
        {/each}
      </div>
    </div>

    <!-- Source WEB -->
    <div class="format-card">
      <div class="format-card-header">
        <Globe class="format-card-icon w-[14px] h-[14px]" aria-hidden="true" />
        <span class="format-card-label">Source WEB</span>
      </div>
      <div class="format-card-options">
        {#each [["WEB-DL", "WEB-DL"], ["WEBDL", "WEBDL"], ["Web-DL", "Web-DL"]] as const as [val, preview]}
          <button
            type="button"
            class="opt-pill {webSourceFmt === val ? 'opt-pill--active' : ''}"
            onclick={() => encoder.setWebSourceFormat(val as WebSourceFormat)}
            >{preview}</button
          >
        {/each}
      </div>
    </div>

    <!-- Séparateur -->
    <div class="format-card">
      <div class="format-card-header">
        <Minus class="format-card-icon w-[14px] h-[14px]" aria-hidden="true" />
        <span class="format-card-label">Séparateur</span>
      </div>
      <div class="format-card-options">
        {#each [[" ", "Espace"], [".", "Point"], ["_", "Underscore"]] as const as [val, preview]}
          <button
            type="button"
            class="opt-pill {tagSep === val ? 'opt-pill--active' : ''}"
            onclick={() => encoder.setTagSeparator(val as TagSeparator)}
            >{preview}</button
          >
        {/each}
      </div>
    </div>

    <!-- Provider -->
    <div class="format-card">
      <div class="format-card-header">
        <Building
          class="format-card-icon w-[14px] h-[14px]"
          aria-hidden="true"
        />
        <span class="format-card-label">Provider</span>
      </div>
      <div class="format-card-options">
        {#each [["upper", "AMZN"], ["lower", "amzn"], ["hidden", "Masqué"]] as const as [val, preview]}
          <button
            type="button"
            class="opt-pill {provCase === val ? 'opt-pill--active' : ''}"
            onclick={() => encoder.setProviderCase(val as ProviderCase)}
            >{preview}</button
          >
        {/each}
      </div>
    </div>

    <!-- (Japanese ver.) -->
    <div class="format-card">
      <div class="format-card-header">
        <Languages
          class="format-card-icon w-[14px] h-[14px]"
          aria-hidden="true"
        />
        <span class="format-card-label">(Japanese ver.)</span>
      </div>
      <div class="format-card-options">
        {#each [[true, "Inclure"], [false, "Exclu"]] as const as [val, preview]}
          <button
            type="button"
            class="opt-pill {japVer === val ? 'opt-pill--active' : ''}"
            onclick={() => encoder.setKeepJapaneseVer(val)}>{preview}</button
          >
        {/each}
      </div>
    </div>

    <!-- Année (films) -->
    <div class="format-card">
      <div class="format-card-header">
        <CalendarDays
          class="format-card-icon w-[14px] h-[14px]"
          aria-hidden="true"
        />
        <span class="format-card-label">Année (films)</span>
      </div>
      <div class="format-card-options">
        {#each [["parentheses", "(2024)"], ["plain", "2024"]] as const as [val, preview]}
          <button
            type="button"
            class="opt-pill {yearFormat === val ? 'opt-pill--active' : ''}"
            onclick={() => encoder.setYearFormat(val)}>{preview}</button
          >
        {/each}
      </div>
    </div>
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

  .reset-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 5px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-panel);
    color: var(--color-subtext);
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    cursor: pointer;
    white-space: nowrap;
    flex-shrink: 0;
    transition:
      border-color 0.12s,
      color 0.12s;
  }
  .reset-btn:hover {
    border-color: var(--color-subtext);
    color: var(--color-text);
  }

  .format-preview-box {
    padding: 14px 16px;
    border-radius: var(--radius-sm);
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    margin-bottom: 20px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .fp-row {
    display: flex;
    align-items: baseline;
    gap: 10px;
  }
  .fp-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-subtext2);
    flex-shrink: 0;
    width: 32px;
  }
  .fp-name {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    color: var(--color-text);
    word-break: break-all;
  }

  .format-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 10px;
  }

  .format-card {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-panel);
    overflow: hidden;
  }
  .format-card-header {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 9px 12px 7px;
    border-bottom: 1px solid var(--color-border);
  }
  :global(.format-card-icon) {
    color: var(--color-subtext);
    flex-shrink: 0;
  }
  .format-card-label {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    font-weight: 600;
    color: var(--color-subtext);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .format-card-options {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
    padding: 9px 12px;
  }

  .opt-pill {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    padding: 3px 9px;
    border-radius: 999px;
    border: 1px solid var(--color-border);
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    transition:
      background 0.1s,
      color 0.1s,
      border-color 0.1s;
    white-space: nowrap;
  }
  .opt-pill:hover {
    background: color-mix(in srgb, var(--color-border) 60%, transparent);
    color: var(--color-text);
  }
  .opt-pill--active {
    background: color-mix(in srgb, var(--color-accent) 15%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 35%, transparent);
    color: var(--color-accent);
  }
</style>