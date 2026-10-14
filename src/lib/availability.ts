import { addDays, format, parseISO } from 'date-fns';
import type { BlockoutWindow } from './types';

/**
 * Planning hours: the window flexible tasks may be placed in (roadmap §4.5,
 * default 07:00–22:00). Minutes from midnight. The core has the per-weekday
 * setting since #20; this mirror goes when General settings land in #38.
 */
export const PLANNING_HOURS = { start: 7 * 60, end: 22 * 60 } as const;

export interface BlockedInterval {
  window: BlockoutWindow;
  /** Minutes from local midnight on the given day, clipped to [0, 1440]. */
  start: number;
  end: number;
}

const minutesInto = (t: Date) => t.getHours() * 60 + t.getMinutes();

/**
 * The parts of block-out `windows` (from the core) that cover `day`, in
 * minutes. A window past midnight (Sleep 23:00–06:30) shows on both days.
 */
export function blockedIntervals(windows: readonly BlockoutWindow[], day: Date): BlockedInterval[] {
  const dayStart = new Date(day.getFullYear(), day.getMonth(), day.getDate());
  const dayEnd = addDays(dayStart, 1);
  const out: BlockedInterval[] = [];
  for (const w of windows) {
    const s = parseISO(w.start);
    const e = parseISO(w.end);
    if (e <= dayStart || s >= dayEnd) continue;
    out.push({
      window: w,
      start: s <= dayStart ? 0 : minutesInto(s),
      end: e >= dayEnd ? 24 * 60 : minutesInto(e),
    });
  }
  return out.filter((i) => i.end > i.start).sort((a, b) => a.start - b.start);
}

/** Local `HH:mm–HH:mm` of a whole window, for labels. */
export function windowTimes(w: BlockoutWindow): string {
  return `${format(parseISO(w.start), 'HH:mm')}–${format(parseISO(w.end), 'HH:mm')}`;
}

/** Total minutes covered by intervals within [lo, hi], counting overlaps once. */
export function coveredMinutes(
  intervals: readonly { start: number; end: number }[],
  lo: number,
  hi: number,
): number {
  const clipped = intervals
    .map((i) => ({ start: Math.max(lo, i.start), end: Math.min(hi, i.end) }))
    .filter((i) => i.end > i.start)
    .sort((a, b) => a.start - b.start);
  let total = 0;
  let cursor = lo;
  for (const i of clipped) {
    const from = Math.max(cursor, i.start);
    if (i.end > from) {
      total += i.end - from;
      cursor = i.end;
    }
  }
  return total;
}

/** Minutes inside planning hours on `day` not covered by a block-out. */
export function availableMinutes(windows: readonly BlockoutWindow[], day: Date): number {
  const { start, end } = PLANNING_HOURS;
  return end - start - coveredMinutes(blockedIntervals(windows, day), start, end);
}
