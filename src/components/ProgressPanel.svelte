<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { formatTime, formatSize, gainPct } from "$lib/utils";
  import { Progress } from "@skeletonlabs/skeleton-svelte";
  import {
    CircleCheck,
    TvMinimalPlay,
    AlertTriangle,
    Zap,
    Package,
    Clock,
    Subtitles,
    ChevronRight,
    FilmIcon,
  } from '@lucide/svelte';

  let p = $derived(encoder.progress);
  let s = $derived(encoder.summary);
  let sp = $derived(encoder.subExtractProgress);
  let extractingSubs = $derived(encoder.extractingSubs);

  // Fichiers actifs = la sélection si elle est active, sinon tous les fichiers
  let activeFiles = $derived.by(() => {
    if (encoder.encodeSelectionMode && encoder.selectedForEncoding.size > 0) {
      return encoder.files.filter(f => encoder.selectedForEncoding.has(f.path));
    }
    if (encoder.extractSelectionMode && encoder.selectedForExtraction.size > 0) {
      return encoder.files.filter(f => encoder.selectedForExtraction.has(f.path));
    }
    return encoder.files;
  });

  let isSelectionActive = $derived(
    (encoder.encodeSelectionMode && encoder.selectedForEncoding.size > 0) ||
    (encoder.extractSelectionMode && encoder.selectedForExtraction.size > 0)
  );

  let totalPercent = $derived(
    p ? ((p.file_index + p.percent / 100) / p.file_total) * 100 : 0,
  );

  let doneFiles = $derived(
    activeFiles.filter(f => f.status === "done" && f.result && f.result.original_mb > 0)
  );

  let avgRatio = $derived(
    doneFiles.length === 0 ? null
      : doneFiles.reduce((sum, f) => sum + f.result!.encoded_mb / f.result!.original_mb, 0) / doneFiles.length
  );

  let totalOriginalMb = $derived(activeFiles.reduce((sum, f) => sum + f.size_mb, 0));
  let realEncodedMb = $derived(doneFiles.reduce((sum, f) => sum + f.result!.encoded_mb, 0));
  let remainingOriginalMb = $derived(
    activeFiles.filter(f => f.status !== "done" && f.status !== "error").reduce((sum, f) => sum + f.size_mb, 0)
  );

  let estimatedTotalMb = $derived(avgRatio === null ? null : realEncodedMb + remainingOriginalMb * avgRatio);
  let estimatedGainPct = $derived(
    estimatedTotalMb === null || totalOriginalMb === 0 ? null
      : ((totalOriginalMb - estimatedTotalMb) / totalOriginalMb) * 100
  );

  // Résout le fichier actuellement en cours depuis p.file_name (nom source brut)
  let currentFile = $derived(
    p ? (encoder.files.find(f => f.filename === p.file_name || f.path.endsWith(p.file_name)) ?? null) : null
  );

  // Résout le fichier en cours d'extraction depuis sp.file_name
  let spFile = $derived(
    sp ? (encoder.files.find(f => f.filename === sp.file_name || f.path.endsWith(sp.file_name)) ?? null) : null
  );

  let nextFile = $derived(activeFiles.find(f => f.status === "queued") ?? null);
  let nextSubFile = $derived(
    extractingSubs
      ? activeFiles.find(f => f.sub_extract_status === "none") ?? null
      : null
  );

  let failedFiles = $derived(s ? s.files.filter(f => f.status !== "ok") : []);
  let okCount = $derived(s ? s.files.filter(f => f.status === "ok").length : 0);

  const radius = 46;
  const circumference = 2 * Math.PI * radius;
  function getOffset(percent: number) {
    return circumference - (percent / 100) * circumference;
  }
</script>

<div class="panel">

  <!-- ── IDLE ── -->
  {#if !encoder.encoding && !extractingSubs && !s}
    {#if encoder.files.length === 0}
      <div class="idle">
        <div class="idle-icon">
          <TvMinimalPlay class="w-7 h-7" />
        </div>
        <p class="idle-title">Prêt à encoder</p>
        <p class="idle-sub">Ajouter des fichiers</p>
      </div>
    {:else}
      {@const readyCount  = activeFiles.filter(f => f.status === "ready").length}
      {@const errorCount  = activeFiles.filter(f => f.status === "error").length}
      {@const totalMb     = activeFiles.reduce((s, f) => s + f.size_mb, 0)}
      {@const hasSubFiles = activeFiles.some(f => f.sub_langs.length > 0)}
      {@const subCount    = activeFiles.reduce((s, f) => s + f.sub_langs.length, 0)}

      <div class="idle-ready">
        <div class="idle-ready-header">
          <span class="idle-ready-dot"></span>
          <span class="idle-ready-label">{isSelectionActive ? 'Sélection' : 'Prêt'}</span>
          <span class="idle-ready-count">{activeFiles.length} fichier{activeFiles.length > 1 ? 's' : ''}{isSelectionActive ? ` / ${encoder.files.length}` : ''}</span>
        </div>

        <div class="idle-metrics">
          <div class="idle-metric">
            <span class="idle-metric-val">{readyCount}</span>
            <span class="idle-metric-label">En attente</span>
          </div>
          <div class="idle-metric-sep"></div>
          <div class="idle-metric">
            <span class="idle-metric-val">{formatSize(totalMb)}</span>
            <span class="idle-metric-label">Total</span>
          </div>
          {#if hasSubFiles}
            <div class="idle-metric-sep"></div>
            <div class="idle-metric">
              <span class="idle-metric-val">{subCount}</span>
              <span class="idle-metric-label">Pistes sub</span>
            </div>
          {/if}
          {#if errorCount > 0}
            <div class="idle-metric-sep"></div>
            <div class="idle-metric">
              <span class="idle-metric-val danger">{errorCount}</span>
              <span class="idle-metric-label">Erreur{errorCount > 1 ? 's' : ''}</span>
            </div>
          {/if}
        </div>

        <div class="idle-cta">
          <TvMinimalPlay class="w-5 h-5" style="color: var(--color-subtext2);" />
          <p class="idle-cta-text">Configurez les options et lancez l'encodage</p>
        </div>
      </div>
    {/if}

  <!-- ── ENCODAGE ── -->
  {:else if encoder.encoding}
    <div class="section-header">
      <span class="dot dot-pulse"></span>
      <span class="header-label">Encodage</span>
      {#if p && p.file_total > 1}
        <span class="header-counter">{p.file_index + 1} / {p.file_total}</span>
      {/if}
      {#if p}
        <span class="header-speed"><Zap class="w-3 h-3" />{p.speed?.toFixed(1)}x</span>
      {/if}
    </div>

    <div class="encode-body">
      {#if p}
        <!-- Arc de progression -->
        <div class="arc-wrap">
          <svg class="arc-svg" viewBox="0 0 120 120">
            <!-- Track -->
            <circle cx="60" cy="60" r={radius} fill="none" stroke="var(--color-surface)" stroke-width="8" />
            <!-- Glow ring -->
            <circle cx="60" cy="60" r={radius} fill="none" stroke="var(--color-accent)" stroke-width="8"
              stroke-linecap="round"
              stroke-dasharray={circumference}
              stroke-dashoffset={getOffset(p.percent)}
              class="arc-progress"
              filter="url(#glow)"
            />
            <defs>
              <filter id="glow" x="-20%" y="-20%" width="140%" height="140%">
                <feGaussianBlur stdDeviation="2.5" result="blur" />
                <feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge>
              </filter>
            </defs>
          </svg>
          <div class="arc-center">
            <span class="arc-pct">{Math.floor(p.percent)}<span class="arc-pct-sym">%</span></span>
          </div>
        </div>

        <!-- Infos à droite -->
        <div class="encode-info">
          <p class="file-name" title={currentFile ? currentFile.output_name + currentFile.output_ext : p.file_name}>
            {currentFile ? currentFile.output_name + currentFile.output_ext : p.file_name}
          </p>

          <div class="stats-row">
            <div class="stat">
              <span class="stat-label">Restant</span>
              <span class="stat-val">{formatTime(p.remaining_file)}</span>
            </div>
            <div class="stat-sep"></div>
            <div class="stat">
              <span class="stat-label">Vitesse</span>
              <span class="stat-val">{p.speed?.toFixed(1)}x</span>
            </div>
            <div class="stat-sep"></div>
            <div class="stat">
              <span class="stat-label">Total</span>
              <span class="stat-val">{p.remaining_total > 0 ? formatTime(p.remaining_total) : '—'}</span>
            </div>
          </div>

          {#if p.file_total > 1}
            <div class="global-progress">
              <div class="global-progress-header">
                <span>Progression globale</span>
                <span>{totalPercent.toFixed(0)}%</span>
              </div>
              <Progress value={totalPercent} max={100}>
                <Progress.Track class="progress-track">
                  <Progress.Range class="progress-range" />
                </Progress.Track>
              </Progress>
            </div>
          {/if}

          <div class="encode-footer">
            {#if estimatedTotalMb !== null}
              <span class="footer-chip">
                <Package class="w-3 h-3" />
                Estimé {formatSize(estimatedTotalMb)}
              </span>
              <span class="footer-chip">
                <FilmIcon class="w-3 h-3" />
                Original {formatSize(totalOriginalMb)}
              </span>
            {/if}
            {#if nextFile}
              <span class="footer-chip next-chip">
                <ChevronRight class="w-3 h-3" />
                <span class="truncate">{nextFile.output_name}{nextFile.output_ext}</span>
              </span>
            {/if}
          </div>
        </div>
      {/if}
    </div>

  <!-- ── EXTRACTION SOUS-TITRES ── -->
  {:else if extractingSubs}
    <div class="section-header">
      <span class="dot dot-pulse"></span>
      <span class="header-label">Extraction sous-titres</span>
      {#if sp && sp.file_total > 1}
        <span class="header-counter">{sp.file_index + 1} / {sp.file_total}</span>
      {/if}
    </div>

    <div class="extract-body">
      {#if sp}
        {@const pct = sp.file_total > 0 ? (sp.file_index / sp.file_total) * 100 : 0}
        <div class="extract-content">
          <div class="extract-icon-wrap">
            <Subtitles class="w-5 h-5" style="color: var(--color-accent);" />
          </div>
          <p class="file-name text-center" title={spFile ? spFile.output_name + spFile.output_ext : sp.file_name}>
            {spFile ? spFile.output_name + spFile.output_ext : sp.file_name}
          </p>

          <div class="global-progress" style="width: 100%; max-width: 340px;">
            <div class="global-progress-header">
              <span>Fichier {sp.file_index + 1} sur {sp.file_total}</span>
              <span>{pct.toFixed(0)}%</span>
            </div>
            <Progress value={pct} max={100}>
              <Progress.Track class="progress-track">
                <Progress.Range class="progress-range" />
              </Progress.Track>
            </Progress>
          </div>

          {#if nextSubFile}
            <span class="footer-chip next-chip" style="max-width: 340px;">
              <ChevronRight class="w-3 h-3 shrink-0" />
              <span class="truncate">{nextSubFile.output_name}{nextSubFile.output_ext}</span>
            </span>
          {/if}
        </div>
      {:else}
        <div class="extract-content">
          <div class="extract-icon-wrap">
            <Subtitles class="w-5 h-5 animate-pulse" style="color: var(--color-accent);" />
          </div>
          <p class="idle-sub">Initialisation…</p>
        </div>
      {/if}
    </div>

  <!-- ── RÉSUMÉ ── -->
  {:else if s}
    <div class="section-header" style="border-color: color-mix(in srgb, var(--color-success) 20%, var(--color-border));">
      <CircleCheck class="w-3.5 h-3.5" style="color: var(--color-success);" />
      <span class="header-label" style="color: var(--color-success);">Terminé</span>
    </div>

    <div class="summary-body">
      <!-- Métriques principales -->
      <div class="summary-metrics">
        <div class="metric">
          <span class="metric-val">{okCount}<span class="metric-sub">/{s.files.length}</span></span>
          <span class="metric-label">Succès</span>
        </div>
        <div class="metric-divider"></div>
        <div class="metric">
          <span class="metric-val success">{gainPct(s.total_original_mb, s.total_encoded_mb)}</span>
          <span class="metric-label">Réduction</span>
        </div>
        <div class="metric-divider"></div>
        <div class="metric">
          <span class="metric-val">{formatTime(s.total_secs)}</span>
          <span class="metric-label">Durée</span>
        </div>
      </div>

      <!-- Résultat -->
      <div class="summary-result">
        {#if failedFiles.length > 0}
          <div class="failed-header">
            <AlertTriangle class="w-3 h-3" />
            <span>{failedFiles.length} fichier{failedFiles.length > 1 ? 's' : ''} en échec</span>
          </div>
          <div class="failed-list">
            {#each failedFiles as r}
              {@const rFile = encoder.files.find(f => f.filename === r.name || f.path === r.path) ?? null}
              {@const displayName = rFile ? rFile.output_name + rFile.output_ext : r.name}
              <div class="failed-row">
                <span class="failed-name" title={displayName}>{displayName}</span>
                <span class="failed-badge">{r.status === 'error' ? 'Échec' : 'Annulé'}</span>
              </div>
            {/each}
          </div>
        {:else}
          <div class="success-message">
            <div class="success-accent"></div>
            <div>
              <p class="success-title">Traitement réussi</p>
              <p class="success-sub">Tous les fichiers ont été encodés sans erreur.</p>
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  /* ── Conteneur ── */
  .panel {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-panel);
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* ── Header commun ── */
  .section-header {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 6px 12px;
    border-bottom: 1px solid var(--color-border);
    background: color-mix(in srgb, var(--color-accent) 4%, transparent);
    flex-shrink: 0;
  }
  .dot {
    width: 6px; height: 6px;
    border-radius: 50%;
    background: var(--color-accent);
    flex-shrink: 0;
  }
  .dot-pulse { animation: pulse 1.8s ease-in-out infinite; }
  @keyframes pulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50%       { opacity: 0.4; transform: scale(0.85); }
  }
  .header-label {
    font-family: 'Geist Mono', monospace;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--color-text);
  }
  .header-counter {
    font-family: 'Geist Mono', monospace;
    font-size: 10px;
    color: var(--color-subtext);
  }
  .header-speed {
    margin-left: auto;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-family: 'Geist Mono', monospace;
    font-size: 10px;
    color: var(--color-subtext);
  }

  /* ── IDLE vide ── */
  .idle {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 16px;
  }
  .idle-icon {
    width: 44px; height: 44px;
    border-radius: 50%;
    background: color-mix(in srgb, var(--color-subtext) 6%, transparent);
    display: flex; align-items: center; justify-content: center;
    color: color-mix(in srgb, var(--color-subtext) 30%, transparent);
    margin-bottom: 4px;
  }
  .idle-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-subtext);
  }
  .idle-sub {
    font-size: 11px;
    color: color-mix(in srgb, var(--color-subtext) 60%, transparent);
  }

  /* ── ENCODAGE ── */
  .encode-body {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 14px;
    overflow: hidden;
    min-height: 0;
  }

  /* Arc SVG */
  .arc-wrap {
    position: relative;
    width: 120px;
    height: 120px;
    flex-shrink: 0;
  }
  .arc-svg {
    width: 100%; height: 100%;
    transform: rotate(-90deg);
  }
  .arc-progress {
    transition: stroke-dashoffset 0.35s cubic-bezier(.4,0,.2,1);
    stroke: var(--color-accent);
  }
  .arc-center {
    position: absolute;
    inset: 0;
    display: flex; align-items: center; justify-content: center;
  }
  .arc-pct {
    font-family: 'Geist Mono', monospace;
    font-size: 24px;
    font-weight: 700;
    color: var(--color-text);
    line-height: 1;
  }
  .arc-pct-sym {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-subtext);
  }

  /* Infos encodage */
  .encode-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
    justify-content: center;
  }
  .file-name {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.3;
  }

  .stats-row {
    display: flex;
    align-items: center;
    gap: 0;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }
  .stat {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 5px 4px;
    gap: 1px;
  }
  .stat-label {
    font-size: 8px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--color-subtext);
    font-family: 'Geist Mono', monospace;
  }
  .stat-val {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
    font-weight: 700;
    color: var(--color-text);
  }
  .stat-sep {
    width: 1px;
    height: 28px;
    background: var(--color-border);
    flex-shrink: 0;
  }

  /* Barre globale */
  .global-progress { display: flex; flex-direction: column; gap: 4px; }
  .global-progress-header {
    display: flex;
    justify-content: space-between;
    font-family: 'Geist Mono', monospace;
    font-size: 9px;
    color: var(--color-subtext);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  :global(.progress-track) {
    height: 5px !important;
    border-radius: 999px !important;
    background: var(--color-surface) !important;
    border: 1px solid var(--color-border) !important;
  }
  :global(.progress-range) {
    border-radius: 999px !important;
    background: var(--color-accent) !important;
    transition: width 0.3s ease !important;
  }

  /* Footer chips */
  .encode-footer {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .footer-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-family: 'Geist Mono', monospace;
    font-size: 9px;
    color: var(--color-subtext);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 999px;
    padding: 2px 7px;
    white-space: nowrap;
    overflow: hidden;
    max-width: 180px;
    text-overflow: ellipsis;
  }
  .next-chip {
    color: color-mix(in srgb, var(--color-accent) 80%, var(--color-subtext));
    border-color: color-mix(in srgb, var(--color-accent) 25%, var(--color-border));
    background: color-mix(in srgb, var(--color-accent) 5%, var(--color-surface));
  }

  /* ── EXTRACTION ── */
  .extract-body {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 12px 20px;
    overflow: hidden;
  }
  .extract-content {
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
  }
  .extract-icon-wrap {
    width: 36px; height: 36px;
    border-radius: 50%;
    background: color-mix(in srgb, var(--color-accent) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-accent) 25%, transparent);
    display: flex; align-items: center; justify-content: center;
  }

  /* ── RÉSUMÉ ── */
  .summary-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-height: 0;
  }

  .summary-metrics {
    display: flex;
    align-items: center;
    padding: 10px 16px;
    gap: 0;
    border-bottom: 1px solid var(--color-border);
    background: color-mix(in srgb, var(--color-success) 3%, transparent);
    flex-shrink: 0;
  }
  .metric {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }
  .metric-val {
    font-family: 'Geist Mono', monospace;
    font-size: 18px;
    font-weight: 700;
    color: var(--color-text);
    line-height: 1;
  }
  .metric-val.success { color: var(--color-success); }
  .metric-sub {
    font-size: 12px;
    font-weight: 500;
    color: var(--color-subtext);
  }
  .metric-label {
    font-size: 8px;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--color-subtext);
    font-family: 'Geist Mono', monospace;
  }
  .metric-divider {
    width: 1px;
    height: 32px;
    background: var(--color-border);
    flex-shrink: 0;
  }

  .summary-result {
    flex: 1;
    overflow-y: auto;
    padding: 10px 14px;
    min-height: 0;
  }

  /* Succès */
  .success-message {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    height: 100%;
  }
  .success-accent {
    width: 3px;
    align-self: stretch;
    border-radius: 999px;
    background: var(--color-success);
    opacity: 0.5;
    flex-shrink: 0;
  }
  .success-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text);
  }
  .success-sub {
    font-size: 10px;
    color: var(--color-subtext);
    margin-top: 2px;
    line-height: 1.5;
  }

  /* Échecs */
  .failed-header {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 9px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--color-danger);
    margin-bottom: 6px;
  }
  .failed-list { display: flex; flex-direction: column; gap: 3px; }
  .failed-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--color-danger) 5%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-danger) 12%, transparent);
  }
  .failed-name {
    font-family: 'Geist Mono', monospace;
    font-size: 10px;
    color: var(--color-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    margin-right: 8px;
  }
  .failed-badge {
    font-family: 'Geist Mono', monospace;
    font-size: 8px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--color-danger);
    flex-shrink: 0;
  }

  /* ── IDLE avec fichiers ── */
  .idle-ready {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-height: 0;
  }

  .idle-ready-header {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 6px 12px;
    border-bottom: 1px solid var(--color-border);
    background: color-mix(in srgb, var(--color-success) 4%, transparent);
    flex-shrink: 0;
  }
  .idle-ready-dot {
    width: 6px; height: 6px;
    border-radius: 50%;
    background: var(--color-success);
    flex-shrink: 0;
  }
  .idle-ready-label {
    font-family: 'Geist Mono', monospace;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--color-success);
  }
  .idle-ready-count {
    font-family: 'Geist Mono', monospace;
    font-size: 10px;
    color: var(--color-subtext);
    margin-left: auto;
  }

  .idle-metrics {
    display: flex;
    align-items: center;
    padding: 8px 14px;
    border-bottom: 1px solid var(--color-border);
    background: color-mix(in srgb, var(--color-surface) 60%, transparent);
    flex-shrink: 0;
    gap: 0;
  }
  .idle-metric {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1px;
  }
  .idle-metric-val {
    font-family: 'Geist Mono', monospace;
    font-size: 15px;
    font-weight: 700;
    color: var(--color-text);
    line-height: 1;
  }
  .idle-metric-val.danger { color: var(--color-danger); }
  .idle-metric-label {
    font-size: 8px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--color-subtext);
    font-family: 'Geist Mono', monospace;
  }
  .idle-metric-sep {
    width: 1px;
    height: 24px;
    background: var(--color-border);
    flex-shrink: 0;
  }

  .idle-cta {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 16px;
    opacity: 0.5;
  }
  .idle-cta-text {
    font-size: 11px;
    color: var(--color-subtext);
    text-align: center;
    line-height: 1.5;
  }
</style>