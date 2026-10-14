import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface DbStatus {
  path: string;
  journal_mode: string;
  schema_version: number;
}

export function App() {
  const [version, setVersion] = useState<string | null>(null);
  const [db, setDb] = useState<DbStatus | string | null>(null);

  useEffect(() => {
    invoke<string>('app_version')
      .then(setVersion)
      .catch(() => setVersion(null));
    invoke<DbStatus>('db_status')
      .then(setDb)
      .catch((e: unknown) => setDb(String(e)));
  }, []);

  return (
    <main className="scaffold">
      <h1>Blotter</h1>
      <p>A keyboard-first daily planner.</p>
      <code>{version ? `core v${version}` : 'core not connected'}</code>
      {typeof db === 'string' && <code className="error">database error: {db}</code>}
      {db && typeof db === 'object' && (
        <code>
          db {db.path} · {db.journal_mode} · schema v{db.schema_version}
        </code>
      )}
    </main>
  );
}
