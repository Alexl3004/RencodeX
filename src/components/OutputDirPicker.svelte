<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { open } from "@tauri-apps/plugin-dialog";

  async function pick() {
    const dir = await open({ directory: true, defaultPath: encoder.outputDir });
    if (dir && typeof dir === "string") {
      encoder.outputDir = dir;
      encoder.log(`Dossier de sortie : ${dir}`, "info");
    }
  }

  let shortPath = $derived(
    encoder.outputDir.length > 48
      ? "…\\" + encoder.outputDir.split(/[\\/]/).slice(-2).join("\\")
      : encoder.outputDir,
  );
</script>

<button
  onclick={pick}
  disabled={encoder.encoding}
  class="btn max-w-full"
  title={encoder.outputDir}
>
  <span class="truncate text-subtext">
    Destination : <span class="text-text font-mono"
      >{shortPath || "Non défini"}</span
    >
  </span>
  <span class="text-subtext ml-1">…</span>
</button>
