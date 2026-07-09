<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { BUILTIN_PRESETS } from "$lib/stores/prefs.store.svelte";
  import type {
    MultipassMode,
    SubExtractFormat,
    SubExtractNaming,
    SubExtractPathMode,
  } from "$lib/stores/types";
  import { FolderOpen, Zap, Volume2, Box, Subtitles, SlidersHorizontal, Languages } from "@lucide/svelte";
  import { prefs } from "$lib/stores/prefs.store.svelte";
  import { filesStore } from "$lib/stores/files.store.svelte";
  import { LANG_NAMES } from "$lib/stores/naming";

  // ── Ordre des langues ─────────────────────────────────────────────────────
  let langOrder = $state<string[]>([...prefs.langOrder]);
  $effect(() => { langOrder = [...prefs.langOrder]; });

  function langLabel(code: string) { return LANG_NAMES[code] ?? code.toUpperCase(); }

  // ── Langues sélectionnées par défaut ──────────────────────────────────────
  let defaultAudioLangs = $state<string[]>([...prefs.defaultAudioLangs]);
  let defaultSubLangs   = $state<string[]>([...prefs.defaultSubLangs]);

  $effect(() => { defaultAudioLangs = [...prefs.defaultAudioLangs]; });
  $effect(() => { defaultSubLangs   = [...prefs.defaultSubLangs]; });

  function toggleDefaultAudio(code: string) {
    const next = defaultAudioLangs.includes(code)
      ? defaultAudioLangs.filter(c => c !== code)
      : [...defaultAudioLangs, code];
    defaultAudioLangs = next;
    prefs.setDefaultAudioLangs(next);
  }
  function toggleDefaultSub(code: string) {
    const next = defaultSubLangs.includes(code)
      ? defaultSubLangs.filter(c => c !== code)
      : [...defaultSubLangs, code];
    defaultSubLangs = next;
    prefs.setDefaultSubLangs(next);
  }

  let { onClose }: { onClose?: () => void } = $props();

  let audioMode    = $derived(encoder.audioMode);
  let audioBitrate = $derived(encoder.audioBitrate);
  let spatialAq    = $derived(encoder.spatialAq);
  let temporalAq   = $derived(encoder.temporalAq);
  let aqStrength   = $derived(encoder.aqStrength);
  let multipass    = $derived(encoder.multipass);
  let container    = $derived(encoder.container);

  // ── État de l'onglet Préréglages ───────────────────────────────────────────
  // "custom" = mode personnalisé (sous-panneau avec tous les contrôles fins)
  type PresetPanelId = string | "custom";
  let selectedPanel = $state<PresetPanelId>(encoder.activePresetId ?? "custom");

  // Descriptions des presets
  const PRESET_DESCS: Record<string, { desc: string; tags: string[] }> = {
    fast:     { desc: "Encodage rapide, taille réduite. Idéal pour un visionnage immédiat.", tags: ["CRF 30", "p3", "Audio copy"] },
    balanced: { desc: "Bon équilibre entre vitesse et qualité pour un usage quotidien.",      tags: ["CRF 28", "p5", "AAC 192k"] },
    quality:  { desc: "Haute fidélité visuelle, temps d'encodage plus long.",                 tags: ["CRF 24", "p7", "AQ spatial+temporel"] },
    archive:  { desc: "Compression maximale pour le stockage longue durée.",                  tags: ["CRF 20", "p7", "Multipass fullres"] },
  };

  // Contrôles fins pour le mode personnalisé
  const crfBands = [
    { range: [18, 20], label: "Archivage",     quality: "Transparente", color: "#6ee7b7" },
    { range: [21, 23], label: "Home cinéma",   quality: "Excellente",   color: "#86efac" },
    { range: [24, 26], label: "Usage général", quality: "Très bonne",   color: "#fbbf24" },
    { range: [27, 29], label: "Web / Mobile",  quality: "Bonne",        color: "#fb923c" },
    { range: [30, 35], label: "Compact",       quality: "Correcte",     color: "#f87171" },
  ];
  let currentBand = $derived(
    crfBands.find(b => encoder.crf >= b.range[0] && encoder.crf <= b.range[1]) ?? crfBands[2]
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

  function applyBuiltinPreset(id: string) {
    encoder.applyPreset(id);
    selectedPanel = id;
  }

  function selectCustom() {
    selectedPanel = "custom";
  }

  // ── Navigation sections ────────────────────────────────────────────────────
  type SectionId = "presets" | "audio" | "container" | "subtitles" | "languages";

  const SECTIONS: { id: SectionId; label: string; icon: any; desc: string }[] = [
    { id: "presets",   label: "Préréglages",   icon: Zap,       desc: "Qualité & vitesse" },
    { id: "audio",     label: "Audio",          icon: Volume2,   desc: "Codec & débit" },
    { id: "container", label: "Conteneur",      icon: Box,       desc: "MKV / MP4" },
    { id: "subtitles", label: "Sous-titres",    icon: Subtitles, desc: "Extraction" },
    { id: "languages", label: "Pistes",        icon: Languages, desc: "Ordre de priorité" },
  ];

  let activeSection = $state<SectionId>("presets");
</script>

<div class="page-root" class:page-root--horizontal={encoder.innerNavLayout === "horizontal"}>

  <!-- ── Sidebar navigation ─────────────────────────────────────────────── -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <span class="sidebar-title">Encodage</span>
      <span class="sidebar-sub">H.265 · NVENC</span>
    </div>

    <nav class="sidebar-nav" aria-label="Sections">
      {#each SECTIONS as sec}
        <button
          type="button"
          class="nav-item {activeSection === sec.id ? 'nav-item--active' : ''}"
          onclick={() => (activeSection = sec.id)}
          aria-current={activeSection === sec.id ? "page" : undefined}
        >
          <div class="nav-item-icon">
            <sec.icon class="w-3.5 h-3.5" />
          </div>
          <div class="nav-item-text">
            <span class="nav-item-label">{sec.label}</span>
            <span class="nav-item-desc">{sec.desc}</span>
          </div>
          {#if activeSection === sec.id}
            <div class="nav-item-indicator" aria-hidden="true"></div>
          {/if}
        </button>
      {/each}
    </nav>

  </aside>

  <!-- ── Content panel ───────────────────────────────────────────────────── -->
  <div class="content-panel">
    <!-- ═══════════════ PRÉRÉGLAGES ═══════════════ -->
    {#if activeSection === "presets"}
      <section class="content-section">
        <header class="section-header">
          <div>
            <h2 class="section-title">Préréglages d'encodage</h2>
            <p class="section-desc">
              Choisissez un profil prédéfini ou configurez chaque paramètre manuellement.
            </p>
          </div>
        </header>

        <!-- Cartes de préréglages -->
        <div class="bp-grid">
          {#each BUILTIN_PRESETS as p}
            {@const meta = PRESET_DESCS[p.id]}
            {@const isActive = selectedPanel === p.id}
            <button
              type="button"
              class="bp-card {isActive ? 'bp-card--active' : ''}"
              onclick={() => applyBuiltinPreset(p.id)}
            >
              <div class="bp-card-top">
                <span class="bp-label">{p.label}</span>
                {#if p.id === "balanced"}
                  <span class="bp-rec">REC</span>
                {/if}
              </div>
              <p class="bp-desc">{meta?.desc ?? ""}</p>
              <div class="bp-tags">
                {#each meta?.tags ?? [] as tag}
                  <span class="bp-tag">{tag}</span>
                {/each}
              </div>
            </button>
          {/each}

          <!-- Carte Personnalisé -->
          <button
            type="button"
            class="bp-card bp-card--custom {selectedPanel === 'custom' ? 'bp-card--active' : ''}"
            onclick={selectCustom}
          >
            <div class="bp-card-top">
              <span class="bp-label">Personnalisé</span>
              <SlidersHorizontal class="w-3 h-3" style="color: var(--color-subtext2)" />
            </div>
            <p class="bp-desc">Configurez CRF, preset encodeur et AQ manuellement.</p>
            <div class="bp-tags">
              <span class="bp-tag">CRF {encoder.crf}</span>
              <span class="bp-tag">{encoder.preset.toUpperCase()}</span>
            </div>
          </button>
        </div>

        <!-- Panneau personnalisé -->
        {#if selectedPanel === "custom"}
          <div class="custom-panel">
            <div class="cp-divider">
              <span class="cp-divider-label">Paramètres personnalisés</span>
            </div>

            <!-- CRF -->
            <div class="field-block">
              <div class="field-label-row">
                <span class="field-label">Qualité CRF</span>
                <div class="section-badge" style="--badge-color: {currentBand.color}">
                  <span class="badge-value">{encoder.crf}</span>
                  <span class="badge-label">{currentBand.label}</span>
                </div>
              </div>
              <div class="crf-track-wrap">
                <input
                  type="range"
                  value={encoder.crf}
                  oninput={(e: Event) => encoder.setCrf(parseInt((e.target as HTMLInputElement).value))}
                  min="18" max="35" step="1"
                  class="crf-slider"
                  aria-label="Valeur CRF"
                />
                <div class="crf-scale">
                  <span>18</span>
                  <span class="crf-scale-label">← Qualité · Taille →</span>
                  <span>35</span>
                </div>
              </div>
              <div class="band-grid band-grid--compact">
                {#each crfBands as band}
                  {@const active = encoder.crf >= band.range[0] && encoder.crf <= band.range[1]}
                  <div class="band-card {active ? 'band-card--active' : ''}" style="--c: {band.color}">
                    <div class="band-range">{band.range[0]}–{band.range[1]}</div>
                    <div class="band-name">{band.label}</div>
                  </div>
                {/each}
              </div>
            </div>

            <!-- Preset encodeur (vitesse) -->
            <div class="field-block">
              <div class="field-label">Vitesse d'encodage</div>
              <div class="speed-grid">
                {#each ["p1","p2","p3","p4","p5","p6","p7"] as p}
                  {@const meta = speedMeta[p]}
                  {@const isActive = encoder.preset === p}
                  <button
                    type="button"
                    class="speed-btn {isActive ? 'speed-btn--active' : ''}"
                    onclick={() => encoder.setPreset(p)}
                  >
                    <span class="speed-id">{p.toUpperCase()}</span>
                    <span class="speed-name">{meta.label}</span>
                    <div class="speed-dots">
                      {#each Array(7) as _, i}
                        <div class="speed-dot {i < meta.speed ? 'speed-dot--on' : ''}"></div>
                      {/each}
                    </div>
                  </button>
                {/each}
              </div>
            </div>

            <!-- AQ -->
            <div class="field-block">
              <div class="field-label">Adaptive Quantization</div>
              <div class="toggle-row">
                <button
                  type="button"
                  class="toggle-opt {spatialAq ? 'toggle-opt--active' : ''}"
                  onclick={() => encoder.setSpatialAq(!spatialAq)}
                >
                  <span class="toggle-opt-title">AQ spatiale</span>
                  <span class="toggle-opt-sub">zones complexes dans l'image</span>
                </button>
                <button
                  type="button"
                  class="toggle-opt {temporalAq ? 'toggle-opt--active' : ''}"
                  onclick={() => encoder.setTemporalAq(!temporalAq)}
                >
                  <span class="toggle-opt-title">AQ temporelle</span>
                  <span class="toggle-opt-sub">cohérence entre les frames</span>
                </button>
              </div>
              {#if spatialAq || temporalAq}
                <div class="field-label-row" style="margin-top:10px">
                  <span class="field-label">Force AQ</span>
                  <span class="field-value-badge">{aqStrength}</span>
                </div>
                <input
                  type="range"
                  value={aqStrength}
                  oninput={(e: Event) => encoder.setAqStrength(parseInt((e.target as HTMLInputElement).value))}
                  min="1" max="15" step="1"
                  class="crf-slider"
                  aria-label="Force AQ"
                />
                <div class="slider-hints">
                  <span>Doux</span>
                  <span>Agressif</span>
                </div>
              {/if}
            </div>

            <!-- Multipass -->
            <div class="field-block">
              <div class="field-label">Multipass</div>
              <div class="multipass-row">
                {#each [
                  { val: "disabled", label: "Aucun",        sub: "passe unique" },
                  { val: "qres",     label: "¼ résolution", sub: "analyse rapide" },
                  { val: "fullres",  label: "Pleine rés.",   sub: "qualité max" },
                ] as opt}
                  <button
                    type="button"
                    class="multipass-btn {multipass === opt.val ? 'multipass-btn--active' : ''}"
                    onclick={() => encoder.setMultipass(opt.val as MultipassMode)}
                  >
                    <span class="mp-label">{opt.label}</span>
                    <span class="mp-sub">{opt.sub}</span>
                  </button>
                {/each}
              </div>
            </div>
          </div>
        {/if}
      </section>

    <!-- ═══════════════ AUDIO ═══════════════ -->
    {:else if activeSection === "audio"}
      <section class="content-section">
        <header class="section-header">
          <div>
            <h2 class="section-title">Audio</h2>
            <p class="section-desc">
              Réencoder en AAC garantit la compatibilité maximale.
              La copie conserve la piste originale sans dégradation supplémentaire.
            </p>
          </div>
        </header>

        <div class="field-block">
          <div class="field-label">Mode</div>
          <div class="toggle-row">
            <button
              type="button"
              class="toggle-opt {audioMode === 'reencode' ? 'toggle-opt--active' : ''}"
              onclick={() => encoder.setAudioMode("reencode")}
            >
              <span class="toggle-opt-title">Réencoder</span>
              <span class="toggle-opt-sub">AAC · compatibilité maximale</span>
            </button>
            <button
              type="button"
              class="toggle-opt {audioMode === 'copy' ? 'toggle-opt--active' : ''}"
              onclick={() => encoder.setAudioMode("copy")}
            >
              <span class="toggle-opt-title">Copier</span>
              <span class="toggle-opt-sub">-c:a copy · sans perte</span>
            </button>
          </div>
        </div>

        {#if audioMode === "reencode"}
          <div class="field-block">
            <div class="field-label">Débit AAC</div>
            <div class="bitrate-row">
              {#each [128, 192, 256, 320] as br}
                <button
                  type="button"
                  class="bitrate-btn {audioBitrate === br ? 'bitrate-btn--active' : ''}"
                  onclick={() => encoder.setAudioBitrate(br)}
                >
                  <span class="bitrate-val">{br}</span>
                  <span class="bitrate-unit">kbps</span>
                </button>
              {/each}
            </div>
          </div>
        {:else}
          <div class="info-callout">
            La piste audio source est conservée telle quelle — aucune dégradation supplémentaire.
          </div>
        {/if}
      </section>

    <!-- ═══════════════ CONTENEUR ═══════════════ -->
    {:else if activeSection === "container"}
      <section class="content-section">
        <header class="section-header">
          <div>
            <h2 class="section-title">Conteneur de sortie</h2>
            <p class="section-desc">
              Le conteneur détermine quels types de pistes et métadonnées peuvent être inclus.
            </p>
          </div>
        </header>

        <div class="field-block">
          <div class="container-cards">
            <button
              type="button"
              class="container-card {container === 'mkv' ? 'container-card--active' : ''}"
              onclick={() => encoder.setContainer("mkv")}
            >
              <div class="cc-format">MKV</div>
              <div class="cc-name">Matroska</div>
              <ul class="cc-features">
                <li>Sous-titres ASS natifs</li>
                <li>Chapitres et métadonnées</li>
                <li>Flux multiples</li>
              </ul>
              <div class="cc-badge cc-badge--rec">Recommandé</div>
            </button>
            <button
              type="button"
              class="container-card {container === 'mp4' ? 'container-card--active' : ''}"
              onclick={() => encoder.setContainer("mp4")}
            >
              <div class="cc-format">MP4</div>
              <div class="cc-name">MPEG-4</div>
              <ul class="cc-features">
                <li>Compatible Apple / mobile</li>
                <li>Streaming HTTP natif</li>
                <li>Sous-titres mov_text</li>
              </ul>
              <div class="cc-badge">Compatible</div>
            </button>
          </div>

          {#if container === "mp4"}
            <div class="info-callout info-callout--warn">
              Les sous-titres ASS avancés seront convertis en mov_text — certains styles peuvent être perdus.
            </div>
          {/if}
        </div>
      </section>

    <!-- ═══════════════ SOUS-TITRES ═══════════════ -->
    {:else if activeSection === "subtitles"}
      <section class="content-section">
        <header class="section-header">
          <div>
            <h2 class="section-title">Extraction sous-titres</h2>
            <p class="section-desc">
              Configure l'extraction des pistes de sous-titres intégrées dans les fichiers source.
            </p>
          </div>
        </header>

        <div class="field-block">
          <div class="field-label">Bouton dans la barre de contrôle</div>
          <button
            type="button"
            class="toggle-full {encoder.showExtractButton ? 'toggle-full--on' : ''}"
            onclick={() => encoder.setShowExtractButton(!encoder.showExtractButton)}
          >
            <div class="tf-dot"></div>
            <span>{encoder.showExtractButton ? "Bouton « Extraire » visible" : "Bouton « Extraire » masqué"}</span>
          </button>
        </div>

        <div class="field-grid-2">
          <div class="field-block">
            <div class="field-label">Format</div>
            <div class="seg-pair">
              {#each [["srt","SRT"],["ass","ASS"]] as [val, lbl]}
                <button type="button"
                  class="seg-btn {encoder.subExtractFormat === val ? 'seg-btn--active' : ''}"
                  onclick={() => encoder.setSubExtractFormat(val as SubExtractFormat)}>
                  {lbl}
                </button>
              {/each}
            </div>
          </div>

          <div class="field-block">
            <div class="field-label">Nommage</div>
            <div class="seg-pair">
              {#each [["source","Source"],["cleaned","Nettoyé"]] as [val, lbl]}
                <button type="button"
                  class="seg-btn {encoder.subExtractNaming === val ? 'seg-btn--active' : ''}"
                  onclick={() => encoder.setSubExtractNaming(val as SubExtractNaming)}>
                  {lbl}
                </button>
              {/each}
            </div>
          </div>
        </div>

        <div class="field-block">
          <div class="field-label">Dossier de destination</div>
          <div class="dest-row">
            {#each [
              { val: "source",    label: "Dossier source" },
              { val: "downloads", label: "Téléchargements" },
              { val: "custom",    label: "Personnalisé…" },
            ] as opt}
              <button
                type="button"
                class="dest-btn {encoder.subExtractPathMode === opt.val ? 'dest-btn--active' : ''}"
                onclick={() => encoder.setSubExtractPathMode(opt.val as SubExtractPathMode)}
              >{opt.label}</button>
            {/each}
          </div>

          {#if encoder.subExtractPathMode === "custom"}
            <div class="custom-path-row">
              <input
                type="text"
                value={encoder.subExtractCustomPath}
                oninput={(e) => encoder.setSubExtractCustomPath((e.target as HTMLInputElement).value)}
                placeholder="Chemin complet du dossier…"
                class="path-input"
              />
              <button
                type="button"
                class="browse-btn"
                aria-label="Parcourir"
                onclick={async () => {
                  try {
                    const dialog = await import("@tauri-apps/plugin-dialog");
                    const selected = await dialog.open({ directory: true });
                    if (selected && typeof selected === "string")
                      encoder.setSubExtractCustomPath(selected);
                  } catch (e) {
                    console.error("Plugin dialog non disponible", e);
                  }
                }}
              >
                <FolderOpen class="w-3.5 h-3.5" />
              </button>
            </div>
          {/if}
        </div>
      </section>

    <!-- ═══════════════ LANGUES ═══════════════ -->
    {:else if activeSection === "languages"}
      <section class="content-section">
        <header class="section-header">
          <div>
            <h2 class="section-title">Langues par défaut</h2>
            <p class="section-desc">
              Pistes activées automatiquement à l'ouverture d'un fichier.
            </p>
          </div>
        </header>

        <div class="lang-grid">
          <!-- ── Colonne Audio ── -->
          <div class="lang-col">
            <div class="lang-col-header">
              <Volume2 class="w-3.5 h-3.5" />
              <span>Audio</span>
            </div>
            <ul class="lang-toggle-list" aria-label="Langues audio par défaut">
              {#each langOrder as code (code)}
                {@const on = defaultAudioLangs.includes(code)}
                <li>
                  <button
                    type="button"
                    class="lang-toggle {on ? 'lang-toggle--on' : ''}"
                    onclick={() => toggleDefaultAudio(code)}
                    aria-pressed={on}
                  >
                    <span class="lang-toggle-name">{langLabel(code)}</span>
                    <span class="lang-toggle-code">{code}</span>
                    <span class="lang-toggle-pill" aria-hidden="true">
                      <span class="lang-toggle-thumb"></span>
                    </span>
                  </button>
                </li>
              {/each}
            </ul>
          </div>

          <!-- ── Colonne Sous-titres ── -->
          <div class="lang-col">
            <div class="lang-col-header">
              <Subtitles class="w-3.5 h-3.5" />
              <span>Sous-titres</span>
            </div>
            <ul class="lang-toggle-list" aria-label="Langues sous-titres par défaut">
              {#each langOrder as code (code)}
                {@const on = defaultSubLangs.includes(code)}
                <li>
                  <button
                    type="button"
                    class="lang-toggle {on ? 'lang-toggle--on' : ''}"
                    onclick={() => toggleDefaultSub(code)}
                    aria-pressed={on}
                  >
                    <span class="lang-toggle-name">{langLabel(code)}</span>
                    <span class="lang-toggle-code">{code}</span>
                    <span class="lang-toggle-pill" aria-hidden="true">
                      <span class="lang-toggle-thumb"></span>
                    </span>
                  </button>
                </li>
              {/each}
            </ul>
          </div>
        </div>
      </section>
    {/if}

  </div><!-- /content-panel -->
</div><!-- /page-root -->

<style>
  /* ── Layout racine ──────────────────────────────────────────────────────── */
  .page-root {
    display: flex;
    height: 100%;
    overflow: hidden;
    background: var(--color-surface);
  }

  /* ── Sidebar ────────────────────────────────────────────────────────────── */
  .sidebar {
    width: 220px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    background: var(--color-panel);
    border-right: 1px solid var(--color-border);
    overflow: hidden;
  }

  .sidebar-header {
    padding: 20px 16px 14px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }
  .sidebar-title {
    display: block;
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text);
    letter-spacing: -0.01em;
  }
  .sidebar-sub {
    display: block;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    letter-spacing: 0.06em;
    color: var(--color-subtext2);
    margin-top: 3px;
    text-transform: uppercase;
  }

  .sidebar-nav {
    flex: 1;
    overflow-y: auto;
    padding: 8px 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .nav-item {
    position: relative;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid transparent;
    background: transparent;
    cursor: pointer;
    text-align: left;
    transition: background 0.1s, border-color 0.1s;
    width: 100%;
  }
  .nav-item:hover {
    background: color-mix(in srgb, var(--color-muted) 30%, transparent);
  }
  .nav-item--active {
    background: color-mix(in srgb, var(--color-accent) 9%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 22%, var(--color-border));
  }

  .nav-item-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-xs);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    color: var(--color-subtext);
    flex-shrink: 0;
    transition: background 0.1s, color 0.1s, border-color 0.1s;
  }
  .nav-item--active .nav-item-icon {
    background: color-mix(in srgb, var(--color-accent) 12%, var(--color-surface));
    border-color: color-mix(in srgb, var(--color-accent) 30%, var(--color-border));
    color: var(--color-accent);
  }

  .nav-item-text {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }
  .nav-item-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--color-subtext);
    line-height: 1.2;
    transition: color 0.1s;
  }
  .nav-item--active .nav-item-label {
    color: var(--color-accent);
    font-weight: 600;
  }
  .nav-item-desc {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext2);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .nav-item-indicator {
    position: absolute;
    right: 0;
    top: 50%;
    transform: translateY(-50%);
    width: 3px;
    height: 18px;
    border-radius: 2px 0 0 2px;
    background: var(--color-accent);
  }


  /* ── Content panel ──────────────────────────────────────────────────────── */
  .content-panel {
    flex: 1;
    overflow-y: auto;
    padding: 0;
  }

  .content-section {
    padding: 28px 32px;
    max-width: 680px;
  }

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

  /* CRF badge */
  .section-badge {
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 72px;
    height: 72px;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--badge-color) 8%, var(--color-surface));
    border: 1px solid color-mix(in srgb, var(--badge-color) 25%, var(--color-border));
    transition: background 0.3s, border-color 0.3s;
  }
  .badge-value {
    font-family: "Geist Mono", monospace;
    font-size: 26px;
    font-weight: 700;
    color: var(--badge-color, var(--color-text));
    line-height: 1;
    transition: color 0.3s;
  }
  .badge-label {
    font-family: "Geist Mono", monospace;
    font-size: 8px;
    color: var(--badge-color, var(--color-subtext));
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-top: 3px;
    opacity: 0.8;
    text-align: center;
    transition: color 0.3s;
  }

  /* ── Field blocks ───────────────────────────────────────────────────────── */
  .field-block {
    margin-bottom: 24px;
  }
  .field-block:last-child { margin-bottom: 0; }

  .field-label {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color-subtext);
    margin-bottom: 10px;
  }
  .field-label-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 10px;
  }
  .field-value-badge {
    font-family: "Geist Mono", monospace;
    font-size: 13px;
    font-weight: 700;
    color: var(--color-accent);
  }

  .field-grid-2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 20px;
    margin-bottom: 24px;
  }

  /* ── CRF slider ─────────────────────────────────────────────────────────── */
  .crf-track-wrap { margin-bottom: 12px; }
  .crf-slider {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 4px;
    border-radius: 2px;
    background: var(--color-border2, var(--color-border));
    outline: none;
    cursor: pointer;
    display: block;
  }
  .crf-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--color-accent);
    border: 2px solid var(--color-panel);
    box-shadow: 0 0 0 1px var(--color-accent);
    cursor: pointer;
    transition: transform 0.1s;
  }
  .crf-slider::-webkit-slider-thumb:hover { transform: scale(1.2); }

  .crf-scale {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext2);
    margin-top: 5px;
  }
  .crf-scale-label { color: var(--color-subtext2); }

  .slider-hints {
    display: flex;
    justify-content: space-between;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext2);
    margin-top: 5px;
  }

  /* ── CRF bands ──────────────────────────────────────────────────────────── */
  .band-grid {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 6px;
  }
  .band-grid--compact .band-card { padding: 7px 6px; }
  .band-card {
    padding: 10px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    text-align: center;
    transition: border-color 0.2s, background 0.2s;
  }
  .band-card--active {
    border-color: color-mix(in srgb, var(--c) 50%, var(--color-border));
    background: color-mix(in srgb, var(--c) 8%, var(--color-surface));
  }
  .band-range {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    font-weight: 700;
    color: var(--color-subtext);
    margin-bottom: 4px;
  }
  .band-card--active .band-range { color: var(--c); }
  .band-name {
    font-size: 10px;
    font-weight: 600;
    color: var(--color-subtext);
    margin-bottom: 2px;
  }
  .band-card--active .band-name { color: var(--color-text); }

  /* ── Builtin preset cards ────────────────────────────────────────────────── */
  .bp-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 10px;
    margin-bottom: 24px;
  }
  .bp-card {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 16px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    text-align: left;
    transition: border-color 0.15s, background 0.15s;
  }
  .bp-card:hover { border-color: var(--color-subtext2); }
  .bp-card--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 7%, var(--color-surface));
  }
  .bp-card--custom {
    grid-column: 1 / -1;
    flex-direction: row;
    align-items: flex-start;
    gap: 12px;
  }
  .bp-card--custom .bp-desc { flex: 1; }

  .bp-card-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .bp-label {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text);
    letter-spacing: -0.01em;
  }
  .bp-card--active .bp-label { color: var(--color-accent); }
  .bp-rec {
    font-family: "Geist Mono", monospace;
    font-size: 7px;
    letter-spacing: 0.05em;
    padding: 2px 5px;
    border-radius: 3px;
    background: color-mix(in srgb, var(--color-success, #4dbb6a) 15%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-success, #4dbb6a) 30%, transparent);
    color: var(--color-success, #4dbb6a);
    line-height: 1.4;
  }
  .bp-desc {
    font-size: 11px;
    color: var(--color-subtext);
    line-height: 1.5;
    margin: 0;
  }
  .bp-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .bp-tag {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    padding: 2px 6px;
    border-radius: 3px;
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    color: var(--color-subtext2);
  }
  .bp-card--active .bp-tag {
    border-color: color-mix(in srgb, var(--color-accent) 20%, var(--color-border));
    color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-panel));
  }

  /* ── Custom panel ────────────────────────────────────────────────────────── */
  .custom-panel {
    animation: fade-in 0.15s ease;
  }
  @keyframes fade-in {
    from { opacity: 0; transform: translateY(4px); }
    to   { opacity: 1; transform: translateY(0); }
  }
  .cp-divider {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 20px;
  }
  .cp-divider::before,
  .cp-divider::after {
    content: "";
    flex: 1;
    height: 1px;
    background: var(--color-border);
  }
  .cp-divider-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--color-subtext2);
    white-space: nowrap;
  }

  /* ── Speed grid (dans custom panel) ─────────────────────────────────────── */
  .speed-grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 5px;
  }
  .speed-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 9px 3px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
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
    transition: color 0.15s;
  }
  .speed-btn--active .speed-id { color: var(--color-accent); }
  .speed-name {
    font-size: 8px;
    color: var(--color-subtext2);
    text-align: center;
    line-height: 1.2;
  }
  .speed-dots {
    display: flex;
    gap: 2px;
    align-items: center;
  }
  .speed-dot {
    width: 3px;
    height: 3px;
    border-radius: 50%;
    background: var(--color-border);
    transition: background 0.15s;
  }
  .speed-dot--on { background: var(--color-accent); opacity: 0.7; }
  .speed-btn--active .speed-dot--on { opacity: 1; }

  /* ── Toggle options (paire) ─────────────────────────────────────────────── */
  .toggle-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }
  .toggle-opt {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 3px;
    padding: 12px 14px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    text-align: left;
    transition: border-color 0.15s, background 0.15s;
  }
  .toggle-opt:hover { border-color: var(--color-subtext2); }
  .toggle-opt--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-surface));
  }
  .toggle-opt-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-subtext);
    transition: color 0.15s;
  }
  .toggle-opt--active .toggle-opt-title { color: var(--color-accent); }
  .toggle-opt-sub {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext2);
  }

  /* ── Bitrate row ────────────────────────────────────────────────────────── */
  .bitrate-row {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
  }
  .bitrate-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 10px 6px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
  }
  .bitrate-btn:hover { border-color: var(--color-subtext2); }
  .bitrate-btn--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-surface));
  }
  .bitrate-val {
    font-family: "Geist Mono", monospace;
    font-size: 16px;
    font-weight: 700;
    color: var(--color-subtext);
    line-height: 1;
    transition: color 0.15s;
  }
  .bitrate-btn--active .bitrate-val { color: var(--color-accent); }
  .bitrate-unit {
    font-family: "Geist Mono", monospace;
    font-size: 8px;
    color: var(--color-subtext2);
  }

  /* ── Multipass ──────────────────────────────────────────────────────────── */
  .multipass-row {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
  }
  .multipass-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 3px;
    padding: 12px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
  }
  .multipass-btn:hover { border-color: var(--color-subtext2); }
  .multipass-btn--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-surface));
  }
  .mp-label {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-subtext);
    transition: color 0.15s;
  }
  .multipass-btn--active .mp-label { color: var(--color-accent); }
  .mp-sub {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext2);
  }

  /* ── Container cards ────────────────────────────────────────────────────── */
  .container-cards {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    margin-bottom: 14px;
  }
  .container-card {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
    padding: 16px 18px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    text-align: left;
    transition: border-color 0.15s, background 0.15s;
  }
  .container-card:hover { border-color: var(--color-subtext2); }
  .container-card--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 7%, var(--color-surface));
  }
  .cc-format {
    font-family: "Geist Mono", monospace;
    font-size: 22px;
    font-weight: 700;
    color: var(--color-subtext);
    line-height: 1;
    transition: color 0.15s;
  }
  .container-card--active .cc-format { color: var(--color-accent); }
  .cc-name {
    font-size: 11px;
    color: var(--color-subtext2);
    margin-bottom: 4px;
  }
  .cc-features {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .cc-features li {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext2);
    padding-left: 10px;
    position: relative;
  }
  .cc-features li::before {
    content: "·";
    position: absolute;
    left: 0;
    color: var(--color-subtext2);
  }
  .cc-badge {
    margin-top: 4px;
    font-family: "Geist Mono", monospace;
    font-size: 8px;
    letter-spacing: 0.05em;
    padding: 2px 7px;
    border-radius: var(--radius-full);
    border: 1px solid var(--color-border);
    color: var(--color-subtext2);
    background: var(--color-panel);
  }
  .cc-badge--rec {
    border-color: color-mix(in srgb, var(--color-success) 30%, var(--color-border));
    color: var(--color-success);
    background: color-mix(in srgb, var(--color-success) 8%, var(--color-panel));
  }

  /* ── Toggle full-width ──────────────────────────────────────────────────── */
  .toggle-full {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 11px 14px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    font-size: 12px;
    font-weight: 500;
    color: var(--color-subtext);
    transition: border-color 0.15s, background 0.15s, color 0.15s;
  }
  .toggle-full--on {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 7%, var(--color-surface));
    color: var(--color-accent);
  }
  .tf-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-border);
    flex-shrink: 0;
    transition: background 0.15s;
  }
  .toggle-full--on .tf-dot { background: var(--color-accent); }

  /* ── Seg pair ───────────────────────────────────────────────────────────── */
  .seg-pair {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px;
  }
  .seg-btn {
    padding: 9px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    font-weight: 600;
    color: var(--color-subtext);
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s, color 0.15s;
    text-align: center;
  }
  .seg-btn:hover { border-color: var(--color-subtext2); color: var(--color-text); }
  .seg-btn--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 10%, var(--color-surface));
    color: var(--color-accent);
  }

  /* ── Dest buttons ───────────────────────────────────────────────────────── */
  .dest-row {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 6px;
    margin-bottom: 10px;
  }
  .dest-btn {
    padding: 9px 6px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    font-size: 11px;
    color: var(--color-subtext);
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s, color 0.15s;
  }
  .dest-btn:hover { border-color: var(--color-subtext2); color: var(--color-text); }
  .dest-btn--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-surface));
    color: var(--color-accent);
  }

  /* ── Custom path ────────────────────────────────────────────────────────── */
  .custom-path-row {
    display: flex;
    gap: 6px;
  }
  .path-input {
    flex: 1;
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    color: var(--color-text);
    outline: none;
    transition: border-color 0.15s;
  }
  .path-input:focus { border-color: var(--color-accent); }
  .browse-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 34px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-subtext);
    cursor: pointer;
    flex-shrink: 0;
    transition: border-color 0.1s, color 0.1s;
  }
  .browse-btn:hover { border-color: var(--color-subtext2); color: var(--color-text); }

  /* ── Info callout ───────────────────────────────────────────────────────── */
  .info-callout {
    padding: 11px 14px;
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    font-size: 12px;
    color: var(--color-subtext);
    line-height: 1.5;
  }
  .info-callout--warn {
    background: color-mix(in srgb, var(--color-warning, #fbbf24) 5%, var(--color-surface));
    border-color: color-mix(in srgb, var(--color-warning, #fbbf24) 25%, var(--color-border));
    color: color-mix(in srgb, var(--color-warning, #fbbf24) 70%, var(--color-subtext));
  }

  /* ── Layout horizontal ──────────────────────────────────────────────────── */
  .page-root--horizontal {
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  .page-root--horizontal .sidebar {
    width: 100%;
    height: auto;
    flex-direction: row;
    align-items: center;
    border-right: none;
    border-bottom: 1px solid var(--color-border);
    padding: 0 12px;
    gap: 0;
    overflow-x: auto;
    overflow-y: visible;
    flex-shrink: 0;
  }
  .page-root--horizontal .sidebar-header {
    display: none;
  }
  .page-root--horizontal .sidebar-nav {
    flex-direction: row;
    padding: 0;
    gap: 2px;
    overflow: visible;
    flex: 1;
    justify-content: center;
  }
  .page-root--horizontal .nav-item {
    flex-direction: row;
    min-height: 36px;
    width: auto;
    padding: 6px 14px;
    gap: 6px;
    border-left: none;
    border-bottom: none;
    border-radius: var(--radius-sm);
    white-space: nowrap;
  }
  .page-root--horizontal .nav-item--active {
    border-left-color: transparent;
  }
  .page-root--horizontal .nav-item-indicator {
    display: none;
  }
  .page-root--horizontal .nav-item-text {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 6px;
  }
  .page-root--horizontal .nav-item-desc {
    display: none;
  }
  .page-root--horizontal .content-panel {
    flex: 1 1 0;
    min-height: 0;
    min-width: 0;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    align-items: center;
  }
  .page-root--horizontal .content-section {
    width: 100%;
    max-width: 720px;
  }

  /* ── Grille langues (2 colonnes) ────────────────────────────────────────── */
  .lang-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    align-items: start;
  }
  .lang-col {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .lang-col-header {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    font-weight: 600;
    color: var(--color-subtext2);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: 0 2px 4px;
    border-bottom: 1px solid var(--color-border);
    margin-bottom: 2px;
  }
  .lang-toggle-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  /* ── Toggle row (bouton complet) ─────────────────────────────────────────── */
  .lang-toggle {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    text-align: left;
    transition: border-color 0.12s, background 0.12s;
    user-select: none;
  }
  .lang-toggle:hover {
    border-color: var(--color-subtext2);
    background: color-mix(in srgb, var(--color-accent) 3%, var(--color-surface));
  }
  .lang-toggle--on {
    border-color: color-mix(in srgb, var(--color-accent) 45%, var(--color-border));
    background: color-mix(in srgb, var(--color-accent) 6%, var(--color-surface));
  }
  .lang-toggle--on:hover {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 10%, var(--color-surface));
  }
  .lang-toggle-name {
    flex: 1;
    font-size: 12px;
    font-weight: 500;
    color: var(--color-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .lang-toggle-code {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext2);
    flex-shrink: 0;
  }

  /* ── Pill (toggle switch visuel) ─────────────────────────────────────────── */
  .lang-toggle-pill {
    position: relative;
    width: 28px;
    height: 16px;
    border-radius: 999px;
    background: var(--color-border);
    flex-shrink: 0;
    transition: background 0.18s;
  }
  .lang-toggle--on .lang-toggle-pill {
    background: var(--color-accent);
  }
  .lang-toggle-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: white;
    box-shadow: 0 1px 3px rgba(0,0,0,0.25);
    transition: transform 0.18s;
  }
  .lang-toggle--on .lang-toggle-thumb {
    transform: translateX(12px);
  }
</style>