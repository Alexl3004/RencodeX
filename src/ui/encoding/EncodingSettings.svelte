<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { Zap, Video, Volume2, Box, Subtitles, Languages } from "@lucide/svelte";

  import PresetsTab   from "./PresetsTab.svelte";
  import VideoTab     from "./VideoTab.svelte";
  import AudioTab     from "./AudioTab.svelte";
  import ContainerTab from "./ContainerTab.svelte";
  import SubtitlesTab from "./SubtitlesTab.svelte";
  import LanguagesTab from "./LanguagesTab.svelte";

  let { onClose }: { onClose?: () => void } = $props();

  type SectionId = "presets" | "video" | "audio" | "container" | "subtitles" | "languages";

  const SECTIONS: { id: SectionId; label: string; icon: any; desc: string }[] = [
    { id: "presets",   label: "Préréglages", icon: Zap,       desc: "Qualité & vitesse"   },
    { id: "video",     label: "Vidéo",       icon: Video,     desc: "Encodage / copie"    },
    { id: "audio",     label: "Audio",       icon: Volume2,   desc: "Codec & débit"       },
    { id: "container", label: "Conteneur",   icon: Box,       desc: "MKV / MP4"           },
    { id: "subtitles", label: "Sous-titres", icon: Subtitles, desc: "Extraction"          },
    { id: "languages", label: "Pistes",      icon: Languages, desc: "Ordre de priorité"   },
  ];

  let activeSection = $state<SectionId>("presets");
</script>

<div
  class="root"
  class:root--horizontal={encoder.innerNavLayout === "horizontal"}
>
  <!-- Sidebar -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <span class="sidebar-title">Encodage</span>
      <span class="sidebar-sub">
        {encoder.videoMode === "copy" ? "Copie · Remux" : "H.265 · NVENC"}
      </span>
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

  <!-- Contenu -->
  <div class="content">
    {#if activeSection === "presets"}
      <PresetsTab />
    {:else if activeSection === "video"}
      <VideoTab />
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
  .root {
    display: flex;
    height: 100%;
    overflow: hidden;
    background: var(--color-surface);
  }

  /* ── Sidebar ────────────────────────────────────────────────────────────── */
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
    font-family: "Geist Mono", monospace;
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text);
    letter-spacing: -0.02em;
  }
  .sidebar-sub {
    display: block;
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
    margin-top: 2px;
  }

  .sidebar-nav {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 8px 0;
    gap: 1px;
    overflow-y: auto;
  }

  .nav-item {
    position: relative;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 14px;
    min-height: 48px;
    width: 100%;
    background: transparent;
    border: none;
    border-left: 2px solid transparent;
    cursor: pointer;
    text-align: left;
    transition: background 0.12s, border-color 0.12s;
  }
  .nav-item:hover {
    background: color-mix(in srgb, var(--color-accent) 6%, transparent);
  }
  .nav-item--active {
    background: color-mix(in srgb, var(--color-accent) 10%, transparent);
    border-left-color: var(--color-accent);
  }

  .nav-item-icon {
    color: var(--color-subtext);
    flex-shrink: 0;
    display: flex;
    align-items: center;
  }
  .nav-item--active .nav-item-icon {
    color: var(--color-accent);
  }

  .nav-item-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .nav-item-label {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    font-weight: 500;
    color: var(--color-subtext);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .nav-item--active .nav-item-label {
    color: var(--color-text);
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
    height: 16px;
    border-radius: 2px 0 0 2px;
    background: var(--color-accent);
  }

  /* ── Contenu ────────────────────────────────────────────────────────────── */
  .content {
    flex: 1;
    overflow-y: auto;
  }

  /* ── Layout horizontal ──────────────────────────────────────────────────── */
  .root--horizontal {
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  .root--horizontal .sidebar {
    width: 100%;
    height: auto;
    flex-direction: row;
    align-items: center;
    border-right: none;
    border-bottom: 1px solid var(--color-border);
    padding: 0 10px;
    overflow-x: auto;
    overflow-y: visible;
    flex-shrink: 0;
  }
  .root--horizontal .sidebar-header { display: none; }
  .root--horizontal .sidebar-nav {
    flex-direction: row;
    padding: 0;
    gap: 2px;
    overflow: visible;
    flex: 1;
    justify-content: center;
  }
  .root--horizontal .nav-item {
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
  .root--horizontal .nav-item--active {
    border-left-color: transparent;
  }
  .root--horizontal .nav-item-indicator { display: none; }
  .root--horizontal .nav-item-text {
    flex-direction: row;
    align-items: center;
    gap: 6px;
  }
  .root--horizontal .nav-item-desc { display: none; }
  .root--horizontal .content {
    flex: 1 1 0;
    min-height: 0;
    min-width: 0;
    overflow-y: auto;
  }
</style>