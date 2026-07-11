<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import type {
    SubExtractFormat,
    SubExtractNaming,
    SubExtractPathMode,
  } from "$lib/stores/types";
  import { FolderOpen } from "@lucide/svelte";
</script>

<section class="content-section">
  <header class="section-header">
    <div>
      <h2 class="section-title">Extraction sous-titres</h2>
      <p class="section-desc">
        Configure l'extraction des pistes de sous-titres intégrées dans les
        fichiers source.
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
      <span
        >{encoder.showExtractButton
          ? "Bouton « Extraire » visible"
          : "Bouton « Extraire » masqué"}</span
      >
    </button>
  </div>

  <div class="field-grid-2">
    <div class="field-block">
      <div class="field-label">Format</div>
      <div class="seg-pair">
        {#each [["srt", "SRT"], ["ass", "ASS"]] as [val, lbl]}
          <button
            type="button"
            class="seg-btn {encoder.subExtractFormat === val
              ? 'seg-btn--active'
              : ''}"
            onclick={() => encoder.setSubExtractFormat(val as SubExtractFormat)}
          >
            {lbl}
          </button>
        {/each}
      </div>
    </div>

    <div class="field-block">
      <div class="field-label">Nommage</div>
      <div class="seg-pair">
        {#each [["source", "Source"], ["cleaned", "Nettoyé"]] as [val, lbl]}
          <button
            type="button"
            class="seg-btn {encoder.subExtractNaming === val
              ? 'seg-btn--active'
              : ''}"
            onclick={() => encoder.setSubExtractNaming(val as SubExtractNaming)}
          >
            {lbl}
          </button>
        {/each}
      </div>
    </div>
  </div>

  <div class="field-block">
    <div class="field-label">Dossier de destination</div>
    <div class="dest-row">
      {#each [{ val: "source", label: "Dossier source" }, { val: "downloads", label: "Téléchargements" }, { val: "custom", label: "Personnalisé…" }] as opt}
        <button
          type="button"
          class="dest-btn {encoder.subExtractPathMode === opt.val
            ? 'dest-btn--active'
            : ''}"
          onclick={() =>
            encoder.setSubExtractPathMode(opt.val as SubExtractPathMode)}
          >{opt.label}</button
        >
      {/each}
    </div>

    {#if encoder.subExtractPathMode === "custom"}
      <div class="custom-path-row">
        <input
          type="text"
          value={encoder.subExtractCustomPath}
          oninput={(e) =>
            encoder.setSubExtractCustomPath(
              (e.target as HTMLInputElement).value,
            )}
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

  .field-block {
    margin-bottom: 24px;
  }
  .field-block:last-child {
    margin-bottom: 0;
  }

  .field-label {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color-subtext);
    margin-bottom: 10px;
  }

  .field-grid-2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 20px;
    margin-bottom: 24px;
  }

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
    transition:
      border-color 0.15s,
      background 0.15s,
      color 0.15s;
  }
  .toggle-full--on {
    border-color: var(--color-accent);
    background: color-mix(
      in srgb,
      var(--color-accent) 7%,
      var(--color-surface)
    );
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
  .toggle-full--on .tf-dot {
    background: var(--color-accent);
  }

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
    transition:
      border-color 0.15s,
      background 0.15s,
      color 0.15s;
    text-align: center;
  }
  .seg-btn:hover {
    border-color: var(--color-subtext2);
    color: var(--color-text);
  }
  .seg-btn--active {
    border-color: var(--color-accent);
    background: color-mix(
      in srgb,
      var(--color-accent) 10%,
      var(--color-surface)
    );
    color: var(--color-accent);
  }

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
    transition:
      border-color 0.15s,
      background 0.15s,
      color 0.15s;
  }
  .dest-btn:hover {
    border-color: var(--color-subtext2);
    color: var(--color-text);
  }
  .dest-btn--active {
    border-color: var(--color-accent);
    background: color-mix(
      in srgb,
      var(--color-accent) 8%,
      var(--color-surface)
    );
    color: var(--color-accent);
  }

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
  .path-input:focus {
    border-color: var(--color-accent);
  }
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
    transition:
      border-color 0.1s,
      color 0.1s;
  }
  .browse-btn:hover {
    border-color: var(--color-subtext2);
    color: var(--color-text);
  }
</style>
