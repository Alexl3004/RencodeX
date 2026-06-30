<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { tick } from "svelte";
  import { Switch } from "@skeletonlabs/skeleton-svelte";
  import { Copy, CircleCheck, Trash2, X } from '@lucide/svelte';


  let { onClose }: { onClose?: () => void } = $props();

  let container = $state<HTMLDivElement | null>(null);
  let autoScroll = $state(true);
  let collapsed = $state(false);
  let copied = $state(false);

  let showInfo = $state(true);
  let showWarn = $state(true);
  let showError = $state(true);
  let showSuccess = $state(true);

  $effect(() => {
    const _len = encoder.logs.length;
    if (autoScroll && container && !collapsed) {
      tick().then(() => {
        container!.scrollTop = container!.scrollHeight;
      });
    }
  });

  const LEVEL_CLASS: Record<string, string> = {
    info: "log-info",
    warn: "log-warning",
    error: "log-error",
    success: "log-success",
  };

  let filteredLogs = $derived(
    encoder.logs.filter((e) => {
      if (e.level === "info" && !showInfo) return false;
      if (e.level === "warn" && !showWarn) return false;
      if (e.level === "error" && !showError) return false;
      if (e.level === "success" && !showSuccess) return false;
      return true;
    }),
  );

  let errorCount = $derived(
    encoder.logs.filter((e) => e.level === "error").length,
  );
  let warnCount = $derived(
    encoder.logs.filter((e) => e.level === "warn").length,
  );

  async function copyAll() {
    await navigator.clipboard.writeText(
      encoder.logs.map((l) => l.msg).join("\n"),
    );
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  function toggleInfo() { showInfo = !showInfo; }
  function toggleWarn() { showWarn = !showWarn; }
  function toggleError() { showError = !showError; }
  function toggleSuccess() { showSuccess = !showSuccess; }
</script>

<div class="border border-[var(--color-border)] rounded-[2px] bg-[var(--color-surface)] flex flex-col h-full">
  <!-- Header -->
  <div class="flex items-center justify-between px-3 py-2 border-b border-[var(--color-border)] bg-[var(--color-panel)] select-none">
    <div class="flex items-center gap-2">
      <div class="w-[3px] h-4 rounded-[1px] bg-[var(--color-accent)]"></div>
      <span class="section-label">Journal</span>
    </div>
    <div class="flex items-center gap-2">
      <!-- Filtres -->
      <div class="flex items-center gap-0.5">
        <button type="button" onclick={toggleInfo}
          class="filter-btn font-mono text-[9px] border transition-all
                 {showInfo ? 'text-[var(--color-subtext)] bg-[var(--color-surface)] border-[var(--color-border)]' : 'text-[var(--color-subtext2)] border-transparent'}"
          aria-pressed={showInfo} aria-label="Filtre info">
          INFO
        </button>
        <button type="button" onclick={toggleWarn}
          class="filter-btn font-mono text-[9px] border transition-all
                 {showWarn ? 'text-[var(--color-warning)] bg-[var(--color-warning)]/10 border-[var(--color-warning)]/30' : 'text-[var(--color-subtext2)] border-transparent'}"
          aria-pressed={showWarn} aria-label="Filtre avertissements">
          WARN
        </button>
        <button type="button" onclick={toggleError}
          class="filter-btn font-mono text-[9px] border transition-all
                 {showError ? 'text-[var(--color-danger)] bg-[var(--color-danger)]/10 border-[var(--color-danger)]/30' : 'text-[var(--color-subtext2)] border-transparent'}"
          aria-pressed={showError} aria-label="Filtre erreurs">
          ERR
        </button>
        <button type="button" onclick={toggleSuccess}
          class="filter-btn font-mono text-[9px] border transition-all
                 {showSuccess ? 'text-[var(--color-success)] bg-[var(--color-success)]/10 border-[var(--color-success)]/30' : 'text-[var(--color-subtext2)] border-transparent'}"
          aria-pressed={showSuccess} aria-label="Filtre succès">
          OK
        </button>
      </div>
      <div class="sep h-3 mx-1" aria-hidden="true"></div>
      <Switch
        checked={autoScroll}
        onCheckedChange={(details) => (autoScroll = details.checked)}
        class="flex items-center gap-1.5"
      >
        <Switch.Control
          class="h-[15px] w-[28px] rounded-full border border-[var(--color-border2)] bg-[var(--color-muted)] transition-colors data-[state=checked]:border-[var(--color-accent2)] data-[state=checked]:bg-[var(--color-accent)]"
        >
          <Switch.Thumb
            class="h-[11px] w-[11px] translate-x-[2px] rounded-full bg-white transition-transform data-[state=checked]:translate-x-[15px]"
          />
        </Switch.Control>
        <Switch.Label class="font-mono text-[10px] text-[var(--color-subtext)]">auto</Switch.Label>
        <Switch.HiddenInput />
      </Switch>
      <div class="sep h-3 mx-1" aria-hidden="true"></div>
      <button type="button" onclick={copyAll}
        class="log-action-btn font-mono text-[10px] flex items-center gap-1
               {copied ? 'text-[var(--color-success)]' : 'text-[var(--color-subtext)]'}"
        aria-label={copied ? 'Copié' : 'Copier tous les logs'}>
        {#if copied}
          <CircleCheck class="w-3.5 h-3.5" />
          COPIÉ
        {:else}
          <Copy class="w-3.5 h-3.5" />
          COPY
        {/if}
      </button>
      <button type="button" onclick={() => encoder.clearLogs()}
        class="log-action-btn font-mono text-[10px] text-[var(--color-subtext)] flex items-center gap-1"
        aria-label="Effacer les logs">
        <Trash2 class="w-3.5 h-3.5" />
        CLR
      </button>
      {#if onClose}
        <div class="sep h-3 mx-1" aria-hidden="true"></div>
        <button type="button" onclick={onClose}
          class="log-action-btn text-[var(--color-subtext)]"
          aria-label="Fermer le journal">
          <X class="w-3.5 h-3.5" />
        </button>
      {/if}
    </div>
  </div>

  <!-- Content -->
  <div bind:this={container} class="flex-1 overflow-y-auto px-3 py-2 font-mono text-[11px] space-y-0.5 leading-relaxed"
       onscroll={() => {
         if (!container) return;
         autoScroll = container.scrollHeight - container.scrollTop - container.clientHeight < 30;
       }}>
    {#if filteredLogs.length === 0}
      <span class="text-[var(--color-subtext2)] italic text-[10px]">
        {encoder.logs.length === 0 ? "Aucune activité" : "Aucune entrée pour les filtres sélectionnés"}
      </span>
    {:else}
      {#each filteredLogs as entry}
        <div class="{LEVEL_CLASS[entry.level] ?? 'log-info'} whitespace-pre-wrap break-all">
          {entry.msg}
        </div>
      {/each}
    {/if}
  </div>

  <!-- Status bar -->
  <div class="flex items-center justify-between px-3 py-1 border-t border-[var(--color-border)] bg-[var(--color-panel)] font-mono text-[9px] text-[var(--color-subtext2)]">
    <span>
      {filteredLogs.length}/{encoder.logs.length} entrée{encoder.logs.length > 1 ? "s" : ""}
      {#if filteredLogs.length !== encoder.logs.length}<span class="text-[var(--color-subtext2)] ml-1">(filtrées)</span>{/if}
    </span>
    {#if encoder.logs.length >= 400}
      <span class="text-[var(--color-warning)]">⚠ LIMITE 500</span>
    {/if}
  </div>
</div>
<style>
  .filter-btn {
    padding: 2px 6px;
    border-radius: 2px;
    background: transparent;
    cursor: pointer;
    font-family: "Geist Mono", monospace;
  }
  .filter-btn:hover {
    opacity: 0.8;
  }
  .log-action-btn {
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 0;
    display: inline-flex;
    align-items: center;
    transition: color 0.1s;
  }
  .log-action-btn:hover {
    color: var(--color-text);
  }
</style>