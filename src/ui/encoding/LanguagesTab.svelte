<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { prefs } from "$lib/stores/prefs.store.svelte";
  import { LANG_NAMES } from "$lib/stores/naming";
  import { Volume2, Subtitles } from "@lucide/svelte";

  let langOrder = $state<string[]>([...prefs.langOrder]);
  $effect(() => { langOrder = [...prefs.langOrder]; });

  let defaultAudioLangs = $state<string[]>([...prefs.defaultAudioLangs]);
  let defaultSubLangs   = $state<string[]>([...prefs.defaultSubLangs]);

  $effect(() => { defaultAudioLangs = [...prefs.defaultAudioLangs]; });
  $effect(() => { defaultSubLangs   = [...prefs.defaultSubLangs]; });

  function langLabel(code: string) {
    return LANG_NAMES[code] ?? code.toUpperCase();
  }

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

<section class="tab">
  <header class="tab-header">
    <h2 class="tab-title">Pistes par défaut</h2>
    <p class="tab-desc">
      Pistes activées automatiquement à l'ouverture d'un fichier.
    </p>
  </header>

  <div class="lang-grid">
    <!-- Audio -->
    <div class="lang-col">
      <div class="lang-col-header">
        <Volume2 class="w-3.5 h-3.5" />
        <span>Audio</span>
      </div>
      <ul class="lang-list" aria-label="Langues audio par défaut">
        {#each langOrder as code (code)}
          {@const on = defaultAudioLangs.includes(code)}
          <li>
            <button
              type="button"
              class="lang-row {on ? 'lang-row--on' : ''}"
              onclick={() => toggleDefaultAudio(code)}
              aria-pressed={on}
            >
              <span class="lang-name">{langLabel(code)}</span>
              <span class="lang-code">{code}</span>
              <span class="pill" aria-hidden="true">
                <span class="pill-thumb"></span>
              </span>
            </button>
          </li>
        {/each}
      </ul>
    </div>

    <!-- Sous-titres -->
    <div class="lang-col">
      <div class="lang-col-header">
        <Subtitles class="w-3.5 h-3.5" />
        <span>Sous-titres</span>
      </div>
      <ul class="lang-list" aria-label="Langues sous-titres par défaut">
        {#each langOrder as code (code)}
          {@const on = defaultSubLangs.includes(code)}
          <li>
            <button
              type="button"
              class="lang-row {on ? 'lang-row--on' : ''}"
              onclick={() => toggleDefaultSub(code)}
              aria-pressed={on}
            >
              <span class="lang-name">{langLabel(code)}</span>
              <span class="lang-code">{code}</span>
              <span class="pill" aria-hidden="true">
                <span class="pill-thumb"></span>
              </span>
            </button>
          </li>
        {/each}
      </ul>
    </div>
  </div>
</section>

<style>
  .tab {
    padding: 24px 28px;
    max-width: 560px;
  }

  .tab-header {
    margin-bottom: 20px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--color-border);
  }
  .tab-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--color-text);
    letter-spacing: -0.02em;
    margin: 0 0 4px;
  }
  .tab-desc {
    font-size: 12px;
    color: var(--color-subtext);
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
    font-size: 10px;
    font-weight: 600;
    color: var(--color-subtext2);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    padding-bottom: 6px;
    border-bottom: 1px solid var(--color-border);
    margin-bottom: 2px;
  }

  .lang-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .lang-row {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    transition: border-color 0.12s, background 0.12s;
    user-select: none;
  }
  .lang-row:hover {
    border-color: var(--color-subtext2);
    background: color-mix(in srgb, var(--color-accent) 3%, var(--color-surface));
  }
  .lang-row--on {
    border-color: color-mix(in srgb, var(--color-accent) 45%, var(--color-border));
    background: color-mix(in srgb, var(--color-accent) 6%, var(--color-surface));
  }
  .lang-row--on:hover {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 10%, var(--color-surface));
  }

  .lang-name {
    flex: 1;
    font-size: 12px;
    font-weight: 500;
    color: var(--color-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    text-align: left;
  }
  .lang-code {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext2);
    flex-shrink: 0;
  }

  .pill {
    position: relative;
    width: 28px;
    height: 16px;
    border-radius: 999px;
    background: var(--color-border);
    flex-shrink: 0;
    transition: background 0.18s;
  }
  .lang-row--on .pill { background: var(--color-accent); }
  .pill-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: white;
    box-shadow: 0 1px 3px rgba(0,0,0,0.25);
    transition: transform 0.18s;
  }
  .lang-row--on .pill-thumb { transform: translateX(12px); }
</style>
