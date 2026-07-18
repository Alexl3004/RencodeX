<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import {
    X,
    Play,
    Pause,
    PlayCircle,
    FileDown,
    Loader2,
    SkipForward,
  } from "@lucide/svelte";
  import { ring2, dotPulse } from "ldrs";
  ring2.register();
  dotPulse.register();

  let readyCount = $derived(
    encoder.files.filter((f) => f.status === "ready").length,
  );

  let encodeTargetCount = $derived(
    encoder.encodeSelectionMode && encoder.selectedForEncoding.size > 0
      ? encoder.selectedForEncoding.size
      : readyCount,
  );

  let canEncode = $derived(!encoder.encoding && encodeTargetCount > 0);

  let encodeLabel = $derived(`Encoder (${encodeTargetCount})`);

  let extractTargetCount = $derived(
    encoder.extractSelectionMode && encoder.selectedForExtraction.size > 0
      ? encoder.selectedForExtraction.size
      : encoder.files.filter(
          (f) => f.status === "ready" && f.sub_langs.length > 0,
        ).length,
  );

  let extractLabel = $derived(`Extraire (${extractTargetCount})`);

  let showExtract = $derived(
    encoder.showExtractButton &&
      !encoder.encoding &&
      encoder.files.some((f) => f.status === "ready" && f.sub_langs.length > 0),
  );

  let canExtract = $derived(
    !encoder.extractingSubs &&
      encoder.selSubs.size > 0 &&
      !(
        encoder.extractSelectionMode && encoder.selectedForExtraction.size === 0
      ),
  );
</script>

<div class="control-bar">
  <!-- Encode -->
  <div class="btn-group">
    {#if encoder.encoding}
      <!-- Indicateur d'état -->
      <button class="cb-btn cb-btn--primary cb-btn--active" disabled>
        {#if encoder.paused}
          <Pause class="cb-icon" />
          <span>En pause…</span>
        {:else}
          <l-ring-2 size="16" stroke="2" color="var(--color-accent)" speed="0.8"
          ></l-ring-2>
          <span>Encodage…</span>
        {/if}
      </button>
      <!-- Pause / Reprendre -->
      <button
        onclick={() =>
          encoder.paused ? encoder.resumeEncoding() : encoder.pauseEncoding()}
        class="cb-btn cb-btn--ghost"
        title={encoder.paused ? "Reprendre l'encodage" : "Mettre en pause"}
      >
        {#if encoder.paused}
          <PlayCircle class="cb-icon" />
          <span>Reprendre</span>
        {:else}
          <Pause class="cb-icon" />
          <span>Pause</span>
        {/if}
      </button>
      <!-- ⏭ Skip — nouveau bouton -->
       {#if encoder.progress && encoder.progress.file_total > 1}
      <button
        onclick={() => encoder.skipEncoding()}
        class="cb-btn cb-btn--ghost"
        title="Ignorer ce fichier et passer au suivant"
        disabled={encoder.paused}
      >
        <SkipForward class="cb-icon" />
        <span>Suivant</span>
      </button>
      {/if}
    {:else}
      <button
        onclick={() => encoder.startEncoding()}
        disabled={!canEncode}
        class="cb-btn cb-btn--primary"
        title={canEncode ? encodeLabel : "Aucun fichier prêt"}
      >
        <Play class="cb-icon" fill="currentColor" stroke="none" />
        <span>{encodeLabel}</span>
      </button>
    {/if}
  </div>

  <!-- Extract -->
  {#if showExtract}
    <div class="bar-sep" aria-hidden="true"></div>

    <div class="btn-group">
      {#if encoder.extractingSubs}
        <button class="cb-btn cb-btn--ghost cb-btn--active" disabled>
          <l-ring-2 size="16" stroke="2" color="var(--color-accent)" speed="0.8"
          ></l-ring-2>
          <span>Extraction…</span>
        </button>
      {:else}
        <button
          onclick={() => encoder.startSubtitleExtraction()}
          disabled={!canExtract}
          class="cb-btn cb-btn--ghost"
          title={extractLabel}
        >
          <FileDown class="cb-icon" />
          <span>{extractLabel}</span>
        </button>
      {/if}
    </div>
  {/if}

  <!-- Annuler collé à droite -->
  {#if encoder.encoding || encoder.extractingSubs}
    <div class="cancel-slot">
      <button
        onclick={() =>
          encoder.encoding
            ? encoder.cancelEncoding()
            : encoder.cancelSubtitleExtraction()}
        class="cb-btn cb-btn--danger"
        title="Annuler"
      >
        <X class="cb-icon" />
        <span>Annuler</span>
      </button>
    </div>
  {/if}
</div>

<style>
  .control-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-top: 1px solid var(--color-border);
    background: var(--color-panel);
    flex-wrap: wrap;
  }

  .btn-group {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .cancel-slot {
    margin-left: auto;
  }

  /* ── Base button ────────────────────────────── */
  .cb-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 14px;
    border-radius: var(--radius-sm);
    border: 1px solid transparent;
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.02em;
    cursor: pointer;
    white-space: nowrap;
    transition:
      background 0.12s,
      border-color 0.12s,
      color 0.12s,
      opacity 0.12s;
    flex-shrink: 0;
  }

  .cb-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  /* ── Primary (encode) ───────────────────────── */
  .cb-btn--primary {
    background: var(--color-accent);
    border-color: var(--color-accent);
    color: #fff;
  }

  .cb-btn--primary:hover:not(:disabled) {
    background: color-mix(in srgb, var(--color-accent) 85%, #000);
    border-color: color-mix(in srgb, var(--color-accent) 85%, #000);
  }

  .cb-btn--primary.cb-btn--active {
    background: color-mix(in srgb, var(--color-accent) 20%, var(--color-panel));
    border-color: color-mix(
      in srgb,
      var(--color-accent) 40%,
      var(--color-border)
    );
    color: var(--color-accent);
  }

  /* ── Ghost (extract) ────────────────────────── */
  .cb-btn--ghost {
    background: transparent;
    border-color: var(--color-border);
    color: var(--color-subtext);
  }

  .cb-btn--ghost:hover:not(:disabled) {
    background: var(--color-panel2, var(--color-surface));
    border-color: var(--color-subtext);
    color: var(--color-text);
  }

  .cb-btn--ghost.cb-btn--active {
    background: transparent;
    border-color: var(--color-border);
    color: var(--color-subtext);
  }

  /* ── Danger (cancel) ────────────────────────── */
  .cb-btn--danger {
    background: transparent;
    border-color: color-mix(
      in srgb,
      var(--color-danger) 40%,
      var(--color-border)
    );
    color: var(--color-danger);
  }

  .cb-btn--danger:hover:not(:disabled) {
    background: color-mix(in srgb, var(--color-danger) 10%, var(--color-panel));
    border-color: var(--color-danger);
  }

  /* ── Separator ──────────────────────────────── */
  .bar-sep {
    width: 1px;
    height: 20px;
    background: var(--color-border);
    flex-shrink: 0;
    margin: 0 2px;
  }
</style>
