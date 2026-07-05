<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { downloadDir } from "@tauri-apps/api/path";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { encoder } from "$lib/stores/encoder.svelte";
  import { theme, DARK_COMBOS, LIGHT_COMBOS } from "$lib/stores/theme.svelte";
  import { Sun, Moon } from "@lucide/svelte";
  import { Eye, EyeOff, X, Check, RotateCcw, FolderInput, Monitor, Cpu, FolderOpen, MessageSquare, Terminal, Palette } from '@lucide/svelte';

  type TabId = "interface" | "ffmpeg" | "presets" | "discord" | "env";
  let activeTab = $state<TabId>("interface");

  const TABS: { id: TabId; label: string; icon: any; desc: string }[] = [
    { id: "interface", label: "Interface",  icon: Monitor,      desc: "Thème · couleurs" },
    { id: "ffmpeg",    label: "FFmpeg",     icon: Cpu,          desc: "Chemin binaire" },
    { id: "presets",   label: "Chemins",    icon: FolderOpen,   desc: "Dossiers de sortie" },
    { id: "discord",   label: "Discord",    icon: MessageSquare, desc: "Bot · notifications" },
    { id: "env",       label: "Env",        icon: Terminal,     desc: "Variables système" },
  ];

  // ── Types ──────────────────────────────────────────────────────────────────

  type FieldDef = {
    id: string;
    label: string;
  };

  type EffectiveConfig = {
    ffmpeg_path: string;
    dark_mode: boolean;
    send_email: boolean;
    email_to: string;
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
    send_email: boolean;
    email_to: string;
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
    /** Champs activés par type : { "start": ["files","size","crf"], … } */
    discord_fields: Record<string, string[]>;
    discord_custom_notes: Record<string, string>;
    /** Chemins de sortie prédéfinis */
    output_dir_presets: string[];
  };

  // ── Props & état ───────────────────────────────────────────────────────────

  let saving    = $state(false);
  let testing   = $state(false);
  let testResult = $state<"ok" | "error" | null>(null);
  let effective  = $state<EffectiveConfig | null>(null);
  let showToken  = $state(false);
  let confirmReset = $state(false);

  /** Catalogue reçu du backend : { notifType -> FieldDef[] } */
  let catalog = $state<Record<string, FieldDef[]>>({});

  /** Références DOM des textarea de note, indexées par notifType — utilisées pour
   *  insérer une variable à la position du curseur au clic sur un chip. */
  let textareaRefs: Record<string, HTMLTextAreaElement> = {};

  let form = $state<SavedConfig>({
    ffmpeg_path: "",
    dark_mode: true,
    send_email: false,
    email_to: "",
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
    nav_layout: "vertical",
    inner_nav_layout: "vertical",
  });

  // ── Définition statique des notifications (labels UI) ─────────────────────

  const NOTIF_ROWS: Array<{
    key: keyof SavedConfig;
    notifType: string;
    label: string;
    desc: string;
    icon: string;
    embedTitle: string;
  }> = [
    { key: "discord_notify_start",     notifType: "start",     label: "Début d'encodage",  desc: "Envoi au démarrage",     icon: "🎬", embedTitle: "🎬 Encodage démarré" },
    { key: "discord_notify_file_done", notifType: "file_done", label: "Fichier terminé",   desc: "Après chaque fichier",   icon: "✅", embedTitle: "✅ Fichier encodé" },
    { key: "discord_notify_progress",  notifType: "progress",  label: "Progression",       desc: "Mise à jour périodique", icon: "📈", embedTitle: "📈 Encodage en cours" },
    { key: "discord_notify_summary",   notifType: "summary",   label: "Résumé global",     desc: "Bilan de session",       icon: "🏁", embedTitle: "✅ Encodage terminé" },
    { key: "discord_notify_error",     notifType: "error",     label: "Erreur d'encodage", desc: "En cas d'échec",         icon: "⚠️", embedTitle: "❌ Erreur d'encodage" },
  ];

  /** Variables disponibles par type de notification, insérables dans la note. */
  const NOTIF_VARS: Record<string, string[]> = {
    summary:   ["{files}", "{success}", "{errors}", "{gain}", "{saved}", "{duration}"],
    start:     ["{files}", "{size}", "{crf}", "{preset}"],
    file_done: ["{file}", "{size_before}", "{size_after}", "{gain}", "{saved}", "{duration}", "{crf}", "{preset}"],
    error:     ["{file}", "{error}"],
    progress:  ["{file}", "{index}", "{total}", "{percent}", "{speed}", "{remaining}"],
  };

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

  // ── Chargement ─────────────────────────────────────────────────────────────

  async function loadSettings() {
    try {
      const [rawSaved, eff, cat] = await Promise.all([
        invoke<Record<string, any>>("load_config"),
        invoke<EffectiveConfig>("get_effective_config"),
        invoke<Record<string, FieldDef[]>>("get_discord_field_catalog"),
      ]);

      // DEBUG — à supprimer une fois le problème résolu
      console.log("[Settings] clés reçues de load_config :", Object.keys(rawSaved));
      console.log("[Settings] rawSaved :", JSON.stringify(rawSaved, null, 2));

      // Tauri peut renvoyer les clés en camelCase ou snake_case selon la version.
      // On normalise en récupérant les deux variantes pour chaque champ.
      const g = (snake: string, camel: string) =>
        rawSaved[snake] ?? rawSaved[camel];

      form = {
        ...form,
        ffmpeg_path:               g("ffmpeg_path",               "ffmpegPath")               ?? form.ffmpeg_path,
        dark_mode:                 g("dark_mode",                 "darkMode")                 ?? form.dark_mode,
        send_email:                g("send_email",                "sendEmail")                ?? form.send_email,
        email_to:                  g("email_to",                  "emailTo")                  ?? form.email_to,
        discord_bot_token:         g("discord_bot_token",         "discordBotToken")          ?? form.discord_bot_token,
        discord_log_channel_id:    g("discord_log_channel_id",    "discordLogChannelId")      ?? form.discord_log_channel_id,
        discord_cmd_channel_id:    g("discord_cmd_channel_id",    "discordCmdChannelId")      ?? form.discord_cmd_channel_id,
        discord_enabled:           g("discord_enabled",           "discordEnabled")           ?? form.discord_enabled,
        discord_notify_start:      g("discord_notify_start",      "discordNotifyStart")       ?? form.discord_notify_start,
        discord_notify_file_done:  g("discord_notify_file_done",  "discordNotifyFileDone")    ?? form.discord_notify_file_done,
        discord_notify_summary:    g("discord_notify_summary",    "discordNotifySummary")     ?? form.discord_notify_summary,
        discord_notify_error:      g("discord_notify_error",      "discordNotifyError")       ?? form.discord_notify_error,
        discord_notify_progress:   g("discord_notify_progress",   "discordNotifyProgress")    ?? form.discord_notify_progress,
        discord_progress_interval: g("discord_progress_interval", "discordProgressInterval")  ?? form.discord_progress_interval,
        discord_fields:            g("discord_fields",            "discordFields")            ?? form.discord_fields,
        discord_custom_notes:      g("discord_custom_notes",      "discordCustomNotes")       ?? form.discord_custom_notes,
        output_dir_presets:        g("output_dir_presets",        "outputDirPresets")         ?? form.output_dir_presets,
        nav_layout:                g("nav_layout",                "navLayout")                ?? form.nav_layout,
        inner_nav_layout:          g("inner_nav_layout",          "innerNavLayout")           ?? form.inner_nav_layout,
      };

      // S'assurer que discord_fields contient une entrée pour chaque type
      // (rétrocompat si l'ancien fichier de config ne les avait pas)
      for (const row of NOTIF_ROWS) {
        if (!form.discord_fields[row.notifType]) {
          form.discord_fields[row.notifType] = (cat[row.notifType] ?? []).map(f => f.id);
        }
      }
      form.discord_fields = { ...form.discord_fields };

      effective = eff;
      catalog   = { ...cat };

      // Si aucun preset configuré, ajouter le dossier Téléchargements par défaut
      if (form.output_dir_presets.length === 0) {
        try {
          const dl = await downloadDir();
          if (dl) await persistPresets([dl]);
        } catch {
          // pas critique
        }
      }
    } catch (e) {
      encoder.log(`Erreur chargement config : ${e}`, "error");
    }
  }

  // ── Sauvegarde ─────────────────────────────────────────────────────────────

  async function save() {
    saving = true;
    try {
      // Envoi explicite en snake_case pour garantir la compatibilité Tauri
      await invoke("save_config", {
        config: {
          ffmpeg_path:               form.ffmpeg_path,
          dark_mode:                 form.dark_mode,
          send_email:                form.send_email,
          email_to:                  form.email_to,
          discord_bot_token:         form.discord_bot_token,
          discord_log_channel_id:    form.discord_log_channel_id,
          discord_cmd_channel_id:    form.discord_cmd_channel_id,
          discord_enabled:           form.discord_enabled,
          discord_notify_start:      form.discord_notify_start,
          discord_notify_file_done:  form.discord_notify_file_done,
          discord_notify_summary:    form.discord_notify_summary,
          discord_notify_error:      form.discord_notify_error,
          discord_notify_progress:   form.discord_notify_progress,
          discord_progress_interval: form.discord_progress_interval,
          discord_fields:            form.discord_fields,
          discord_custom_notes:      form.discord_custom_notes,
          output_dir_presets:        form.output_dir_presets,
          nav_layout:                form.nav_layout,
          inner_nav_layout:          form.inner_nav_layout,
        }
      });
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

  // ── Test Discord ───────────────────────────────────────────────────────────

  async function testDiscord() {
    testing = true;
    testResult = null;
    try {
      await invoke("send_discord_notification", {
        botToken: form.discord_bot_token,
        logChannelId: form.discord_log_channel_id,
        summary: { files: [], total_original_mb: 10.0, total_encoded_mb: 6.2, total_secs: 42.0 },
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

  // ── Helpers champs ─────────────────────────────────────────────────────────

  /** Retourne true si le champ `fieldId` est actif pour `notifType`. */
  function isFieldEnabled(notifType: string, fieldId: string): boolean {
    return form.discord_fields[notifType]?.includes(fieldId) ?? true;
  }

  /** Toggle un champ dans discord_fields[notifType]. */
  function toggleField(notifType: string, fieldId: string) {
    const current = form.discord_fields[notifType] ?? [];
    if (current.includes(fieldId)) {
      form.discord_fields[notifType] = current.filter(id => id !== fieldId);
    } else {
      form.discord_fields[notifType] = [...current, fieldId];
    }
    // Forcer la réactivité de Svelte 5 (objet nested)
    form.discord_fields = { ...form.discord_fields };
  }

  /** Active ou désactive tous les champs d'un type d'un coup. */
  function toggleAllFields(notifType: string, enable: boolean) {
    if (enable) {
      form.discord_fields[notifType] = (catalog[notifType] ?? []).map(f => f.id);
    } else {
      form.discord_fields[notifType] = [];
    }
    form.discord_fields = { ...form.discord_fields };
  }

  /** Nombre de champs actifs pour un type. */
  function activeFieldCount(notifType: string): number {
    return form.discord_fields[notifType]?.length ?? 0;
  }

  /** Nombre total de champs disponibles pour un type. */
  function totalFieldCount(notifType: string): number {
    return catalog[notifType]?.length ?? 0;
  }

  /**
   * Insère une variable dans la note personnalisée d'un type de notification,
   * à la position actuelle du curseur (ou en fin de texte si le champ n'a
   * jamais eu le focus).
   */
  function insertVariable(notifType: string, variable: string) {
    const current = form.discord_custom_notes[notifType] ?? "";
    const el = textareaRefs[notifType];

    if (!el) {
      form.discord_custom_notes[notifType] = current + variable;
      form.discord_custom_notes = { ...form.discord_custom_notes };
      return;
    }

    const start = el.selectionStart ?? current.length;
    const end   = el.selectionEnd ?? current.length;
    const next  = current.slice(0, start) + variable + current.slice(end);

    form.discord_custom_notes[notifType] = next;
    form.discord_custom_notes = { ...form.discord_custom_notes };

    // Remet le focus et replace le curseur juste après la variable insérée,
    // une fois que Svelte a mis à jour la valeur du textarea.
    const cursorPos = start + variable.length;
    requestAnimationFrame(() => {
      el.focus();
      el.setSelectionRange(cursorPos, cursorPos);
    });
  }

  // ── Effets ─────────────────────────────────────────────────────────────────

  onMount(() => loadSettings());

  // ── Chemins prédéfinis ─────────────────────────────────────────────────────

  let newPreset = $state("");

  /**
   * Construit l'objet config complet à partir du form actuel.
   * Utilisé pour la sauvegarde immédiate des presets (sans passer par le bouton).
   */
  function buildConfigPayload(overridePresets?: string[]) {
    return {
      ffmpeg_path:               form.ffmpeg_path,
      dark_mode:                 form.dark_mode,
      send_email:                form.send_email,
      email_to:                  form.email_to,
      discord_bot_token:         form.discord_bot_token,
      discord_log_channel_id:    form.discord_log_channel_id,
      discord_cmd_channel_id:    form.discord_cmd_channel_id,
      discord_enabled:           form.discord_enabled,
      discord_notify_start:      form.discord_notify_start,
      discord_notify_file_done:  form.discord_notify_file_done,
      discord_notify_summary:    form.discord_notify_summary,
      discord_notify_error:      form.discord_notify_error,
      discord_notify_progress:   form.discord_notify_progress,
      discord_progress_interval: form.discord_progress_interval,
      discord_fields:            form.discord_fields,
      discord_custom_notes:      form.discord_custom_notes,
      output_dir_presets:        overridePresets ?? form.output_dir_presets,
    };
  }

  /** Persiste les presets immédiatement (même pattern que persistPrefs dans le store). */
  async function persistPresets(presets: string[]) {
    form.output_dir_presets = presets;
    try {
      await invoke("save_config", { config: buildConfigPayload(presets) });
      // Notifier la DropZone pour qu'elle recharge depuis le store
      window.dispatchEvent(new CustomEvent("rencodex:config-saved"));
    } catch (e) {
      encoder.log(`Erreur sauvegarde presets : ${e}`, "error");
    }
  }

  async function addPreset() {
    const p = newPreset.trim();
    if (!p || form.output_dir_presets.includes(p)) return;
    newPreset = "";
    await persistPresets([...form.output_dir_presets, p]);
  }

  async function browsePreset() {
    const dir = await openDialog({ directory: true });
    if (dir && typeof dir === "string") {
      newPreset = dir;
    }
  }

  async function removePreset(path: string) {
    await persistPresets(form.output_dir_presets.filter((p) => p !== path));
  }

  async function handlePresetKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") { e.preventDefault(); await addPreset(); }
  }

  // ── Dérivés Discord ────────────────────────────────────────────────────────

  let discordFromEnv = $derived(
    effective?.discord_token_set && !form.discord_bot_token,
  );


</script>

<div class="page-root" class:page-root--horizontal={encoder.innerNavLayout === "horizontal"}>

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

    <!-- Footer : bouton sauvegarder -->
    <div class="sidebar-footer">
      <button
        onclick={save}
        disabled={saving}
        class="save-btn"
        aria-label={saving ? "Sauvegarde en cours" : "Sauvegarder les paramètres"}
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

    <!-- ── Interface ─────────────────────────────────────────── -->
    {#if activeTab === "interface"}
      <section class="content-section">
      <header class="section-header">
        <div>
          <h2 class="section-title">Interface</h2>
          <p class="section-desc">Personnalisez l'apparence de l'application.</p>
        </div>
      </header>

      <!-- Navigation globale -->
      <div>
        <span class="field-label">Navigation globale</span>
        <p class="field-hint">Disposition de la barre de navigation principale de l'application.</p>
        <div class="flex gap-2">
          <button type="button" onclick={() => { form.nav_layout = "vertical"; encoder.navLayout = "vertical"; save(); }}
            class="layout-btn {form.nav_layout === 'vertical' ? 'layout-btn--active' : ''}"
            aria-pressed={form.nav_layout === "vertical"}>
            <svg width="36" height="28" viewBox="0 0 36 28" fill="none" xmlns="http://www.w3.org/2000/svg">
              <rect x="0.5" y="0.5" width="35" height="27" rx="3.5" stroke="currentColor" stroke-opacity="0.3"/>
              <rect x="2" y="2" width="8" height="24" rx="2" fill="currentColor" fill-opacity="0.15"/>
              <rect x="3" y="4" width="6" height="2" rx="1" fill="currentColor" fill-opacity="0.5"/>
              <rect x="3" y="8" width="6" height="2" rx="1" fill="currentColor" fill-opacity="0.5"/>
              <rect x="3" y="12" width="6" height="2" rx="1" fill="currentColor" fill-opacity="0.5"/>
            </svg>
            <span>Verticale</span>
          </button>
          <button type="button" onclick={() => { form.nav_layout = "horizontal"; encoder.navLayout = "horizontal"; save(); }}
            class="layout-btn {form.nav_layout === 'horizontal' ? 'layout-btn--active' : ''}"
            aria-pressed={form.nav_layout === "horizontal"}>
            <svg width="36" height="28" viewBox="0 0 36 28" fill="none" xmlns="http://www.w3.org/2000/svg">
              <rect x="0.5" y="0.5" width="35" height="27" rx="3.5" stroke="currentColor" stroke-opacity="0.3"/>
              <rect x="2" y="2" width="32" height="7" rx="2" fill="currentColor" fill-opacity="0.15"/>
              <rect x="4" y="3.5" width="5" height="2" rx="1" fill="currentColor" fill-opacity="0.5"/>
              <rect x="11" y="3.5" width="5" height="2" rx="1" fill="currentColor" fill-opacity="0.5"/>
              <rect x="18" y="3.5" width="5" height="2" rx="1" fill="currentColor" fill-opacity="0.5"/>
            </svg>
            <span>Horizontale</span>
          </button>
        </div>
      </div>

      <!-- Navigation interne -->
      <div>
        <span class="field-label">Navigation des pages</span>
        <p class="field-hint">Disposition des menus de navigation dans chaque page (Encodage, Renommage, etc.).</p>
        <div class="flex gap-2">
          <button type="button" onclick={() => { form.inner_nav_layout = "vertical"; encoder.innerNavLayout = "vertical"; save(); }}
            class="layout-btn {form.inner_nav_layout === 'vertical' ? 'layout-btn--active' : ''}"
            aria-pressed={form.inner_nav_layout === "vertical"}>
            <svg width="36" height="28" viewBox="0 0 36 28" fill="none" xmlns="http://www.w3.org/2000/svg">
              <rect x="0.5" y="0.5" width="35" height="27" rx="3.5" stroke="currentColor" stroke-opacity="0.3"/>
              <rect x="2" y="2" width="8" height="24" rx="2" fill="currentColor" fill-opacity="0.15"/>
              <rect x="3" y="4" width="6" height="2" rx="1" fill="currentColor" fill-opacity="0.5"/>
              <rect x="3" y="8" width="6" height="2" rx="1" fill="currentColor" fill-opacity="0.5"/>
              <rect x="3" y="12" width="6" height="2" rx="1" fill="currentColor" fill-opacity="0.5"/>
            </svg>
            <span>Verticale</span>
          </button>
          <button type="button" onclick={() => { form.inner_nav_layout = "horizontal"; encoder.innerNavLayout = "horizontal"; save(); }}
            class="layout-btn {form.inner_nav_layout === 'horizontal' ? 'layout-btn--active' : ''}"
            aria-pressed={form.inner_nav_layout === "horizontal"}>
            <svg width="36" height="28" viewBox="0 0 36 28" fill="none" xmlns="http://www.w3.org/2000/svg">
              <rect x="0.5" y="0.5" width="35" height="27" rx="3.5" stroke="currentColor" stroke-opacity="0.3"/>
              <rect x="2" y="2" width="32" height="7" rx="2" fill="currentColor" fill-opacity="0.15"/>
              <rect x="4" y="3.5" width="5" height="2" rx="1" fill="currentColor" fill-opacity="0.5"/>
              <rect x="11" y="3.5" width="5" height="2" rx="1" fill="currentColor" fill-opacity="0.5"/>
              <rect x="18" y="3.5" width="5" height="2" rx="1" fill="currentColor" fill-opacity="0.5"/>
            </svg>
            <span>Horizontale</span>
          </button>
        </div>
      </div>

      <!-- Mode sombre / clair -->
      <div>
        <span class="field-label">Apparence</span>
        <div class="mode-toggle-row">
          <button
            type="button"
            onclick={() => { if (!theme.dark) theme.toggle(); }}
            class="mode-btn {theme.dark ? 'mode-btn--active' : ''}"
            aria-pressed={theme.dark}
          >
            <Moon class="w-3.5 h-3.5" />
            Sombre
          </button>
          <button
            type="button"
            onclick={() => { if (theme.dark) theme.toggle(); }}
            class="mode-btn {!theme.dark ? 'mode-btn--active' : ''}"
            aria-pressed={!theme.dark}
          >
            <Sun class="w-3.5 h-3.5" />
            Clair
          </button>
        </div>
      </div>

      <!-- Thèmes prédéfinis -->
      <div>
        <span class="field-label">Thèmes {theme.dark ? "sombres" : "clairs"}</span>
        <div class="combo-grid">
          {#each (theme.dark ? DARK_COMBOS : LIGHT_COMBOS) as combo}
            {@const isActive =
              theme.backgroundColor.toLowerCase() === combo.bg &&
              theme.accent.toLowerCase() === combo.accent}
            <button
              type="button"
              onclick={() => theme.applyCombo(combo)}
              class="combo-btn {isActive ? 'combo-btn--active' : ''}"
              title={combo.label}
              aria-pressed={isActive}
            >
              <span class="combo-preview">
                <span class="combo-bg"    style="background: {combo.bg};"></span>
                <span class="combo-accent" style="background: {combo.accent};"></span>
              </span>
              <span class="combo-label">{combo.label}</span>
            </button>
          {/each}
        </div>
      </div>

      <!-- Couleur primaire -->
      <div>
        <span class="field-label flex items-center gap-1.5">
          <Palette class="w-3 h-3" />
          Couleur primaire
        </span>
        <div class="flex items-center gap-2 flex-wrap">
          <label class="color-swatch-input" style="background: {theme.accent};" title="Choisir une couleur personnalisée">
            <input
              type="color"
              value={theme.accent}
              oninput={(e) => theme.setAccent((e.currentTarget as HTMLInputElement).value)}
              aria-label="Couleur primaire personnalisée"
            />
          </label>
          <input
            type="text"
            value={theme.accent}
            onchange={(e) => theme.setAccent((e.currentTarget as HTMLInputElement).value.trim())}
            placeholder="#e07b39"
            maxlength="7"
            class="field-input px-2 py-1.5 w-24 text-center"
            aria-label="Code hexadécimal de la couleur primaire"
          />
          {#if theme.accent.toLowerCase() !== (theme.dark ? theme.defaultAccentDark : theme.defaultAccentLight)}
            <button type="button" onclick={() => theme.resetAccent()} class="icon-btn-inline" title="Réinitialiser">
              <RotateCcw class="w-3.5 h-3.5" />
            </button>
          {/if}
        </div>
      </div>

      <!-- Couleur de fond -->
      <div>
        <span class="field-label flex items-center gap-1.5">
          <Palette class="w-3 h-3" />
          Couleur de fond
        </span>
        <div class="flex items-center gap-2 flex-wrap">
          <label class="color-swatch-input" style="background: {theme.backgroundColor};" title="Choisir une couleur de fond">
            <input
              type="color"
              value={theme.backgroundColor}
              oninput={(e) => theme.setBackground((e.currentTarget as HTMLInputElement).value)}
              aria-label="Couleur de fond personnalisée"
            />
          </label>
          <input
            type="text"
            value={theme.backgroundColor}
            onchange={(e) => theme.setBackground((e.currentTarget as HTMLInputElement).value.trim())}
            placeholder="#1d1d1d"
            maxlength="7"
            class="field-input px-2 py-1.5 w-24 text-center"
            aria-label="Code hexadécimal du fond"
          />
          {#if theme.backgroundColor.toLowerCase() !== (theme.dark ? theme.defaultBgDark : theme.defaultBgLight)}
            <button type="button" onclick={() => theme.resetBackground()} class="icon-btn-inline" title="Réinitialiser">
              <RotateCcw class="w-3.5 h-3.5" />
            </button>
          {/if}
        </div>
      </div>

      <button
        onclick={handleReset}
        disabled={encoder.encoding || encoder.extractingSubs}
        class="w-full btn font-mono text-[11px] justify-center gap-1.5"
        class:btn-danger={confirmReset}
        aria-label={confirmReset ? "Confirmer la réinitialisation" : "Réinitialiser aux valeurs par défaut"}
      >
        <RotateCcw class="w-3.5 h-3.5 shrink-0" />
        {confirmReset ? "CONFIRMER LA RÉINITIALISATION ?" : "RÉINITIALISER LES PARAMÈTRES"}
      </button>
    </section>

    <!-- ── FFmpeg ──────────────────────────────────────────────── -->
    {:else if activeTab === "ffmpeg"}
      <section class="content-section">
      <header class="section-header">
        <div>
          <h2 class="section-title">FFmpeg</h2>
          <p class="section-desc">Chemin vers le binaire ffmpeg utilisé pour l'encodage.</p>
        </div>
      </header>
      <div>
        <label for="ffmpeg-path" class="field-label">Chemin vers ffmpeg.exe</label>
        <input
          id="ffmpeg-path"
          type="text"
          bind:value={form.ffmpeg_path}
          placeholder="C:\Outil\ffmpeg\bin\ffmpeg.exe"
          class="field-input w-full px-3 py-2"
        />
      </div>
    </section>

    <!-- ── Chemins ──────────────────────────────────────────────── -->
    {:else if activeTab === "presets"}
      <section class="content-section">
      <header class="section-header">
        <div>
          <h2 class="section-title">Chemins de sortie</h2>
          <p class="section-desc">Ces chemins apparaîtront dans le menu de la DropZone, en plus de l'historique récent.</p>
        </div>
      </header>

      <!-- Champ d'ajout -->
      <div class="flex gap-2 mb-2">
        <input
          type="text"
          bind:value={newPreset}
          onkeydown={handlePresetKeydown}
          placeholder="D:\Encodage\Sorties"
          class="field-input flex-1 px-3 py-2"
          aria-label="Nouveau chemin prédéfini"
        />
        <button
          type="button"
          onclick={browsePreset}
          class="btn btn-secondary font-mono text-[11px] px-3"
          aria-label="Parcourir"
          title="Choisir un dossier"
        >
          <FolderInput class="w-3.5 h-3.5" />
        </button>
        <button
          type="button"
          onclick={addPreset}
          disabled={!newPreset.trim()}
          class="btn btn-primary font-mono text-[11px] px-3"
          aria-label="Ajouter le chemin"
          title="Ajouter"
        >
          <Check class="w-3.5 h-3.5" />
        </button>
      </div>

      <!-- Liste des presets -->
      {#if form.output_dir_presets.length > 0}
        <div class="space-y-1" role="list" aria-label="Chemins prédéfinis">
          {#each form.output_dir_presets as preset, i}
            <div
              class="flex items-center gap-2 px-3 py-2 rounded-[var(--radius-sm)]"
              style="background: var(--color-surface); border: 1px solid var(--color-border);"
              role="listitem"
            >
              <span
                class="font-mono text-[10px] flex-1 min-w-0 truncate"
                style="color: var(--color-text); direction: rtl; text-align: left;"
                title={preset}
              >{preset}</span>
              <button
                type="button"
                onclick={() => removePreset(preset)}
                class="icon-btn-inline shrink-0"
                aria-label="Supprimer ce chemin"
              >
                <X class="w-3 h-3" />
              </button>
            </div>
          {/each}
        </div>
      {:else}
        <p class="font-mono text-[10px] text-center py-2" style="color: var(--color-subtext); opacity: 0.5;">
          Aucun chemin prédéfini
        </p>
      {/if}
    </section>

    <!-- ── Discord ──────────────────────────────────────────────── -->
    {:else if activeTab === "discord"}
      <section class="content-section">
      <header class="section-header">
        <div>
          <h2 class="section-title">Discord</h2>
          <p class="section-desc">Configurez le bot Discord pour recevoir des notifications d'encodage.</p>
        </div>
        <label class="toggle-row">
          <span class="font-mono text-[10px]" style="color: var(--color-subtext);">Activer</span>
          <button
            type="button"
            role="switch"
            aria-checked={form.discord_enabled}
            aria-label={form.discord_enabled ? "Désactiver Discord" : "Activer Discord"}
            onclick={() => (form.discord_enabled = !form.discord_enabled)}
            class="toggle {form.discord_enabled ? 'on' : ''}"
          ></button>
        </label>
      </header>

      {#if form.discord_enabled}

        {#if discordFromEnv}
          <div class="flex items-center gap-2 font-mono text-[10px] px-3 py-2 rounded-[var(--radius-sm)]"
               style="color: var(--color-success); background: color-mix(in srgb, var(--color-success) 10%, transparent); border: 1px solid color-mix(in srgb, var(--color-success) 20%, transparent);"
               role="status">
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
          <input id="discord-log-channel" type="text" bind:value={form.discord_log_channel_id}
                 placeholder="ID du salon" class="field-input w-full px-3 py-2" />
        </div>
        <div>
          <label for="discord-cmd-channel" class="field-label">Salon de commandes</label>
          <input id="discord-cmd-channel" type="text" bind:value={form.discord_cmd_channel_id}
                 placeholder="ID du salon" class="field-input w-full px-3 py-2" />
        </div>

        <!-- ── Notifications ──────────────────────────────────────────────── -->
        <div class="pt-3" style="border-top: 1px solid var(--color-border);">
          <div class="section-label mb-2">Notifications</div>

          <div class="notif-grid">
          {#each NOTIF_ROWS as row}
            {@const isOn = (form as any)[row.key] as boolean}
            {@const hasFields = (catalog[row.notifType] ?? []).length > 0}
            {@const active = activeFieldCount(row.notifType)}
            {@const total  = totalFieldCount(row.notifType)}
            {@const note   = form.discord_custom_notes[row.notifType] ?? ""}

            <div class="discord-card" class:discord-card--off={!isOn}>

              <!-- En-tête de la card : icône, libellé, badge, toggle -->
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
                    onclick={() => { (form as any)[row.key] = !isOn; }}
                    class="toggle {isOn ? 'on' : ''}"
                    aria-label={row.label}
                  ></button>
                </div>
              </div>

              <!-- Intervalle progression -->
              {#if row.key === 'discord_notify_progress' && isOn}
                <div class="discord-card-subfield">
                  <label for="discord-progress-interval" class="field-label">Intervalle (secondes)</label>
                  <input
                    id="discord-progress-interval"
                    type="number"
                    bind:value={form.discord_progress_interval}
                    min="10" max="300"
                    placeholder="30"
                    class="field-input w-full px-3 py-2"
                  />
                </div>
              {/if}

              <!-- Corps de la card : toujours visible, pas de chevron -->
              {#if hasFields}
                <div class="discord-card-body">

                  <!-- En-tête corps avec tout cocher/décocher -->
                  <div class="fields-panel-header">
                    <span class="font-mono text-[9px] uppercase tracking-wider"
                          style="color: var(--color-subtext);">Champs inclus dans l'embed</span>
                    <div class="flex gap-2">
                      <button
                        type="button"
                        class="micro-btn"
                        onclick={() => toggleAllFields(row.notifType, true)}
                        aria-label="Tout activer"
                      >Tout</button>
                      <button
                        type="button"
                        class="micro-btn"
                        onclick={() => toggleAllFields(row.notifType, false)}
                        aria-label="Tout désactiver"
                      >Aucun</button>
                    </div>
                  </div>

                  <!-- Liste des champs -->
                  <div class="fields-list">
                    {#each (catalog[row.notifType] ?? []) as field}
                      {@const enabled = isFieldEnabled(row.notifType, field.id)}
                      <button
                        type="button"
                        class="field-chip"
                        class:field-chip--on={enabled}
                        aria-pressed={enabled}
                        aria-label={field.label}
                        onclick={() => toggleField(row.notifType, field.id)}
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
                    <label class="font-mono text-[9px] uppercase tracking-wider"
                           style="color: var(--color-subtext);"
                           for="custom-note-{row.notifType}">
                      📝 Note personnalisée
                    </label>
                    <textarea
                      id="custom-note-{row.notifType}"
                      bind:this={textareaRefs[row.notifType]}
                      rows="2"
                      maxlength="500"
                      placeholder="Texte libre affiché en bas de l'embed (optionnel)…"
                      class="custom-note-textarea"
                      value={note}
                      oninput={(e) => {
                        form.discord_custom_notes[row.notifType] = (e.currentTarget as HTMLTextAreaElement).value;
                        form.discord_custom_notes = { ...form.discord_custom_notes };
                      }}
                    ></textarea>
                    <div class="custom-note-footer">
                      <div class="custom-note-vars">
                        {#each (NOTIF_VARS[row.notifType] ?? []) as v}
                          <button
                            type="button"
                            class="var-chip"
                            title="Cliquer pour insérer {v} dans la note"
                            onclick={() => insertVariable(row.notifType, v)}
                          >{v}</button>
                        {/each}
                      </div>
                      <div class="custom-note-counter">
                        {note.length}/500
                      </div>
                    </div>
                  </div>

                  <!-- Aperçu visuel embed Discord -->
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
                          {#each (catalog[row.notifType] ?? []) as field}
                            {#if isFieldEnabled(row.notifType, field.id)}
                              <div class="embed-mock-field">
                                <div class="embed-mock-field-name">{field.label}</div>
                                <div class="embed-mock-field-value">—</div>
                              </div>
                            {/if}
                          {/each}
                          {#if active === 0}
                            <span class="font-mono text-[9px]" style="color: var(--color-subtext);">
                              Aucun champ — embed vide
                            </span>
                          {/if}
                          {#if note.trim()}
                            <div class="embed-mock-field" style="grid-column: 1 / -1;">
                              <div class="embed-mock-field-name">📝 Note</div>
                              <div class="embed-mock-field-value" style="white-space: pre-wrap; word-break: break-word;">
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
          {/each}
          </div>
        </div>


        <!-- Test Discord -->
        <button
          onclick={testDiscord}
          disabled={testing || (!form.discord_bot_token && !effective?.discord_token_set) || !form.discord_log_channel_id.trim()}
          class="w-full btn font-mono text-[11px] justify-center"
          class:btn-primary={testResult === "ok"}
          class:btn-danger={testResult === "error"}
          aria-label={testing ? "Test en cours" : testResult === "ok" ? "Test réussi" : testResult === "error" ? "Test échoué" : "Tester la notification Discord"}
        >
          {#if testing}
            <span class="spinner w-3 h-3 border-2 border-current/30 border-t-current shrink-0 rounded-full animate-spin"></span>
            ENVOI…
          {:else if testResult === "ok"}<Check class="w-3.5 h-3.5" /> TEST ENVOYÉ
          {:else if testResult === "error"}<X class="w-3.5 h-3.5" /> ÉCHEC
          {:else}TESTER LA NOTIFICATION{/if}
        </button>

      {/if}
    </section>

    <!-- ── Env ──────────────────────────────────────────────── -->
    {:else if activeTab === "env"}
      <section class="content-section">
      <header class="section-header">
        <div>
          <h2 class="section-title">Variables d'environnement</h2>
          <p class="section-desc">Surcharge de la configuration via les variables système détectées au démarrage.</p>
        </div>
      </header>
      <div class="space-y-1" role="list" aria-label="Variables d'environnement actives">
        {#each [
          { key: 'RENCODEX_FFMPEG_PATH',         active: effective?.ffmpeg_path !== form.ffmpeg_path },
          { key: 'RENCODEX_DISCORD_TOKEN',        active: effective?.discord_token_set },
          { key: 'RENCODEX_DISCORD_LOG_CHANNEL',  active: effective?.discord_log_channel_id !== form.discord_log_channel_id },
          { key: 'RENCODEX_DISCORD_CMD_CHANNEL',  active: effective?.discord_cmd_channel_id !== form.discord_cmd_channel_id },
        ] as v}
          <div class="flex items-center justify-between px-1 py-2 rounded-[var(--radius-sm)]"
               style="background: var(--color-surface); border: 1px solid var(--color-border);"
               role="listitem">
            <span class="font-mono text-[10px]" style="color: var(--color-subtext);">{v.key}</span>
            <span class="font-mono text-[10px] inline-flex items-center gap-1"
                  style="color: {v.active ? 'var(--color-success)' : 'var(--color-subtext2)'};"
                  aria-label={v.active ? "Variable définie" : "Variable non définie"}>
              {#if v.active}<Check class="w-3 h-3" />définie{:else}—{/if}
            </span>
          </div>
        {/each}
      </div>
    </section>

    {/if}
    </div><!-- end content-inner -->
  </div><!-- end content-panel -->

</div><!-- end page-root -->

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
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text);
    letter-spacing: -0.01em;
  }
  .sidebar-sub {
    display: block;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    letter-spacing: 0.06em;
    color: var(--color-subtext2);
    margin-top: 3px;
    text-transform: uppercase;
  }

  .sidebar-nav {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .nav-item {
    position: relative;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid transparent;
    background: transparent;
    cursor: pointer;
    text-align: left;
    transition: background 0.1s, border-color 0.1s;
    width: 100%;
  }
  .nav-item:hover { background: color-mix(in srgb, var(--color-muted) 30%, transparent); }
  .nav-item--active {
    background: color-mix(in srgb, var(--color-accent) 9%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 22%, var(--color-border));
  }

  .nav-item-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-xs);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    color: var(--color-subtext);
    flex-shrink: 0;
    transition: background 0.1s, color 0.1s, border-color 0.1s;
  }
  .nav-item--active .nav-item-icon {
    background: color-mix(in srgb, var(--color-accent) 12%, var(--color-surface));
    border-color: color-mix(in srgb, var(--color-accent) 30%, var(--color-border));
    color: var(--color-accent);
  }

  .nav-item-text {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
    flex: 1;
  }
  .nav-item-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--color-subtext);
    line-height: 1.2;
    transition: color 0.1s;
  }
  .nav-item--active .nav-item-label {
    color: var(--color-accent);
    font-weight: 600;
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
    height: 18px;
    border-radius: 2px 0 0 2px;
    background: var(--color-accent);
  }

  /* Sidebar footer */
  .sidebar-footer {
    padding: 12px;
    border-top: 1px solid var(--color-border);
    flex-shrink: 0;
  }
  .save-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    width: 100%;
    padding: 9px 12px;
    border-radius: var(--radius-sm);
    border: 1px solid color-mix(in srgb, var(--color-accent) 35%, var(--color-border));
    background: color-mix(in srgb, var(--color-accent) 12%, transparent);
    color: var(--color-accent);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s, border-color 0.15s;
  }
  .save-btn:hover {
    background: color-mix(in srgb, var(--color-accent) 20%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 50%, var(--color-border));
  }
  .save-btn:disabled { opacity: 0.5; cursor: not-allowed; }

  .spinner {
    display: inline-block;
    width: 12px;
    height: 12px;
    border: 2px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    flex-shrink: 0;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  /* ── Content panel ───────────────────────────────────────────────────────── */

  .content-panel {
    flex: 1;
    overflow-y: auto;
  }

  .content-inner {
    max-width: 560px;
  }

  .content-section {
    padding: 28px 32px;
  }

  .section-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 24px;
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

  /* ── Icônes ───────────────────────────────────────────────────────────────── */

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
  .icon-btn-inline:hover { color: var(--color-text); }

  /* ── Couleur primaire ─────────────────────────────────────────────────────── */

  .color-swatch-input {
    position: relative;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    cursor: pointer;
    flex-shrink: 0;
    overflow: hidden;
    transition: border-color 0.12s, transform 0.12s;
  }
  .color-swatch-input:hover { border-color: var(--color-border2); transform: scale(1.04); }
  .color-swatch-input input[type="color"] {
    position: absolute;
    inset: -4px;
    width: calc(100% + 8px);
    height: calc(100% + 8px);
    border: none;
    padding: 0;
    cursor: pointer;
    opacity: 0;
  }

  /* ── Formulaire ───────────────────────────────────────────────────────────── */

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
  .field-input:focus    { border-color: var(--color-accent); }
  .field-input:disabled { opacity: 0.5; cursor: not-allowed; }

  .toggle-row {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }

  /* ── Grille 2 colonnes ──────────────────────────────────────────────────────── */

  .notif-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px;
    align-items: start;
  }

  /* ── Card de notification ──────────────────────────────────────────────────── */

  .discord-card {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-panel2, var(--color-panel));
    overflow: hidden;
    transition: opacity 0.15s, border-color 0.15s;
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

  /* Badge champs actifs */
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
    background: color-mix(in srgb, var(--color-danger, #E74C3C) 15%, transparent);
    color: var(--color-danger, #E74C3C);
    border-color: color-mix(in srgb, var(--color-danger, #E74C3C) 25%, transparent);
  }

  /* ── Corps de la card : champs, note, aperçu (toujours visible) ─────────────── */

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

  /* Boutons Tout / Aucun */
  .micro-btn {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    padding: 2px 6px;
    border-radius: var(--radius-xs);
    border: 1px solid var(--color-border);
    background: var(--color-panel2, var(--color-panel));
    color: var(--color-subtext);
    cursor: pointer;
    transition: background 0.1s, color 0.1s;
  }
  .micro-btn:hover {
    background: var(--color-border);
    color: var(--color-text);
  }

  /* Liste des chips de champs */
  .fields-list {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
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
    transition: background 0.1s, transform 0.08s;
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
  .custom-note-textarea:focus { border-color: var(--color-accent); }
  .custom-note-textarea::placeholder { color: var(--color-subtext2); }
  .custom-note-counter {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext2);
    text-align: right;
  }

  /* Chip individuel */
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
    transition: background 0.1s, color 0.1s, border-color 0.1s;
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

  /* ── Aperçu embed Discord ──────────────────────────────────────────────────── */

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
  .field-hint {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
    margin: 2px 0 6px;
    opacity: 0.7;
  }

  /* ── Layout picker ─────────────────────────────────────────────────────── */
  .layout-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 5px;
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-subtext);
    cursor: pointer;
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    transition: border-color 0.15s, color 0.15s, background 0.15s;
  }
  .layout-btn:hover {
    border-color: var(--color-accent);
    color: var(--color-text);
  }
  .layout-btn--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 10%, transparent);
    color: var(--color-accent);
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
  .page-root--horizontal .sidebar-header {
    display: none;
  }
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
    display: flex;
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

  /* ── Mode toggle ────────────────────────────────────────────────────────── */
  .mode-toggle-row {
    display: flex;
    gap: 6px;
    margin-top: 6px;
  }
  .mode-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 14px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-subtext);
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s, color 0.15s, border-color 0.15s;
  }
  .mode-btn:hover {
    border-color: var(--color-subtext2);
    color: var(--color-text);
  }
  .mode-btn--active {
    background: color-mix(in srgb, var(--color-accent) 12%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 35%, transparent);
    color: var(--color-accent);
  }

  /* ── Combo presets ──────────────────────────────────────────────────────── */
  .combo-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 6px;
  }
  .combo-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 6px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    transition: border-color 0.15s, transform 0.1s;
  }
  .combo-btn:hover {
    border-color: var(--color-subtext2);
    transform: translateY(-1px);
  }
  .combo-btn--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 8%, transparent);
  }
  .combo-preview {
    width: 40px;
    height: 26px;
    border-radius: var(--radius-xs);
    overflow: hidden;
    display: flex;
    border: 1px solid color-mix(in srgb, var(--color-border) 60%, transparent);
  }
  .combo-bg     { flex: 3; }
  .combo-accent { flex: 1; }
  .combo-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    white-space: nowrap;
  }
  .combo-btn--active .combo-label { color: var(--color-accent); }
</style>