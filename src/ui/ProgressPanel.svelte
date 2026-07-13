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
    PauseCircle,
    XCircle,
    Video,
    Volume2,
    Box,
  } from "@lucide/svelte";

  let p = $derived(encoder.progress);
  let s = $derived(encoder.summary);
  let sp = $derived(encoder.subExtractProgress);
  let extractingSubs = $derived(encoder.extractingSubs);
  let paused = $derived(encoder.paused);
  let wasCancelled = $derived(
    s ? s.files.some((f) => f.status === "cancelled") && s.files.filter((f) => f.status === "ok").length === 0 : false
  );
  let cancelledCount = $derived(s ? s.files.filter((f) => f.status === "cancelled").length : 0);

  let activeFiles = $derived.by(() => {
    if (encoder.encodeSelectionMode && encoder.selectedForEncoding.size > 0)
      return encoder.files.filter((f) =>
        encoder.selectedForEncoding.has(f.path),
      );
    if (encoder.extractSelectionMode && encoder.selectedForExtraction.size > 0)
      return encoder.files.filter((f) =>
        encoder.selectedForExtraction.has(f.path),
      );
    return encoder.files;
  });

  let isSelectionActive = $derived(
    (encoder.encodeSelectionMode && encoder.selectedForEncoding.size > 0) ||
      (encoder.extractSelectionMode && encoder.selectedForExtraction.size > 0),
  );

  let totalPercent = $derived(
    p ? ((p.file_index + p.percent / 100) / p.file_total) * 100 : 0,
  );

  let doneFiles = $derived(
    activeFiles.filter(
      (f) => f.status === "done" && f.result && f.result.original_mb > 0,
    ),
  );

  let avgRatio = $derived(
    doneFiles.length === 0
      ? null
      : doneFiles.reduce(
          (sum, f) => sum + f.result!.encoded_mb / f.result!.original_mb,
          0,
        ) / doneFiles.length,
  );

  let totalOriginalMb = $derived(
    activeFiles.reduce((sum, f) => sum + f.size_mb, 0),
  );
  let realEncodedMb = $derived(
    doneFiles.reduce((sum, f) => sum + f.result!.encoded_mb, 0),
  );
  let remainingOriginalMb = $derived(
    activeFiles
      .filter((f) => f.status !== "done" && f.status !== "error")
      .reduce((sum, f) => sum + f.size_mb, 0),
  );
  let estimatedTotalMb = $derived(
    avgRatio === null ? null : realEncodedMb + remainingOriginalMb * avgRatio,
  );
  let estimatedGainPct = $derived(
    estimatedTotalMb === null || totalOriginalMb === 0
      ? null
      : ((totalOriginalMb - estimatedTotalMb) / totalOriginalMb) * 100,
  );

  let speedKnown = $derived(p ? p.speed > 0.05 : false);
  let currentFile = $derived(
    p
      ? (encoder.files.find(
          (f) => f.filename === p.file_name || f.path.endsWith(p.file_name),
        ) ?? null)
      : null,
  );
  let spFile = $derived(
    sp
      ? (encoder.files.find(
          (f) => f.filename === sp.file_name || f.path.endsWith(sp.file_name),
        ) ?? null)
      : null,
  );
  let nextFile = $derived(
    activeFiles.find((f) => f.status === "queued") ?? null,
  );
  let nextSubFile = $derived(
    extractingSubs
      ? (activeFiles.find((f) => f.sub_extract_status === "none") ?? null)
      : null,
  );

  let failedFiles = $derived(s ? s.files.filter((f) => f.status !== "ok") : []);
  let okCount = $derived(
    s ? s.files.filter((f) => f.status === "ok").length : 0,
  );

  // ── Résumé EncodingSettings ─────────────────────────────────────────────
  let presetLabel = $derived.by(() => {
    const id = encoder.activePresetId;
    if (!id) return null;
    const labels: Record<string, string> = {
      fast: "Rapide", balanced: "Équilibré", quality: "Qualité", archive: "Archive",
    };
    return labels[id] ?? null;
  });
  let videoSummary = $derived(
    encoder.videoMode === "copy" ? "Copie" : `H.265 · CRF ${encoder.crf} · ${(encoder.preset ?? "p5").toUpperCase()}`
  );
  let audioSummary = $derived(
    encoder.audioMode === "copy" ? "Copie" : `AAC · ${encoder.audioBitrate}k`
  );
  let containerSummary = $derived((encoder.container ?? "mkv").toUpperCase());
  let subtitleSummary = $derived(encoder.subExtractFormat);
</script>

<div class="panel">
  <!-- ── IDLE vide ── -->
  {#if !encoder.encoding && !extractingSubs && !s}
    {#if encoder.files.length === 0}
      <div class="idle-empty">
        <div class="idle-empty-top">
          <TvMinimalPlay
            class="w-3.5 h-3.5 shrink-0"
            style="color: var(--color-subtext2);"
          />
          <span class="idle-empty-text">Prêt à encoder — ajoutez des fichiers</span>
        </div>
        <div class="settings-strip">
          <span class="setting-chip">
            <Zap class="w-3 h-3 shrink-0 chip-icon" />
            <span class="chip-label">Préréglage</span>
            <span class="chip-val">{presetLabel ?? "Custom"}</span>
          </span>
          <span class="strip-sep"></span>
          <span class="setting-chip">
            <Video class="w-3 h-3 shrink-0 chip-icon" />
            <span class="chip-label">Vidéo</span>
            <span class="chip-val">{videoSummary}</span>
          </span>
          <span class="strip-sep"></span>
          <span class="setting-chip">
            <Volume2 class="w-3 h-3 shrink-0 chip-icon" />
            <span class="chip-label">Audio</span>
            <span class="chip-val">{audioSummary}</span>
          </span>
          <span class="strip-sep"></span>
          <span class="setting-chip">
            <Box class="w-3 h-3 shrink-0 chip-icon" />
            <span class="chip-label">Conteneur</span>
            <span class="chip-val">{containerSummary}</span>
          </span>
          <span class="strip-sep"></span>
          <span class="setting-chip">
            <Subtitles class="w-3 h-3 shrink-0 chip-icon" />
            <span class="chip-label">Sous-titres</span>
            <span class="chip-val">{subtitleSummary}</span>
          </span>
        </div>
      </div>
    {:else}
      {@const readyCount = activeFiles.filter(
        (f) => f.status === "ready",
      ).length}
      {@const errorCount = activeFiles.filter(
        (f) => f.status === "error",
      ).length}
      {@const totalMb = activeFiles.reduce((s, f) => s + f.size_mb, 0)}
      {@const hasSubFiles = activeFiles.some((f) => f.sub_langs.length > 0)}
      {@const subCount = activeFiles.reduce(
        (s, f) => s + f.sub_langs.length,
        0,
      )}

      <div class="grid-panel idle-ready-row">
        <!-- Ligne 1 : statut -->
        <div class="grid-row row-1">
          <span class="dot-ready"></span>
          <span class="state-label"
            >{isSelectionActive ? "Sélection" : "Prêt"}</span
          >
        </div>
        <!-- Ligne 2 : métriques -->
        <div class="grid-row row-2">
          <span class="meta-item">
            <span class="meta-val">{activeFiles.length}</span>
            <span class="meta-label"
              >fichier{activeFiles.length > 1 ? "s" : ""}</span
            >
          </span>
          <span class="meta-item">
            <span class="meta-val">{formatSize(totalMb)}</span>
            <span class="meta-label">total</span>
          </span>
          {#if hasSubFiles}
            <span class="meta-item">
              <span class="meta-val">{subCount}</span>
              <span class="meta-label">pistes sub</span>
            </span>
          {/if}
          {#if errorCount > 0}
            <span class="meta-item">
              <span class="meta-val danger">{errorCount}</span>
              <span class="meta-label">erreur{errorCount > 1 ? "s" : ""}</span>
            </span>
          {/if}
        </div>
        <!-- Ligne 3 : résumé des réglages d'encodage -->
        <div class="grid-row row-3">
          <div class="settings-strip">
            <span class="setting-chip">
              <Zap class="w-3 h-3 shrink-0 chip-icon" />
              <span class="chip-label">Préréglage</span>
              <span class="chip-val">{presetLabel}</span>
            </span>
            <span class="strip-sep"></span>
            <span class="setting-chip">
              <Video class="w-3 h-3 shrink-0 chip-icon" />
              <span class="chip-label">Vidéo</span>
              <span class="chip-val">{videoSummary}</span>
            </span>
            <span class="strip-sep"></span>
            <span class="setting-chip">
              <Volume2 class="w-3 h-3 shrink-0 chip-icon" />
              <span class="chip-label">Audio</span>
              <span class="chip-val">{audioSummary}</span>
            </span>
            <span class="strip-sep"></span>
            <span class="setting-chip">
              <Box class="w-3 h-3 shrink-0 chip-icon" />
              <span class="chip-label">Conteneur</span>
              <span class="chip-val">{containerSummary}</span>
            </span>
            <span class="strip-sep"></span>
            <span class="setting-chip">
              <Subtitles class="w-3 h-3 shrink-0 chip-icon" />
              <span class="chip-label">Sous-titres</span>
              <span class="chip-val">{subtitleSummary}</span>
            </span>
          </div>
        </div>
      </div>
    {/if}

    <!-- ── ENCODAGE ── -->
  {:else if encoder.encoding}
    <div class="grid-panel encode-row">
      <!-- Ligne 1 : statut + numéro -->
      <div class="grid-row row-1">
        {#if paused}
          <PauseCircle class="w-3.5 h-3.5 shrink-0" style="color: var(--color-warning);" />
          <span class="state-label" style="color: var(--color-warning);">En pause</span>
        {:else}
          <span class="dot dot-pulse"></span>
          <span class="state-label">Encodage</span>
        {/if}
        {#if p && p.file_total > 1}
          <span class="counter">{p.file_index + 1} / {p.file_total}</span>
        {/if}
      </div>

      {#if p}
        <!-- Ligne 2 : nom fichier · barre fichier · temps fichier -->
        <div class="grid-row row-2">
          <span
            class="filename"
            title={currentFile
              ? currentFile.output_name + currentFile.output_ext
              : p.file_name}
          >
            {currentFile
              ? currentFile.output_name + currentFile.output_ext
              : p.file_name}
          </span>
          <div class="spacer"></div>
          <div class="progress-wrap">
            <Progress value={p.percent} max={100}>
              <Progress.Track class="progress-track">
                <Progress.Range class="progress-range" />
              </Progress.Track>
            </Progress>
            <span class="progress-pct">{Math.floor(p.percent)}%</span>
          </div>
          <div class="vsep"></div>
          <span class="meta-item">
            <Clock
              class="w-3 h-3 shrink-0"
              style="color: var(--color-subtext2);"
            />
            <span class="meta-val"
              >{speedKnown ? "" : "~"}{formatTime(p.remaining_file)}</span
            >
          </span>
        </div>

        <!-- Ligne 3 : fichier suivant · barre globale · vitesse · temps total -->
        <div class="grid-row row-3">
          {#if nextFile}
            <span class="next-chip">
              <ChevronRight class="w-3 h-3 shrink-0" />
              <span class="truncate"
                >{nextFile.output_name}{nextFile.output_ext}</span
              >
            </span>
          {:else}
            <span class="meta-label" style="opacity:0.4;">—</span>
          {/if}
          <div class="spacer"></div>
          {#if p.file_total > 1}
            <div class="progress-wrap">
              <Progress value={totalPercent} max={100}>
                <Progress.Track class="progress-track progress-track--muted">
                  <Progress.Range
                    class="progress-range progress-range--muted"
                  />
                </Progress.Track>
              </Progress>
              <span class="progress-pct muted">{totalPercent.toFixed(0)}%</span>
            </div>
            <div class="vsep"></div>
          {/if}
          <span class="meta-item">
            <Zap
              class="w-3 h-3 shrink-0"
              style="color: var(--color-subtext2);"
            />
            <span class="meta-val">{p.speed?.toFixed(1)}x</span>
          </span>
          {#if p.remaining_total > 0}
            <div class="vsep"></div>
            <span class="meta-item">
              <span class="meta-label">total</span>
              <span class="meta-val"
                >{speedKnown ? "" : "~"}{formatTime(p.remaining_total)}</span
              >
            </span>
          {/if}
          {#if estimatedGainPct !== null}
            <div class="vsep"></div>
            <span class="gain-chip">{estimatedGainPct.toFixed(1)}%</span>
          {/if}
        </div>
      {/if}
    </div>

    <!-- ── EXTRACTION SOUS-TITRES ── -->
  {:else if extractingSubs}
    <div class="grid-panel encode-row">
      <!-- Ligne 1 : statut -->
      <div class="grid-row row-1">
        <span class="dot dot-pulse"></span>
        <span class="state-label">Extraction sous-titres</span>
        {#if sp && sp.file_total > 1}
          <span class="counter">{sp.file_index + 1} / {sp.file_total}</span>
        {/if}
      </div>

      {#if sp}
        {@const pct =
          sp.file_total > 0 ? (sp.file_index / sp.file_total) * 100 : 0}

        <!-- Ligne 2 : nom fichier · barre · % -->
        <div class="grid-row row-2">
          <span
            class="filename"
            title={spFile
              ? spFile.output_name + spFile.output_ext
              : sp.file_name}
          >
            <Subtitles
              class="w-3 h-3 shrink-0"
              style="color: var(--color-accent);"
            />
            {spFile ? spFile.output_name + spFile.output_ext : sp.file_name}
          </span>
          <div class="spacer"></div>
          <div class="progress-wrap">
            <Progress value={pct} max={100}>
              <Progress.Track class="progress-track">
                <Progress.Range class="progress-range" />
              </Progress.Track>
            </Progress>
            <span class="progress-pct">{pct.toFixed(0)}%</span>
          </div>
        </div>

        <!-- Ligne 3 : fichier suivant -->
        <div class="grid-row row-3">
          {#if nextSubFile}
            <span class="next-chip">
              <ChevronRight class="w-3 h-3 shrink-0" />
              <span class="truncate"
                >{nextSubFile.output_name}{nextSubFile.output_ext}</span
              >
            </span>
          {/if}
        </div>
      {:else}
        <div class="grid-row row-2">
          <span class="idle-cta-hint">Initialisation…</span>
        </div>
        <div class="grid-row row-3"></div>
      {/if}
    </div>

    <!-- ── RÉSUMÉ ── -->
  {:else if s}
    <div class="grid-panel {wasCancelled ? 'cancelled-row' : 'summary-row'}">
      <!-- Ligne 1 : statut -->
      <div class="grid-row row-1">
        {#if wasCancelled}
          <XCircle class="w-3.5 h-3.5 shrink-0" style="color: var(--color-warning);" />
          <span class="state-label" style="color: var(--color-warning);">Annulé</span>
        {:else}
          <CircleCheck
            class="w-3.5 h-3.5 shrink-0"
            style="color: var(--color-success);"
          />
          <span class="state-label" style="color: var(--color-success);"
            >Terminé</span
          >
        {/if}
      </div>

      <!-- Ligne 2 : métriques -->
      <div class="grid-row row-2">
        <span class="meta-item">
          <span class="meta-val">{okCount}/{s.files.length}</span>
          <span class="meta-label">succès</span>
        </span>
        {#if cancelledCount > 0}
          <div class="vsep"></div>
          <span class="meta-item">
            <span class="meta-val" style="color: var(--color-warning);">{cancelledCount}</span>
            <span class="meta-label">annulé{cancelledCount > 1 ? "s" : ""}</span>
          </span>
        {/if}
        <div class="vsep"></div>
        <span class="meta-item">
          <span class="meta-val success"
            >{gainPct(s.total_original_mb, s.total_encoded_mb)}</span
          >
          <span class="meta-label">réduction</span>
        </span>
        <div class="vsep"></div>
        <span class="meta-item">
          <Clock
            class="w-3 h-3 shrink-0"
            style="color: var(--color-subtext2);"
          />
          <span class="meta-val">{formatTime(s.total_secs)}</span>
        </span>
      </div>

      <!-- Ligne 3 : échecs éventuels -->
      <div class="grid-row row-3">
        {#if failedFiles.length > 0}
          <span class="meta-item">
            <AlertTriangle
              class="w-3 h-3 shrink-0"
              style="color: var(--color-danger);"
            />
            <span class="meta-val danger"
              >{failedFiles.length} échec{failedFiles.length > 1
                ? "s"
                : ""}</span
            >
          </span>
          <div class="failed-pills">
            {#each failedFiles as r}
              {@const rFile =
                encoder.files.find(
                  (f) => f.filename === r.name || f.path === r.path,
                ) ?? null}
              {@const displayName = rFile
                ? rFile.output_name + rFile.output_ext
                : r.name}
              <span class="failed-pill" title={displayName}>{displayName}</span>
            {/each}
          </div>
        {:else}
          <span class="idle-cta-hint success-hint"
            >Tous les fichiers encodés sans erreur</span
          >
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  /* ── Conteneur ─────────────────────────────────────────────── */
  .panel {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-panel);
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* ── Grille 3 lignes ────────────────────────────────────────── */
  .grid-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 5px 12px;
    gap: 2px;
    min-height: 0;
    justify-content: center;
  }

  .grid-row {
    display: flex;
    align-items: center;
    gap: 7px;
    min-width: 0;
    overflow: hidden;
  }

  /* Hauteurs des lignes */
  .row-1 {
    flex: 0 0 auto;
  }
  .row-2 {
    flex: 0 0 auto;
  }
  .row-3 {
    flex: 0 0 auto;
  }

  /* ── Séparateurs ────────────────────────────────────────────── */
  .vsep {
    width: 1px;
    height: 12px;
    background: var(--color-border);
    flex-shrink: 0;
    opacity: 0.6;
  }
  .spacer {
    flex: 1;
    min-width: 8px;
  }

  /* ── Dots ───────────────────────────────────────────────────── */
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--color-accent);
    flex-shrink: 0;
  }
  .dot-pulse {
    animation: pulse 1.8s ease-in-out infinite;
  }
  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
      transform: scale(1);
    }
    50% {
      opacity: 0.4;
      transform: scale(0.8);
    }
  }
  .dot-ready {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--color-success);
    flex-shrink: 0;
  }

  /* ── Textes ─────────────────────────────────────────────────── */
  .state-label {
    font-family: "Geist Mono", monospace;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--color-text);
    flex-shrink: 0;
  }
  .counter {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
    flex-shrink: 0;
  }
  .filename {
    font-size: 11px;
    font-weight: 500;
    color: var(--color-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 1;
    min-width: 0;
  }
  .meta-item {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }
  .meta-val {
    font-family: "Geist Mono", monospace;
    font-size: 12px;
    font-weight: 700;
    color: var(--color-text);
    line-height: 1;
  }
  .meta-val.danger {
    color: var(--color-danger);
  }
  .meta-val.success {
    color: var(--color-success);
  }
  .meta-label {
    font-family: "Geist Mono", monospace;
    font-size: 12px;
    color: var(--color-subtext);
  }

  /* ── Barres Skeleton ────────────────────────────────────────── */
  .progress-wrap {
    display: flex;
    align-items: center;
    gap: 5px;
    flex-shrink: 0;
  }
  :global(.progress-track) {
    height: 5px !important;
    border-radius: 999px !important;
    background: var(--color-surface) !important;
    border: 1px solid var(--color-border) !important;
    width: 150px;
  }
  :global(.progress-track--muted) {
    width: 150px !important;
    opacity: 0.9;
  }
  :global(.progress-range) {
    border-radius: 999px !important;
    background: var(--color-accent) !important;
    transition: width 0.3s ease !important;
  }
  :global(.progress-range--muted) {
    background: var(--color-subtext) !important;
  }
  .progress-pct {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    font-weight: 700;
    color: var(--color-text);
    min-width: 30px;
    text-align: right;
    flex-shrink: 0;
  }
  .progress-pct.muted {
    color: var(--color-subtext);
  }

  /* ── Chips ──────────────────────────────────────────────────── */
  .gain-chip {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 700;
    color: var(--color-success);
    background: color-mix(in srgb, var(--color-success) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-success) 20%, transparent);
    border-radius: 999px;
    padding: 1px 6px;
    flex-shrink: 0;
  }
  .next-chip {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: color-mix(in srgb, var(--color-accent) 80%, var(--color-subtext));
    background: color-mix(in srgb, var(--color-accent) 6%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-accent) 15%, transparent);
    border-radius: 999px;
    padding: 1px 7px 1px 5px;
    max-width: 200px;
    overflow: hidden;
    flex-shrink: 1;
  }
  .next-chip .truncate {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* ── IDLE ───────────────────────────────────────────────────── */
  .idle-empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 5px 12px;
  }
  .idle-empty-top {
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .idle-empty-text {
    font-size: 11px;
    color: var(--color-subtext);
  }
  .idle-cta-hint {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 10px;
    color: var(--color-subtext2);
    opacity: 0.6;
  }
  .success-hint {
    color: var(--color-success);
    opacity: 0.8;
  }

  /* ── Settings strip ────────────────────────────────────────── */
  .settings-strip {
    display: flex;
    align-items: center;
    gap: 0;
    overflow: hidden;
    min-width: 0;
    margin: 5px;
  }

  .setting-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
    white-space: nowrap;
    overflow: hidden;
  }

  .chip-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext2);
    letter-spacing: 0.02em;
    flex-shrink: 0;
  }

  .chip-val {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 600;
    color: var(--color-subtext);
    flex-shrink: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .strip-sep {
    width: 1px;
    height: 10px;
    background: var(--color-border);
    flex-shrink: 0;
    margin: 0 8px;
    opacity: 0.7;
  }

  /* ── Pills échec ────────────────────────────────────────────── */
  .failed-pills {
    display: flex;
    align-items: center;
    gap: 4px;
    overflow: hidden;
    flex: 1;
    min-width: 0;
  }
  .failed-pill {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-danger);
    background: color-mix(in srgb, var(--color-danger) 8%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-danger) 15%, transparent);
    border-radius: 999px;
    padding: 1px 7px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 160px;
    flex-shrink: 1;
  }


</style>