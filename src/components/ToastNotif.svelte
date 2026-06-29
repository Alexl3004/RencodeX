<script lang="ts">
  import { CircleAlert, CircleCheck, Info, X } from '@lucide/svelte';
  import { toasts } from "$lib/stores/toasts.svelte";

  const config = {
    success: { icon: CircleCheck, accent: "var(--color-success)" },
    error:   { icon: X,           accent: "var(--color-danger)"  },
    warn:    { icon: CircleAlert,  accent: "var(--color-warning)" },
    info:    { icon: Info,         accent: "var(--color-accent)"  },
  };

  let statuses = $state<Record<number, boolean>>({});

  $effect(() => {
    for (const t of toasts.items) {
      if (!(t.id in statuses)) {
        statuses[t.id] = true;
      }
    }
  });

  function dismiss(id: number) {
    statuses[id] = false;
    setTimeout(() => toasts.remove(id), 300);
  }
</script>

<div class="toast-stack">
  {#each toasts.items as toast (toast.id)}
    {@const cfg = config[toast.type]}
    <div
      class="toast-item"
      class:hidden={!(statuses[toast.id] ?? true)}
      style="--toast-accent: {cfg.accent}"
    >
      <span class="toast-icon">
        <cfg.icon class="w-4 h-4" />
      </span>
      <span class="toast-msg">{toast.message}</span>
      <button class="toast-close" onclick={() => dismiss(toast.id)}>
        <X class="w-3.5 h-3.5" />
      </button>
    </div>
  {/each}
</div>

<style>
  .toast-stack {
    position: fixed;
    top: 1.25rem;
    right: 1.25rem;
    z-index: 9999;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    pointer-events: none;
  }

  .toast-item {
    pointer-events: all;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    border-left: 3px solid var(--toast-accent);
    border-radius: 2px;
    min-width: 220px;
    max-width: 360px;
    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
  }

  .toast-item.hidden {
    opacity: 0;
    transform: translateX(8px);
    transition: opacity 0.3s, transform 0.3s;
  }

  .toast-icon {
    color: var(--toast-accent);
    flex-shrink: 0;
  }

  .toast-msg {
    flex: 1;
    font-size: 12px;
    color: var(--color-text);
    font-family: var(--font-mono, monospace);
    line-height: 1.4;
  }

  .toast-close {
    flex-shrink: 0;
    color: var(--color-subtext);
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    display: flex;
    align-items: center;
  }

  .toast-close:hover {
    color: var(--color-text);
  }
</style>