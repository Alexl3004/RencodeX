<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { formatSize } from "$lib/utils";

  let { onClose }: { onClose?: () => void } = $props();
  let collapsed = $state(false);
  let crf = $derived(encoder.crf);
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
    p1: { speed: "Ultra rapide", gain: "Gain faible", desc: "Encodage très rapide, fichier plus gros" },
    p2: { speed: "Très rapide", gain: "Gain faible+", desc: "Rapide, bon compromis" },
    p3: { speed: "Rapide", gain: "Gain moyen", desc: "Rapide, bonne qualité" },
    p4: { speed: "Normal+", gain: "Bon gain", desc: "Bon équilibre vitesse/qualité" },
    p5: { speed: "Normal", gain: "Excellent gain", desc: "Meilleur compromis (recommandé)" },
    p6: { speed: "Lent", gain: "Gain supérieur", desc: "Lent, fichier plus petit" },
    p7: { speed: "Très lent", gain: "Gain maximal", desc: "Très lent, fichier très petit" },
  };

  const crfGuide = [
    { range: [18, 20], label: "Archivage", size: "Très gros", quality: "Parfaite" },
    { range: [21, 23], label: "Home cinéma", size: "Gros", quality: "Excellente" },
    { range: [24, 26], label: "Usage général", size: "Moyen", quality: "Très bonne" },
    { range: [27, 29], label: "Web / Mobile", size: "Raisonnable", quality: "Bonne" },
    { range: [30, 35], label: "Compact", size: "Petit", quality: "Correcte" },
  ];
  
  let currentCrfInfo = $derived(
    crfGuide.find((g) => crf >= g.range[0] && crf <= g.range[1]) ?? crfGuide[3],
  );
</script>

<div class="border border-[var(--color-border)] rounded-[2px] bg-[var(--color-panel)] flex flex-col">
  <!-- Header -->
  <div class="flex items-center justify-between px-4 py-3 border-b border-[var(--color-border)] bg-[var(--color-panel)]">
    <div class="flex items-center gap-2">
      <div class="w-[3px] h-4 rounded-[1px] bg-[var(--color-accent)]"></div>
      <span class="section-label">Paramètres d'encodage</span>
    </div>
    {#if onClose}
      <button onclick={onClose} class="text-[var(--color-subtext)] hover:text-[var(--color-text)] transition-colors">
        ✕
      </button>
    {/if}
  </div>

  <!-- Contenu -->
  <div class="px-4 pb-4 space-y-5 overflow-y-auto max-h-[70vh]">
    <!-- CRF -->
    <div class="space-y-3 pt-4">
      <div class="flex justify-between items-start">
        <div>
          <div class="section-label">Qualité CRF</div>
          <p class="font-mono text-[10px] text-[var(--color-subtext)] mt-0.5">Niveau de compression vidéo</p>
        </div>
        <div class="text-right">
          <span class="font-mono text-[22px] font-bold text-[var(--color-accent)] leading-none">{crf}</span>
          <span class="font-mono text-[10px] text-[var(--color-subtext)] ml-1">/ 35</span>
        </div>
      </div>

      <input type="range" bind:value={crf}
        oninput={(e) => encoder.setCrf(parseInt((e.target as HTMLInputElement).value))}
        min="18" max="35" step="1" class="w-full" />

      <div class="flex justify-between font-mono text-[9px] text-[var(--color-subtext2)]">
        <span>← Meilleure qualité</span><span>Fichier plus petit →</span>
      </div>

      <div class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[2px] px-3 py-2.5">
        <span class="font-mono text-[11px] text-[var(--color-text)]">{currentCrfInfo.label}</span>
        <div class="font-mono text-[9px] text-[var(--color-subtext)] mt-0.5">
          Taille {currentCrfInfo.size} · Qualité {currentCrfInfo.quality}
        </div>
      </div>
    </div>

    <!-- Preset -->
    <div class="space-y-3 pt-1 border-t border-[var(--color-border)]">
      <div class="pt-2">
        <div class="section-label">Vitesse d'encodage</div>
        <p class="font-mono text-[10px] text-[var(--color-subtext)] mt-0.5">Vitesse vs taille finale</p>
      </div>
      
      <div class="grid grid-cols-7 gap-1">
        {#each ["p1", "p2", "p3", "p4", "p5", "p6", "p7"] as p}
          <button type="button" onclick={() => encoder.setPreset(p)}
            class="btn font-mono text-[10px] px-0 py-1.5 rounded-[2px] {preset === p ? 'btn-lang-active' : 'btn-secondary'}">
            {p.toUpperCase()}
          </button>
        {/each}
      </div>
      
      <div class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[2px] px-3 py-2.5">
        <div class="flex items-center gap-2">
          <span class="font-mono text-[11px] text-[var(--color-text)]">{presetInfo[preset].speed}</span>
          {#if preset === "p5"}
            <span class="font-mono text-[9px] bg-[var(--color-success)]/15 text-[var(--color-success)] px-1.5 py-0.5 rounded-[2px]">REC.</span>
          {/if}
        </div>
        <div class="font-mono text-[10px] text-[var(--color-subtext)] mt-0.5">{presetInfo[preset].desc}</div>
      </div>
    </div>

    <!-- Estimation -->
    {#if totalOriginalMb > 0}
      <div class="space-y-3 pt-1 border-t border-[var(--color-border)]">
        <div class="pt-2">
          <div class="section-label">Estimation résultat</div>
          <p class="font-mono text-[10px] text-[var(--color-subtext)] mt-0.5">Prévision basée sur les paramètres actuels</p>
        </div>
        
        <div class="grid grid-cols-3 gap-2">
          <div class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[2px] p-2.5 text-center">
            <div class="section-label mb-1">ORIGINAL</div>
            <div class="font-mono text-[12px] font-bold text-[var(--color-text)]">{formatSize(totalOriginalMb)}</div>
          </div>
          <div class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[2px] p-2.5 text-center">
            <div class="section-label mb-1">ESTIMÉ</div>
            <div class="font-mono text-[12px] font-bold text-[var(--color-success)]">{formatSize(estimatedTotalMb)}</div>
          </div>
          <div class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[2px] p-2.5 text-center">
            <div class="section-label mb-1">GAIN</div>
            <div class="font-mono text-[12px] font-bold text-[var(--color-success)]">-{estimatedGainPct.toFixed(1)}%</div>
          </div>
        </div>
        
        <div class="relative h-5 bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[2px] overflow-hidden">
          <div class="absolute inset-y-0 left-0 bg-[var(--color-success)]/30 transition-all duration-300" style:width="{100 - estimatedGainPct}%"></div>
          <div class="absolute inset-0 flex items-center justify-center font-mono text-[10px] text-[var(--color-text)]">
            {formatSize(estimatedTotalMb)} / {formatSize(totalOriginalMb)}
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>