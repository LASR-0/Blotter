import { useLayoutEffect, useRef, useState, type KeyboardEvent } from 'react';
import { format, parseISO } from 'date-fns';
import { Check, Tray } from '@phosphor-icons/react';
import { useDraft } from '@/app/drafts';
import { startPointerDrag, useDrag } from '@/app/drag';
import { useFocus, useScrollFocusIntoView, useViewController } from '@/app/focus';
import { useOverlay } from '@/app/overlay';
import { useSelection } from '@/app/selection';
import { EmptyState, ErrorPanel } from '@/components/EmptyState';
import { Kbd } from '@/components/ui/kbd';
import { isOpError, runBatch } from '@/lib/api';
import { cn } from '@/lib/cn';
import { formatDuration } from '@/lib/duration';
import { moveInColumns } from '@/lib/focusNav';
import { addedAgo } from '@/lib/format';
import { parseQuick, toCreateArgs } from '@/lib/quickParse';
import { useOp } from '@/lib/queries';
import { tagColor } from '@/lib/tags';
import type { Item } from '@/lib/types';
import { BulkBar } from './BulkBar';

const COLUMNS = 'grid grid-cols-[28px_minmax(180px,1fr)_64px_40px_72px_76px] items-center gap-2.5';

const PRIORITY_COLOR = ['text-text-muted', 'text-text-muted', 'text-warning', 'text-danger'];

function InboxRow({ item }: { item: Item }) {
  const selected = useSelection((s) => s.ids.has(item.id));
  const toggle = useSelection((s) => s.toggle);
  const focused = useFocus((s) => s.itemId === item.id);
  const dragged = useDrag((s) => s.item?.id === item.id);
  const done = item.status === 'done';
  return (
    <div
      data-focus-id={item.id}
      aria-selected={selected}
      onClick={() => {
        useFocus.getState().setItem(item.id);
        toggle(item.id);
      }}
      onDoubleClick={() => useOverlay.getState().openItem({ id: item.id })}
      onPointerDown={(e) => startPointerDrag(e, item)}
      className={cn(
        COLUMNS,
        'h-[38px] flex-none cursor-default rounded-md px-2.5 select-none hover:bg-hover',
        selected && 'bg-accent-tint hover:bg-accent-tint',
        focused && 'outline-2 -outline-offset-2 outline-focus-ring',
        dragged && 'opacity-45',
      )}
    >
      <span
        aria-hidden
        className={cn(
          'grid size-4 place-items-center rounded-sm text-on-accent',
          selected ? 'border border-accent bg-accent' : 'border-[1.5px] border-border-strong',
        )}
      >
        {selected && <Check weight="bold" size={10} />}
      </span>
      <span className="flex min-w-0 items-center gap-2.5">
        <span className={cn('truncate font-medium', done && 'text-text-muted line-through')}>
          {item.title}
        </span>
        {item.tags.map((t) => (
          <span key={t} className="flex flex-none items-center gap-1 text-xs text-text-muted">
            <span className="size-1.5 rounded-full" style={{ background: tagColor(t) }} />
            {t}
          </span>
        ))}
      </span>
      <span className="font-mono text-sm text-text-muted">
        {item.duration_min ? formatDuration(item.duration_min) : '—'}
      </span>
      <span className={cn('font-mono text-sm', PRIORITY_COLOR[item.priority])}>
        {item.priority ? `!${item.priority}` : '—'}
      </span>
      <span className={cn('font-mono text-sm', item.due_at ? 'text-text' : 'text-text-muted')}>
        {item.due_at ? format(parseISO(item.due_at), 'EEE d') : '—'}
      </span>
      <span className="font-mono text-sm text-text-muted">{addedAgo(item.created_at)}</span>
    </div>
  );
}

/** The full Inbox (design V3): brain-dump box, task table and bulk actions. */
export function InboxView() {
  const inbox = useOp<Item[]>('list_inbox');
  const items = (inbox.data ?? []).filter((i) => i.status !== 'done');
  const [draft, setDraft] = useDraft('inbox');
  // The box starts at two lines and grows a line at a time with the text (up to half the window).
  const box = useRef<HTMLTextAreaElement>(null);
  useLayoutEffect(() => {
    const el = box.current;
    if (!el) return;
    el.style.height = 'auto';
    el.style.height = `${el.scrollHeight}px`;
  }, [draft]);
  const [error, setError] = useState<string | null>(null);

  const focusedId = useFocus((s) => s.itemId);
  useScrollFocusIntoView(focusedId);
  useViewController({
    move: (dir) =>
      useFocus.getState().setItem(moveInColumns([items.map((i) => i.id)], focusedId, dir)),
  });

  // Enter adds every line as its own item, in one batch; Shift+Enter is a
  // newline. Each line takes the quick-add grammar, so "~45m #home" works here too.
  const onKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key !== 'Enter' || e.shiftKey) return;
    e.preventDefault();
    const lines = draft
      .split('\n')
      .map((l) => l.trim())
      .filter(Boolean);
    if (!lines.length) return;
    const now = new Date();
    runBatch(lines.map((l) => ({ name: 'create_item', args: toCreateArgs(parseQuick(l, now)) })))
      .then(() => {
        setDraft('');
        setError(null);
      })
      // The text stays in the box so nothing is lost.
      .catch((err: unknown) => setError(isOpError(err) ? err.message : String(err)));
  };

  return (
    <div className="relative flex h-full min-h-0 flex-col">
      <div className="mx-auto flex w-full max-w-[860px] flex-col gap-3.5 px-7 pt-5">
        <div className="flex items-baseline gap-2.5">
          <h1 className="m-0 font-heading text-xl font-heading-weight">Inbox</h1>
          <span className="font-mono text-sm text-text-muted">{items.length} unscheduled</span>
        </div>
        <div className="flex flex-col rounded-md border border-border-strong bg-surface focus-within:border-accent">
          <textarea
            ref={box}
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            onKeyDown={onKeyDown}
            rows={2}
            placeholder="Brain-dump, one task per line…"
            aria-label="Brain-dump tasks, one per line"
            className="max-h-[50vh] resize-none overflow-y-auto bg-transparent px-3 py-2.5 text-md text-text outline-none"
          />
          <div className="flex items-center gap-3 border-t border-border px-2.5 py-1.5 text-xs text-text-muted">
            <span className="font-mono">~45m</span>
            <span className="font-mono">#tag</span>
            <span className="font-mono">!1–!3</span>
            <span className="font-mono">due fri</span>
            <span className="flex-1" />
            <span className="flex items-center gap-1.5">
              <Kbd>↵</Kbd>add lines
            </span>
            <span className="flex items-center gap-1.5">
              <Kbd>Shift ↵</Kbd>new line
            </span>
          </div>
        </div>
        {error && <code className="font-mono text-xs text-danger">{error}</code>}
        <div className={cn(COLUMNS, 'px-2.5 text-xs tracking-[.05em] text-text-muted uppercase')}>
          <span />
          <span>Task</span>
          <span>Duration</span>
          <span>Pri</span>
          <span>Due</span>
          <span>Added</span>
        </div>
      </div>

      <div className="mx-auto flex min-h-0 w-full max-w-[860px] flex-1 flex-col gap-0.5 overflow-auto px-7 pt-1 pb-[90px]">
        {inbox.isPending &&
          [1, 0.7, 0.45].map((o) => (
            <div
              key={o}
              style={{ opacity: o }}
              className="h-[34px] flex-none rounded-md bg-surface"
            />
          ))}
        {inbox.isError && (
          <ErrorPanel
            title="Couldn't load the Inbox"
            message={isOpError(inbox.error) ? inbox.error.message : String(inbox.error)}
            onRetry={() => void inbox.refetch()}
          />
        )}
        {inbox.isSuccess && items.length === 0 && (
          <EmptyState icon={Tray} title="Inbox zero" className="min-h-[220px] flex-none">
            Everything has a time. Brain-dump new tasks above, or press <Kbd>i</Kbd> from anywhere
            to come back here.
          </EmptyState>
        )}
        {items.map((item) => (
          <InboxRow key={item.id} item={item} />
        ))}
      </div>
      <BulkBar />
    </div>
  );
}
