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

/**
 * Parses what people type into a duration field: `45`, `45m`, `1h`,
 * `1h30`, `1h 30m`, `1.5h`, `90min`. Returns minutes, or null if it can't
 * be read or isn't positive.
 */
export function parseDuration(input: string): number | null {
  const s = input.trim().toLowerCase().replace(/\s+/g, '');
  if (!s) return null;
  if (/^\d+$/.test(s)) return positive(Number(s));
  const m = /^(?:(\d+(?:\.\d+)?)h(?:ours?|rs?)?)?(?:(\d+)m?(?:in(?:s|utes?)?)?)?$/.exec(s);
  if (!m || (m[1] === undefined && m[2] === undefined)) return null;
  const hours = m[1] ? Number(m[1]) : 0;
  const minutes = m[2] ? Number(m[2]) : 0;
  return positive(Math.round(hours * 60 + minutes));
}

function positive(n: number): number | null {
  return Number.isFinite(n) && n > 0 ? n : null;
}
