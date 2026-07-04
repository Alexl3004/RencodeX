<script lang="ts">
  import {
    encoder,
    TAG_LABELS,
    SEASON_EPISODE_FORMATS,
    formatSeasonEpisode,
    type TagId,
    type ResolutionCase,
    type TitleCaseMode,
    type CodecFormat,
    type SourceCase,
  } from "$lib/stores/encoder.svelte";
  import { ArrowUp, ArrowDown, GripVertical, RotateCcw, Tag, AlignLeft, Users, Eye } from "@lucide/svelte";

  let { onClose }: { onClose?: () => void } = $props();

  let tagOrder     = $derived(encoder.tagOrder);
  let disabledTags = $derived(encoder.disabledTags);
  let resCase      = $derived(encoder.resolutionCase);
  let titleCase    = $derived(encoder.titleCase);
  let codecFmt     = $derived(encoder.codecFormat);
  let sourceCase   = $derived(encoder.sourceCase);
  let seFormat     = $derived(encoder.seasonEpisodeFormat);
  let team         = $derived(encoder.team);

  // ── Navigation sections ──────────────────────────────────────────────────
  type SectionId = "tags" | "format" | "team" | "preview";

  const SECTIONS: { id: SectionId; label: string; icon: any; desc: string }[] = [
    { id: "tags",    label: "Ordre des tags",  icon: Tag,      desc: "Glisser · activer" },
    { id: "format",  label: "Format",          icon: AlignLeft, desc: "Casse · séparateurs" },
    { id: "team",    label: "Team",            icon: Users,    desc: "non définie" },
    { id: "preview", label: "Aperçu",          icon: Eye,      desc: "Nom généré" },
  ];

  let activeSection = $state<SectionId>("tags");

  // ── Drag & drop ──────────────────────────────────────────────────────────
  let localOrder    = $state<TagId[]>([]);
  let dragIndex     = $state<number | null>(null);
  let dragOverIndex = $state<number | null>(null);
  let listEl        = $state<HTMLElement | null>(null);

  $effect(() => { localOrder = [...tagOrder]; });

  function getRowIndex(y: number): number | null {
    if (!listEl) return null;
    const rows = Array.from(listEl.children) as HTMLElement[];
    for (let i = 0; i < rows.length; i++) {
      const r = rows[i].getBoundingClientRect();
      if (y >= r.top && y <= r.bottom) return i;
    }
    if (!rows.length) return null;
    if (y < rows[0].getBoundingClientRect().top) return 0;
    if (y > rows[rows.length - 1].getBoundingClientRect().bottom) return rows.length - 1;
    return null;
  }

  function onHandlePointerDown(e: PointerEvent, i: number) {
    if (e.button !== 0 && e.pointerType === "mouse") return;
    e.preventDefault();
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    dragIndex = i;
    dragOverIndex = i;
  }
  function onHandlePointerMove(e: PointerEvent) {
    if (dragIndex === null) return;
    e.preventDefault();
    const over = getRowIndex(e.clientY);
    if (over !== null) dragOverIndex = over;
  }
  function onHandlePointerUp() {
    if (dragIndex === null) return;
    const toIndex = dragOverIndex ?? dragIndex;
    if (toIndex !== dragIndex) {
      const id = localOrder[dragIndex];
      const newOrder = [...localOrder];
      newOrder.splice(dragIndex, 1);
      newOrder.splice(toIndex, 0, id);
      localOrder = newOrder;
      const steps = toIndex - dragIndex;
      const dir   = steps > 0 ? 1 : -1;
      for (let s = 0; s < Math.abs(steps); s++) encoder.moveTag(id, dir);
    }
    dragIndex = null;
    dragOverIndex = null;
  }

  // ── Aperçu ───────────────────────────────────────────────────────────────
  function previewName() {
    return encoder.getDisplayName({
      cleaned: {
        title: "Jujutsu Kaisen",
        season_episode: "S03E01",
        resolution: "1080P",
        source: "BluRay",
        provider: "",
        audio_tags: "VOSTFR",
        suggested: "",
      },
      output_name: "VOSTFR AAC",
      path: "", filename: "", size_mb: 0, duration_secs: 0,
      fps: 0, audio_langs: [], sub_langs: [], streams: [],
      status: "ready", output_ext: ".mkv", sub_extract_status: "none",
    });
  }

  const TITLE_CASE_OPTIONS: { value: TitleCaseMode; label: string; preview: string; desc: string }[] = [
    { value: "original", label: "Original",   preview: "Jujutsu Kaisen", desc: "Conserve la casse source" },
    { value: "upper",    label: "MAJUSCULES",  preview: "JUJUTSU KAISEN", desc: "Tout en majuscules" },
    { value: "lower",    label: "minuscules",  preview: "jujutsu kaisen", desc: "Tout en minuscules" },
    { value: "title",    label: "Titre",       preview: "Jujutsu Kaisen", desc: "1ère lettre de chaque mot" },
  ];

  // Nombre de tags actifs pour l'aperçu sidebar
  let activeTagCount = $derived(tagOrder.length - disabledTags.size);
</script>

<div class="page-root" class:page-root--horizontal={encoder.innerNavLayout === "horizontal"}>

  <!-- ── Sidebar ─────────────────────────────────────────────────────────── -->
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
              {sec.id === "team" ? (team || "non définie") : sec.desc}
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
      <div class="sp-name">{previewName()}</div>
      <div class="sp-meta">{activeTagCount} tag{activeTagCount > 1 ? "s" : ""} actif{activeTagCount > 1 ? "s" : ""}</div>
    </div>
  </aside>

  <!-- ── Content panel ───────────────────────────────────────────────────── -->
  <div class="content-panel">

    <!-- ═══════════════ TAGS ═══════════════ -->
    {#if activeSection === "tags"}
      <section class="content-section">
        <header class="section-header">
          <div>
            <h2 class="section-title">Ordre des tags</h2>
            <p class="section-desc">
              Glissez les tags pour réorganiser leur ordre dans le nom de fichier.
              Désactivez ceux que vous ne souhaitez pas inclure.
            </p>
          </div>
          <button
            type="button"
            class="reset-btn"
            onclick={() => encoder.resetTagOrder()}
            title="Remettre l'ordre par défaut"
          >
            <RotateCcw class="w-3 h-3" />
            Réinitialiser
          </button>
        </header>

        <div
          class="tag-list"
          role="list"
          bind:this={listEl}
          onpointermove={onHandlePointerMove}
          onpointerup={onHandlePointerUp}
          onpointercancel={onHandlePointerUp}
        >
          {#each localOrder as id, i (id)}
            {@const disabled = disabledTags.has(id)}
            <div
              class="tag-row
                {dragIndex === i ? 'tag-row--dragging' : ''}
                {dragOverIndex === i && dragIndex !== null && dragIndex !== i ? 'tag-row--over' : ''}
                {disabled ? 'tag-row--disabled' : ''}"
              role="listitem"
            >
              <!-- Position badge -->
              <div class="tag-pos">{disabled ? "—" : i + 1 - [...localOrder.slice(0, i)].filter(t => disabledTags.has(t)).length}</div>

              <!-- Toggle -->
              <button
                type="button"
                class="toggle-switch {disabled ? '' : 'toggle-switch--on'}"
                onclick={() => encoder.toggleTag(id)}
                aria-pressed={!disabled}
                aria-label={disabled ? `Activer ${TAG_LABELS[id]}` : `Désactiver ${TAG_LABELS[id]}`}
              >
                <span class="toggle-thumb"></span>
              </button>

              <!-- Label -->
              <span class="tag-label {disabled ? 'tag-label--off' : ''}">
                {TAG_LABELS[id]}
                {#if id === "team" && !team}
                  <span class="tag-empty">vide</span>
                {/if}
              </span>

              <!-- Actions -->
              <div class="tag-actions">
                <button
                  type="button"
                  class="icon-btn"
                  disabled={i === 0}
                  onclick={() => encoder.moveTag(id, -1)}
                  aria-label="Monter"
                >
                  <ArrowUp class="w-3 h-3" />
                </button>
                <button
                  type="button"
                  class="icon-btn"
                  disabled={i === localOrder.length - 1}
                  onclick={() => encoder.moveTag(id, 1)}
                  aria-label="Descendre"
                >
                  <ArrowDown class="w-3 h-3" />
                </button>
                <button
                  type="button"
                  class="drag-handle"
                  aria-label="Déplacer"
                  onpointerdown={(e) => onHandlePointerDown(e, i)}
                >
                  <GripVertical class="w-3.5 h-3.5" />
                </button>
              </div>
            </div>
          {/each}
        </div>
      </section>

    <!-- ═══════════════ FORMAT ═══════════════ -->
    {:else if activeSection === "format"}
      <section class="content-section">
        <header class="section-header">
          <div>
            <h2 class="section-title">Format</h2>
            <p class="section-desc">
              Personnalisez la casse et le format de chaque composant du nom de fichier.
            </p>
          </div>
          <button
            type="button"
            class="reset-btn"
            onclick={() => encoder.resetFormatOptions()}
            title="Remettre les options par défaut"
          >
            <RotateCcw class="w-3 h-3" />
            Réinitialiser
          </button>
        </header>

        <!-- Résolution -->
        <div class="format-block">
          <div class="format-block-label">Résolution</div>
          <div class="format-block-desc">Casse du tag de résolution dans le nom de fichier.</div>
          <div class="option-pair">
            {#each [["upper", "1080P", "Majuscules"], ["lower", "1080p", "Minuscules"]] as [val, preview, lbl]}
              <button
                type="button"
                class="option-card {resCase === val ? 'option-card--active' : ''}"
                onclick={() => encoder.setResolutionCase(val as ResolutionCase)}
              >
                <span class="oc-preview">{preview}</span>
                <span class="oc-label">{lbl}</span>
              </button>
            {/each}
          </div>
        </div>

        <!-- Titre -->
        <div class="format-block">
          <div class="format-block-label">Titre</div>
          <div class="format-block-desc">Transformation appliquée au titre de l'œuvre.</div>
          <div class="option-col">
            {#each TITLE_CASE_OPTIONS as opt}
              <button
                type="button"
                class="option-row {titleCase === opt.value ? 'option-row--active' : ''}"
                onclick={() => encoder.setTitleCase(opt.value)}
              >
                <div class="or-left">
                  <span class="or-label">{opt.label}</span>
                  <span class="or-desc">{opt.desc}</span>
                </div>
                <span class="or-preview">{opt.preview}</span>
              </button>
            {/each}
          </div>
        </div>

        <!-- Saison / Épisode -->
        <div class="format-block">
          <div class="format-block-label">Saison / Épisode</div>
          <div class="format-block-desc">Format du tag de numérotation des épisodes.</div>
          <div class="option-col">
            {#each SEASON_EPISODE_FORMATS as f}
              <button
                type="button"
                class="option-row {seFormat === f.value ? 'option-row--active' : ''}"
                onclick={() => encoder.setSeasonEpisodeFormat(f.value)}
              >
                <div class="or-left">
                  <span class="or-label">{f.label}</span>
                </div>
                <span class="or-preview">{formatSeasonEpisode("S03E01", f.value)}</span>
              </button>
            {/each}
          </div>
        </div>

        <!-- Codec -->
        <div class="format-block">
          <div class="format-block-label">Codec vidéo</div>
          <div class="format-block-desc">Libellé du codec H.265 dans le nom de fichier.</div>
          <div class="option-pair option-pair--3">
            {#each (["H265", "H.265", "HEVC"] as const) as val}
              <button
                type="button"
                class="option-card {codecFmt === val ? 'option-card--active' : ''}"
                onclick={() => encoder.setCodecFormat(val as CodecFormat)}
              >
                <span class="oc-preview">{val}</span>
              </button>
            {/each}
          </div>
        </div>

        <!-- Source -->
        <div class="format-block format-block--last">
          <div class="format-block-label">Source</div>
          <div class="format-block-desc">Casse du tag de source (BluRay, WEB-DL…).</div>
          <div class="option-pair option-pair--3">
            {#each [["original","BluRay","Original"],["upper","BLURAY","Majuscules"],["lower","bluray","Minuscules"]] as [val, preview, lbl]}
              <button
                type="button"
                class="option-card {sourceCase === val ? 'option-card--active' : ''}"
                onclick={() => encoder.setSourceCase(val as SourceCase)}
              >
                <span class="oc-preview">{preview}</span>
                <span class="oc-label">{lbl}</span>
              </button>
            {/each}
          </div>
        </div>

      </section>

    <!-- ═══════════════ TEAM ═══════════════ -->
    {:else if activeSection === "team"}
      <section class="content-section">
        <header class="section-header">
          <div>
            <h2 class="section-title">Team</h2>
            <p class="section-desc">
              Le nom de la team est inséré dans le nom de fichier à la position définie
              dans l'onglet « Ordre des tags ». Laissez vide pour l'omettre.
            </p>
          </div>
        </header>

        <div class="field-block">
          <div class="field-label">Nom de la team</div>
          <input
            type="text"
            value={team}
            oninput={(e: Event) => encoder.setTeam((e.target as HTMLInputElement).value)}
            placeholder="ex : SubsPlease, Erai-raws…"
            class="text-input"
          />
        </div>

        {#if team}
          <div class="team-preview-box">
            <div class="tpb-label">TAG INSÉRÉ</div>
            <div class="tpb-value">{team}</div>
            <div class="tpb-example">
              <span class="tpb-ex-label">Exemple ·</span>
              <span class="tpb-ex-name">{previewName()}</span>
            </div>
          </div>
        {:else}
          <div class="info-callout">
            Aucune team définie — le tag sera ignoré même s'il est activé dans l'ordre.
          </div>
        {/if}
      </section>

    <!-- ═══════════════ APERÇU ═══════════════ -->
    {:else if activeSection === "preview"}
      <section class="content-section">
        <header class="section-header">
          <div>
            <h2 class="section-title">Aperçu du nom</h2>
            <p class="section-desc">
              Nom de fichier généré avec les paramètres actuels sur un exemple fictif.
            </p>
          </div>
        </header>

        <div class="preview-name-box">
          <div class="pnb-label">NOM GÉNÉRÉ</div>
          <div class="pnb-name">{previewName()}</div>
        </div>

        <div class="preview-tags-grid">
          {#each localOrder as id}
            {@const disabled = disabledTags.has(id)}
            <div class="ptg-item {disabled ? 'ptg-item--off' : ''}">
              <span class="ptg-dot {disabled ? 'ptg-dot--off' : ''}"></span>
              <span class="ptg-name">{TAG_LABELS[id]}</span>
              {#if id === "team" && team}
                <span class="ptg-val">{team}</span>
              {/if}
            </div>
          {/each}
        </div>

        <div class="preview-params">
          <div class="pp-row">
            <span class="pp-key">Résolution</span>
            <span class="pp-val">{resCase === "upper" ? "1080P" : "1080p"}</span>
          </div>
          <div class="pp-row">
            <span class="pp-key">Casse titre</span>
            <span class="pp-val">{TITLE_CASE_OPTIONS.find(o => o.value === titleCase)?.label ?? titleCase}</span>
          </div>
          <div class="pp-row">
            <span class="pp-key">S/E format</span>
            <span class="pp-val">{formatSeasonEpisode("S03E01", seFormat)}</span>
          </div>
          <div class="pp-row">
            <span class="pp-key">Codec</span>
            <span class="pp-val">{codecFmt}</span>
          </div>
          <div class="pp-row">
            <span class="pp-key">Source</span>
            <span class="pp-val">{{ original: "BluRay", upper: "BLURAY", lower: "bluray" }[sourceCase]}</span>
          </div>
          {#if team}
            <div class="pp-row">
              <span class="pp-key">Team</span>
              <span class="pp-val">{team}</span>
            </div>
          {/if}
        </div>
      </section>
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

  /* Sidebar preview */
  .sidebar-preview {
    padding: 14px 16px;
    border-top: 1px solid var(--color-border);
    flex-shrink: 0;
  }
  .sp-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    letter-spacing: 0.07em;
    color: var(--color-subtext2);
    text-transform: uppercase;
    margin-bottom: 6px;
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
    font-size: 9px;
    color: var(--color-subtext2);
  }

  /* ── Content ────────────────────────────────────────────────────────────── */
  .content-panel {
    flex: 1;
    overflow-y: auto;
  }

  .content-section {
    padding: 28px 32px;
    max-width: 640px;
  }

  .section-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 28px;
    padding-bottom: 20px;
    border-bottom: 1px solid var(--color-border);
  }
  .section-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text);
    letter-spacing: -0.02em;
    margin: 0 0 6px;
  }
  .section-desc {
    font-size: 12px;
    color: var(--color-subtext);
    line-height: 1.6;
    max-width: 400px;
    margin: 0;
  }

  /* ── Tags list ──────────────────────────────────────────────────────────── */
  .tag-list {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .tag-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    transition: opacity 0.12s, border-color 0.12s, background 0.12s;
    user-select: none;
  }
  .tag-row--dragging { opacity: 0.3; }
  .tag-row--over {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 7%, var(--color-panel));
  }
  .tag-row--disabled { opacity: 0.5; }

  .tag-pos {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    font-weight: 700;
    color: var(--color-subtext2);
    width: 16px;
    text-align: center;
    flex-shrink: 0;
  }

  /* Toggle switch */
  .toggle-switch {
    width: 28px;
    height: 15px;
    border-radius: 8px;
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    padding: 0 2px;
    flex-shrink: 0;
    transition: background 0.15s, border-color 0.15s;
  }
  .toggle-switch--on {
    background: var(--color-accent);
    border-color: var(--color-accent);
  }
  .toggle-thumb {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: var(--color-subtext2);
    transition: transform 0.15s, background 0.15s;
  }
  .toggle-switch--on .toggle-thumb {
    transform: translateX(13px);
    background: #fff;
  }

  .tag-label {
    flex: 1;
    font-size: 12px;
    font-weight: 500;
    color: var(--color-text);
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .tag-label--off {
    color: var(--color-subtext2);
    text-decoration: line-through;
    text-decoration-color: color-mix(in srgb, var(--color-subtext2) 50%, transparent);
  }
  .tag-empty {
    font-family: "Geist Mono", monospace;
    font-size: 8px;
    padding: 1px 5px;
    border-radius: var(--radius-xs);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    color: var(--color-subtext2);
  }

  .tag-actions {
    display: flex;
    align-items: center;
    gap: 2px;
    flex-shrink: 0;
  }

  .icon-btn {
    width: 24px;
    height: 24px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-xs);
    border: 1px solid transparent;
    background: transparent;
    color: var(--color-subtext2);
    cursor: pointer;
    transition: background 0.1s, color 0.1s, border-color 0.1s;
  }
  .icon-btn:hover {
    background: var(--color-surface);
    border-color: var(--color-border);
    color: var(--color-text);
  }
  .icon-btn:disabled { opacity: 0.3; cursor: not-allowed; }

  .drag-handle {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: var(--radius-xs);
    border: 1px solid transparent;
    background: transparent;
    color: var(--color-subtext2);
    cursor: grab;
    touch-action: none;
    transition: color 0.1s, background 0.1s;
  }
  .drag-handle:hover { color: var(--color-subtext); background: var(--color-surface); border-color: var(--color-border); }
  .drag-handle:active { cursor: grabbing; }

  /* ── Reset button ───────────────────────────────────────────────────────── */
  .reset-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    flex-shrink: 0;
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    padding: 6px 11px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: transparent;
    color: var(--color-subtext);
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.1s, color 0.1s, border-color 0.1s;
  }
  .reset-btn:hover {
    background: color-mix(in srgb, var(--color-danger) 8%, transparent);
    border-color: color-mix(in srgb, var(--color-danger) 40%, var(--color-border));
    color: var(--color-danger);
  }

  /* ── Format blocks ──────────────────────────────────────────────────────── */
  .format-block {
    margin-bottom: 28px;
    padding-bottom: 28px;
    border-bottom: 1px solid var(--color-border);
  }
  .format-block--last {
    border-bottom: none;
    margin-bottom: 0;
    padding-bottom: 0;
  }
  .format-block-label {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color-subtext);
    margin-bottom: 4px;
  }
  .format-block-desc {
    font-size: 11px;
    color: var(--color-subtext2);
    margin-bottom: 12px;
    line-height: 1.5;
  }

  /* Option pair (2 colonnes) */
  .option-pair {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }
  .option-pair--3 {
    grid-template-columns: repeat(3, 1fr);
  }

  .option-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 12px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-panel);
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
  }
  .option-card:hover { border-color: var(--color-subtext2); }
  .option-card--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 8%, var(--color-panel));
  }
  .oc-preview {
    font-family: "Geist Mono", monospace;
    font-size: 13px;
    font-weight: 700;
    color: var(--color-subtext);
    transition: color 0.15s;
  }
  .option-card--active .oc-preview { color: var(--color-accent); }
  .oc-label {
    font-size: 9px;
    color: var(--color-subtext2);
  }

  /* Option col (liste de rows) */
  .option-col {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .option-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 11px 14px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-panel);
    cursor: pointer;
    text-align: left;
    transition: border-color 0.15s, background 0.15s;
    gap: 12px;
  }
  .option-row:hover { border-color: var(--color-subtext2); }
  .option-row--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 7%, var(--color-panel));
  }
  .or-left { display: flex; flex-direction: column; gap: 2px; }
  .or-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--color-subtext);
    transition: color 0.15s;
  }
  .option-row--active .or-label { color: var(--color-accent); }
  .or-desc {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext2);
  }
  .or-preview {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    color: var(--color-subtext2);
    white-space: nowrap;
    flex-shrink: 0;
  }
  .option-row--active .or-preview { color: var(--color-accent); opacity: 0.85; }

  /* ── Team ───────────────────────────────────────────────────────────────── */
  .field-block { margin-bottom: 20px; }
  .field-label {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color-subtext);
    margin-bottom: 10px;
  }
  .text-input {
    width: 100%;
    font-family: "Geist Mono", monospace;
    font-size: 12px;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    color: var(--color-text);
    outline: none;
    transition: border-color 0.15s;
    box-sizing: border-box;
  }
  .text-input:focus { border-color: var(--color-accent); }
  .text-input::placeholder { color: var(--color-subtext2); }

  .team-preview-box {
    padding: 16px 18px;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--color-accent) 6%, var(--color-panel));
    border: 1px solid color-mix(in srgb, var(--color-accent) 22%, var(--color-border));
  }
  .tpb-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    letter-spacing: 0.06em;
    color: var(--color-subtext2);
    text-transform: uppercase;
    margin-bottom: 5px;
  }
  .tpb-value {
    font-family: "Geist Mono", monospace;
    font-size: 18px;
    font-weight: 700;
    color: var(--color-accent);
    margin-bottom: 10px;
  }
  .tpb-example {
    display: flex;
    align-items: baseline;
    gap: 6px;
    flex-wrap: wrap;
  }
  .tpb-ex-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext2);
    white-space: nowrap;
  }
  .tpb-ex-name {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
    word-break: break-all;
  }

  .info-callout {
    padding: 12px 14px;
    border-radius: var(--radius-sm);
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    font-size: 12px;
    color: var(--color-subtext);
    line-height: 1.5;
  }

  /* ── Preview section ────────────────────────────────────────────────────── */
  .preview-name-box {
    padding: 18px 20px;
    border-radius: var(--radius-sm);
    background: var(--color-panel);
    border: 1px solid var(--color-border);
    margin-bottom: 20px;
  }
  .pnb-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    letter-spacing: 0.07em;
    color: var(--color-subtext2);
    text-transform: uppercase;
    margin-bottom: 8px;
  }
  .pnb-name {
    font-family: "Geist Mono", monospace;
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text);
    line-height: 1.5;
    word-break: break-all;
  }

  .preview-tags-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 20px;
  }
  .ptg-item {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 9px;
    border-radius: var(--radius-full);
    border: 1px solid var(--color-border);
    background: var(--color-panel);
    font-size: 11px;
    color: var(--color-subtext);
    transition: opacity 0.15s;
  }
  .ptg-item--off {
    opacity: 0.35;
    text-decoration: line-through;
  }
  .ptg-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--color-accent);
    flex-shrink: 0;
  }
  .ptg-dot--off { background: var(--color-subtext2); }
  .ptg-name { font-size: 11px; }
  .ptg-val {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 10%, transparent);
    padding: 1px 5px;
    border-radius: 3px;
  }

  .preview-params {
    display: flex;
    flex-direction: column;
    gap: 0;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }
  .pp-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 9px 14px;
    border-bottom: 1px solid var(--color-border);
  }
  .pp-row:last-child { border-bottom: none; }
  .pp-key {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext2);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .pp-val {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    font-weight: 600;
    color: var(--color-text);
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
  }
  .page-root--horizontal .content-panel > *{
    max-width: 760px;
    margin-left: auto;
    margin-right: auto;
    width: 100%;
  }
</style>