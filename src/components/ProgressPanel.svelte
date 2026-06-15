<script lang="ts">
  import { encoder } from "$lib/stores/encoder.svelte";
  import { formatTime, formatSize, gainPct } from "$lib/utils";

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
</script>

<div class="border border-[var(--color-border)] rounded-[2px] bg-[var(--color-panel)] h-[25vh] flex flex-col overflow-hidden">
  {#if encoder.encoding}
    <!-- Header -->
    <div class="flex items-center gap-2 px-4 py-2 border-b border-[var(--color-border)] bg-[var(--color-accent)]/5 shrink-0">
      <div class="w-2 h-2 rounded-full bg-[var(--color-accent)] animate-pulse"></div>
      <span class="text-[11px] font-mono text-[var(--color-text)] uppercase tracking-wider">Encodage</span>
      {#if p && p.file_total > 1}
        <span class="text-[10px] text-[var(--color-subtext)]">{p.file_index + 1}/{p.file_total}</span>
      {/if}
      <span class="flex-1"></span>
      <span class="text-[10px] font-mono text-[var(--color-subtext)]">⚡ {p ? p.speed?.toFixed(1) : "—"}x</span>
    </div>

    <!-- Content -->
    <div class="flex-1 p-4 space-y-4 overflow-auto">
      {#if p}
        <!-- Fichier actuel -->
        <div class="space-y-2">
          <div class="flex justify-between items-center">
            <div class="flex items-center gap-2 min-w-0">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="text-[var(--color-subtext)] shrink-0">
                <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/>
                <polyline points="13 2 13 9 20 9"/>
              </svg>
              <span class="text-[11px] font-mono text-[var(--color-text)] truncate" title={p.file_name}>
                {p.file_name}
              </span>
            </div>
            <span class="text-[13px] font-mono font-bold text-[var(--color-accent)]">{Math.floor(p.percent)}%</span>
          </div>
          
          <!-- Barre progression -->
          <div class="relative h-3 bg-[var(--color-surface)] border border-[var(--color-border)] rounded-full overflow-hidden">
            <div 
              class="absolute inset-y-0 left-0 bg-[var(--color-accent)] transition-all duration-500 ease-out rounded-full"
              style:width="{p.percent}%"
            ></div>
          </div>
          
          <!-- Temps restant fichier -->
          <div class="flex items-center justify-between text-[11px]">
            <span class="text-[var(--color-subtext)]">⏱️ Restant fichier</span>
            <span class="font-mono font-semibold text-[var(--color-text)]">{formatTime(p.remaining_file)}</span>
          </div>
        </div>

        <!-- Progression totale (si multi-fichiers) -->
        {#if p.file_total > 1}
          <div class="space-y-2 pt-1 border-t border-[var(--color-border)]">
            <div class="flex justify-between text-[11px]">
              <span class="text-[var(--color-text)]">📊 Progression globale</span>
              <span class="font-mono text-[var(--color-subtext)]">{totalPercent.toFixed(0)}%</span>
            </div>
            <div class="h-2 bg-[var(--color-surface)] border border-[var(--color-border)] rounded-full overflow-hidden">
              <div 
                class="h-full bg-[var(--color-accent)]/40 transition-all duration-500 ease-out rounded-full"
                style:width="{totalPercent}%"
              ></div>
            </div>
            <div class="flex items-center justify-between text-[11px]">
              <span class="text-[var(--color-subtext)]">⏱️ Restant total</span>
              <span class="font-mono font-semibold text-[var(--color-accent)]">{formatTime(p.remaining_total)}</span>
            </div>
          </div>
        {/if}

        <!-- Estimation gain -->
        {#if estimatedGainPct !== null}
          <div class="space-y-2 pt-1 border-t border-[var(--color-border)]">
            <div class="flex justify-between text-[11px]">
              <span class="text-[var(--color-text)]">📈 Gain estimé</span>
              <span class="font-mono font-semibold text-[var(--color-success)]">-{estimatedGainPct.toFixed(0)}%</span>
            </div>
            <div class="h-2 bg-[var(--color-surface)] border border-[var(--color-border)] rounded-full overflow-hidden">
              <div 
                class="h-full bg-[var(--color-success)]/40 transition-all duration-500 ease-out rounded-full"
                style:width="{100 - Math.min(95, estimatedGainPct)}%"
              ></div>
            </div>
            <div class="flex justify-between text-[10px] text-[var(--color-subtext)]">
              <span>📦 {formatSize(estimatedTotalMb!)}</span>
              <span>📁 {formatSize(totalOriginalMb)}</span>
            </div>
          </div>
        {:else if p.file_total > 1}
          <div class="pt-2 text-center text-[10px] text-[var(--color-subtext)]">
            📈 Estimation disponible après 1er fichier
          </div>
        {/if}
      {/if}
    </div>

  {:else if s}
    <!-- Summary -->
    <div class="flex items-center gap-2 px-4 py-2 border-b border-[var(--color-border)] bg-[var(--color-success)]/5 shrink-0">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" class="text-[var(--color-success)]">
        <polyline points="20 6 9 17 4 12"/>
      </svg>
      <span class="text-[11px] font-mono text-[var(--color-success)] uppercase tracking-wider">Terminé</span>
    </div>

    <div class="flex-1 p-4 overflow-auto">
      <!-- Stats -->
      <div class="grid grid-cols-3 gap-3 mb-4">
        <div class="text-center p-2 bg-[var(--color-surface)] rounded-[2px]">
          <div class="text-[9px] text-[var(--color-subtext)]">✅ Succès</div>
          <div class="text-[20px] font-bold text-[var(--color-text)]">{s.files.filter(f => f.status === "ok").length}/{s.files.length}</div>
        </div>
        <div class="text-center p-2 bg-[var(--color-surface)] rounded-[2px]">
          <div class="text-[9px] text-[var(--color-subtext)]">📦 Gain</div>
          <div class="text-[20px] font-bold text-[var(--color-success)]">{gainPct(s.total_original_mb, s.total_encoded_mb)}</div>
        </div>
        <div class="text-center p-2 bg-[var(--color-surface)] rounded-[2px]">
          <div class="text-[9px] text-[var(--color-subtext)]">⏱️ Durée</div>
          <div class="text-[20px] font-bold text-[var(--color-text)]">{formatTime(s.total_secs)}</div>
        </div>
      </div>
      
      <!-- Liste fichiers -->
      <div class="space-y-1 max-h-32 overflow-auto">
        {#each s.files as r}
          <div class="flex justify-between items-center text-[10px] font-mono px-2 py-1 rounded hover:bg-[var(--color-surface)] transition-colors">
            <span class="truncate text-[var(--color-subtext)] flex-1" title={r.name}>{r.name.slice(-35)}</span>
            <span class="shrink-0 ml-2 {r.status === 'ok' ? 'text-[var(--color-success)]' : 'text-[var(--color-danger)]'}">
              {r.status === 'ok' ? gainPct(r.original_mb, r.encoded_mb) : r.status === 'error' ? '❌ ERREUR' : '⏹️ ANNULÉ'}
            </span>
          </div>
        {/each}
      </div>
    </div>

  {:else}
    <!-- Empty state -->
    <div class="flex-1 flex items-center justify-center p-4">
      <div class="text-center">
        <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" class="mx-auto mb-2 text-[var(--color-subtext)]/20">
          <polygon points="22 12 18 9 6 9 2 12 6 15 18 15 22 12"/>
          <path d="M6 9L6 15"/><path d="M18 9L18 15"/>
        </svg>
        <p class="text-[12px] font-medium text-[var(--color-subtext)]">Prêt à encoder</p>
        <p class="text-[10px] text-[var(--color-subtext2)] mt-1">Ajoutez des fichiers pour commencer</p>
      </div>
    </div>
  {/if}
</div>