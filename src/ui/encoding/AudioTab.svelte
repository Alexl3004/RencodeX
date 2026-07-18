<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import type { AudioCodec, AudioCodecRules, AudioPreset } from "$lib/stores/types";
  import { Check, Pencil, X, Plus } from "@lucide/svelte";
  import { SegmentedControl } from "@skeletonlabs/skeleton-svelte";

  let audioCodecRules = $derived(encoder.audioCodecRules);
  let audioBitrate    = $derived(encoder.audioBitrate);
  let container       = $derived(encoder.container);

  const KNOWN_CODECS: { id: string; label: string }[] = [
    { id: "aac",         label: "AAC"     },
    { id: "ac3",         label: "AC3"     },
    { id: "eac3",        label: "E-AC3"   },
    { id: "dts",         label: "DTS"     },
    { id: "truehd",      label: "TrueHD"  },
    { id: "flac",        label: "FLAC"    },
    { id: "mp3",         label: "MP3"     },
    { id: "opus",        label: "OPUS"    },
    { id: "vorbis",      label: "Vorbis"  },
    { id: "pcm_s16le",   label: "PCM 16"  },
    { id: "pcm_s24le",   label: "PCM 24"  },
    { id: "__default__", label: "Autre"   },
  ];

  const TARGET_CODECS: { id: AudioCodec; label: string; mkvOnly: boolean }[] = [
    { id: "aac",  label: "AAC",  mkvOnly: false },
    { id: "ac3",  label: "AC3",  mkvOnly: false },
    { id: "opus", label: "OPUS", mkvOnly: true  },
  ];

  const BITRATE_HINTS: Record<number, string> = {
    128: "Standard",
    192: "Recommandé",
    256: "Haute qualité",
    320: "Maximum",
  };

  let presentSrcCodecs = $derived(
    new Set(
      encoder.files
        .flatMap((f: { streams: { codec_type: string; codec_name: string }[] }) => f.streams)
        .filter((s: { codec_type: string }) => s.codec_type === "audio")
        .map((s: { codec_name: string }) => s.codec_name.toLowerCase()),
    ),
  );

  let displayedCodecs = $derived([
    ...KNOWN_CODECS.filter(c => c.id !== "__default__" && presentSrcCodecs.has(c.id)),
    ...KNOWN_CODECS.filter(c => c.id !== "__default__" && !presentSrcCodecs.has(c.id)),
    KNOWN_CODECS.find(c => c.id === "__default__")!,
  ]);

  function getRule(srcId: string) {
    return (
      audioCodecRules[srcId] ??
      audioCodecRules["__default__"] ??
      { action: "reencode" as const, targetCodec: "aac" as AudioCodec }
    );
  }

  function setAction(srcId: string, action: "copy" | "reencode") {
    encoder.setAudioCodecRule(srcId, { ...getRule(srcId), action });
  }

  function setTarget(srcId: string, targetCodec: AudioCodec) {
    encoder.setAudioCodecRule(srcId, { ...getRule(srcId), action: "reencode", targetCodec });
  }

  // ─── Préréglages ────────────────────────────────────────────────────────────
  let allAudioPresets = $derived(encoder.allAudioPresets());

  let activeAudioPresetId = $derived(
    allAudioPresets.find((p: AudioPreset) => rulesMatch(p.rules, audioCodecRules))?.id ?? null,
  );

  function rulesMatch(a: AudioCodecRules, b: AudioCodecRules): boolean {
    const keysA = Object.keys(a);
    const keysB = Object.keys(b);
    if (keysA.length !== keysB.length) return false;
    return keysA.every(k => b[k] && a[k].action === b[k].action && a[k].targetCodec === b[k].targetCodec);
  }

  let showSaveInput  = $state(false);
  let newPresetLabel = $state("");
  let editingId      = $state<string | null>(null);
  let editingLabel   = $state("");

  function confirmSave() {
    const label = newPresetLabel.trim();
    if (!label) { showSaveInput = false; return; }
    encoder.saveAudioPreset(label);
    showSaveInput = false;
    newPresetLabel = "";
  }

  function confirmRename() {
    if (editingId && editingLabel.trim())
      encoder.renameAudioPreset(editingId, editingLabel.trim());
    editingId = null;
    editingLabel = "";
  }
</script>

<section class="tab">

  <!-- ── Header ─────────────────────────────────────────────────────────────── -->
  <header class="tab-header">
    <h2 class="tab-title">Audio</h2>
    <p class="tab-desc">Action par codec source — copie ou réencodage.</p>
  </header>

  <!-- ── Préréglages ────────────────────────────────────────────────────────── -->
  <div class="section">
    <span class="section-label">Préréglages</span>
    <div class="presets-row">
      {#each allAudioPresets as p (p.id)}
        <div class="preset-wrap">
          {#if editingId === p.id}
            <input
              class="preset-rename-input"
              bind:value={editingLabel}
              onkeydown={(e) => { if (e.key === "Enter") confirmRename(); if (e.key === "Escape") editingId = null; }}
              onblur={confirmRename}
            />
          {:else}
            <button
              type="button"
              class="preset-btn"
              class:preset-btn--active={activeAudioPresetId === p.id}
              onclick={() => encoder.applyAudioPreset(p.id)}
              title={p.builtin ? "Préréglage intégré" : "Préréglage personnalisé"}
            >
              {#if activeAudioPresetId === p.id}
                <Check class="w-3 h-3" />
              {/if}
              {p.label}
              {#if !p.builtin}<span class="custom-dot"></span>{/if}
            </button>
          {/if}
          {#if !p.builtin && editingId !== p.id}
            <button class="preset-icon-btn" onclick={() => { editingId = p.id; editingLabel = p.label; }} title="Renommer">
              <Pencil class="w-2.5 h-2.5" />
            </button>
            <button class="preset-icon-btn preset-icon-btn--delete" onclick={() => encoder.deleteAudioPreset(p.id)} title="Supprimer">
              <X class="w-2.5 h-2.5" />
            </button>
          {/if}
        </div>
      {/each}

      {#if showSaveInput}
        <div class="save-form">
          <input
            class="save-input"
            placeholder="Nom…"
            bind:value={newPresetLabel}
            onkeydown={(e) => { if (e.key === "Enter") confirmSave(); if (e.key === "Escape") { showSaveInput = false; newPresetLabel = ""; } }}
          />
          <button class="save-confirm" onclick={confirmSave}>OK</button>
          <button class="save-cancel" onclick={() => { showSaveInput = false; newPresetLabel = ""; }}>
            <X class="w-2.5 h-2.5" />
          </button>
        </div>
      {:else}
        <button class="preset-save-btn" onclick={() => { newPresetLabel = ""; showSaveInput = true; }}>
          <Plus class="w-3 h-3" />
          Sauvegarder
        </button>
      {/if}
    </div>
  </div>

  <!-- ── Table règles ───────────────────────────────────────────────────────── -->
  <div class="section section--table">
    <div class="table-head">
      <span class="th th--src">Source</span>
      <span class="th th--action">Action</span>
      <span class="th th--target">Cible</span>
    </div>

    <div class="table-body">
      {#each displayedCodecs as src}
        {@const rule    = getRule(src.id)}
        {@const present = src.id !== "__default__" && presentSrcCodecs.has(src.id)}
        {@const isCopy  = rule.action === "copy"}
        <div class="rule-row" class:rule-row--present={present}>

          <!-- Source -->
          <div class="td td--src">
            <span class="src-badge" class:src-badge--present={present}>{src.label}</span>
            {#if present}<span class="presence-dot" title="Présent dans les fichiers"></span>{/if}
          </div>

          <!-- Action -->
          <div class="td td--action">
            <SegmentedControl
              value={rule.action}
              onValueChange={(d) => setAction(src.id, d.value as "copy" | "reencode")}
              class="action-segment"
            >
              <SegmentedControl.Control>
                <SegmentedControl.Indicator />
                <SegmentedControl.Item value="copy">
                  <SegmentedControl.ItemText>Copier</SegmentedControl.ItemText>
                  <SegmentedControl.ItemHiddenInput />
                </SegmentedControl.Item>
                <SegmentedControl.Item value="reencode">
                  <SegmentedControl.ItemText>Réencoder</SegmentedControl.ItemText>
                  <SegmentedControl.ItemHiddenInput />
                </SegmentedControl.Item>
              </SegmentedControl.Control>
            </SegmentedControl>
          </div>

          <!-- Cible -->
          <div class="td td--target">
            {#if isCopy}
              <span class="copy-label">— piste source</span>
            {:else}
              <SegmentedControl
                value={rule.targetCodec}
                onValueChange={(d) => setTarget(src.id, d.value as AudioCodec)}
                class="target-segment"
              >
                <SegmentedControl.Control>
                  <SegmentedControl.Indicator />
                  {#each TARGET_CODECS as t}
                    {@const disabled = t.mkvOnly && container === "mp4"}
                    <SegmentedControl.Item value={t.id} disabled={disabled} title={disabled ? "MKV uniquement" : t.label}>
                      <SegmentedControl.ItemText>{t.label}</SegmentedControl.ItemText>
                      <SegmentedControl.ItemHiddenInput />
                    </SegmentedControl.Item>
                  {/each}
                </SegmentedControl.Control>
              </SegmentedControl>
            {/if}
          </div>

        </div>
      {/each}
    </div>
  </div>

  <!-- ── Débit ─────────────────────────────────────────────────────────────── -->
  <div class="section">
    <span class="section-label">Débit de réencodage</span>
    <div class="bitrate-row">
      {#each [128, 192, 256, 320] as br}
        {@const active = audioBitrate === br}
        <button
          type="button"
          class="bitrate-btn"
          class:bitrate-btn--active={active}
          onclick={() => encoder.setAudioBitrate(br)}
        >
          <span class="bitrate-val">{br}</span>
          <span class="bitrate-unit">kbps</span>
          <span class="bitrate-hint">{BITRATE_HINTS[br]}</span>
        </button>
      {/each}
    </div>
  </div>

</section>

<style>
  /* ── Layout ─────────────────────────────────────────────────────────────── */
  .tab {
    padding: 24px 28px;
    max-width: 600px;
    display: flex;
    flex-direction: column;
    gap: 22px;
  }

  /* ── Header ─────────────────────────────────────────────────────────────── */
  .tab-header {
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

  /* ── Section ─────────────────────────────────────────────────────────────── */
  .section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .section-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--color-subtext2);
  }

  /* ── Préréglages ─────────────────────────────────────────────────────────── */
  .presets-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 5px;
  }

  .preset-wrap {
    display: flex;
    align-items: center;
    gap: 1px;
  }

  .preset-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
    font-weight: 500;
    padding: 4px 11px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-subtext);
    cursor: pointer;
    white-space: nowrap;
    transition: border-color 0.1s, background 0.1s, color 0.1s;
  }
  .preset-btn:hover {
    border-color: var(--color-subtext2);
    color: var(--color-text);
  }
  .preset-btn--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 10%, var(--color-surface));
    color: var(--color-accent);
  }

  .custom-dot {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: currentColor;
    opacity: 0.7;
    flex-shrink: 0;
  }

  .preset-icon-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border: none;
    background: transparent;
    color: var(--color-subtext2);
    cursor: pointer;
    border-radius: 3px;
    transition: color 0.1s, background 0.1s;
  }
  .preset-icon-btn:hover { color: var(--color-subtext); background: var(--color-surface); }
  .preset-icon-btn--delete:hover { color: var(--color-danger, #e05252); }

  .preset-rename-input {
    font-size: 11px;
    padding: 3px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-accent);
    background: var(--color-bg);
    color: var(--color-text);
    outline: none;
    width: 120px;
  }

  .save-form {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .save-input {
    font-size: 11px;
    padding: 3px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-bg);
    color: var(--color-text);
    outline: none;
    width: 130px;
    transition: border-color 0.1s;
  }
  .save-input:focus { border-color: var(--color-accent); }
  .save-confirm {
    font-size: 10px;
    font-weight: 600;
    padding: 3px 9px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 12%, transparent);
    color: var(--color-accent);
    cursor: pointer;
    transition: background 0.1s;
  }
  .save-confirm:hover { background: color-mix(in srgb, var(--color-accent) 20%, transparent); }
  .save-cancel {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border: 1px solid var(--color-border);
    background: transparent;
    color: var(--color-subtext2);
    cursor: pointer;
    border-radius: var(--radius-sm);
    transition: color 0.1s;
  }
  .save-cancel:hover { color: var(--color-danger, #e05252); }

  .preset-save-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 10px;
    font-weight: 500;
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    border: 1px dashed var(--color-border);
    background: transparent;
    color: var(--color-subtext2);
    cursor: pointer;
    white-space: nowrap;
    transition: border-color 0.1s, color 0.1s;
  }
  .preset-save-btn:hover { border-color: var(--color-accent); color: var(--color-accent); }

  /* ── Table ───────────────────────────────────────────────────────────────── */
  .section--table { gap: 0; }

  .table-head {
    display: grid;
    grid-template-columns: 80px 160px 1fr;
    gap: 8px;
    padding: 0 10px 5px;
    border-bottom: 1px solid var(--color-border);
    margin-bottom: 2px;
  }
  .th {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--color-subtext2);
  }

  .table-body {
    display: flex;
    flex-direction: column;
  }

  .rule-row {
    display: grid;
    grid-template-columns: 80px 160px 1fr;
    gap: 8px;
    align-items: center;
    padding: 5px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid transparent;
    transition: background 0.1s, border-color 0.1s;
  }

  .rule-row--present {
    background: color-mix(in srgb, var(--color-accent) 4%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 15%, transparent);
  }
  .rule-row--present:hover {
    background: color-mix(in srgb, var(--color-accent) 7%, transparent);
  }

  .td { display: flex; align-items: center; }
  .td--src { gap: 6px; }

  .src-badge {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    font-weight: 600;
    color: var(--color-subtext);
    padding: 2px 6px;
    border-radius: 4px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    white-space: nowrap;
  }
  .src-badge--present {
    color: var(--color-accent);
    border-color: color-mix(in srgb, var(--color-accent) 35%, var(--color-border));
    background: color-mix(in srgb, var(--color-accent) 7%, var(--color-surface));
  }
  .presence-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--color-accent);
    flex-shrink: 0;
    opacity: 0.7;
  }

  /* Toggle Copier / Réencoder — via SegmentedControl */
  :global(.action-segment) {
    width: fit-content;
  }
  :global(.action-segment [data-part="control"]) {
    font-size: 10px;
    padding: 2px;
    gap: 2px;
  }
  :global(.action-segment [data-part="item"]) {
    padding: 0;
  }
  :global(.action-segment [data-part="item-text"]) {
    font-size: 10px;
    padding: 2px 8px;
    white-space: nowrap;
    opacity: 0.7;
  }
  :global(.action-segment [data-part="indicator"]) {
    background: transparent !important;
    border: 1px solid var(--color-accent);
    border-radius: var(--radius-sm);
  }
  :global(.action-segment [data-part="item"][data-state="checked"] [data-part="item-text"]) {
    color: var(--color-text);
    opacity: 0.7;
  }

  /* Cibles — via SegmentedControl */
  :global(.target-segment) {
    width: fit-content;
  }
  :global(.target-segment [data-part="control"]) {
    padding: 2px;
    gap: 2px;
  }
  :global(.target-segment [data-part="item"]) {
    padding: 0;
  }
  :global(.target-segment [data-part="item-text"]) {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    font-weight: 600;
    padding: 2px 7px;
    opacity: 0.7;
  }
  :global(.target-segment [data-part="indicator"]) {
    background: transparent !important;
    border: 1px solid var(--color-accent);
    border-radius: var(--radius-sm);
  }
  :global(.target-segment [data-part="item"][data-state="checked"] [data-part="item-text"]) {
    color: var(--color-text);
  }

  .copy-label {
    font-size: 10px;
    color: var(--color-subtext2);
    font-style: italic;
  }

  /* ── Débit ───────────────────────────────────────────────────────────────── */
  .bitrate-row {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 6px;
  }
  .bitrate-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 11px 6px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    transition: border-color 0.12s, background 0.12s;
  }
  .bitrate-btn:hover { border-color: var(--color-subtext2); }
  .bitrate-btn--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-surface));
  }
  .bitrate-val {
    font-family: "Geist Mono", monospace;
    font-size: 17px;
    font-weight: 700;
    color: var(--color-subtext);
    line-height: 1;
    transition: color 0.12s;
  }
  .bitrate-btn--active .bitrate-val { color: var(--color-accent); }
  .bitrate-unit {
    font-family: "Geist Mono", monospace;
    font-size: 8px;
    color: var(--color-subtext2);
  }
  .bitrate-hint {
    font-size: 9px;
    color: var(--color-subtext2);
    margin-top: 1px;
  }
  .bitrate-btn--active .bitrate-hint { color: var(--color-accent); opacity: 0.8; }
</style>  