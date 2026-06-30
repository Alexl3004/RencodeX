<script lang="ts">
  import { stats } from "$lib/stores/stats.svelte";
  import { formatSize } from "$lib/utils";
  import { X } from '@lucide/svelte';
  import { Progress, Popover, Portal } from "@skeletonlabs/skeleton-svelte";

  let { onClose }: { onClose?: () => void } = $props();

  let totalFiles   = $derived(stats.totalFiles);
  let totalSavedMb = $derived(stats.totalSavedMb);
  let avgInputMb   = $derived(stats.avgInputMb);
  let avgOutputMb  = $derived(stats.avgOutputMb);
  let avgRatioPct  = $derived(stats.avgRatioPct);
  let lastUpdated  = $derived(stats.lastUpdated);

  function formattedDate(iso: string | null): string {
    if (!iso) return "--";
    const d = new Date(iso);
    return d.toLocaleDateString("fr-FR", { day: "2-digit", month: "2-digit", year: "numeric" }) +
      " " + d.toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" });
  }

  let progressVal = $derived(Math.max(0, 100 - avgRatioPct));
</script>

<div class="panel-root">
  <!-- Header -->
  <div class="panel-header">
    <div class="flex items-center gap-2">
      <div class="w-[3px] h-4 rounded-[1px]" style="background: var(--color-accent);"></div>
      <span class="section-label">Dashboard</span>
    </div>
    {#if onClose}
      <button onclick={onClose} class="icon-btn" aria-label="Fermer">
        <X class="w-4 h-4" />
      </button>
    {/if}
  </div>

  <!-- Body -->
  <div class="panel-body space-y-5">
    <p class="font-mono text-[10px]" style="color: var(--color-subtext);">
      Statistiques cumulées sur l'ensemble des encodages réussis
    </p>

    <!-- Espace économisé -->
    <div class="px-4 py-3 rounded-[2px]"
         style="background: color-mix(in srgb, var(--color-success) 10%, transparent); border: 1px solid color-mix(in srgb, var(--color-success) 30%, transparent);">
      <div class="section-label mb-1">Espace économisé total</div>
      <div class="font-mono text-[24px] font-bold leading-none" style="color: var(--color-success);">
        {formatSize(totalSavedMb)}
      </div>
    </div>

    <!-- Grille -->
    <div class="grid grid-cols-2 gap-3">
      <div class="p-3 rounded-[2px]" style="background: var(--color-surface); border: 1px solid var(--color-border);">
        <div class="section-label mb-1.5">Fichiers encodés</div>
        <div class="font-mono text-[18px] font-bold leading-none" style="color: var(--color-text);">{totalFiles}</div>
      </div>
      <div class="p-3 rounded-[2px]" style="background: var(--color-surface); border: 1px solid var(--color-border);">
        <div class="section-label mb-1.5">Ratio compression moy.</div>
        <div class="font-mono text-[18px] font-bold leading-none" style="color: var(--color-accent);">
          {avgRatioPct > 0 ? "-" : ""}{avgRatioPct.toFixed(1)}%
        </div>
      </div>
      <div class="p-3 rounded-[2px]" style="background: var(--color-surface); border: 1px solid var(--color-border);">
        <div class="section-label mb-1.5">Taille entrée moy.</div>
        <div class="font-mono text-[18px] font-bold leading-none" style="color: var(--color-text);">{formatSize(avgInputMb)}</div>
      </div>
      <div class="p-3 rounded-[2px]" style="background: var(--color-surface); border: 1px solid var(--color-border);">
        <div class="section-label mb-1.5">Taille sortie moy.</div>
        <div class="font-mono text-[18px] font-bold leading-none" style="color: var(--color-text);">{formatSize(avgOutputMb)}</div>
      </div>
    </div>

    <!-- Barre de ratio -->
    <Progress value={progressVal} max={100} class="relative block">
      <Progress.Track
        class="h-[20px] rounded-[2px] overflow-hidden border border-[var(--color-border)] bg-[var(--color-surface)]"
      >
        <Progress.Range class="rounded-[1px] bg-[var(--color-success)] transition-[width] duration-300" />
      </Progress.Track>
      <div
        class="pointer-events-none absolute inset-0 flex items-center justify-center font-mono text-[10px]"
        style="color: var(--color-text);"
      >
        {formatSize(avgOutputMb)} / {formatSize(avgInputMb)} en moyenne
      </div>
    </Progress>

    <!-- Footer -->
    <div class="flex items-center justify-between pt-1" style="border-top: 1px solid var(--color-border);">
      <span class="font-mono text-[9px] pt-2" style="color: var(--color-subtext2);">
        Dernière mise à jour : {formattedDate(lastUpdated)}
      </span>
      <Popover positioning={{ placement: "top-end" }}>
        <Popover.Trigger class="btn font-mono text-[9px] px-2 py-1 mt-1">
          Réinitialiser
        </Popover.Trigger>
        <Portal>
          <Popover.Positioner>
            <Popover.Content
              class="w-56 p-3 rounded-[2px] shadow-xl"
              style="background: var(--color-panel); border: 1px solid var(--color-border);"
            >
              <Popover.Description class="text-[11px] leading-snug" style="color: var(--color-text);">
                Réinitialiser toutes les statistiques cumulées ?
              </Popover.Description>
              <div class="flex gap-2 mt-3">
                <Popover.CloseTrigger
                  onclick={() => stats.reset()}
                  class="btn btn-danger flex-1 justify-center font-mono text-[9px]"
                >
                  Confirmer
                </Popover.CloseTrigger>
                <Popover.CloseTrigger class="btn btn-secondary flex-1 justify-center font-mono text-[9px]">
                  Annuler
                </Popover.CloseTrigger>
              </div>
            </Popover.Content>
          </Popover.Positioner>
        </Portal>
      </Popover>
    </div>
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
</style>