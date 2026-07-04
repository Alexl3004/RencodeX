<script lang="ts">
  import {
    encoder,
    type MultipassMode,
    type SubExtractFormat,
    type SubExtractNaming,
    type SubExtractPathMode,
  } from "$lib/stores/encoder.svelte";
  import { formatSize } from "$lib/utils";
  import { FolderOpen, Gauge, Volume2, Cpu, Box, Subtitles, BarChart3 } from "@lucide/svelte";

  let { onClose }: { onClose?: () => void } = $props();

  let crf          = $derived(encoder.crf);
  let preset       = $derived(encoder.preset);
  let audioMode    = $derived(encoder.audioMode);
  let audioBitrate = $derived(encoder.audioBitrate);
  let spatialAq    = $derived(encoder.spatialAq);
  let temporalAq   = $derived(encoder.temporalAq);
  let aqStrength   = $derived(encoder.aqStrength);
  let multipass    = $derived(encoder.multipass);
  let container    = $derived(encoder.container);

  let totalOriginalMb = $derived(
    encoder.files.reduce((s: number, f: any) => s + (f.size_mb || 0), 0),
  );

  let estimatedRatio = $derived.by(() => {
    const crfBase = 0.25 + (28 - crf) * 0.025;
    const pf: Record<string, number> = {
      p1: 1.3, p2: 1.2, p3: 1.1, p4: 1.05, p5: 1.0, p6: 0.92, p7: 0.85,
    };
    return Math.min(0.7, crfBase * (pf[preset] || 1.0));
  });

  let estimatedTotalMb = $derived(totalOriginalMb * estimatedRatio);
  let estimatedGainPct = $derived(
    totalOriginalMb > 0
      ? ((totalOriginalMb - estimatedTotalMb) / totalOriginalMb) * 100
      : 0,
  );

  const presetMeta: Record<string, { label: string; desc: string; speed: number }> = {
    p1: { label: "Ultra rapide", desc: "Encodage rapide, compression minimale",        speed: 7 },
    p2: { label: "Très rapide",  desc: "Bon compromis pour les tests",                 speed: 6 },
    p3: { label: "Rapide",       desc: "Rapide avec bonne qualité",                    speed: 5 },
    p4: { label: "Normal+",      desc: "Équilibre vitesse / compression",              speed: 4 },
    p5: { label: "Normal",       desc: "Recommandé — meilleur ratio qualité/taille",   speed: 3 },
    p6: { label: "Lent",         desc: "Fichier plus petit, temps plus long",          speed: 2 },
    p7: { label: "Très lent",    desc: "Compression maximale, très lent",              speed: 1 },
  };

  const crfBands = [
    { range: [18, 20], label: "Archivage",    quality: "Transparente",  color: "#6ee7b7" },
    { range: [21, 23], label: "Home cinéma",  quality: "Excellente",    color: "#86efac" },
    { range: [24, 26], label: "Usage général",quality: "Très bonne",    color: "#fbbf24" },
    { range: [27, 29], label: "Web / Mobile", quality: "Bonne",         color: "#fb923c" },
    { range: [30, 35], label: "Compact",      quality: "Correcte",      color: "#f87171" },
  ];
  let currentBand = $derived(
    crfBands.find(b => crf >= b.range[0] && crf <= b.range[1]) ?? crfBands[2]
  );

  // ── Navigation sections ────────────────────────────────────────────────────
  type SectionId = "quality" | "speed" | "audio" | "nvenc" | "container" | "subtitles";

  const SECTIONS: { id: SectionId; label: string; icon: any; desc: string }[] = [
    { id: "quality",   label: "Qualité",       icon: Gauge,       desc: "CRF & compression" },
    { id: "speed",     label: "Vitesse",        icon: BarChart3,   desc: "Preset encodeur" },
    { id: "audio",     label: "Audio",          icon: Volume2,     desc: "Codec & débit" },
    { id: "nvenc",     label: "NVENC",          icon: Cpu,         desc: "AQ & multipass" },
    { id: "container", label: "Conteneur",      icon: Box,         desc: "MKV / MP4" },
    { id: "subtitles", label: "Sous-titres",    icon: Subtitles,   desc: "Extraction" },
  ];

  let activeSection = $state<SectionId>("quality");

  // ── Compression bar width ──────────────────────────────────────────────────
  let barWidth = $derived(Math.round(100 - estimatedGainPct));
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

    <!-- ═══════════════ QUALITÉ ═══════════════ -->
    {#if activeSection === "quality"}
      <section class="content-section">
        <header class="section-header">
          <div>
            <h2 class="section-title">Qualité CRF</h2>
            <p class="section-desc">
              Contrôle le taux de compression vidéo. Une valeur basse préserve plus de détails,
              une valeur haute réduit davantage la taille du fichier.
            </p>
          </div>
          <div class="section-badge" style="--badge-color: {currentBand.color}">
            <span class="badge-value">{crf}</span>
            <span class="badge-label">{currentBand.label}</span>
          </div>
        </header>

        <div class="field-block">
          <div class="crf-track-wrap">
            <input
              type="range"
              value={crf}
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
        </div>

        <div class="band-grid">
          {#each crfBands as band}
            {@const active = crf >= band.range[0] && crf <= band.range[1]}
            <div class="band-card {active ? 'band-card--active' : ''}" style="--c: {band.color}">
              <div class="band-range">{band.range[0]}–{band.range[1]}</div>
              <div class="band-name">{band.label}</div>
              <div class="band-quality">{band.quality}</div>
            </div>
          {/each}
        </div>
      </section>

    <!-- ═══════════════ VITESSE ═══════════════ -->
    {:else if activeSection === "speed"}
      <section class="content-section">
        <header class="section-header">
          <div>
            <h2 class="section-title">Vitesse d'encodage</h2>
            <p class="section-desc">
              Détermine le temps passé par l'encodeur à optimiser chaque image.
              Un preset plus lent produit un fichier plus compact à qualité égale.
            </p>
          </div>
          <div class="active-preset-badge">
            <span class="apb-id">{preset.toUpperCase()}</span>
            <span class="apb-label">{presetMeta[preset].label}</span>
          </div>
        </header>

        <div class="field-block">
          <div class="preset-grid">
            {#each ["p1","p2","p3","p4","p5","p6","p7"] as p}
              {@const meta = presetMeta[p]}
              {@const isActive = preset === p}
              <button
                type="button"
                class="preset-card {isActive ? 'preset-card--active' : ''}"
                onclick={() => encoder.setPreset(p)}
              >
                <div class="preset-card-top">
                  <span class="preset-id">{p.toUpperCase()}</span>
                  {#if p === "p5"}
                    <span class="preset-rec">REC</span>
                  {/if}
                </div>
                <div class="preset-name">{meta.label}</div>
                <div class="preset-speed-bar">
                  {#each Array(7) as _, i}
                    <div class="speed-dot {i < meta.speed ? 'speed-dot--on' : ''}"></div>
                  {/each}
                </div>
              </button>
            {/each}
          </div>

          <div class="preset-desc-box">
            <p class="pdb-text">{presetMeta[preset].desc}</p>
          </div>
        </div>
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

    <!-- ═══════════════ NVENC ═══════════════ -->
    {:else if activeSection === "nvenc"}
      <section class="content-section">
        <header class="section-header">
          <div>
            <h2 class="section-title">NVENC avancé</h2>
            <p class="section-desc">
              L'AQ adaptative améliore la distribution des bits sur les zones complexes.
              Le multipass analyse l'image en plusieurs passes pour affiner la compression.
            </p>
          </div>
        </header>

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
        </div>

        {#if spatialAq || temporalAq}
          <div class="field-block">
            <div class="field-label-row">
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
          </div>
        {/if}

        <div class="field-block">
          <div class="field-label">Multipass</div>
          <div class="multipass-row">
            {#each [
              { val: "disabled", label: "Aucun",       sub: "passe unique" },
              { val: "qres",     label: "¼ résolution", sub: "analyse rapide" },
              { val: "fullres",  label: "Pleine rés.",  sub: "qualité max" },
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

  /* Active preset badge */
  .active-preset-badge {
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 10px 16px;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-surface));
    border: 1px solid color-mix(in srgb, var(--color-accent) 22%, var(--color-border));
    gap: 2px;
  }
  .apb-id {
    font-family: "Geist Mono", monospace;
    font-size: 20px;
    font-weight: 700;
    color: var(--color-accent);
    line-height: 1;
  }
  .apb-label {
    font-size: 10px;
    color: var(--color-subtext);
    white-space: nowrap;
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
  .band-quality {
    font-family: "Geist Mono", monospace;
    font-size: 8px;
    color: var(--color-subtext2);
  }

  /* ── Preset grid ────────────────────────────────────────────────────────── */
  .preset-grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 6px;
    margin-bottom: 14px;
  }
  .preset-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 5px;
    padding: 10px 4px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
  }
  .preset-card:hover { border-color: var(--color-subtext2); }
  .preset-card--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 10%, var(--color-surface));
  }

  .preset-card-top {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }
  .preset-id {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    font-weight: 700;
    color: var(--color-subtext);
    transition: color 0.15s;
  }
  .preset-card--active .preset-id { color: var(--color-accent); }

  .preset-rec {
    font-family: "Geist Mono", monospace;
    font-size: 7px;
    letter-spacing: 0.04em;
    padding: 1px 4px;
    border-radius: 3px;
    background: color-mix(in srgb, var(--color-success) 15%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-success) 30%, transparent);
    color: var(--color-success);
    line-height: 1.4;
  }

  .preset-name {
    font-size: 9px;
    color: var(--color-subtext2);
    text-align: center;
    line-height: 1.2;
  }

  .preset-speed-bar {
    display: flex;
    gap: 2px;
    align-items: center;
  }
  .speed-dot {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--color-border);
    transition: background 0.15s;
  }
  .speed-dot--on { background: var(--color-accent); opacity: 0.7; }
  .preset-card--active .speed-dot--on { opacity: 1; }

  .preset-desc-box {
    padding: 12px 14px;
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
  }
  .pdb-text {
    font-size: 12px;
    color: var(--color-subtext);
    margin: 0;
    line-height: 1.5;
  }

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
</style>