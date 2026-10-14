import { useEffect, useMemo } from 'react';
import { buildActions, NAV_CONTEXT } from '@/actions/actions';
import { useKeymap } from '@/keymap/useKeymap';
import { usePlannerSync } from '@/lib/queries';
import { useTheme } from '@/theme/theme';
import { DiaryView } from '@/views/diary/DiaryView';
import { InboxView } from '@/views/InboxView';
import { PendingView } from '@/views/PendingView';
import { SettingsView } from '@/views/SettingsView';
import { MonthView } from '@/views/MonthView';
import { WeekView } from '@/views/WeekView';
import { YearView } from '@/views/YearView';
import { CommandPalette } from './CommandPalette';
import { ConfirmDialog } from './ConfirmDialog';
import { useFocus } from './focus';
import { ItemDialog } from './ItemDialog';
import { SIDEBAR_AUTO_COLLAPSE_PX, useNav } from './nav';
import { useOverlay } from './overlay';
import { ShortcutSheet } from './ShortcutSheet';
import { Sidebar } from './Sidebar';
import { TopBar } from './TopBar';

/** Tracks whether the window is below the sidebar auto-collapse width. */
function useNarrowWindow() {
  const setNarrow = useNav((s) => s.setNarrow);
  useEffect(() => {
    const query = window.matchMedia(`(max-width: ${SIDEBAR_AUTO_COLLAPSE_PX - 1}px)`);
    const update = () => setNarrow(query.matches);
    update();
    query.addEventListener('change', update);
    return () => query.removeEventListener('change', update);
  }, [setNarrow]);
}

/** Views with roving keyboard focus. */
const NAV_VIEWS: readonly string[] = ['diary', 'week', 'month', 'year', 'inbox'];

export function AppShell() {
  const view = useNav((s) => s.view);
  const actions = useMemo(() => buildActions(), []);
  const contexts = useMemo(() => (NAV_VIEWS.includes(view) ? [view, NAV_CONTEXT] : [view]), [view]);

  // Focus belongs to the view it was set in.
  useEffect(() => {
    useFocus.getState().setItem(null);
  }, [view]);

  const overlayOpen = useOverlay((s) => s.overlay !== null);
  useKeymap(actions, contexts, overlayOpen);
  usePlannerSync();
  useNarrowWindow();

  const loadTheme = useTheme((s) => s.load);
  useEffect(() => {
    loadTheme().catch((e: unknown) => console.error('Could not load theme', e));
  }, [loadTheme]);

  return (
    <div className="flex h-full min-h-0 overflow-hidden bg-bg text-base text-text">
      <Sidebar />
      <div className="flex min-w-0 flex-1 flex-col">
        <TopBar />
        <main className="relative min-h-0 flex-1">
          {view === 'diary' && <DiaryView />}
          {view === 'inbox' && <InboxView />}
          {view === 'settings' && <SettingsView />}
          {view === 'week' && <WeekView />}
          {view === 'month' && <MonthView />}
          {view === 'year' && <YearView />}
          {view === 'history' && <PendingView view="history" />}
        </main>
      </div>
      <CommandPalette actions={actions} />
      <ShortcutSheet actions={actions} />
      <ItemDialog />
      <ConfirmDialog />
    </div>
  );
}
