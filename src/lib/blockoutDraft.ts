// Pure rules behind the Block-outs settings page: the edit form's draft,
// validation, the minimal operation calls a save becomes, reordering and
// lifting a range of dates.

import { addDays, differenceInCalendarDays, format, parseISO } from 'date-fns';
import { parseTime } from './format';
import type { OpCallSpec } from './itemDraft';
import type { Blockout, Weekday } from './types';

/** Monday-first, as the day toggles show them. */
export const WEEKDAYS: readonly Weekday[] = ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun'];
const JS_DAY: readonly Weekday[] = ['sun', 'mon', 'tue', 'wed', 'thu', 'fri', 'sat'];

/** Longest range "lift from … to …" accepts in one go. */
export const MAX_LIFT_DAYS = 366;

export interface BlockoutDraft {
  label: string;
  weekdays: Weekday[];
  /** As typed; normalised to HH:MM on save. */
  start: string;
  end: string;
  permitsFixed: boolean;
}

export function emptyBlockoutDraft(): BlockoutDraft {
  return {
    label: '',
    weekdays: ['mon', 'tue', 'wed', 'thu', 'fri'],
    start: '09:00',
    end: '17:00',
    permitsFixed: true,
  };
}

export function draftFromBlockout(b: Blockout): BlockoutDraft {
  return {
    label: b.label,
    weekdays: [...b.weekdays],
    start: b.start_time,
    end: b.end_time,
    permitsFixed: b.permits_fixed,
  };
}

/** The first problem with a draft, or null if it can be saved. */
export function validateBlockout(d: BlockoutDraft): string | null {
  if (!d.label.trim()) return 'Give the block-out a label.';
  if (d.weekdays.length === 0) return 'Pick at least one day.';
  const start = parseTime(d.start);
  const end = parseTime(d.end);
  if (!start) return `"${d.start}" isn't a time.`;
  if (!end) return `"${d.end}" isn't a time.`;
  if (start === end) return 'Start and end must differ.';
  return null;
}

const sameDays = (a: readonly Weekday[], b: readonly Weekday[]) =>
  a.length === b.length && a.every((d) => b.includes(d));

const ordered = (days: readonly Weekday[]) => WEEKDAYS.filter((d) => days.includes(d));

/**
 * The call a valid draft's save becomes: a create for a new block-out,
 * an update with only the changed fields, or null when nothing changed.
 */
export function blockoutSave(before: Blockout | null, d: BlockoutDraft): OpCallSpec | null {
  const start = parseTime(d.start) as string;
  const end = parseTime(d.end) as string;
  const label = d.label.trim();
  if (!before) {
    return {
      name: 'create_blockout',
      args: {
        label,
        weekdays: ordered(d.weekdays),
        start_time: start,
        end_time: end,
        permits_fixed: d.permitsFixed,
      },
    };
  }
  const args: Record<string, unknown> = {};
  if (label !== before.label) args.label = label;
  if (!sameDays(d.weekdays, before.weekdays)) args.weekdays = ordered(d.weekdays);
  if (start !== before.start_time) args.start_time = start;
  if (end !== before.end_time) args.end_time = end;
  if (d.permitsFixed !== before.permits_fixed) args.permits_fixed = d.permitsFixed;
  return Object.keys(args).length
    ? { name: 'update_blockout', args: { id: before.id, ...args } }
    : null;
}

/** Moves `id` to `index` and returns the update calls for rows whose position changed. */
export function reorderCalls(
  blockouts: readonly Blockout[],
  id: string,
  index: number,
): OpCallSpec[] {
  const ids = blockouts.map((b) => b.id).filter((x) => x !== id);
  const at = Math.max(0, Math.min(index, ids.length));
  ids.splice(at, 0, id);
  return ids.flatMap((bid, i) => {
    const b = blockouts.find((x) => x.id === bid);
    return b && b.sort_order !== i
      ? [{ name: 'update_blockout', args: { id: bid, sort_order: i } }]
      : [];
  });
}

/** Whether a block-out's window starts on `date` (weekday and active range). */
export function appliesOn(b: Blockout, date: Date): boolean {
  const key = format(date, 'yyyy-MM-dd');
  if (!b.weekdays.includes(JS_DAY[date.getDay()] as Weekday)) return false;
  if (b.active_from && key < b.active_from) return false;
  return !(b.active_until && key > b.active_until);
}

/** Whether `date` is lifted for this block-out. */
export function liftedOn(b: Blockout, date: Date): boolean {
  const key = format(date, 'yyyy-MM-dd');
  return b.exceptions.some((e) => e.date === key);
}

/**
 * Exception calls lifting `b` on every date from `from` to `to` (inclusive)
 * where it applies and isn't lifted already. Throws a readable message for
 * a range it can't take.
 */
export function liftCalls(b: Blockout, from: string, to: string, note: string): OpCallSpec[] {
  const first = parseISO(from);
  const last = parseISO(to || from);
  const days = differenceInCalendarDays(last, first);
  if (Number.isNaN(days)) throw new Error('Pick a date.');
  if (days < 0) throw new Error('The end date is before the start.');
  if (days >= MAX_LIFT_DAYS) throw new Error(`Lift at most ${MAX_LIFT_DAYS} days at a time.`);
  const calls: OpCallSpec[] = [];
  for (let i = 0; i <= days; i++) {
    const date = addDays(first, i);
    if (appliesOn(b, date) && !liftedOn(b, date)) {
      calls.push({
        name: 'add_blockout_exception',
        args: { blockout_id: b.id, date: format(date, 'yyyy-MM-dd'), note: note.trim() },
      });
    }
  }
  return calls;
}

const DAY_LABEL: Record<Weekday, string> = {
  mon: 'Mon',
  tue: 'Tue',
  wed: 'Wed',
  thu: 'Thu',
  fri: 'Fri',
  sat: 'Sat',
  sun: 'Sun',
};

/** Readable days, e.g. "Mon–Fri", "Every day", "Weekends", "Mon, Wed, Fri". */
export function describeDays(days: readonly Weekday[]): string {
  const set = ordered(days);
  if (set.length === 7) return 'Every day';
  if (sameDays(set, ['sat', 'sun'])) return 'Weekends';
  const idx = set.map((d) => WEEKDAYS.indexOf(d));
  const consecutive = idx.every((v, i) => i === 0 || v === (idx[i - 1] as number) + 1);
  if (consecutive && set.length >= 3) {
    return `${DAY_LABEL[set[0] as Weekday]}–${DAY_LABEL[set[set.length - 1] as Weekday]}`;
  }
  return set.map((d) => DAY_LABEL[d]).join(', ');
}
