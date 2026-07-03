<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { encoder, buildOutputName, computeTag, formatDuration } from "$lib/stores/encoder.svelte";
  import type { AppFile, CleanedName } from "$lib/stores/encoder.svelte";
  import { formatSize } from "$lib/utils";
  import {
    Loader2, RefreshCw, CircleCheck, X, Film,
    Volume2, Subtitles, Clock, HardDrive, Gauge, Activity,
  } from "@lucide/svelte";

  type Props = {
    file:     AppFile;
    onclose:  () => void;
    onrename: (newName: string) => void;
  };

  let { file, onclose, onrename }: Props = $props();

  let initFilename  = $derived(file.path.split(/[\\/]/).pop() ?? file.filename);
  let initOutputExt = $derived(file.output_ext);
  let audio_langs   = $derived(file.audio_langs);
  let sub_langs     = $derived(file.sub_langs);

  let initAudioLangs = $derived(audio_langs.filter((l: string) => encoder.selAudio.has(l)));
  let initSubLangs   = $derived(sub_langs.filter((l: string) => encoder.selSubs.has(l)));

  let editValue = $state("");
  let loading   = $state(true);
  let cleaned   = $state<CleanedName | null>(null);
  let canRename = $derived(!encoder.encoding && file.status !== "analysing");

  let suggestedName = $derived.by(() => {
    if (!cleaned) return "";
    const tag = computeTag(audio_langs, sub_langs, encoder.selAudio, encoder.selSubs);
    return buildOutputName(cleaned, tag, encoder.seasonEpisodeFormat, "AAC", encoder.tagOrder, encoder.team);
  });

  $effect(() => { editValue = file.output_name; });

  $effect(() => {
    loading = true;
    invoke<CleanedName>("clean_filename", {
      raw:        initFilename,
      audioLangs: initAudioLangs,
      subLangs:   initSubLangs,
    })
      .then((r) => {
        cleaned = r;
        const tag = computeTag(audio_langs, sub_langs, encoder.selAudio, encoder.selSubs);
        if (r) editValue = buildOutputName(r, tag, encoder.seasonEpisodeFormat, "AAC", encoder.tagOrder, encoder.team);
      })
      .catch(() => {})
      .finally(() => { loading = false; });
  });

  function confirm() {
    const v = editValue.trim();
    if (v) onrename(v);
    onclose();
  }

  function focusAndSelect(node: HTMLInputElement) {
    node.focus();
    node.select();
  }

  const STATUS_MAP: Record<string, { label: string; color: string }> = {
    analysing: { label: "Analyse…", color: "var(--color-accent)" },
    encoding:  { label: "En cours", color: "var(--color-accent)" },
    queued:    { label: "En file",  color: "var(--color-subtext)" },
    ready:     { label: "Prêt",     color: "var(--color-success)" },
    done:      { label: "Terminé",  color: "var(--color-success)" },
    error:     { label: "Erreur",   color: "var(--color-danger)" },
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
</script>

<svelte:window onkeydown={(e) => {
  if (e.key === "Escape") onclose();
  if (e.key === "Enter") confirm();
}} />

<div
  class="backdrop"
  role="dialog"
  aria-modal="true"
  aria-label="Détails du fichier"
  tabindex="-1"
  onclick={(e) => { if (e.target === e.currentTarget) onclose(); }}
  onkeydown={(e) => { if (e.key === "Escape") onclose(); }}
>
  <div class="modal">

    <!-- ── Header ── -->
    <div class="modal-header">
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

    <!-- ── Body (tout en un) ── -->
    <div class="modal-body">

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

      <!-- ── Renommage ── -->
      {#if canRename}
        <div class="rename-sep">
          <span class="rename-sep-label">Renommer</span>
        </div>

        <div class="section">
          <div class="input-header">
            <span class="section-title">Nouveau nom</span>
            {#if suggestedName && editValue !== suggestedName}
              <button onclick={() => (editValue = suggestedName)} class="reset-btn">
                <RefreshCw class="w-3 h-3" />
                Réinitialiser
              </button>
            {/if}
          </div>
          <input
            type="text"
            bind:value={editValue}
            placeholder="Nom du fichier de sortie…"
            use:focusAndSelect
            class="name-input"
          />
        </div>

        <div class="section">
          <span class="section-title">Nom suggéré</span>
          {#if loading}
            <div class="loading-row">
              <Loader2 class="w-3.5 h-3.5 animate-spin" style="color: var(--color-subtext);" />
              <span>Analyse en cours…</span>
            </div>
          {:else if suggestedName}
            <button
              onclick={() => (editValue = suggestedName)}
              class="suggestion-btn {editValue === suggestedName ? 'suggestion-btn--applied' : ''}"
            >
              {#if editValue === suggestedName}
                <CircleCheck class="w-3.5 h-3.5 shrink-0" />
              {/if}
              <span class="truncate">{suggestedName}</span>
            </button>
          {:else}
            <span class="empty-label">Aucune suggestion disponible</span>
          {/if}
        </div>
      {/if}

      <!-- Langs -->
      {#if file.audio_langs?.length > 0 || file.sub_langs?.length > 0}
        <div class="langs-row">
          {#if file.audio_langs?.length > 0}
            <div class="lang-group">
              <span class="section-title">Audio</span>
              <div class="lang-chips">
                {#each file.audio_langs as lang}
                  <span class="lang-chip {encoder.selAudio.has(lang) ? 'lang-chip--audio' : ''}">{lang.toUpperCase()}</span>
                {/each}
              </div>
            </div>
          {/if}
          <div class="lang-group">
            <span class="section-title">Sous-titres</span>
            <div class="lang-chips">
              {#if file.sub_langs?.length > 0}
                {#each file.sub_langs as lang}
                  <span class="lang-chip {encoder.selSubs.has(lang) ? 'lang-chip--sub' : ''}">{lang.toUpperCase()}</span>
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

    <!-- ── Footer ── -->
    <div class="modal-footer">
      <button onclick={onclose} class="footer-btn footer-btn--ghost">Fermer</button>
      {#if canRename}
        <button onclick={confirm} disabled={!editValue.trim()} class="footer-btn footer-btn--primary">
          <CircleCheck class="w-3.5 h-3.5" />
          Renommer
        </button>
      {/if}
    </div>

  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 9999;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    backdrop-filter: blur(3px);
  }

  .modal {
    width: 540px;
    max-width: 95vw;
    max-height: 88vh;
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    display: flex;
    flex-direction: column;
    box-shadow: 0 32px 80px rgba(0, 0, 0, 0.55);
    overflow: hidden;
  }

  /* ── Header ── */
  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }
  .header-left {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }
  .file-icon {
    width: 32px;
    height: 32px;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--color-accent) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-accent) 25%, transparent);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  .header-title {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .header-name {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    font-weight: 600;
    color: var(--color-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 400px;
  }
  .status-pill {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }
  .close-btn {
    width: 28px;
    height: 28px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-xs);
    border: 1px solid transparent;
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    flex-shrink: 0;
    transition: background 0.1s, color 0.1s, border-color 0.1s;
  }
  .close-btn:hover {
    background: var(--color-surface);
    border-color: var(--color-border);
    color: var(--color-text);
  }

  /* ── Body ── */
  .modal-body {
    flex: 1;
    overflow-y: auto;
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-height: 0;
  }

  /* ── Metrics ── */
  .metrics-row {
    display: flex;
    align-items: center;
    padding: 10px 12px;
    border-radius: var(--radius-md);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    flex-shrink: 0;
  }
  .metric {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 3px;
  }
  .metric-icon {
    width: 12px;
    height: 12px;
    color: var(--color-subtext);
  }
  .metric-val {
    font-family: "Geist Mono", monospace;
    font-size: 13px;
    font-weight: 700;
    color: var(--color-text);
    line-height: 1;
  }
  .metric-label {
    font-size: 8px;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--color-subtext);
    font-family: "Geist Mono", monospace;
  }
  .metric-div {
    width: 1px;
    height: 28px;
    background: var(--color-border);
    flex-shrink: 0;
  }

  /* ── Langs ── */
  .langs-row {
    display: flex;
    gap: 12px;
  }
  .lang-group {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .lang-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .lang-chip {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: var(--radius-full);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-subtext);
  }
  .lang-chip--audio {
    background: color-mix(in srgb, var(--color-accent) 12%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 35%, transparent);
    color: var(--color-accent);
  }
  .lang-chip--sub {
    background: color-mix(in srgb, var(--color-success) 10%, transparent);
    border-color: color-mix(in srgb, var(--color-success) 30%, transparent);
    color: var(--color-success);
  }

  /* ── Section ── */
  .section {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .section-title {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--color-subtext);
  }
  .empty-label {
    font-size: 10px;
    font-style: italic;
    color: var(--color-subtext);
  }

  /* ── Streams — hauteur max fixe ── */
  .stream-table {
    display: flex;
    flex-direction: column;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    overflow-y: auto;
    max-height: 180px;
    flex-shrink: 0;
  }
  .stream-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 5px 10px;
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    border-bottom: 1px solid color-mix(in srgb, var(--color-border) 40%, transparent);
    flex-shrink: 0;
  }
  .stream-row:last-child { border-bottom: none; }
  .stream-row:nth-child(even) {
    background: color-mix(in srgb, var(--color-surface) 60%, transparent);
  }
  .stream-type {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--color-accent);
    font-weight: 600;
    min-width: 72px;
  }
  .stream-codec { color: var(--color-text); flex: 1; }
  .stream-dim   { color: var(--color-subtext); }
  .stream-lang  {
    font-size: 9px;
    padding: 1px 5px;
    border-radius: var(--radius-full);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-subtext2);
  }

  /* ── Result grid ── */
  .result-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 6px;
  }
  .result-cell {
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding: 7px 10px;
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
  }
  .result-label {
    font-size: 8px;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--color-subtext);
    font-family: "Geist Mono", monospace;
  }
  .result-val {
    font-family: "Geist Mono", monospace;
    font-size: 12px;
    font-weight: 700;
    color: var(--color-text);
  }

  /* ── Rename separator ── */
  .rename-sep {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 2px 0;
  }
  .rename-sep::before,
  .rename-sep::after {
    content: "";
    flex: 1;
    height: 1px;
    background: var(--color-border);
  }
  .rename-sep-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--color-subtext);
    white-space: nowrap;
  }

  /* ── Input ── */
  .input-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .name-input {
    width: 100%;
    padding: 7px 12px;
    font-family: "Geist Mono", monospace;
    font-size: 12px;
    color: var(--color-text);
    background: var(--color-surface);
    border: 1px solid var(--color-accent);
    border-radius: var(--radius-sm);
    outline: none;
    transition: box-shadow 0.15s;
  }
  .name-input:focus {
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-accent) 18%, transparent);
  }
  .reset-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-accent);
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 0;
    opacity: 0.8;
    transition: opacity 0.1s;
  }
  .reset-btn:hover { opacity: 1; }

  /* ── Suggestion ── */
  .loading-row {
    display: flex;
    align-items: center;
    gap: 6px;
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
  }
  .suggestion-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    padding: 7px 12px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-subtext);
    cursor: pointer;
    width: 100%;
    overflow: hidden;
    transition: background 0.12s, border-color 0.12s, color 0.12s;
  }
  .suggestion-btn:hover {
    background: var(--color-panel2, var(--color-panel));
    border-color: var(--color-subtext2);
    color: var(--color-text);
  }
  .suggestion-btn--applied {
    background: color-mix(in srgb, var(--color-success) 10%, transparent);
    border-color: color-mix(in srgb, var(--color-success) 35%, transparent);
    color: var(--color-success);
  }
  .suggestion-btn--applied:hover {
    background: color-mix(in srgb, var(--color-success) 15%, transparent);
    color: var(--color-success);
  }

  /* ── Footer ── */
  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    padding: 10px 16px;
    border-top: 1px solid var(--color-border);
    flex-shrink: 0;
  }
  .footer-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 16px;
    border-radius: var(--radius-sm);
    border: 1px solid transparent;
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.12s, border-color 0.12s, color 0.12s, opacity 0.12s;
  }
  .footer-btn:disabled { opacity: 0.35; cursor: not-allowed; }
  .footer-btn--primary {
    background: var(--color-accent);
    border-color: var(--color-accent);
    color: #fff;
  }
  .footer-btn--primary:hover:not(:disabled) {
    background: color-mix(in srgb, var(--color-accent) 85%, #000);
  }
  .footer-btn--ghost {
    background: transparent;
    border-color: var(--color-border);
    color: var(--color-subtext);
  }
  .footer-btn--ghost:hover:not(:disabled) {
    background: var(--color-surface);
    border-color: var(--color-subtext2);
    color: var(--color-text);
  }
</style>