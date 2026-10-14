import { addDays, addMinutes, format, parseISO } from 'date-fns';
import type { OpCallSpec } from './itemDraft';
import type { Item } from './types';

/** Local wall-clock form the operation layer reads in the user's timezone. */
const localStamp = (d: Date) => format(d, "yyyy-MM-dd'T'HH:mm");
const shiftDate = (ymd: string, days: number) => format(addDays(parseISO(ymd), days), 'yyyy-MM-dd');

/** Moves an item by whole days, keeping its time. Null for unscheduled items. */
export function moveByDays(item: Item, days: number): OpCallSpec | null {
  if (item.all_day && item.start_date) {
    const args: Record<string, unknown> = { id: item.id, date: shiftDate(item.start_date, days) };
    if (item.end_date) args.end_date = shiftDate(item.end_date, days);
    return { name: 'update_item', args };
  }
  if (item.start_at) {
    return {
      name: 'update_item',
      args: { id: item.id, start: localStamp(addDays(parseISO(item.start_at), days)) },
    };
  }
  return null;
}

/** Moves a timed item by minutes, keeping its duration. */
export function moveByMinutes(item: Item, minutes: number): OpCallSpec | null {
  if (item.all_day || !item.start_at) return null;
  const start = addMinutes(parseISO(item.start_at), minutes);
  const args: Record<string, unknown> = { id: item.id, start: localStamp(start) };
  if (item.end_at) {
    const length = parseISO(item.end_at).getTime() - parseISO(item.start_at).getTime();
    args.end = localStamp(new Date(start.getTime() + length));
  }
  return { name: 'update_item', args };
}

/** Shortest a keyboard resize may make an item. */
export const MIN_LENGTH_MIN = 15;

/** Moves a timed item's end by minutes, never below MIN_LENGTH_MIN. */
export function resizeByMinutes(item: Item, minutes: number): OpCallSpec | null {
  if (item.all_day || !item.start_at) return null;
  const start = parseISO(item.start_at);
  const end = item.end_at ? parseISO(item.end_at) : start;
  const next = addMinutes(end, minutes);
  const floor = addMinutes(start, MIN_LENGTH_MIN);
  const clamped = next < floor ? floor : next;
  if (clamped.getTime() === end.getTime()) return null;
  return { name: 'update_item', args: { id: item.id, end: localStamp(clamped) } };
}
