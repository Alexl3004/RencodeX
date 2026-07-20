<svelte:head>
  <script type="module" src="https://cdn.jsdelivr.net/npm/ldrs/dist/auto/dot-wave.js"></script>
</svelte:head>

<script lang="ts">
  import { Check, X, Send, ToggleLeft, ToggleRight, ChevronDown, Settings2 } from "@lucide/svelte";
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
    { key: "discord_notify_start",     notifType: "start",     label: "Démarrage",   desc: "Au lancement",          icon: "🎬", embedTitle: "🎬 Encodage démarré"  },
    { key: "discord_notify_file_done", notifType: "file_done", label: "Fichier",     desc: "Après chaque fichier",  icon: "✅", embedTitle: "✅ Fichier encodé"    },
    { key: "discord_notify_progress",  notifType: "progress",  label: "Progression", desc: "Mise à jour périodique",icon: "📈", embedTitle: "📈 Encodage en cours" },
    { key: "discord_notify_summary",   notifType: "summary",   label: "Résumé",      desc: "Bilan de session",      icon: "🏁", embedTitle: "✅ Encodage terminé"  },
    { key: "discord_notify_error",     notifType: "error",     label: "Erreur",      desc: "En cas d'échec",        icon: "⚠️", embedTitle: "❌ Erreur d'encodage" },
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

  let ready = $derived(effective !== null);

  let discordFromEnv = $derived(
    !!effective?.discord_token_set && !form.discord_bot_token.trim()
  );

  let canTest = $derived(
    !form.discord_bot_token.trim() ? !!effective?.discord_token_set : true
  );

  let activeCount = $derived(
    NOTIF_ROWS.filter(r => (form as any)[r.key] as boolean).length
  );

  let openRow = $state<string | null>(null);

  function toggleRow(notifType: string) {
    openRow = openRow === notifType ? null : notifType;
  }
</script>

<section class="content-section">

  <!-- ── Header ──────────────────────────────────────────────────────────────── -->
  <header class="section-header">
    <div>
      <h2 class="section-title">Notifications Discord</h2>
      <p class="section-desc">
        Messages envoyés dans votre salon lors des encodages.
        Bot &amp; salon de commandes → onglet <strong>Bot</strong>.
      </p>
    </div>

    <div class="header-right">
      {#if !ready}
        <div class="skel skel-badge"></div>
      {:else}
        <div class="status-pill" class:pill-active={activeCount > 0}>
          <span class="pill-dot" class:dot-active={activeCount > 0}></span>
          {activeCount}/{NOTIF_ROWS.length}
        </div>
        {#if discordFromEnv}
          <div class="env-chip">
            <Check class="w-3 h-3" />
            ENV
          </div>
        {/if}
      {/if}
    </div>
  </header>

  <!-- ── Collapsibles ─────────────────────────────────────────────────────── -->
  <div class="rows-wrap">
    {#if !ready}
      {#each { length: 5 } as _}
        <div class="skel skel-row"></div>
      {/each}
    {:else}
      {#each NOTIF_ROWS as row}
        {@const isOn    = (form as any)[row.key] as boolean}
        {@const isOpen  = openRow === row.notifType}

        <div class="collapsible" class:is-on={isOn} class:is-open={isOpen}>

          <!-- Ligne de titre -->
          <div class="row-header">

            <!-- Toggle on/off -->
            <button
              class="row-toggle"
              onclick={() => { (form as any)[row.key] = !isOn; }}
              aria-label="{isOn ? 'Désactiver' : 'Activer'} {row.label}"
            >
              {#if isOn}
                <ToggleRight class="w-4 h-4 toggle-icon-on" />
              {:else}
                <ToggleLeft class="w-4 h-4 toggle-icon-off" />
              {/if}
            </button>

            <!-- Label cliquable → expand -->
            <button
              class="row-expand"
              onclick={() => toggleRow(row.notifType)}
              aria-expanded={isOpen}
            >
              <span class="row-icon">{row.icon}</span>
              <span class="row-label" class:label-on={isOn}>{row.label}</span>
              <span class="row-desc">{row.desc}</span>
              <Settings2 class="w-3 h-3 row-settings-icon {isOpen ? 'settings-open' : ''}" />
              <ChevronDown class="w-3.5 h-3.5 row-chevron {isOpen ? 'chevron-open' : ''}" />
            </button>

          </div>

          <!-- Contenu dépliable -->
          {#if isOpen}
            <div class="row-body">
              <NotifRow
                {row}
                {isOn}
                {catalog}
                discordFields={form.discord_fields}
                discordCustomNotes={form.discord_custom_notes}
                discordProgressInterval={form.discord_progress_interval}
                onToggle={() => { (form as any)[row.key] = !isOn; }}
                {onToggleField}
                {onToggleAllFields}
                {onInsertVariable}
                onProgressIntervalChange={(val) => { form.discord_progress_interval = val; }}
                bind:textareaRef={textareaRefs[row.notifType]}
              />
            </div>
          {/if}

        </div>
      {/each}
    {/if}
  </div>

  <!-- ── Bouton test ─────────────────────────────────────────────────────── -->
  {#if !ready}
    <div class="skel skel-btn"></div>
  {:else}
    <button
      onclick={onTestDiscord}
      disabled={testing || !canTest}
      class="test-btn"
      class:btn-ok={testResult === "ok"}
      class:btn-err={testResult === "error"}
    >
      {#if testing}
        <l-dot-wave size="18" speed="1.1" color="currentColor"></l-dot-wave>
        <span>ENVOI EN COURS</span>
      {:else if testResult === "ok"}
        <Check class="w-3.5 h-3.5" /><span>TEST ENVOYÉ</span>
      {:else if testResult === "error"}
        <X class="w-3.5 h-3.5" /><span>ÉCHEC</span>
      {:else}
        <Send class="w-3.5 h-3.5" /><span>TESTER LA NOTIFICATION</span>
      {/if}
    </button>
  {/if}

</section>

<style>
  /* ── Layout ──────────────────────────────────────────────────────────────── */

  .content-section {
    padding: 24px 28px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  /* ── Header ──────────────────────────────────────────────────────────────── */

  .section-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .section-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text);
    letter-spacing: -0.02em;
    margin: 0 0 4px;
  }

  .section-desc {
    font-size: 11px;
    color: var(--color-subtext);
    line-height: 1.55;
    margin: 0;
  }

  .section-desc strong { color: var(--color-text); font-weight: 600; }

  .header-right {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
    margin-top: 2px;
  }

  .status-pill {
    display: flex;
    align-items: center;
    gap: 5px;
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    font-weight: 600;
    color: var(--color-subtext);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 999px;
    padding: 2px 8px 2px 6px;
    transition: color 0.15s, border-color 0.15s, background 0.15s;
  }

  .status-pill.pill-active {
    color: var(--color-accent);
    border-color: color-mix(in srgb, var(--color-accent) 28%, transparent);
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-surface));
  }

  .pill-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--color-subtext);
    opacity: 0.4;
    transition: background 0.15s, opacity 0.15s;
  }

  .pill-dot.dot-active {
    background: var(--color-accent);
    opacity: 1;
    box-shadow: 0 0 5px color-mix(in srgb, var(--color-accent) 55%, transparent);
  }

  .env-chip {
    display: flex;
    align-items: center;
    gap: 4px;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.06em;
    color: var(--color-success);
    background: color-mix(in srgb, var(--color-success) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-success) 22%, transparent);
    border-radius: 999px;
    padding: 2px 7px;
  }

  /* ── Collapsibles ────────────────────────────────────────────────────────── */

  .rows-wrap {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .collapsible {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    overflow: hidden;
    transition: border-color 0.12s;
  }

  .collapsible:hover {
    border-color: color-mix(in srgb, var(--color-accent) 25%, var(--color-border));
  }

  .collapsible.is-on {
    border-color: color-mix(in srgb, var(--color-accent) 28%, var(--color-border));
    background: color-mix(in srgb, var(--color-accent) 3%, var(--color-surface));
  }

  .collapsible.is-open {
    border-color: var(--color-accent);
  }

  /* Ligne header */
  .row-header {
    display: flex;
    align-items: stretch;
  }

  /* Bouton toggle gauche */
  .row-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0 10px;
    background: transparent;
    border: none;
    border-right: 1px solid var(--color-border);
    cursor: pointer;
    flex-shrink: 0;
    transition: background 0.1s;
  }

  .row-toggle:hover {
    background: color-mix(in srgb, var(--color-accent) 7%, transparent);
  }

  :global(.toggle-icon-on)  { color: var(--color-accent); }
  :global(.toggle-icon-off) { color: var(--color-subtext); opacity: 0.5; }

  /* Bouton expand */
  .row-expand {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    padding: 9px 10px 9px 12px;
    background: transparent;
    border: none;
    cursor: pointer;
    text-align: left;
    min-width: 0;
    transition: background 0.1s;
  }

  .row-expand:hover {
    background: color-mix(in srgb, var(--color-accent) 4%, transparent);
  }

  .row-icon {
    font-size: 13px;
    line-height: 1;
    flex-shrink: 0;
  }

  .row-label {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    font-weight: 600;
    color: var(--color-subtext);
    white-space: nowrap;
    transition: color 0.12s;
    flex-shrink: 0;
  }

  .row-label.label-on { color: var(--color-text); }

  .row-desc {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
    opacity: 0.45;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }

  :global(.row-settings-icon) {
    color: var(--color-subtext);
    opacity: 0;
    flex-shrink: 0;
    transition: opacity 0.12s, color 0.12s;
  }

  .row-expand:hover :global(.row-settings-icon),
  :global(.row-settings-icon.settings-open) {
    opacity: 1;
    color: var(--color-accent);
  }

  :global(.row-chevron) {
    color: var(--color-subtext);
    flex-shrink: 0;
    transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1), color 0.12s;
    opacity: 0.5;
  }

  :global(.row-chevron.chevron-open) {
    transform: rotate(180deg);
    color: var(--color-accent);
    opacity: 1;
  }

  /* Corps dépliable */
  .row-body {
    border-top: 1px solid var(--color-border);
    background: color-mix(in srgb, var(--color-accent) 2%, var(--color-panel));
    padding: 12px 14px;
    animation: expand-in 0.18s cubic-bezier(0.16, 1, 0.3, 1) both;
  }

  @keyframes expand-in {
    from {
      opacity: 0;
      transform: translateY(-6px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  /* ── Bouton test ─────────────────────────────────────────────────────────── */

  .test-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 7px;
    width: 100%;
    padding: 9px 16px;
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.06em;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-subtext);
    cursor: pointer;
    transition: all 0.12s;
  }

  .test-btn:hover:not(:disabled) {
    border-color: var(--color-accent);
    color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 6%, var(--color-surface));
  }

  .test-btn:active:not(:disabled) { transform: scale(0.995); }
  .test-btn:disabled { opacity: 0.35; cursor: not-allowed; }

  .test-btn.btn-ok {
    color: var(--color-success);
    border-color: color-mix(in srgb, var(--color-success) 35%, transparent);
    background: color-mix(in srgb, var(--color-success) 8%, var(--color-surface));
  }

  .test-btn.btn-err {
    color: var(--color-danger, #e05c5c);
    border-color: color-mix(in srgb, var(--color-danger, #e05c5c) 35%, transparent);
    background: color-mix(in srgb, var(--color-danger, #e05c5c) 8%, var(--color-surface));
  }

  /* ── Skeletons ───────────────────────────────────────────────────────────── */

  @keyframes shimmer {
    0%   { background-position: -200% center; }
    100% { background-position:  200% center; }
  }

  .skel {
    background: linear-gradient(
      90deg,
      var(--color-border) 0%,
      color-mix(in srgb, var(--color-border) 55%, var(--color-surface)) 45%,
      var(--color-border) 90%
    );
    background-size: 200% 100%;
    animation: shimmer 1.5s ease-in-out infinite;
    border-radius: var(--radius-sm);
  }

  .skel-badge { width: 48px; height: 20px; border-radius: 999px; }
  .skel-row   { height: 38px; }
  .skel-btn   { height: 36px; }
</style>