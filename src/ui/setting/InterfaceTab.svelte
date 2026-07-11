<script lang="ts">
  import { Sun, Moon, RotateCcw, Palette } from "@lucide/svelte";
  import { theme, DARK_COMBOS, LIGHT_COMBOS } from "$lib/stores/theme.svelte";
  import { encoder } from "$lib/stores/encoder.svelte";

  type SavedConfig = {
    nav_layout: "vertical" | "horizontal";
    inner_nav_layout: "vertical" | "horizontal";
  };

  let {
    form,
    confirmReset,
    onReset,
    onSave,
  }: {
    form: SavedConfig;
    confirmReset: boolean;
    onReset: () => void;
    onSave: () => void;
  } = $props();
</script>

<section class="content-section">
  <header class="section-header">
    <div>
      <h2 class="section-title">Interface</h2>
      <p class="section-desc">Personnalisez l'apparence de l'application.</p>
    </div>
  </header>

  <!-- Navigation globale -->
  <div>
    <span class="field-label">Navigation globale</span>
    <p class="field-hint">
      Disposition de la barre de navigation principale de l'application.
    </p>
    <div class="flex gap-2">
      <button
        type="button"
        onclick={() => {
          form.nav_layout = "vertical";
          encoder.navLayout = "vertical";
          onSave();
        }}
        class="layout-btn {form.nav_layout === 'vertical'
          ? 'layout-btn--active'
          : ''}"
        aria-pressed={form.nav_layout === "vertical"}
      >
        <svg
          width="36"
          height="28"
          viewBox="0 0 36 28"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <rect
            x="0.5"
            y="0.5"
            width="35"
            height="27"
            rx="3.5"
            stroke="currentColor"
            stroke-opacity="0.3"
          />
          <rect
            x="2"
            y="2"
            width="8"
            height="24"
            rx="2"
            fill="currentColor"
            fill-opacity="0.15"
          />
          <rect
            x="3"
            y="4"
            width="6"
            height="2"
            rx="1"
            fill="currentColor"
            fill-opacity="0.5"
          />
          <rect
            x="3"
            y="8"
            width="6"
            height="2"
            rx="1"
            fill="currentColor"
            fill-opacity="0.5"
          />
          <rect
            x="3"
            y="12"
            width="6"
            height="2"
            rx="1"
            fill="currentColor"
            fill-opacity="0.5"
          />
        </svg>
        <span>Verticale</span>
      </button>
      <button
        type="button"
        onclick={() => {
          form.nav_layout = "horizontal";
          encoder.navLayout = "horizontal";
          onSave();
        }}
        class="layout-btn {form.nav_layout === 'horizontal'
          ? 'layout-btn--active'
          : ''}"
        aria-pressed={form.nav_layout === "horizontal"}
      >
        <svg
          width="36"
          height="28"
          viewBox="0 0 36 28"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <rect
            x="0.5"
            y="0.5"
            width="35"
            height="27"
            rx="3.5"
            stroke="currentColor"
            stroke-opacity="0.3"
          />
          <rect
            x="2"
            y="2"
            width="32"
            height="7"
            rx="2"
            fill="currentColor"
            fill-opacity="0.15"
          />
          <rect
            x="4"
            y="3.5"
            width="5"
            height="2"
            rx="1"
            fill="currentColor"
            fill-opacity="0.5"
          />
          <rect
            x="11"
            y="3.5"
            width="5"
            height="2"
            rx="1"
            fill="currentColor"
            fill-opacity="0.5"
          />
          <rect
            x="18"
            y="3.5"
            width="5"
            height="2"
            rx="1"
            fill="currentColor"
            fill-opacity="0.5"
          />
        </svg>
        <span>Horizontale</span>
      </button>
    </div>
  </div>

  <!-- Navigation interne -->
  <div>
    <span class="field-label">Navigation des pages</span>
    <p class="field-hint">
      Disposition des menus de navigation dans chaque page (Encodage, Renommage,
      etc.).
    </p>
    <div class="flex gap-2">
      <button
        type="button"
        onclick={() => {
          form.inner_nav_layout = "vertical";
          encoder.innerNavLayout = "vertical";
          onSave();
        }}
        class="layout-btn {form.inner_nav_layout === 'vertical'
          ? 'layout-btn--active'
          : ''}"
        aria-pressed={form.inner_nav_layout === "vertical"}
      >
        <svg
          width="36"
          height="28"
          viewBox="0 0 36 28"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <rect
            x="0.5"
            y="0.5"
            width="35"
            height="27"
            rx="3.5"
            stroke="currentColor"
            stroke-opacity="0.3"
          />
          <rect
            x="2"
            y="2"
            width="8"
            height="24"
            rx="2"
            fill="currentColor"
            fill-opacity="0.15"
          />
          <rect
            x="3"
            y="4"
            width="6"
            height="2"
            rx="1"
            fill="currentColor"
            fill-opacity="0.5"
          />
          <rect
            x="3"
            y="8"
            width="6"
            height="2"
            rx="1"
            fill="currentColor"
            fill-opacity="0.5"
          />
          <rect
            x="3"
            y="12"
            width="6"
            height="2"
            rx="1"
            fill="currentColor"
            fill-opacity="0.5"
          />
        </svg>
        <span>Verticale</span>
      </button>
      <button
        type="button"
        onclick={() => {
          form.inner_nav_layout = "horizontal";
          encoder.innerNavLayout = "horizontal";
          onSave();
        }}
        class="layout-btn {form.inner_nav_layout === 'horizontal'
          ? 'layout-btn--active'
          : ''}"
        aria-pressed={form.inner_nav_layout === "horizontal"}
      >
        <svg
          width="36"
          height="28"
          viewBox="0 0 36 28"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <rect
            x="0.5"
            y="0.5"
            width="35"
            height="27"
            rx="3.5"
            stroke="currentColor"
            stroke-opacity="0.3"
          />
          <rect
            x="2"
            y="2"
            width="32"
            height="7"
            rx="2"
            fill="currentColor"
            fill-opacity="0.15"
          />
          <rect
            x="4"
            y="3.5"
            width="5"
            height="2"
            rx="1"
            fill="currentColor"
            fill-opacity="0.5"
          />
          <rect
            x="11"
            y="3.5"
            width="5"
            height="2"
            rx="1"
            fill="currentColor"
            fill-opacity="0.5"
          />
          <rect
            x="18"
            y="3.5"
            width="5"
            height="2"
            rx="1"
            fill="currentColor"
            fill-opacity="0.5"
          />
        </svg>
        <span>Horizontale</span>
      </button>
    </div>
  </div>

  <!-- Mode sombre / clair -->
  <div>
    <span class="field-label">Apparence</span>
    <div class="mode-toggle-row">
      <button
        type="button"
        onclick={() => {
          if (!theme.dark) theme.toggle();
        }}
        class="mode-btn {theme.dark ? 'mode-btn--active' : ''}"
        aria-pressed={theme.dark}
      >
        <Moon class="w-3.5 h-3.5" />
        Sombre
      </button>
      <button
        type="button"
        onclick={() => {
          if (theme.dark) theme.toggle();
        }}
        class="mode-btn {!theme.dark ? 'mode-btn--active' : ''}"
        aria-pressed={!theme.dark}
      >
        <Sun class="w-3.5 h-3.5" />
        Clair
      </button>
    </div>
  </div>

  <!-- Thèmes prédéfinis -->
  <div>
    <span class="field-label">Thèmes {theme.dark ? "sombres" : "clairs"}</span>
    <div class="combo-grid">
      {#each theme.dark ? DARK_COMBOS : LIGHT_COMBOS as combo}
        {@const isActive =
          theme.backgroundColor.toLowerCase() === combo.bg &&
          theme.accent.toLowerCase() === combo.accent}
        <button
          type="button"
          onclick={() => theme.applyCombo(combo)}
          class="combo-btn {isActive ? 'combo-btn--active' : ''}"
          title={combo.label}
          aria-pressed={isActive}
        >
          <span class="combo-preview">
            <span class="combo-bg" style="background: {combo.bg};"></span>
            <span class="combo-accent" style="background: {combo.accent};"
            ></span>
          </span>
          <span class="combo-label">{combo.label}</span>
        </button>
      {/each}
    </div>
  </div>

  <!-- Couleur primaire -->
  <div>
    <span class="field-label flex items-center gap-1.5">
      <Palette class="w-3 h-3" />
      Couleur primaire
    </span>
    <div class="flex items-center gap-2 flex-wrap">
      <label
        class="color-swatch-input"
        style="background: {theme.accent};"
        title="Choisir une couleur personnalisée"
      >
        <input
          type="color"
          value={theme.accent}
          oninput={(e) =>
            theme.setAccent((e.currentTarget as HTMLInputElement).value)}
          aria-label="Couleur primaire personnalisée"
        />
      </label>
      <input
        type="text"
        value={theme.accent}
        onchange={(e) =>
          theme.setAccent((e.currentTarget as HTMLInputElement).value.trim())}
        placeholder="#e07b39"
        maxlength="7"
        class="field-input px-2 py-1.5 w-24 text-center"
        aria-label="Code hexadécimal de la couleur primaire"
      />
      {#if theme.accent.toLowerCase() !== (theme.dark ? theme.defaultAccentDark : theme.defaultAccentLight)}
        <button
          type="button"
          onclick={() => theme.resetAccent()}
          class="icon-btn-inline"
          title="Réinitialiser"
        >
          <RotateCcw class="w-3.5 h-3.5" />
        </button>
      {/if}
    </div>
  </div>

  <!-- Couleur de fond -->
  <div>
    <span class="field-label flex items-center gap-1.5">
      <Palette class="w-3 h-3" />
      Couleur de fond
    </span>
    <div class="flex items-center gap-2 flex-wrap">
      <label
        class="color-swatch-input"
        style="background: {theme.backgroundColor};"
        title="Choisir une couleur de fond"
      >
        <input
          type="color"
          value={theme.backgroundColor}
          oninput={(e) =>
            theme.setBackground((e.currentTarget as HTMLInputElement).value)}
          aria-label="Couleur de fond personnalisée"
        />
      </label>
      <input
        type="text"
        value={theme.backgroundColor}
        onchange={(e) =>
          theme.setBackground(
            (e.currentTarget as HTMLInputElement).value.trim(),
          )}
        placeholder="#1d1d1d"
        maxlength="7"
        class="field-input px-2 py-1.5 w-24 text-center"
        aria-label="Code hexadécimal du fond"
      />
      {#if theme.backgroundColor.toLowerCase() !== (theme.dark ? theme.defaultBgDark : theme.defaultBgLight)}
        <button
          type="button"
          onclick={() => theme.resetBackground()}
          class="icon-btn-inline"
          title="Réinitialiser"
        >
          <RotateCcw class="w-3.5 h-3.5" />
        </button>
      {/if}
    </div>
  </div>

  <button
    onclick={onReset}
    disabled={encoder.encoding || encoder.extractingSubs}
    class="w-full btn font-mono text-[11px] justify-center gap-1.5"
    class:btn-danger={confirmReset}
    aria-label={confirmReset
      ? "Confirmer la réinitialisation"
      : "Réinitialiser aux valeurs par défaut"}
  >
    <RotateCcw class="w-3.5 h-3.5 shrink-0" />
    {confirmReset
      ? "CONFIRMER LA RÉINITIALISATION ?"
      : "RÉINITIALISER LES PARAMÈTRES"}
  </button>
</section>

<style>
  .field-label {
    display: block;
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
    margin-bottom: 4px;
  }
  .field-hint {
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    color: var(--color-subtext);
    margin: 2px 0 6px;
    opacity: 0.7;
  }
  .field-input {
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    color: var(--color-text);
    border-radius: var(--radius-sm);
    outline: none;
    transition: border-color 0.12s;
  }
  .field-input:focus {
    border-color: var(--color-accent);
  }

  .content-section {
    padding: 28px 32px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }
  .section-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 4px;
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
    max-width: 380px;
    margin: 0;
  }

  .icon-btn-inline {
    color: var(--color-subtext);
    background: transparent;
    border: none;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: color 0.1s;
  }
  .icon-btn-inline:hover {
    color: var(--color-text);
  }

  .color-swatch-input {
    position: relative;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    cursor: pointer;
    flex-shrink: 0;
    overflow: hidden;
    transition:
      border-color 0.12s,
      transform 0.12s;
  }
  .color-swatch-input:hover {
    border-color: var(--color-border2);
    transform: scale(1.04);
  }
  .color-swatch-input input[type="color"] {
    position: absolute;
    inset: -4px;
    width: calc(100% + 8px);
    height: calc(100% + 8px);
    border: none;
    padding: 0;
    cursor: pointer;
    opacity: 0;
  }

  .layout-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 5px;
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-subtext);
    cursor: pointer;
    font-family: "Geist Mono", monospace;
    font-size: 10px;
    transition:
      border-color 0.15s,
      color 0.15s,
      background 0.15s;
  }
  .layout-btn:hover {
    border-color: var(--color-accent);
    color: var(--color-text);
  }
  .layout-btn--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 10%, transparent);
    color: var(--color-accent);
  }

  .mode-toggle-row {
    display: flex;
    gap: 6px;
    margin-top: 6px;
  }
  .mode-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 14px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-subtext);
    font-family: "Geist Mono", monospace;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition:
      background 0.15s,
      color 0.15s,
      border-color 0.15s;
  }
  .mode-btn:hover {
    border-color: var(--color-subtext2);
    color: var(--color-text);
  }
  .mode-btn--active {
    background: color-mix(in srgb, var(--color-accent) 12%, transparent);
    border-color: color-mix(in srgb, var(--color-accent) 35%, transparent);
    color: var(--color-accent);
  }

  .combo-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 6px;
  }
  .combo-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 6px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    cursor: pointer;
    transition:
      border-color 0.15s,
      transform 0.1s;
  }
  .combo-btn:hover {
    border-color: var(--color-subtext2);
    transform: translateY(-1px);
  }
  .combo-btn--active {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 8%, transparent);
  }
  .combo-preview {
    width: 40px;
    height: 26px;
    border-radius: var(--radius-xs);
    overflow: hidden;
    display: flex;
    border: 1px solid color-mix(in srgb, var(--color-border) 60%, transparent);
  }
  .combo-bg {
    flex: 3;
  }
  .combo-accent {
    flex: 1;
  }
  .combo-label {
    font-family: "Geist Mono", monospace;
    font-size: 9px;
    color: var(--color-subtext);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    white-space: nowrap;
  }
  .combo-btn--active .combo-label {
    color: var(--color-accent);
  }
</style>
