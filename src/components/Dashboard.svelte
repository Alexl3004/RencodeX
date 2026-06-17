<script lang="ts">
  import { stats } from "$lib/stores/stats.svelte";
  import { formatSize } from "$lib/utils";

  let { onClose }: { onClose?: () => void } = $props();

  let totalFiles    = $derived(stats.totalFiles);
  let totalSavedMb  = $derived(stats.totalSavedMb);
  let avgInputMb    = $derived(stats.avgInputMb);
  let avgOutputMb   = $derived(stats.avgOutputMb);
  let avgRatioPct   = $derived(stats.avgRatioPct);
  let lastUpdated   = $derived(stats.lastUpdated);

  let confirmReset = $state(false);

  function formattedDate(iso: string | null): string {
    if (!iso) return "--";
    const d = new Date(iso);
    return d.toLocaleDateString("fr-FR", { day: "2-digit", month: "2-digit", year: "numeric" }) +
      " " + d.toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" });
  }

  function handleReset() {
    if (!confirmReset) {
      confirmReset = true;
      setTimeout(() => (confirmReset = false), 3000);
      return;
    }
    stats.reset();
    confirmReset = false;
  }
</script>

<div class="border border-[var(--color-border)] rounded-[2px] bg-[var(--color-panel)] flex flex-col">
  <!-- Header -->
  <div class="flex items-center justify-between px-4 py-3 border-b border-[var(--color-border)] bg-[var(--color-panel)]">
    <div class="flex items-center gap-2">
      <div class="w-[3px] h-4 rounded-[1px] bg-[var(--color-accent)]"></div>
      <span class="section-label">Dashboard</span>
    </div>
    {#if onClose}
      <button onclick={onClose} class="text-[var(--color-subtext)] hover:text-[var(--color-text)] transition-colors">
        ✕
      </button>
    {/if}
  </div>

  <!-- Contenu -->
  <div class="px-4 pb-4 space-y-5 overflow-y-auto max-h-[70vh]">

    <div class="pt-4">
      <p class="font-mono text-[10px] text-[var(--color-subtext)]">
        Statistiques cumulées sur l'ensemble des encodages réussis
      </p>
    </div>

    <!-- Espace économisé total (mise en avant) -->
    <div class="bg-[var(--color-success)]/10 border border-[var(--color-success)]/30 rounded-[2px] px-4 py-3">
      <div class="section-label mb-1">Espace économisé total</div>
      <div class="font-mono text-[24px] font-bold text-[var(--color-success)] leading-none">
        {formatSize(totalSavedMb)}
      </div>
    </div>

    <!-- Grille de stats -->
    <div class="grid grid-cols-2 gap-3">
      <div class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[2px] p-3">
        <div class="section-label mb-1.5">Fichiers encodés</div>
        <div class="font-mono text-[18px] font-bold text-[var(--color-text)] leading-none">
          {totalFiles}
        </div>
      </div>

      <div class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[2px] p-3">
        <div class="section-label mb-1.5">Ratio compression moy.</div>
        <div class="font-mono text-[18px] font-bold text-[var(--color-accent)] leading-none">
          {avgRatioPct > 0 ? "-" : ""}{avgRatioPct.toFixed(1)}%
        </div>
      </div>

      <div class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[2px] p-3">
        <div class="section-label mb-1.5">Taille entrée moy.</div>
        <div class="font-mono text-[18px] font-bold text-[var(--color-text)] leading-none">
          {formatSize(avgInputMb)}
        </div>
      </div>

      <div class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[2px] p-3">
        <div class="section-label mb-1.5">Taille sortie moy.</div>
        <div class="font-mono text-[18px] font-bold text-[var(--color-text)] leading-none">
          {formatSize(avgOutputMb)}
        </div>
      </div>
    </div>

    <!-- Barre de visualisation du ratio moyen -->
    <div class="relative h-5 bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[2px] overflow-hidden">
      <div class="absolute inset-y-0 left-0 bg-[var(--color-success)]/30 transition-all duration-300"
           style:width="{Math.max(0, 100 - avgRatioPct)}%"></div>
      <div class="absolute inset-0 flex items-center justify-center font-mono text-[10px] text-[var(--color-text)]">
        {formatSize(avgOutputMb)} / {formatSize(avgInputMb)} en moyenne
      </div>
    </div>

    <!-- Footer : dernière maj + reset -->
    <div class="flex items-center justify-between pt-1 border-t border-[var(--color-border)] mt-1">
      <span class="font-mono text-[9px] text-[var(--color-subtext2)] pt-2">
        Dernière mise à jour : {formattedDate(lastUpdated)}
      </span>
      <button type="button" onclick={handleReset}
        class="btn font-mono text-[9px] px-2 py-1 mt-1 {confirmReset ? 'btn-danger' : 'btn-secondary'}">
        {confirmReset ? "Confirmer ?" : "Réinitialiser"}
      </button>
    </div>
  </div>
</div>