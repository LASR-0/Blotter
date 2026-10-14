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
