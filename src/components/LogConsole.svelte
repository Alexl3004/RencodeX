<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { tick } from "svelte";
  import {
    Copy, CircleCheck, Trash2, X, AlertTriangle,
    Info, TriangleAlert, CircleX, CircleCheckBig, ScrollText,
  } from '@lucide/svelte';

  let { onClose }: { onClose?: () => void } = $props();

  // ── État ────────────────────────────────────────────────────────────────────
  let container = $state<HTMLDivElement | null>(null);
  let autoScroll = $state(true);
  let copied = $state(false);

  // ── Sections sidebar ────────────────────────────────────────────────────────
  type FilterId = "all" | "info" | "warn" | "error" | "success";

  const SECTIONS: { id: FilterId; label: string; icon: any; desc: string }[] = [
    { id: "all",     label: "Tout",            icon: ScrollText,   desc: "Toutes les entrées" },
    { id: "info",    label: "Info",            icon: Info,         desc: "Messages d'info" },
    { id: "warn",    label: "Avertissements",  icon: TriangleAlert, desc: "Warnings" },
    { id: "error",   label: "Erreurs",         icon: CircleX,      desc: "Erreurs critiques" },
    { id: "success", label: "Succès",          icon: CircleCheckBig, desc: "Opérations réussies" },
  ];

  let activeFilter = $state<FilterId>("all");

  // ── Compteurs dérivés ───────────────────────────────────────────────────────
  let countInfo    = $derived(encoder.logs.filter(e => e.level === "info").length);
  let countWarn    = $derived(encoder.logs.filter(e => e.level === "warn").length);
  let countError   = $derived(encoder.logs.filter(e => e.level === "error").length);
  let countSuccess = $derived(encoder.logs.filter(e => e.level === "success").length);

  function countFor(id: FilterId): number {
    if (id === "all")     return encoder.logs.length;
    if (id === "info")    return countInfo;
    if (id === "warn")    return countWarn;
    if (id === "error")   return countError;
    if (id === "success") return countSuccess;
    return 0;
  }

  // ── Logs filtrés ────────────────────────────────────────────────────────────
  let filteredLogs = $derived(
    activeFilter === "all"
      ? encoder.logs
      : encoder.logs.filter(e => e.level === activeFilter)
  );

  // ── Auto-scroll ─────────────────────────────────────────────────────────────
  $effect(() => {
    const _len = filteredLogs.length;
    if (autoScroll && container) {
      tick().then(() => {
        container!.scrollTop = container!.scrollHeight;
      });
    }
  });

  // ── Couleur par niveau ──────────────────────────────────────────────────────
  const LEVEL_CLASS: Record<string, string> = {
    info:    "log-info",
    warn:    "log-warning",
    error:   "log-error",
    success: "log-success",
  };

  // ── Actions ─────────────────────────────────────────────────────────────────
  async function copyAll() {
    await navigator.clipboard.writeText(
      encoder.logs.map(l => l.msg).join("\n"),
    );
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }
</script>

<div class="page-root" class:page-root--horizontal={encoder.innerNavLayout === "horizontal"}>

  <!-- ── Sidebar ──────────────────────────────────────────────────────────── -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <span class="sidebar-title">Journal</span>
      <span class="sidebar-sub">Logs · Activité</span>
    </div>

    <nav class="sidebar-nav" aria-label="Filtres">
      {#each SECTIONS as sec}
        {@const count = countFor(sec.id)}
        <button
          type="button"
          class="nav-item {activeFilter === sec.id ? 'nav-item--active' : ''}"
          onclick={() => (activeFilter = sec.id)}
          aria-current={activeFilter === sec.id ? "page" : undefined}
        >
          <div class="nav-item-icon nav-item-icon--{sec.id}">
            <sec.icon class="w-3.5 h-3.5" />
          </div>
          <div class="nav-item-text">
            <span class="nav-item-label">{sec.label}</span>
            <span class="nav-item-desc">{sec.desc}</span>
          </div>
          {#if count > 0}
            <span class="nav-badge nav-badge--{sec.id}">{count}</span>
          {/if}
          {#if activeFilter === sec.id}
            <div class="nav-item-indicator" aria-hidden="true"></div>
          {/if}
        </button>
      {/each}
    </nav>

    <!-- Sidebar footer : auto-scroll + actions -->
    <div class="sidebar-footer">
      <!-- Auto-scroll toggle -->
      <button
        type="button"
        role="switch"
        aria-checked={autoScroll}
        onclick={() => (autoScroll = !autoScroll)}
        class="autoscroll-toggle {autoScroll ? 'autoscroll-toggle--on' : ''}"
        aria-label="Auto-scroll"
      >
        <span class="switch-track {autoScroll ? 'switch-on' : 'switch-off'}">
          <span class="switch-thumb {autoScroll ? 'thumb-on' : 'thumb-off'}"></span>
        </span>
        <span class="toggle-label">Auto-scroll</span>
      </button>

      <!-- Actions -->
      <div class="footer-actions">
        <button
          type="button"
          onclick={copyAll}
          class="footer-btn {copied ? 'footer-btn--success' : ''}"
          aria-label={copied ? 'Copié' : 'Copier tous les logs'}
        >
          {#if copied}
            <CircleCheck class="w-3.5 h-3.5" />
            Copié
          {:else}
            <Copy class="w-3.5 h-3.5" />
            Copier tout
          {/if}
        </button>
        <button
          type="button"
          onclick={() => encoder.clearLogs()}
          class="footer-btn footer-btn--danger"
          aria-label="Effacer les logs"
        >
          <Trash2 class="w-3.5 h-3.5" />
          Effacer
        </button>
      </div>
    </div>
  </aside>

  <!-- ── Content panel ────────────────────────────────────────────────────── -->
  <div class="content-panel">
    <!-- Header content -->
    <header class="content-header">
      <div>
        <h2 class="content-title">
          {SECTIONS.find(s => s.id === activeFilter)?.label ?? "Journal"}
        </h2>
        <p class="content-desc">
          {filteredLogs.length} entrée{filteredLogs.length !== 1 ? "s" : ""}
          {#if activeFilter !== "all" && filteredLogs.length !== encoder.logs.length}
            <span class="desc-total">sur {encoder.logs.length} au total</span>
          {/if}
        </p>
      </div>
      <div class="header-right">
        {#if encoder.logs.length >= 400}
          <span class="warn-badge">
            <AlertTriangle class="w-3 h-3" />
            Limite 500
          </span>
        {/if}
        {#if onClose}
          <button
            type="button"
            onclick={onClose}
            class="close-btn"
            aria-label="Fermer le journal"
          >
            <X class="w-4 h-4" />
          </button>
        {/if}
      </div>
    </header>

    <!-- Log entries -->
    <div
      bind:this={container}
      class="log-body"
      onscroll={() => {
        if (!container) return;
        const atBottom = container.scrollHeight - container.scrollTop - container.clientHeight < 30;
        if (!atBottom && autoScroll) autoScroll = false;
      }}
    >
      {#if filteredLogs.length === 0}
        <div class="empty-state">
          <ScrollText class="w-8 h-8" style="color:var(--color-subtext2);" />
          <span class="empty-label">
            {encoder.logs.length === 0 ? "Aucune activité" : "Aucune entrée pour ce filtre"}
          </span>
          <span class="empty-sub">
            {encoder.logs.length === 0
              ? "Les logs apparaîtront ici lors d'un encodage."
              : "Sélectionnez un autre filtre pour voir les entrées."}
          </span>
        </div>
      {:else}
        {#each filteredLogs as entry}
          <div class="log-row {LEVEL_CLASS[entry.level] ?? 'log-info'}">
            <span class="log-dot"></span>
            <span class="log-msg">{entry.msg}</span>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</div>

<style>
  /* ── Layout ─────────────────────────────────────────────────────────────── */
  .page-root {
    display: flex;
    height: 100%;
    overflow: hidden;
    background: var(--color-surface);
  }

  /* ── Sidebar ─────────────────────────────────────────────────────────────── */
  .sidebar {
    width: 200px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    background: var(--color-panel);
    border-right: 1px solid var(--color-border);
    overflow: hidden;
  }

  .sidebar-header {
    padding: 20px 16px 14px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }
  .sidebar-title {
    display: block;
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text);
    letter-spacing: -0.01em;
  }
  .sidebar-sub {
    display: block;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    letter-spacing: 0.06em;
    color: var(--color-subtext2);
    margin-top: 3px;
    text-transform: uppercase;
  }

  .sidebar-nav {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  /* Nav items */
  .nav-item {
    position: relative;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid transparent;
    background: transparent;
    cursor: pointer;
    text-align: left;
    transition: background 0.1s, border-color 0.1s;
    width: 100%;
  }
  .nav-item:hover { background: color-mix(in srgb, var(--color-muted) 30%, transparent); }
  .nav-item--active {
    background: color-mix(in srgb, var(--color-accent) 9%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 22%, var(--color-border));
  }

  .nav-item-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-xs);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    color: var(--color-subtext);
    flex-shrink: 0;
    transition: background 0.1s, color 0.1s, border-color 0.1s;
  }
  .nav-item--active .nav-item-icon {
    background: color-mix(in srgb, var(--color-accent) 12%, var(--color-surface));
    border-color: color-mix(in srgb, var(--color-accent) 30%, var(--color-border));
    color: var(--color-accent);
  }

  /* Couleurs icône par type — état inactif */
  .nav-item-icon--warn  { color: var(--color-warning); }
  .nav-item-icon--error { color: var(--color-danger); }
  .nav-item-icon--success { color: var(--color-success); }

  .nav-item-text {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
    flex: 1;
  }
  .nav-item-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--color-subtext);
    line-height: 1.2;
    transition: color 0.1s;
  }
  .nav-item--active .nav-item-label {
    color: var(--color-accent);
    font-weight: 600;
  }
  .nav-item-desc {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext2);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .nav-item-indicator {
    position: absolute;
    right: 0;
    top: 50%;
    transform: translateY(-50%);
    width: 3px;
    height: 18px;
    border-radius: 2px 0 0 2px;
    background: var(--color-accent);
  }

  /* Badges de comptage */
  .nav-badge {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    font-weight: 600;
    padding: 1px 5px;
    border-radius: var(--radius-full, 999px);
    background: var(--color-muted);
    color: var(--color-subtext2);
    flex-shrink: 0;
    margin-right: 10px;
  }
  .nav-badge--warn    { background: color-mix(in srgb, var(--color-warning) 15%, transparent); color: var(--color-warning); }
  .nav-badge--error   { background: color-mix(in srgb, var(--color-danger)  15%, transparent); color: var(--color-danger); }
  .nav-badge--success { background: color-mix(in srgb, var(--color-success) 15%, transparent); color: var(--color-success); }

  /* ── Sidebar footer ──────────────────────────────────────────────────────── */
  .sidebar-footer {
    padding: 12px 12px;
    border-top: 1px solid var(--color-border);
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  /* Auto-scroll toggle */
  .autoscroll-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    background: transparent;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 7px 10px;
    cursor: pointer;
    width: 100%;
    transition: border-color 0.15s, background 0.15s;
  }
  .autoscroll-toggle--on {
    border-color: color-mix(in srgb, var(--color-accent) 30%, var(--color-border));
    background: color-mix(in srgb, var(--color-accent) 6%, transparent);
  }
  .toggle-label {
    font-size: 11px;
    font-weight: 500;
    color: var(--color-subtext);
  }

  /* Switch inline */
  .switch-track {
    position: relative;
    display: inline-block;
    width: 28px;
    height: 15px;
    border-radius: 999px;
    border: 1px solid;
    transition: background 0.15s, border-color 0.15s;
    flex-shrink: 0;
  }
  .switch-on  { background: var(--color-accent); border-color: var(--color-accent); }
  .switch-off { background: var(--color-muted);  border-color: var(--color-border2, var(--color-border)); }
  .switch-thumb {
    position: absolute;
    top: 50%;
    transform: translateY(-50%);
    width: 11px;
    height: 11px;
    border-radius: 50%;
    background: white;
    transition: left 0.15s;
  }
  .thumb-on  { left: 15px; }
  .thumb-off { left: 2px; }

  /* Action buttons */
  .footer-actions {
    display: flex;
    gap: 6px;
  }
  .footer-btn {
    flex: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    padding: 7px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    font-size: 11px;
    font-weight: 500;
    color: var(--color-subtext);
    cursor: pointer;
    transition: border-color 0.1s, color 0.1s, background 0.1s;
  }
  .footer-btn:hover { border-color: var(--color-subtext2); color: var(--color-text); }
  .footer-btn--success {
    border-color: color-mix(in srgb, var(--color-success) 30%, var(--color-border));
    color: var(--color-success);
    background: color-mix(in srgb, var(--color-success) 7%, var(--color-surface));
  }
  .footer-btn--danger:hover {
    border-color: color-mix(in srgb, var(--color-danger) 40%, var(--color-border));
    color: var(--color-danger);
    background: color-mix(in srgb, var(--color-danger) 6%, var(--color-surface));
  }

  /* ── Content panel ───────────────────────────────────────────────────────── */
  .content-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .content-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
    padding: 16px 20px 14px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }
  .content-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text);
    letter-spacing: -0.01em;
    margin: 0 0 4px;
  }
  .content-desc {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext2);
    margin: 0;
  }
  .desc-total {
    opacity: 0.7;
    margin-left: 4px;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }
  .warn-badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-warning);
    background: color-mix(in srgb, var(--color-warning) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-warning) 25%, transparent);
    padding: 3px 8px;
    border-radius: var(--radius-full, 999px);
  }
  .close-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-xs);
    border: 1px solid var(--color-border);
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    transition: border-color 0.1s, color 0.1s;
  }
  .close-btn:hover { border-color: var(--color-subtext2); color: var(--color-text); }

  /* ── Log body ────────────────────────────────────────────────────────────── */
  .log-body {
    flex: 1;
    overflow-y: auto;
    padding: 12px 16px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .log-row {
    display: flex;
    align-items: baseline;
    gap: 8px;
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    line-height: 1.6;
    padding: 3px 8px;
    border-radius: var(--radius-xs);
    border: 1px solid transparent;
  }
  .log-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    flex-shrink: 0;
    margin-top: 5px;
  }
  .log-msg {
    flex: 1;
    white-space: pre-wrap;
    word-break: break-all;
  }

  /* Niveaux */
  .log-info {
    color: var(--color-subtext);
  }
  .log-info .log-dot { background: var(--color-subtext2); }

  .log-warning {
    color: var(--color-warning);
    background: color-mix(in srgb, var(--color-warning) 5%, transparent);
    border-color: color-mix(in srgb, var(--color-warning) 12%, transparent);
  }
  .log-warning .log-dot { background: var(--color-warning); }

  .log-error {
    color: var(--color-danger);
    background: color-mix(in srgb, var(--color-danger) 5%, transparent);
    border-color: color-mix(in srgb, var(--color-danger) 12%, transparent);
  }
  .log-error .log-dot { background: var(--color-danger); }

  .log-success {
    color: var(--color-success);
    background: color-mix(in srgb, var(--color-success) 5%, transparent);
    border-color: color-mix(in srgb, var(--color-success) 12%, transparent);
  }
  .log-success .log-dot { background: var(--color-success); }

  /* ── Empty state ─────────────────────────────────────────────────────────── */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    padding: 48px 24px;
    gap: 8px;
    text-align: center;
  }
  .empty-label {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-subtext);
    margin-top: 4px;
  }
  .empty-sub {
    font-size: 11px;
    color: var(--color-subtext2);
    max-width: 260px;
    line-height: 1.5;
  }

  /* ── Layout horizontal ──────────────────────────────────────────────────── */
  .page-root--horizontal {
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  .page-root--horizontal .sidebar {
    width: 100%;
    height: auto;
    flex-direction: column;
    border-right: none;
    border-bottom: 1px solid var(--color-border);
    padding: 0;
    gap: 0;
    overflow: visible;
    flex-shrink: 0;
  }
  .page-root--horizontal .sidebar-header {
    display: none;
  }

  /* Ligne 1 : filtres — centrés */
  .page-root--horizontal .sidebar-nav {
    flex-direction: row;
    padding: 0 12px;
    gap: 2px;
    overflow-x: auto;
    overflow-y: visible;
    border-bottom: 1px solid var(--color-border);
    justify-content: center;
  }
  .page-root--horizontal .nav-item {
    flex-direction: row;
    min-height: 34px;
    width: auto;
    padding: 5px 12px;
    gap: 6px;
    border-left: none;
    border-bottom: none;
    border-radius: var(--radius-sm);
    white-space: nowrap;
  }
  .page-root--horizontal .nav-item--active {
    border-left-color: transparent;
  }
  .page-root--horizontal .nav-item-indicator {
    display: none;
  }
  .page-root--horizontal .nav-item-text {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 6px;
  }
  .page-root--horizontal .nav-item-desc {
    display: none;
  }

  /* Ligne 2 : auto-scroll + actions — alignés à droite */
  .page-root--horizontal .sidebar-footer {
    flex-direction: row;
    align-items: center;
    justify-content: flex-end;
    padding: 4px 12px;
    gap: 8px;
    border-top: none;
  }
  .page-root--horizontal .autoscroll-toggle {
    width: fit-content;
    padding: 4px 10px;
    min-height: 28px;
  }
  .page-root--horizontal .footer-actions {
    flex-direction: row;
    gap: 6px;
  }
  .page-root--horizontal .footer-btn {
    flex: none;
    padding: 4px 10px;
    font-size: 11px;
  }

  .page-root--horizontal .content-panel {
    flex: 1 1 0;
    min-height: 0;
    min-width: 0;
    overflow-y: auto;
  }
</style>