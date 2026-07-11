<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { Zap, Volume2, Box, Subtitles, Languages } from "@lucide/svelte";

  import PresetsTab from "./PresetsTab.svelte";
  import AudioTab from "./AudioTab.svelte";
  import ContainerTab from "./ContainerTab.svelte";
  import SubtitlesTab from "./SubtitlesTab.svelte";
  import LanguagesTab from "./LanguagesTab.svelte";

  let { onClose }: { onClose?: () => void } = $props();

  type SectionId =
    | "presets"
    | "audio"
    | "container"
    | "subtitles"
    | "languages";

  const SECTIONS: { id: SectionId; label: string; icon: any; desc: string }[] =
    [
      {
        id: "presets",
        label: "Préréglages",
        icon: Zap,
        desc: "Qualité & vitesse",
      },
      { id: "audio", label: "Audio", icon: Volume2, desc: "Codec & débit" },
      { id: "container", label: "Conteneur", icon: Box, desc: "MKV / MP4" },
      {
        id: "subtitles",
        label: "Sous-titres",
        icon: Subtitles,
        desc: "Extraction",
      },
      {
        id: "languages",
        label: "Pistes",
        icon: Languages,
        desc: "Ordre de priorité",
      },
    ];

  let activeSection = $state<SectionId>("presets");
</script>

<div
  class="page-root"
  class:page-root--horizontal={encoder.innerNavLayout === "horizontal"}
>
  <!-- ── Sidebar navigation ─────────────────────────────────────────────── -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <span class="sidebar-title">Encodage</span>
      <span class="sidebar-sub">H.265 · NVENC</span>
    </div>

    <nav class="sidebar-nav" aria-label="Sections">
      {#each SECTIONS as sec}
        <button
          type="button"
          class="nav-item {activeSection === sec.id ? 'nav-item--active' : ''}"
          onclick={() => (activeSection = sec.id)}
          aria-current={activeSection === sec.id ? "page" : undefined}
        >
          <div class="nav-item-icon">
            <sec.icon class="w-3.5 h-3.5" />
          </div>
          <div class="nav-item-text">
            <span class="nav-item-label">{sec.label}</span>
            <span class="nav-item-desc">{sec.desc}</span>
          </div>
          {#if activeSection === sec.id}
            <div class="nav-item-indicator" aria-hidden="true"></div>
          {/if}
        </button>
      {/each}
    </nav>
  </aside>

  <!-- ── Content panel ───────────────────────────────────────────────────── -->
  <div class="content-panel">
    {#if activeSection === "presets"}
      <PresetsTab />
    {:else if activeSection === "audio"}
      <AudioTab />
    {:else if activeSection === "container"}
      <ContainerTab />
    {:else if activeSection === "subtitles"}
      <SubtitlesTab />
    {:else if activeSection === "languages"}
      <LanguagesTab />
    {/if}
  </div>
</div>

<style>
  /* ── Layout racine ──────────────────────────────────────────────────────── */
  .page-root {
    display: flex;
    height: 100%;
    overflow: hidden;
    background: var(--color-surface);
  }

  /* ── Sidebar ────────────────────────────────────────────────────────────── */
  .sidebar {
    width: 220px;
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
    padding: 8px 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

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
    transition:
      background 0.1s,
      border-color 0.1s;
    width: 100%;
  }
  .nav-item:hover {
    background: color-mix(in srgb, var(--color-muted) 30%, transparent);
  }
  .nav-item--active {
    background: color-mix(in srgb, var(--color-accent) 9%, transparent);
    border-color: color-mix(
      in srgb,
      var(--color-accent) 22%,
      var(--color-border)
    );
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
    transition:
      background 0.1s,
      color 0.1s,
      border-color 0.1s;
  }
  .nav-item--active .nav-item-icon {
    background: color-mix(
      in srgb,
      var(--color-accent) 12%,
      var(--color-surface)
    );
    border-color: color-mix(
      in srgb,
      var(--color-accent) 30%,
      var(--color-border)
    );
    color: var(--color-accent);
  }

  .nav-item-text {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
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

  /* ── Content panel ──────────────────────────────────────────────────────── */
  .content-panel {
    flex: 1;
    overflow-y: auto;
    padding: 0;
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
    flex-direction: row;
    align-items: center;
    border-right: none;
    border-bottom: 1px solid var(--color-border);
    padding: 0 12px;
    gap: 0;
    overflow-x: auto;
    overflow-y: visible;
    flex-shrink: 0;
  }
  .page-root--horizontal .sidebar-header {
    display: none;
  }
  .page-root--horizontal .sidebar-nav {
    flex-direction: row;
    padding: 0;
    gap: 2px;
    overflow: visible;
    flex: 1;
    justify-content: center;
  }
  .page-root--horizontal .nav-item {
    flex-direction: row;
    min-height: 36px;
    width: auto;
    padding: 6px 14px;
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
  .page-root--horizontal .content-panel {
    flex: 1 1 0;
    min-height: 0;
    min-width: 0;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    align-items: center;
  }
</style>
