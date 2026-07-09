<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { formatDuration } from "$lib/stores/naming";
  import type { AppFile } from "$lib/stores/encoder.svelte";
  import { formatSize } from "$lib/utils";
  import {
    X, Film, Volume2, Subtitles, Clock, HardDrive, Gauge, Activity,
  } from "@lucide/svelte";

  type Props = {
    file:     AppFile;
    position: { x: number; y: number };
    onclose:  () => void;
  };

  let { file, position, onclose }: Props = $props();

  let initFilename = $derived(file.path.split(/[\\/]/).pop() ?? file.filename);
  let audio_langs  = $derived(file.audio_langs);
  let sub_langs    = $derived(file.sub_langs);

  const STATUS_MAP: Record<string, { label: string; color: string }> = {
    analysing: { label: "Analyse…",  color: "var(--color-accent)"  },
    encoding:  { label: "En cours",  color: "var(--color-accent)"  },
    queued:    { label: "En file",   color: "var(--color-subtext)" },
    ready:     { label: "Prêt",      color: "var(--color-success)" },
    done:      { label: "Terminé",   color: "var(--color-success)" },
    error:     { label: "Erreur",    color: "var(--color-danger)"  },
  };

  const STREAM_ICON: Record<string, any> = {
    video:    Film,
    audio:    Volume2,
    subtitle: Subtitles,
  };

  let result  = $derived(file.result);
  let gainPct = $derived(
    result && result.original_mb > 0
      ? ((result.original_mb - result.encoded_mb) / result.original_mb) * 100
      : null,
  );

  // ── Positionnement automatique ──
  let popoverEl = $state<HTMLDivElement | null>(null);

  let popoverStyle = $derived.by(() => {
    if (!popoverEl) return { left: position.x, top: position.y + 8 };
    const rect = popoverEl.getBoundingClientRect();
    const viewWidth = window.innerWidth;
    const viewHeight = window.innerHeight;

    let left = position.x;
    let top = position.y + 8;

    if (left + rect.width > viewWidth - 12) {
      left = viewWidth - rect.width - 12;
    }
    if (left < 12) left = 12;

    if (top + rect.height > viewHeight - 12) {
      top = position.y - rect.height - 8;
      if (top < 12) top = 12;
    }

    return { left, top };
  });
</script>

<svelte:window onkeydown={(e) => { if (e.key === "Escape") onclose(); }} />

<!-- Overlay transparent pour fermer au clic en dehors -->
<div
  class="backdrop"
  role="presentation"
  onclick={onclose}
  oncontextmenu={(e) => { e.preventDefault(); onclose(); }}
></div>

<!-- Popover -->
<div
  bind:this={popoverEl}
  class="popover"
  style="left:{popoverStyle.left}px; top:{popoverStyle.top}px;"
  role="dialog"
  aria-modal="true"
  aria-label="Détails du fichier"
>
  <!-- ── Header ── -->
  <div class="popover-header">
    <div class="header-left">
      <div class="file-icon">
        <Film class="w-4 h-4" style="color: var(--color-accent);" />
      </div>
      <div class="header-title">
        <span class="header-name" title={initFilename}>{initFilename}</span>
        {#if STATUS_MAP[file.status]}
          <span class="status-pill" style="color: {STATUS_MAP[file.status].color};">
            {STATUS_MAP[file.status].label}
          </span>
        {/if}
      </div>
    </div>
    <button onclick={onclose} class="close-btn" aria-label="Fermer">
      <X class="w-4 h-4" />
    </button>
  </div>

  <!-- ── Body ── -->
  <div class="popover-body">
    <!-- Métriques -->
    <div class="metrics-row">
      <div class="metric">
        <HardDrive class="metric-icon" />
        <span class="metric-val">{formatSize(file.size_mb)}</span>
        <span class="metric-label">Taille</span>
      </div>
      <div class="metric-div"></div>
      <div class="metric">
        <Clock class="metric-icon" />
        <span class="metric-val">{formatDuration(file.duration_secs)}</span>
        <span class="metric-label">Durée</span>
      </div>
      <div class="metric-div"></div>
      <div class="metric">
        <Gauge class="metric-icon" />
        <span class="metric-val">{file.fps?.toFixed(2) ?? "—"}</span>
        <span class="metric-label">FPS</span>
      </div>
      {#if gainPct !== null}
        <div class="metric-div"></div>
        <div class="metric">
          <Activity class="metric-icon" />
          <span class="metric-val" style="color: var(--color-success);">−{gainPct.toFixed(1)}%</span>
          <span class="metric-label">Gain</span>
        </div>
      {/if}
    </div>

    <!-- Fichier source (chemin complet) -->
    <div class="source-info">
      <span class="source-label">Fichier source</span>
      <div class="source-path" title={file.path}>{file.path}</div>
    </div>

    <!-- Langs -->
    {#if audio_langs?.length > 0 || sub_langs?.length > 0}
      <div class="langs-row">
        {#if audio_langs?.length > 0}
          <div class="lang-group">
            <div class="lang-group-hdr">
              <span class="section-title">Audio</span>
              {#if encoder.fileSelAudio.has(file.path)}
                <span class="override-badge">override</span>
              {/if}
            </div>
            <div class="lang-chips">
              {#each audio_langs as lang}
                {@const active = (encoder.fileSelAudio.get(file.path) ?? encoder.selAudio).has(lang)}
                <span class="lang-chip {active ? 'lang-chip--audio' : ''}">{lang.toUpperCase()}</span>
              {/each}
            </div>
          </div>
        {/if}
        <div class="lang-group">
          <div class="lang-group-hdr">
            <span class="section-title">Sous-titres</span>
            {#if encoder.fileSelSubs.has(file.path)}
              <span class="override-badge">override</span>
            {/if}
          </div>
          <div class="lang-chips">
            {#if sub_langs?.length > 0}
              {#each sub_langs as lang}
                {@const active = (encoder.fileSelSubs.get(file.path) ?? encoder.selSubs).has(lang)}
                <span class="lang-chip {active ? 'lang-chip--sub' : ''}">{lang.toUpperCase()}</span>
              {/each}
            {:else}
              <span class="empty-label">Aucun</span>
            {/if}
          </div>
        </div>
      </div>
    {/if}

    <!-- Flux -->
    {#if file.streams?.length > 0}
      <div class="section">
        <span class="section-title">Flux ({file.streams.length})</span>
        <div class="stream-table">
          {#each file.streams as s}
            {@const Icon = STREAM_ICON[s.codec_type] ?? Film}
            <div class="stream-row">
              <span class="stream-type">
                <Icon class="w-3 h-3" />
                {s.codec_type?.toUpperCase()}
              </span>
              <span class="stream-codec">{s.codec_name}</span>
              {#if s.width && s.height}
                <span class="stream-dim">{s.width}×{s.height}</span>
              {/if}
              <span class="stream-lang">{s.language?.toUpperCase() || "UND"}</span>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Résultat encodage -->
    {#if result}
      <div class="section">
        <span class="section-title">Résultat encodage</span>
        <div class="result-grid">
          <div class="result-cell">
            <span class="result-label">Original</span>
            <span class="result-val">{formatSize(result.original_mb)}</span>
          </div>
          <div class="result-cell">
            <span class="result-label">Encodé</span>
            <span class="result-val">{formatSize(result.encoded_mb)}</span>
          </div>
          <div class="result-cell">
            <span class="result-label">Durée</span>
            <span class="result-val">{formatDuration(result.duration_secs)}</span>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .backdrop {
    position: fixed; inset: 0; z-index: 9998;
    background: transparent;
    /* capture les clics sans fond visible */
  }

  .popover {
    position: fixed; z-index: 9999;
    width: 480px; max-width: 92vw;
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    box-shadow: 0 12px 40px rgba(0,0,0,0.5);
    display: flex; flex-direction: column;
    overflow: hidden;
    max-height: 80vh;
  }

  /* Header */
  .popover-header {
    display: flex; align-items: center; justify-content: space-between;
    gap: 12px; padding: 10px 14px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
    background: var(--color-surface);
  }
  .header-left {
    display: flex; align-items: center; gap: 10px; min-width: 0;
  }
  .file-icon {
    width: 28px; height: 28px;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--color-accent) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-accent) 25%, transparent);
    display: flex; align-items: center; justify-content: center;
    flex-shrink: 0;
  }
  .header-title {
    display: flex; flex-direction: column; gap: 1px; min-width: 0;
  }
  .header-name {
    font-family: "Geist Mono", monospace;
    font-size: 11px; font-weight: 600;
    color: var(--color-text);
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
    max-width: 300px;
  }
  .status-pill {
    font-family: "Geist Mono", monospace;
    font-size: 9px; font-weight: 700;
    text-transform: uppercase; letter-spacing: 0.1em;
  }
  .close-btn {
    width: 26px; height: 26px;
    display: inline-flex; align-items: center; justify-content: center;
    border-radius: var(--radius-xs); border: 1px solid transparent;
    background: transparent; color: var(--color-subtext);
    cursor: pointer; flex-shrink: 0;
    transition: background 0.1s, color 0.1s, border-color 0.1s;
  }
  .close-btn:hover {
    background: var(--color-panel2); border-color: var(--color-border);
    color: var(--color-text);
  }

  /* Body */
  .popover-body {
    flex: 1; overflow-y: auto; padding: 12px 14px;
    display: flex; flex-direction: column; gap: 10px;
  }

  /* Metrics */
  .metrics-row {
    display: flex; align-items: center;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    flex-shrink: 0;
  }
  .metric {
    flex: 1; display: flex; flex-direction: column;
    align-items: center; gap: 2px;
  }
  .metric-val {
    font-family: "Geist Mono", monospace;
    font-size: 12px; font-weight: 700; color: var(--color-text); line-height: 1;
  }
  .metric-label {
    font-size: 8px; text-transform: uppercase; letter-spacing: 0.1em;
    color: var(--color-subtext); font-family: "Geist Mono", monospace;
  }
  .metric-div {
    width: 1px; height: 22px; background: var(--color-border); flex-shrink: 0;
  }

  /* Source info */
  .source-info {
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding: 2px 0;
  }
  .source-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--color-subtext);
  }
  .source-path {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
    word-break: break-all;
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
  }

  /* Langs */
  .langs-row { display: flex; gap: 12px; }
  .lang-group { flex: 1; display: flex; flex-direction: column; gap: 4px; }
  .lang-chips { display: flex; flex-wrap: wrap; gap: 3px; }
  .lang-group-hdr { display: flex; align-items: center; gap: 6px; }
  .override-badge {
    font-family: "Geist Mono", monospace; font-size: 8px; font-weight: 700;
    padding: 0px 5px; border-radius: var(--radius-full);
    background: color-mix(in srgb, var(--color-accent) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-accent) 25%, transparent);
    color: var(--color-accent); text-transform: uppercase; letter-spacing: 0.06em;
  }
  .lang-chip {
    font-family: "Geist Mono", monospace; font-size: 9px; font-weight: 600;
    padding: 1px 7px; border-radius: var(--radius-full);
    border: 1px solid var(--color-border);
    background: var(--color-surface); color: var(--color-subtext2); opacity: 0.5;
  }
  .lang-chip--audio {
    background: color-mix(in srgb, var(--color-accent) 12%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 35%, transparent);
    color: var(--color-accent); opacity: 1;
  }
  .lang-chip--sub {
    background: color-mix(in srgb, var(--color-success) 10%, transparent);
    border-color: color-mix(in srgb, var(--color-success) 30%, transparent);
    color: var(--color-success); opacity: 1;
  }

  /* Section */
  .section { display: flex; flex-direction: column; gap: 5px; }
  .section-title {
    font-family: "Geist Mono", monospace; font-size: 9px; font-weight: 700;
    text-transform: uppercase; letter-spacing: 0.12em; color: var(--color-subtext);
  }
  .empty-label { font-size: 10px; font-style: italic; color: var(--color-subtext); }

  /* Streams */
  .stream-table {
    display: flex; flex-direction: column;
    border-radius: var(--radius-sm); border: 1px solid var(--color-border);
    overflow-y: auto; max-height: 160px;
  }
  .stream-row {
    display: flex; align-items: center; gap: 8px;
    padding: 4px 8px;
    font-family: "Geist Mono", monospace; font-size: 10px;
    border-bottom: 1px solid color-mix(in srgb, var(--color-border) 40%, transparent);
  }
  .stream-row:last-child { border-bottom: none; }
  .stream-row:nth-child(even) {
    background: color-mix(in srgb, var(--color-surface) 60%, transparent);
  }
  .stream-type {
    display: inline-flex; align-items: center; gap: 4px;
    color: var(--color-accent); font-weight: 600; min-width: 66px;
  }
  .stream-codec { color: var(--color-text); flex: 1; }
  .stream-dim   { color: var(--color-subtext); }
  .stream-lang {
    font-size: 9px; padding: 0px 5px;
    border-radius: var(--radius-full);
    border: 1px solid var(--color-border);
    background: var(--color-surface); color: var(--color-subtext2);
  }

  /* Result */
  .result-grid {
    display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px;
  }
  .result-cell {
    display: flex; flex-direction: column; gap: 2px;
    padding: 6px 8px;
    border-radius: var(--radius-sm);
    background: var(--color-surface); border: 1px solid var(--color-border);
  }
  .result-label {
    font-size: 8px; text-transform: uppercase; letter-spacing: 0.1em;
    color: var(--color-subtext); font-family: "Geist Mono", monospace;
  }
  .result-val {
    font-family: "Geist Mono", monospace;
    font-size: 12px; font-weight: 700; color: var(--color-text);
  }
</style>