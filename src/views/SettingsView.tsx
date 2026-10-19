import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Bell,
  CalendarDots,
  ClockCounterClockwise,
  Hash,
  Info,
  Keyboard,
  Palette,
  Prohibit,
  SlidersHorizontal,
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
import { GeneralPage } from './settings/GeneralPage';
import { KeybindingsPage } from './settings/KeybindingsPage';
import { NotificationsPage } from './settings/NotificationsPage';
import { TagsPage } from './settings/TagsPage';

// Pages appear here as their phases land: General, Block-outs, Calendars,
// AI providers, Notifications, Keybindings and Backups (design S1–S9).
const PAGES: readonly { id: SettingsPage; label: string; icon: Icon }[] = [
  { id: 'general', label: 'General', icon: SlidersHorizontal },
  { id: 'blockouts', label: 'Block-outs', icon: Prohibit },
  { id: 'calendars', label: 'Calendars (ICS)', icon: CalendarDots },
  { id: 'tags', label: 'Tags', icon: Hash },
  { id: 'notifications', label: 'Notifications', icon: Bell },
  { id: 'ai', label: 'AI providers', icon: Sparkle },
  { id: 'keybindings', label: 'Keybindings', icon: Keyboard },
  { id: 'theme', label: 'Theme', icon: Palette },
  { id: 'backups', label: 'Backups', icon: ClockCounterClockwise },
  { id: 'about', label: 'About', icon: Info },
];

/** Windows at least this wide show every page but Theme on one scrolling page. */
const ONE_PAGE_MIN_PX = 1600;

function useWideWindow(): boolean {
  const query = `(min-width: ${ONE_PAGE_MIN_PX}px)`;
  const [wide, setWide] = useState(() => window.matchMedia(query).matches);
  useEffect(() => {
    const mq = window.matchMedia(query);
    const update = () => setWide(mq.matches);
    mq.addEventListener('change', update);
    return () => mq.removeEventListener('change', update);
  }, [query]);
  return wide;
}

function PageBody({ page }: { page: SettingsPage }) {
  switch (page) {
    case 'general':
      return <GeneralPage />;
    case 'blockouts':
      return <BlockoutsPage />;
    case 'calendars':
      return <CalendarsPage />;
    case 'tags':
      return <TagsPage />;
    case 'notifications':
      return <NotificationsPage />;
    case 'ai':
      return <AiProvidersPage />;
    case 'keybindings':
      return <KeybindingsPage />;
    case 'backups':
      return <BackupsPage />;
    case 'about':
      return <AboutPage />;
    case 'theme':
      return <ThemePage />;
  }
}

const sectionId = (p: SettingsPage) => `settings-${p}`;

export function SettingsView() {
  const page = useNav((s) => s.settingsPage);
  const openSettings = useNav((s) => s.openSettings);
  const wide = useWideWindow();
  // Wide: every page but Theme on one long page; the nav scrolls to them.
  const onePage = wide && page !== 'theme';
  const mainRef = useRef<HTMLElement>(null);
  /** The section in view, for the nav highlight on the long page. */
  const [inView, setInView] = useState<SettingsPage>(page);

  /**
   * Scrolls the settings column (only it: scrollIntoView would also scroll
   * the app's frame and shift the sidebar) so `id`'s section is at the top.
   */
  const scrollTo = (id: SettingsPage, behavior: ScrollBehavior = 'auto') => {
    const main = mainRef.current;
    const el = document.getElementById(sectionId(id));
    if (!main || !el) return;
    const top = el.getBoundingClientRect().top - main.getBoundingClientRect().top + main.scrollTop;
    main.scrollTo({ top, behavior });
  };

  // Opening a page (from the nav, the palette or a link) scrolls to it.
  useEffect(() => {
    if (onePage) scrollTo(page);
  }, [onePage, page]);

  const onScroll = () => {
    const main = mainRef.current;
    if (!main) return;
    const top = main.getBoundingClientRect().top + 80;
    let current: SettingsPage = 'general';
    for (const p of PAGES) {
      const el = document.getElementById(sectionId(p.id));
      if (el && el.getBoundingClientRect().top <= top) current = p.id;
    }
    setInView(current);
  };

  const pick = (id: SettingsPage) => {
    if (wide && id !== 'theme' && page === id) {
      // Already the open page: scroll back to it.
      scrollTo(id, 'smooth');
    }
    setInView(id);
    openSettings(id);
  };
  const current = onePage ? inView : page;

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
            onClick={() => pick(p.id)}
            aria-current={current === p.id ? 'page' : undefined}
            className={cn(
              'flex h-[30px] cursor-pointer items-center gap-2.5 rounded-md px-2 text-left hover:bg-hover',
              current === p.id && 'bg-hover font-medium',
            )}
          >
            <p.icon size={15} className="text-text-muted" />
            {p.label}
          </button>
        ))}
      </nav>
      <main
        ref={mainRef}
        onScroll={onePage ? onScroll : undefined}
        className={cn(
          'min-w-0 flex-1 overflow-auto',
          // Content centred: on the long page each section is also wider than a single page.
          onePage ? '[&>section>*]:mx-auto [&>section>*]:max-w-[1120px]' : '[&>*]:mx-auto',
        )}
      >
        {onePage ? (
          <>
            {PAGES.filter((p) => p.id !== 'theme').map((p) => (
              <section
                key={p.id}
                id={sectionId(p.id)}
                aria-label={p.label}
                className="border-b border-border last:border-b-0"
              >
                <PageBody page={p.id} />
              </section>
            ))}
            {/* Room for the last section to scroll to the top. */}
            <div aria-hidden className="h-[50vh]" />
          </>
        ) : (
          <PageBody page={page} />
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
      {/* The sidebar's lockup at twice the size (word as tall as the mark), version below. */}
      <div className="flex flex-col gap-2">
        <div className="flex items-baseline gap-4">
          <BlotterMark className="size-12 flex-none" />
          <span className="wordmark-shadow font-brand text-[50px] leading-none font-brand-weight">
            Blotter
          </span>
        </div>
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
