<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { encoder } from "$lib/stores/encoder.svelte";
  

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

  let open = $state(false),
    saving = $state(false),
    testing = $state(false);
  let testResult = $state<"ok" | "error" | null>(null);
  let effective = $state<EffectiveConfig | null>(null);
  let showToken = $state(false);

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
        summary: {
          files: [],
          total_original_mb: 10.0,
          total_encoded_mb: 6.2,
          total_secs: 42.0,
        },
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
  function toggle() {
    if (!open) loadSettings();
    open = !open;
  }

  let discordFromEnv = $derived(
    effective?.discord_token_set && !form.discord_bot_token,
  );
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === "Escape" && open) open = false;
  }}
/>

<button
  onclick={toggle}
  class="btn btn-ghost px-2 py-1 relative"
  title="Paramètres"
  aria-label="Paramètres"
>
  <svg
    width="14"
    height="14"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="1.8"
    aria-hidden="true"
  >
    <circle cx="12" cy="12" r="3" />
    <path
      d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33
             1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33
             l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4
             h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06
             A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51
             1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9
             a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"
    />
  </svg>
  {#if effective?.discord_enabled}
    <span
      class="absolute -top-0.5 -right-0.5 w-2 h-2 rounded-full bg-[var(--color-success)] border border-[var(--color-panel)]"
      aria-hidden="true"
    ></span>
  {/if}
</button>

{#if open}
  <div class="contents" style="isolation: isolate;">
    <div
      class="fixed inset-0 z-40 bg-black/50"
      onclick={() => (open = false)}
      role="presentation"
    ></div>

    <aside
      class="fixed top-0 right-0 h-full w-[400px] z-50 bg-[var(--color-panel)]
                  border-l border-[var(--color-border)] flex flex-col shadow-[-20px_0_60px_rgba(0,0,0,0.6)]"
      aria-label="Panneau de paramètres"
    >
    <!-- Header -->
    <div
      class="flex items-center justify-between px-5 py-3 border-b border-[var(--color-border)] shrink-0"
    >
      <div class="flex items-center gap-2">
        <div class="w-[3px] h-5 rounded-[1px] bg-[var(--color-accent)]"></div>
        <span
          class="font-mono text-[12px] font-semibold text-[var(--color-text)] uppercase tracking-wider"
          >Paramètres</span
        >
      </div>
      <button
        onclick={() => (open = false)}
        class="btn btn-ghost p-1.5"
        title="Fermer"
        aria-label="Fermer le panneau des paramètres"
      >
        <svg
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
          aria-hidden="true"
        >
          <line x1="18" y1="6" x2="6" y2="18" /><line
            x1="6"
            y1="6"
            x2="18"
            y2="18"
          />
        </svg>
      </button>
    </div>

    <!-- Body -->
    <div class="flex-1 overflow-y-auto px-5 py-4 space-y-6">
      <!-- FFmpeg -->
      <section class="space-y-3">
        <div class="section-label border-b border-[var(--color-border)] pb-2">
          FFmpeg
        </div>
        <div>
          <label
            for="ffmpeg-path"
            class="font-mono text-[10px] text-[var(--color-subtext)] block mb-1"
          >
            Chemin vers ffmpeg.exe
          </label>
          <input
            id="ffmpeg-path"
            type="text"
            bind:value={form.ffmpeg_path}
            placeholder="C:\Outil\ffmpeg\bin\ffmpeg.exe"
            class="w-full px-3 py-2 text-[11px]"
          />
        </div>
      </section>

      <!-- Discord -->
      <section class="space-y-3">
        <div
          class="flex items-center justify-between border-b border-[var(--color-border)] pb-2"
        >
          <div class="section-label">Discord</div>
          <label class="flex items-center gap-2 cursor-pointer">
            <span class="font-mono text-[10px] text-[var(--color-subtext)]"
              >Activer</span
            >
            <button
              class="toggle {form.discord_enabled ? 'on' : ''}"
              onclick={() => (form.discord_enabled = !form.discord_enabled)}
              role="switch"
              aria-checked={form.discord_enabled}
              aria-label={form.discord_enabled
                ? "Désactiver Discord"
                : "Activer Discord"}
              tabindex="0"
              type="button"
            ></button>
          </label>
        </div>

        {#if discordFromEnv}
          <div
            class="flex items-center gap-2 font-mono text-[10px] text-[var(--color-success)] bg-[var(--color-success)]/10 border border-[var(--color-success)]/20 rounded-[2px] px-3 py-2"
            role="status"
          >
            <svg
              width="10"
              height="10"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.5"
              aria-hidden="true"><polyline points="20 6 9 17 4 12" /></svg
            >
            Token actif via variable d'environnement
          </div>
        {/if}

        <div>
          <label
            for="discord-token"
            class="font-mono text-[10px] text-[var(--color-subtext)] block mb-1"
            >Token du bot</label
          >
          <div class="relative">
            <input
              id="discord-token"
              type={showToken ? "text" : "password"}
              bind:value={form.discord_bot_token}
              disabled={discordFromEnv}
              placeholder="MTEx…"
              class="w-full px-3 py-2 text-[11px] pr-9"
            />
            <button
              onclick={() => (showToken = !showToken)}
              type="button"
              class="absolute right-2 top-1/2 -translate-y-1/2 text-[var(--color-subtext)] hover:text-[var(--color-text)] transition-colors"
              aria-label={showToken ? "Masquer le token" : "Afficher le token"}
            >
              <svg
                width="13"
                height="13"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                aria-hidden="true"
              >
                {#if showToken}
                  <path
                    d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94"
                  />
                  <path
                    d="M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19"
                  />
                  <line x1="1" y1="1" x2="23" y2="23" />
                {:else}
                  <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
                  <circle cx="12" cy="12" r="3" />
                {/if}
              </svg>
            </button>
          </div>
        </div>

        <!-- Discord Channels -->
        <div>
          <label
            for="discord-log-channel"
            class="font-mono text-[10px] text-[var(--color-subtext)] block mb-1"
            >Salon de logs</label
          >
          <input
            id="discord-log-channel"
            type="text"
            bind:value={form.discord_log_channel_id}
            placeholder="ID du salon"
            class="w-full px-3 py-2 text-[11px]"
          />
        </div>

        <div>
          <label
            for="discord-cmd-channel"
            class="font-mono text-[10px] text-[var(--color-subtext)] block mb-1"
            >Salon de commandes</label
          >
          <input
            id="discord-cmd-channel"
            type="text"
            bind:value={form.discord_cmd_channel_id}
            placeholder="ID du salon"
            class="w-full px-3 py-2 text-[11px]"
          />
        </div>

        <!-- Notifications -->
        <div class="space-y-0 border-t border-[var(--color-border)] pt-3">
          <div class="section-label mb-2">Notifications</div>

          <label
            class="flex items-center justify-between py-2 cursor-pointer border-b border-[var(--color-border)]/50"
          >
            <div>
              <span class="text-[12px] text-[var(--color-text)]"
                >Début d'encodage</span
              >
              <p
                class="font-mono text-[9px] text-[var(--color-subtext)] mt-0.5"
              >
                Envoi au démarrage
              </p>
            </div>
            <button
              class="toggle {form.discord_notify_start ? 'on' : ''}"
              onclick={() =>
                (form.discord_notify_start = !form.discord_notify_start)}
              role="switch"
              aria-checked={form.discord_notify_start}
              aria-label={form.discord_notify_start
                ? "Désactiver la notification de début d'encodage"
                : "Activer la notification de début d'encodage"}
              tabindex="0"
              type="button"
            ></button>
          </label>

          <label
            class="flex items-center justify-between py-2 cursor-pointer border-b border-[var(--color-border)]/50"
          >
            <div>
              <span class="text-[12px] text-[var(--color-text)]"
                >Fichier terminé</span
              >
              <p
                class="font-mono text-[9px] text-[var(--color-subtext)] mt-0.5"
              >
                Après chaque fichier
              </p>
            </div>
            <button
              class="toggle {form.discord_notify_file_done ? 'on' : ''}"
              onclick={() =>
                (form.discord_notify_file_done =
                  !form.discord_notify_file_done)}
              role="switch"
              aria-checked={form.discord_notify_file_done}
              aria-label={form.discord_notify_file_done
                ? "Désactiver la notification de fichier terminé"
                : "Activer la notification de fichier terminé"}
              tabindex="0"
              type="button"
            ></button>
          </label>

          <label
            class="flex items-center justify-between py-2 cursor-pointer border-b border-[var(--color-border)]/50"
          >
            <div>
              <span class="text-[12px] text-[var(--color-text)]"
                >Progression</span
              >
              <p
                class="font-mono text-[9px] text-[var(--color-subtext)] mt-0.5"
              >
                Mise à jour périodique
              </p>
            </div>
            <button
              class="toggle {form.discord_notify_progress ? 'on' : ''}"
              onclick={() =>
                (form.discord_notify_progress =
                  !form.discord_notify_progress)}
              role="switch"
              aria-checked={form.discord_notify_progress}
              aria-label={form.discord_notify_progress
                ? "Désactiver la notification de progression"
                : "Activer la notification de progression"}
              tabindex="0"
              type="button"
            ></button>
          </label>

          {#if form.discord_notify_progress}
            <div class="pb-2">
              <label
                for="discord-progress-interval"
                class="font-mono text-[10px] text-[var(--color-subtext)] block mb-1"
                >Intervalle (secondes)</label
              >
              <input
                id="discord-progress-interval"
                type="number"
                bind:value={form.discord_progress_interval}
                min="10"
                max="300"
                placeholder="30"
                class="w-full px-3 py-2 text-[11px]"
              />
            </div>
          {/if}

          <label
            class="flex items-center justify-between py-2 cursor-pointer border-b border-[var(--color-border)]/50"
          >
            <div>
              <span class="text-[12px] text-[var(--color-text)]"
                >Résumé global</span
              >
              <p
                class="font-mono text-[9px] text-[var(--color-subtext)] mt-0.5"
              >
                Bilan de session
              </p>
            </div>
            <button
              class="toggle {form.discord_notify_summary ? 'on' : ''}"
              onclick={() =>
                (form.discord_notify_summary = !form.discord_notify_summary)}
              role="switch"
              aria-checked={form.discord_notify_summary}
              aria-label={form.discord_notify_summary
                ? "Désactiver la notification de résumé global"
                : "Activer la notification de résumé global"}
              tabindex="0"
              type="button"
            ></button>
          </label>

          <label
            class="flex items-center justify-between py-2 cursor-pointer border-b border-[var(--color-border)]/50"
          >
            <div>
              <span class="text-[12px] text-[var(--color-text)]"
                >Erreur d'encodage</span
              >
              <p
                class="font-mono text-[9px] text-[var(--color-subtext)] mt-0.5"
              >
                En cas d'échec
              </p>
            </div>
            <button
              class="toggle {form.discord_notify_error ? 'on' : ''}"
              onclick={() =>
                (form.discord_notify_error = !form.discord_notify_error)}
              role="switch"
              aria-checked={form.discord_notify_error}
              aria-label={form.discord_notify_error
                ? "Désactiver la notification d'erreur"
                : "Activer la notification d'erreur"}
              tabindex="0"
              type="button"
            ></button>
          </label>
        </div>

        <button
          onclick={testDiscord}
          disabled={testing ||
            (!form.discord_bot_token && !discordFromEnv) ||
            !form.discord_log_channel_id}
          class="w-full btn font-mono text-[11px] justify-center disabled:opacity-40
                 {testResult === 'ok'
            ? 'btn-lang-active'
            : testResult === 'error'
              ? 'border-[var(--color-danger)] text-[var(--color-danger)]'
              : 'btn-secondary'}"
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
              class="w-3 h-3 border-2 border-[var(--color-subtext)]/30 border-t-[var(--color-subtext)] rounded-full animate-spin"
              aria-hidden="true"
            ></span>
            ENVOI…
          {:else if testResult === "ok"}✓ TEST ENVOYÉ
          {:else if testResult === "error"}✗ ÉCHEC
          {:else}TESTER LA NOTIFICATION{/if}
        </button>
      </section>

      <!-- Email -->
      <section class="space-y-3">
        <div
          class="flex items-center justify-between border-b border-[var(--color-border)] pb-2"
        >
          <div class="section-label">Email</div>
          <button
            class="toggle {form.send_email ? 'on' : ''}"
            onclick={() => (form.send_email = !form.send_email)}
            role="switch"
            aria-checked={form.send_email}
            aria-label={form.send_email
              ? "Désactiver les notifications email"
              : "Activer les notifications email"}
            tabindex="0"
            type="button"
          ></button>
        </div>
        {#if form.send_email}
          <div>
            <label
              for="email-to"
              class="font-mono text-[10px] text-[var(--color-subtext)] block mb-1"
              >Email de destination</label
            >
            <input
              id="email-to"
              type="email"
              bind:value={form.email_to}
              placeholder="moi@exemple.com"
              class="w-full px-3 py-2 text-[11px]"
            />
          </div>
        {/if}
      </section>

      <!-- Env vars -->
      <section class="space-y-3">
        <div class="section-label border-b border-[var(--color-border)] pb-2">
          Variables d'environnement
        </div>
        <div
          class="space-y-1"
          role="list"
          aria-label="Variables d'environnement actives"
        >
          <div
            class="flex items-center justify-between px-3 py-2 bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[2px]"
            role="listitem"
          >
            <span class="font-mono text-[10px] text-[var(--color-subtext)]"
              >RENCODEX_FFMPEG_PATH</span
            >
            <span
              class="font-mono text-[10px] {effective?.ffmpeg_path !==
              form.ffmpeg_path
                ? 'text-[var(--color-success)]'
                : 'text-[var(--color-subtext2)]'}"
              aria-label={effective?.ffmpeg_path !== form.ffmpeg_path
                ? "Variable définie"
                : "Variable non définie"}
            >
              {effective?.ffmpeg_path !== form.ffmpeg_path ? "✓ définie" : "—"}
            </span>
          </div>
          <div
            class="flex items-center justify-between px-3 py-2 bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[2px]"
            role="listitem"
          >
            <span class="font-mono text-[10px] text-[var(--color-subtext)]"
              >RENCODEX_DISCORD_TOKEN</span
            >
            <span
              class="font-mono text-[10px] {effective?.discord_token_set
                ? 'text-[var(--color-success)]'
                : 'text-[var(--color-subtext2)]'}"
              aria-label={effective?.discord_token_set
                ? "Variable définie"
                : "Variable non définie"}
            >
              {effective?.discord_token_set ? "✓ définie" : "—"}
            </span>
          </div>
          <div
            class="flex items-center justify-between px-3 py-2 bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[2px]"
            role="listitem"
          >
            <span class="font-mono text-[10px] text-[var(--color-subtext)]"
              >RENCODEX_DISCORD_LOG_CHANNEL</span
            >
            <span
              class="font-mono text-[10px] {effective?.discord_log_channel_id !==
              form.discord_log_channel_id
                ? 'text-[var(--color-success)]'
                : 'text-[var(--color-subtext2)]'}"
              aria-label={effective?.discord_log_channel_id !==
              form.discord_log_channel_id
                ? "Variable définie"
                : "Variable non définie"}
            >
              {effective?.discord_log_channel_id !== form.discord_log_channel_id
                ? "✓ définie"
                : "—"}
            </span>
          </div>
          <div
            class="flex items-center justify-between px-3 py-2 bg-[var(--color-surface)] border border-[var(--color-border)] rounded-[2px]"
            role="listitem"
          >
            <span class="font-mono text-[10px] text-[var(--color-subtext)]"
              >RENCODEX_DISCORD_CMD_CHANNEL</span
            >
            <span
              class="font-mono text-[10px] {effective?.discord_cmd_channel_id !==
              form.discord_cmd_channel_id
                ? 'text-[var(--color-success)]'
                : 'text-[var(--color-subtext2)]'}"
              aria-label={effective?.discord_cmd_channel_id !==
              form.discord_cmd_channel_id
                ? "Variable définie"
                : "Variable non définie"}
            >
              {effective?.discord_cmd_channel_id !== form.discord_cmd_channel_id
                ? "✓ définie"
                : "—"}
            </span>
          </div>
        </div>
      </section>
    </div>

    <!-- Footer -->
    <div
      class="px-5 py-3 border-t border-[var(--color-border)] flex gap-2 shrink-0"
    >
      <button
        onclick={save}
        disabled={saving}
        class="btn btn-primary flex-1 justify-center font-mono text-[11px]"
        aria-label={saving
          ? "Sauvegarde en cours"
          : "Sauvegarder les paramètres"}
      >
        {#if saving}
          <span
            class="w-3 h-3 border-2 border-white/30 border-t-white rounded-full animate-spin"
            aria-hidden="true"
          ></span>SAUVEGARDE…
        {:else}SAUVEGARDER{/if}
      </button>
      <button
        onclick={() => (open = false)}
        class="btn font-mono text-[11px]"
        aria-label="Fermer sans sauvegarder"
      >
        FERMER
      </button>
    </div>
  </aside>
  </div>
{/if}
