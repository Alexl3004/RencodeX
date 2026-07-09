<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { encoder } from "$lib/stores/encoder.svelte";
  import { buildOutputName, computeTag } from "$lib/stores/naming";
  import type { AppFile, CleanedName } from "$lib/stores/encoder.svelte";
  import { Loader2, RefreshCw, CircleCheck, X, Pencil } from "@lucide/svelte";

  type Props = {
    file: AppFile;
    position: { x: number; y: number };
    onclose: () => void;
    onrename: (newName: string) => void;
  };

  let { file, position, onclose, onrename }: Props = $props();

  let initFilename = $derived(file.path.split(/[\\/]/).pop() ?? file.filename);
  let audio_langs = $derived(file.audio_langs);
  let sub_langs = $derived(file.sub_langs);
  let initAudioLangs = $derived(
    audio_langs.filter((l: string) => encoder.selAudio.has(l)),
  );
  let initSubLangs = $derived(
    sub_langs.filter((l: string) => encoder.selSubs.has(l)),
  );

  let editValue = $state("");
  let loading = $state(true);
  let cleaned = $state<CleanedName | null>(null);

  let suggestedName = $derived.by(() => {
    if (!cleaned) return "";
    const tag = computeTag(
      audio_langs,
      sub_langs,
      encoder.selAudio,
      encoder.selSubs,
    );
    return buildOutputName(
      cleaned,
      tag,
      encoder.seasonEpisodeFormat,
      "AAC",
      encoder.tagOrder,
      encoder.team,
      {
        disabledTags: encoder.disabledTags,
        resolutionCase: encoder.resolutionCase,
        titleCase: encoder.titleCase,
        codecFormat: encoder.codecFormat,
        sourceCase: encoder.sourceCase,
      },
    );
  });

  $effect(() => {
    editValue = file.output_name;
  });

  $effect(() => {
    loading = true;
    invoke<CleanedName>("clean_filename", {
      raw: initFilename,
      audioLangs: initAudioLangs,
      subLangs: initSubLangs,
    })
      .then((r) => {
        cleaned = r;
        const tag = computeTag(
          audio_langs,
          sub_langs,
          encoder.selAudio,
          encoder.selSubs,
        );
        if (r)
          editValue = buildOutputName(
            r,
            tag,
            encoder.seasonEpisodeFormat,
            "AAC",
            encoder.tagOrder,
            encoder.team,
            {
              disabledTags: encoder.disabledTags,
              resolutionCase: encoder.resolutionCase,
              titleCase: encoder.titleCase,
              codecFormat: encoder.codecFormat,
              sourceCase: encoder.sourceCase,
            },
          );
      })
      .catch(() => {})
      .finally(() => {
        loading = false;
      });
  });

  function confirm() {
    const v = editValue.trim();
    if (v) onrename(v);
    onclose();
  }

  function focusAndSelect(node: HTMLInputElement) {
    node.focus();
    node.select();
  }

  // ── Positionnement automatique ──
  let popoverEl = $state<HTMLDivElement | null>(null);

  let popoverStyle = $derived.by(() => {
    if (!popoverEl) return { left: position.x, top: position.y + 8 };
    const rect = popoverEl.getBoundingClientRect();
    const viewWidth = window.innerWidth;
    const viewHeight = window.innerHeight;

    let left = position.x;
    let top = position.y + 8;

    if (left + rect.width > viewWidth - 12) {
      left = viewWidth - rect.width - 12;
    }
    if (left < 12) left = 12;

    if (top + rect.height > viewHeight - 12) {
      top = position.y - rect.height - 8;
      if (top < 12) top = 12;
    }

    return { left, top };
  });
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === "Escape") onclose();
    if (e.key === "Enter") confirm();
  }}
/>

<!-- Overlay transparent pour fermer au clic en dehors -->
<div
  class="backdrop"
  role="presentation"
  onclick={onclose}
  oncontextmenu={(e) => {
    e.preventDefault();
    onclose();
  }}
></div>

<!-- Popover -->
<div
  bind:this={popoverEl}
  class="popover"
  style="left:{popoverStyle.left}px; top:{popoverStyle.top}px;"
  role="dialog"
  aria-modal="true"
  aria-label="Renommer le fichier"
>
  <!-- ── Header ── -->
  <div class="popover-header">
    <div class="header-left">
      <div class="file-icon">
        <Pencil class="w-3.5 h-3.5" style="color: var(--color-accent);" />
      </div>
      <div class="header-title">
        <span class="header-label">Renommer</span>
        <span class="header-name" title={initFilename}>{initFilename}</span>
      </div>
    </div>
    <button onclick={onclose} class="close-btn" aria-label="Fermer">
      <X class="w-4 h-4" />
    </button>
  </div>

  <!-- ── Body ── -->
  <div class="popover-body">
    <!-- Nom actuel -->
    <div class="current-name-block">
      <span class="field-label">Nom actuel</span>
      <div class="current-name">{file.output_name}{file.output_ext}</div>
    </div>

    <!-- Nouveau nom -->
    <div class="section">
      <div class="input-header">
        <span class="field-label">Nouveau nom</span>
        {#if suggestedName && editValue !== suggestedName}
          <button onclick={() => (editValue = suggestedName)} class="reset-btn">
            <RefreshCw class="w-3 h-3" />
            Réinitialiser
          </button>
        {/if}
      </div>
      <input
        type="text"
        bind:value={editValue}
        placeholder="Nom du fichier de sortie…"
        use:focusAndSelect
        class="name-input"
      />
    </div>

    <!-- Nom suggéré -->
    <div class="section">
      <span class="field-label">Nom suggéré</span>
      {#if loading}
        <div class="loading-row">
          <Loader2
            class="w-3.5 h-3.5 animate-spin"
            style="color: var(--color-subtext);"
          />
          <span>Analyse en cours…</span>
        </div>
      {:else if suggestedName}
        <button
          onclick={() => (editValue = suggestedName)}
          class="suggestion-btn {editValue === suggestedName
            ? 'suggestion-btn--applied'
            : ''}"
        >
          {#if editValue === suggestedName}
            <CircleCheck class="w-3.5 h-3.5 shrink-0" />
          {/if}
          <span class="truncate">{suggestedName}</span>
        </button>
      {:else}
        <span class="empty-label">Aucune suggestion disponible</span>
      {/if}
    </div>
  </div>

  <!-- ── Footer ── -->
  <div class="popover-footer">
    <button onclick={onclose} class="footer-btn footer-btn--ghost"
      >Annuler</button
    >
    <button
      onclick={confirm}
      disabled={!editValue.trim()}
      class="footer-btn footer-btn--primary"
    >
      <CircleCheck class="w-3.5 h-3.5" />
      Renommer
    </button>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 9998;
    background: transparent;
    /* capture les clics sans fond visible */
  }

  .popover {
    position: fixed;
    z-index: 9999;
    width: 480px;
    max-width: 92vw;
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    max-height: 80vh;
  }

  /* Header */
  .popover-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 14px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
    background: var(--color-surface);
  }
  .header-left {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }
  .file-icon {
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--color-accent) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-accent) 25%, transparent);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  .header-title {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }
  .header-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--color-accent);
  }
  .header-name {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 300px;
  }
  .close-btn {
    width: 26px;
    height: 26px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-xs);
    border: 1px solid transparent;
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    flex-shrink: 0;
    transition:
      background 0.1s,
      color 0.1s,
      border-color 0.1s;
  }
  .close-btn:hover {
    background: var(--color-panel2);
    border-color: var(--color-border);
    color: var(--color-text);
  }

  /* Body */
  .popover-body {
    flex: 1;
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    overflow-y: auto;
  }

  .current-name-block {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .current-name {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    color: var(--color-subtext);
    word-break: break-all;
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
  }

  .field-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--color-subtext);
  }
  .section {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .input-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .name-input {
    width: 100%;
    padding: 7px 10px;
    font-family: "Geist Mono", monospace;
    font-size: 12px;
    color: var(--color-text);
    background: var(--color-surface);
    border: 1px solid var(--color-accent);
    border-radius: var(--radius-sm);
    outline: none;
    box-sizing: border-box;
    transition: box-shadow 0.15s;
  }
  .name-input:focus {
    box-shadow: 0 0 0 3px
      color-mix(in srgb, var(--color-accent) 18%, transparent);
  }
  .reset-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-accent);
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 0;
    opacity: 0.8;
    transition: opacity 0.1s;
  }
  .reset-btn:hover {
    opacity: 1;
  }

  .loading-row {
    display: flex;
    align-items: center;
    gap: 6px;
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
  }
  .suggestion-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-subtext);
    cursor: pointer;
    width: 100%;
    overflow: hidden;
    transition:
      background 0.12s,
      border-color 0.12s,
      color 0.12s;
  }
  .suggestion-btn:hover {
    background: var(--color-panel);
    border-color: var(--color-subtext2);
    color: var(--color-text);
  }
  .suggestion-btn--applied {
    background: color-mix(in srgb, var(--color-success) 10%, transparent);
    border-color: color-mix(in srgb, var(--color-success) 35%, transparent);
    color: var(--color-success);
  }
  .suggestion-btn--applied:hover {
    background: color-mix(in srgb, var(--color-success) 15%, transparent);
    color: var(--color-success);
  }
  .empty-label {
    font-size: 10px;
    font-style: italic;
    color: var(--color-subtext);
  }

  /* Footer */
  .popover-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    padding: 10px 16px;
    border-top: 1px solid var(--color-border);
    flex-shrink: 0;
    background: var(--color-surface);
  }
  .footer-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 14px;
    border-radius: var(--radius-sm);
    border: 1px solid transparent;
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
    transition:
      background 0.12s,
      border-color 0.12s,
      color 0.12s,
      opacity 0.12s;
  }
  .footer-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }
  .footer-btn--primary {
    background: var(--color-accent);
    border-color: var(--color-accent);
    color: #fff;
  }
  .footer-btn--primary:hover:not(:disabled) {
    background: color-mix(in srgb, var(--color-accent) 85%, #000);
  }
  .footer-btn--ghost {
    background: transparent;
    border-color: var(--color-border);
    color: var(--color-subtext);
  }
  .footer-btn--ghost:hover:not(:disabled) {
    background: var(--color-surface);
    border-color: var(--color-subtext2);
    color: var(--color-text);
  }
</style>
