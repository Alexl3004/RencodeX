<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { toasts } from "$lib/stores/toasts.svelte";

  async function pick() {
    const dir = await open({ directory: true, defaultPath: encoder.outputDir });
    if (dir && typeof dir === "string") {
      encoder.outputDir = dir;
      encoder.log(`Dossier de sortie : ${dir}`, "info");
      toasts.info(`Dossier de sortie : ${dir}`);
    }
  }

  let shortPath = $derived(
    encoder.outputDir.length > 48
      ? "…\\" + encoder.outputDir.split(/[\\/]/).slice(-2).join("\\")
      : encoder.outputDir,
  );
</script>

<button
  type="button"
  onclick={pick}
  disabled={encoder.encoding}
  class="btn btn-secondary max-w-full justify-start"
  title={encoder.outputDir}
>
  <span class="truncate text-[var(--color-subtext)]">
    Destination : <span class="text-[var(--color-text)] font-mono">{shortPath || "Non défini"}</span>
  </span>
  <span class="text-[var(--color-subtext)] ml-1">…</span>
</button>