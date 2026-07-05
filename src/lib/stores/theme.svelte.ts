// Svelte 5 runes-based theme store

const DEFAULT_ACCENT_DARK  = "#d33636";
const DEFAULT_BG_DARK      = "#1d1d1d";
const DEFAULT_ACCENT_LIGHT = "#d33636";
const DEFAULT_BG_LIGHT     = "#f5f4f2";

const ACCENT_STORAGE_KEY = "accentColor";
const BG_STORAGE_KEY     = "bgColor";
const THEME_STORAGE_KEY  = "theme";

export type ThemeCombo = {
  id:     string;
  label:  string;
  bg:     string;
  accent: string;
  dark:   boolean;
};

export const DARK_COMBOS: ThemeCombo[] = [
  { id: "default",  label: "Default",  bg: "#1d1d1d", accent: "#d33636", dark: true },
  { id: "midnight", label: "Midnight", bg: "#0d1117", accent: "#58a6ff", dark: true },
  { id: "forest",   label: "Forest",   bg: "#0f1a12", accent: "#4dbb6a", dark: true },
  { id: "crimson",  label: "Crimson",  bg: "#1a0d0d", accent: "#e05252", dark: true },
  { id: "violet",   label: "Violet",   bg: "#100d1a", accent: "#b25fd1", dark: true },
  { id: "slate",    label: "Slate",    bg: "#131820", accent: "#4d8fbb", dark: true },
];

export const LIGHT_COMBOS: ThemeCombo[] = [
  { id: "paper",   label: "Paper",   bg: "#f5f4f2", accent: "#d33636", dark: false },
  { id: "stone",   label: "Stone",   bg: "#f0ede8", accent: "#27834a", dark: false },
  { id: "sky",     label: "Sky",     bg: "#eef4fb", accent: "#2f7ab8", dark: false },
  { id: "rose",    label: "Rose",    bg: "#fdf0f0", accent: "#c0392b", dark: false },
  { id: "sage",    label: "Sage",    bg: "#eff4ef", accent: "#4a7c59", dark: false },
  { id: "sand",    label: "Sand",    bg: "#faf7f0", accent: "#b07d20", dark: false },
];

/** Assombrit une couleur hex de `amount` (0-1). */
function darkenHex(hex: string, amount = 0.12): string {
  const m = hex.replace("#", "");
  const full = m.length === 3 ? m.split("").map((c) => c + c).join("") : m;
  const num = parseInt(full, 16);
  if (Number.isNaN(num)) return hex;
  const r = Math.max(0, Math.round(((num >> 16) & 0xff) * (1 - amount)));
  const g = Math.max(0, Math.round(((num >> 8) & 0xff) * (1 - amount)));
  const b = Math.max(0, Math.round((num & 0xff) * (1 - amount)));
  return `#${[r, g, b].map((v) => v.toString(16).padStart(2, "0")).join("")}`;
}

/** Éclaircit ou assombrit selon la direction donnée. */
function shiftHex(hex: string, amount: number): string {
  const m = hex.replace("#", "");
  const full = m.length === 3 ? m.split("").map((c) => c + c).join("") : m;
  const num = parseInt(full, 16);
  if (Number.isNaN(num)) return hex;
  const shift = Math.round(255 * Math.abs(amount));
  const op = amount > 0 ? (v: number) => Math.min(255, v + shift) : (v: number) => Math.max(0, v - shift);
  const r = op((num >> 16) & 0xff);
  const g = op((num >> 8) & 0xff);
  const b = op(num & 0xff);
  return `#${[r, g, b].map((v) => v.toString(16).padStart(2, "0")).join("")}`;
}

function isValidHex(hex: string): boolean {
  return /^#([0-9a-f]{3}|[0-9a-f]{6})$/i.test(hex);
}

/** Estime si une couleur hex est claire (luminosité > 0.5). */
function isLight(hex: string): boolean {
  const m = hex.replace("#", "");
  const full = m.length === 3 ? m.split("").map((c) => c + c).join("") : m;
  const num = parseInt(full, 16);
  const r = (num >> 16) & 0xff;
  const g = (num >> 8) & 0xff;
  const b = num & 0xff;
  return (0.299 * r + 0.587 * g + 0.114 * b) / 255 > 0.55;
}

function createTheme() {
  let dark            = $state(true);
  let accent          = $state(DEFAULT_ACCENT_DARK);
  let backgroundColor = $state(DEFAULT_BG_DARK);

  function applyAccent(color: string) {
    document.documentElement.style.setProperty("--color-accent", color);
    document.documentElement.style.setProperty("--color-accent2", darkenHex(color));
  }

  function applyBackground(color: string) {
    const light = isLight(color);
    const sign  = light ? -1 : 1; // dark bg → lighten; light bg → darken

    document.documentElement.style.setProperty("--color-surface", color);
    document.documentElement.style.setProperty("--color-panel",   shiftHex(color, sign * 0.03));
    document.documentElement.style.setProperty("--color-panel2",  shiftHex(color, sign * 0.015));
    document.documentElement.style.setProperty("--color-border",  shiftHex(color, sign * 0.08));
    document.documentElement.style.setProperty("--color-border2", shiftHex(color, sign * 0.11));
    document.documentElement.style.setProperty("--color-muted",   shiftHex(color, sign * 0.14));

    // Text colors
    if (light) {
      document.documentElement.style.setProperty("--color-text",     "#1a1a1a");
      document.documentElement.style.setProperty("--color-subtext",  "#7a7670");
      document.documentElement.style.setProperty("--color-subtext2", "#aaa69e");
    } else {
      document.documentElement.style.setProperty("--color-text",     "#e8e8e8");
      document.documentElement.style.setProperty("--color-subtext",  "#6b6b6b");
      document.documentElement.style.setProperty("--color-subtext2", "#4a4a4a");
    }
  }

  function toggle() {
    dark = !dark;
    document.documentElement.classList.toggle("dark", dark);
    document.documentElement.classList.toggle("light", !dark);
    if (typeof localStorage !== "undefined") {
      localStorage.setItem(THEME_STORAGE_KEY, dark ? "dark" : "light");
    }
    // Reset bg & accent to defaults for the new mode
    const defaultBg     = dark ? DEFAULT_BG_DARK     : DEFAULT_BG_LIGHT;
    const defaultAccent = dark ? DEFAULT_ACCENT_DARK  : DEFAULT_ACCENT_LIGHT;
    setBackground(defaultBg);
    setAccent(defaultAccent);
  }

  function setAccent(color: string) {
    if (!isValidHex(color)) return;
    accent = color;
    applyAccent(color);
    if (typeof localStorage !== "undefined") {
      localStorage.setItem(ACCENT_STORAGE_KEY, color);
    }
  }

  function resetAccent() {
    setAccent(dark ? DEFAULT_ACCENT_DARK : DEFAULT_ACCENT_LIGHT);
  }

  function setBackground(color: string) {
    if (!isValidHex(color)) return;
    backgroundColor = color;
    applyBackground(color);
    if (typeof localStorage !== "undefined") {
      localStorage.setItem(BG_STORAGE_KEY, color);
    }
  }

  function resetBackground() {
    setBackground(dark ? DEFAULT_BG_DARK : DEFAULT_BG_LIGHT);
  }

  function applyCombo(combo: ThemeCombo) {
    if (combo.dark !== dark) {
      dark = combo.dark;
      document.documentElement.classList.toggle("dark", dark);
      document.documentElement.classList.toggle("light", !dark);
      if (typeof localStorage !== "undefined") {
        localStorage.setItem(THEME_STORAGE_KEY, dark ? "dark" : "light");
      }
    }
    setAccent(combo.accent);
    setBackground(combo.bg);
  }

  function init() {
    const saved = typeof localStorage !== "undefined"
      ? localStorage.getItem(THEME_STORAGE_KEY)
      : null;
    dark = saved !== "light";
    document.documentElement.classList.toggle("dark", dark);
    document.documentElement.classList.toggle("light", !dark);

    const savedAccent = typeof localStorage !== "undefined"
      ? localStorage.getItem(ACCENT_STORAGE_KEY)
      : null;
    accent = savedAccent && isValidHex(savedAccent)
      ? savedAccent
      : (dark ? DEFAULT_ACCENT_DARK : DEFAULT_ACCENT_LIGHT);
    applyAccent(accent);

    const savedBg = typeof localStorage !== "undefined"
      ? localStorage.getItem(BG_STORAGE_KEY)
      : null;
    backgroundColor = savedBg && isValidHex(savedBg)
      ? savedBg
      : (dark ? DEFAULT_BG_DARK : DEFAULT_BG_LIGHT);
    applyBackground(backgroundColor);
  }

  return {
    get dark()            { return dark; },
    get accent()          { return accent; },
    get backgroundColor() { return backgroundColor; },
    defaultAccentDark:  DEFAULT_ACCENT_DARK,
    defaultAccentLight: DEFAULT_ACCENT_LIGHT,
    defaultBgDark:      DEFAULT_BG_DARK,
    defaultBgLight:     DEFAULT_BG_LIGHT,
    toggle,
    setAccent,
    resetAccent,
    setBackground,
    resetBackground,
    applyCombo,
    init,
  };
}

export const theme = createTheme();