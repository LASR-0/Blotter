import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Bell,
  CalendarDots,
  ClockCounterClockwise,
  Info,
  Keyboard,
  Palette,
  Prohibit,
  Sparkle,
  type Icon,
} from '@phosphor-icons/react';
import { useNav, type SettingsPage } from '@/app/nav';
import { BlotterMark } from '@/components/BlotterMark';
import { cn } from '@/lib/cn';

import { AiProvidersPage } from './settings/AiProvidersPage';
import { ThemePage } from './settings/ThemePage';
import { BackupsPage } from './settings/BackupsPage';
import { BlockoutsPage } from './settings/BlockoutsPage';
import { CalendarsPage } from './settings/CalendarsPage';
import { KeybindingsPage } from './settings/KeybindingsPage';
import { NotificationsPage } from './settings/NotificationsPage';

// Pages appear here as their phases land: General, Block-outs, Calendars,
// AI providers, Notifications, Keybindings and Backups (design S1–S9).
const PAGES: readonly { id: SettingsPage; label: string; icon: Icon }[] = [
  { id: 'blockouts', label: 'Block-outs', icon: Prohibit },
  { id: 'calendars', label: 'Calendars (ICS)', icon: CalendarDots },
  { id: 'notifications', label: 'Notifications', icon: Bell },
  { id: 'ai', label: 'AI providers', icon: Sparkle },
  { id: 'keybindings', label: 'Keybindings', icon: Keyboard },
  { id: 'theme', label: 'Theme', icon: Palette },
  { id: 'backups', label: 'Backups', icon: ClockCounterClockwise },
  { id: 'about', label: 'About', icon: Info },
];

export function SettingsView() {
  const page = useNav((s) => s.settingsPage);
  const openSettings = useNav((s) => s.openSettings);
  return (
    <div className="flex h-full min-h-0">
      <nav
        aria-label="Settings pages"
        className="flex w-[200px] flex-none flex-col gap-0.5 border-r border-border px-2.5 py-4"
      >
        <span className="px-2 pb-2 text-xs tracking-[.06em] text-text-muted uppercase">
          Settings
        </span>
        {PAGES.map((p) => (
          <button
            key={p.id}
            type="button"
            onClick={() => openSettings(p.id)}
            aria-current={page === p.id ? 'page' : undefined}
            className={cn(
              'flex h-[30px] cursor-pointer items-center gap-2.5 rounded-md px-2 text-left hover:bg-hover',
              page === p.id && 'bg-hover font-medium',
            )}
          >
            <p.icon size={15} className="text-text-muted" />
            {p.label}
          </button>
        ))}
      </nav>
      <main className="min-w-0 flex-1 overflow-auto">
        {page === 'about' ? (
          <AboutPage />
        ) : page === 'ai' ? (
          <AiProvidersPage />
        ) : page === 'backups' ? (
          <BackupsPage />
        ) : page === 'notifications' ? (
          <NotificationsPage />
        ) : page === 'blockouts' ? (
          <BlockoutsPage />
        ) : page === 'keybindings' ? (
          <KeybindingsPage />
        ) : page === 'calendars' ? (
          <CalendarsPage />
        ) : (
          <ThemePage />
        )}
      </main>
    </div>
  );
}

interface DbStatus {
  path: string;
  journal_mode: string;
  schema_version: number;
}

function AboutPage() {
  const [version, setVersion] = useState('');
  const [db, setDb] = useState<DbStatus | null>(null);
  useEffect(() => {
    invoke<string>('app_version')
      .then(setVersion)
      .catch(() => setVersion('?'));
    invoke<DbStatus>('db_status')
      .then(setDb)
      .catch(() => setDb(null));
  }, []);

  const rows: [string, string][] = [
    ['Version', version ? `v${version}` : '…'],
    ['Data', db?.path ?? '…'],
    [
      'Database',
      db ? `SQLite · ${db.journal_mode.toUpperCase()} · schema v${db.schema_version}` : '…',
    ],
    ['Licence', 'MIT'],
  ];

  return (
    <div className="flex max-w-[720px] flex-col gap-[22px] p-8">
      {/* Baseline lockup like the sidebar's: word and version sit on the mark's bottom edge. */}
      <div className="flex items-baseline gap-3.5">
        <BlotterMark className="size-12 flex-none" />
        <span className="wordmark-shadow font-brand text-[30px] leading-none font-brand-weight">
          Blotter
        </span>
        <span className="font-mono text-sm text-text-muted">{version ? `v${version}` : ''}</span>
      </div>
      <p className="m-0 max-w-[520px] leading-relaxed text-text-muted">
        A keyboard-first daily planner. Your data stays on this device. MIT licence.
      </p>
      <dl className="m-0 flex flex-col rounded-lg border border-border">
        {rows.map(([k, v], i) => (
          <div
            key={k}
            className={cn(
              'flex items-center gap-3 px-3.5 py-3',
              i < rows.length - 1 && 'border-b border-border',
            )}
          >
            <dt className="w-[120px] text-text-muted">{k}</dt>
            <dd className="m-0 flex-1 truncate font-mono text-sm select-text" title={v}>
              {v}
            </dd>
          </div>
        ))}
      </dl>
    </div>
  );
}
