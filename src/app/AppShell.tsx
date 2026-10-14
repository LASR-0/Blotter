import { useEffect, useMemo } from 'react';
import { buildActions } from '@/actions/actions';
import { useKeymap } from '@/keymap/useKeymap';
import { usePlannerSync } from '@/lib/queries';
import { useTheme } from '@/theme/theme';
import { DiaryView } from '@/views/DiaryView';
import { InboxView } from '@/views/InboxView';
import { PendingView } from '@/views/PendingView';
import { SettingsView } from '@/views/SettingsView';
import { SIDEBAR_AUTO_COLLAPSE_PX, useNav } from './nav';
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

export function AppShell() {
  const view = useNav((s) => s.view);
  const actions = useMemo(() => buildActions(), []);
  const contexts = useMemo(() => [view], [view]);

  useKeymap(actions, contexts);
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
          {(view === 'week' || view === 'month' || view === 'year' || view === 'history') && (
            <PendingView view={view} />
          )}
        </main>
      </div>
    </div>
  );
}
