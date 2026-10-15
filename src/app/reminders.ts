import { create } from 'zustand';
import { addDays, differenceInMinutes, format, parseISO } from 'date-fns';
import { formatDuration, parseDuration } from '@/lib/duration';
import { parseTime } from '@/lib/format';

/** One reminder firing, as the engine sends it (`alerts::Due`). */
export interface Due {
  reminder_id: string;
  /** The item to open: an occurrence id for a recurring series. */
  item_id: string;
  title: string;
  start: string | null;
  end: string | null;
  all_day: boolean;
  due: string;
  snoozed: boolean;
}

/** Reminders that came due while Blotter was closed or the computer asleep. */
export interface Missed {
  since: string;
  while_closed: boolean;
  items: Due[];
}

/** What fired before the window was listening (`take_reminder_notices`). */
export interface PendingNotices {
  fired: Due[];
  missed: Missed[];
}

export type ReminderToast =
  { kind: 'reminder'; key: string; due: Due } | { kind: 'missed'; key: string; missed: Missed };

interface RemindersState {
  toasts: ReminderToast[];
  addFired: (due: Due) => void;
  addMissed: (missed: Missed) => void;
  dismiss: (key: string) => void;
}

/** Most toasts kept on screen; older ones drop off. */
const MAX_TOASTS = 4;

const keep = (toasts: ReminderToast[], t: ReminderToast) =>
  [...toasts.filter((x) => x.key !== t.key), t].slice(-MAX_TOASTS);

export const useReminders = create<RemindersState>((set) => ({
  toasts: [],
  addFired: (due) =>
    set((s) => ({
      toasts: keep(s.toasts, { kind: 'reminder', key: `${due.reminder_id}@${due.due}`, due }),
    })),
  addMissed: (missed) =>
    set((s) => ({
      toasts: keep(s.toasts, { kind: 'missed', key: `missed@${missed.since}`, missed }),
    })),
  dismiss: (key) => set((s) => ({ toasts: s.toasts.filter((t) => t.key !== key) })),
}));

/** "in 10 min · 11:30–12:15", "now · 14:50", "started 5 min ago · 09:00", "All day". */
export function reminderStatus(due: Due, now: Date): string {
  if (!due.start) return 'Reminder';
  if (due.all_day) return 'All day';
  const start = parseISO(due.start);
  const end = due.end ? parseISO(due.end) : null;
  const when =
    end && end > start
      ? `${format(start, 'HH:mm')}–${format(end, 'HH:mm')}`
      : format(start, 'HH:mm');
  const m = differenceInMinutes(start, now);
  const rel = m > 1 ? `in ${m} min` : m >= -1 ? 'now' : `started ${-m} min ago`;
  return `${rel} · ${when}${due.snoozed ? ' · snoozed' : ''}`;
}

/** "5m", "15m", "1h": a snooze option's label. */
export function snoozeLabel(minutes: number): string {
  return formatDuration(minutes);
}

/** A custom snooze: a length ("25m", "2h") or a time of day ("14:00", "2pm"). */
export type SnoozeRequest = { minutes: number } | { until: string };

/**
 * Reads a custom snooze. A time of day that has already passed means
 * tomorrow. `until` is a local `yyyy-MM-ddTHH:mm`, which the core
 * interprets in the user's zone.
 */
export function parseSnooze(input: string, now: Date): SnoozeRequest | null {
  const s = input.trim();
  if (!s) return null;
  const looksLikeTime = s.includes(':') || /\d\s*[ap]m?$/i.test(s);
  if (looksLikeTime) {
    const t = parseTime(s);
    if (!t) return null;
    let day = now;
    if (t <= format(now, 'HH:mm')) day = addDays(now, 1);
    return { until: `${format(day, 'yyyy-MM-dd')}T${t}` };
  }
  const minutes = parseDuration(s);
  return minutes && minutes > 0 && minutes <= 7 * 24 * 60 ? { minutes } : null;
}

/** "3 reminders missed while Blotter was closed". */
export function missedTitle(m: Missed): string {
  const n = m.items.length;
  const what = n === 1 ? '1 reminder' : `${n} reminders`;
  return `${what} missed while ${m.while_closed ? 'Blotter was closed' : 'your computer was asleep'}`;
}
