<script lang="ts">
  import { FolderInput, Check, X } from "@lucide/svelte";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";

  let {
    presets,
    onAdd,
    onRemove,
  }: {
    presets: string[];
    onAdd: (path: string) => Promise<void>;
    onRemove: (path: string) => Promise<void>;
  } = $props();

  let newPreset = $state("");

  async function addPreset() {
    const p = newPreset.trim();
    if (!p || presets.includes(p)) return;
    newPreset = "";
    await onAdd(p);
  }

  async function browsePreset() {
    const dir = await openDialog({ directory: true });
    if (dir && typeof dir === "string") {
      newPreset = dir;
    }
  }

  async function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      await addPreset();
    }
  }
</script>

<section class="content-section">
  <header class="section-header">
    <div>
      <h2 class="section-title">Chemins de sortie</h2>
      <p class="section-desc">
        Ces chemins apparaîtront dans le menu de la DropZone, en plus de
        l'historique récent.
      </p>
    </div>
  </header>

  <!-- Champ d'ajout -->
  <div class="flex gap-2 mb-2">
    <input
      type="text"
      bind:value={newPreset}
      onkeydown={handleKeydown}
      placeholder="D:\Encodage\Sorties"
      class="field-input flex-1 px-3 py-2"
      aria-label="Nouveau chemin prédéfini"
    />
    <button
      type="button"
      onclick={browsePreset}
      class="btn btn-secondary font-mono text-[11px] px-3"
      aria-label="Parcourir"
      title="Choisir un dossier"
    >
      <FolderInput class="w-3.5 h-3.5" />
    </button>
    <button
      type="button"
      onclick={addPreset}
      disabled={!newPreset.trim()}
      class="btn btn-primary font-mono text-[11px] px-3"
      aria-label="Ajouter le chemin"
      title="Ajouter"
    >
      <Check class="w-3.5 h-3.5" />
    </button>
  </div>

  <!-- Liste des presets -->
  {#if presets.length > 0}
    <div class="space-y-1" role="list" aria-label="Chemins prédéfinis">
      {#each presets as preset}
        <div
          class="flex items-center gap-2 px-3 py-2 rounded-[var(--radius-sm)]"
          style="background: var(--color-surface); border: 1px solid var(--color-border);"
          role="listitem"
        >
          <span
            class="font-mono text-[10px] flex-1 min-w-0 truncate"
            style="color: var(--color-text); direction: rtl; text-align: left;"
            title={preset}>{preset}</span
          >
          <button
            type="button"
            onclick={() => onRemove(preset)}
            class="icon-btn-inline shrink-0"
            aria-label="Supprimer ce chemin"
          >
            <X class="w-3 h-3" />
          </button>
        </div>
      {/each}
    </div>
  {:else}
    <p
      class="font-mono text-[10px] text-center py-2"
      style="color: var(--color-subtext); opacity: 0.5;"
    >
      Aucun chemin prédéfini
    </p>
  {/if}
</section>

<style>
  .content-section {
    padding: 28px 32px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .section-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 4px;
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
    max-width: 380px;
    margin: 0;
  }

  .field-input {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    color: var(--color-text);
    border-radius: var(--radius-sm);
    outline: none;
    transition: border-color 0.12s;
  }
  .field-input:focus {
    border-color: var(--color-accent);
  }

  .icon-btn-inline {
    color: var(--color-subtext);
    background: transparent;
    border: none;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: color 0.1s;
  }
  .icon-btn-inline:hover {
    color: var(--color-text);
  }
</style>
