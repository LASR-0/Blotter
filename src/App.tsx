import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

export function App() {
  const [version, setVersion] = useState<string | null>(null);

  useEffect(() => {
    invoke<string>('app_version')
      .then(setVersion)
      .catch(() => setVersion(null));
  }, []);

  return (
    <main className="scaffold">
      <h1>Blotter</h1>
      <p>A keyboard-first daily planner.</p>
      <code>{version ? `core v${version}` : 'core not connected'}</code>
    </main>
  );
}
