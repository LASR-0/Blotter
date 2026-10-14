import { format, subDays } from 'date-fns';
import type { Blockout, Weekday } from './types';

/**
 * Planning hours: the window flexible tasks may be placed in (roadmap §4.5,
 * default 07:00–22:00). Minutes from midnight. Becomes a setting in #38.
 */
export const PLANNING_HOURS = { start: 7 * 60, end: 22 * 60 } as const;

const WEEKDAYS: Weekday[] = ['sun', 'mon', 'tue', 'wed', 'thu', 'fri', 'sat'];

const toMinutes = (hhmm: string) => {
  const [h, m] = hhmm.split(':').map(Number);
  return (h ?? 0) * 60 + (m ?? 0);
};

/** Whether a block-out's window starts on `date` (weekday, active range, not lifted). */
export function blockoutStartsOn(b: Blockout, date: Date): boolean {
  const key = format(date, 'yyyy-MM-dd');
  const weekday = WEEKDAYS[date.getDay()] as Weekday;
  if (!b.weekdays.includes(weekday)) return false;
  if (b.active_from && key < b.active_from) return false;
  if (b.active_until && key > b.active_until) return false;
  return !b.exceptions.some((e) => e.date === key);
}

export interface BlockedInterval {
  blockout: Blockout;
  /** Minutes from midnight on the given day, clipped to [0, 1440]. */
  start: number;
  end: number;
}

/**
 * The parts of `blockouts` that cover `date`. A window wrapping past
 * midnight (Sleep 23:00–06:30) covers the evening it starts and the early
 * morning of the next day.
 */
export function blockedIntervals(blockouts: readonly Blockout[], date: Date): BlockedInterval[] {
  const prev = subDays(date, 1);
  const out: BlockedInterval[] = [];
  for (const b of blockouts) {
    const start = toMinutes(b.start_time);
    const end = toMinutes(b.end_time);
    if (end > start) {
      if (blockoutStartsOn(b, date)) out.push({ blockout: b, start, end });
    } else {
      if (blockoutStartsOn(b, prev) && end > 0) out.push({ blockout: b, start: 0, end });
      if (blockoutStartsOn(b, date)) out.push({ blockout: b, start, end: 24 * 60 });
    }
  }
  return out.sort((a, b) => a.start - b.start);
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

/** Minutes inside planning hours on `date` not covered by a block-out. */
export function availableMinutes(blockouts: readonly Blockout[], date: Date): number {
  const { start, end } = PLANNING_HOURS;
  return end - start - coveredMinutes(blockedIntervals(blockouts, date), start, end);
}
