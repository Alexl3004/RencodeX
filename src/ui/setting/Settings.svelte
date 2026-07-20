<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { downloadDir } from "@tauri-apps/api/path";
  import { encoder } from "$lib/stores/encoder.svelte";
  import { Check } from "@lucide/svelte";
  import {
    Monitor,
    Cpu,
    FolderOpen,
    MessageSquare,
    Terminal,
    Bot,
  } from "@lucide/svelte";

  import InterfaceTab from "./InterfaceTab.svelte";
  import FfmpegTab from "./FfmpegTab.svelte";
  import PresetsTab from "./PresetsTab.svelte";
  import DiscordTab from "./DiscordTab.svelte";
  import EnvTab from "./EnvTab.svelte";
  import BotTab from "./BotTab.svelte";

  // ── Types ──────────────────────────────────────────────────────────────────

  type TabId = "interface" | "ffmpeg" | "presets" | "discord" | "bot" | "env";

  type FieldDef = { id: string; label: string };

  type EffectiveConfig = {
    ffmpeg_path: string;
    dark_mode: boolean;
    discord_token_set: boolean;
    discord_log_channel_id: string;
    discord_cmd_channel_id: string;
    discord_enabled: boolean;
    discord_notify_start: boolean;
    discord_notify_file_done: boolean;
    discord_notify_summary: boolean;
    discord_notify_error: boolean;
    discord_notify_progress: boolean;
    discord_progress_interval: number;
  };

  type SavedConfig = {
    ffmpeg_path: string;
    dark_mode: boolean;
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
    nav_layout: "vertical" | "horizontal";
    inner_nav_layout: "vertical" | "horizontal";
    discord_fields: Record<string, string[]>;
    discord_custom_notes: Record<string, string>;
    output_dir_presets: string[];
    discord_bot_stopped: boolean;
  };

  type FfmpegCheckResult = {
    path: string;
    exists: boolean;
    executable: boolean;
    version: string | null;
  };

  // ── Constantes UI ──────────────────────────────────────────────────────────

  const TABS: { id: TabId; label: string; icon: any; desc: string }[] = [
    {
      id: "interface",
      label: "Interface",
      icon: Monitor,
      desc: "Thème · couleurs",
    },
    { id: "ffmpeg", label: "FFmpeg", icon: Cpu, desc: "Chemin binaire" },
    {
      id: "presets",
      label: "Chemins",
      icon: FolderOpen,
      desc: "Dossiers de sortie",
    },
    {
      id: "discord",
      label: "Discord",
      icon: MessageSquare,
      desc: "Bot · notifications",
    },
    { id: "env", label: "Env", icon: Terminal, desc: "Variables système" },
    { id: "bot", label: "Bot", icon: Bot, desc: "Statut · commandes" },
  ];

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

  // ── État ───────────────────────────────────────────────────────────────────

  let activeTab = $state<TabId>("interface");
  let saving = $state(false);
  let testing = $state(false);
  let testResult = $state<"ok" | "error" | null>(null);
  let effective = $state<EffectiveConfig | null>(null);
  let confirmReset = $state(false);
  let catalog = $state<Record<string, FieldDef[]>>({});

  let textareaRefs = $state<Record<string, HTMLTextAreaElement>>({});

  let form = $state<SavedConfig>({
    ffmpeg_path: "",
    dark_mode: true,
    discord_bot_token: "",
    discord_log_channel_id: "",
    discord_cmd_channel_id: "",
    discord_enabled: false,
    discord_notify_start: true,
    discord_notify_file_done: true,
    discord_notify_summary: true,
    discord_notify_error: true,
    discord_notify_progress: false,
    discord_progress_interval: 30,
    discord_fields: {},
    discord_custom_notes: {},
    output_dir_presets: [],
    discord_bot_stopped: false,
    nav_layout: "vertical",
    inner_nav_layout: "vertical",
  });

  // ── FFmpeg live check ──────────────────────────────────────────────────────

  let ffmpegCheck = $state<FfmpegCheckResult | null>(null);
  let ffmpegChecking = $state(false);
  let ffmpegCheckTimer: ReturnType<typeof setTimeout> | null = null;

  async function runFfmpegCheck(path: string) {
    ffmpegChecking = true;
    try {
      ffmpegCheck = await invoke<FfmpegCheckResult>("check_ffmpeg", { path });
    } catch {
      ffmpegCheck = null;
    } finally {
      ffmpegChecking = false;
    }
  }

  function scheduleFfmpegCheck(path: string) {
    if (ffmpegCheckTimer) clearTimeout(ffmpegCheckTimer);
    ffmpegCheckTimer = setTimeout(() => runFfmpegCheck(path), 500);
  }

  $effect(() => {
    if (activeTab === "ffmpeg") runFfmpegCheck(form.ffmpeg_path);
  });

  // ── Chargement ─────────────────────────────────────────────────────────────

  async function loadSettings() {
    try {
      const [rawSaved, eff, cat] = await Promise.all([
        invoke<Record<string, any>>("load_config"),
        invoke<EffectiveConfig>("get_effective_config"),
        invoke<Record<string, FieldDef[]>>("get_discord_field_catalog"),
      ]);
      const g = (snake: string, camel: string) =>
        rawSaved[snake] ?? rawSaved[camel];

      form = {
        ...form,
        ffmpeg_path: g("ffmpeg_path", "ffmpegPath") ?? form.ffmpeg_path,
        dark_mode: g("dark_mode", "darkMode") ?? form.dark_mode,
        discord_bot_token:
          g("discord_bot_token", "discordBotToken") ?? form.discord_bot_token,
        discord_log_channel_id:
          g("discord_log_channel_id", "discordLogChannelId") ??
          form.discord_log_channel_id,
        discord_cmd_channel_id:
          g("discord_cmd_channel_id", "discordCmdChannelId") ??
          form.discord_cmd_channel_id,
        discord_enabled:
          g("discord_enabled", "discordEnabled") ?? form.discord_enabled,
        discord_notify_start:
          g("discord_notify_start", "discordNotifyStart") ??
          form.discord_notify_start,
        discord_notify_file_done:
          g("discord_notify_file_done", "discordNotifyFileDone") ??
          form.discord_notify_file_done,
        discord_notify_summary:
          g("discord_notify_summary", "discordNotifySummary") ??
          form.discord_notify_summary,
        discord_notify_error:
          g("discord_notify_error", "discordNotifyError") ??
          form.discord_notify_error,
        discord_notify_progress:
          g("discord_notify_progress", "discordNotifyProgress") ??
          form.discord_notify_progress,
        discord_progress_interval:
          g("discord_progress_interval", "discordProgressInterval") ??
          form.discord_progress_interval,
        discord_fields:
          g("discord_fields", "discordFields") ?? form.discord_fields,
        discord_custom_notes:
          g("discord_custom_notes", "discordCustomNotes") ??
          form.discord_custom_notes,
        output_dir_presets:
          g("output_dir_presets", "outputDirPresets") ??
          form.output_dir_presets,
        discord_bot_stopped:
          g("discord_bot_stopped", "discordBotStopped") ??
          form.discord_bot_stopped,
        nav_layout: g("nav_layout", "navLayout") ?? form.nav_layout,
        inner_nav_layout:
          g("inner_nav_layout", "innerNavLayout") ?? form.inner_nav_layout,
      };

      // Rétrocompat : initialiser discord_fields pour les types manquants
      for (const row of NOTIF_ROWS) {
        if (!form.discord_fields[row.notifType]) {
          form.discord_fields[row.notifType] = (cat[row.notifType] ?? []).map(
            (f) => f.id,
          );
        }
      }
      form.discord_fields = { ...form.discord_fields };

      effective = eff;
      catalog = { ...cat };

      if (form.output_dir_presets.length === 0) {
        try {
          const dl = await downloadDir();
          if (dl) await persistPresets([dl]);
        } catch {
          /* pas critique */
        }
      }
    } catch (e) {
      encoder.log(`Erreur chargement config : ${e}`, "error");
    }
  }

  // ── Sauvegarde ─────────────────────────────────────────────────────────────

  function buildConfigPayload(overridePresets?: string[]) {
    return {
      ffmpeg_path: form.ffmpeg_path,
      dark_mode: form.dark_mode,
      discord_bot_token: form.discord_bot_token,
      discord_log_channel_id: form.discord_log_channel_id,
      discord_cmd_channel_id: form.discord_cmd_channel_id,
      discord_enabled: form.discord_enabled,
      discord_notify_start: form.discord_notify_start,
      discord_notify_file_done: form.discord_notify_file_done,
      discord_notify_summary: form.discord_notify_summary,
      discord_notify_error: form.discord_notify_error,
      discord_notify_progress: form.discord_notify_progress,
      discord_progress_interval: form.discord_progress_interval,
      discord_fields: form.discord_fields,
      discord_custom_notes: form.discord_custom_notes,
      output_dir_presets: overridePresets ?? form.output_dir_presets,
      discord_bot_stopped: form.discord_bot_stopped,
      nav_layout: form.nav_layout,
      inner_nav_layout: form.inner_nav_layout,
    };
  }

  async function save() {
    saving = true;
    try {
      await invoke("save_config", { config: buildConfigPayload() });
      encoder.navLayout = form.nav_layout;
      encoder.innerNavLayout = form.inner_nav_layout;
      effective = await invoke<EffectiveConfig>("get_effective_config");
      encoder.log("Configuration sauvegardée", "success");
      window.dispatchEvent(new CustomEvent("rencodex:config-saved"));
    } catch (e) {
      encoder.log(`Erreur sauvegarde : ${e}`, "error");
    } finally {
      saving = false;
    }
  }

  // ── Réinitialisation ───────────────────────────────────────────────────────

  function handleReset() {
    if (!confirmReset) {
      confirmReset = true;
      setTimeout(() => (confirmReset = false), 3000);
      return;
    }
    confirmReset = false;
    encoder.resetToDefault();
    setTimeout(() => window.dispatchEvent(new Event("resize")), 10);
  }

  // ── Test Discord ───────────────────────────────────────────────────────────

  async function testDiscord() {
    testing = true;
    testResult = null;
    try {
      await invoke("send_discord_notification", {
        botToken: form.discord_bot_token,
        logChannelId: form.discord_log_channel_id,
        summary: {
          files: [],
          total_original_mb: 10.0,
          total_encoded_mb: 6.2,
          total_secs: 42.0,
        },
      });
      testResult = "ok";
      encoder.log("Test Discord envoyé", "success");
    } catch (e) {
      testResult = "error";
      encoder.log(`Erreur Discord : ${e}`, "error");
    } finally {
      testing = false;
      setTimeout(() => (testResult = null), 4000);
    }
  }

  // ── Helpers Discord ────────────────────────────────────────────────────────

  function toggleField(notifType: string, fieldId: string) {
    const current = form.discord_fields[notifType] ?? [];
    form.discord_fields[notifType] = current.includes(fieldId)
      ? current.filter((id) => id !== fieldId)
      : [...current, fieldId];
    form.discord_fields = { ...form.discord_fields };
  }

  function toggleAllFields(notifType: string, enable: boolean) {
    form.discord_fields[notifType] = enable
      ? (catalog[notifType] ?? []).map((f) => f.id)
      : [];
    form.discord_fields = { ...form.discord_fields };
  }

  function insertVariable(notifType: string, variable: string) {
    const current = form.discord_custom_notes[notifType] ?? "";
    const el = textareaRefs[notifType];

    if (!el) {
      form.discord_custom_notes[notifType] = current + variable;
      form.discord_custom_notes = { ...form.discord_custom_notes };
      return;
    }

    const start = el.selectionStart ?? current.length;
    const end = el.selectionEnd ?? current.length;
    form.discord_custom_notes[notifType] =
      current.slice(0, start) + variable + current.slice(end);
    form.discord_custom_notes = { ...form.discord_custom_notes };

    const cursorPos = start + variable.length;
    requestAnimationFrame(() => {
      el.focus();
      el.setSelectionRange(cursorPos, cursorPos);
    });
  }

  // ── Helpers Presets ────────────────────────────────────────────────────────

  async function persistPresets(presets: string[]) {
    form.output_dir_presets = presets;
    try {
      await invoke("save_config", { config: buildConfigPayload(presets) });
      window.dispatchEvent(new CustomEvent("rencodex:config-saved"));
    } catch (e) {
      encoder.log(`Erreur sauvegarde presets : ${e}`, "error");
    }
  }

  async function addPreset(path: string) {
    if (!path || form.output_dir_presets.includes(path)) return;
    await persistPresets([...form.output_dir_presets, path]);
  }

  async function removePreset(path: string) {
    await persistPresets(form.output_dir_presets.filter((p) => p !== path));
  }

  // ── Init ───────────────────────────────────────────────────────────────────

  onMount(() => loadSettings());
</script>

<div
  class="page-root"
  class:page-root--horizontal={encoder.innerNavLayout === "horizontal"}
>
  <!-- ── Sidebar ──────────────────────────────────────────────────────────── -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <span class="sidebar-title">Paramètres</span>
      <span class="sidebar-sub">Config · Outils</span>
    </div>

    <nav class="sidebar-nav" aria-label="Navigation paramètres">
      {#each TABS as tab}
        <button
          type="button"
          onclick={() => (activeTab = tab.id)}
          class="nav-item {activeTab === tab.id ? 'nav-item--active' : ''}"
          aria-current={activeTab === tab.id ? "page" : undefined}
        >
          <div class="nav-item-icon">
            <tab.icon class="w-3.5 h-3.5" />
          </div>
          <div class="nav-item-text">
            <span class="nav-item-label">{tab.label}</span>
            <span class="nav-item-desc">{tab.desc}</span>
          </div>
          {#if activeTab === tab.id}
            <div class="nav-item-indicator" aria-hidden="true"></div>
          {/if}
        </button>
      {/each}
    </nav>

    <div class="sidebar-footer">
      <button
        onclick={save}
        disabled={saving}
        class="save-btn"
        aria-label={saving
          ? "Sauvegarde en cours"
          : "Sauvegarder les paramètres"}
      >
        {#if saving}
          <span class="spinner" aria-hidden="true"></span>
          Sauvegarde…
        {:else}
          <Check class="w-3.5 h-3.5" />
          Sauvegarder
        {/if}
      </button>
    </div>
  </aside>

  <!-- ── Content panel ────────────────────────────────────────────────────── -->
  <div class="content-panel">
    <div class="content-inner">
      {#if activeTab === "interface"}
        <InterfaceTab
          {form}
          {confirmReset}
          onReset={handleReset}
          onSave={save}
        />
      {:else if activeTab === "ffmpeg"}
        <FfmpegTab
          bind:ffmpegPath={form.ffmpeg_path}
          {ffmpegCheck}
          {ffmpegChecking}
          onInput={(path) => scheduleFfmpegCheck(path)}
          onBrowse={(path) => {
            form.ffmpeg_path = path;
            runFfmpegCheck(path);
          }}
        />
      {:else if activeTab === "presets"}
        <PresetsTab
          presets={form.output_dir_presets}
          onAdd={addPreset}
          onRemove={removePreset}
        />
      {:else if activeTab === "discord"}
        <DiscordTab
          bind:form
          {effective}
          {catalog}
          {testing}
          {testResult}
          onTestDiscord={testDiscord}
          onToggleField={toggleField}
          onToggleAllFields={toggleAllFields}
          onInsertVariable={insertVariable}
          bind:textareaRefs
        />
      {:else if activeTab === "env"}
        <EnvTab {effective} {form} />
      {:else if activeTab === "bot"}
        <BotTab
          discordEnabled={form.discord_enabled}
          discordBotToken={form.discord_bot_token}
          discordCmdChannelId={form.discord_cmd_channel_id}
          discordLogChannelId={form.discord_log_channel_id}
          discordBotStopped={form.discord_bot_stopped}
          onConfigChange={(token, cmdChannelId, logChannelId, enabled, botStopped) => {
            form.discord_bot_token = token;
            form.discord_cmd_channel_id = cmdChannelId;
            form.discord_log_channel_id = logChannelId;
            form.discord_enabled = enabled;
            form.discord_bot_stopped = botStopped;
          }}
        />
      {/if}
    </div>
  </div>
</div>

<style>
  /* ── Layout ──────────────────────────────────────────────────────────────── */

  .page-root {
    display: flex;
    height: 100%;
    overflow: hidden;
    background: var(--color-surface);
  }

  /* ── Sidebar ─────────────────────────────────────────────────────────────── */

  .sidebar {
    width: 200px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    background: var(--color-panel);
    border-right: 1px solid var(--color-border);
    overflow: hidden;
  }

  .sidebar-header {
    padding: 20px 16px 14px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .sidebar-title {
    display: block;
    font-family: "Geist Mono", monospace;
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text);
    letter-spacing: -0.02em;
  }

  .sidebar-sub {
    display: block;
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
    margin-top: 2px;
  }

  .sidebar-nav {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 8px 0;
    gap: 1px;
    overflow-y: auto;
  }

  .sidebar-footer {
    padding: 12px;
    border-top: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  /* ── Nav items ───────────────────────────────────────────────────────────── */

  .nav-item {
    position: relative;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 14px;
    min-height: 48px;
    width: 100%;
    background: transparent;
    border: none;
    border-left: 2px solid transparent;
    cursor: pointer;
    text-align: left;
    transition:
      background 0.12s,
      border-color 0.12s;
  }

  .nav-item:hover {
    background: color-mix(in srgb, var(--color-accent) 6%, transparent);
  }

  .nav-item--active {
    background: color-mix(in srgb, var(--color-accent) 10%, transparent);
    border-left-color: var(--color-accent);
  }

  .nav-item-icon {
    color: var(--color-subtext);
    flex-shrink: 0;
    display: flex;
    align-items: center;
  }
  .nav-item--active .nav-item-icon {
    color: var(--color-accent);
  }

  .nav-item-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .nav-item-label {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    font-weight: 500;
    color: var(--color-subtext);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .nav-item--active .nav-item-label {
    color: var(--color-text);
  }

  .nav-item-desc {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext2);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .nav-item-indicator {
    position: absolute;
    right: 0;
    top: 50%;
    transform: translateY(-50%);
    width: 3px;
    height: 16px;
    border-radius: 2px 0 0 2px;
    background: var(--color-accent);
  }

  /* ── Save button ─────────────────────────────────────────────────────────── */

  .save-btn {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--color-accent) 15%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-accent) 30%, transparent);
    color: var(--color-accent);
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition:
      background 0.15s,
      border-color 0.15s;
  }

  .save-btn:hover:not(:disabled) {
    background: color-mix(in srgb, var(--color-accent) 22%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 45%, transparent);
  }

  .save-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* ── Content panel ───────────────────────────────────────────────────────── */

  .content-panel {
    flex: 1 1 0;
    min-width: 0;
    overflow-y: auto;
  }

  .content-inner {
    max-width: 760px;
    margin: 0 auto;
    width: 100%;
  }

  /* ── Spinner ─────────────────────────────────────────────────────────────── */

  .spinner {
    display: inline-block;
    width: 12px;
    height: 12px;
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

  /* ── Layout horizontal ──────────────────────────────────────────────────── */

  .page-root--horizontal {
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }

  .page-root--horizontal .sidebar {
    width: 100%;
    height: auto;
    flex-direction: row;
    align-items: center;
    border-right: none;
    border-bottom: 1px solid var(--color-border);
    padding: 0 12px;
    gap: 0;
    overflow-x: auto;
    overflow-y: visible;
    flex-shrink: 0;
  }

  .page-root--horizontal .sidebar-header,
  .page-root--horizontal .sidebar-footer {
    display: none;
  }

  .page-root--horizontal .sidebar-nav {
    flex-direction: row;
    padding: 0;
    gap: 2px;
    overflow: visible;
    flex: 1;
    justify-content: center;
  }

  .page-root--horizontal .nav-item {
    flex-direction: row;
    min-height: 36px;
    width: auto;
    padding: 6px 14px;
    gap: 6px;
    border-left: none;
    border-bottom: none;
    border-radius: var(--radius-sm);
    white-space: nowrap;
  }

  .page-root--horizontal .nav-item--active {
    border-left-color: transparent;
  }
  .page-root--horizontal .nav-item-indicator {
    display: none;
  }

  .page-root--horizontal .nav-item-text {
    flex-direction: row;
    align-items: center;
    gap: 6px;
  }

  .page-root--horizontal .nav-item-desc {
    display: none;
  }

  .page-root--horizontal .content-panel {
    flex: 1 1 0;
    min-height: 0;
    min-width: 0;
    overflow-y: auto;
  }

  .page-root--horizontal .content-panel > *,
  .page-root--horizontal .content-inner {
    max-width: 760px;
    margin-left: auto;
    margin-right: auto;
    width: 100%;
  }
</style>