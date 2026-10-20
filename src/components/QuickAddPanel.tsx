import { useMemo, useState, type KeyboardEvent } from 'react';
import {
  ArrowsClockwise,
  CalendarBlank,
  CalendarCheck,
  ChatText,
  Clock,
  Flag,
  FlagCheckered,
  Hash,
  Lightning,
  Notebook,
  PlusCircle,
  PushPin,
  Timer,
  Tray,
  type Icon,
} from '@phosphor-icons/react';
import { Kbd } from '@/components/ui/kbd';
import { isOpError, runOp } from '@/lib/api';
import { cn } from '@/lib/cn';
import {
  describeParse,
  destination,
  parseQuick,
  toCreateArgs,
  type ChipKind,
} from '@/lib/quickParse';
import { tagColor } from '@/lib/tags';
import type { Item } from '@/lib/types';

const CHIP_ICONS: Record<ChipKind, Icon> = {
  date: CalendarBlank,
  time: Clock,
  duration: Timer,
  priority: Flag,
  tag: Hash,
  anchoring: PushPin,
  repeat: ArrowsClockwise,
  due: FlagCheckered,
  context: ChatText,
};

const DESTINATIONS = {
  inbox: { label: 'Inbox', icon: Tray },
  day: { label: 'Diary', icon: Notebook },
  calendar: { label: 'Calendar', icon: CalendarCheck },
} as const;

/**
 * The quick-add bar (design V8 / W6): type a line, see what it means, and
 * create it. `capture` mode is the quick-capture popup's variant.
 */
export function QuickAddPanel({
  mode = 'bar',
  initialText = '',
  onClose,
  onCreated,
}: {
  mode?: 'bar' | 'capture';
  initialText?: string;
  onClose: () => void;
  /** Called with the new item; `open` when Ctrl+Enter asked for details. */
  onCreated?: (item: Item, open: boolean) => void;
}) {
  const [text, setText] = useState(initialText);
  const [status, setStatus] = useState<{ ok: boolean; text: string } | null>(null);
  // Re-parsed per keystroke; relative dates use the time the line was typed.
  const parsed = useMemo(() => parseQuick(text, new Date()), [text]);
  const dest = DESTINATIONS[destination(parsed)];
  const color = parsed.tags[0] ? tagColor(parsed.tags[0]) : 'var(--color-accent)';

  const create = async (keepOpen: boolean, open: boolean) => {
    if (!text.trim()) return;
    try {
      const item = await runOp<Item>('create_item', toCreateArgs(parsed));
      setStatus({ ok: true, text: `Created "${item.title}"` });
      if (keepOpen) setText('');
      else onCreated?.(item, open);
    } catch (e) {
      // The text stays so nothing is lost.
      setStatus({ ok: false, text: isOpError(e) ? e.message : String(e) });
    }
  };

  const onKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    } else if (e.key === 'Enter') {
      e.preventDefault();
      void create(e.altKey, e.ctrlKey || e.metaKey);
    }
  };

  return (
    <div
      className={cn(
        'overflow-hidden bg-surface-raised text-base text-text',
        // In capture mode the window itself is the popup.
        mode === 'bar' && 'rounded-lg shadow-overlay',
      )}
    >
      {mode === 'capture' && (
        <div className="flex items-center gap-2 border-b border-border px-3.5 py-2 text-xs text-text-muted">
          <Lightning weight="fill" className="text-accent-text" />
          Quick capture
          <span className="flex-1" />
          writes straight to Blotter
        </div>
      )}
      <div className="flex items-center gap-2.5 px-3.5 py-3">
        <PlusCircle size={18} className="text-accent-text" />
        <input
          autoFocus
          value={text}
          onChange={(e) => {
            setText(e.target.value);
            setStatus(null);
          }}
          onKeyDown={onKeyDown}
          placeholder="Add a task…  e.g. dentist tue 3pm ~45m @fixed"
          aria-label="Quick add"
          className="h-7 min-w-0 flex-1 bg-transparent text-lg text-text outline-none"
        />
        <Kbd>Esc</Kbd>
      </div>
      <div className="flex min-h-[26px] flex-wrap gap-1.5 px-3.5 pb-3" aria-live="polite">
        {parsed.chips.length === 0 ? (
          <span className="text-sm text-text-muted">
            Type a date, time, ~duration, #tag, !priority, @fixed, "due …", "every …" or // context
          </span>
        ) : (
          parsed.chips.map((c, i) => {
            const ChipIcon = CHIP_ICONS[c.kind];
            return (
              <span
                key={`${c.kind}-${i}`}
                className="flex h-6 items-center gap-1.5 rounded-full border border-[color-mix(in_srgb,var(--color-accent)_30%,transparent)] bg-accent-tint px-2 text-sm"
              >
                <ChipIcon className="text-accent-text" />
                <span className="text-xs text-text-muted">{c.label}</span>
                <span className="font-mono text-[11.5px]">{c.value}</span>
                {c.source && c.source !== c.value && (
                  <span className="font-mono text-[10px] text-text-muted opacity-80">
                    {c.source}
                  </span>
                )}
              </span>
            );
          })
        )}
      </div>
      <div className="flex items-center gap-3 border-t border-border bg-surface px-3.5 py-2.5">
        <span className="w-1 self-stretch rounded-[2px]" style={{ background: color }} />
        <div className="flex min-w-0 flex-1 flex-col gap-[3px]">
          <span className="truncate font-medium">{parsed.title || 'Untitled'}</span>
          <span className="font-mono text-xs text-text-muted">{describeParse(parsed)}</span>
        </div>
        <span className="flex items-center gap-[5px] rounded-full border border-border-strong px-2 py-0.5 text-xs whitespace-nowrap text-text-muted">
          <dest.icon />
          {dest.label}
        </span>
      </div>
      {status && !status.ok && (
        // Errors can be long (e.g. a block-out refusal), so they get a row of their own.
        <div role="alert" className="border-t border-border px-3.5 py-2 text-sm text-danger">
          {status.text}
        </div>
      )}
      <div className="flex items-center gap-3.5 border-t border-border px-3.5 py-2 text-xs whitespace-nowrap text-text-muted">
        <span className="flex items-center gap-[5px]">
          <Kbd>↵</Kbd>Create
        </span>
        {mode === 'bar' && (
          <span className="flex items-center gap-[5px]">
            <Kbd>Ctrl ↵</Kbd>Create &amp; open details
          </span>
        )}
        <span className="flex items-center gap-[5px]">
          <Kbd>Alt ↵</Kbd>Keep open
        </span>
        <span className="flex-1" />
        {status?.ok && (
          <span role="status" className="min-w-0 truncate">
            {status.text}
          </span>
        )}
      </div>
    </div>
  );
}
