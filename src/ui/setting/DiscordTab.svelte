<script lang="ts">
  import { Eye, EyeOff, Check, X } from "@lucide/svelte";
  import NotifRow from "./NotifRow.svelte";

  type FieldDef = { id: string; label: string };

  type SavedConfig = {
    discord_bot_token: string;
    discord_log_channel_id: string;
    discord_cmd_channel_id: string;
    discord_enabled: boolean;
    discord_notify_start: boolean;
    discord_notify_file_done: boolean;
    discord_notify_summary: boolean;
    discord_notify_error: boolean;
    discord_notify_progress: boolean;
    discord_progress_interval: number;
    discord_fields: Record<string, string[]>;
    discord_custom_notes: Record<string, string>;
  };

  type EffectiveConfig = {
    discord_token_set: boolean;
    discord_log_channel_id: string;
    discord_cmd_channel_id: string;
  };

  const NOTIF_ROWS: Array<{
    key: keyof SavedConfig;
    notifType: string;
    label: string;
    desc: string;
    icon: string;
    embedTitle: string;
  }> = [
    {
      key: "discord_notify_start",
      notifType: "start",
      label: "Début d'encodage",
      desc: "Envoi au démarrage",
      icon: "🎬",
      embedTitle: "🎬 Encodage démarré",
    },
    {
      key: "discord_notify_file_done",
      notifType: "file_done",
      label: "Fichier terminé",
      desc: "Après chaque fichier",
      icon: "✅",
      embedTitle: "✅ Fichier encodé",
    },
    {
      key: "discord_notify_progress",
      notifType: "progress",
      label: "Progression",
      desc: "Mise à jour périodique",
      icon: "📈",
      embedTitle: "📈 Encodage en cours",
    },
    {
      key: "discord_notify_summary",
      notifType: "summary",
      label: "Résumé global",
      desc: "Bilan de session",
      icon: "🏁",
      embedTitle: "✅ Encodage terminé",
    },
    {
      key: "discord_notify_error",
      notifType: "error",
      label: "Erreur d'encodage",
      desc: "En cas d'échec",
      icon: "⚠️",
      embedTitle: "❌ Erreur d'encodage",
    },
  ];

  let {
    form = $bindable(),
    effective,
    catalog,
    testing,
    testResult,
    onTestDiscord,
    onToggleField,
    onToggleAllFields,
    onInsertVariable,
    textareaRefs = $bindable(),
  }: {
    form: SavedConfig;
    effective: EffectiveConfig | null;
    catalog: Record<string, FieldDef[]>;
    testing: boolean;
    testResult: "ok" | "error" | null;
    onTestDiscord: () => void;
    onToggleField: (notifType: string, fieldId: string) => void;
    onToggleAllFields: (notifType: string, enable: boolean) => void;
    onInsertVariable: (notifType: string, variable: string) => void;
    textareaRefs: Record<string, HTMLTextAreaElement>;
  } = $props();

  let showToken = $state(false);

  let discordFromEnv = $derived(false);
</script>

<section class="content-section">
  <header class="section-header">
    <div>
      <h2 class="section-title">Discord</h2>
      <p class="section-desc">
        Configurez le bot Discord pour recevoir des notifications d'encodage.
      </p>
    </div>
    <label class="toggle-row">
      <span class="font-mono text-[10px]" style="color: var(--color-subtext);"
        >Activer</span
      >
      <button
        type="button"
        role="switch"
        aria-checked={form.discord_enabled}
        aria-label={form.discord_enabled
          ? "Désactiver Discord"
          : "Activer Discord"}
        onclick={() => (form.discord_enabled = !form.discord_enabled)}
        class="toggle {form.discord_enabled ? 'on' : ''}"
      ></button>
    </label>
  </header>

  {#if form.discord_enabled}
    {#if discordFromEnv}
      <div
        class="flex items-center gap-2 font-mono text-[10px] px-3 py-2 rounded-[var(--radius-sm)]"
        style="color: var(--color-success); background: color-mix(in srgb, var(--color-success) 10%, transparent); border: 1px solid color-mix(in srgb, var(--color-success) 20%, transparent);"
        role="status"
      >
        <Check class="w-3.5 h-3.5" aria-hidden="true" />
        Token actif via variable d'environnement
      </div>
    {/if}

    <!-- Bot token -->
    <div>
      <label for="discord-token" class="field-label">Token du bot</label>
      <div class="relative">
        <input
          id="discord-token"
          type={showToken ? "text" : "password"}
          bind:value={form.discord_bot_token}
          disabled={discordFromEnv}
          placeholder="MTEx…"
          class="field-input w-full px-3 py-2 pr-9"
        />
        <button
          onclick={() => (showToken = !showToken)}
          type="button"
          class="absolute right-2 top-1/2 -translate-y-1/2 icon-btn-inline"
          aria-label={showToken ? "Masquer le token" : "Afficher le token"}
        >
          {#if showToken}
            <EyeOff class="w-3.5 h-3.5" aria-hidden="true" />
          {:else}
            <Eye class="w-3.5 h-3.5" aria-hidden="true" />
          {/if}
        </button>
      </div>
    </div>

    <!-- Channels -->
    <div>
      <label for="discord-log-channel" class="field-label">Salon de logs</label>
      <input
        id="discord-log-channel"
        type="text"
        bind:value={form.discord_log_channel_id}
        placeholder="ID du salon"
        class="field-input w-full px-3 py-2"
      />
    </div>
    <div>
      <label for="discord-cmd-channel" class="field-label"
        >Salon de commandes</label
      >
      <input
        id="discord-cmd-channel"
        type="text"
        bind:value={form.discord_cmd_channel_id}
        placeholder="ID du salon"
        class="field-input w-full px-3 py-2"
      />
    </div>

    <!-- Notifications -->
    <div class="pt-3" style="border-top: 1px solid var(--color-border);">
      <div class="section-label mb-2">Notifications</div>

      <div class="notif-grid">
        {#each NOTIF_ROWS as row}
          {@const isOn = (form as any)[row.key] as boolean}
          <NotifRow
            {row}
            {isOn}
            {catalog}
            discordFields={form.discord_fields}
            discordCustomNotes={form.discord_custom_notes}
            discordProgressInterval={form.discord_progress_interval}
            onToggle={() => {
              (form as any)[row.key] = !isOn;
            }}
            {onToggleField}
            {onToggleAllFields}
            {onInsertVariable}
            onProgressIntervalChange={(val) => {
              form.discord_progress_interval = val;
            }}
            bind:textareaRef={textareaRefs[row.notifType]}
          />
        {/each}
      </div>
    </div>

    <!-- Test Discord -->
    <button
      onclick={onTestDiscord}
      disabled={testing ||
        (!form.discord_bot_token && !effective?.discord_token_set) ||
        !form.discord_log_channel_id.trim()}
      class="w-full btn font-mono text-[11px] justify-center"
      class:btn-primary={testResult === "ok"}
      class:btn-danger={testResult === "error"}
      aria-label={testing
        ? "Test en cours"
        : testResult === "ok"
          ? "Test réussi"
          : testResult === "error"
            ? "Test échoué"
            : "Tester la notification Discord"}
    >
      {#if testing}
        <span
          class="spinner w-3 h-3 border-2 border-current/30 border-t-current shrink-0 rounded-full animate-spin"
        ></span>
        ENVOI…
      {:else if testResult === "ok"}<Check class="w-3.5 h-3.5" /> TEST ENVOYÉ
      {:else if testResult === "error"}<X class="w-3.5 h-3.5" /> ÉCHEC
      {:else}TESTER LA NOTIFICATION{/if}
    </button>
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
  .section-label {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
    text-transform: uppercase;
    letter-spacing: 0.06em;
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
  .field-input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .toggle-row {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
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

  .notif-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px;
    align-items: start;
  }
</style>
