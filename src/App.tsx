import { useCallback, useEffect, useState, type FormEvent } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Check, Plus, Sparkle } from '@phosphor-icons/react';
import { Button } from '@/components/ui/button';
import { Kbd, KeyHint } from '@/components/ui/kbd';
import { Tooltip } from '@/components/ui/tooltip';
import { isOpError, listOps, onPlannerChanged, runOp } from '@/lib/api';
import { cn } from '@/lib/cn';
import { PRESETS } from '@/theme/presets';
import { useTheme } from '@/theme/theme';

interface DbStatus {
  path: string;
  journal_mode: string;
  schema_version: number;
}

interface InboxItem {
  id: string;
  title: string;
  status: 'open' | 'done' | 'cancelled';
}

// Temporary scaffold screen proving the UI → operation registry → SQLite
// path and the theme tokens end to end. Replaced by the app layout in #08.
export function App() {
  const [version, setVersion] = useState<string | null>(null);
  const [db, setDb] = useState<DbStatus | null>(null);
  const [opCount, setOpCount] = useState(0);
  const [inbox, setInbox] = useState<InboxItem[]>([]);
  const [draft, setDraft] = useState('');
  const [error, setError] = useState<string | null>(null);
  const { theme, setTheme, load } = useTheme();

  const report = (e: unknown) => setError(isOpError(e) ? e.message : String(e));

  const refresh = useCallback(() => {
    runOp<InboxItem[]>('list_inbox').then(setInbox).catch(report);
  }, []);

  useEffect(() => {
    invoke<string>('app_version').then(setVersion).catch(report);
    invoke<DbStatus>('db_status').then(setDb).catch(report);
    listOps()
      .then((ops) => setOpCount(ops.length))
      .catch(report);
    load().catch(report);
    refresh();
    const unlisten = onPlannerChanged(refresh);
    return () => {
      void unlisten.then((stop) => stop());
    };
  }, [refresh, load]);

  const add = (e: FormEvent) => {
    e.preventDefault();
    if (!draft.trim()) return;
    runOp('create_item', { title: draft })
      .then(() => {
        setDraft('');
        setError(null);
      })
      .catch(report);
  };

  const toggle = (item: InboxItem) => {
    runOp(item.status === 'done' ? 'uncomplete_item' : 'complete_item', { id: item.id }).catch(
      report,
    );
  };

  return (
    <main className="flex h-full flex-col justify-center gap-3 px-16">
      <h1 className="font-heading text-display leading-none font-heading-weight tracking-tight">
        Blotter
      </h1>
      <p className="text-text-muted">A keyboard-first daily planner.</p>
      <code className="font-mono text-xs text-text-muted">
        {version ? `core v${version} · ${opCount} operations` : 'core not connected'}
      </code>
      {db && (
        <code className="font-mono text-xs text-text-muted">
          db {db.path} · {db.journal_mode} · schema v{db.schema_version}
        </code>
      )}

      <div
        role="radiogroup"
        aria-label="Theme"
        className="mt-4 flex self-start rounded-md border border-border bg-surface p-0.5"
      >
        {PRESETS.map((t) => (
          <button
            key={t.id}
            role="radio"
            aria-checked={theme.id === t.id}
            onClick={() => setTheme(t.id)}
            className={cn(
              'h-6 cursor-pointer rounded-sm px-2.5 text-sm',
              theme.id === t.id
                ? 'bg-surface-raised font-medium text-text shadow-[0_0_0_1px_var(--color-border-strong)]'
                : 'text-text-muted',
            )}
          >
            {t.label}
          </button>
        ))}
      </div>

      <div className="flex items-center gap-2">
        <Button variant="primary">Save</Button>
        <Button>Cancel</Button>
        <Button variant="ghost">Skip</Button>
        <Tooltip label="Toggle AI chat" shortcut="Ctrl /">
          <Button variant="icon" size="icon" aria-label="Toggle AI chat">
            <Sparkle />
          </Button>
        </Tooltip>
        <KeyHint keys="g w" size="md" />
        <Kbd>n</Kbd>
      </div>

      <form className="mt-2 flex max-w-[420px] flex-col gap-2" onSubmit={add}>
        <label className="flex h-8 items-center gap-2 rounded-md border border-border-strong bg-surface px-2.5 focus-within:border-accent focus-within:shadow-[0_0_0_3px_color-mix(in_srgb,var(--color-accent)_22%,transparent)]">
          <Plus className="text-text-muted" />
          <input
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            placeholder="Add to Inbox and press Enter"
            aria-label="New Inbox task"
            className="flex-1 bg-transparent outline-none"
          />
        </label>
        <ul className="flex flex-col gap-1">
          {inbox.map((item) => {
            const done = item.status === 'done';
            return (
              <li key={item.id}>
                <button
                  type="button"
                  onClick={() => toggle(item)}
                  className="flex w-full cursor-pointer items-center gap-2.5 rounded-md border border-border bg-surface px-3 py-2 text-left hover:border-border-strong hover:bg-hover"
                >
                  <span
                    className={cn(
                      'grid size-4 place-items-center rounded-sm',
                      done ? 'bg-accent text-on-accent' : 'border-[1.5px] border-border-strong',
                    )}
                  >
                    {done && <Check weight="bold" size={10} />}
                  </span>
                  <span className={cn('font-medium', done && 'text-text-muted line-through')}>
                    {item.title}
                  </span>
                </button>
              </li>
            );
          })}
        </ul>
      </form>
      {error && <code className="font-mono text-xs text-danger">{error}</code>}
    </main>
  );
}
