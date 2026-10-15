import { format, isToday, isYesterday, parseISO } from 'date-fns';

/** "Today 10:41", "Yesterday 07:02", "Mon 12 Oct 20:14". */
export function takenAt(iso: string): string {
  const d = parseISO(iso);
  if (isToday(d)) return `Today ${format(d, 'HH:mm')}`;
  if (isYesterday(d)) return `Yesterday ${format(d, 'HH:mm')}`;
  return format(d, 'EEE d MMM HH:mm');
}

/** "2.1 MB", "840 KB". */
export function fileSize(bytes: number | null): string {
  if (bytes === null) return '—';
  if (bytes < 1024 * 1024) return `${Math.max(1, Math.round(bytes / 1024))} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
