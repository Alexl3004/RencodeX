<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { encoder } from "$lib/stores/encoder.svelte";
  import { fly } from "svelte/transition";
  import { sineIn } from "svelte/easing";
  import { Eye, EyeOff, X, Check } from '@lucide/svelte';


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
  };

  let {
    open = $bindable(false),
  } = $props();

  let saving    = $state(false),
    testing   = $state(false);
  let testResult = $state<"ok" | "error" | null>(null);
  let effective  = $state<EffectiveConfig | null>(null);
  let showToken  = $state(false);

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
  });

  async function loadSettings() {
    try {
      const [saved, eff] = await Promise.all([
        invoke<SavedConfig>("load_config"),
        invoke<EffectiveConfig>("get_effective_config"),
      ]);
      form = { ...form, ...saved };
      effective = eff;
    } catch (e) {
      encoder.log(`Erreur chargement config : ${e}`, "error");
    }
  }
  async function save() {
    saving = true;
    try {
      await invoke("save_config", { config: form });
      effective = await invoke<EffectiveConfig>("get_effective_config");
      encoder.log("Configuration sauvegardée ✓", "success");
    } catch (e) {
      encoder.log(`Erreur sauvegarde : ${e}`, "error");
    } finally {
      saving = false;
    }
  }
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
      encoder.log("Test Discord envoyé ✓", "success");
    } catch (e) {
      testResult = "error";
      encoder.log(`Erreur Discord : ${e}`, "error");
    } finally {
      testing = false;
      setTimeout(() => (testResult = null), 4000);
    }
  }
  $effect(() => {
    if (open) loadSettings();
  });

  let discordFromEnv = $derived(
    effective?.discord_token_set && !form.discord_bot_token,
  );

  const flyParams = { x: 320, duration: 200, easing: sineIn };
</script>

{#if open}
<!-- Backdrop -->
<div
  class="settings-backdrop"
  role="presentation"
  onclick={() => (open = false)}
  onkeydown={(e) => { if (e.key === 'Escape') open = false; }}
></div>

<!-- Panel -->
<div
  class="settings-panel"
  role="dialog"
  aria-modal="true"
  aria-label="Panneau de paramètres"
  tabindex="-1"
  transition:fly={flyParams}
>
  <!-- Header -->
  <div class="drawer-header">
    <div class="flex items-center gap-2">
      <div class="w-[3px] h-5 rounded-[1px]" style="background: var(--color-accent);"></div>
      <span class="font-mono text-[12px] font-semibold uppercase tracking-wider"
            style="color: var(--color-text);">Paramètres</span>
    </div>
    <button onclick={() => (open = false)} class="icon-btn" title="Fermer" aria-label="Fermer le panneau">
      <X class="w-4 h-4" />
    </button>
  </div>

  <!-- Body -->
  <div class="flex-1 overflow-y-auto px-5 py-4 space-y-6">

    <!-- FFmpeg -->
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

    <!-- Discord -->
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

      {#if discordFromEnv}
        <div class="flex items-center gap-2 font-mono text-[10px] px-3 py-2 rounded-[2px]"
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

      <!-- Notifications -->
      <div class="space-y-0 pt-3" style="border-top: 1px solid var(--color-border);">
        <div class="section-label mb-2">Notifications</div>

        {#each [
          { key: 'discord_notify_start',     label: "Début d'encodage",   desc: "Envoi au démarrage" },
          { key: 'discord_notify_file_done', label: "Fichier terminé",    desc: "Après chaque fichier" },
          { key: 'discord_notify_progress',  label: "Progression",        desc: "Mise à jour périodique" },
          { key: 'discord_notify_summary',   label: "Résumé global",      desc: "Bilan de session" },
          { key: 'discord_notify_error',     label: "Erreur d'encodage",  desc: "En cas d'échec" },
        ] as row}
          <div class="flex items-center justify-between py-2"
               style="border-bottom: 1px solid color-mix(in srgb, var(--color-border) 50%, transparent);">
            <div>
              <span class="text-[12px]" style="color: var(--color-text);">{row.label}</span>
              <p class="font-mono text-[9px] mt-0.5" style="color: var(--color-subtext);">{row.desc}</p>
            </div>
            <button
              type="button"
              role="switch"
              aria-checked={(form as any)[row.key]}
              onclick={() => ((form as any)[row.key] = !(form as any)[row.key])}
              class="toggle {(form as any)[row.key] ? 'on' : ''}"
              aria-label={row.label}
            ></button>
          </div>
          {#if row.key === 'discord_notify_progress' && form.discord_notify_progress}
            <div class="pb-2 pt-1">
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
        {/each}
      </div>

      <!-- Test Discord -->
      <button
        onclick={testDiscord}
        disabled={testing || (!form.discord_bot_token && !discordFromEnv) || !form.discord_log_channel_id}
        class="w-full btn font-mono text-[11px] justify-center"
        class:btn-primary={testResult === "ok"}
        class:btn-danger={testResult === "error"}
        aria-label={testing ? "Test en cours" : testResult === "ok" ? "Test réussi" : testResult === "error" ? "Test échoué" : "Tester la notification Discord"}
      >
        {#if testing}
          <span class="spinner w-3 h-3 border-2 border-current/30 border-t-current shrink-0 rounded-full animate-spin"></span>
          ENVOI…
        {:else if testResult === "ok"}✓ TEST ENVOYÉ
        {:else if testResult === "error"}✗ ÉCHEC
        {:else}TESTER LA NOTIFICATION{/if}
      </button>
    </section>

    <!-- Email -->
    <section class="space-y-3">
      <div class="flex items-center justify-between pb-2" style="border-bottom: 1px solid var(--color-border);">
        <div class="section-label">Email</div>
        <button
          type="button"
          role="switch"
          aria-checked={form.send_email}
          onclick={() => (form.send_email = !form.send_email)}
          class="toggle {form.send_email ? 'on' : ''}"
          aria-label={form.send_email ? "Désactiver les notifications email" : "Activer les notifications email"}
        ></button>
      </div>
      {#if form.send_email}
        <div>
          <label for="email-to" class="field-label">Email de destination</label>
          <input id="email-to" type="email" bind:value={form.email_to}
                 placeholder="moi@exemple.com" class="field-input w-full px-3 py-2" />
        </div>
      {/if}
    </section>

    <!-- Variables d'env -->
    <section class="space-y-3">
      <div class="section-label pb-2" style="border-bottom: 1px solid var(--color-border);">Variables d'environnement</div>
      <div class="space-y-1" role="list" aria-label="Variables d'environnement actives">
        {#each [
          { key: 'RENCODEX_FFMPEG_PATH',         active: effective?.ffmpeg_path !== form.ffmpeg_path },
          { key: 'RENCODEX_DISCORD_TOKEN',        active: effective?.discord_token_set },
          { key: 'RENCODEX_DISCORD_LOG_CHANNEL',  active: effective?.discord_log_channel_id !== form.discord_log_channel_id },
          { key: 'RENCODEX_DISCORD_CMD_CHANNEL',  active: effective?.discord_cmd_channel_id !== form.discord_cmd_channel_id },
        ] as v}
          <div class="flex items-center justify-between px-3 py-2 rounded-[2px]"
               style="background: var(--color-surface); border: 1px solid var(--color-border);"
               role="listitem">
            <span class="font-mono text-[10px]" style="color: var(--color-subtext);">{v.key}</span>
            <span class="font-mono text-[10px]"
                  style="color: {v.active ? 'var(--color-success)' : 'var(--color-subtext2)'};"
                  aria-label={v.active ? "Variable définie" : "Variable non définie"}>
              {v.active ? "✓ définie" : "—"}
            </span>
          </div>
        {/each}
      </div>
    </section>
  </div>

  <!-- Footer -->
  <div class="drawer-footer">
    <button
      onclick={save}
      disabled={saving}
      class="btn btn-primary flex-1 justify-center font-mono text-[11px]"
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
    box-shadow: -20px 0 60px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    color: var(--color-text);
  }

  .drawer-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 20px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .drawer-footer {
    display: flex;
    gap: 8px;
    padding: 10px 20px;
    border-top: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .icon-btn {
    width: 26px;
    height: 26px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
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
    border-radius: 2px;
    outline: none;
    transition: border-color 0.12s;
  }
  .field-input:focus { border-color: var(--color-accent); }
  .field-input:disabled { opacity: 0.5; cursor: not-allowed; }

  .toggle-row {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }
</style>