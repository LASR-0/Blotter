import {
  addDays,
  differenceInCalendarDays,
  format,
  isSameDay,
  isSameYear,
  parseISO,
} from 'date-fns';

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

/** Mirrors `SubscriptionInfo` in crates/blotter-core/src/subscriptions.rs. */
export interface Subscription {
  id: string;
  name: string;
  url: string;
  refresh_min: number;
  color: string | null;
  as_blockout: boolean;
  enabled: boolean;
  last_fetch_at: string | null;
  last_ok_at: string | null;
  last_status: 'ok' | 'error' | null;
  last_error: string | null;
  created_at: string;
  updated_at: string;
  /** Events in the cache. */
  events: number;
}

/** Refresh intervals offered, in minutes (the core takes 5 minutes to 7 days). */
export const REFRESH_CHOICES = [15, 30, 60, 180, 360, 720, 1440] as const;

/** "15 min", "1 hour", "6 hours", "Daily", "2 days". */
export function refreshLabel(min: number): string {
  if (min < 60) return `${min} min`;
  if (min % 1440 === 0) return min === 1440 ? 'Daily' : `${min / 1440} days`;
  const h = min / 60;
  return Number.isInteger(h) ? count(h, 'hour', 'hours') : `${min} min`;
}

/** When `iso` was: "10:30" today, "yesterday 10:30", else "Mon 12 Oct". */
function when(iso: string, now: Date): string {
  const t = parseISO(iso);
  if (isSameDay(t, now)) return format(t, 'HH:mm');
  if (isSameDay(t, addDays(now, -1))) return `yesterday ${format(t, 'HH:mm')}`;
  return format(t, isSameYear(t, now) ? 'EEE d MMM' : 'd MMM yyyy');
}

function ago(iso: string, now: Date): string {
  const days = differenceInCalendarDays(now, parseISO(iso));
  if (days <= 0) return `at ${format(parseISO(iso), 'HH:mm')}`;
  return days === 1 ? 'yesterday' : `${days} days ago`;
}

export interface SubscriptionStatus {
  tone: 'ok' | 'error' | 'pending' | 'off';
  text: string;
}

/** The status line: "Fetched 10:30 · 48 events", "HTTP 404 · last success 3 days ago". */
export function subscriptionStatus(s: Subscription, now = new Date()): SubscriptionStatus {
  if (!s.enabled) return { tone: 'off', text: 'Paused' };
  if (!s.last_fetch_at) return { tone: 'pending', text: 'Not fetched yet' };
  if (s.last_status === 'error') {
    const why = s.last_error ?? 'Couldn’t fetch';
    const reason = why.charAt(0).toUpperCase() + why.slice(1);
    return {
      tone: 'error',
      text: s.last_ok_at
        ? `${reason} · last success ${ago(s.last_ok_at, now)}`
        : `${reason} · never fetched`,
    };
  }
  return {
    tone: 'ok',
    text: `Fetched ${when(s.last_fetch_at, now)} · ${count(s.events, 'event', 'events')}`,
  };
}
