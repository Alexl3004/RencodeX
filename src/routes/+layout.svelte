<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { theme } from "$lib/stores/theme.svelte";

  let { children } = $props();

  const ACCENT_STORAGE_KEY = "rencodex-accent-color";

  onMount(() => {
    // Restaurer la couleur d'accentuation sauvegardée
    const saved = localStorage.getItem(ACCENT_STORAGE_KEY);
    if (saved && /^#[0-9a-fA-F]{6}$/.test(saved)) {
      document.documentElement.style.setProperty("--color-accent", saved);
    }
  });

  $effect(() => {
    document.documentElement.classList.toggle("dark", theme.dark);
  });
</script>

{@render children()}