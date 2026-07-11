<script lang="ts">
  import { Check } from "@lucide/svelte";

  type FieldDef = { id: string; label: string };

  type NotifRowDef = {
    key: string;
    notifType: string;
    label: string;
    desc: string;
    icon: string;
    embedTitle: string;
  };

  const NOTIF_VARS: Record<string, string[]> = {
    summary: [
      "{files}",
      "{success}",
      "{errors}",
      "{gain}",
      "{saved}",
      "{duration}",
    ],
    start: ["{files}", "{size}", "{crf}", "{preset}"],
    file_done: [
      "{file}",
      "{size_before}",
      "{size_after}",
      "{gain}",
      "{saved}",
      "{duration}",
      "{crf}",
      "{preset}",
    ],
    error: ["{file}", "{error}"],
    progress: [
      "{file}",
      "{index}",
      "{total}",
      "{percent}",
      "{speed}",
      "{remaining}",
    ],
  };

  let {
    row,
    isOn,
    catalog,
    discordFields,
    discordCustomNotes,
    discordProgressInterval,
    onToggle,
    onToggleField,
    onToggleAllFields,
    onInsertVariable,
    onProgressIntervalChange,
    textareaRef = $bindable(),
  }: {
    row: NotifRowDef;
    isOn: boolean;
    catalog: Record<string, FieldDef[]>;
    discordFields: Record<string, string[]>;
    discordCustomNotes: Record<string, string>;
    discordProgressInterval: number;
    onToggle: () => void;
    onToggleField: (notifType: string, fieldId: string) => void;
    onToggleAllFields: (notifType: string, enable: boolean) => void;
    onInsertVariable: (notifType: string, variable: string) => void;
    onProgressIntervalChange: (val: number) => void;
    textareaRef?: HTMLTextAreaElement;
  } = $props();

  function isFieldEnabled(notifType: string, fieldId: string): boolean {
    return discordFields[notifType]?.includes(fieldId) ?? true;
  }

  function activeFieldCount(notifType: string): number {
    return discordFields[notifType]?.length ?? 0;
  }

  function totalFieldCount(notifType: string): number {
    return catalog[notifType]?.length ?? 0;
  }

  let hasFields = $derived((catalog[row.notifType] ?? []).length > 0);
  let active = $derived(activeFieldCount(row.notifType));
  let total = $derived(totalFieldCount(row.notifType));
  let note = $derived(discordCustomNotes[row.notifType] ?? "");
</script>

<div class="discord-card" class:discord-card--off={!isOn}>
  <!-- En-tête -->
  <div class="discord-card-header">
    <div class="discord-card-header-main">
      <span class="discord-card-icon" aria-hidden="true">{row.icon}</span>
      <div class="min-w-0">
        <span class="discord-card-title">{row.label}</span>
        <p class="discord-card-desc">{row.desc}</p>
      </div>
    </div>

    <div class="discord-card-header-actions">
      {#if hasFields}
        <span
          class="field-badge"
          class:field-badge--warn={isOn && active === 0}
          title="{active}/{total} champs actifs"
        >
          {active}/{total}
        </span>
      {/if}
      <button
        type="button"
        role="switch"
        aria-checked={isOn}
        onclick={onToggle}
        class="toggle {isOn ? 'on' : ''}"
        aria-label={row.label}
      ></button>
    </div>
  </div>

  <!-- Intervalle progression -->
  {#if row.key === "discord_notify_progress" && isOn}
    <div class="discord-card-subfield">
      <label for="discord-progress-interval" class="field-label"
        >Intervalle (secondes)</label
      >
      <input
        id="discord-progress-interval"
        type="number"
        value={discordProgressInterval}
        oninput={(e) =>
          onProgressIntervalChange(
            Number((e.currentTarget as HTMLInputElement).value),
          )}
        min="10"
        max="300"
        placeholder="30"
        class="field-input w-full px-3 py-2"
      />
    </div>
  {/if}

  {#if hasFields}
    <div class="discord-card-body">
      <!-- Header champs -->
      <div class="fields-panel-header">
        <span
          class="font-mono text-[9px] uppercase tracking-wider"
          style="color: var(--color-subtext);">Champs inclus dans l'embed</span
        >
        <div class="flex gap-2">
          <button
            type="button"
            class="micro-btn"
            onclick={() => onToggleAllFields(row.notifType, true)}
            aria-label="Tout activer">Tout</button
          >
          <button
            type="button"
            class="micro-btn"
            onclick={() => onToggleAllFields(row.notifType, false)}
            aria-label="Tout désactiver">Aucun</button
          >
        </div>
      </div>

      <!-- Liste des chips -->
      <div class="fields-list">
        {#each catalog[row.notifType] ?? [] as field}
          {@const enabled = isFieldEnabled(row.notifType, field.id)}
          <button
            type="button"
            class="field-chip"
            class:field-chip--on={enabled}
            aria-pressed={enabled}
            aria-label={field.label}
            onclick={() => onToggleField(row.notifType, field.id)}
          >
            {#if enabled}
              <Check class="w-2.5 h-2.5 shrink-0" aria-hidden="true" />
            {/if}
            <span>{field.label}</span>
          </button>
        {/each}
      </div>

      <!-- Note personnalisée -->
      <div class="custom-note-block">
        <label
          class="font-mono text-[9px] uppercase tracking-wider"
          style="color: var(--color-subtext);"
          for="custom-note-{row.notifType}"
        >
          📝 Note personnalisée
        </label>
        <textarea
          id="custom-note-{row.notifType}"
          bind:this={textareaRef}
          rows="2"
          maxlength="500"
          placeholder="Texte libre affiché en bas de l'embed (optionnel)…"
          class="custom-note-textarea"
          value={note}
          oninput={(e) => {
            // handled by parent via discordCustomNotes binding
          }}
          onchange={(e) => {
            discordCustomNotes[row.notifType] = (
              e.currentTarget as HTMLTextAreaElement
            ).value;
          }}
        ></textarea>
        <div class="custom-note-footer">
          <div class="custom-note-vars">
            {#each NOTIF_VARS[row.notifType] ?? [] as v}
              <button
                type="button"
                class="var-chip"
                title="Cliquer pour insérer {v} dans la note"
                onclick={() => onInsertVariable(row.notifType, v)}>{v}</button
              >
            {/each}
          </div>
          <div class="custom-note-counter">{note.length}/500</div>
        </div>
      </div>

      <!-- Aperçu embed Discord -->
      <div class="embed-preview" aria-label="Aperçu de l'embed Discord">
        <div class="embed-preview-label">Aperçu embed</div>
        <div class="embed-mock">
          <div class="embed-mock-bar"></div>
          <div class="embed-mock-body">
            <div class="embed-mock-meta">
              <span class="embed-mock-avatar" aria-hidden="true">🤖</span>
              <span class="embed-mock-botname">CleanEncode</span>
              <span class="embed-mock-bot-tag">BOT</span>
              <span class="embed-mock-timestamp">aujourd'hui à 14:32</span>
            </div>
            <div class="embed-mock-title">{row.embedTitle}</div>
            <div class="embed-mock-fields">
              {#each catalog[row.notifType] ?? [] as field}
                {#if isFieldEnabled(row.notifType, field.id)}
                  <div class="embed-mock-field">
                    <div class="embed-mock-field-name">{field.label}</div>
                    <div class="embed-mock-field-value">—</div>
                  </div>
                {/if}
              {/each}
              {#if active === 0}
                <span
                  class="font-mono text-[9px]"
                  style="color: var(--color-subtext);"
                >
                  Aucun champ — embed vide
                </span>
              {/if}
              {#if note.trim()}
                <div class="embed-mock-field" style="grid-column: 1 / -1;">
                  <div class="embed-mock-field-name">📝 Note</div>
                  <div
                    class="embed-mock-field-value"
                    style="white-space: pre-wrap; word-break: break-word;"
                  >
                    {note}
                  </div>
                </div>
              {/if}
            </div>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .discord-card {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-panel2, var(--color-panel));
    overflow: hidden;
    transition:
      opacity 0.15s,
      border-color 0.15s;
  }
  .discord-card--off {
    opacity: 0.55;
  }

  .discord-card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    padding: 7px 9px;
  }
  .discord-card-header-main {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }
  .discord-card-icon {
    font-size: 13px;
    line-height: 1;
    flex-shrink: 0;
  }
  .discord-card-title {
    font-size: 11px;
    color: var(--color-text);
    display: block;
  }
  .discord-card-desc {
    font-family: "Geist Mono", monospace;
    font-size: 8px;
    color: var(--color-subtext);
    margin-top: 1px;
  }
  .discord-card-header-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }
  .discord-card-subfield {
    padding: 0 9px 7px;
  }

  .field-badge {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    padding: 1px 5px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--color-accent) 15%, transparent);
    color: var(--color-accent);
    border: 1px solid color-mix(in srgb, var(--color-accent) 25%, transparent);
    white-space: nowrap;
    cursor: default;
  }
  .field-badge--warn {
    background: color-mix(
      in srgb,
      var(--color-danger, #e74c3c) 15%,
      transparent
    );
    color: var(--color-danger, #e74c3c);
    border-color: color-mix(
      in srgb,
      var(--color-danger, #e74c3c) 25%,
      transparent
    );
  }

  .discord-card-body {
    padding: 7px 9px 9px;
    border-top: 1px solid var(--color-border);
    background: var(--color-surface);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .fields-panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .micro-btn {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    padding: 2px 6px;
    border-radius: var(--radius-xs);
    border: 1px solid var(--color-border);
    background: var(--color-panel2, var(--color-panel));
    color: var(--color-subtext);
    cursor: pointer;
    transition:
      background 0.1s,
      color 0.1s;
  }
  .micro-btn:hover {
    background: var(--color-border);
    color: var(--color-text);
  }

  .fields-list {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
  }

  .field-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    padding: 3px 8px;
    border-radius: 999px;
    border: 1px solid var(--color-border);
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    transition:
      background 0.1s,
      color 0.1s,
      border-color 0.1s;
    white-space: nowrap;
  }
  .field-chip:hover {
    background: color-mix(in srgb, var(--color-border) 60%, transparent);
    color: var(--color-text);
  }
  .field-chip--on {
    background: color-mix(in srgb, var(--color-accent) 18%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 40%, transparent);
    color: var(--color-accent);
  }
  .field-chip--on:hover {
    background: color-mix(in srgb, var(--color-accent) 28%, transparent);
  }

  .custom-note-block {
    display: flex;
    flex-direction: column;
    gap: 5px;
    padding-top: 10px;
    border-top: 1px solid var(--color-border);
  }
  .custom-note-footer {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 8px;
  }
  .custom-note-vars {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    flex: 1;
  }
  .var-chip {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    padding: 2px 7px;
    border-radius: var(--radius-full);
    background: color-mix(in srgb, var(--color-accent) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-accent) 25%, transparent);
    color: var(--color-accent);
    cursor: pointer;
    transition:
      background 0.1s,
      transform 0.08s;
  }
  .var-chip:hover {
    background: color-mix(in srgb, var(--color-accent) 22%, transparent);
  }
  .var-chip:active {
    transform: scale(0.95);
  }

  .custom-note-textarea {
    width: 100%;
    resize: vertical;
    min-height: 46px;
    padding: 7px 10px;
    font-family: "DM Sans", sans-serif;
    font-size: 11px;
    line-height: 1.45;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-text);
    outline: none;
    transition: border-color 0.12s;
  }
  .custom-note-textarea:focus {
    border-color: var(--color-accent);
  }
  .custom-note-textarea::placeholder {
    color: var(--color-subtext2);
  }
  .custom-note-counter {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext2);
    text-align: right;
  }

  .embed-preview {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .embed-preview-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-subtext);
  }
  .embed-mock {
    display: flex;
    border-radius: 4px;
    overflow: hidden;
    background: #2b2d31;
    border: 1px solid #1e1f22;
  }
  .embed-mock-bar {
    width: 4px;
    background: var(--color-accent);
    flex-shrink: 0;
  }
  .embed-mock-body {
    padding: 5px 7px;
    flex: 1;
    min-width: 0;
  }
  .embed-mock-meta {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-bottom: 4px;
  }
  .embed-mock-avatar {
    width: 13px;
    height: 13px;
    border-radius: 50%;
    background: color-mix(in srgb, var(--color-accent) 30%, #1e1f22);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 7px;
    flex-shrink: 0;
  }
  .embed-mock-botname {
    font-size: 9px;
    font-weight: 600;
    color: #f2f3f5;
  }
  .embed-mock-bot-tag {
    font-family: "Geist Mono", monospace;
    font-size: 6px;
    font-weight: 700;
    padding: 0px 2px;
    border-radius: 2px;
    background: var(--color-accent);
    color: #1e1f22;
    letter-spacing: 0.02em;
    line-height: 1.4;
  }
  .embed-mock-timestamp {
    font-family: "Geist Mono", monospace;
    font-size: 7px;
    color: #949ba4;
    margin-left: auto;
    white-space: nowrap;
  }
  .embed-mock-title {
    font-size: 10px;
    font-weight: 600;
    color: #f2f3f5;
    margin-bottom: 4px;
  }
  .embed-mock-fields {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 8px;
  }
  .embed-mock-field {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .embed-mock-field-name {
    font-family: "Geist Mono", monospace;
    font-size: 8px;
    font-weight: 600;
    color: #b5bac1;
  }
  .embed-mock-field-value {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: #5c6370;
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
</style>
