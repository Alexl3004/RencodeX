<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { BUILTIN_PRESETS } from "$lib/stores/prefs.store.svelte";
  import type { MultipassMode } from "$lib/stores/types";

  type PresetPanelId = string | "custom";
  // "balanced" par défaut au lieu de "custom"
  let selectedPanel = $state<PresetPanelId>(encoder.activePresetId ?? "balanced");

  const PRESET_META: Record<string, { desc: string; tags: string[] }> = {
    fast: {
      desc: "Encodage rapide, taille réduite. Idéal pour un visionnage immédiat.",
      tags: ["CRF 30", "p3", "Audio copy"],
    },
    balanced: {
      desc: "Bon équilibre entre vitesse et qualité pour un usage quotidien.",
      tags: ["CRF 28", "p5", "AAC 192k"],
    },
    quality: {
      desc: "Haute fidélité visuelle, temps d'encodage plus long.",
      tags: ["CRF 24", "p7", "AQ activée"],
    },
    archive: {
      desc: "Compression maximale pour le stockage longue durée.",
      tags: ["CRF 20", "p7", "Multipass"],
    },
  };

  const crfBands = [
    { range: [18, 20], label: "Archivage",    color: "#6ee7b7" },
    { range: [21, 23], label: "Home cinéma",  color: "#86efac" },
    { range: [24, 26], label: "Général",      color: "#fbbf24" },
    { range: [27, 29], label: "Web / Mobile", color: "#fb923c" },
    { range: [30, 35], label: "Compact",      color: "#f87171" },
  ];

  let currentBand = $derived(
    crfBands.find((b) => encoder.crf >= b.range[0] && encoder.crf <= b.range[1]) ?? crfBands[2],
  );

  const speedMeta: Record<string, { label: string; speed: number }> = {
    p1: { label: "Ultra rapide", speed: 7 },
    p2: { label: "Très rapide",  speed: 6 },
    p3: { label: "Rapide",       speed: 5 },
    p4: { label: "Normal+",      speed: 4 },
    p5: { label: "Normal",       speed: 3 },
    p6: { label: "Lent",         speed: 2 },
    p7: { label: "Très lent",    speed: 1 },
  };

  let spatialAq  = $derived(encoder.spatialAq);
  let temporalAq = $derived(encoder.temporalAq);
  let aqStrength = $derived(encoder.aqStrength);
  let multipass  = $derived(encoder.multipass);

  function applyBuiltinPreset(id: string) {
    encoder.applyPreset(id);
    selectedPanel = id;
  }
  function selectCustom() {
    selectedPanel = "custom";
  }
</script>

<section class="tab">
  <header class="tab-header">
    <h2 class="tab-title">Préréglages</h2>
    <p class="tab-desc">Choisissez un profil ou ajustez manuellement.</p>
  </header>

  <!-- Grille des préréglages -->
  <div class="preset-grid">
    {#each BUILTIN_PRESETS as p}
      {@const meta = PRESET_META[p.id]}
      {@const active = selectedPanel === p.id}
      <button
        type="button"
        class="preset-card {active ? 'preset-card--active' : ''}"
        onclick={() => applyBuiltinPreset(p.id)}
      >
        <div class="preset-top">
          <span class="preset-name">{p.label}</span>
          {#if p.id === "balanced"}
            <span class="badge-rec">REC</span>
          {/if}
        </div>
        <p class="preset-desc">{meta?.desc ?? ""}</p>
        <div class="preset-tags">
          {#each meta?.tags ?? [] as tag}
            <span class="tag">{tag}</span>
          {/each}
        </div>
      </button>
    {/each}
  </div>

  <!-- Lien vers les paramètres avancés -->
  <button
    type="button"
    class="custom-toggle {selectedPanel === 'custom' ? 'custom-toggle--active' : ''}"
    onclick={selectCustom}
  >
    <span class="custom-toggle-label">Paramètres avancés</span>
    <span class="custom-toggle-hint">CRF {encoder.crf} · {encoder.preset.toUpperCase()}</span>
    <svg class="custom-toggle-chevron {selectedPanel === 'custom' ? 'custom-toggle-chevron--open' : ''}"
      width="12" height="12" viewBox="0 0 12 12" fill="none">
      <path d="M3 4.5L6 7.5L9 4.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
    </svg>
  </button>

  <!-- Panneau avancé (affiché seulement si "custom") -->
  {#if selectedPanel === "custom"}
    <div class="advanced-panel">

      <!-- CRF -->
      <div class="field">
        <div class="field-header">
          <span class="field-label">Qualité CRF</span>
          <span class="field-badge" style="--c: {currentBand.color}">
            {encoder.crf} · {currentBand.label}
          </span>
        </div>
        <input
          type="range"
          value={encoder.crf}
          oninput={(e: Event) => encoder.setCrf(parseInt((e.target as HTMLInputElement).value))}
          min="18" max="35" step="1"
          class="slider"
          aria-label="Valeur CRF"
        />
        <div class="slider-hints">
          <span>18 · Meilleure qualité</span>
          <span>35 · Fichier plus petit</span>
        </div>
      </div>

      <!-- Vitesse -->
      <div class="field">
        <span class="field-label">Vitesse d'encodage</span>
        <div class="speed-row">
          {#each ["p1","p2","p3","p4","p5","p6","p7"] as p}
            {@const m = speedMeta[p]}
            {@const active = encoder.preset === p}
            <button
              type="button"
              class="speed-btn {active ? 'speed-btn--active' : ''}"
              onclick={() => encoder.setPreset(p)}
              title={m.label}
            >
              <span class="speed-id">{p.toUpperCase()}</span>
              <span class="speed-name">{m.label}</span>
              <div class="speed-dots">
                {#each Array(7) as _, i}
                  <div class="speed-dot {i < m.speed ? 'speed-dot--on' : ''}"></div>
                {/each}
              </div>
            </button>
          {/each}
        </div>
      </div>

      <!-- AQ -->
      <div class="field">
        <span class="field-label">Adaptive Quantization</span>
        <div class="row-2">
          <button
            type="button"
            class="toggle-card {spatialAq ? 'toggle-card--active' : ''}"
            onclick={() => encoder.setSpatialAq(!spatialAq)}
          >
            <span class="toggle-card-name">AQ spatiale</span>
            <span class="toggle-card-hint">zones complexes</span>
          </button>
          <button
            type="button"
            class="toggle-card {temporalAq ? 'toggle-card--active' : ''}"
            onclick={() => encoder.setTemporalAq(!temporalAq)}
          >
            <span class="toggle-card-name">AQ temporelle</span>
            <span class="toggle-card-hint">cohérence entre frames</span>
          </button>
        </div>
        {#if spatialAq || temporalAq}
          <div class="field-header" style="margin-top:12px">
            <span class="field-label">Force AQ</span>
            <span class="field-value">{aqStrength}</span>
          </div>
          <input
            type="range"
            value={aqStrength}
            oninput={(e: Event) => encoder.setAqStrength(parseInt((e.target as HTMLInputElement).value))}
            min="1" max="15" step="1"
            class="slider"
            aria-label="Force AQ"
          />
          <div class="slider-hints">
            <span>Doux</span>
            <span>Agressif</span>
          </div>
        {/if}
      </div>

      <!-- Multipass -->
      <div class="field">
        <span class="field-label">Multipass</span>
        <div class="row-3">
          {#each [
            { val: "disabled", label: "Désactivé",    hint: "passe unique"    },
            { val: "qres",     label: "¼ résolution", hint: "analyse rapide"  },
            { val: "fullres",  label: "Pleine rés.",  hint: "qualité maximum" },
          ] as opt}
            <button
              type="button"
              class="toggle-card {multipass === opt.val ? 'toggle-card--active' : ''}"
              onclick={() => encoder.setMultipass(opt.val as MultipassMode)}
            >
              <span class="toggle-card-name">{opt.label}</span>
              <span class="toggle-card-hint">{opt.hint}</span>
            </button>
          {/each}
        </div>
      </div>

    </div>
  {/if}
</section>

<style>
  /* ── Structure ──────────────────────────────────────────────────────────── */
  .tab {
    padding: 24px 28px;
    max-width: 660px;
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

  /* ── Grille préréglages ─────────────────────────────────────────────────── */
  .preset-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
    margin-bottom: 12px;
  }

  .preset-card {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 14px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    text-align: left;
    transition: border-color 0.12s, background 0.12s;
  }
  .preset-card:hover {
    border-color: var(--color-subtext2);
  }
  .preset-card--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 7%, var(--color-surface));
  }

  .preset-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .preset-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text);
    letter-spacing: -0.01em;
  }
  .preset-card--active .preset-name {
    color: var(--color-accent);
  }

  .badge-rec {
    font-family: "Geist Mono", monospace;
    font-size: 7px;
    letter-spacing: 0.05em;
    padding: 2px 5px;
    border-radius: 3px;
    background: color-mix(in srgb, var(--color-success, #4dbb6a) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-success, #4dbb6a) 28%, transparent);
    color: var(--color-success, #4dbb6a);
  }

  .preset-desc {
    font-size: 11px;
    color: var(--color-subtext);
    line-height: 1.5;
    margin: 0;
  }

  .preset-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: 2px;
  }
  .tag {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    padding: 2px 6px;
    border-radius: 3px;
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    color: var(--color-subtext2);
  }
  .preset-card--active .tag {
    border-color: color-mix(in srgb, var(--color-accent) 22%, var(--color-border));
    color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-panel));
  }

  /* ── Lien paramètres avancés ────────────────────────────────────────────── */
  .custom-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 10px 14px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: transparent;
    cursor: pointer;
    text-align: left;
    transition: border-color 0.12s, background 0.12s;
    margin-bottom: 0;
  }
  .custom-toggle:hover {
    border-color: var(--color-subtext2);
    background: color-mix(in srgb, var(--color-muted) 20%, transparent);
  }
  .custom-toggle--active {
    border-color: color-mix(in srgb, var(--color-accent) 30%, var(--color-border));
    background: color-mix(in srgb, var(--color-accent) 5%, transparent);
  }
  .custom-toggle-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--color-subtext);
    flex: 1;
  }
  .custom-toggle--active .custom-toggle-label {
    color: var(--color-text);
  }
  .custom-toggle-hint {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext2);
  }
  .custom-toggle-chevron {
    color: var(--color-subtext2);
    transition: transform 0.18s;
    flex-shrink: 0;
  }
  .custom-toggle-chevron--open {
    transform: rotate(180deg);
  }

  /* ── Panneau avancé ─────────────────────────────────────────────────────── */
  .advanced-panel {
    margin-top: 16px;
    display: flex;
    flex-direction: column;
    gap: 20px;
    animation: slide-in 0.15s ease;
  }
  @keyframes slide-in {
    from { opacity: 0; transform: translateY(-4px); }
    to   { opacity: 1; transform: translateY(0); }
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .field-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .field-label {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color-subtext);
  }
  .field-badge {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--c, var(--color-subtext2));
    padding: 2px 7px;
    border-radius: 3px;
    border: 1px solid color-mix(in srgb, var(--c, var(--color-border)) 35%, var(--color-border));
    background: color-mix(in srgb, var(--c, transparent) 8%, var(--color-surface));
  }
  .field-value {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    font-weight: 600;
    color: var(--color-accent);
  }

  /* Slider */
  .slider {
    width: 100%;
    accent-color: var(--color-accent);
    cursor: pointer;
  }
  .slider-hints {
    display: flex;
    justify-content: space-between;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext2);
  }

  /* Vitesse */
  .speed-row {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 4px;
  }
  .speed-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 8px 2px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    transition: border-color 0.12s, background 0.12s;
  }
  .speed-btn:hover { border-color: var(--color-subtext2); }
  .speed-btn--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 10%, var(--color-surface));
  }
  .speed-id {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    font-weight: 700;
    color: var(--color-subtext);
  }
  .speed-btn--active .speed-id { color: var(--color-accent); }
  .speed-name {
    font-size: 7.5px;
    color: var(--color-subtext2);
    text-align: center;
    line-height: 1.2;
  }
  .speed-dots {
    display: flex;
    gap: 2px;
  }
  .speed-dot {
    width: 3px;
    height: 3px;
    border-radius: 50%;
    background: var(--color-border);
  }
  .speed-dot--on { background: var(--color-accent); opacity: 0.7; }
  .speed-btn--active .speed-dot--on { opacity: 1; }

  /* Toggle cards (AQ / Multipass) */
  .row-2 { display: grid; grid-template-columns: 1fr 1fr; gap: 6px; }
  .row-3 { display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px; }

  .toggle-card {
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding: 11px 12px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    text-align: left;
    transition: border-color 0.12s, background 0.12s;
  }
  .toggle-card:hover { border-color: var(--color-subtext2); }
  .toggle-card--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-surface));
  }
  .toggle-card-name {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-subtext);
    transition: color 0.12s;
  }
  .toggle-card--active .toggle-card-name { color: var(--color-accent); }
  .toggle-card-hint {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext2);
  }
</style>
