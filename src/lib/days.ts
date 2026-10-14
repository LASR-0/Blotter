import { addDays, format, parseISO, startOfDay } from 'date-fns';
import type { Item } from './types';

/** Local-date key, `YYYY-MM-DD`. */
export const dayKey = (d: Date): string => format(d, 'yyyy-MM-dd');

/**
 * Buckets items by every local day they touch: timed items on each day
 * between start and end (an item ending exactly at midnight doesn't spill
 * into the next day), all-day items on each date from start to end.
 * Within a day, all-day items come first, then by start time.
 */
export function itemsByDay(items: readonly Item[]): Map<string, Item[]> {
  const out = new Map<string, Item[]>();
  const add = (key: string, item: Item) => {
    const list = out.get(key);
    if (list) list.push(item);
    else out.set(key, [item]);
  };

  for (const item of items) {
    if (item.all_day && item.start_date) {
      const last = parseISO(item.end_date ?? item.start_date);
      for (let d = parseISO(item.start_date); d <= last; d = addDays(d, 1)) add(dayKey(d), item);
    } else if (item.start_at) {
      const start = parseISO(item.start_at);
      const end = item.end_at ? parseISO(item.end_at) : start;
      // The last day touched is the day of (end − 1 ms), unless zero-length.
      const lastInstant = end > start ? new Date(end.getTime() - 1) : start;
      for (let d = startOfDay(start); d <= lastInstant; d = addDays(d, 1)) add(dayKey(d), item);
    }
  }

  for (const list of out.values())
    list.sort((a, b) => {
      if (a.all_day !== b.all_day) return a.all_day ? -1 : 1;
      return (a.start_at ?? '').localeCompare(b.start_at ?? '') || a.sort_order - b.sort_order;
    });
  return out;
}

/** Completed items keyed by the local day they were completed. */
export function completionsByDay(items: readonly Item[]): Map<string, Item[]> {
  const out = new Map<string, Item[]>();
  for (const item of items) {
    if (item.status !== 'done' || !item.completed_at) continue;
    const key = dayKey(parseISO(item.completed_at));
    const list = out.get(key);
    if (list) list.push(item);
    else out.set(key, [item]);
  }
  return out;
}

/** Heatmap level 0–4 for an item count (design thresholds: 0, <3, <5, <7, 7+). */
export function heatLevel(count: number): 0 | 1 | 2 | 3 | 4 {
  if (count <= 0) return 0;
  if (count < 3) return 1;
  if (count < 5) return 2;
  if (count < 7) return 3;
  return 4;
}
