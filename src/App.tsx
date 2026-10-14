import { useCallback, useEffect, useState, type FormEvent } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { isOpError, listOps, onPlannerChanged, runOp } from './lib/api';

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
// path end to end. Replaced by the real app layout in commit #08.
export function App() {
  const [version, setVersion] = useState<string | null>(null);
  const [db, setDb] = useState<DbStatus | null>(null);
  const [opCount, setOpCount] = useState(0);
  const [inbox, setInbox] = useState<InboxItem[]>([]);
  const [draft, setDraft] = useState('');
  const [error, setError] = useState<string | null>(null);

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
    refresh();
    const unlisten = onPlannerChanged(refresh);
    return () => {
      void unlisten.then((stop) => stop());
    };
  }, [refresh]);

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
    <main className="scaffold">
      <h1>Blotter</h1>
      <p>A keyboard-first daily planner.</p>
      <code>{version ? `core v${version} · ${opCount} operations` : 'core not connected'}</code>
      {db && (
        <code>
          db {db.path} · {db.journal_mode} · schema v{db.schema_version}
        </code>
      )}
      <form className="scaffold-inbox" onSubmit={add}>
        <input
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          placeholder="Add to Inbox and press Enter"
          aria-label="New Inbox task"
        />
        <ul>
          {inbox.map((item) => (
            <li key={item.id}>
              <label>
                <input
                  type="checkbox"
                  checked={item.status === 'done'}
                  onChange={() => toggle(item)}
                />
                <span className={item.status === 'done' ? 'done' : undefined}>{item.title}</span>
              </label>
            </li>
          ))}
        </ul>
      </form>
      {error && <code className="error">{error}</code>}
    </main>
  );
}
