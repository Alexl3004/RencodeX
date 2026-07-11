<script lang="ts">
  import { FolderInput, AlertTriangle, CircleCheck } from "@lucide/svelte";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";

  type FfmpegCheckResult = {
    path: string;
    exists: boolean;
    executable: boolean;
    version: string | null;
  };

  let {
    ffmpegPath = $bindable(),
    ffmpegCheck,
    ffmpegChecking,
    onInput,
    onBrowse,
  }: {
    ffmpegPath: string;
    ffmpegCheck: FfmpegCheckResult | null;
    ffmpegChecking: boolean;
    onInput: (path: string) => void;
    onBrowse: (path: string) => void;
  } = $props();
</script>

<section class="content-section">
  <header class="section-header">
    <div>
      <h2 class="section-title">FFmpeg</h2>
      <p class="section-desc">
        Chemin vers le binaire ffmpeg utilisé pour l'encodage.
      </p>
    </div>
  </header>

  <div>
    <label for="ffmpeg-path" class="field-label">Chemin vers ffmpeg.exe</label>

    <div class="flex gap-2 mb-2">
      <input
        id="ffmpeg-path"
        type="text"
        bind:value={ffmpegPath}
        oninput={() => onInput(ffmpegPath)}
        placeholder="C:\Outil\ffmpeg\bin\ffmpeg.exe"
        class="field-input flex-1 px-3 py-2"
        class:field-input--error={ffmpegCheck &&
          (!ffmpegCheck.exists || !ffmpegCheck.executable)}
        class:field-input--ok={ffmpegCheck?.exists && ffmpegCheck?.executable}
        aria-describedby="ffmpeg-status"
      />
      <button
        type="button"
        onclick={async () => {
          const selected = await openDialog({
            multiple: false,
            filters: [{ name: "Exécutable", extensions: ["exe", "*"] }],
          });
          if (typeof selected === "string" && selected) {
            onBrowse(selected);
          }
        }}
        class="btn btn-secondary font-mono text-[11px] px-3"
        title="Parcourir"
        aria-label="Choisir le binaire ffmpeg"
      >
        <FolderInput class="w-3.5 h-3.5" />
      </button>
    </div>

    <!-- Badge de statut -->
    <div id="ffmpeg-status" class="ffmpeg-status" aria-live="polite">
      {#if ffmpegChecking}
        <span class="ffmpeg-status--checking">
          <span class="spinner spinner--xs" aria-hidden="true"></span>
          Vérification…
        </span>
      {:else if ffmpegCheck === null}
        <!-- pas encore vérifié -->
      {:else if !ffmpegCheck.exists}
        <span class="ffmpeg-status--error">
          <AlertTriangle class="w-3.5 h-3.5 shrink-0" aria-hidden="true" />
          Binaire introuvable — l'encodage échouera
        </span>
      {:else if !ffmpegCheck.executable}
        <span class="ffmpeg-status--error">
          <AlertTriangle class="w-3.5 h-3.5 shrink-0" aria-hidden="true" />
          Fichier présent mais non exécutable
        </span>
      {:else}
        <span class="ffmpeg-status--ok">
          <CircleCheck class="w-3.5 h-3.5 shrink-0" aria-hidden="true" />
          ffmpeg {ffmpegCheck.version ?? ""} — OK
        </span>
      {/if}
    </div>
  </div>
</section>

<style>
  .content-section {
    padding: 28px 32px;
    display: flex;
    flex-direction: column;
    gap: 20px;
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

  .field-label {
    display: block;
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
    margin-bottom: 4px;
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
  .field-input--error {
    border-color: #f87171 !important;
  }
  .field-input--ok {
    border-color: #4ade80 !important;
  }

  .ffmpeg-status {
    min-height: 20px;
    font-family: "Geist Mono", monospace;
    font-size: 11px;
  }
  .ffmpeg-status--checking,
  .ffmpeg-status--ok,
  .ffmpeg-status--error {
    display: inline-flex;
    align-items: center;
    gap: 5px;
  }
  .ffmpeg-status--checking {
    color: var(--color-subtext);
  }
  .ffmpeg-status--ok {
    color: #4ade80;
  }
  .ffmpeg-status--error {
    color: #f87171;
  }

  .spinner--xs {
    display: inline-block;
    width: 10px;
    height: 10px;
    border: 1.5px solid color-mix(in srgb, currentColor 30%, transparent);
    border-top-color: currentColor;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
