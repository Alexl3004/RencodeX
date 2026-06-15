<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { tick } from "svelte";

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
          class="px-1.5 py-0.5 rounded-[2px] font-mono text-[9px] border transition-all
                 {showInfo ? 'text-[var(--color-subtext)] bg-[var(--color-surface)] border-[var(--color-border)]' : 'text-[var(--color-subtext2)] border-transparent'}">
          INFO
        </button>
        <button type="button" onclick={toggleWarn}
          class="px-1.5 py-0.5 rounded-[2px] font-mono text-[9px] border transition-all
                 {showWarn ? 'text-[var(--color-warning)] bg-[var(--color-warning)]/10 border-[var(--color-warning)]/30' : 'text-[var(--color-subtext2)] border-transparent'}">
          WARN
        </button>
        <button type="button" onclick={toggleError}
          class="px-1.5 py-0.5 rounded-[2px] font-mono text-[9px] border transition-all
                 {showError ? 'text-[var(--color-danger)] bg-[var(--color-danger)]/10 border-[var(--color-danger)]/30' : 'text-[var(--color-subtext2)] border-transparent'}">
          ERR
        </button>
        <button type="button" onclick={toggleSuccess}
          class="px-1.5 py-0.5 rounded-[2px] font-mono text-[9px] border transition-all
                 {showSuccess ? 'text-[var(--color-success)] bg-[var(--color-success)]/10 border-[var(--color-success)]/30' : 'text-[var(--color-subtext2)] border-transparent'}">
          OK
        </button>
      </div>
      <div class="sep h-3 mx-1" aria-hidden="true"></div>
      <label class="flex items-center gap-1 cursor-pointer font-mono text-[10px] text-[var(--color-subtext)]">
        <input type="checkbox" bind:checked={autoScroll} class="w-3 h-3 accent-[var(--color-accent)]" />
        auto
      </label>
      <div class="sep h-3 mx-1" aria-hidden="true"></div>
      <button type="button" onclick={copyAll}
        class="font-mono text-[10px] transition-colors flex items-center gap-1
               {copied ? 'text-[var(--color-success)]' : 'text-[var(--color-subtext)] hover:text-[var(--color-text)]'}">
        {#if copied}
          <svg width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <polyline points="20 6 9 17 4 12" />
          </svg>
          COPIÉ
        {:else}
          <svg width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
          </svg>
          COPY
        {/if}
      </button>
      <button type="button" onclick={() => encoder.clearLogs()}
        class="font-mono text-[10px] text-[var(--color-subtext)] hover:text-[var(--color-danger)] transition-colors flex items-center gap-1">
        <svg width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
          <polyline points="3 6 5 6 21 6" />
          <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
          <path d="M10 11v6" /><path d="M14 11v6" />
        </svg>
        CLR
      </button>
      {#if onClose}
        <div class="sep h-3 mx-1" aria-hidden="true"></div>
        <button type="button" onclick={onClose}
          class="font-mono text-[10px] text-[var(--color-subtext)] hover:text-[var(--color-text)] transition-colors">
          ✕
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