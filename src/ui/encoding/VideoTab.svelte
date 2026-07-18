<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";

  let videoMode  = $derived(encoder.videoMode);
  let preserveDv = $derived(encoder.preserveDv);

  let hasDvFiles = $derived(
    encoder.files.some((f) =>
      f.hdr_format?.includes("DV") || f.hdr_format?.includes("Dolby"),
    ),
  );
</script>

<section class="tab">
  <header class="tab-header">
    <h2 class="tab-title">Vidéo</h2>
    <p class="tab-desc">
      Choisissez entre ré-encoder le flux vidéo ou le copier tel quel.
    </p>
  </header>

  <div class="mode-grid">
    <!-- Encoder -->
    <button
      type="button"
      class="mode-card {videoMode === 'encode' ? 'mode-card--active' : ''}"
      onclick={() => encoder.setVideoMode("encode")}
    >
      <div class="mode-icon" aria-hidden="true">
        <svg width="18" height="18" viewBox="0 0 18 18" fill="none">
          <rect x="1" y="3" width="16" height="12" rx="2" stroke="currentColor" stroke-width="1.5"/>
          <path d="M7 6.5L12 9L7 11.5V6.5Z" fill="currentColor"/>
        </svg>
      </div>
      <div class="mode-body">
        <span class="mode-name">Encoder</span>
        <p class="mode-desc">
          Ré-encode en H.265 via NVENC. Réduit la taille avec un contrôle fin sur la qualité.
        </p>
        <div class="mode-tags">
          <span class="tag">hevc_nvenc</span>
          <span class="tag">CRF actif</span>
          <span class="tag">GPU</span>
        </div>
      </div>
    </button>

    <!-- Copier -->
    <button
      type="button"
      class="mode-card {videoMode === 'copy' ? 'mode-card--active' : ''}"
      onclick={() => encoder.setVideoMode("copy")}
    >
      <div class="mode-icon" aria-hidden="true">
        <svg width="18" height="18" viewBox="0 0 18 18" fill="none">
          <rect x="1" y="3" width="16" height="12" rx="2" stroke="currentColor" stroke-width="1.5"/>
          <path d="M5 9H13M10 6L13 9L10 12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </div>
      <div class="mode-body">
        <span class="mode-name">Copier</span>
        <p class="mode-desc">
          Copie le flux vidéo sans ré-encoder. Remuxage instantané, qualité préservée à l'identique.
        </p>
        <div class="mode-tags">
          <span class="tag">-c:v copy</span>
          <span class="tag">Sans perte</span>
          <span class="tag">Instantané</span>
        </div>
      </div>
    </button>
  </div>

  <!-- Callout mode -->
  {#if videoMode === "copy"}
    <div class="callout callout--info">
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none" class="callout-icon" aria-hidden="true">
        <circle cx="7" cy="7" r="6" stroke="currentColor" stroke-width="1.4"/>
        <path d="M7 6V10M7 4.5V4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
      </svg>
      <div>
        <strong>Mode copie actif</strong> — CRF, preset, AQ et multipass sont ignorés.
        L'audio et les sous-titres restent traités normalement.
      </div>
    </div>
  {:else}
    <div class="callout">
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none" class="callout-icon" aria-hidden="true">
        <circle cx="7" cy="7" r="6" stroke="currentColor" stroke-width="1.4"/>
        <path d="M5 7.5L6.5 9L9 5.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
      <div>
        <strong>Mode encodage actif</strong> — Configurez CRF, preset et AQ dans l'onglet
        <strong>Préréglages</strong>.
      </div>
    </div>
  {/if}

  <!-- ── Dolby Vision ──────────────────────────────────────────────────────── -->
  <div class="dv-section">
    <div class="dv-header">
      <div class="dv-label">
        <svg width="13" height="13" viewBox="0 0 13 13" fill="none" aria-hidden="true">
          <rect x="0.5" y="0.5" width="12" height="12" rx="2" stroke="currentColor" stroke-width="1"/>
          <path d="M3 4h3.5a2.5 2.5 0 0 1 0 5H3V4Z" fill="currentColor" opacity=".9"/>
          <path d="M8.5 6.5 10 4.5V8.5L8.5 6.5Z" fill="currentColor" opacity=".6"/>
        </svg>
        <span class="dv-label-text">Préserver Dolby Vision</span>
        {#if hasDvFiles}
          <span class="dv-pill dv-pill--active">DV détecté</span>
        {:else}
          <span class="dv-pill">Aucun fichier DV</span>
        {/if}
      </div>

      <button
        type="button"
        class="dv-toggle"
        class:dv-toggle--on={preserveDv}
        onclick={() => encoder.setPreserveDv(!preserveDv)}
        title={preserveDv ? "Désactiver la préservation DV" : "Activer la préservation DV"}
        aria-pressed={preserveDv}
      >
        <span class="dv-toggle-track">
          <span class="dv-toggle-thumb"></span>
        </span>
        <span class="dv-toggle-label">{preserveDv ? "Activé" : "Désactivé"}</span>
      </button>
    </div>

    {#if !hasDvFiles && !preserveDv}
      <p class="dv-hint">
        Chargez des fichiers contenant une couche Dolby Vision pour activer cette option.
        FFmpeg ajoutera <code>-dolbyvision_profile 8 -strict experimental</code> lors du ré-encodage.
      </p>
    {:else if preserveDv && videoMode === "encode"}
      <div class="callout callout--warn">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none" class="callout-icon" aria-hidden="true">
          <path d="M7 1.5L12.5 11H1.5L7 1.5Z" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round"/>
          <path d="M7 5.5V8M7 9.5V10" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
        </svg>
        <div>
          <strong>Compatibilité NVENC requise</strong> — Ajoute
          <code>-dolbyvision_profile 8 -strict experimental</code> à la commande FFmpeg.
          Certains pilotes ou versions FFmpeg peuvent ne pas supporter cette option.
          En cas d'erreur, désactivez ou passez en mode <strong>Copie</strong>.
        </div>
      </div>
    {:else if preserveDv && videoMode === "copy"}
      <div class="callout callout--info">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none" class="callout-icon" aria-hidden="true">
          <circle cx="7" cy="7" r="6" stroke="currentColor" stroke-width="1.4"/>
          <path d="M5 7.5L6.5 9L9 5.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
        <div>
          <strong>Mode copie + DV</strong> — Le flux est copié tel quel,
          les métadonnées Dolby Vision sont préservées nativement.
        </div>
      </div>
    {:else}
      <p class="dv-hint">
        Activez pour injecter les paramètres FFmpeg nécessaires à la préservation
        de la couche Dolby Vision lors du ré-encodage NVENC.
      </p>
    {/if}
  </div>
</section>

<style>
  .tab {
    padding: 24px 28px;
    max-width: 580px;
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

  /* ── Cartes de mode ─────────────────────────────────────────────────────── */
  .mode-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
    margin-bottom: 16px;
  }

  .mode-card {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
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
    background: color-mix(in srgb, var(--color-accent) 7%, var(--color-surface));
  }

  .mode-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: var(--radius-xs);
    border: 1px solid var(--color-border);
    background: var(--color-panel);
    color: var(--color-subtext);
    flex-shrink: 0;
    transition: color 0.12s, border-color 0.12s, background 0.12s;
  }
  .mode-card--active .mode-icon {
    color: var(--color-accent);
    border-color: color-mix(in srgb, var(--color-accent) 30%, var(--color-border));
    background: color-mix(in srgb, var(--color-accent) 10%, var(--color-panel));
  }

  .mode-body { display: flex; flex-direction: column; gap: 6px; }
  .mode-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text);
    letter-spacing: -0.01em;
    transition: color 0.12s;
  }
  .mode-card--active .mode-name { color: var(--color-accent); }
  .mode-desc {
    font-size: 11px;
    color: var(--color-subtext);
    line-height: 1.55;
    margin: 0;
  }
  .mode-tags { display: flex; flex-wrap: wrap; gap: 4px; }
  .tag {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    padding: 2px 6px;
    border-radius: 3px;
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    color: var(--color-subtext2);
    transition: color 0.12s, border-color 0.12s, background 0.12s;
  }
  .mode-card--active .tag {
    border-color: color-mix(in srgb, var(--color-accent) 22%, var(--color-border));
    color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-panel));
  }

  /* ── Callout ────────────────────────────────────────────────────────────── */
  .callout {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 12px 14px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    font-size: 11px;
    color: var(--color-subtext);
    line-height: 1.55;
    animation: fade-in 0.14s ease;
  }
  .callout strong { color: var(--color-text); font-weight: 600; }
  .callout code {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    border-radius: 3px;
    padding: 0 4px;
  }
  .callout--info {
    background: color-mix(in srgb, var(--color-accent) 6%, var(--color-surface));
    border-color: color-mix(in srgb, var(--color-accent) 22%, var(--color-border));
    color: color-mix(in srgb, var(--color-accent) 60%, var(--color-subtext));
  }
  .callout--info strong { color: var(--color-accent); }
  .callout--warn {
    background: color-mix(in srgb, #e5a020 6%, var(--color-surface));
    border-color: color-mix(in srgb, #e5a020 30%, var(--color-border));
    color: color-mix(in srgb, #e5a020 70%, var(--color-subtext));
  }
  .callout--warn strong { color: #e5a020; }
  .callout--warn code {
    background: color-mix(in srgb, #e5a020 8%, var(--color-panel));
    border-color: color-mix(in srgb, #e5a020 25%, var(--color-border));
  }
  .callout-icon {
    flex-shrink: 0;
    margin-top: 1px;
    color: var(--color-subtext2);
  }
  .callout--info .callout-icon { color: var(--color-accent); }
  .callout--warn .callout-icon { color: #e5a020; }

  /* ── Dolby Vision ───────────────────────────────────────────────────────── */
  .dv-section {
    margin-top: 16px;
    padding-top: 16px;
    border-top: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .dv-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .dv-label {
    display: flex;
    align-items: center;
    gap: 7px;
    color: var(--color-subtext);
  }

  .dv-label-text {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text);
  }

  .dv-pill {
    font-size: 9px;
    font-weight: 600;
    font-family: "Geist Mono", monospace;
    padding: 2px 6px;
    border-radius: 3px;
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    color: var(--color-subtext2);
    letter-spacing: 0.03em;
  }
  .dv-pill--active {
    background: color-mix(in srgb, #e5a020 10%, var(--color-panel));
    border-color: color-mix(in srgb, #e5a020 35%, var(--color-border));
    color: #e5a020;
  }

  /* Toggle switch */
  .dv-toggle {
    display: flex;
    align-items: center;
    gap: 7px;
    background: none;
    border: none;
    padding: 0;
    font-size: 11px;
    font-weight: 500;
    color: var(--color-subtext2);
    transition: color 0.12s;
    cursor: pointer;
  }
  .dv-toggle--on { color: var(--color-accent); }

  .dv-toggle-track {
    position: relative;
    width: 30px;
    height: 17px;
    border-radius: 9px;
    background: var(--color-border);
    border: 1px solid var(--color-border);
    transition: background 0.15s, border-color 0.15s;
    flex-shrink: 0;
  }
  .dv-toggle--on .dv-toggle-track {
    background: color-mix(in srgb, var(--color-accent) 25%, var(--color-surface));
    border-color: var(--color-accent);
  }

  .dv-toggle-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 11px;
    height: 11px;
    border-radius: 50%;
    background: var(--color-subtext2);
    transition: transform 0.15s, background 0.15s;
  }
  .dv-toggle--on .dv-toggle-thumb {
    transform: translateX(13px);
    background: var(--color-accent);
  }

  .dv-hint {
    font-size: 11px;
    color: var(--color-subtext2);
    line-height: 1.55;
    margin: 0;
  }
  .dv-hint code {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    border-radius: 3px;
    padding: 0 4px;
  }

  @keyframes fade-in {
    from { opacity: 0; transform: translateY(3px); }
    to   { opacity: 1; transform: translateY(0); }
  }
</style>