<script lang="ts">
  import { CircleCheck, CircleAlert, Info, AlertTriangle, X } from "@lucide/svelte";
  import { toasts, type Toast } from "$lib/stores/toasts.svelte";

  const CONFIG = {
    success: { Icon: CircleCheck,  accent: "var(--color-success)", bg: "color-mix(in srgb, var(--color-success) 8%, var(--color-panel))"  },
    error:   { Icon: CircleAlert,  accent: "var(--color-danger)",  bg: "color-mix(in srgb, var(--color-danger)  8%, var(--color-panel))"  },
    warn:    { Icon: AlertTriangle, accent: "var(--color-warning)", bg: "color-mix(in srgb, var(--color-warning) 8%, var(--color-panel))"  },
    info:    { Icon: Info,         accent: "var(--color-accent)",  bg: "color-mix(in srgb, var(--color-accent)  8%, var(--color-panel))"  },
  } as const;

  let hoveredId = $state<number | null>(null);

  function dismiss(id: number) { toasts.remove(id); }

  function progressPct(t: Toast, now: number): number {
    if (t.duration <= 0) return 100;
    return Math.max(0, 100 - ((now - t.addedAt) / t.duration) * 100);
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
    {@const paused = hoveredId === toast.id}

    <div
      class="toast"
      style="--accent: {cfg.accent}; --toast-bg: {cfg.bg};"
      onmouseenter={() => (hoveredId = toast.id)}
      onmouseleave={() => (hoveredId = null)}
      role="alert"
    >
      <!-- Barre de progression (haut) -->
      {#if toast.duration > 0}
        <div class="toast-progress-track">
          <div
            class="toast-progress-bar"
            style="width: {pct}%; transition: width {paused ? 'none' : '50ms linear'};"
          ></div>
        </div>
      {/if}

      <!-- Corps -->
      <div class="toast-body">
        <!-- Icône dans bulle colorée -->
        <div class="toast-icon-wrap">
          <cfg.Icon class="w-4 h-4" />
        </div>

        <div class="toast-text">
          {#if toast.title}
            <span class="toast-title">{toast.title}</span>
          {/if}
          <span class="toast-msg" class:toast-msg--sub={!!toast.title}>{toast.message}</span>
        </div>

        <div class="toast-right">
          {#if toast.count > 1}
            <span class="toast-count" title="{toast.count} occurrences">×{toast.count}</span>
          {/if}
          <button class="toast-close" onclick={() => dismiss(toast.id)} aria-label="Fermer">
            <X class="w-3 h-3" />
          </button>
        </div>
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
  /* ── Conteneur ── */
  .toast-stack {
    position: fixed;
    top: 1rem;
    right: 1rem;
    z-index: 9999;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    pointer-events: none;
    align-items: flex-end;
    width: 360px;
  }

  /* ── Carte toast ── */
  .toast {
    pointer-events: all;
    width: 100%;
    background: var(--toast-bg);
    border: 1px solid color-mix(in srgb, var(--accent) 22%, var(--color-border));
    border-radius: var(--radius-md);
    box-shadow:
      0 4px 16px rgba(0, 0, 0, 0.28),
      0 1px 4px  rgba(0, 0, 0, 0.16),
      inset 0 1px 0 rgba(255, 255, 255, 0.04);
    overflow: hidden;
    animation: toast-in 0.25s cubic-bezier(0.34, 1.3, 0.64, 1) both;
    backdrop-filter: blur(8px);
  }

  @keyframes toast-in {
    from { opacity: 0; transform: translateX(16px) scale(0.97); }
    to   { opacity: 1; transform: translateX(0)    scale(1);    }
  }

  /* ── Barre de progression ── */
  .toast-progress-track {
    height: 2px;
    background: color-mix(in srgb, var(--accent) 15%, transparent);
  }
  .toast-progress-bar {
    height: 100%;
    background: var(--accent);
    opacity: 0.8;
    border-radius: 0 1px 1px 0;
  }

  /* ── Corps ── */
  .toast-body {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 12px 12px 12px 14px;
  }

  /* Icône dans cercle coloré semi-transparent */
  .toast-icon-wrap {
    flex-shrink: 0;
    width: 32px;
    height: 32px;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--accent) 16%, transparent);
    border: 1px solid color-mix(in srgb, var(--accent) 25%, transparent);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--accent);
    margin-top: 1px;
  }

  /* Texte */
  .toast-text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    padding-top: 4px;
  }

  .toast-title {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    font-weight: 700;
    color: var(--color-text);
    line-height: 1.3;
    letter-spacing: 0.01em;
  }

  .toast-msg {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    color: var(--color-text);
    line-height: 1.45;
    word-break: break-word;
  }

  .toast-msg--sub {
    font-size: 10px;
    color: var(--color-subtext);
  }

  /* Zone droite : badge + bouton fermer */
  .toast-right {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
    padding-top: 2px;
  }

  /* Badge dédoublonnage */
  .toast-count {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 700;
    padding: 1px 5px;
    border-radius: var(--radius-full);
    background: var(--accent);
    color: #fff;
    min-width: 20px;
    text-align: center;
    line-height: 1.6;
  }

  /* Bouton fermer */
  .toast-close {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: var(--radius-xs);
    border: 1px solid transparent;
    background: transparent;
    color: var(--color-subtext2);
    cursor: pointer;
    transition: color 0.1s, background 0.1s, border-color 0.1s;
  }
  .toast-close:hover {
    color: var(--color-text);
    background: color-mix(in srgb, var(--color-border) 60%, transparent);
    border-color: var(--color-border);
  }

  /* CTA action */
  .toast-action-row {
    padding: 0 12px 10px 14px;
    display: flex;
    justify-content: flex-end;
  }
  .toast-action-btn {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    font-weight: 600;
    padding: 4px 12px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--accent);
    background: transparent;
    color: var(--accent);
    cursor: pointer;
    letter-spacing: 0.04em;
    transition: background 0.12s, color 0.12s;
  }
  .toast-action-btn:hover {
    background: var(--accent);
    color: #fff;
  }
</style>