/**
 * Formats a minute count the way Blotter shows durations everywhere:
 * `45m`, `1h`, `1h 30m`. Negative or non-finite input renders as `0m`.
 */
export function formatDuration(minutes: number): string {
  if (!Number.isFinite(minutes) || minutes <= 0) return '0m';
  const total = Math.round(minutes);
  const h = Math.floor(total / 60);
  const m = total % 60;
  if (h === 0) return `${m}m`;
  if (m === 0) return `${h}h`;
  return `${h}h ${m}m`;
}
