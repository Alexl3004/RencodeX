<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { prefs } from "$lib/stores/prefs.store.svelte";
  import { LANG_NAMES } from "$lib/stores/naming";
  import { Volume2, Subtitles } from "@lucide/svelte";

  let langOrder = $state<string[]>([...prefs.langOrder]);
  $effect(() => {
    langOrder = [...prefs.langOrder];
  });

  function langLabel(code: string) {
    return LANG_NAMES[code] ?? code.toUpperCase();
  }

  let defaultAudioLangs = $state<string[]>([...prefs.defaultAudioLangs]);
  let defaultSubLangs = $state<string[]>([...prefs.defaultSubLangs]);

  $effect(() => {
    defaultAudioLangs = [...prefs.defaultAudioLangs];
  });
  $effect(() => {
    defaultSubLangs = [...prefs.defaultSubLangs];
  });

  function toggleDefaultAudio(code: string) {
    const next = defaultAudioLangs.includes(code)
      ? defaultAudioLangs.filter((c) => c !== code)
      : [...defaultAudioLangs, code];
    defaultAudioLangs = next;
    prefs.setDefaultAudioLangs(next);
  }
  function toggleDefaultSub(code: string) {
    const next = defaultSubLangs.includes(code)
      ? defaultSubLangs.filter((c) => c !== code)
      : [...defaultSubLangs, code];
    defaultSubLangs = next;
    prefs.setDefaultSubLangs(next);
  }
</script>

<section class="content-section">
  <header class="section-header">
    <div>
      <h2 class="section-title">Langues par défaut</h2>
      <p class="section-desc">
        Pistes activées automatiquement à l'ouverture d'un fichier.
      </p>
    </div>
  </header>

  <div class="lang-grid">
    <!-- ── Colonne Audio ── -->
    <div class="lang-col">
      <div class="lang-col-header">
        <Volume2 class="w-3.5 h-3.5" />
        <span>Audio</span>
      </div>
      <ul class="lang-toggle-list" aria-label="Langues audio par défaut">
        {#each langOrder as code (code)}
          {@const on = defaultAudioLangs.includes(code)}
          <li>
            <button
              type="button"
              class="lang-toggle {on ? 'lang-toggle--on' : ''}"
              onclick={() => toggleDefaultAudio(code)}
              aria-pressed={on}
            >
              <span class="lang-toggle-name">{langLabel(code)}</span>
              <span class="lang-toggle-code">{code}</span>
              <span class="lang-toggle-pill" aria-hidden="true">
                <span class="lang-toggle-thumb"></span>
              </span>
            </button>
          </li>
        {/each}
      </ul>
    </div>

    <!-- ── Colonne Sous-titres ── -->
    <div class="lang-col">
      <div class="lang-col-header">
        <Subtitles class="w-3.5 h-3.5" />
        <span>Sous-titres</span>
      </div>
      <ul class="lang-toggle-list" aria-label="Langues sous-titres par défaut">
        {#each langOrder as code (code)}
          {@const on = defaultSubLangs.includes(code)}
          <li>
            <button
              type="button"
              class="lang-toggle {on ? 'lang-toggle--on' : ''}"
              onclick={() => toggleDefaultSub(code)}
              aria-pressed={on}
            >
              <span class="lang-toggle-name">{langLabel(code)}</span>
              <span class="lang-toggle-code">{code}</span>
              <span class="lang-toggle-pill" aria-hidden="true">
                <span class="lang-toggle-thumb"></span>
              </span>
            </button>
          </li>
        {/each}
      </ul>
    </div>
  </div>
</section>

<style>
  .content-section {
    padding: 28px 32px;
    max-width: 680px;
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
    max-width: 420px;
    margin: 0;
  }

  .lang-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    align-items: start;
  }
  .lang-col {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .lang-col-header {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    font-weight: 600;
    color: var(--color-subtext2);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: 0 2px 4px;
    border-bottom: 1px solid var(--color-border);
    margin-bottom: 2px;
  }
  .lang-toggle-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .lang-toggle {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    text-align: left;
    transition:
      border-color 0.12s,
      background 0.12s;
    user-select: none;
  }
  .lang-toggle:hover {
    border-color: var(--color-subtext2);
    background: color-mix(
      in srgb,
      var(--color-accent) 3%,
      var(--color-surface)
    );
  }
  .lang-toggle--on {
    border-color: color-mix(
      in srgb,
      var(--color-accent) 45%,
      var(--color-border)
    );
    background: color-mix(
      in srgb,
      var(--color-accent) 6%,
      var(--color-surface)
    );
  }
  .lang-toggle--on:hover {
    border-color: var(--color-accent);
    background: color-mix(
      in srgb,
      var(--color-accent) 10%,
      var(--color-surface)
    );
  }
  .lang-toggle-name {
    flex: 1;
    font-size: 12px;
    font-weight: 500;
    color: var(--color-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .lang-toggle-code {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext2);
    flex-shrink: 0;
  }

  .lang-toggle-pill {
    position: relative;
    width: 28px;
    height: 16px;
    border-radius: 999px;
    background: var(--color-border);
    flex-shrink: 0;
    transition: background 0.18s;
  }
  .lang-toggle--on .lang-toggle-pill {
    background: var(--color-accent);
  }
  .lang-toggle-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: white;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.25);
    transition: transform 0.18s;
  }
  .lang-toggle--on .lang-toggle-thumb {
    transform: translateX(12px);
  }
</style>
