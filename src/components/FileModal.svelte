<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { encoder, buildOutputName, computeTag, formatDuration } from "$lib/stores/encoder.svelte";
  import type { AppFile, CleanedName } from "$lib/stores/encoder.svelte";
  import { formatSize } from "$lib/utils";
  import { Loader, RefreshCw, CircleCheck, X, FileText, Pencil } from '@lucide/svelte';

  type Props = {
    file:     AppFile;
    onclose:  () => void;
    onrename: (newName: string) => void;
    startTab?: "info" | "rename";
  };

  let { file, onclose, onrename, startTab = "info" }: Props = $props();

  let activeTab = $state<"info" | "rename">("info");
  $effect(() => { activeTab = startTab; });

  let initFilename  = $derived(file.path.split(/[\\/]/).pop() ?? file.filename);
  let initOutputExt = $derived(file.output_ext);
  let audio_langs   = $derived(file.audio_langs);
  let sub_langs     = $derived(file.sub_langs);

  let initAudioLangs = $derived(audio_langs.filter((l: string) => encoder.selAudio.has(l)));
  let initSubLangs   = $derived(sub_langs.filter((l: string) => encoder.selSubs.has(l)));

  let editValue = $state('');
  let loading   = $state(true);
  let cleaned   = $state<CleanedName | null>(null);
  let canRename = $derived(!encoder.encoding && file.status !== 'analysing');

  let suggestedName = $derived.by(() => {
    if (!cleaned) return "";
    const tag = computeTag(audio_langs, sub_langs, encoder.selAudio, encoder.selSubs);
    return buildOutputName(cleaned, tag, encoder.seasonEpisodeFormat, "AAC", encoder.tagOrder, encoder.team);
  });

  $effect(() => { editValue = file.output_name; });

  $effect(() => {
    loading = true;
    invoke<CleanedName>("clean_filename", {
      raw:        initFilename,
      audioLangs: initAudioLangs,
      subLangs:   initSubLangs,
    })
      .then((r) => {
        cleaned = r;
        const tag = computeTag(audio_langs, sub_langs, encoder.selAudio, encoder.selSubs);
        if (r) editValue = buildOutputName(r, tag, encoder.seasonEpisodeFormat, "AAC", encoder.tagOrder, encoder.team);
      })
      .catch(() => {})
      .finally(() => { loading = false; });
  });

  function confirm() {
    const v = editValue.trim();
    if (v) onrename(v);
    onclose();
  }

  function applyTag(tag: string) { editValue = tag; }

  function focusAndSelect(node: HTMLInputElement) {
    node.focus();
    node.select();
  }

  function getStatusLabel(f: AppFile): string {
    if (f.status === "analysing") return "Analyse";
    if (f.status === "encoding")  return "En cours";
    if (f.status === "queued")    return "En file";
    if (f.status === "ready")     return "Prêt";
    if (f.status === "done")      return "Terminé";
    if (f.status === "error")     return "Erreur";
    return "—";
  }
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === "Escape") onclose();
    if (e.key === "Enter" && activeTab === "rename") confirm();
  }}
/>

<div
  class="modal-backdrop"
  role="dialog"
  aria-modal="true"
  aria-label="Informations / Renommer"
  tabindex="-1"
  onclick={(e) => { if (e.target === e.currentTarget) onclose(); }}
  onkeydown={(e) => { if (e.key === 'Escape') onclose(); }}
>
  <div class="modal-box" style="width: 520px; max-width: 95vw;">

    <!-- Header -->
    <div class="modal-header">
      <div class="flex items-center gap-1">
        <button
          class="tab-btn {activeTab === 'info' ? 'tab-active' : ''}"
          onclick={() => activeTab = 'info'}
        >
          <FileText class="w-3.5 h-3.5" />
          Infos
        </button>
        <button
          class="tab-btn {activeTab === 'rename' ? 'tab-active' : ''} {!canRename ? 'opacity-40 cursor-not-allowed' : ''}"
          onclick={() => { if (canRename) activeTab = 'rename'; }}
          disabled={!canRename}
          title={!canRename ? "Impossible pendant l'encodage" : undefined}
        >
          <Pencil class="w-3.5 h-3.5" />
          Renommer
        </button>
      </div>
      <button onclick={onclose} class="modal-close-btn" aria-label="Fermer">
        <X class="w-4 h-4" />
      </button>
    </div>

    <!-- ── Onglet INFOS ── -->
    {#if activeTab === 'info'}
      <div class="modal-body space-y-3">
        <div>
          <div class="section-label mb-1">Fichier source</div>
          <div class="px-3 py-2 rounded-[var(--radius-sm)] font-mono text-[11px] break-all"
               style="background: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-text);">
            {file.filename}
          </div>
        </div>

        <div>
          <div class="section-label mb-1">Fichier de sortie</div>
          <div class="px-3 py-2 rounded-[var(--radius-sm)] font-mono text-[11px] break-all"
               style="background: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-success);">
            {file.output_name}{file.output_ext}
          </div>
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div class="p-3 rounded-[var(--radius-sm)]" style="background: var(--color-surface); border: 1px solid var(--color-border);">
            <div class="section-label mb-1">Taille</div>
            <div class="font-mono text-[13px]" style="color: var(--color-text);">{formatSize(file.size_mb)}</div>
          </div>
          <div class="p-3 rounded-[var(--radius-sm)]" style="background: var(--color-surface); border: 1px solid var(--color-border);">
            <div class="section-label mb-1">Durée</div>
            <div class="font-mono text-[13px]" style="color: var(--color-text);">{formatDuration(file.duration_secs)}</div>
          </div>
          <div class="p-3 rounded-[var(--radius-sm)]" style="background: var(--color-surface); border: 1px solid var(--color-border);">
            <div class="section-label mb-1">FPS</div>
            <div class="font-mono text-[13px]" style="color: var(--color-text);">{file.fps?.toFixed(2) || "—"} fps</div>
          </div>
          <div class="p-3 rounded-[var(--radius-sm)]" style="background: var(--color-surface); border: 1px solid var(--color-border);">
            <div class="section-label mb-1">Statut</div>
            <div class="font-mono text-[13px]" style="color: var(--color-text);">{getStatusLabel(file)}</div>
          </div>
        </div>

        {#if file.audio_langs?.length > 0}
          <div>
            <div class="section-label mb-1.5">Pistes audio</div>
            <div class="flex flex-wrap gap-1">
              {#each file.audio_langs as lang}
                <span class="font-mono text-[10px] px-2 py-0.5 rounded-[var(--radius-full)]"
                      style="background: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-subtext);">
                  {lang.toUpperCase()}
                </span>
              {/each}
            </div>
          </div>
        {/if}

        <div>
          <div class="section-label mb-1.5">Sous-titres</div>
          <div class="flex flex-wrap gap-1">
            {#if file.sub_langs?.length > 0}
              {#each file.sub_langs as lang}
                <span class="font-mono text-[10px] px-2 py-0.5 rounded-[var(--radius-full)]"
                      style="background: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-subtext);">
                  {lang.toUpperCase()}
                </span>
              {/each}
            {:else}
              <span class="text-[11px] italic" style="color: var(--color-subtext);">Aucun</span>
            {/if}
          </div>
        </div>

        {#if file.streams?.length > 0}
          <div>
            <div class="section-label mb-1.5">Flux</div>
            <div class="space-y-0.5 max-h-32 overflow-y-auto">
              {#each file.streams as s}
                <div class="font-mono text-[10px] py-1 flex gap-2"
                     style="border-bottom: 1px solid color-mix(in srgb, var(--color-border) 30%, transparent);">
                  <span style="color: var(--color-accent);">{s.codec_type?.toUpperCase()}</span>
                  <span style="color: var(--color-subtext);">{s.codec_name}</span>
                  {#if s.width && s.height}
                    <span style="color: var(--color-subtext2);">{s.width}×{s.height}</span>
                  {/if}
                  <span style="color: var(--color-subtext2);">[{s.language?.toUpperCase() || "und"}]</span>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>

      <div class="modal-footer">
        {#if canRename}
          <button
            onclick={() => activeTab = 'rename'}
            class="btn btn-secondary font-mono text-[11px] flex items-center gap-1.5"
          >
            <Pencil class="w-3.5 h-3.5" />
            Renommer
          </button>
        {/if}
        <button onclick={onclose} class="btn btn-primary font-mono text-[11px]">Fermer</button>
      </div>

    <!-- ── Onglet RENAME ── -->
    {:else}
      <div class="modal-body space-y-4">

        <!-- Source / Sortie empilés -->
        <div class="space-y-2">
          <div class="space-y-1">
            <div class="section-label">Source</div>
            <div
              class="font-mono text-[11px] truncate px-3 py-2 rounded-[var(--radius-sm)]"
              style="background: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-subtext);"
              title={initFilename}
            >
              {initFilename}
            </div>
          </div>
          <div class="space-y-1">
            <div class="section-label">Sortie</div>
            <div
              class="font-mono text-[11px] truncate px-3 py-2 rounded-[var(--radius-sm)]"
              style="background: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-success);"
              title="{editValue}{initOutputExt}"
            >
              <span>{editValue || "…"}</span><span style="color: var(--color-subtext);">{initOutputExt}</span>
            </div>
          </div>
        </div>

        <!-- Nouveau nom -->
        <div class="space-y-1.5">
          <div class="flex items-center justify-between">
            <div class="section-label">Nouveau nom</div>
            {#if suggestedName && editValue !== suggestedName}
              <button
                onclick={() => applyTag(suggestedName)}
                class="flex items-center gap-1 text-[10px] transition-colors"
                style="color: var(--color-accent);"
              >
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
            class="w-full px-3 py-2 text-[11px] rounded-[var(--radius-sm)]"
            style="background: var(--color-surface); border: 1px solid var(--color-accent); color: var(--color-text); font-family: 'Geist Mono', monospace; outline: none;"
          />
        </div>

        <!-- Suggestion -->
        {#if loading}
          <div class="flex items-center gap-2 text-[11px]" style="color: var(--color-subtext);">
            <Loader size="4" />
            Analyse du nom en cours…
          </div>
        {:else if suggestedName}
          <div class="flex items-center gap-2 flex-wrap">
            <span class="section-label whitespace-nowrap">Suggéré</span>
            <button
              onclick={() => applyTag(suggestedName)}
              class="inline-flex items-center gap-1 text-[11px] px-2.5 py-1 rounded-[var(--radius-full)] font-mono transition-colors truncate"
              style="border: 1px solid {editValue === suggestedName ? 'color-mix(in srgb, var(--color-success) 40%, transparent)' : 'var(--color-border)'}; background: {editValue === suggestedName ? 'color-mix(in srgb, var(--color-success) 12%, transparent)' : 'var(--color-surface)'}; color: {editValue === suggestedName ? 'var(--color-success)' : 'var(--color-subtext)'};"
              title="Appliquer le nom suggéré"
            >
              {#if editValue === suggestedName}
                <CircleCheck class="w-3 h-3 shrink-0" />
              {/if}
              <span class="truncate">{suggestedName}</span>
            </button>
          </div>
        {/if}

      </div>

      <div class="modal-footer">
        <button onclick={() => activeTab = 'info'} class="btn btn-secondary font-mono text-[11px]">Retour</button>
        <button
          onclick={confirm}
          disabled={!editValue.trim()}
          class="btn btn-primary font-mono text-[11px] flex items-center gap-1.5"
        >
          <CircleCheck class="w-3.5 h-3.5" />
          Renommer
        </button>
      </div>
    {/if}

  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 9999;
    background: rgba(0, 0, 0, 0.65);
    display: flex;
    align-items: center;
    justify-content: center;
    backdrop-filter: blur(2px);
  }
  .modal-box {
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    display: flex;
    flex-direction: column;
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.5);
    overflow: hidden;
    max-height: 85vh;
  }
  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-panel);
    flex-shrink: 0;
  }
  .modal-body {
    padding: 16px;
    overflow-y: auto;
    flex: 1;
  }
  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 16px;
    border-top: 1px solid var(--color-border);
    background: var(--color-panel);
    flex-shrink: 0;
  }
  .modal-close-btn {
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
  .modal-close-btn:hover {
    background: var(--color-panel2);
    border-color: var(--color-border);
    color: var(--color-text);
  }
  .tab-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-family: 'Geist Mono', monospace;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid transparent;
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    transition: background 0.1s, color 0.1s, border-color 0.1s;
  }
  .tab-btn:hover:not(:disabled) {
    background: var(--color-surface);
    color: var(--color-text);
    border-color: var(--color-border);
  }
  .tab-active {
    background: color-mix(in srgb, var(--color-accent) 15%, transparent);
    color: var(--color-accent);
    border-color: color-mix(in srgb, var(--color-accent) 35%, transparent);
  }
  .tab-active:hover {
    background: color-mix(in srgb, var(--color-accent) 20%, transparent) !important;
    color: var(--color-accent) !important;
  }
</style>