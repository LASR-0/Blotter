import { format, parseISO } from 'date-fns';
import { formatDuration } from './duration';
import type { Item } from './types';

/** Local `HH:mm` for an RFC 3339 instant. */
export function timeOf(iso: string): string {
  return format(parseISO(iso), 'HH:mm');
}

/** Card meta line, e.g. `17:30 · 45m · health` or `~20m` for unscheduled. */
export function itemMeta(item: Item): string {
  const parts: string[] = [];
  if (item.all_day) parts.push('all day');
  else if (item.start_at) parts.push(timeOf(item.start_at));
  if (item.duration_min)
    parts.push(
      item.start_at ? formatDuration(item.duration_min) : `~${formatDuration(item.duration_min)}`,
    );
  if (item.anchoring === 'fixed' && item.kind === 'task') parts.push('fixed');
  parts.push(...item.tags);
  return parts.join(' · ');
}

/**
 * The time part of a card's meta line, as in the design:
 * `07:15 · 30m` (flexible), `09:00–09:15` (fixed or event), `all day`,
 * `~30m` (unscheduled), or `due Mon 12 · 20m` when overdue.
 */
export function cardTime(item: Item, overdue = false): string {
  const dur = item.duration_min ? formatDuration(item.duration_min) : '';
  if (overdue) {
    const when = item.end_at ?? item.due_at ?? item.end_date ?? item.start_date;
    const day = when ? format(parseISO(when), 'EEE d') : '';
    return [day && `due ${day}`, dur].filter(Boolean).join(' · ');
  }
  if (item.all_day) return 'all day';
  if (item.start_at) {
    const fixedRange = item.anchoring === 'fixed' || item.kind === 'event';
    if (fixedRange && item.end_at) return `${timeOf(item.start_at)}–${timeOf(item.end_at)}`;
    return [timeOf(item.start_at), dur].filter(Boolean).join(' · ');
  }
  return dur ? `~${dur}` : '';
}

/**
 * Reads a typed time of day into `HH:MM` (24-hour): `17:30`, `1730`,
 * `9:05`, `930`, `5pm`, `5:30 PM`, `12am`. Null if it isn't a valid time.
 */
export function parseTime(input: string): string | null {
  const m = /^(\d{1,2})(?::?(\d{2}))?\s*(am|pm|a|p)?$/.exec(input.trim().toLowerCase());
  if (!m) return null;
  let h = Number(m[1]);
  const min = m[2] === undefined ? 0 : Number(m[2]);
  const meridiem = m[3]?.[0];
  if (min > 59) return null;
  if (meridiem) {
    if (h < 1 || h > 12) return null;
    h = (h % 12) + (meridiem === 'p' ? 12 : 0);
  } else if (h > 23) {
    return null;
  }
  return `${String(h).padStart(2, '0')}:${String(min).padStart(2, '0')}`;
}

/** How long ago something was added: `now`, `today`, `yesterday`, `3d ago`, or `12 Oct`. */
export function addedAgo(iso: string, now: Date = new Date()): string {
  const then = parseISO(iso);
  if (now.getTime() - then.getTime() < 60_000) return 'now';
  const days = Math.round(
    (new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime() -
      new Date(then.getFullYear(), then.getMonth(), then.getDate()).getTime()) /
      86_400_000,
  );
  if (days <= 0) return 'today';
  if (days === 1) return 'yesterday';
  if (days < 7) return `${days}d ago`;
  return format(then, 'd MMM');
}
