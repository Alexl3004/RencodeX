export function formatTime(secs: number): string {
  if (!isFinite(secs) || secs < 0) return "--:--";
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = Math.floor(secs % 60);
  if (h > 0) return `${h}h ${String(m).padStart(2,"0")}m ${String(s).padStart(2,"0")}s`;
  if (m > 0) return `${String(m).padStart(2,"0")}m ${String(s).padStart(2,"0")}s`;
  return `${String(s).padStart(2,"0")}s`;
}

export function formatSize(mb: number): string {
  if (mb >= 1024) return `${(mb / 1024).toFixed(2)} GB`;
  return `${mb.toFixed(1)} MB`;
}

export function gainPct(before: number, after: number): string {
  if (before <= 0) return "--";
  return `${(100 * (before - after) / before).toFixed(1)}%`;
}
