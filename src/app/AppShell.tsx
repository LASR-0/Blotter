import { invoke } from '@tauri-apps/api/core';
import { useEffect, useMemo } from 'react';
import { listen } from '@tauri-apps/api/event';
import { buildActions, NAV_CONTEXT, NAV_VIEWS } from '@/actions/actions';
import { useKeybindings } from '@/keymap/keybindingStore';
import { FirstRun } from './FirstRun';
import { useSlidePresence } from '@/lib/useSlidePresence';
import { onPlannerChanged } from '@/lib/api';
import { syncPrefs, usePrefs } from '@/lib/prefs';
import { applyOverrides } from '@/keymap/overrides';
import { useKeymap } from '@/keymap/useKeymap';
import { usePlannerSync } from '@/lib/queries';
import { useBlockoutWindows } from '@/lib/useBlockoutWindows';
import { useTheme } from '@/theme/theme';
import { DiaryView } from '@/views/diary/DiaryView';
import { InboxView } from '@/views/inbox/InboxView';
import { HistoryView } from '@/views/HistoryView';
import { SettingsView } from '@/views/SettingsView';
import { MonthView } from '@/views/MonthView';
import { WeekView } from '@/views/WeekView';
import { YearView } from '@/views/YearView';
import { CommandPalette } from './CommandPalette';
import { ConfirmDialog } from './ConfirmDialog';
import { DragLayer } from './DragLayer';
import { useFocus } from './focus';
import { useSelection } from './selection';
import { ItemDialog } from './ItemDialog';
import { LiftDialog } from './LiftDialog';
import { AutoSlotPanel } from './AutoSlotPanel';
import { RolloverBanner } from './RolloverBanner';
import { ReminderToasts } from './ReminderToasts';
import { Notice } from './Notice';
import { ErrorBoundary } from './ErrorBoundary';
import { QuickAddDialog } from './QuickAddDialog';
import { SIDEBAR_AUTO_COLLAPSE_PX, useNav } from './nav';
import { useOverlay } from './overlay';
import { ShortcutSheet } from './ShortcutSheet';
import { Sidebar } from './Sidebar';
import { TopBar } from './TopBar';
import { ChatPanel } from '@/chat/ChatPanel';
import { SlidePanel } from '@/components/ui/slide-panel';
import { useChat, useChatEvents } from '@/chat/chatStore';

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

/** Opens the docked chat for `blotter --chat` and when the pop-out docks back. */
function useChatOpeners() {
  useEffect(() => {
    const open = (threadId?: string | null) => {
      const chat = useChat.getState();
      if (threadId !== undefined) chat.showThread(threadId);
      chat.setOpen(true);
    };
    const offCli = listen<string>('app:cli', (e) => {
      if (e.payload === 'chat') open();
    });
    const offDock = listen<{ thread_id: string | null }>('chat:dock', (e) =>
      open(e.payload.thread_id),
    );
    return () => {
      void offCli.then((stop) => stop());
      void offDock.then((stop) => stop());
    };
  }, []);
}

export function AppShell() {
  const view = useNav((s) => s.view);
  const baseActions = useMemo(() => buildActions(), []);
  const overrides = useKeybindings((s) => s.overrides);
  const actions = useMemo(() => applyOverrides(baseActions, overrides), [baseActions, overrides]);
  const loadKeybindings = useKeybindings((s) => s.load);
  useEffect(() => {
    const load = () =>
      loadKeybindings().catch((e: unknown) => console.error('Could not load keybindings', e));
    void load();
    // A restored snapshot brings its own keybindings.
    const stop = onPlannerChanged((e) => {
      if (e.ops.includes('restore')) void load();
    });
    return () => void stop.then((unlisten) => unlisten());
  }, [loadKeybindings]);
  const contexts = useMemo(() => (NAV_VIEWS.includes(view) ? [view, NAV_CONTEXT] : [view]), [view]);

  // Focus belongs to the view it was set in.
  useEffect(() => {
    useFocus.getState().setItem(null);
    useSelection.getState().clear();
  }, [view]);

  const overlayOpen = useOverlay((s) => s.overlay !== null);
  useKeymap(actions, contexts, overlayOpen);
  usePlannerSync();
  const windows = useBlockoutWindows();
  useNarrowWindow();

  const chatOpen = useChat((s) => s.open);
  const chat = useSlidePresence(chatOpen);
  useChatEvents();
  useChatOpeners();

  useEffect(() => syncPrefs(), []);
  // First run: the setup, once, on a fresh database.
  useEffect(() => {
    invoke<boolean>('first_run_pending')
      .then((pending) => pending && useOverlay.getState().open('setup'))
      .catch((e: unknown) => console.error('Could not check first run', e));
  }, []);
  // Calendar views read the week start, clock and planning hours as they
  // draw; a change redraws them (Settings itself keeps its place).
  const prefsKey = usePrefs((s) => `${s.weekStart}:${s.clock}:${JSON.stringify(s.planning)}`);
  const viewKey = view === 'settings' ? view : `${view}:${prefsKey}`;

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
          {/* Keyed by view (and prefs): a crash in one view doesn't take the app down, and
              navigating elsewhere starts it fresh. */}
          <ErrorBoundary key={viewKey} what="This view">
            {view === 'diary' && <DiaryView />}
            {view === 'inbox' && <InboxView />}
            {view === 'settings' && <SettingsView />}
            {view === 'week' && <WeekView />}
            {view === 'month' && <MonthView />}
            {view === 'year' && <YearView />}
            {view === 'history' && <HistoryView />}
          </ErrorBoundary>
        </main>
      </div>
      {chat.mounted && (
        <SlidePanel width={chat.open ? 360 : 0} label="AI chat" className="border-l border-border">
          <ErrorBoundary what="The chat">
            <ChatPanel mode="docked" />
          </ErrorBoundary>
        </SlidePanel>
      )}
      <CommandPalette actions={actions} />
      <ShortcutSheet actions={actions} />
      <QuickAddDialog />
      <ItemDialog />
      <ConfirmDialog />
      <LiftDialog />
      <FirstRun />
      <AutoSlotPanel />
      <RolloverBanner />
      <ReminderToasts />
      <Notice />
      <DragLayer windows={windows} />
    </div>
  );
}
