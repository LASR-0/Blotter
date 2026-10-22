import { differenceInCalendarDays, format, parseISO } from 'date-fns';
import type { Item } from './types';

/** Mirrors `ops::history::Completion`. */
export interface Completion extends Item {
  done_by: 'user' | 'ai' | 'mcp' | 'system';
  /** Local day it was completed on, `YYYY-MM-DD`. */
  done_on: string;
  /** The other device it was completed on, by name; absent for this one. */
  device?: string;
}

/** Mirrors `ops::history::History`. */
export interface History {
  start: string;
  end: string;
  timezone: string;
  completions: Completion[];
  truncated: boolean;
  streaks: { current: number; best: number };
  today: number;
  this_week: number;
  this_month: number;
  recent: { date: string; count: number }[];
}

export interface DayGroup {
  date: string;
  /** "Today", "Yesterday", a weekday within the week, else the date. */
  label: string;
  /** "Tue 13 Oct". */
  sub: string;
  items: Completion[];
}

/** Whether a completion passes the search text and isn't in a hidden tag. */
export function matches(c: Completion, query: string, hidden: ReadonlySet<string>): boolean {
  if (c.tags.length > 0 && c.tags.every((t) => hidden.has(t))) return false;
  const q = query.trim().toLowerCase();
  return !q || c.title.toLowerCase().includes(q) || c.notes.toLowerCase().includes(q);
}

/** Completions grouped by the day they were done, newest day first. */
export function groupByDay(completions: readonly Completion[], today: Date): DayGroup[] {
  const groups = new Map<string, Completion[]>();
  for (const c of completions) {
    const list = groups.get(c.done_on);
    if (list) list.push(c);
    else groups.set(c.done_on, [c]);
  }
  return [...groups.entries()]
    .sort(([a], [b]) => b.localeCompare(a))
    .map(([date, items]) => {
      const d = parseISO(date);
      const ago = differenceInCalendarDays(today, d);
      const label =
        ago === 0
          ? 'Today'
          : ago === 1
            ? 'Yesterday'
            : ago < 7
              ? format(d, 'EEEE')
              : format(d, 'd MMMM');
      return { date, label, sub: format(d, 'EEE d MMM'), items };
    });
}

/** Tags among `completions`, most used first. */
export function tagsOf(completions: readonly Completion[]): string[] {
  const counts = new Map<string, number>();
  for (const c of completions) for (const t of c.tags) counts.set(t, (counts.get(t) ?? 0) + 1);
  return [...counts.entries()]
    .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
    .map(([t]) => t);
}

/** "9 days", "1 day", "—" for none. */
export const daysText = (n: number): string => (n === 0 ? '—' : `${n} ${n === 1 ? 'day' : 'days'}`);
