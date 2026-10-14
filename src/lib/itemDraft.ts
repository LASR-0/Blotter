import { format, parseISO } from 'date-fns';
import { formatDuration, parseDuration } from './duration';
import { parseTime } from './format';
import type { Anchoring, Item, ItemKind } from './types';

/** An item being edited, in the shape of the dialog's fields (local times). */
export interface ItemDraft {
  title: string;
  kind: ItemKind;
  anchoring: Anchoring;
  /** `YYYY-MM-DD`, or '' for unscheduled. */
  date: string;
  /** `HH:MM`, or '' for all-day (when a date is set). */
  time: string;
  /** Free text, e.g. `30m` or `1h30`; '' for none. */
  duration: string;
  dueDate: string;
  dueTime: string;
  priority: number;
  tags: string[];
  notes: string;
  /** RRULE, or '' for none. */
  repeat: string;
  /** Existing reminders keep their id; new ones have none. */
  reminders: { id?: string; offset_min: number | null; at: string | null }[];
}

export interface ReminderLike {
  id: string;
  offset_min: number | null;
  at: string | null;
}

/** Which occurrences of a recurring item a change applies to. */
export type RepeatScope = 'this' | 'following' | 'all';

/** Whether an item is (an occurrence of) a recurring series. */
export function isRecurring(item: Pick<Item, 'rrule' | 'parent_id'>): boolean {
  return Boolean(item.rrule || item.parent_id);
}

export interface OpCallSpec {
  name: string;
  args: Record<string, unknown>;
}

const localDate = (iso: string) => format(parseISO(iso), 'yyyy-MM-dd');
const localTime = (iso: string) => format(parseISO(iso), 'HH:mm');

export function emptyDraft(defaults: Partial<ItemDraft> = {}): ItemDraft {
  return {
    title: '',
    kind: 'task',
    anchoring: 'flexible',
    date: '',
    time: '',
    duration: '',
    dueDate: '',
    dueTime: '',
    priority: 0,
    tags: [],
    notes: '',
    repeat: '',
    reminders: [],
    ...defaults,
  };
}

export function draftFromItem(item: Item, reminders: readonly ReminderLike[] = []): ItemDraft {
  return {
    title: item.title,
    kind: item.kind,
    anchoring: item.anchoring,
    date: item.all_day ? (item.start_date ?? '') : item.start_at ? localDate(item.start_at) : '',
    time: !item.all_day && item.start_at ? localTime(item.start_at) : '',
    duration: item.duration_min ? formatDuration(item.duration_min).replace(' ', '') : '',
    dueDate: item.due_at ? localDate(item.due_at) : '',
    dueTime: item.due_at ? localTime(item.due_at) : '',
    priority: item.priority,
    tags: [...item.tags],
    notes: item.notes,
    repeat: item.rrule ?? '',
    reminders: reminders.map((r) => ({ id: r.id, offset_min: r.offset_min, at: r.at })),
  };
}

export type DraftResult =
  | { ok: false; error: string }
  | {
      ok: true;
      /** For a new item: `create_item` args. */
      create?: Record<string, unknown>;
      /** For an existing item: calls to run as one batch. */
      calls: OpCallSpec[];
      /** Reminder offsets to add once a new item has an id. */
      newReminders: number[];
    };

/** Time a due date means when no time is given (end of the working day). */
const DEFAULT_DUE_TIME = '17:00';

function validate(d: ItemDraft): string | null {
  if (!d.title.trim()) return 'Give it a title.';
  if (d.time.trim() && parseTime(d.time) === null)
    return `Can't read "${d.time}" as a time; try 17:30 or 5:30pm.`;
  if (d.dueTime.trim() && parseTime(d.dueTime) === null)
    return `Can't read "${d.dueTime}" as a time.`;
  if (d.time.trim() && !d.date) return 'A time needs a date.';
  if (d.duration.trim() && parseDuration(d.duration) === null)
    return `Can't read "${d.duration}" as a duration; try 30m or 1h30.`;
  if (d.dueTime.trim() && !d.dueDate) return 'A due time needs a due date.';
  if ((d.kind === 'event' || d.kind === 'reminder') && !d.date) return `A ${d.kind} needs a date.`;
  if (d.repeat && !d.date) return 'A repeating item needs a date to start from.';
  return null;
}

const dueValue = (d: ItemDraft) =>
  d.dueDate ? `${d.dueDate}T${d.dueTime || DEFAULT_DUE_TIME}` : null;
const sameList = (a: readonly string[], b: readonly string[]) =>
  a.length === b.length && a.every((x, i) => x === b[i]);

/**
 * Turns a draft into operations: `create_item` args for a new item, or a
 * minimal batch (update, unschedule, reminder changes) for an existing one.
 */
export function buildChanges(
  draft: ItemDraft,
  original?: { item: Item; draft: ItemDraft; scope?: RepeatScope },
): DraftResult {
  const error = validate(draft);
  if (error) return { ok: false, error };
  // Normalise typed times (5:30pm → 17:30) before comparing or sending.
  draft = { ...draft, time: parseTime(draft.time) ?? '', dueTime: parseTime(draft.dueTime) ?? '' };
  const minutes = draft.duration.trim() ? parseDuration(draft.duration) : null;

  if (!original) {
    const args: Record<string, unknown> = {
      title: draft.title.trim(),
      kind: draft.kind,
      anchoring: draft.anchoring,
    };
    if (draft.date && draft.time) args.start = `${draft.date}T${draft.time}`;
    else if (draft.date) args.date = draft.date;
    if (minutes) args.duration_min = minutes;
    if (dueValue(draft)) args.due = dueValue(draft);
    if (draft.priority) args.priority = draft.priority;
    if (draft.tags.length) args.tags = draft.tags;
    if (draft.notes.trim()) args.notes = draft.notes;
    if (draft.repeat) args.rrule = draft.repeat;
    const newReminders = draft.reminders.flatMap((r) =>
      r.offset_min !== null ? [r.offset_min] : [],
    );
    // Always explicit, so the core's default event reminder doesn't add another.
    args.reminders = draft.date ? newReminders : [];
    return { ok: true, create: args, calls: [], newReminders };
  }

  const { item, draft: before } = original;
  const update: Record<string, unknown> = {};
  const calls: OpCallSpec[] = [];

  if (draft.title.trim() !== before.title) update.title = draft.title.trim();
  if (draft.kind !== before.kind) update.kind = draft.kind;
  if (draft.anchoring !== before.anchoring) update.anchoring = draft.anchoring;
  if (draft.priority !== before.priority) update.priority = draft.priority;
  if (!sameList(draft.tags, before.tags)) update.tags = draft.tags;
  if (draft.notes !== before.notes) update.notes = draft.notes;
  if (dueValue(draft) !== dueValue(before)) update.due = dueValue(draft);
  if (draft.repeat !== before.repeat) update.rrule = draft.repeat || null;

  const beforeMinutes = before.duration ? parseDuration(before.duration) : null;
  const scheduleChanged = draft.date !== before.date || draft.time !== before.time;
  const durationChanged = minutes !== beforeMinutes;
  let unschedule = false;
  if (!draft.date) {
    if (before.date) unschedule = true;
    if (durationChanged) update.duration_min = minutes;
  } else if (draft.time) {
    if (scheduleChanged) {
      update.start = `${draft.date}T${draft.time}`;
      if (minutes) update.duration_min = minutes;
    } else if (durationChanged) {
      update.duration_min = minutes;
    }
  } else {
    if (scheduleChanged) update.date = draft.date;
    if (durationChanged) update.duration_min = minutes;
  }

  // One occurrence of a series: say which occurrences the change is for.
  const scope = isRecurring(item) ? { scope: original.scope ?? 'this' } : {};
  if (Object.keys(update).length > 0)
    calls.push({ name: 'update_item', args: { id: item.id, ...update, ...scope } });
  if (unschedule) calls.push({ name: 'unschedule_item', args: { id: item.id } });

  const keptIds = new Set(draft.reminders.flatMap((r) => (r.id ? [r.id] : [])));
  for (const r of before.reminders)
    if (r.id && !keptIds.has(r.id))
      calls.push({ name: 'clear_reminder', args: { reminder_id: r.id } });
  for (const r of draft.reminders)
    if (!r.id && r.offset_min !== null)
      calls.push({ name: 'set_reminder', args: { item_id: item.id, offset_min: r.offset_min } });

  return { ok: true, calls, newReminders: [] };
}
