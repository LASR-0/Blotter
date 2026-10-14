// Pure drag-and-drop rules: where a pointer lands on a time grid, whether a
// drop is refused, and which operation a drop becomes.

import { format, parseISO } from 'date-fns';
import { blockedIntervals } from './availability';
import type { OpCallSpec } from './itemDraft';
import type { Blockout, Item } from './types';

/** Drop slots snap to 15 minutes (design annotation). */
export const SNAP_MIN = 15;
const DAY_MIN = 24 * 60;

export const snap = (minutes: number, step = SNAP_MIN) => Math.round(minutes / step) * step;

export const hhmm = (min: number) =>
  `${String(Math.floor(min / 60)).padStart(2, '0')}:${String(min % 60).padStart(2, '0')}`;

/**
 * Start minute for a moved item. `pointerMin` is the time under the
 * pointer; `grabMin` how far into the item it was grabbed, so the item
 * doesn't jump. Snapped and kept within the day.
 */
export function slotStart(pointerMin: number, grabMin: number, durationMin: number): number {
  const start = snap(pointerMin - grabMin);
  return Math.max(0, Math.min(DAY_MIN - Math.max(SNAP_MIN, durationMin), start));
}

/** End minute for a resize: snapped, at least one step after the start, within the day. */
export function resizeEnd(pointerMin: number, startMin: number): number {
  return Math.max(startMin + SNAP_MIN, Math.min(DAY_MIN, snap(pointerMin)));
}

/** Minutes an item lasts (its span, else its duration, else 30). */
export function itemLength(item: Item): number {
  if (item.start_at && item.end_at)
    return Math.max(
      SNAP_MIN,
      (parseISO(item.end_at).getTime() - parseISO(item.start_at).getTime()) / 60_000,
    );
  return item.duration_min ?? 30;
}

/**
 * The block-out a flexible item would land in, or null if the drop is
 * allowed. Fixed items may sit inside block-outs (roadmap §4.5 rule 2).
 */
export function refusedBy(
  item: Pick<Item, 'anchoring'>,
  date: Date,
  startMin: number,
  endMin: number,
  blockouts: readonly Blockout[],
): Blockout | null {
  if (item.anchoring === 'fixed') return null;
  const hit = blockedIntervals(blockouts, date).find((b) => startMin < b.end && endMin > b.start);
  return hit?.blockout ?? null;
}

/** A sort_order between neighbours (either may be missing at the ends). */
export function between(before: number | undefined, after: number | undefined): number {
  if (before === undefined && after === undefined) return 0;
  if (before === undefined) return (after as number) - 1;
  if (after === undefined) return before + 1;
  return (before + after) / 2;
}

const ymd = (d: Date) => format(d, 'yyyy-MM-dd');

export type Drop =
  | { kind: 'slot'; date: Date; startMin: number }
  | { kind: 'resize'; date: Date; endMin: number }
  | { kind: 'day'; date: Date; sortOrder?: number }
  | { kind: 'inbox' };

/** The operation a drop performs, or an error message if it can't apply. */
export function dropCall(item: Item, drop: Drop): OpCallSpec | { error: string } {
  switch (drop.kind) {
    case 'slot':
      return {
        name: 'update_item',
        args: { id: item.id, start: `${ymd(drop.date)}T${hhmm(drop.startMin)}` },
      };
    case 'resize':
      return {
        name: 'update_item',
        args: { id: item.id, end: `${ymd(drop.date)}T${hhmm(drop.endMin)}` },
      };
    case 'day': {
      const args: Record<string, unknown> = { id: item.id };
      if (item.start_at && !item.all_day) {
        // Timed items keep their time on the new day.
        args.start = `${ymd(drop.date)}T${format(parseISO(item.start_at), 'HH:mm')}`;
      } else {
        args.date = ymd(drop.date);
        if (drop.sortOrder !== undefined) args.sort_order = drop.sortOrder;
      }
      return { name: 'update_item', args };
    }
    case 'inbox':
      if (item.kind !== 'task')
        return { error: `Only tasks go to the Inbox; this is a ${item.kind}.` };
      return { name: 'unschedule_item', args: { id: item.id } };
  }
}
