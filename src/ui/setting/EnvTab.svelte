<script lang="ts">
  import { Check } from "@lucide/svelte";

  type EffectiveConfig = {
    ffmpeg_path: string;
    discord_token_set: boolean;
    discord_log_channel_id: string;
    discord_cmd_channel_id: string;
  };

  type SavedConfig = {
    ffmpeg_path: string;
    discord_log_channel_id: string;
    discord_cmd_channel_id: string;
  };

  let {
    effective,
    form,
  }: {
    effective: EffectiveConfig | null;
    form: SavedConfig;
  } = $props();

  const vars = $derived([
    {
      key: "RENCODEX_FFMPEG_PATH",
      active: !!effective?.ffmpeg_path,
    },
    {
      key: "RENCODEX_DISCORD_TOKEN",
      active: effective?.discord_token_set,
    },
    {
      key: "RENCODEX_DISCORD_LOG_CHANNEL",
      active: !!effective?.discord_log_channel_id,
    },
    {
      key: "RENCODEX_DISCORD_CMD_CHANNEL",
      active: !!effective?.discord_cmd_channel_id,
    },
  ]);
</script>

<section class="content-section">
  <header class="section-header">
    <div>
      <h2 class="section-title">Variables d'environnement</h2>
      <p class="section-desc">
        Surcharge de la configuration via les variables système détectées au
        démarrage.
      </p>
    </div>
  </header>

  <div
    class="space-y-1"
    role="list"
    aria-label="Variables d'environnement actives"
  >
    {#each vars as v}
      <div
        class="flex items-center justify-between px-3 py-2 rounded-[var(--radius-sm)]"
        style="background: var(--color-surface); border: 1px solid var(--color-border);"
        role="listitem"
      >
        <span class="font-mono text-[10px]" style="color: var(--color-subtext);"
          >{v.key}</span
        >
        <span
          class="font-mono text-[10px] inline-flex items-center gap-1"
          style="color: {v.active
            ? 'var(--color-success)'
            : 'var(--color-subtext2)'};"
          aria-label={v.active ? "Variable définie" : "Variable non définie"}
        >
          {#if v.active}<Check class="w-3 h-3" />définie{:else}—{/if}
        </span>
      </div>
    {/each}
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
</style>