<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { theme } from "$lib/stores/theme.svelte";

  let { children } = $props();

  onMount(() => {
    theme.init();
  });

  $effect(() => {
    document.documentElement.classList.toggle("dark", theme.dark);
    document.documentElement.classList.toggle("light", !theme.dark);
  });
</script>

{@render children()}