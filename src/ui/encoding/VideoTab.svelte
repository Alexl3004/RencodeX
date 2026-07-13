<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import type { VideoMode } from "$lib/stores/types";

  let videoMode = $derived(encoder.videoMode);
</script>

<section class="content-section">
  <header class="section-header">
    <div>
      <h2 class="section-title">Vidéo</h2>
      <p class="section-desc">
        La copie conserve le flux vidéo original sans aucune dégradation ni
        temps d'encodage. Les réglages CRF, preset et AQ sont ignorés.
      </p>
    </div>
  </header>

  <!-- ── Mode vidéo ─────────────────────────────────────────────────────── -->
  <div class="field-block">
    <div class="field-label">Mode</div>
    <div class="mode-grid">
      <!-- Encoder -->
      <button
        type="button"
        class="mode-card {videoMode === 'encode' ? 'mode-card--active' : ''}"
        onclick={() => encoder.setVideoMode("encode")}
      >
        <div class="mode-card-top">
          <div class="mode-icon mode-icon--encode" aria-hidden="true">
            <svg width="18" height="18" viewBox="0 0 18 18" fill="none">
              <rect x="1" y="3" width="16" height="12" rx="2" stroke="currentColor" stroke-width="1.5"/>
              <path d="M7 6.5L12 9L7 11.5V6.5Z" fill="currentColor"/>
            </svg>
          </div>
          <span class="mode-label">Encoder</span>
        </div>
        <p class="mode-desc">
          Réencode en H.265 via NVENC. Réduit la taille du fichier avec un
          contrôle fin sur la qualité.
        </p>
        <div class="mode-tags">
          <span class="mode-tag">hevc_nvenc</span>
          <span class="mode-tag">CRF actif</span>
          <span class="mode-tag">GPU</span>
        </div>
      </button>

      <!-- Copier -->
      <button
        type="button"
        class="mode-card {videoMode === 'copy' ? 'mode-card--active' : ''}"
        onclick={() => encoder.setVideoMode("copy")}
      >
        <div class="mode-card-top">
          <div class="mode-icon mode-icon--copy" aria-hidden="true">
            <svg width="18" height="18" viewBox="0 0 18 18" fill="none">
              <rect x="1" y="3" width="16" height="12" rx="2" stroke="currentColor" stroke-width="1.5"/>
              <path d="M5 9H13M10 6L13 9L10 12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </div>
          <span class="mode-label">Copier</span>
        </div>
        <p class="mode-desc">
          Copie le flux vidéo tel quel, sans réencodage. Remuxage instantané,
          qualité préservée à l'identique.
        </p>
        <div class="mode-tags">
          <span class="mode-tag">-c:v copy</span>
          <span class="mode-tag">Sans perte</span>
          <span class="mode-tag">Instantané</span>
        </div>
      </button>
    </div>
  </div>

  <!-- ── Callout contextuel ─────────────────────────────────────────────── -->
  {#if videoMode === "copy"}
    <div class="callout callout--info">
      <div class="callout-icon" aria-hidden="true">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
          <circle cx="7" cy="7" r="6" stroke="currentColor" stroke-width="1.4"/>
          <path d="M7 6V10M7 4.5V4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
        </svg>
      </div>
      <div class="callout-body">
        <span class="callout-title">Mode copie actif</span>
        <span class="callout-text">
          Les paramètres CRF, preset d'encodage, AQ et multipass sont ignorés.
          L'audio et les sous-titres continuent d'être traités normalement.
        </span>
      </div>
    </div>
  {:else}
    <div class="callout callout--muted">
      <div class="callout-icon" aria-hidden="true">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
          <circle cx="7" cy="7" r="6" stroke="currentColor" stroke-width="1.4"/>
          <path d="M5 7.5L6.5 9L9 5.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </div>
      <div class="callout-body">
        <span class="callout-title">Mode encodage actif</span>
        <span class="callout-text">
          Configurez la qualité dans l'onglet <strong>Préréglages</strong>.
          CRF, preset, AQ et multipass sont appliqués au rendu NVENC.
        </span>
      </div>
    </div>
  {/if}

  <!-- ── Tableau comparatif ─────────────────────────────────────────────── -->
  <div class="compare-block">
    <div class="field-label" style="margin-bottom: 12px;">Comparaison</div>
    <div class="compare-table">
      <div class="compare-header">
        <div class="compare-cell compare-cell--header compare-cell--prop"></div>
        <div class="compare-cell compare-cell--header {videoMode === 'encode' ? 'compare-cell--hl' : ''}">
          Encoder
        </div>
        <div class="compare-cell compare-cell--header {videoMode === 'copy' ? 'compare-cell--hl' : ''}">
          Copier
        </div>
      </div>
      {#each [
        { prop: "Qualité",        encode: "CRF réglable",     copy: "Identique source" },
        { prop: "Temps",          encode: "Long (GPU)",        copy: "Instantané"       },
        { prop: "Taille sortie",  encode: "Réduite",           copy: "Identique source" },
        { prop: "Codec sortie",   encode: "H.265 / HEVC",      copy: "Inchangé"         },
        { prop: "CRF / Preset",   encode: "✓ Actif",           copy: "— Ignoré"         },
        { prop: "AQ / Multipass", encode: "✓ Actif",           copy: "— Ignoré"         },
        { prop: "Audio",          encode: "✓ Traité",          copy: "✓ Traité"         },
        { prop: "Sous-titres",    encode: "✓ Traités",         copy: "✓ Traités"        },
      ] as row}
        <div class="compare-row">
          <div class="compare-cell compare-cell--prop">{row.prop}</div>
          <div class="compare-cell {videoMode === 'encode' ? 'compare-cell--hl' : ''}">{row.encode}</div>
          <div class="compare-cell {videoMode === 'copy' ? 'compare-cell--hl' : ''}">{row.copy}</div>
        </div>
      {/each}
    </div>
  </div>
</section>

<style>
  .content-section {
    padding: 28px 32px;
    max-width: 680px;
  }

  /* ── En-tête ────────────────────────────────────────────────────────────── */
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

  /* ── Blocs ──────────────────────────────────────────────────────────────── */
  .field-block {
    margin-bottom: 24px;
  }
  .field-label {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color-subtext);
    margin-bottom: 10px;
  }

  /* ── Cartes de mode ─────────────────────────────────────────────────────── */
  .mode-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }
  .mode-card {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 16px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    text-align: left;
    transition:
      border-color 0.15s,
      background 0.15s;
  }
  .mode-card:hover {
    border-color: var(--color-subtext2);
  }
  .mode-card--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 7%, var(--color-surface));
  }

  .mode-card-top {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .mode-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-xs);
    border: 1px solid var(--color-border);
    background: var(--color-panel);
    color: var(--color-subtext);
    flex-shrink: 0;
    transition: color 0.15s, border-color 0.15s, background 0.15s;
  }
  .mode-card--active .mode-icon {
    color: var(--color-accent);
    border-color: color-mix(in srgb, var(--color-accent) 30%, var(--color-border));
    background: color-mix(in srgb, var(--color-accent) 10%, var(--color-panel));
  }
  .mode-label {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text);
    letter-spacing: -0.01em;
    transition: color 0.15s;
  }
  .mode-card--active .mode-label {
    color: var(--color-accent);
  }
  .mode-desc {
    font-size: 11px;
    color: var(--color-subtext);
    line-height: 1.55;
    margin: 0;
  }
  .mode-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: 2px;
  }
  .mode-tag {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    padding: 2px 6px;
    border-radius: 3px;
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    color: var(--color-subtext2);
    transition: color 0.15s, border-color 0.15s, background 0.15s;
  }
  .mode-card--active .mode-tag {
    border-color: color-mix(in srgb, var(--color-accent) 20%, var(--color-border));
    color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-panel));
  }

  /* ── Callouts ───────────────────────────────────────────────────────────── */
  .callout {
    display: flex;
    gap: 10px;
    padding: 12px 14px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    margin-bottom: 24px;
    animation: fade-in 0.15s ease;
  }
  .callout--info {
    background: color-mix(in srgb, var(--color-accent) 6%, var(--color-surface));
    border-color: color-mix(in srgb, var(--color-accent) 22%, var(--color-border));
  }
  .callout--muted {
    background: var(--color-surface);
  }
  .callout-icon {
    flex-shrink: 0;
    margin-top: 1px;
    color: var(--color-subtext2);
  }
  .callout--info .callout-icon {
    color: var(--color-accent);
  }
  .callout-body {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .callout-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--color-text);
  }
  .callout--info .callout-title {
    color: var(--color-accent);
  }
  .callout-text {
    font-size: 11px;
    color: var(--color-subtext);
    line-height: 1.55;
  }
  .callout-text strong {
    color: var(--color-text);
    font-weight: 600;
  }

  /* ── Tableau comparatif ─────────────────────────────────────────────────── */
  .compare-block {
    margin-top: 4px;
  }
  .compare-table {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }
  .compare-header,
  .compare-row {
    display: grid;
    grid-template-columns: 140px 1fr 1fr;
  }
  .compare-row {
    border-top: 1px solid var(--color-border);
  }
  .compare-row:first-child {
    border-top: none;
  }
  .compare-cell {
    padding: 8px 12px;
    font-size: 11px;
    color: var(--color-subtext);
    display: flex;
    align-items: center;
  }
  .compare-cell + .compare-cell {
    border-left: 1px solid var(--color-border);
  }
  .compare-cell--prop {
    font-family: "Geist Mono", monospace;
    font-size: 9.5px;
    color: var(--color-subtext2);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    background: var(--color-panel);
  }
  .compare-cell--header {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color-subtext2);
    padding: 7px 12px;
    background: var(--color-panel);
    border-bottom: 1px solid var(--color-border);
  }
  .compare-cell--header.compare-cell--hl {
    color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-panel));
  }
  .compare-cell--hl {
    color: var(--color-text);
    background: color-mix(in srgb, var(--color-accent) 4%, var(--color-surface));
  }

  @keyframes fade-in {
    from { opacity: 0; transform: translateY(4px); }
    to   { opacity: 1; transform: translateY(0);   }
  }
</style>
