import { useEffect, useState, type ReactNode } from 'react';
import { Command, useCommandState } from 'cmdk';
import { Dialog } from 'radix-ui';
import { CalendarBlank, CheckSquare, MagnifyingGlass, Tray } from '@phosphor-icons/react';
import { ACTION_GROUPS, type Action } from '@/actions/actions';
import { Kbd, KeyChips } from '@/components/ui/kbd';
import { itemMeta } from '@/lib/format';
import { useOp } from '@/lib/queries';
import type { Item } from '@/lib/types';
import { useOverlay } from './overlay';

/** Waits this long after the last keystroke before searching items. */
const SEARCH_DEBOUNCE_MS = 150;

function useDebounced<T>(value: T, ms: number): T {
  const [debounced, setDebounced] = useState(value);
  useEffect(() => {
    const t = window.setTimeout(() => setDebounced(value), ms);
    return () => window.clearTimeout(t);
  }, [value, ms]);
  return debounced;
}

const ROW =
  'flex h-[34px] cursor-pointer items-center gap-2.5 rounded-md px-2.5 data-[selected=true]:bg-accent-tint data-[selected=true]:shadow-[inset_2px_0_0_var(--color-accent)]';

function Row({
  value,
  keywords,
  icon,
  label,
  hint,
  binding,
  onSelect,
}: {
  value: string;
  keywords?: string[];
  icon: ReactNode;
  label: string;
  hint?: string;
  binding?: string;
  onSelect: () => void;
}) {
  return (
    <Command.Item value={value} keywords={keywords} onSelect={onSelect} className={ROW}>
      <span className="text-[15px] text-text-muted">{icon}</span>
      <span className="min-w-0 flex-1 truncate">
        {label}
        {hint && <span className="ml-2 text-sm text-text-muted">{hint}</span>}
      </span>
      {binding && <KeyChips binding={binding} />}
    </Command.Item>
  );
}

function FooterCount() {
  const count = useCommandState((s) => s.filtered.count);
  return <span>{count} results</span>;
}

export function CommandPalette({ actions }: { actions: readonly Action[] }) {
  const open = useOverlay((s) => s.overlay === 'palette');
  const close = useOverlay((s) => s.close);
  return (
    <Dialog.Root open={open} onOpenChange={(o) => !o && close()}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-40 bg-[color-mix(in_srgb,var(--color-bg)_62%,transparent)]" />
        <Dialog.Content
          aria-describedby={undefined}
          className="fixed top-24 left-1/2 z-41 w-[620px] max-w-[calc(100vw-32px)] -translate-x-1/2 overflow-hidden rounded-lg bg-surface-raised shadow-overlay focus:outline-none"
        >
          <Dialog.Title className="sr-only">Command palette</Dialog.Title>
          {/* Content unmounts on close, so each opening starts with an empty query. */}
          <PaletteBody actions={actions} close={close} />
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

function PaletteBody({ actions, close }: { actions: readonly Action[]; close: () => void }) {
  const [query, setQuery] = useState('');
  const term = useDebounced(query.trim(), SEARCH_DEBOUNCE_MS);
  const search = useOp<Item[]>(
    'search_items',
    { query: term, limit: 8 },
    { enabled: term.length >= 2 },
  );

  const run = (fn: () => void) => {
    close();
    fn();
  };

  const groups = ACTION_GROUPS.map((g) => ({
    name: g,
    actions: actions.filter((a) => a.group === g),
  })).filter((g) => g.actions.length > 0);
  const items = term.length >= 2 ? (search.data ?? []) : [];

  return (
    <Command loop label="Command palette" className="flex flex-col">
      <div className="flex h-12 items-center gap-2.5 border-b border-border px-3.5">
        <MagnifyingGlass size={16} className="text-text-muted" />
        <Command.Input
          value={query}
          onValueChange={setQuery}
          placeholder="Type a command or search items…"
          className="flex-1 bg-transparent text-lg text-text outline-none"
        />
        <Kbd>Esc</Kbd>
      </div>
      <Command.List className="max-h-[420px] overflow-auto p-1.5 [&_[cmdk-group-heading]]:px-2.5 [&_[cmdk-group-heading]]:pt-2 [&_[cmdk-group-heading]]:pb-1 [&_[cmdk-group-heading]]:text-xs [&_[cmdk-group-heading]]:text-text-muted">
        <Command.Empty className="px-2.5 py-6 text-center text-text-muted">
          {search.isFetching ? 'Searching…' : `Nothing matches “${query}”.`}
        </Command.Empty>
        {groups.map((g) => (
          <Command.Group key={g.name} heading={g.name}>
            {g.actions.map((a) => (
              <Row
                key={a.id}
                value={a.label}
                keywords={a.keywords}
                icon={<a.icon />}
                label={a.label}
                hint={a.hint}
                binding={a.binding}
                onSelect={() => run(a.run)}
              />
            ))}
          </Command.Group>
        ))}
        {items.length > 0 && (
          <Command.Group heading="Items">
            {items.map((item) => (
              <Row
                key={item.id}
                value={`item ${item.id} ${item.title}`}
                // Matched by the backend; keep cmdk's filter from hiding them.
                keywords={[term]}
                icon={
                  item.status === 'done' ? (
                    <CheckSquare />
                  ) : item.start_at || item.start_date ? (
                    <CalendarBlank />
                  ) : (
                    <Tray />
                  )
                }
                label={item.title}
                hint={itemMeta(item) || (item.start_at || item.start_date ? '' : 'Inbox')}
                onSelect={() => run(() => useOverlay.getState().openItem({ id: item.id }))}
              />
            ))}
          </Command.Group>
        )}
      </Command.List>
      <div className="flex items-center gap-3.5 border-t border-border px-3.5 py-2 text-xs text-text-muted">
        <span className="flex items-center gap-1.5">
          <Kbd>↑↓</Kbd>navigate
        </span>
        <span className="flex items-center gap-1.5">
          <Kbd>↵</Kbd>run
        </span>
        <span className="flex-1" />
        <FooterCount />
      </div>
    </Command>
  );
}
