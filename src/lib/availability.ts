import { addDays, parseISO } from 'date-fns';
import { clockTime } from './format';
import { usePrefs } from './prefs';
import type { BlockoutWindow } from './types';

const DAY_KEYS = ['sun', 'mon', 'tue', 'wed', 'thu', 'fri', 'sat'] as const;
const toMinutes = (hhmm: string) => {
  const [h, m] = hhmm.split(':').map(Number);
  return (h ?? 0) * 60 + (m ?? 0);
};

/**
 * Planning hours on `day` (roadmap §4.5; Settings → General): the window
 * flexible tasks may be placed in, in minutes from midnight, or null on a
 * day with none.
 */
export function planningMinutes(day: Date): { start: number; end: number } | null {
  const span = usePrefs.getState().planning[DAY_KEYS[day.getDay()] ?? 'mon'];
  return span ? { start: toMinutes(span.start), end: toMinutes(span.end) } : null;
}

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
  return `${clockTime(parseISO(w.start))}–${clockTime(parseISO(w.end))}`;
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
  const hours = planningMinutes(day);
  if (!hours) return 0;
  const { start, end } = hours;
  return end - start - coveredMinutes(blockedIntervals(windows, day), start, end);
}

/** Hover text for a window: "Work 08:30–17:00 (block-out)", or "Team sync 10:00–11:00 (busy in Work calendar)". */
export function windowTitle(w: BlockoutWindow): string {
  const what = w.calendar ? `busy in ${w.calendar}` : 'block-out';
  return `${w.label} ${windowTimes(w)} (${what})`;
}
