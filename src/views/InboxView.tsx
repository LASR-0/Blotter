import { useState, type FormEvent } from 'react';
import { Plus, Tray } from '@phosphor-icons/react';
import { useFocus, useScrollFocusIntoView, useViewController } from '@/app/focus';
import { EmptyState, ErrorPanel } from '@/components/EmptyState';
import { ItemCard } from '@/components/ItemCard';
import { Kbd } from '@/components/ui/kbd';
import { isOpError, runOp } from '@/lib/api';
import { moveInColumns } from '@/lib/focusNav';
import { useOp } from '@/lib/queries';
import type { Item } from '@/lib/types';

// Brain-dump input and the Inbox list. Commit #16 adds multi-select, the
// bulk bar and the side panel; #17 adds natural-language parsing.
export function InboxView() {
  const inbox = useOp<Item[]>('list_inbox');
  const [draft, setDraft] = useState('');
  const [error, setError] = useState<string | null>(null);
  const focusedId = useFocus((s) => s.itemId);
  const ids = (inbox.data ?? []).map((i) => i.id);
  useScrollFocusIntoView(focusedId);
  useViewController({
    move: (dir) => useFocus.getState().setItem(moveInColumns([ids], focusedId, dir)),
  });

  const add = (e: FormEvent) => {
    e.preventDefault();
    const title = draft.trim();
    if (!title) return;
    runOp('create_item', { title })
      .then(() => {
        setDraft('');
        setError(null);
      })
      // The text stays in the input so nothing is lost.
      .catch((err: unknown) => setError(isOpError(err) ? err.message : String(err)));
  };

  const items = inbox.data ?? [];

  return (
    <div className="h-full overflow-auto">
      <div className="flex max-w-[640px] flex-col gap-3 px-8 py-6">
        <form onSubmit={add}>
          <label className="flex h-9 items-center gap-2 rounded-md border border-border-strong bg-surface px-2.5 focus-within:border-accent focus-within:shadow-[0_0_0_3px_color-mix(in_srgb,var(--color-accent)_22%,transparent)]">
            <Plus className="text-text-muted" />
            <input
              value={draft}
              onChange={(e) => setDraft(e.target.value)}
              placeholder="Brain-dump a task and press Enter"
              aria-label="New Inbox task"
              className="flex-1 bg-transparent outline-none"
            />
          </label>
        </form>
        {error && <code className="font-mono text-xs text-danger">{error}</code>}

        {inbox.isPending &&
          [1, 0.7, 0.45].map((o) => (
            <div key={o} style={{ opacity: o }} className="h-[34px] rounded-md bg-surface" />
          ))}
        {inbox.isError && (
          <ErrorPanel
            title="Couldn't load the Inbox"
            message={isOpError(inbox.error) ? inbox.error.message : String(inbox.error)}
            onRetry={() => void inbox.refetch()}
          />
        )}
        {inbox.isSuccess && items.length === 0 && (
          <EmptyState icon={Tray} title="Inbox zero" className="min-h-[220px]">
            Everything has a time. Brain-dump new tasks above, or press <Kbd>i</Kbd> from anywhere
            to come back here.
          </EmptyState>
        )}
        <div className="flex flex-col gap-2">
          {items.map((item) => (
            <ItemCard key={item.id} item={item} onError={setError} />
          ))}
        </div>
      </div>
    </div>
  );
}
