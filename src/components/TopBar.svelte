<script lang="ts">
  import { theme } from "$lib/stores/theme.svelte";
  import { encoder } from "$lib/stores/encoder.svelte";
  import { formatSize } from "$lib/utils";
  import LangPopover from "$components/LangPopover.svelte";
  import {
    Sun, Moon, SlidersHorizontal, Tags,
    Terminal , RefreshCw, Cpu, ChartColumnDecreasing, Home, Settings,
  } from "@lucide/svelte";

  // ── Onglets ───────────────────────────────────────────────────────────────
  type TabId = "home" | "encoding" | "renaming" | "stats" | "logs" | "settings";

  const TABS: { id: TabId; label: string; icon: any; badge?: () => number | null }[] = [
    { id: "home",     label: "Accueil",      icon: Home },
    { id: "encoding", label: "Encodage",     icon: SlidersHorizontal },
    { id: "renaming", label: "Nommage",      icon: Tags },
    { id: "stats",    label: "Statistiques", icon: ChartColumnDecreasing },
    { id: "logs",     label: "Logs",         icon: Terminal ,
      badge: () => (errorCount + warnCount) || null },
    { id: "settings", label: "Paramètres",  icon: Settings },
  ];

  let activeTab = $state<TabId>("home");

  function setTab(id: TabId) {
    activeTab = id;
  }

  // Expose activeTab et TABS pour que +page.svelte pilote la sidebar
  export { activeTab, TABS };

  // ── Refresh ───────────────────────────────────────────────────────────────
  async function handleRefresh() {
    encoder.clearSession();
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      encoder.outputDir = await invoke<string>("get_default_output_dir");
    } catch { /* silently ignore */ }
  }

  // ── Métriques ─────────────────────────────────────────────────────────────
  let totalSize  = $derived(encoder.files.reduce((a, f) => a + (f.size_mb ?? 0), 0));
  let readyCount = $derived(encoder.files.filter(f => f.status === "ready").length);
  let doneCount  = $derived(encoder.files.filter(f => f.status === "done").length);
  let errorCount = $derived(encoder.logs.filter(e => e.level === "error").length);
  let warnCount  = $derived(encoder.logs.filter(e => e.level === "warn").length);

  type StatusKind = "encoding" | "done" | "ready" | "idle";
  let statusKind = $derived<StatusKind>(
    encoder.encoding                                       ? "encoding"
    : doneCount > 0 && doneCount === encoder.files.length ? "done"
    : encoder.files.length > 0                            ? "ready"
    :                                                       "idle"
  );
  let statusLabel = $derived(
    statusKind === "encoding"
      ? `ENCODING${encoder.progress ? " · " + encoder.progress.percent.toFixed(0) + "%" : ""}`
    : statusKind === "done"
      ? `DONE · ${doneCount} fichier${doneCount > 1 ? "s" : ""}`
    : statusKind === "ready"
      ? `${readyCount}/${encoder.files.length} prêt${readyCount > 1 ? "s" : ""}${totalSize > 0 ? " · " + formatSize(totalSize) : ""}`
    : "IDLE"
  );
</script>

<!-- ═══════════════════════════════════════════════════════════════════════ -->
<div class="topbar-root">
  <!-- ── Rangée 1 : identité + status + utilitaires ──────────────────────── -->
  <header class="row-top">
    <!-- Logo -->
    <div class="logo-zone">
      <div class="logo-icon" aria-hidden="true">
        <Cpu class="w-[14px] h-[14px]" />
      </div>
      <span class="logo-name">RenCodeX</span>
      <span class="logo-badge">H.265 · NVENC</span>
    </div>

    <!-- Status -->
    <div class="status-zone" aria-live="polite" aria-atomic="true">
      <span class="status-pill status-pill--{statusKind}">
        <span class="status-dot {statusKind === 'encoding' ? 'dot-pulse' : ''}" aria-hidden="true"></span>
        {statusLabel}
      </span>
    </div>

    <!-- Utilitaires -->
    <div class="utils-zone">
      <LangPopover />
      <div class="v-sep" aria-hidden="true"></div>
      <button
        type="button"
        onclick={handleRefresh}
        class="util-btn"
        title="Vider la session"
        aria-label="Vider la session"
      >
        <RefreshCw class="w-[14px] h-[14px]" aria-hidden="true" />
      </button>
      <div class="v-sep" aria-hidden="true"></div>
      <button
        type="button"
        onclick={() => theme.toggle()}
        class="util-btn"
        aria-label={theme.dark ? "Mode clair" : "Mode sombre"}
        title={theme.dark ? "Passer en mode clair" : "Passer en mode sombre"}
      >
        {#if theme.dark}
          <Sun  class="w-[14px] h-[14px]" aria-hidden="true" />
        {:else}
          <Moon class="w-[14px] h-[14px]" aria-hidden="true" />
        {/if}
      </button>
    </div>
  </header>
</div>

<style>
  /* ── Conteneur global ────────────────────────────────────────────────────── */
  .topbar-root {
    flex-shrink: 0;
    background: var(--color-panel);
    border-bottom: 1px solid var(--color-border);
    user-select: none;
  }

  /* ── Rangée 1 ────────────────────────────────────────────────────────────── */
  .row-top {
    display: flex;
    align-items: center;
    height: 40px;
    padding: 0 12px;
    gap: 10px;
    border-bottom: 1px solid var(--color-border);
  }

  /* Logo */
  .logo-zone {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }
  .logo-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--color-accent) 14%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-accent) 26%, var(--color-border));
    color: var(--color-accent);
    flex-shrink: 0;
  }
  .logo-name {
    font-size: 13px;
    font-weight: 600;
    letter-spacing: -0.025em;
    color: var(--color-text);
    line-height: 1;
  }
  .logo-badge {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    letter-spacing: 0.06em;
    padding: 2px 7px;
    border-radius: var(--radius-full);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    color: var(--color-subtext2);
    line-height: 1.6;
    white-space: nowrap;
  }

  /* Status */
  .status-zone {
    flex: 1;
    display: flex;
    justify-content: center;
  }
  .status-pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 3px 10px;
    border-radius: var(--radius-full);
    border: 1px solid transparent;
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    font-weight: 500;
    letter-spacing: 0.05em;
    white-space: nowrap;
    line-height: 1;
  }
  .status-pill--encoding {
    background: color-mix(in srgb, var(--color-accent) 10%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 28%, var(--color-border));
    color: var(--color-accent);
  }
  .status-pill--done {
    background: color-mix(in srgb, var(--color-success) 10%, transparent);
    border-color: color-mix(in srgb, var(--color-success) 28%, var(--color-border));
    color: var(--color-success);
  }
  .status-pill--ready {
    background: var(--color-surface);
    border-color: var(--color-border);
    color: var(--color-subtext);
  }
  .status-pill--idle {
    color: var(--color-subtext2);
  }
  .status-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: currentColor;
    flex-shrink: 0;
    opacity: 0.7;
  }
  .dot-pulse { animation: pulse 1.8s ease-in-out infinite; }

  /* Utilitaires */
  .utils-zone {
    display: flex;
    align-items: center;
    gap: 2px;
    flex-shrink: 0;
  }
  .v-sep {
    width: 1px;
    height: 14px;
    background: var(--color-border);
    margin: 0 4px;
    flex-shrink: 0;
  }
  .util-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-xs);
    border: 1px solid transparent;
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    transition: background 0.1s, border-color 0.1s, color 0.1s;
  }
  .util-btn:hover {
    background: var(--color-panel2);
    border-color: var(--color-border);
    color: var(--color-text);
  }
  .util-btn:active { transform: translateY(1px); }
  .util-btn--on {
    background: color-mix(in srgb, var(--color-accent) 10%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 28%, var(--color-border));
    color: var(--color-accent);
  }

  /* ── Animations ──────────────────────────────────────────────────────────── */
  @keyframes pulse {
    0%, 100% { opacity: 0.7; }
    50%       { opacity: 0.2; }
  }

  .nav-tab:focus-visible,
  .util-btn:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 2px;
  }

  @media (prefers-reduced-motion: reduce) {
    .dot-pulse { animation: none; }
  }
</style>