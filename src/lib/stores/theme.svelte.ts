// Svelte 5 runes-based theme store

const DEFAULT_ACCENT = "#e07b39";
const ACCENT_STORAGE_KEY = "accentColor";
const THEME_STORAGE_KEY = "theme";

/** Assombrit une couleur hex de `amount` (0-1). Utilisé pour dériver --color-accent2. */
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

function isValidHex(hex: string): boolean {
  return /^#([0-9a-f]{3}|[0-9a-f]{6})$/i.test(hex);
}

function createTheme() {
  let dark = $state(true);
  let accent = $state(DEFAULT_ACCENT);

  function applyAccent(color: string) {
    document.documentElement.style.setProperty("--color-accent", color);
    document.documentElement.style.setProperty("--color-accent2", darkenHex(color));
  }

  function toggle() {
    dark = !dark;
    document.documentElement.classList.toggle("dark", dark);
    document.documentElement.classList.toggle("light", !dark);
    // Persist
    if (typeof localStorage !== "undefined") {
      localStorage.setItem(THEME_STORAGE_KEY, dark ? "dark" : "light");
    }
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
    setAccent(DEFAULT_ACCENT);
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
    accent = savedAccent && isValidHex(savedAccent) ? savedAccent : DEFAULT_ACCENT;
    applyAccent(accent);
  }

  return {
    get dark() { return dark; },
    get accent() { return accent; },
    defaultAccent: DEFAULT_ACCENT,
    toggle,
    setAccent,
    resetAccent,
    init,
  };
}

export const theme = createTheme();