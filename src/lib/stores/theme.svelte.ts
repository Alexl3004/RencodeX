// Svelte 5 runes-based theme store
function createTheme() {
  let dark = $state(true);

  function toggle() {
    dark = !dark;
    document.documentElement.classList.toggle("dark", dark);
    document.documentElement.classList.toggle("light", !dark);
    // Persist
    if (typeof localStorage !== "undefined") {
      localStorage.setItem("theme", dark ? "dark" : "light");
    }
  }

  function init() {
    const saved = typeof localStorage !== "undefined"
      ? localStorage.getItem("theme")
      : null;
    dark = saved !== "light";
    document.documentElement.classList.toggle("dark", dark);
    document.documentElement.classList.toggle("light", !dark);
  }

  return {
    get dark() { return dark; },
    toggle,
    init,
  };
}

export const theme = createTheme();
