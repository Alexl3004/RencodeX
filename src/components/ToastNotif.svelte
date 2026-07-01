<script lang="ts">
  import { CircleCheck, CircleAlert, Info, AlertTriangle, X } from "@lucide/svelte";
  import { toasts, type Toast } from "$lib/stores/toasts.svelte";

  const CONFIG = {
    success: { Icon: CircleCheck,   accent: "var(--color-success)", },
    error:   { Icon: CircleAlert,   accent: "var(--color-danger)",  },
    warn:    { Icon: AlertTriangle,  accent: "var(--color-warning)", },
    info:    { Icon: Info,           accent: "var(--color-accent)",  },
  } as const;

  let hoveredId = $state<number | null>(null);

  function dismiss(id: number) {
    toasts.remove(id);
  }

  function progressPct(t: Toast, now: number): number {
    if (t.duration <= 0) return 100;
    const elapsed = now - t.addedAt;
    return Math.max(0, 100 - (elapsed / t.duration) * 100);
  }

  let now = $state(Date.now());
  $effect(() => {
    const id = setInterval(() => { now = Date.now(); }, 50);
    return () => clearInterval(id);
  });
</script>

<div class="toast-stack" role="region" aria-label="Notifications" aria-live="polite">
  {#each toasts.items as toast (toast.id)}
    {@const cfg = CONFIG[toast.type]}
    {@const pct = progressPct(toast, now)}
    <div
      class="toast"
      style="--accent: {cfg.accent};"
      onmouseenter={() => hoveredId = toast.id}
      onmouseleave={() => hoveredId = null}
      role="alert"
    >
      <!-- Barre de progression -->
      {#if toast.duration > 0}
        <div class="toast-progress-track">
          <div
            class="toast-progress-bar"
            style="width: {pct}%; transition: width {hoveredId === toast.id ? 'none' : '50ms linear'};"
          ></div>
        </div>
      {/if}

      <!-- Corps -->
      <div class="toast-body">
        <span class="toast-icon">
          <cfg.Icon class="w-4 h-4" />
        </span>

        <div class="toast-text">
          {#if toast.title}
            <span class="toast-title">{toast.title}</span>
          {/if}
          <span class="toast-msg" class:toast-msg--sub={!!toast.title}>{toast.message}</span>
        </div>

        {#if toast.count > 1}
          <span class="toast-count" title="{toast.count} occurrences">{toast.count}</span>
        {/if}

        <button
          class="toast-close"
          onclick={() => dismiss(toast.id)}
          aria-label="Fermer"
        >
          <X class="w-3 h-3" />
        </button>
      </div>

      <!-- Action CTA -->
      {#if toast.action}
        <div class="toast-action-row">
          <button
            class="toast-action-btn"
            onclick={() => { toast.action!.onClick(); dismiss(toast.id); }}
          >
            {toast.action.label}
          </button>
        </div>
      {/if}
    </div>
  {/each}
</div>

<style>
  .toast-stack {
    position: fixed;
    bottom: 1.25rem;
    right: 1.25rem;
    z-index: 9999;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    pointer-events: none;
    align-items: flex-end;
  }

  .toast {
    pointer-events: all;
    min-width: 260px;
    max-width: 380px;
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    border-left: 3px solid var(--accent);
    border-radius: var(--radius-md);
    box-shadow: 0 8px 24px rgba(0,0,0,0.35), 0 2px 6px rgba(0,0,0,0.2);
    overflow: hidden;
    animation: toast-in 0.22s cubic-bezier(0.34, 1.4, 0.64, 1) both;
  }

  @keyframes toast-in {
    from { opacity: 0; transform: translateX(20px) scale(0.96); }
    to   { opacity: 1; transform: translateX(0)    scale(1);    }
  }

  /* Progression */
  .toast-progress-track {
    height: 2px;
    background: color-mix(in srgb, var(--accent) 20%, transparent);
  }

  .toast-progress-bar {
    height: 100%;
    background: var(--accent);
    opacity: 0.75;
  }

  /* Corps */
  .toast-body {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    padding: 0.625rem 0.75rem;
  }

  .toast-icon {
    color: var(--accent);
    flex-shrink: 0;
    margin-top: 1px;
  }

  .toast-text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .toast-title {
    font-family: var(--font-mono, monospace);
    font-size: 11px;
    font-weight: 700;
    color: var(--color-text);
    line-height: 1.3;
  }

  .toast-msg {
    font-family: var(--font-mono, monospace);
    font-size: 11px;
    color: var(--color-text);
    line-height: 1.4;
    word-break: break-word;
  }

  .toast-msg--sub {
    font-size: 10px;
    color: var(--color-subtext);
  }

  /* Badge dédoublonnage */
  .toast-count {
    flex-shrink: 0;
    font-family: var(--font-mono, monospace);
    font-size: 9px;
    font-weight: 700;
    padding: 1px 5px;
    border-radius: 999px;
    background: var(--accent);
    color: #fff;
    min-width: 18px;
    text-align: center;
    align-self: center;
  }

  /* Fermer */
  .toast-close {
    flex-shrink: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: var(--radius-xs);
    border: none;
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    transition: color 0.1s, background 0.1s;
    margin-top: 0;
  }

  .toast-close:hover {
    color: var(--color-text);
    background: var(--color-surface);
  }

  /* Action */
  .toast-action-row {
    padding: 0 0.75rem 0.625rem;
    display: flex;
    justify-content: flex-end;
  }

  .toast-action-btn {
    font-family: var(--font-mono, monospace);
    font-size: 10px;
    font-weight: 600;
    padding: 3px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--accent);
    background: transparent;
    color: var(--accent);
    cursor: pointer;
    letter-spacing: 0.03em;
    transition: background 0.12s, color 0.12s;
  }

  .toast-action-btn:hover {
    background: var(--accent);
    color: #fff;
  }
</style>