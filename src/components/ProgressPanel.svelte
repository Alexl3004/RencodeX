<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { formatTime, formatSize, gainPct } from "$lib/utils";
  import { Progress } from "@skeletonlabs/skeleton-svelte";
  import { CircleCheck, TvMinimalPlay, AlertTriangle, Zap, Package, FolderOpen } from '@lucide/svelte';

  let p = $derived(encoder.progress);
  let s = $derived(encoder.summary);

  let totalPercent = $derived(
    p ? ((p.file_index + p.percent / 100) / p.file_total) * 100 : 0,
  );

  let doneFiles = $derived(
    encoder.files.filter(f => f.status === "done" && f.result && f.result.original_mb > 0)
  );

  let avgRatio = $derived(
    doneFiles.length === 0 ? null
      : doneFiles.reduce((sum, f) => sum + f.result!.encoded_mb / f.result!.original_mb, 0) / doneFiles.length
  );

  let totalOriginalMb = $derived(encoder.files.reduce((sum, f) => sum + f.size_mb, 0));
  let realEncodedMb = $derived(doneFiles.reduce((sum, f) => sum + f.result!.encoded_mb, 0));
  let remainingOriginalMb = $derived(
    encoder.files.filter(f => f.status !== "done" && f.status !== "error").reduce((sum, f) => sum + f.size_mb, 0)
  );

  let estimatedTotalMb = $derived(avgRatio === null ? null : realEncodedMb + remainingOriginalMb * avgRatio);
  let estimatedGainPct = $derived(
    estimatedTotalMb === null || totalOriginalMb === 0 ? null
      : ((totalOriginalMb - estimatedTotalMb) / totalOriginalMb) * 100
  );

  // Filtrer les erreurs pour le récapitulatif
  let failedFiles = $derived(s ? s.files.filter(f => f.status !== "ok") : []);

  const radius = 54;
  const circumference = 2 * Math.PI * radius;
  function getOffset(percent: number) {
    return circumference - (percent / 100) * circumference;
  }
</script>

<div class="border border-[var(--color-border)] rounded-[var(--radius-md)] bg-[var(--color-panel)] h-full flex flex-col overflow-hidden">
  {#if encoder.encoding}
    <div class="flex items-center gap-2 px-3 py-1 border-b border-[var(--color-border)] bg-[var(--color-accent)]/5 shrink-0 text-[11px]">
      <div class="w-1.5 h-1.5 rounded-full bg-[var(--color-accent)] animate-pulse"></div>
      <span class="font-mono text-[var(--color-text)] uppercase tracking-wider">Encodage</span>
      {#if p && p.file_total > 1}
        <span class="text-[var(--color-subtext)]">{p.file_index + 1}/{p.file_total}</span>
      {/if}
      <span class="flex-1"></span>
      <span class="font-mono text-[var(--color-subtext)] inline-flex items-center gap-1"><Zap class="w-3 h-3" />{p ? p.speed?.toFixed(1) : "—"}x</span>
    </div>

    <div class="flex-1 flex items-stretch px-4 py-0 gap-4 overflow-hidden">
      {#if p}
        <div class="flex items-center justify-center shrink-0">
          <div class="relative w-36 h-36">
            <svg class="w-full h-full -rotate-90" viewBox="0 0 128 128">
              <circle cx="64" cy="64" r={radius} fill="none" stroke="var(--color-surface)" stroke-width="10" />
              <circle
                cx="64" cy="64"
                r={radius}
                fill="none"
                stroke="var(--color-accent)"
                stroke-width="10"
                stroke-linecap="round"
                stroke-dasharray={circumference}
                stroke-dashoffset={getOffset(p.percent)}
                class="transition-all duration-300 ease-out"
              />
            </svg>
            <div class="absolute inset-0 flex items-center justify-center">
              <span class="text-2xl font-bold text-[var(--color-text)]">{Math.floor(p.percent)}%</span>
            </div>
          </div>
        </div>

        <div class="flex-1 min-w-0 flex flex-col justify-center gap-0.5 py-0.5">
          <div class="text-[14px] font-semibold text-[var(--color-text)] truncate" title={p.file_name}>
            {p.file_name}
          </div>

          <div class="grid grid-cols-3 gap-2 text-[12px] mt-0.5">
            <div>
              <div class="text-[9px] text-[var(--color-subtext)] uppercase tracking-wide">RESTANT</div>
              <div class="font-mono font-semibold text-[var(--color-text)]">{formatTime(p.remaining_file)}</div>
            </div>
            <div>
              <div class="text-[9px] text-[var(--color-subtext)] uppercase tracking-wide">VITESSE</div>
              <div class="font-mono font-semibold text-[var(--color-text)]">{p.speed?.toFixed(1)}x</div>
            </div>
            <div>
              <div class="text-[9px] text-[var(--color-subtext)] uppercase tracking-wide">TOTAL</div>
              <div class="font-mono font-semibold text-[var(--color-text)]">
                {p.remaining_total > 0 ? formatTime(p.remaining_total) : '—'}
              </div>
            </div>
          </div>

          {#if p.file_total > 1}
            <div class="space-y-0.5 mt-1">
              <div class="flex justify-between text-[10px] text-[var(--color-subtext)]">
                <span>Progression globale</span>
                <span>{totalPercent.toFixed(0)}%</span>
              </div>
              <Progress value={totalPercent} max={100}>
                <Progress.Track
                  class="h-2 rounded-full border border-[var(--color-border)] bg-[var(--color-surface)] opacity-40"
                >
                  <Progress.Range class="rounded-full bg-[var(--color-accent)]" />
                </Progress.Track>
              </Progress>
            </div>
          {/if}

          {#if estimatedTotalMb !== null}
            <div class="flex gap-4 text-[10px] text-[var(--color-subtext)] mt-0.5">
              <span class="inline-flex items-center gap-1"><Package class="w-3 h-3" />Estimé {formatSize(estimatedTotalMb)}</span>
              <span class="inline-flex items-center gap-1"><FolderOpen class="w-3 h-3" />Original {formatSize(totalOriginalMb)}</span>
            </div>
          {/if}
        </div>
      {/if}
    </div>

  {:else if s}
    <div class="flex items-center gap-2 px-3 py-1 border-b border-[var(--color-border)] bg-[var(--color-success)]/5 shrink-0 text-[11px]">
      <CircleCheck class="w-3.5 h-3.5 text-[var(--color-success)]" />
      <span class="font-mono text-[var(--color-success)] uppercase tracking-wider">Terminé</span>
    </div>

    <div class="flex-1 flex items-stretch px-4 py-2.5 gap-4 overflow-hidden">
      
      <div class="w-36 flex flex-col justify-between shrink-0">
        <div class="h-full flex flex-col justify-center gap-1">
          <div class="py-1 bg-[var(--color-surface)] rounded-[var(--radius-sm)] flex justify-between items-center px-2">
            <div class="text-[9px] text-[var(--color-subtext)] uppercase tracking-wide">Succès</div>
            <div class="text-base font-bold text-[var(--color-text)] font-mono">{s.files.filter(f => f.status === "ok").length}/{s.files.length}</div>
          </div>
          <div class="py-1 bg-[var(--color-surface)] rounded-[var(--radius-sm)] flex justify-between items-center px-2">
            <div class="text-[9px] text-[var(--color-subtext)] uppercase tracking-wide">Gain</div>
            <div class="text-base font-bold text-[var(--color-success)] font-mono">{gainPct(s.total_original_mb, s.total_encoded_mb)}</div>
          </div>
          <div class="py-1 bg-[var(--color-surface)] rounded-[var(--radius-sm)] flex justify-between items-center px-2">
            <div class="text-[9px] text-[var(--color-subtext)] uppercase tracking-wide">Durée</div>
            <div class="text-base font-bold text-[var(--color-text)] font-mono">{formatTime(s.total_secs)}</div>
          </div>
        </div>
      </div>

      <div class="flex-1 flex flex-col justify-between overflow-hidden min-h-0">
        {#if failedFiles.length > 0}
          <div class="flex flex-col h-full overflow-hidden">
            <div class="text-[9px] text-[var(--color-danger)] font-medium mb-1 uppercase tracking-wide flex items-center gap-1 shrink-0">
              <AlertTriangle class="w-3 h-3" /> Fichiers en échec ({failedFiles.length})
            </div>
            <div class="w-full space-y-0.5 overflow-y-auto flex-1 text-[10px] pr-1">
              {#each failedFiles as r}
                <div class="flex justify-between items-center px-2 py-0.5 rounded bg-[var(--color-danger)]/5 border border-[var(--color-danger)]/10">
                  <span class="truncate text-[var(--color-text)] flex-1 mr-2" title={r.name}>{r.name.slice(-25)}</span>
                  <span class="shrink-0 font-mono font-semibold text-[var(--color-danger)] text-[9px] uppercase">
                    {r.status === 'error' ? 'Échec' : 'Annulé'}
                  </span>
                </div>
              {/each}
            </div>
          </div>
        {:else}
          <div class="h-full flex items-center pl-2 border-l-2 border-[var(--color-success)]/40">
            <div>
              <p class="text-[13px] font-semibold text-[var(--color-text)]">Traitement réussi</p>
              <p class="text-[11px] text-[var(--color-subtext)] mt-0.5">Tous vos fichiers ont été encodés avec succès sans erreur.</p>
            </div>
          </div>
        {/if}
      </div>

    </div>

  {:else}
    <div class="flex-1 flex items-center justify-center p-4">
      <div class="text-center">
        <TvMinimalPlay class="w-10 h-10 mx-auto mb-2 text-[var(--color-subtext)]/20" />
        <p class="text-[13px] font-medium text-[var(--color-subtext)]">Prêt à encoder</p>
        <p class="text-[11px] text-[var(--color-subtext2)]">Ajoutez des fichiers</p>
      </div>
    </div>
  {/if}
</div>