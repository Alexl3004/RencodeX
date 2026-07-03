<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { downloadDir } from "@tauri-apps/api/path";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { encoder } from "$lib/stores/encoder.svelte";
  import { theme } from "$lib/stores/theme.svelte";
  import { fly } from "svelte/transition";
  import { cubicOut, cubicIn } from "svelte/easing";
  import { Eye, EyeOff, X, Check, ChevronDown, ChevronRight, RotateCcw, FolderInput, Monitor, Cpu, FolderOpen, MessageSquare, Terminal, Palette } from '@lucide/svelte';

  type TabId = "interface" | "ffmpeg" | "presets" | "discord" | "env";
  let activeTab = $state<TabId>("interface");

  const TABS: { id: TabId; label: string; icon: any }[] = [
    { id: "interface", label: "Interface",  icon: Monitor },
    { id: "ffmpeg",    label: "FFmpeg",     icon: Cpu },
    { id: "presets",   label: "Chemins",    icon: FolderOpen },
    { id: "discord",   label: "Discord",    icon: MessageSquare },
    { id: "env",       label: "Env",        icon: Terminal },
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
    /** Champs activés par type : { "start": ["files","size","crf"], … } */
    discord_fields: Record<string, string[]>;
    /** Chemins de sortie prédéfinis */
    output_dir_presets: string[];
  };

  // ── Props & état ───────────────────────────────────────────────────────────

  let {
    open = $bindable(false),
  } = $props();

  let saving    = $state(false);
  let testing   = $state(false);
  let testResult = $state<"ok" | "error" | null>(null);
  let effective  = $state<EffectiveConfig | null>(null);
  let showToken  = $state(false);
  let confirmReset = $state(false);

  /** Catalogue reçu du backend : { notifType -> FieldDef[] } */
  let catalog = $state<Record<string, FieldDef[]>>({});

  /** Quel panneau de champs est déplié : null = aucun, sinon notifType */
  let expandedFields = $state<string | null>(null);

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
    output_dir_presets: [],
  });

  // ── Définition statique des notifications (labels UI) ─────────────────────

  const NOTIF_ROWS: Array<{
    key: keyof SavedConfig;
    notifType: string;
    label: string;
    desc: string;
  }> = [
    { key: "discord_notify_start",     notifType: "start",     label: "Début d'encodage",  desc: "Envoi au démarrage" },
    { key: "discord_notify_file_done", notifType: "file_done", label: "Fichier terminé",   desc: "Après chaque fichier" },
    { key: "discord_notify_progress",  notifType: "progress",  label: "Progression",       desc: "Mise à jour périodique" },
    { key: "discord_notify_summary",   notifType: "summary",   label: "Résumé global",     desc: "Bilan de session" },
    { key: "discord_notify_error",     notifType: "error",     label: "Erreur d'encodage", desc: "En cas d'échec" },
  ];

  // ── Réinitialisation ───────────────────────────────────────────────────────

  function handleReset() {
    if (!confirmReset) {
      confirmReset = true;
      setTimeout(() => (confirmReset = false), 3000);
      return;
    }
    confirmReset = false;
    open = false;
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
        output_dir_presets:        g("output_dir_presets",        "outputDirPresets")         ?? form.output_dir_presets,
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
          output_dir_presets:        form.output_dir_presets,
        }
      });
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

  // ── Effets ─────────────────────────────────────────────────────────────────

  $effect(() => {
    if (open) loadSettings();
  });

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

  const flyParams = { x: 400, duration: 320, easing: cubicOut };
</script>

{#if open}
<!-- Backdrop -->
<div
  class="settings-backdrop"
  role="presentation"
  onclick={() => (open = false)}
  onkeydown={(e) => { if (e.key === 'Escape') open = false; }}
></div>

<!-- Drawer -->
<div
  class="settings-panel"
  role="dialog"
  aria-modal="true"
  aria-label="Paramètres"
  tabindex="-1"
  in:fly={{ x: 400, duration: 320, easing: cubicOut }}
  out:fly={{ x: 400, duration: 220, easing: cubicIn }}
>
  <!-- Header -->
  <div class="drawer-header">
    <div class="flex items-center gap-2">
      <div class="w-[3px] h-4 rounded-[1px]" style="background: var(--color-accent);"></div>
      <span class="font-mono text-[12px] font-semibold uppercase tracking-wider" style="color: var(--color-text);">Paramètres</span>
    </div>
    <button onclick={() => (open = false)} class="icon-btn" title="Fermer" aria-label="Fermer">
      <X class="w-4 h-4" />
    </button>
  </div>

  <!-- Body : sidebar nav + content -->
  <div class="drawer-body">

    <!-- Sidebar nav -->
    <nav class="drawer-nav" aria-label="Navigation paramètres">
      {#each TABS as tab}
        <button
          type="button"
          onclick={() => (activeTab = tab.id)}
          class="nav-item {activeTab === tab.id ? 'nav-item--active' : ''}"
          aria-current={activeTab === tab.id ? "page" : undefined}
        >
          {#if tab.id === 'interface'}<Monitor class="w-3.5 h-3.5 shrink-0" />
          {:else if tab.id === 'ffmpeg'}<Cpu class="w-3.5 h-3.5 shrink-0" />
          {:else if tab.id === 'presets'}<FolderOpen class="w-3.5 h-3.5 shrink-0" />
          {:else if tab.id === 'discord'}<MessageSquare class="w-3.5 h-3.5 shrink-0" />
          {:else if tab.id === 'env'}<Terminal class="w-3.5 h-3.5 shrink-0" />
          {/if}
          <span class="nav-label">{tab.label}</span>
        </button>
      {/each}
    </nav>

    <!-- Tab content -->
    <div class="drawer-content">

    <!-- ── Interface ─────────────────────────────────────────── -->
    {#if activeTab === "interface"}
      <section class="space-y-3">
      <div class="section-label pb-2" style="border-bottom: 1px solid var(--color-border);">Interface</div>

      <!-- Couleur primaire -->
      <div>
        <span class="field-label flex items-center gap-1.5">
          <Palette class="w-3 h-3" />
          Couleur primaire
        </span>
        <div class="flex items-center gap-2">
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
          <div class="flex items-center gap-1.5">
            {#each ["#e07b39", "#4d8fbb", "#5fb37b", "#b25fd1", "#d14d6c", "#c9b13a"] as preset}
              <button
                type="button"
                class="color-swatch-preset {theme.accent.toLowerCase() === preset ? 'color-swatch-preset--active' : ''}"
                style="background: {preset};"
                onclick={() => theme.setAccent(preset)}
                aria-label="Utiliser la couleur {preset}"
                title={preset}
              ></button>
            {/each}
          </div>
          {#if theme.accent.toLowerCase() !== theme.defaultAccent}
            <button
              type="button"
              onclick={() => theme.resetAccent()}
              class="icon-btn-inline"
              title="Réinitialiser la couleur par défaut"
              aria-label="Réinitialiser la couleur par défaut"
            >
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
      <section class="space-y-3">
      <div class="section-label pb-2" style="border-bottom: 1px solid var(--color-border);">FFmpeg</div>
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
      <section class="space-y-3">
      <div class="section-label pb-2" style="border-bottom: 1px solid var(--color-border);">
        Chemins de sortie prédéfinis
      </div>
      <p class="font-mono text-[10px]" style="color: var(--color-subtext); line-height: 1.5;">
        Ces chemins apparaîtront dans le menu de la DropZone, en plus de l'historique récent.
      </p>

      <!-- Champ d'ajout -->
      <div class="flex gap-2">
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
      <section class="space-y-3">
      <div class="flex items-center justify-between pb-2" style="border-bottom: 1px solid var(--color-border);">
        <div class="section-label">Discord</div>
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
      </div>

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
        <div class="space-y-0 pt-3" style="border-top: 1px solid var(--color-border);">
          <div class="section-label mb-2">Notifications</div>

          {#each NOTIF_ROWS as row}
            {@const isOn = (form as any)[row.key] as boolean}
            {@const hasFields = (catalog[row.notifType] ?? []).length > 0}
            {@const active = activeFieldCount(row.notifType)}
            {@const total  = totalFieldCount(row.notifType)}
            {@const isExpanded = expandedFields === row.notifType}

            <!-- Ligne toggle principale -->
            <div class="notif-row"
                 style="border-bottom: 1px solid color-mix(in srgb, var(--color-border) 50%, transparent);">
              <div class="flex items-center gap-2 min-w-0">
                <!-- Chevron dépliage champs (uniquement si la notif est activée) -->
                {#if isOn && hasFields}
                  <button
                    type="button"
                    class="chevron-btn"
                    aria-label={isExpanded ? "Masquer les champs" : "Configurer les champs"}
                    onclick={() => (expandedFields = isExpanded ? null : row.notifType)}
                  >
                    {#if isExpanded}
                      <ChevronDown class="w-3.5 h-3.5" />
                    {:else}
                      <ChevronRight class="w-3.5 h-3.5" />
                    {/if}
                  </button>
                {:else}
                  <!-- Placeholder pour aligner -->
                  <span class="w-4 h-4 shrink-0"></span>
                {/if}

                <div class="min-w-0">
                  <span class="text-[12px]" style="color: var(--color-text);">{row.label}</span>
                  <p class="font-mono text-[9px] mt-0.5" style="color: var(--color-subtext);">{row.desc}</p>
                </div>
              </div>

              <div class="flex items-center gap-2 shrink-0">
                <!-- Badge champs actifs/total (affiché si la notif est ON et qu'il y a des champs) -->
                {#if isOn && hasFields}
                  <span
                    class="field-badge"
                    class:field-badge--warn={active === 0}
                    title="{active}/{total} champs actifs"
                  >
                    {active}/{total}
                  </span>
                {/if}
                <!-- Toggle ON/OFF -->
                <button
                  type="button"
                  role="switch"
                  aria-checked={isOn}
                  onclick={() => {
                    (form as any)[row.key] = !isOn;
                    // Replier le panneau si on désactive
                    if (isOn && expandedFields === row.notifType) expandedFields = null;
                  }}
                  class="toggle {isOn ? 'on' : ''}"
                  aria-label={row.label}
                ></button>
              </div>
            </div>

            <!-- Intervalle progression -->
            {#if row.key === 'discord_notify_progress' && isOn}
              <div class="pb-2 pt-1 pl-6">
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

            <!-- Panneau de champs (déplié) -->
            {#if isOn && hasFields && isExpanded}
              <div class="fields-panel" role="group" aria-label="Champs de la notification {row.label}">

                <!-- En-tête panneau avec tout cocher/décocher -->
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

                <!-- Aperçu visuel embed Discord -->
                <div class="embed-preview" aria-label="Aperçu de l'embed Discord">
                  <div class="embed-preview-label">Aperçu embed</div>
                  <div class="embed-mock">
                    <div class="embed-mock-bar"></div>
                    <div class="embed-mock-body">
                      <div class="embed-mock-title">
                        {row.notifType === 'start'     ? '🎬 Encodage démarré'    :
                         row.notifType === 'file_done' ? '✅ Fichier encodé'       :
                         row.notifType === 'progress'  ? '📈 Encodage en cours'   :
                         row.notifType === 'summary'   ? '✅ Encodage terminé'    :
                                                         '❌ Erreur d\'encodage'}
                      </div>
                      <div class="embed-mock-fields">
                        {#each (catalog[row.notifType] ?? []) as field}
                          {#if isFieldEnabled(row.notifType, field.id)}
                            <div class="embed-mock-field">
                              <div class="embed-mock-field-name">{field.label}</div>
                              <div class="embed-mock-field-value">—</div>
                            </div>
                          {/if}
                        {/each}
                        {#if activeFieldCount(row.notifType) === 0}
                          <span class="font-mono text-[9px]" style="color: var(--color-subtext);">
                            Aucun champ — embed vide
                          </span>
                        {/if}
                      </div>
                    </div>
                  </div>
                </div>

              </div>
            {/if}
          {/each}
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
      <section class="space-y-3">
      <div class="section-label pb-2" style="border-bottom: 1px solid var(--color-border);">Variables d'environnement</div>
      <div class="space-y-1" role="list" aria-label="Variables d'environnement actives">
        {#each [
          { key: 'RENCODEX_FFMPEG_PATH',         active: effective?.ffmpeg_path !== form.ffmpeg_path },
          { key: 'RENCODEX_DISCORD_TOKEN',        active: effective?.discord_token_set },
          { key: 'RENCODEX_DISCORD_LOG_CHANNEL',  active: effective?.discord_log_channel_id !== form.discord_log_channel_id },
          { key: 'RENCODEX_DISCORD_CMD_CHANNEL',  active: effective?.discord_cmd_channel_id !== form.discord_cmd_channel_id },
        ] as v}
          <div class="flex items-center justify-between px-3 py-2 rounded-[var(--radius-sm)]"
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
    </div><!-- end drawer-content -->
  </div><!-- end drawer-body -->

  <!-- Footer -->
  <div class="drawer-footer">
    <button
      onclick={save}
      disabled={saving}
      class="btn btn-primary justify-center font-mono text-[11px]"
      aria-label={saving ? "Sauvegarde en cours" : "Sauvegarder les paramètres"}
    >
      {#if saving}
        <span class="spinner w-3 h-3 border-2 border-white/30 border-t-white shrink-0 rounded-full animate-spin"></span>
        SAUVEGARDE…
      {:else}SAUVEGARDER{/if}
    </button>
    <button
      onclick={() => (open = false)}
      class="btn btn-secondary font-mono text-[11px]"
      aria-label="Fermer sans sauvegarder"
    >
      FERMER
    </button>
  </div>
</div>
{/if}

<style>
  /* ── Backdrop & modal ─────────────────────────────────────────────────────── */

  .settings-backdrop {
    position: fixed;
    inset: 0;
    z-index: 9980;
    background: transparent;
  }

  .settings-panel {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    width: 400px;
    z-index: 9981;
    background: var(--color-panel);
    border-left: 1px solid var(--color-border);
    border-radius: var(--radius-lg) 0 0 var(--radius-lg);
    box-shadow: -20px 0 60px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    color: var(--color-text);
    overflow: hidden;
  }

  .drawer-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .drawer-body {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  /* Sidebar nav */
  .drawer-nav {
    width: 44px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 12px 6px;
    border-right: 1px solid var(--color-border);
    background: var(--color-surface);
    overflow: hidden;
    position: relative;
    z-index: 10;
    /* Fermeture : même easing/durée que le out:fly de la modal (cubicIn, 220ms) */
    transition: width 220ms cubic-bezier(0.32, 0, 0.67, 0);
  }
  .drawer-nav:hover {
    width: 112px;
    /* Ouverture : même easing/durée que le in:fly de la modal (cubicOut, 320ms), avec un délai pour éviter les survols rapides */
    transition: width 320ms cubic-bezier(0.33, 1, 0.68, 1) 180ms;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 8px;
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    font-weight: 500;
    padding: 7px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid transparent;
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    text-align: left;
    transition: background 0.15s, color 0.15s, border-color 0.15s, transform 0.15s cubic-bezier(0.22, 1, 0.36, 1);
    width: 100%;
    white-space: nowrap;
    overflow: hidden;
  }
  .nav-item:hover {
    background: var(--color-panel);
    color: var(--color-text);
    transform: translateX(2px);
  }
  .nav-item--active {
    background: color-mix(in srgb, var(--color-accent) 12%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 28%, transparent);
    color: var(--color-accent);
  }
  .nav-item--active:hover {
    background: color-mix(in srgb, var(--color-accent) 18%, transparent);
    color: var(--color-accent);
  }

  .nav-label {
    opacity: 0;
    transform: translateX(-6px);
    transition: opacity 180ms cubic-bezier(0.32, 0, 0.67, 0) 0s, transform 180ms cubic-bezier(0.32, 0, 0.67, 0) 0s;
    pointer-events: none;
  }
  .drawer-nav:hover .nav-label {
    opacity: 1;
    transform: translateX(0);
    transition: opacity 280ms cubic-bezier(0.33, 1, 0.68, 1) 260ms, transform 280ms cubic-bezier(0.33, 1, 0.68, 1) 260ms;
  }

  /* Content area */
  .drawer-content {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
  }

  .drawer-footer {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    padding: 10px 16px;
    border-top: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  /* ── Icônes ───────────────────────────────────────────────────────────────── */

  .icon-btn {
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
    transition: background 0.1s, color 0.1s, border-color 0.1s;
  }
  .icon-btn:hover {
    background: var(--color-panel2);
    border-color: var(--color-border);
    color: var(--color-text);
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

  .color-swatch-preset {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    border: 1px solid var(--color-border);
    cursor: pointer;
    padding: 0;
    flex-shrink: 0;
    transition: transform 0.12s, border-color 0.12s;
  }
  .color-swatch-preset:hover { transform: scale(1.15); }
  .color-swatch-preset--active {
    border: 2px solid var(--color-text);
    transform: scale(1.15);
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

  /* ── Ligne de notification ─────────────────────────────────────────────────── */

  .notif-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 8px 0;
  }

  /* Bouton chevron dépliage */
  .chevron-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    border: none;
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    padding: 0;
    border-radius: var(--radius-xs);
    transition: color 0.1s;
  }
  .chevron-btn:hover { color: var(--color-accent); }

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

  /* ── Panneau de champs ─────────────────────────────────────────────────────── */

  .fields-panel {
    margin: 0 0 4px 22px;
    padding: 10px;
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    gap: 8px;
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
    padding: 8px 10px;
    flex: 1;
    min-width: 0;
  }

  .embed-mock-title {
    font-size: 11px;
    font-weight: 600;
    color: #f2f3f5;
    margin-bottom: 6px;
  }

  .embed-mock-fields {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 12px;
  }

  .embed-mock-field {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .embed-mock-field-name {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 600;
    color: #b5bac1;
  }

  .embed-mock-field-value {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: #5c6370;
  }
</style>