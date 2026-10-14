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
