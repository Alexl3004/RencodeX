<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import {
    RefreshCw, Wifi, WifiOff, Bot, Hash, Key, Activity,
    Pause, Play, Square, SkipForward, Terminal, Clock,
    ChevronDown, ChevronRight, Zap, AlertTriangle,
    Eye, EyeOff, Save,
  } from "@lucide/svelte";

  // ── Types ────────────────────────────────────────────────────────────────────

  type BotStatus = "connecting" | "connected" | "disconnected" | "error";

  interface ProgressEvent {
    file_index: number;
    file_total: number;
    file_name: string;
    percent: number;
    speed: number;
    remaining_file: number;
    remaining_total: number;
  }

  type Props = {
    discordEnabled: boolean;
    discordBotToken: string;
    discordCmdChannelId: string;
    onConfigChange?: (token: string, channelId: string, enabled: boolean) => void;
  };

  let { discordEnabled, discordBotToken, discordCmdChannelId, onConfigChange }: Props = $props();

  // Champs locaux éditables — initialisés depuis les props dans onMount
  let localToken     = $state("");
  let localChannelId = $state("");
  let showToken      = $state(false);
  let savingCreds    = $state(false);

  // ── Constantes de persistance (cache UI seulement) ───────────────────────
  // La source de vérité est get_bot_status() côté Rust.
  // localStorage sert uniquement à éviter un flash visuel au montage.

  const LS_STATUS_KEY  = "rencodex:bot:status";
  const LS_BOTNAME_KEY = "rencodex:bot:name";
  const LS_SINCE_KEY   = "rencodex:bot:since";

  // ── État du bot ─────────────────────────────────────────────────────────────

  // Valeur initiale depuis localStorage (évite le flash "Déconnecté" → "Connecté")
  // puis écrasée immédiatement par get_bot_status() dans onMount.
  const storedStatus = (localStorage.getItem(LS_STATUS_KEY) ?? "disconnected") as BotStatus;

  let status    = $state<BotStatus>(storedStatus);
  let botName   = $state<string | null>(localStorage.getItem(LS_BOTNAME_KEY));
  let lastError = $state<string | null>(null);
  let restarting = $state(false);
  let logs      = $state<{ time: string; msg: string; type: "info" | "error" | "success" }[]>([]);

  // Uptime
  const storedSince = localStorage.getItem(LS_SINCE_KEY);
  let connectedSince = $state<number | null>(storedSince ? Number(storedSince) : null);
  let uptimeStr  = $state("—");
  let uptimeTimer: ReturnType<typeof setInterval> | null = null;

  function startUptimeTimer() {
    stopUptimeTimer();
    uptimeTimer = setInterval(() => {
      if (!connectedSince) { uptimeStr = "—"; return; }
      const secs = Math.floor((Date.now() - connectedSince) / 1000);
      const h = Math.floor(secs / 3600);
      const m = Math.floor((secs % 3600) / 60);
      const s = secs % 60;
      uptimeStr = h > 0
        ? `${h}h ${m.toString().padStart(2, "0")}m`
        : `${m}m ${s.toString().padStart(2, "0")}s`;
    }, 1000);
  }
  function stopUptimeTimer() {
    if (uptimeTimer !== null) { clearInterval(uptimeTimer); uptimeTimer = null; }
  }

  // ── Contrôle encodage ────────────────────────────────────────────────────────

  let paused    = $state(false);
  let encoding  = $state(false);
  let progress  = $state<ProgressEvent | null>(null);
  let actionBusy = $state<"pause" | "resume" | "cancel" | "skip" | null>(null);

  async function doAction(action: "pause" | "resume" | "cancel" | "skip") {
    if (actionBusy) return;
    actionBusy = action;
    try {
      if (action === "pause")  await invoke("pause_encoding");
      if (action === "resume") await invoke("resume_encoding");
      if (action === "cancel") await invoke("cancel_encoding");
      if (action === "skip")   await invoke("skip_encoding");
    } catch (e) {
      addLog(`Erreur commande ${action} : ${e}`, "error");
    } finally {
      actionBusy = null;
    }
  }

  // ── Référence des commandes ──────────────────────────────────────────────────

  const BOT_COMMANDS = [
    { cmd: "!status",  desc: "Progression de l'encodage en cours" },
    { cmd: "!queue",   desc: "Liste des fichiers en attente" },
    { cmd: "!skip",    desc: "Passer au fichier suivant" },
    { cmd: "!pause",   desc: "Mettre l'encodage en pause" },
    { cmd: "!resume",  desc: "Reprendre l'encodage" },
    { cmd: "!cancel",  desc: "Annuler l'encodage" },
    { cmd: "!panel",   desc: "Ouvrir le panneau interactif avec boutons" },
    { cmd: "!help",    desc: "Afficher l'aide" },
  ] as const;

  let showCommands = $state(false);

  // ── Listeners Tauri ─────────────────────────────────────────────────────────

  let unlisten: UnlistenFn[] = [];

  function addLog(msg: string, type: "info" | "error" | "success" = "info") {
    const time = new Date().toLocaleTimeString("fr-FR", {
      hour: "2-digit", minute: "2-digit", second: "2-digit",
    });
    logs = [{ time, msg, type }, ...logs].slice(0, 100);
  }

  function persistStatus(s: BotStatus, name: string | null = null) {
    localStorage.setItem(LS_STATUS_KEY, s);
    if (name !== null) localStorage.setItem(LS_BOTNAME_KEY, name);
    else localStorage.removeItem(LS_BOTNAME_KEY);
  }

  onMount(async () => {
    // Initialisation des champs locaux depuis les props
    localToken     = discordBotToken;
    localChannelId = discordCmdChannelId;

    // ── Source de vérité : interroger Rust directement ──────────────────────
    // get_bot_status() retourne l'état courant des globaux BOT_CONNECTED /
    // BOT_NAME mis à jour par le handler `ready` du bot.
    // C'est la seule façon fiable de connaître l'état réel au montage du
    // composant, que ce soit la première ouverture ou un retour sur l'onglet.
    try {
      const s = await invoke<{ connected: boolean; name: string }>("get_bot_status");
      if (s.connected) {
        status  = "connected";
        botName = s.name || botName;
        lastError = null;
        if (!connectedSince) {
          // On ne connaît pas l'heure exacte de connexion ; on la marque
          // maintenant pour avoir un uptime relatif (sous-estimé mais exact).
          connectedSince = Date.now();
          localStorage.setItem(LS_SINCE_KEY, String(connectedSince));
        }
        persistStatus("connected", botName);
        startUptimeTimer();
        addLog(`Bot connecté en tant que ${botName ?? s.name}`, "success");
      } else {
        // Le bot n'est pas connecté : corriger un éventuel cache localStorage
        // périmé (ex: crash de la session précédente).
        if (status === "connected") {
          status = "disconnected";
          persistStatus("disconnected");
        }
        connectedSince = null;
        localStorage.removeItem(LS_SINCE_KEY);

        if (!discordEnabled || !localToken) {
          addLog("Bot désactivé ou token manquant.", "info");
        } else {
          // Activé + token présent mais pas encore connecté → en cours de connexion.
          status = "connecting";
          addLog("Connexion en cours…", "info");
        }
      }
    } catch (e) {
      // Commande indisponible (version Rust non mise à jour) → fallback gracieux.
      addLog("Impossible de vérifier le statut du bot.", "info");
    }

    // Lire l'état pause courant
    try {
      paused = await invoke<boolean>("get_paused");
    } catch { /* ignore */ }

    unlisten.push(
      await listen<{ name: string }>("discord-bot-connected", (e) => {
        status = "connected";
        botName = e.payload.name;
        lastError = null;
        connectedSince = Date.now();
        localStorage.setItem(LS_SINCE_KEY, String(connectedSince));
        persistStatus("connected", botName);
        startUptimeTimer();
        addLog(`Connecté en tant que ${e.payload.name}`, "success");
      }),

      await listen("discord-bot-disconnected", () => {
        status = "disconnected";
        botName = null;
        connectedSince = null;
        stopUptimeTimer();
        uptimeStr = "—";
        persistStatus("disconnected");
        localStorage.removeItem(LS_SINCE_KEY);
        addLog("Bot déconnecté", "info");
      }),

      await listen<{ message: string }>("discord-bot-error", (e) => {
        status = "error";
        lastError = e.payload.message;
        connectedSince = null;
        stopUptimeTimer();
        uptimeStr = "—";
        persistStatus("error");
        localStorage.removeItem(LS_SINCE_KEY);
        addLog(`Erreur : ${e.payload.message}`, "error");
      }),

      await listen<ProgressEvent>("encode-progress", (e) => {
        progress = e.payload;
        encoding = true;
      }),

      await listen<boolean>("encode-paused", (e) => {
        paused = e.payload;
        addLog(paused ? "Encodage mis en pause (Discord)" : "Encodage repris (Discord)", "info");
      }),

      await listen("encode-cancelled", () => {
        encoding = false;
        progress = null;
        paused = false;
        addLog("Encodage annulé", "info");
      }),

      await listen("encode-done", () => {
        encoding = false;
        progress = null;
        paused = false;
      }),
    );

  });

  onDestroy(() => {
    unlisten.forEach(fn => fn());
    stopUptimeTimer();
  });

  // ── Redémarrage ─────────────────────────────────────────────────────────────

  async function saveCredentials() {
    if (savingCreds) return;
    savingCreds = true;
    try {
      const currentCfg = await invoke<Record<string, unknown>>("load_config");
      await invoke("save_config", {
        config: {
          ...currentCfg,
          discord_bot_token: localToken,
          discord_cmd_channel_id: localChannelId,
          discord_enabled: discordEnabled,
        },
      });
      onConfigChange?.(localToken, localChannelId, discordEnabled);
      addLog("Identifiants sauvegardés.", "success");
    } catch (e) {
      addLog(`Erreur sauvegarde : ${e}`, "error");
    } finally {
      savingCreds = false;
    }
  }

  async function restartBot() {
    if (restarting) return;
    restarting = true;
    status = "connecting";
    addLog("Redémarrage du bot…", "info");
    try {
      const currentCfg = await invoke<Record<string, unknown>>("load_config");
      await invoke("save_config", {
        config: {
          ...currentCfg,
          discord_bot_token: localToken,
          discord_cmd_channel_id: localChannelId,
          discord_enabled: discordEnabled,
        },
      });
      onConfigChange?.(localToken, localChannelId, discordEnabled);
      addLog("Signal de redémarrage envoyé", "info");
    } catch (e) {
      addLog(`Erreur redémarrage : ${e}`, "error");
      status = "disconnected";
    } finally {
      restarting = false;
    }
  }

  // ── Formatage ────────────────────────────────────────────────────────────────

  function fmtSecs(s: number) {
    if (!isFinite(s) || s <= 0) return "—";
    const m = Math.floor(s / 60);
    const sec = Math.floor(s % 60);
    return m > 0 ? `${m}m ${sec.toString().padStart(2, "0")}s` : `${sec}s`;
  }

  // ── Dérivés UI ───────────────────────────────────────────────────────────────

  let statusLabel = $derived(
    status === "connected"  ? "Connecté"   :
    status === "connecting" ? "Connexion…" :
    status === "error"      ? "Erreur"     :
                              "Déconnecté"
  );

  let statusColor = $derived(
    status === "connected"  ? "var(--color-success)"  :
    status === "connecting" ? "var(--color-accent)"   :
    status === "error"      ? "var(--color-danger)"   :
                              "var(--color-subtext)"
  );

  let maskedToken = $derived(
    localToken.length > 12
      ? localToken.slice(0, 6) + "•".repeat(20) + localToken.slice(-4)
      : localToken
        ? "•".repeat(localToken.length)
        : "—"
  );

  // Détecter si les champs ont changé par rapport aux props reçues
  let credsDirty = $derived(
    localToken !== discordBotToken || localChannelId !== discordCmdChannelId
  );

  let progressFilled = $derived(
    progress ? Math.round((progress.percent / 100) * 20) : 0
  );
</script>

<section class="content-section">
  <header class="section-header">
    <div>
      <h2 class="section-title">Bot Discord</h2>
      <p class="section-desc">Statut de connexion et contrôle direct du bot de commandes.</p>
    </div>
  </header>

  <!-- ── Carte statut ───────────────────────────────────────────────────────── -->
  <div class="status-card" style="--status-color: {statusColor}">
    <div class="status-icon">
      {#if status === "connected"}
        <Wifi class="w-5 h-5" />
      {:else if status === "connecting"}
        <Activity class="w-5 h-5 animate-pulse" />
      {:else if status === "error"}
        <AlertTriangle class="w-5 h-5" />
      {:else}
        <WifiOff class="w-5 h-5" />
      {/if}
    </div>

    <div class="status-body">
      <div class="status-label">{statusLabel}</div>
      {#if botName}
        <div class="status-name">
          <Bot class="w-3 h-3" aria-hidden="true" />
          {botName}
        </div>
      {/if}
      {#if lastError}
        <div class="status-error" title={lastError}>{lastError}</div>
      {/if}
    </div>

    {#if status === "connected" && connectedSince}
      <div class="uptime-badge">
        <Clock class="w-3 h-3" />
        <span>{uptimeStr}</span>
      </div>
    {/if}

    <button
      onclick={restartBot}
      disabled={restarting || !discordEnabled || !localToken}
      class="restart-btn"
      aria-label="Redémarrer le bot"
      title={!discordEnabled || !localToken ? "Bot désactivé ou token manquant" : "Redémarrer le bot"}
    >
      <RefreshCw class="w-3.5 h-3.5 {restarting ? 'animate-spin' : ''}" />
      {restarting ? "…" : "Redémarrer"}
    </button>
  </div>

  <!-- ── Identifiants du bot ────────────────────────────────────────────────── -->
  <div class="creds-section">
    <div class="creds-header">
      <Key class="w-3.5 h-3.5" style="color: var(--color-accent)" />
      <span class="creds-title">Identifiants du bot</span>
    </div>

    <div class="creds-body">
      <!-- Token -->
      <div class="cred-field">
        <label for="bot-token" class="cred-label">Token du bot</label>
        <div class="cred-input-wrap">
          <input
            id="bot-token"
            type={showToken ? "text" : "password"}
            bind:value={localToken}
            placeholder="MTEx…"
            class="cred-input"
            autocomplete="off"
            spellcheck="false"
          />
          <button
            type="button"
            onclick={() => (showToken = !showToken)}
            class="cred-eye"
            aria-label={showToken ? "Masquer le token" : "Afficher le token"}
          >
            {#if showToken}
              <EyeOff class="w-3.5 h-3.5" />
            {:else}
              <Eye class="w-3.5 h-3.5" />
            {/if}
          </button>
        </div>
        <div class="cred-badge-row">
          <span class="info-badge" class:badge-ok={!!localToken} class:badge-missing={!localToken}>
            {localToken ? "Défini" : "Manquant"}
          </span>
        </div>
      </div>

      <!-- Salon de commandes -->
      <div class="cred-field">
        <label for="bot-cmd-channel" class="cred-label">Salon de commandes</label>
        <input
          id="bot-cmd-channel"
          type="text"
          bind:value={localChannelId}
          placeholder="ID du salon (ex : 1234567890)"
          class="cred-input"
          autocomplete="off"
          inputmode="numeric"
        />
        <div class="cred-badge-row">
          <span class="info-badge" class:badge-ok={!!localChannelId} class:badge-missing={!localChannelId}>
            {localChannelId ? "Configuré" : "Manquant"}
          </span>
        </div>
      </div>

      <!-- Supervision info + bouton save -->
      <div class="creds-footer">
        <div class="info-row-inline">
          <Activity class="w-3 h-3" style="color: var(--color-subtext)" />
          <span class="cred-note">Redémarrage automatique · backoff 5s → 5min</span>
        </div>
        <button
          type="button"
          onclick={saveCredentials}
          disabled={savingCreds || !credsDirty}
          class="save-creds-btn"
          title={credsDirty ? "Sauvegarder les identifiants" : "Aucune modification"}
        >
          <Save class="w-3.5 h-3.5" />
          {savingCreds ? "…" : "Sauvegarder"}
        </button>
      </div>
    </div>
  </div>

  {#if !discordEnabled}
    <div class="notice notice-warn">
      Le bot est désactivé. Activez Discord dans l'onglet <strong>Discord</strong> pour le démarrer.
    </div>
  {/if}

  <!-- ── Panneau de contrôle encodage ──────────────────────────────────────── -->
  <div class="control-panel">
    <div class="control-header">
      <Zap class="w-3.5 h-3.5" style="color: var(--color-accent)" />
      <span class="control-title">Contrôle encodage</span>
      {#if encoding}
        <span class="control-badge badge-encoding">EN COURS</span>
      {:else}
        <span class="control-badge badge-idle">INACTIF</span>
      {/if}
    </div>

    {#if encoding && progress}
      <!-- Barre de progression -->
      <div class="progress-block">
        <div class="progress-meta">
          <span class="progress-filename" title={progress.file_name}>{progress.file_name}</span>
          <span class="progress-pos">{progress.file_index + 1}/{progress.file_total}</span>
        </div>

        <div class="progress-bar-wrap" role="progressbar" aria-valuenow={progress.percent} aria-valuemin={0} aria-valuemax={100}>
          <div class="progress-bar" style="width: {progress.percent.toFixed(1)}%"></div>
        </div>

        <div class="progress-stats">
          <span class="prog-stat">
            <span class="prog-stat-label">%</span>
            <span class="prog-stat-val">{progress.percent.toFixed(1)}</span>
          </span>
          <span class="prog-stat">
            <span class="prog-stat-label">Vitesse</span>
            <span class="prog-stat-val">{progress.speed.toFixed(2)}x</span>
          </span>
          <span class="prog-stat">
            <span class="prog-stat-label">Restant</span>
            <span class="prog-stat-val">{fmtSecs(progress.remaining_file)}</span>
          </span>
          <span class="prog-stat">
            <span class="prog-stat-label">Total</span>
            <span class="prog-stat-val">{fmtSecs(progress.remaining_total)}</span>
          </span>
        </div>

        {#if paused}
          <div class="paused-badge">
            <Pause class="w-3 h-3" />
            En pause
          </div>
        {/if}
      </div>
    {:else}
      <p class="control-empty">Aucun encodage en cours. Les boutons seront actifs dès le démarrage.</p>
    {/if}

    <!-- Boutons de contrôle -->
    <div class="control-btns">
      {#if !paused}
        <button
          onclick={() => doAction("pause")}
          disabled={!encoding || !!actionBusy}
          class="ctrl-btn ctrl-btn--secondary"
          title="Mettre en pause"
        >
          <Pause class="w-3.5 h-3.5" />
          Pause
          {#if actionBusy === "pause"}<span class="btn-spinner"></span>{/if}
        </button>
      {:else}
        <button
          onclick={() => doAction("resume")}
          disabled={!encoding || !!actionBusy}
          class="ctrl-btn ctrl-btn--success"
          title="Reprendre"
        >
          <Play class="w-3.5 h-3.5" />
          Reprendre
          {#if actionBusy === "resume"}<span class="btn-spinner"></span>{/if}
        </button>
      {/if}

      <button
        onclick={() => doAction("skip")}
        disabled={!encoding || !!actionBusy}
        class="ctrl-btn ctrl-btn--secondary"
        title="Passer au fichier suivant"
      >
        <SkipForward class="w-3.5 h-3.5" />
        Suivant
        {#if actionBusy === "skip"}<span class="btn-spinner"></span>{/if}
      </button>

      <button
        onclick={() => doAction("cancel")}
        disabled={!encoding || !!actionBusy}
        class="ctrl-btn ctrl-btn--danger"
        title="Annuler l'encodage"
      >
        <Square class="w-3.5 h-3.5" />
        Annuler
        {#if actionBusy === "cancel"}<span class="btn-spinner"></span>{/if}
      </button>
    </div>
  </div>

  <!-- ── Référence des commandes bot ────────────────────────────────────────── -->
  <div class="commands-section">
    <button
      class="commands-toggle"
      onclick={() => (showCommands = !showCommands)}
      aria-expanded={showCommands}
      type="button"
    >
      <Terminal class="w-3.5 h-3.5" style="color: var(--color-accent)" />
      <span class="commands-toggle-label">Commandes Discord disponibles</span>
      {#if showCommands}
        <ChevronDown class="w-3.5 h-3.5 ml-auto" style="color: var(--color-subtext)" />
      {:else}
        <ChevronRight class="w-3.5 h-3.5 ml-auto" style="color: var(--color-subtext)" />
      {/if}
    </button>

    {#if showCommands}
      <div class="commands-body">
        {#each BOT_COMMANDS as row}
          <div class="cmd-row">
            <code class="cmd-name">{row.cmd}</code>
            <span class="cmd-desc">{row.desc}</span>
          </div>
        {/each}
        <div class="cmd-note">
          Limite : 2 commandes / 10 s par utilisateur · Les boutons via <code class="cmd-name-inline">!panel</code> répondent de façon éphémère.
        </div>
      </div>
    {/if}
  </div>

  <!-- ── Journal ────────────────────────────────────────────────────────────── -->
  <div class="log-section">
    <div class="log-header">
      <span class="log-title">Journal</span>
      {#if logs.length > 0}
        <button onclick={() => (logs = [])} class="log-clear" type="button">Effacer</button>
      {/if}
    </div>
    <div class="log-body" aria-live="polite" aria-label="Journal du bot Discord">
      {#if logs.length === 0}
        <span class="log-empty">Aucun événement pour cette session.</span>
      {:else}
        {#each logs as entry (entry.time + entry.msg)}
          <div class="log-entry log-entry--{entry.type}">
            <span class="log-time">{entry.time}</span>
            <span class="log-msg">{entry.msg}</span>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</section>

<style>
  .content-section {
    padding: 28px 32px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .section-header {
    padding-bottom: 20px;
    border-bottom: 1px solid var(--color-border);
    margin-bottom: 4px;
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
    margin: 0;
  }

  /* ── Status card ─────────────────────────────────────────────────────────── */

  .status-card {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px 16px;
    border-radius: var(--radius-sm);
    border: 1px solid color-mix(in srgb, var(--status-color) 25%, var(--color-border));
    background: color-mix(in srgb, var(--status-color) 6%, var(--color-surface));
  }

  .status-icon {
    color: var(--status-color);
    flex-shrink: 0;
    display: flex;
    align-items: center;
  }

  .status-body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .status-label {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    font-weight: 600;
    color: var(--status-color);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .status-name {
    font-family: "Geist Mono", monospace;
    font-size: 12px;
    color: var(--color-text);
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .status-error {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-danger, #e05c5c);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 280px;
  }

  .uptime-badge {
    display: flex;
    align-items: center;
    gap: 4px;
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-success);
    background: color-mix(in srgb, var(--color-success) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-success) 20%, transparent);
    border-radius: 999px;
    padding: 2px 8px;
    flex-shrink: 0;
  }

  .restart-btn {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 5px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-panel);
    color: var(--color-subtext);
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    cursor: pointer;
    flex-shrink: 0;
    transition: color 0.12s, border-color 0.12s, background 0.12s;
  }
  .restart-btn:hover:not(:disabled) {
    color: var(--color-text);
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-panel));
  }
  .restart-btn:disabled { opacity: 0.4; cursor: not-allowed; }

  /* ── Credentials section ─────────────────────────────────────────────────── */

  .creds-section {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .creds-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 14px;
    background: var(--color-panel);
    border-bottom: 1px solid var(--color-border);
  }

  .creds-title {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-subtext);
  }

  .creds-body {
    background: var(--color-surface);
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .cred-field {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .cred-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-subtext);
  }

  .cred-input-wrap {
    position: relative;
    display: flex;
    align-items: center;
  }

  .cred-input {
    width: 100%;
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    color: var(--color-text);
    border-radius: var(--radius-sm);
    padding: 7px 36px 7px 10px;
    outline: none;
    transition: border-color 0.12s;
  }
  .cred-input:focus {
    border-color: var(--color-accent);
  }

  .cred-eye {
    position: absolute;
    right: 8px;
    color: var(--color-subtext);
    background: transparent;
    border: none;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    transition: color 0.1s;
    padding: 0;
  }
  .cred-eye:hover { color: var(--color-text); }

  /* Input sans bouton œil */
  .cred-field > .cred-input {
    padding: 7px 10px;
  }

  .cred-badge-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .creds-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding-top: 10px;
    border-top: 1px solid var(--color-border);
  }

  .info-row-inline {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }

  .cred-note {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .save-creds-btn {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 5px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid color-mix(in srgb, var(--color-accent) 35%, transparent);
    background: color-mix(in srgb, var(--color-accent) 10%, transparent);
    color: var(--color-accent);
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    cursor: pointer;
    flex-shrink: 0;
    transition: background 0.12s, opacity 0.12s;
  }
  .save-creds-btn:hover:not(:disabled) {
    background: color-mix(in srgb, var(--color-accent) 18%, transparent);
  }
  .save-creds-btn:disabled { opacity: 0.35; cursor: not-allowed; }

  /* ── Info grid ───────────────────────────────────────────────────────────── */

  .info-grid {
    display: flex;
    flex-direction: column;
    gap: 2px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .info-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 14px;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
  }
  .info-row:last-child { border-bottom: none; }

  .info-icon {
    color: var(--color-subtext);
    flex-shrink: 0;
    display: flex;
    align-items: center;
  }

  .info-content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .info-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-subtext);
  }

  .info-value {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    color: var(--color-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .info-badge {
    flex-shrink: 0;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    padding: 2px 6px;
    border-radius: 999px;
    font-weight: 500;
  }
  .badge-ok      { color: var(--color-success); background: color-mix(in srgb, var(--color-success) 12%, transparent); }
  .badge-missing { color: var(--color-danger, #e05c5c); background: color-mix(in srgb, var(--color-danger, #e05c5c) 12%, transparent); }

  /* ── Notice ──────────────────────────────────────────────────────────────── */

  .notice {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    padding: 10px 14px;
    border-radius: var(--radius-sm);
    line-height: 1.5;
  }
  .notice-warn {
    color: var(--color-warning, #d4a017);
    background: color-mix(in srgb, var(--color-warning, #d4a017) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-warning, #d4a017) 25%, transparent);
  }
  .notice strong { font-weight: 600; }

  /* ── Panneau de contrôle ─────────────────────────────────────────────────── */

  .control-panel {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .control-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 14px;
    background: var(--color-panel);
    border-bottom: 1px solid var(--color-border);
  }

  .control-title {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-subtext);
  }

  .control-badge {
    font-family: "Geist Mono", monospace;
    font-size: 8px;
    padding: 1px 6px;
    border-radius: 999px;
    font-weight: 700;
    letter-spacing: 0.05em;
  }
  .badge-encoding {
    color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 15%, transparent);
  }
  .badge-idle {
    color: var(--color-subtext);
    background: color-mix(in srgb, var(--color-subtext) 10%, transparent);
  }

  .control-empty {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
    padding: 12px 14px 0;
    margin: 0;
    line-height: 1.5;
  }

  /* Barre de progression */
  .progress-block {
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .progress-meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .progress-filename {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    color: var(--color-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }

  .progress-pos {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
    flex-shrink: 0;
  }

  .progress-bar-wrap {
    height: 4px;
    background: color-mix(in srgb, var(--color-accent) 15%, var(--color-border));
    border-radius: 999px;
    overflow: hidden;
  }
  .progress-bar {
    height: 100%;
    background: var(--color-accent);
    border-radius: 999px;
    transition: width 0.5s ease;
  }

  .progress-stats {
    display: flex;
    gap: 16px;
    flex-wrap: wrap;
  }

  .prog-stat {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .prog-stat-label {
    font-family: "Geist Mono", monospace;
    font-size: 8px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-subtext);
  }
  .prog-stat-val {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    color: var(--color-text);
    font-weight: 600;
  }

  .paused-badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 600;
    color: var(--color-warning, #d4a017);
    background: color-mix(in srgb, var(--color-warning, #d4a017) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-warning, #d4a017) 25%, transparent);
    border-radius: 999px;
    padding: 2px 8px;
    width: fit-content;
  }

  /* Boutons de contrôle */
  .control-btns {
    display: flex;
    gap: 8px;
    padding: 12px 14px;
    background: var(--color-surface);
    flex-wrap: wrap;
  }

  .ctrl-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    font-weight: 600;
    cursor: pointer;
    transition: opacity 0.1s, background 0.12s, border-color 0.12s;
    border: 1px solid transparent;
    position: relative;
  }
  .ctrl-btn:disabled { opacity: 0.35; cursor: not-allowed; }

  .ctrl-btn--secondary {
    background: var(--color-panel);
    border-color: var(--color-border);
    color: var(--color-text);
  }
  .ctrl-btn--secondary:hover:not(:disabled) {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-panel));
  }

  .ctrl-btn--success {
    background: color-mix(in srgb, var(--color-success) 15%, var(--color-panel));
    border-color: color-mix(in srgb, var(--color-success) 35%, transparent);
    color: var(--color-success);
  }
  .ctrl-btn--success:hover:not(:disabled) {
    background: color-mix(in srgb, var(--color-success) 22%, var(--color-panel));
  }

  .ctrl-btn--danger {
    background: color-mix(in srgb, var(--color-danger, #e05c5c) 12%, var(--color-panel));
    border-color: color-mix(in srgb, var(--color-danger, #e05c5c) 30%, transparent);
    color: var(--color-danger, #e05c5c);
  }
  .ctrl-btn--danger:hover:not(:disabled) {
    background: color-mix(in srgb, var(--color-danger, #e05c5c) 20%, var(--color-panel));
  }

  .btn-spinner {
    display: inline-block;
    width: 8px;
    height: 8px;
    border: 1.5px solid currentColor;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    margin-left: 2px;
  }

  @keyframes spin { to { transform: rotate(360deg); } }

  /* ── Référence commandes ─────────────────────────────────────────────────── */

  .commands-section {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .commands-toggle {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 14px;
    background: var(--color-panel);
    border: none;
    cursor: pointer;
    text-align: left;
    transition: background 0.1s;
  }
  .commands-toggle:hover {
    background: color-mix(in srgb, var(--color-accent) 5%, var(--color-panel));
  }

  .commands-toggle-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-subtext);
  }

  .commands-body {
    background: var(--color-surface);
    border-top: 1px solid var(--color-border);
    padding: 4px 0;
  }

  .cmd-row {
    display: flex;
    align-items: baseline;
    gap: 12px;
    padding: 5px 14px;
  }
  .cmd-row:hover {
    background: color-mix(in srgb, var(--color-accent) 4%, transparent);
  }

  .cmd-name {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    color: var(--color-accent);
    flex-shrink: 0;
    min-width: 80px;
    font-weight: 600;
  }

  .cmd-name-inline {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-accent);
    font-weight: 600;
  }

  .cmd-desc {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
    line-height: 1.5;
  }

  .cmd-note {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext);
    padding: 6px 14px 8px;
    border-top: 1px solid var(--color-border);
    margin-top: 4px;
    line-height: 1.6;
    opacity: 0.7;
  }

  /* ── Journal ─────────────────────────────────────────────────────────────── */

  .log-section {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .log-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 14px;
    background: var(--color-panel);
    border-bottom: 1px solid var(--color-border);
  }

  .log-title {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-subtext);
  }

  .log-clear {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext);
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 2px 4px;
    border-radius: 3px;
    transition: color 0.1s;
  }
  .log-clear:hover { color: var(--color-text); }

  .log-body {
    background: var(--color-surface);
    padding: 8px 0;
    max-height: 180px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .log-empty {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext2, var(--color-subtext));
    padding: 8px 14px;
  }

  .log-entry {
    display: flex;
    align-items: baseline;
    gap: 10px;
    padding: 3px 14px;
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    line-height: 1.5;
  }
  .log-entry:hover { background: color-mix(in srgb, var(--color-accent) 4%, transparent); }

  .log-time {
    color: var(--color-subtext2, var(--color-subtext));
    flex-shrink: 0;
    font-size: 9px;
  }

  .log-msg { color: var(--color-text); }
  .log-entry--error   .log-msg { color: var(--color-danger, #e05c5c); }
  .log-entry--success .log-msg { color: var(--color-success); }
</style>