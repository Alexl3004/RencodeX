<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { formatSize } from "$lib/utils";
  import { X } from '@lucide/svelte';

  let { onClose }: { onClose?: () => void } = $props();
  let crf    = $derived(encoder.crf);
  let preset = $derived(encoder.preset);

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
  <div class="panel-body space-y-5">

    <!-- CRF -->
    <div class="space-y-3">
      <div class="flex justify-between items-start">
        <div>
          <div class="section-label">Qualité CRF</div>
          <p class="font-mono text-[10px] mt-0.5" style="color: var(--color-subtext);">Niveau de compression vidéo</p>
        </div>
        <div class="text-right">
          <span class="font-mono text-[22px] font-bold leading-none" style="color: var(--color-accent);">{crf}</span>
          <span class="font-mono text-[10px] ml-1" style="color: var(--color-subtext);">/ 35</span>
        </div>
      </div>

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

      <div class="px-3 py-2.5 rounded-[2px]" style="background: var(--color-surface); border: 1px solid var(--color-border);">
        <span class="font-mono text-[11px]" style="color: var(--color-text);">{currentCrfInfo.label}</span>
        <div class="font-mono text-[9px] mt-0.5" style="color: var(--color-subtext);">
          Taille {currentCrfInfo.size} · Qualité {currentCrfInfo.quality}
        </div>
      </div>
    </div>

    <!-- Preset -->
    <div class="space-y-3 pt-1" style="border-top: 1px solid var(--color-border);">
      <div class="pt-2">
        <div class="section-label">Vitesse d'encodage</div>
        <p class="font-mono text-[10px] mt-0.5" style="color: var(--color-subtext);">Vitesse vs taille finale</p>
      </div>

      <div class="grid grid-cols-7 gap-1">
        {#each ["p1", "p2", "p3", "p4", "p5", "p6", "p7"] as p}
          <button
            type="button"
            onclick={() => encoder.setPreset(p)}
            class="font-mono text-[10px] px-0 py-1.5 rounded-[2px] transition-all preset-btn"
            class:preset-btn--active={preset === p}
          >
            {p.toUpperCase()}
          </button>
        {/each}
      </div>

      <div class="px-3 py-2.5 rounded-[2px]" style="background: var(--color-surface); border: 1px solid var(--color-border);">
        <div class="flex items-center gap-2">
          <span class="font-mono text-[11px]" style="color: var(--color-text);">{presetInfo[preset].speed}</span>
          {#if preset === "p5"}
            <span class="font-mono text-[9px] px-1.5 py-0.5 rounded-[2px]"
                  style="background: color-mix(in srgb, var(--color-success) 15%, transparent); border: 1px solid color-mix(in srgb, var(--color-success) 30%, transparent); color: var(--color-success);">
              REC.
            </span>
          {/if}
        </div>
        <div class="font-mono text-[10px] mt-0.5" style="color: var(--color-subtext);">{presetInfo[preset].desc}</div>
      </div>
    </div>

    <!-- Estimation -->
    {#if totalOriginalMb > 0}
      <div class="space-y-3 pt-1" style="border-top: 1px solid var(--color-border);">
        <div class="pt-2">
          <div class="section-label">Estimation résultat</div>
          <p class="font-mono text-[10px] mt-0.5" style="color: var(--color-subtext);">Prévision basée sur les paramètres actuels</p>
        </div>

        <div class="grid grid-cols-3 gap-2">
          <div class="p-2.5 rounded-[2px] text-center" style="background: var(--color-surface); border: 1px solid var(--color-border);">
            <div class="section-label mb-1">ORIGINAL</div>
            <div class="font-mono text-[12px] font-bold" style="color: var(--color-text);">{formatSize(totalOriginalMb)}</div>
          </div>
          <div class="p-2.5 rounded-[2px] text-center" style="background: var(--color-surface); border: 1px solid var(--color-border);">
            <div class="section-label mb-1">ESTIMÉ</div>
            <div class="font-mono text-[12px] font-bold" style="color: var(--color-success);">{formatSize(estimatedTotalMb)}</div>
          </div>
          <div class="p-2.5 rounded-[2px] text-center" style="background: var(--color-surface); border: 1px solid var(--color-border);">
            <div class="section-label mb-1">GAIN</div>
            <div class="font-mono text-[12px] font-bold" style="color: var(--color-success);">-{estimatedGainPct.toFixed(1)}%</div>
          </div>
        </div>

        <div class="relative rounded-[2px] overflow-hidden" style="height: 20px; background: var(--color-surface); border: 1px solid var(--color-border);">
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
</div>

<style>
  .panel-root {
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    border-radius: 4px;
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
    padding: 16px;
    overflow-y: auto;
  }

  .icon-btn {
    width: 24px;
    height: 24px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
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

  /* Range CRF */
  .crf-range {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 4px;
    border-radius: 2px;
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