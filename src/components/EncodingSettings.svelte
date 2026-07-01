<script lang="ts">
  import { encoder, SEASON_EPISODE_FORMATS, formatSeasonEpisode, TAG_LABELS, type TagId } from "$lib/stores/encoder.svelte";
  import { formatSize } from "$lib/utils";
  import { X, ChevronDown, ChevronUp, ArrowUp, ArrowDown } from '@lucide/svelte';

  let { onClose }: { onClose?: () => void } = $props();
  let crf    = $derived(encoder.crf);
  let preset = $derived(encoder.preset);
  let seFormat = $derived(encoder.seasonEpisodeFormat);
  let tagOrder = $derived(encoder.tagOrder);
  let team     = $derived(encoder.team);

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
    totalOriginalMb > 0 ? ((totalOriginalMb - estimatedTotalMb) / totalOriginalMb) * 100 : 0,
  );

  const presetInfo: Record<string, { speed: string; gain: string; desc: string }> = {
    p1: { speed: "Ultra rapide", gain: "Gain faible",    desc: "Encodage très rapide, fichier plus gros" },
    p2: { speed: "Très rapide",  gain: "Gain faible+",   desc: "Rapide, bon compromis" },
    p3: { speed: "Rapide",       gain: "Gain moyen",     desc: "Rapide, bonne qualité" },
    p4: { speed: "Normal+",      gain: "Bon gain",       desc: "Bon équilibre vitesse/qualité" },
    p5: { speed: "Normal",       gain: "Excellent gain", desc: "Meilleur compromis (recommandé)" },
    p6: { speed: "Lent",         gain: "Gain supérieur", desc: "Lent, fichier plus petit" },
    p7: { speed: "Très lent",    gain: "Gain maximal",   desc: "Très lent, fichier très petit" },
  };

  const crfGuide = [
    { range: [18, 20], label: "Archivage",    size: "Très gros",    quality: "Parfaite" },
    { range: [21, 23], label: "Home cinéma",  size: "Gros",         quality: "Excellente" },
    { range: [24, 26], label: "Usage général",size: "Moyen",        quality: "Très bonne" },
    { range: [27, 29], label: "Web / Mobile", size: "Raisonnable",  quality: "Bonne" },
    { range: [30, 35], label: "Compact",      size: "Petit",        quality: "Correcte" },
  ];

  let currentCrfInfo = $derived(
    crfGuide.find((g) => crf >= g.range[0] && crf <= g.range[1]) ?? crfGuide[3],
  );

  let progressVal = $derived(Math.round(100 - estimatedGainPct));

  // --- Accordion state ---
  let openSections = $state<Record<string, boolean>>({
    crf: false,
    preset: false,
    seFormat: false,
    tagOrder: false,
    audio: false,
    nvenc: false,
    container: false,
    estimate: false,
  });

  function toggle(key: string) {
    openSections[key] = !openSections[key];
  }
  function chevronClass(open: boolean) {
    return open ? "acc-chevron acc-chevron--open" : "acc-chevron";
  }
</script>

<div class="panel-root">
  <!-- Header -->
  <div class="panel-header">
    <div class="flex items-center gap-2">
      <div class="w-[3px] h-4 rounded-[1px]" style="background: var(--color-accent);"></div>
      <span class="section-label">Paramètres d'encodage</span>
    </div>
    {#if onClose}
      <button onclick={onClose} class="icon-btn" aria-label="Fermer">
        <X class="w-4 h-4" />
      </button>
    {/if}
  </div>

  <!-- Body -->
  <div class="panel-body">

    <!-- CRF -->
    <div class="acc-item">
      <button type="button" class="acc-trigger" onclick={() => toggle('crf')} aria-expanded={openSections.crf}>
        <div class="acc-trigger-left">
          <span class="section-label">Qualité CRF</span>
          {#if !openSections.crf}
            <span class="acc-summary">{crf} · {currentCrfInfo.label}</span>
          {/if}
        </div>
        <div class="acc-trigger-right">
          <span class="acc-value">{crf}</span>
          <ChevronDown class={chevronClass(openSections.crf)} />
        </div>
      </button>

      {#if openSections.crf}
        <div class="acc-content space-y-3">
          <p class="font-mono text-[10px]" style="color: var(--color-subtext);">Niveau de compression vidéo</p>

          <input
            type="range"
            value={crf}
            oninput={(e: Event) => encoder.setCrf(parseInt((e.target as HTMLInputElement).value))}
            min="18" max="35" step="1"
            class="crf-range w-full"
            aria-label="Valeur CRF"
          />

          <div class="flex justify-between font-mono text-[9px]" style="color: var(--color-subtext2);">
            <span>← Meilleure qualité</span>
            <span>Fichier plus petit →</span>
          </div>

          <div class="info-box">
            <span class="font-mono text-[11px]" style="color: var(--color-text);">{currentCrfInfo.label}</span>
            <div class="font-mono text-[9px] mt-0.5" style="color: var(--color-subtext);">
              Taille {currentCrfInfo.size} · Qualité {currentCrfInfo.quality}
            </div>
          </div>
        </div>
      {/if}
    </div>

    <!-- Preset -->
    <div class="acc-item">
      <button type="button" class="acc-trigger" onclick={() => toggle('preset')} aria-expanded={openSections.preset}>
        <div class="acc-trigger-left">
          <span class="section-label">Vitesse d'encodage</span>
          {#if !openSections.preset}
            <span class="acc-summary">{preset.toUpperCase()} · {presetInfo[preset].speed}</span>
          {/if}
        </div>
        <div class="acc-trigger-right">
          <ChevronDown class={chevronClass(openSections.preset)} />
        </div>
      </button>

      {#if openSections.preset}
        <div class="acc-content space-y-3">
          <p class="font-mono text-[10px]" style="color: var(--color-subtext);">Vitesse vs taille finale</p>

          <div class="grid grid-cols-7 gap-1">
            {#each ["p1", "p2", "p3", "p4", "p5", "p6", "p7"] as p}
              <button
                type="button"
                onclick={() => encoder.setPreset(p)}
                class="font-mono text-[10px] px-0 py-1.5 rounded-[var(--radius-sm)] transition-all preset-btn"
                class:preset-btn--active={preset === p}
              >
                {p.toUpperCase()}
              </button>
            {/each}
          </div>

          <div class="info-box">
            <div class="flex items-center gap-2">
              <span class="font-mono text-[11px]" style="color: var(--color-text);">{presetInfo[preset].speed}</span>
              {#if preset === "p5"}
                <span class="font-mono text-[9px] px-1.5 py-0.5 rounded-[var(--radius-full)]"
                      style="background: color-mix(in srgb, var(--color-success) 15%, transparent); border: 1px solid color-mix(in srgb, var(--color-success) 30%, transparent); color: var(--color-success);">
                  REC.
                </span>
              {/if}
            </div>
            <div class="font-mono text-[10px] mt-0.5" style="color: var(--color-subtext);">{presetInfo[preset].desc}</div>
          </div>
        </div>
      {/if}
    </div>

    <!-- Format Saison/Épisode -->
    <div class="acc-item">
      <button type="button" class="acc-trigger" onclick={() => toggle('seFormat')} aria-expanded={openSections.seFormat}>
        <div class="acc-trigger-left">
          <span class="section-label">Format saison/épisode</span>
          {#if !openSections.seFormat}
            <span class="acc-summary">{SEASON_EPISODE_FORMATS.find((f: any) => f.value === seFormat)?.label}</span>
          {/if}
        </div>
        <div class="acc-trigger-right">
          <ChevronDown class={chevronClass(openSections.seFormat)} />
        </div>
      </button>

      {#if openSections.seFormat}
        <div class="acc-content space-y-3">
          <p class="font-mono text-[10px]" style="color: var(--color-subtext);">Affichage dans le nom de sortie</p>

          <div class="grid grid-cols-2 gap-1.5">
            {#each SEASON_EPISODE_FORMATS as f}
              <button
                type="button"
                onclick={() => encoder.setSeasonEpisodeFormat(f.value)}
                class="font-mono text-[11px] px-2 py-1.5 rounded-[var(--radius-sm)] transition-all preset-btn"
                class:preset-btn--active={seFormat === f.value}
              >
                {f.label}
              </button>
            {/each}
          </div>

          <div class="info-box">
            <span class="font-mono text-[9px]" style="color: var(--color-subtext);">Exemple :</span>
            <span class="font-mono text-[11px] ml-1.5" style="color: var(--color-text);">
              Jujutsu Kaisen {formatSeasonEpisode("S03E01", seFormat)} VOSTFR 1080P BluRay H265 10bit AAC
            </span>
          </div>
        </div>
      {/if}
    </div>

    <!-- Ordre des tags / Team -->
    <div class="acc-item">
      <button type="button" class="acc-trigger" onclick={() => toggle('tagOrder')} aria-expanded={openSections.tagOrder}>
        <div class="acc-trigger-left">
          <span class="section-label">Ordre des tags &amp; team</span>
          {#if !openSections.tagOrder}
            <span class="acc-summary">{team ? `Team : ${team}` : "Ordre par défaut"}</span>
          {/if}
        </div>
        <div class="acc-trigger-right">
          <ChevronDown class={chevronClass(openSections.tagOrder)} />
        </div>
      </button>

      {#if openSections.tagOrder}
        <div class="acc-content space-y-3">
          <p class="font-mono text-[10px]" style="color: var(--color-subtext);">
            Réorganisez les éléments du nom de sortie. Renseignez un nom de team pour l'ajouter où vous voulez.
          </p>

          <input
            type="text"
            value={team}
            oninput={(e: Event) => encoder.setTeam((e.target as HTMLInputElement).value)}
            placeholder="Nom de la team (optionnel)"
            class="w-full font-mono text-[11px] px-2 py-1.5 rounded-[var(--radius-sm)]"
            style="background: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-text);"
          />

          <div class="space-y-1">
            {#each tagOrder as id, i (id)}
              <div class="tag-row">
                <span class="font-mono text-[11px]" style="color: {id === 'team' && !team ? 'var(--color-subtext2)' : 'var(--color-text)'};">
                  {TAG_LABELS[id]}{#if id === 'team' && !team} (vide — ignoré tant qu'aucun nom n'est saisi){/if}
                </span>
                <div class="tag-row-actions">
                  <button
                    type="button"
                    class="icon-btn"
                    disabled={i === 0}
                    onclick={() => encoder.moveTag(id, -1)}
                    aria-label={`Monter ${TAG_LABELS[id]}`}
                  >
                    <ArrowUp class="w-3.5 h-3.5" />
                  </button>
                  <button
                    type="button"
                    class="icon-btn"
                    disabled={i === tagOrder.length - 1}
                    onclick={() => encoder.moveTag(id, 1)}
                    aria-label={`Descendre ${TAG_LABELS[id]}`}
                  >
                    <ArrowDown class="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>
            {/each}
          </div>

          <div class="info-box">
            <span class="font-mono text-[9px]" style="color: var(--color-subtext);">Exemple :</span>
            <span class="font-mono text-[11px] ml-1.5" style="color: var(--color-text);">
              {encoder.getDisplayName({
                cleaned: {
                  title: "Jujutsu Kaisen", season_episode: "S03E01",
                  resolution: "1080P", source: "BluRay", provider: "",
                  audio_tags: "VOSTFR", suggested: "",
                },
                output_name: "VOSTFR AAC",
                path: "", filename: "", size_mb: 0, duration_secs: 0, fps: 0,
                audio_langs: [], sub_langs: [], streams: [], status: "ready", output_ext: ".mkv",
              })}
            </span>
          </div>
        </div>
      {/if}
    </div>

    <!-- Audio -->
    <div class="acc-item">
      <button type="button" class="acc-trigger" onclick={() => toggle('audio')} aria-expanded={openSections.audio}>
        <div class="acc-trigger-left">
          <span class="section-label">Audio</span>
          {#if !openSections.audio}
            <span class="acc-summary">
              {audioMode === "reencode" ? `AAC ${audioBitrate}k` : "Copie sans perte"}
            </span>
          {/if}
        </div>
        <div class="acc-trigger-right">
          <ChevronDown class={chevronClass(openSections.audio)} />
        </div>
      </button>

      {#if openSections.audio}
        <div class="acc-content space-y-3">
          <p class="font-mono text-[10px]" style="color: var(--color-subtext);">Réencodage AAC ou copie sans perte</p>

          <div class="grid grid-cols-2 gap-1.5">
            <button
              type="button"
              onclick={() => encoder.setAudioMode("reencode")}
              class="font-mono text-[11px] px-2 py-1.5 rounded-[var(--radius-sm)] transition-all preset-btn"
              class:preset-btn--active={audioMode === "reencode"}
            >
              Réencoder (AAC)
            </button>
            <button
              type="button"
              onclick={() => encoder.setAudioMode("copy")}
              class="font-mono text-[11px] px-2 py-1.5 rounded-[var(--radius-sm)] transition-all preset-btn"
              class:preset-btn--active={audioMode === "copy"}
            >
              Copier (-c:a copy)
            </button>
          </div>

          {#if audioMode === "reencode"}
            <div class="grid grid-cols-4 gap-1.5">
              {#each [128, 192, 256, 320] as br}
                <button
                  type="button"
                  onclick={() => encoder.setAudioBitrate(br)}
                  class="font-mono text-[10px] px-0 py-1.5 rounded-[var(--radius-sm)] transition-all preset-btn"
                  class:preset-btn--active={audioBitrate === br}
                >
                  {br}k
                </button>
              {/each}
            </div>
          {:else}
            <div class="info-box">
              <span class="font-mono text-[10px]" style="color: var(--color-subtext);">
                La piste audio source sera conservée telle quelle, sans perte de qualité supplémentaire.
              </span>
            </div>
          {/if}
        </div>
      {/if}
    </div>

    <!-- Qualité NVENC avancée -->
    <div class="acc-item">
      <button type="button" class="acc-trigger" onclick={() => toggle('nvenc')} aria-expanded={openSections.nvenc}>
        <div class="acc-trigger-left">
          <span class="section-label">Qualité NVENC avancée</span>
          {#if !openSections.nvenc}
            <span class="acc-summary">
              AQ {spatialAq || temporalAq ? `(${aqStrength})` : "off"} · Multipass {multipass === "disabled" ? "aucun" : multipass === "qres" ? "1/4" : "plein"}
            </span>
          {/if}
        </div>
        <div class="acc-trigger-right">
          <ChevronDown class={chevronClass(openSections.nvenc)} />
        </div>
      </button>

      {#if openSections.nvenc}
        <div class="acc-content space-y-3">
          <p class="font-mono text-[10px]" style="color: var(--color-subtext);">AQ adaptative et multipass</p>

          <div class="grid grid-cols-2 gap-1.5">
            <button
              type="button"
              onclick={() => encoder.setSpatialAq(!spatialAq)}
              class="font-mono text-[11px] px-2 py-1.5 rounded-[var(--radius-sm)] transition-all preset-btn"
              class:preset-btn--active={spatialAq}
            >
              AQ spatiale
            </button>
            <button
              type="button"
              onclick={() => encoder.setTemporalAq(!temporalAq)}
              class="font-mono text-[11px] px-2 py-1.5 rounded-[var(--radius-sm)] transition-all preset-btn"
              class:preset-btn--active={temporalAq}
            >
              AQ temporelle
            </button>
          </div>

          {#if spatialAq || temporalAq}
            <div class="flex justify-between items-start">
              <span class="font-mono text-[10px]" style="color: var(--color-subtext);">Force AQ</span>
              <span class="font-mono text-[12px] font-bold" style="color: var(--color-accent);">{aqStrength}</span>
            </div>
            <input
              type="range"
              value={aqStrength}
              oninput={(e: Event) => encoder.setAqStrength(parseInt((e.target as HTMLInputElement).value))}
              min="1" max="15" step="1"
              class="crf-range w-full"
              aria-label="Force AQ"
            />
          {/if}

          <div class="grid grid-cols-3 gap-1.5">
            <button
              type="button"
              onclick={() => encoder.setMultipass("disabled")}
              class="font-mono text-[10px] px-0 py-1.5 rounded-[var(--radius-sm)] transition-all preset-btn"
              class:preset-btn--active={multipass === "disabled"}
            >
              Aucun
            </button>
            <button
              type="button"
              onclick={() => encoder.setMultipass("qres")}
              class="font-mono text-[10px] px-0 py-1.5 rounded-[var(--radius-sm)] transition-all preset-btn"
              class:preset-btn--active={multipass === "qres"}
            >
              1/4 rés.
            </button>
            <button
              type="button"
              onclick={() => encoder.setMultipass("fullres")}
              class="font-mono text-[10px] px-0 py-1.5 rounded-[var(--radius-sm)] transition-all preset-btn"
              class:preset-btn--active={multipass === "fullres"}
            >
              Pleine rés.
            </button>
          </div>
        </div>
      {/if}
    </div>

    <!-- Conteneur -->
    <div class="acc-item">
      <button type="button" class="acc-trigger" onclick={() => toggle('container')} aria-expanded={openSections.container}>
        <div class="acc-trigger-left">
          <span class="section-label">Conteneur de sortie</span>
          {#if !openSections.container}
            <span class="acc-summary">{container.toUpperCase()}</span>
          {/if}
        </div>
        <div class="acc-trigger-right">
          <ChevronDown class={chevronClass(openSections.container)} />
        </div>
      </button>

      {#if openSections.container}
        <div class="acc-content space-y-3">
          <p class="font-mono text-[10px]" style="color: var(--color-subtext);">Format du fichier final</p>

          <div class="grid grid-cols-2 gap-1.5">
            <button
              type="button"
              onclick={() => encoder.setContainer("mkv")}
              class="font-mono text-[11px] px-2 py-1.5 rounded-[var(--radius-sm)] transition-all preset-btn"
              class:preset-btn--active={container === "mkv"}
            >
              MKV
            </button>
            <button
              type="button"
              onclick={() => encoder.setContainer("mp4")}
              class="font-mono text-[11px] px-2 py-1.5 rounded-[var(--radius-sm)] transition-all preset-btn"
              class:preset-btn--active={container === "mp4"}
            >
              MP4
            </button>
          </div>

          {#if container === "mp4"}
            <div class="info-box">
              <span class="font-mono text-[10px]" style="color: var(--color-subtext);">
                Sous-titres convertis en mov_text — meilleure compatibilité Apple/mobile, MKV reste recommandé pour l'ASS avancé.
              </span>
            </div>
          {/if}
        </div>
      {/if}
    </div>

    <!-- Estimation -->
    {#if totalOriginalMb > 0}
      <div class="acc-item">
        <button type="button" class="acc-trigger" onclick={() => toggle('estimate')} aria-expanded={openSections.estimate}>
          <div class="acc-trigger-left">
            <span class="section-label">Estimation résultat</span>
            {#if !openSections.estimate}
              <span class="acc-summary">{formatSize(estimatedTotalMb)} (-{estimatedGainPct.toFixed(1)}%)</span>
            {/if}
          </div>
          <div class="acc-trigger-right">
            <ChevronDown class={chevronClass(openSections.estimate)} />
          </div>
        </button>

        {#if openSections.estimate}
          <div class="acc-content space-y-3">
            <p class="font-mono text-[10px]" style="color: var(--color-subtext);">Prévision basée sur les paramètres actuels</p>

            <div class="grid grid-cols-3 gap-2">
              <div class="stat-box">
                <div class="section-label mb-1">ORIGINAL</div>
                <div class="font-mono text-[12px] font-bold" style="color: var(--color-text);">{formatSize(totalOriginalMb)}</div>
              </div>
              <div class="stat-box">
                <div class="section-label mb-1">ESTIMÉ</div>
                <div class="font-mono text-[12px] font-bold" style="color: var(--color-success);">{formatSize(estimatedTotalMb)}</div>
              </div>
              <div class="stat-box">
                <div class="section-label mb-1">GAIN</div>
                <div class="font-mono text-[12px] font-bold" style="color: var(--color-success);">-{estimatedGainPct.toFixed(1)}%</div>
              </div>
            </div>

            <div class="relative rounded-[var(--radius-sm)] overflow-hidden" style="height: 20px; background: var(--color-surface); border: 1px solid var(--color-border);">
              <div class="absolute inset-y-0 left-0 rounded-[1px]"
                   style="width: {progressVal}%; background: var(--color-success); transition: width 0.3s;"></div>
              <div class="absolute inset-0 flex items-center justify-center font-mono text-[10px]"
                   style="color: var(--color-text);">
                {formatSize(estimatedTotalMb)} / {formatSize(totalOriginalMb)}
              </div>
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .panel-root {
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-panel);
    flex-shrink: 0;
  }

  .panel-body {
    padding: 8px 16px 16px;
    overflow-y: auto;
  }

  .icon-btn {
    width: 24px;
    height: 24px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-xs);
    border: 1px solid transparent;
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    transition: background 0.1s, color 0.1s, border-color 0.1s;
  }
  .icon-btn:hover {
    background: var(--color-panel2);
    border-color: var(--color-border);
    color: var(--color-text);
  }
  .icon-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .tag-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
  }
  .tag-row-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  /* Accordion */
  .acc-item {
    border-bottom: 1px solid var(--color-border);
  }
  .acc-item:last-child {
    border-bottom: none;
  }

  .acc-trigger {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    background: transparent;
    border: none;
    padding: 12px 2px;
    cursor: pointer;
    text-align: left;
  }
  .acc-trigger:hover .section-label {
    color: var(--color-text);
  }

  .acc-trigger-left {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .acc-summary {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .acc-trigger-right {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .acc-value {
    font-family: "Geist Mono", monospace;
    font-size: 14px;
    font-weight: 700;
    color: var(--color-accent);
  }

  :global(.acc-chevron) {
    width: 14px;
    height: 14px;
    color: var(--color-subtext);
    transition: transform 0.15s ease;
  }
  :global(.acc-chevron--open) {
    transform: rotate(180deg);
  }

  .acc-content {
    padding: 0 2px 14px;
  }

  .info-box {
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
  }

  .stat-box {
    padding: 10px;
    border-radius: var(--radius-sm);
    text-align: center;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
  }

  /* Range CRF */
  .crf-range {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 4px;
    border-radius: var(--radius-xs);
    background: var(--color-border2);
    outline: none;
    cursor: pointer;
  }
  .crf-range::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--color-accent);
    border: 2px solid var(--color-panel);
    box-shadow: 0 0 0 1px var(--color-accent);
    cursor: pointer;
    transition: transform 0.1s;
  }
  .crf-range::-webkit-slider-thumb:hover {
    transform: scale(1.15);
  }

  /* Preset buttons */
  .preset-btn {
    border: 1px solid var(--color-border);
    background: var(--color-panel2);
    color: var(--color-subtext);
    cursor: pointer;
    font-family: "Geist Mono", monospace;
  }
  .preset-btn:hover {
    border-color: var(--color-subtext2);
    color: var(--color-text);
  }
  .preset-btn--active {
    background: var(--color-accent);
    border-color: var(--color-accent);
    color: #fff;
  }
</style>