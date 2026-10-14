import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Check, Info, Palette, type Icon } from '@phosphor-icons/react';
import { useNav, type SettingsPage } from '@/app/nav';
import { cn } from '@/lib/cn';
import { PRESETS, type Theme } from '@/theme/presets';
import { useTheme } from '@/theme/theme';

// Pages appear here as their phases land: General, Block-outs, Calendars,
// AI providers, Notifications, Keybindings and Backups (design S1–S9).
const PAGES: readonly { id: SettingsPage; label: string; icon: Icon }[] = [
  { id: 'theme', label: 'Theme', icon: Palette },
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
        {page === 'about' ? <AboutPage /> : <ThemePage />}
      </main>
    </div>
  );
}

function PresetCard({
  theme,
  active,
  onPick,
}: {
  theme: Theme;
  active: boolean;
  onPick: () => void;
}) {
  return (
    <button
      type="button"
      role="radio"
      aria-checked={active}
      onClick={onPick}
      // The card renders in its own theme: tokens set on the element apply to its subtree.
      style={theme.vars as React.CSSProperties}
      className={cn(
        'flex cursor-pointer flex-col gap-2.5 rounded-lg border bg-bg p-4 text-left font-ui text-base text-text',
        active ? 'border-accent shadow-[0_0_0_1px_var(--color-accent)]' : 'border-border',
      )}
    >
      <span className="flex items-center">
        <span className="flex-1 font-heading text-[15px] font-heading-weight">{theme.label}</span>
        {active && <Check weight="bold" className="text-accent-text" />}
      </span>
      <span className="flex flex-col gap-[3px] rounded-md border border-dashed border-pencil bg-[color-mix(in_srgb,var(--color-surface)_45%,transparent)] px-[11px] py-[9px]">
        <span className="font-medium">Call Mum</span>
        <span className="font-mono text-xs text-pencil">✎ proposed · 18:30</span>
      </span>
      <span className="flex flex-col gap-[3px] rounded-md border border-border bg-surface px-[11px] py-[9px]">
        <span className="font-medium">Gym</span>
        <span className="font-mono text-xs text-text-muted">17:30 · 45m · health</span>
      </span>
      <span className="h-[30px] rounded-sm bg-hatch px-1.5 py-[3px] font-mono text-[10px] text-text-muted">
        Work
      </span>
      <span className="text-sm text-text-muted">{theme.note}</span>
    </button>
  );
}

function ThemePage() {
  const { theme, setTheme } = useTheme();
  return (
    <div className="flex max-w-[960px] flex-col gap-4 px-8 pt-6 pb-12">
      <h1 className="m-0 font-heading text-xl font-heading-weight">Theme</h1>
      <p className="m-0 max-w-[560px] leading-normal text-text-muted">
        Pick a built-in preset. The token editor with a live preview, plus import and export,
        arrives in a later build.
      </p>
      <div role="radiogroup" aria-label="Theme preset" className="grid grid-cols-4 gap-3.5">
        {PRESETS.map((t) => (
          <PresetCard
            key={t.id}
            theme={t}
            active={theme.id === t.id}
            onPick={() => setTheme(t.id)}
          />
        ))}
      </div>
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
      <div className="flex items-center gap-3.5">
        <span className="grid size-12 place-items-center rounded-[12px] bg-accent font-heading text-[24px] font-semibold text-on-accent">
          B
        </span>
        <div className="flex flex-col gap-0.5">
          <span className="font-heading text-xl font-heading-weight">Blotter</span>
          <span className="font-mono text-sm text-text-muted">{version ? `v${version}` : ''}</span>
        </div>
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
