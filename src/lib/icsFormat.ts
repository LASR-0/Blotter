import { format, isSameYear, parseISO } from 'date-fns';

/** Mirrors `ImportSummary` in crates/blotter-core/src/ics/import.rs. */
export interface IcsSummary {
  scheduled: number;
  inbox: number;
  recurring: number;
  occurrences: number;
  reminders: number;
  already: number;
  skipped: number;
  first: string | null;
  last: string | null;
  warnings: string[];
}

const count = (n: number, one: string, many: string) => `${n} ${n === 1 ? one : many}`;

/** "12 calendar items and 3 Inbox tasks", or "nothing new". */
export function importWhat(s: IcsSummary): string {
  const parts = [
    s.scheduled > 0 && count(s.scheduled, 'calendar item', 'calendar items'),
    s.inbox > 0 && count(s.inbox, 'Inbox task', 'Inbox tasks'),
  ].filter(Boolean);
  if (parts.length === 0 && s.occurrences > 0) {
    return count(s.occurrences, 'change to a repeating item', 'changes to repeating items');
  }
  return parts.length ? parts.join(' and ') : 'nothing new';
}

/** "on Tue 20 Oct", "from Mon 12 Oct to Fri 30 Oct", years shown when they differ. */
export function importSpan(s: IcsSummary, today = new Date()): string {
  if (!s.first || !s.last) return '';
  const first = parseISO(s.first);
  const last = parseISO(s.last);
  const sameYear = isSameYear(first, last) && isSameYear(first, today);
  const day = (d: Date) => format(d, sameYear ? 'EEE d MMM' : 'EEE d MMM yyyy');
  return s.first === s.last ? `on ${day(first)}` : `from ${day(first)} to ${day(last)}`;
}

/** The details after the headline: repeats, reminders and what was left out. */
export function importNotes(s: IcsSummary): string[] {
  const notes: string[] = [];
  if (s.recurring > 0) {
    notes.push(`${count(s.recurring, 'item repeats', 'items repeat')}.`);
  }
  if (s.reminders > 0) notes.push(`${count(s.reminders, 'reminder', 'reminders')}.`);
  if (s.already > 0) {
    notes.push(`${count(s.already, 'item is', 'items are')} already in Blotter and left out.`);
  }
  if (s.skipped > 0) {
    notes.push(
      `${count(s.skipped, 'cancelled or undated item was', 'cancelled or undated items were')} left out.`,
    );
  }
  return [...notes, ...s.warnings];
}

/** "Exported 12 calendar items and 3 Inbox tasks to C:\…\blotter.ics". */
export function exportedText(events: number, todos: number, path: string): string {
  const parts = [
    count(events, 'calendar item', 'calendar items'),
    todos > 0 && count(todos, 'Inbox task', 'Inbox tasks'),
  ].filter(Boolean);
  return `Exported ${parts.join(' and ')} to ${path}`;
}
