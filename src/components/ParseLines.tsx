import { useMemo } from 'react';
import { CalendarCheck, Notebook, Tray } from '@phosphor-icons/react';
import { cn } from '@/lib/cn';
import { describeParse, destination, parseQuick } from '@/lib/quickParse';
import { tagColor } from '@/lib/tags';

/** Lines shown before the rest are summed up as "+N more". */
const MAX_LINES = 6;

const WHERE = {
  inbox: { label: 'Inbox', icon: Tray },
  day: { label: 'Diary', icon: Notebook },
  calendar: { label: 'Calendar', icon: CalendarCheck },
} as const;

/**
 * What each line of a brain-dump will create, read with the quick-add
 * grammar: its title, kind (an event when it reads like one), when, and
 * where it lands.
 */
export function ParseLines({ text, className }: { text: string; className?: string }) {
  // Parsed once per edit; the clock only matters for relative dates.
  const parsed = useMemo(() => {
    const now = new Date();
    return text
      .split('\n')
      .map((l) => l.trim())
      .filter(Boolean)
      .map((l) => parseQuick(l, now));
  }, [text]);
  if (!parsed.length) return null;

  return (
    <ul aria-label="Will add" className={cn('m-0 flex list-none flex-col gap-1 p-0', className)}>
      {parsed.slice(0, MAX_LINES).map((p, i) => {
        const where = WHERE[destination(p)];
        return (
          <li key={i} className="flex min-w-0 items-center gap-2 text-sm">
            <span
              className="h-3.5 w-[3px] flex-none rounded-[2px]"
              style={{ background: p.tags[0] ? tagColor(p.tags[0]) : 'var(--color-accent)' }}
            />
            <span className="min-w-0 truncate font-medium">{p.title || 'Untitled'}</span>
            <span className="min-w-0 flex-1 truncate font-mono text-xs text-text-muted">
              {describeParse(p)}
              {p.tags.length > 0 && ` · ${p.tags.map((t) => `#${t}`).join(' ')}`}
            </span>
            <span className="flex flex-none items-center gap-1 text-xs text-text-muted">
              <where.icon />
              {where.label}
            </span>
          </li>
        );
      })}
      {parsed.length > MAX_LINES && (
        <li className="text-xs text-text-muted">+{parsed.length - MAX_LINES} more</li>
      )}
    </ul>
  );
}
