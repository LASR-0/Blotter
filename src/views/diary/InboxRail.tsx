import { useState, type FormEvent } from 'react';
import {
  CaretDoubleLeft,
  CaretDoubleRight,
  DotsSixVertical,
  Plus,
  Tray,
} from '@phosphor-icons/react';
import { useNav } from '@/app/nav';
import { useOverlay } from '@/app/overlay';
import { Tooltip } from '@/components/ui/tooltip';
import { isOpError, runOp } from '@/lib/api';
import { cn } from '@/lib/cn';
import { formatDuration } from '@/lib/duration';
import { useOp } from '@/lib/queries';
import { itemColor } from '@/lib/tags';
import type { Item } from '@/lib/types';

/** The Diary's collapsible Inbox rail: brain-dump input and open Inbox tasks. */
export function InboxRail() {
  const open = useNav((s) => s.diaryInbox);
  const toggle = useNav((s) => s.toggleDiaryInbox);
  const inbox = useOp<Item[]>('list_inbox');
  const items = (inbox.data ?? []).filter((i) => i.status === 'open');
  const [draft, setDraft] = useState('');
  const [error, setError] = useState<string | null>(null);

  if (!open) {
    return (
      <aside className="flex w-10 flex-none flex-col items-center gap-2 border-r border-border bg-surface pt-3.5">
        <Tooltip label="Show Inbox" side="right">
          <button
            type="button"
            onClick={toggle}
            aria-label="Show Inbox"
            className="flex cursor-pointer flex-col items-center gap-1 rounded-md px-1.5 py-1 text-text-muted hover:bg-hover hover:text-text"
          >
            <Tray size={15} />
            <span className="font-mono text-xs">{items.length}</span>
            <CaretDoubleRight size={12} />
          </button>
        </Tooltip>
      </aside>
    );
  }

  const add = (e: FormEvent) => {
    e.preventDefault();
    const title = draft.trim();
    if (!title) return;
    runOp('create_item', { title })
      .then(() => {
        setDraft('');
        setError(null);
      })
      .catch((err: unknown) => setError(isOpError(err) ? err.message : String(err)));
  };

  return (
    <aside className="flex min-h-0 w-[232px] flex-none flex-col border-r border-border bg-surface">
      <div className="flex items-center gap-2 px-3.5 pt-3.5 pb-2.5">
        <Tray size={15} className="text-text-muted" />
        <span className="font-semibold">Inbox</span>
        <span className="font-mono text-xs text-text-muted">{items.length}</span>
        <span className="flex-1" />
        <Tooltip label="Hide Inbox">
          <button
            type="button"
            onClick={toggle}
            aria-label="Hide Inbox"
            className="grid size-6 cursor-pointer place-items-center rounded-sm text-text-muted hover:bg-hover hover:text-text"
          >
            <CaretDoubleLeft />
          </button>
        </Tooltip>
      </div>
      <form onSubmit={add} className="px-2.5 pb-2.5">
        <label className="flex h-[30px] items-center gap-2 rounded-md border border-border bg-bg px-2 text-text-muted focus-within:border-accent">
          <Plus />
          <input
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            placeholder="Brain-dump a task…"
            aria-label="New Inbox task"
            className="min-w-0 flex-1 bg-transparent text-text outline-none"
          />
        </label>
        {error && <p className="m-0 pt-1.5 font-mono text-xs text-danger">{error}</p>}
      </form>
      <div className="flex min-h-0 flex-1 flex-col gap-1 overflow-auto px-2.5 pb-2.5">
        {items.map((item) => (
          <div
            key={item.id}
            title={item.title}
            onDoubleClick={() => useOverlay.getState().openItem({ id: item.id })}
            className="flex h-8 flex-none items-center gap-2 rounded-md border border-border bg-bg px-2"
          >
            <DotsSixVertical size={13} className="text-text-muted opacity-70" />
            <span
              className="size-[7px] flex-none rounded-full"
              style={{ background: itemColor(item) }}
            />
            <span className="min-w-0 flex-1 truncate">{item.title}</span>
            {item.duration_min && (
              <span className="font-mono text-xs text-text-muted">
                {formatDuration(item.duration_min)}
              </span>
            )}
          </div>
        ))}
        {inbox.isSuccess && items.length === 0 && (
          <p className={cn('m-0 px-1 pt-2 text-sm leading-normal text-text-muted')}>
            Inbox zero. Brain-dump new tasks above.
          </p>
        )}
      </div>
    </aside>
  );
}
