<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { RefreshCw, Wifi, WifiOff, Bot, Hash, Key, Activity } from "@lucide/svelte";

  type BotStatus = "connecting" | "connected" | "disconnected" | "error";

  type Props = {
    discordEnabled: boolean;
    discordBotToken: string;
    discordCmdChannelId: string;
  };

  let { discordEnabled, discordBotToken, discordCmdChannelId }: Props = $props();

  // ── État du bot ─────────────────────────────────────────────────────────────

  let status = $state<BotStatus>("disconnected");
  let botName = $state<string | null>(null);
  let lastError = $state<string | null>(null);
  let restarting = $state(false);
  let logs = $state<{ time: string; msg: string; type: "info" | "error" | "success" }[]>([]);

  // ── Listeners Tauri ─────────────────────────────────────────────────────────

  let unlistenConnected: UnlistenFn | null = null;
  let unlistenDisconnected: UnlistenFn | null = null;
  let unlistenError: UnlistenFn | null = null;

  function addLog(msg: string, type: "info" | "error" | "success" = "info") {
    const time = new Date().toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit", second: "2-digit" });
    logs = [{ time, msg, type }, ...logs].slice(0, 50);
  }

  onMount(async () => {
    // Écoute les événements émis par le bot Rust
    unlistenConnected = await listen<{ name: string }>("discord-bot-connected", (e) => {
      status = "connected";
      botName = e.payload.name;
      lastError = null;
      addLog(`Connecté en tant que ${e.payload.name}`, "success");
    });

    unlistenDisconnected = await listen("discord-bot-disconnected", () => {
      status = "disconnected";
      botName = null;
      addLog("Bot déconnecté", "info");
    });

    unlistenError = await listen<{ message: string }>("discord-bot-error", (e) => {
      status = "error";
      lastError = e.payload.message;
      addLog(`Erreur : ${e.payload.message}`, "error");
    });

    // État initial basé sur la config
    if (!discordEnabled || !discordBotToken) {
      status = "disconnected";
      addLog("Bot désactivé ou token manquant", "info");
    } else {
      status = "connecting";
      addLog("Vérification du statut…", "info");
    }
  });

  onDestroy(() => {
    unlistenConnected?.();
    unlistenDisconnected?.();
    unlistenError?.();
  });

  // ── Redémarrage ─────────────────────────────────────────────────────────────

  async function restartBot() {
    if (restarting) return;
    restarting = true;
    addLog("Redémarrage du bot…", "info");
    try {
      // Sauvegarder la config déclenche config-saved → supervise_discord_bot se relance
      await invoke("save_config", {
        config: {
          discord_bot_token: discordBotToken,
          discord_cmd_channel_id: discordCmdChannelId,
          discord_enabled: discordEnabled,
          // Valeurs minimales requises par AppConfig — le backend merge avec l'existant
        }
      });
      status = "connecting";
      addLog("Signal de redémarrage envoyé", "info");
    } catch (e) {
      addLog(`Erreur redémarrage : ${e}`, "error");
    } finally {
      restarting = false;
    }
  }

  // ── Dérivés UI ──────────────────────────────────────────────────────────────

  let statusLabel = $derived(
    status === "connected"    ? "Connecté"      :
    status === "connecting"   ? "Connexion…"    :
    status === "error"        ? "Erreur"        :
                                "Déconnecté"
  );

  let statusColor = $derived(
    status === "connected"  ? "var(--color-success)"  :
    status === "connecting" ? "var(--color-accent)"   :
    status === "error"      ? "var(--color-danger)"   :
                              "var(--color-subtext)"
  );

  // Token masqué : montre les 6 premiers et 4 derniers caractères
  let maskedToken = $derived(
    discordBotToken.length > 12
      ? discordBotToken.slice(0, 6) + "•".repeat(20) + discordBotToken.slice(-4)
      : discordBotToken
        ? "•".repeat(discordBotToken.length)
        : "—"
  );
</script>

<section class="content-section">
  <header class="section-header">
    <div>
      <h2 class="section-title">Bot Discord</h2>
      <p class="section-desc">Statut de connexion et contrôle du bot de commandes.</p>
    </div>
  </header>

  <!-- Statut principal -->
  <div class="status-card" style="--status-color: {statusColor}">
    <div class="status-icon">
      {#if status === "connected"}
        <Wifi class="w-5 h-5" />
      {:else if status === "connecting"}
        <Activity class="w-5 h-5 animate-pulse" />
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
        <div class="status-error">{lastError}</div>
      {/if}
    </div>
    <button
      onclick={restartBot}
      disabled={restarting || !discordEnabled || !discordBotToken}
      class="restart-btn"
      aria-label="Redémarrer le bot"
      title={!discordEnabled || !discordBotToken ? "Bot désactivé ou token manquant" : "Redémarrer le bot"}
    >
      <RefreshCw class="w-3.5 h-3.5 {restarting ? 'animate-spin' : ''}" />
      {restarting ? "…" : "Redémarrer"}
    </button>
  </div>

  <!-- Infos de connexion -->
  <div class="info-grid">
    <div class="info-row">
      <div class="info-icon"><Key class="w-3.5 h-3.5" /></div>
      <div class="info-content">
        <span class="info-label">Token</span>
        <span class="info-value font-mono">{maskedToken}</span>
      </div>
      <div class="info-badge" class:badge-ok={!!discordBotToken} class:badge-missing={!discordBotToken}>
        {discordBotToken ? "Défini" : "Manquant"}
      </div>
    </div>

    <div class="info-row">
      <div class="info-icon"><Hash class="w-3.5 h-3.5" /></div>
      <div class="info-content">
        <span class="info-label">Salon de commandes</span>
        <span class="info-value font-mono">{discordCmdChannelId || "—"}</span>
      </div>
      <div class="info-badge" class:badge-ok={!!discordCmdChannelId} class:badge-missing={!discordCmdChannelId}>
        {discordCmdChannelId ? "Configuré" : "Manquant"}
      </div>
    </div>

    <div class="info-row">
      <div class="info-icon"><Activity class="w-3.5 h-3.5" /></div>
      <div class="info-content">
        <span class="info-label">Supervision</span>
        <span class="info-value">Redémarrage automatique · backoff 5s → 5min</span>
      </div>
    </div>
  </div>

  {#if !discordEnabled}
    <div class="notice notice-warn">
      Le bot est désactivé. Activez Discord dans l'onglet <strong>Discord</strong> pour le démarrer.
    </div>
  {/if}

  <!-- Journal -->
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
  .restart-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

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
  .badge-ok {
    color: var(--color-success);
    background: color-mix(in srgb, var(--color-success) 12%, transparent);
  }
  .badge-missing {
    color: var(--color-danger, #e05c5c);
    background: color-mix(in srgb, var(--color-danger, #e05c5c) 12%, transparent);
  }

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

  /* ── Log ─────────────────────────────────────────────────────────────────── */

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
  .log-entry:hover {
    background: color-mix(in srgb, var(--color-accent) 4%, transparent);
  }

  .log-time {
    color: var(--color-subtext2, var(--color-subtext));
    flex-shrink: 0;
    font-size: 9px;
  }

  .log-msg { color: var(--color-text); }
  .log-entry--error  .log-msg { color: var(--color-danger, #e05c5c); }
  .log-entry--success .log-msg { color: var(--color-success); }
</style>