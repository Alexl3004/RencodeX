<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { Tag, AlignLeft, Users, Eye } from "@lucide/svelte";

  import TagsTab from "./TagsTab.svelte";
  import FormatTab from "./FormatTab.svelte";
  import TeamTab from "./TeamTab.svelte";
  import PreviewTab from "./PreviewTab.svelte";

  let { onClose }: { onClose?: () => void } = $props();

  // ── Navigation ─────────────────────────────────────────────────────────────

  type SectionId = "tags" | "format" | "team" | "preview";

  const SECTIONS: { id: SectionId; label: string; icon: any; desc: string }[] =
    [
      {
        id: "tags",
        label: "Ordre des tags",
        icon: Tag,
        desc: "Glisser · activer",
      },
      {
        id: "format",
        label: "Format",
        icon: AlignLeft,
        desc: "Casse · séparateurs",
      },
      { id: "team", label: "Team", icon: Users, desc: "non définie" },
      { id: "preview", label: "Aperçu", icon: Eye, desc: "Nom généré" },
    ];

  let activeSection = $state<SectionId>("tags");

  // ── État dérivé du store ───────────────────────────────────────────────────

  let tagOrder = $derived(encoder.tagOrder);
  let disabledTags = $derived(encoder.disabledTags);
  let resCase = $derived(encoder.resolutionCase);
  let titleCase = $derived(encoder.titleCase);
  let codecFmt = $derived(encoder.codecFormat);
  let sourceCase = $derived(encoder.sourceCase);
  let yearParentheses = $derived(encoder.yearParentheses);
  let webSourceFmt = $derived(encoder.webSourceFormat);
  let tagSep = $derived(encoder.tagSeparator);
  let provCase = $derived(encoder.providerCase);
  let seFormat = $derived(encoder.seasonEpisodeFormat);
  let team = $derived(encoder.team);
  let japVer = $derived(encoder.keepJapaneseVer);

  let activeTagCount = $derived(tagOrder.length - disabledTags.size);

  // ── Aperçu partagé ────────────────────────────────────────────────────────

  function previewName(mode: "series" | "movie" = "series") {
    if (mode === "movie") {
      return encoder.getDisplayName({
        cleaned: {
          title: "Inception",
          year: "2010",
          season_episode: "",
          resolution: "1080P",
          source: "BluRay",
          provider: "AMZN",
          audio_tags: "VOSTFR",
          suggested: "",
        },
        output_name: "VOSTFR AAC",
        path: "",
        filename: "",
        size_mb: 0,
        duration_secs: 0,
        fps: 0,
        audio_langs: [],
        sub_langs: [],
        streams: [],
        status: "ready",
        output_ext: ".mkv",
        sub_extract_status: "none",
      });
    }
    return encoder.getDisplayName({
      cleaned: {
        title: "Jujutsu Kaisen",
        year: "",
        season_episode: "S03E01",
        resolution: "1080P",
        source: "BluRay",
        provider: "NF",
        audio_tags: "VOSTFR",
        suggested: "",
      },
      output_name: "VOSTFR AAC",
      path: "",
      filename: "",
      size_mb: 0,
      duration_secs: 0,
      fps: 0,
      audio_langs: [],
      sub_langs: [],
      streams: [],
      status: "ready",
      output_ext: ".mkv",
      sub_extract_status: "none",
    });
  }

  let previewSeries = $derived(previewName("series"));
  let previewMovie = $derived(previewName("movie"));
</script>

<div
  class="page-root"
  class:page-root--horizontal={encoder.innerNavLayout === "horizontal"}
>
  <!-- ── Sidebar ──────────────────────────────────────────────────────────── -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <span class="sidebar-title">Renommage</span>
      <span class="sidebar-sub">Tags &amp; format</span>
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
            <span class="nav-item-desc">
              {sec.id === "team" ? team || "non définie" : sec.desc}
            </span>
          </div>
          {#if activeSection === sec.id}
            <div class="nav-item-indicator" aria-hidden="true"></div>
          {/if}
        </button>
      {/each}
    </nav>

    <!-- Preview live en sidebar -->
    <div class="sidebar-preview">
      <div class="sp-label">APERÇU</div>
      <div class="sp-name">{previewSeries}</div>
      <div class="sp-meta">
        {activeTagCount} tag{activeTagCount > 1 ? "s" : ""} actif{activeTagCount >
        1
          ? "s"
          : ""}
      </div>
    </div>
  </aside>

  <!-- ── Content panel ────────────────────────────────────────────────────── -->
  <div class="content-panel">
    {#if activeSection === "tags"}
      <TagsTab {tagOrder} {disabledTags} />
    {:else if activeSection === "format"}
      <FormatTab
        {previewSeries}
        {previewMovie}
        {resCase}
        {titleCase}
        {codecFmt}
        {sourceCase}
        {yearParentheses}
        {webSourceFmt}
        {tagSep}
        {provCase}
        {seFormat}
        {japVer}
      />
    {:else if activeSection === "team"}
      <TeamTab {team} {previewSeries} />
    {:else if activeSection === "preview"}
      <PreviewTab
        {previewSeries}
        {previewMovie}
        {tagOrder}
        {disabledTags}
        {team}
        {resCase}
        {titleCase}
        {seFormat}
        {japVer}
        {codecFmt}
        {sourceCase}
        {webSourceFmt}
        {tagSep}
        {provCase}
        {yearParentheses}
      />
    {/if}
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
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  /* ── Nav items ───────────────────────────────────────────────────────────── */
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
  }
  .nav-item--active .nav-item-icon {
    color: var(--color-accent);
    border-color: color-mix(
      in srgb,
      var(--color-accent) 30%,
      var(--color-border)
    );
    background: color-mix(
      in srgb,
      var(--color-accent) 8%,
      var(--color-surface)
    );
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
    right: 8px;
    top: 50%;
    transform: translateY(-50%);
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--color-accent);
  }

  /* ── Sidebar preview ─────────────────────────────────────────────────────── */
  .sidebar-preview {
    padding: 12px 14px;
    border-top: 1px solid var(--color-border);
    flex-shrink: 0;
  }
  .sp-label {
    font-family: "Geist Mono", monospace;
    font-size: 8px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--color-subtext2);
    margin-bottom: 5px;
  }
  .sp-name {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-text);
    line-height: 1.5;
    word-break: break-all;
    margin-bottom: 4px;
  }
  .sp-meta {
    font-family: "Geist Mono", monospace;
    font-size: 8px;
    color: var(--color-subtext2);
  }

  /* ── Content panel ───────────────────────────────────────────────────────── */
  .content-panel {
    flex: 1 1 0;
    min-width: 0;
    overflow-y: auto;
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
  .page-root--horizontal .sidebar-header,
  .page-root--horizontal .sidebar-preview {
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
    padding: 6px 12px;
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
  }
  .page-root--horizontal .content-panel > :global(*) {
    max-width: 760px;
    margin-left: auto;
    margin-right: auto;
    width: 100%;
  }
</style>
