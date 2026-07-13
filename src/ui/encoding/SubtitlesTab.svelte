<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import type {
    SubExtractFormat,
    SubExtractNaming,
    SubExtractPathMode,
  } from "$lib/stores/types";
  import { FolderOpen } from "@lucide/svelte";
</script>

<section class="tab">
  <header class="tab-header">
    <h2 class="tab-title">Sous-titres</h2>
    <p class="tab-desc">
      Configure l'extraction des pistes de sous-titres intégrées dans les fichiers source.
    </p>
  </header>

  <!-- Bouton d'extraction -->
  <div class="field">
    <span class="field-label">Bouton dans la barre de contrôle</span>
    <button
      type="button"
      class="toggle-row {encoder.showExtractButton ? 'toggle-row--on' : ''}"
      onclick={() => encoder.setShowExtractButton(!encoder.showExtractButton)}
    >
      <span class="toggle-dot"></span>
      <span class="toggle-row-label">
        {encoder.showExtractButton ? 'Bouton « Extraire » visible' : 'Bouton « Extraire » masqué'}
      </span>
    </button>
  </div>

  <!-- Format + Nommage côte à côte -->
  <div class="field-2col">
    <div class="field">
      <span class="field-label">Format</span>
      <div class="seg-group">
        {#each [["srt", "SRT"], ["ass", "ASS"]] as [val, lbl]}
          <button
            type="button"
            class="seg-btn {encoder.subExtractFormat === val ? 'seg-btn--active' : ''}"
            onclick={() => encoder.setSubExtractFormat(val as SubExtractFormat)}
          >{lbl}</button>
        {/each}
      </div>
    </div>

    <div class="field">
      <span class="field-label">Nommage</span>
      <div class="seg-group">
        {#each [["source", "Source"], ["cleaned", "Nettoyé"]] as [val, lbl]}
          <button
            type="button"
            class="seg-btn {encoder.subExtractNaming === val ? 'seg-btn--active' : ''}"
            onclick={() => encoder.setSubExtractNaming(val as SubExtractNaming)}
          >{lbl}</button>
        {/each}
      </div>
    </div>
  </div>

  <!-- Dossier de destination -->
  <div class="field">
    <span class="field-label">Dossier de destination</span>
    <div class="dest-group">
      {#each [
        { val: "source",    label: "Dossier source"  },
        { val: "downloads", label: "Téléchargements" },
        { val: "custom",    label: "Personnalisé…"   },
      ] as opt}
        <button
          type="button"
          class="dest-btn {encoder.subExtractPathMode === opt.val ? 'dest-btn--active' : ''}"
          onclick={() => encoder.setSubExtractPathMode(opt.val as SubExtractPathMode)}
        >{opt.label}</button>
      {/each}
    </div>

    {#if encoder.subExtractPathMode === "custom"}
      <div class="path-row">
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

<style>
  .tab {
    padding: 24px 28px;
    max-width: 560px;
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

  .field {
    margin-bottom: 18px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .field:last-child { margin-bottom: 0; }

  .field-label {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color-subtext);
  }

  .field-2col {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    margin-bottom: 18px;
  }
  .field-2col .field { margin-bottom: 0; }

  /* Toggle on/off */
  .toggle-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 11px 14px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    font-size: 12px;
    font-weight: 500;
    color: var(--color-subtext);
    transition: border-color 0.12s, background 0.12s, color 0.12s;
  }
  .toggle-row:hover { border-color: var(--color-subtext2); }
  .toggle-row--on {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 7%, var(--color-surface));
    color: var(--color-accent);
  }
  .toggle-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-border);
    flex-shrink: 0;
    transition: background 0.12s;
  }
  .toggle-row--on .toggle-dot { background: var(--color-accent); }
  .toggle-row-label { flex: 1; }

  /* Segments */
  .seg-group {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 5px;
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
    text-align: center;
    transition: border-color 0.12s, background 0.12s, color 0.12s;
  }
  .seg-btn:hover { border-color: var(--color-subtext2); color: var(--color-text); }
  .seg-btn--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 10%, var(--color-surface));
    color: var(--color-accent);
  }

  /* Destination */
  .dest-group {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 5px;
  }
  .dest-btn {
    padding: 9px 6px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    font-size: 11px;
    color: var(--color-subtext);
    cursor: pointer;
    transition: border-color 0.12s, background 0.12s, color 0.12s;
  }
  .dest-btn:hover { border-color: var(--color-subtext2); color: var(--color-text); }
  .dest-btn--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-surface));
    color: var(--color-accent);
  }

  /* Chemin personnalisé */
  .path-row {
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
    transition: border-color 0.12s;
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
</style>
