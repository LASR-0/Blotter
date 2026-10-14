import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { isCalendarView, stepPeriod, type CalendarView, type View } from '@/lib/period';

export type SettingsPage = 'theme' | 'about';

interface NavState {
  view: View;
  /** The calendar view `v` returns to from the Diary. */
  calView: CalendarView;
  /** Date the current period is built around (local midnight). */
  anchor: Date;
  /** Explicit sidebar choice; null follows the window width. */
  sidebarCollapsed: boolean | null;
  /** Window is narrower than SIDEBAR_AUTO_COLLAPSE_PX. */
  narrow: boolean;
  settingsPage: SettingsPage;
  /** Keys pressed so far in an unfinished chord, for the top-bar hint. */
  chord: { prefix: string[]; continuations: string[] } | null;

  go: (view: View) => void;
  toggleDiaryCalendar: () => void;
  step: (dir: 1 | -1) => void;
  today: () => void;
  /** Shows `date` in the Diary (or the current calendar view if one is open). */
  goToDate: (date: Date) => void;
  /** Opens the Week view on the week containing `date`. */
  openWeek: (date: Date) => void;
  toggleSidebar: () => void;
  setNarrow: (narrow: boolean) => void;
  openSettings: (page?: SettingsPage) => void;
  setChord: (chord: NavState['chord']) => void;
}

const midnight = (d = new Date()) => new Date(d.getFullYear(), d.getMonth(), d.getDate());

export const useNav = create<NavState>()(
  persist(
    (set) => ({
      view: 'diary',
      calView: 'week',
      anchor: midnight(),
      sidebarCollapsed: null,
      narrow: false,
      settingsPage: 'theme',
      chord: null,

      go: (view) => set((s) => ({ view, calView: isCalendarView(view) ? view : s.calView })),
      toggleDiaryCalendar: () => set((s) => ({ view: s.view === 'diary' ? s.calView : 'diary' })),
      step: (dir) => set((s) => ({ anchor: stepPeriod(s.view, s.anchor, dir) })),
      today: () => set({ anchor: midnight() }),
      goToDate: (date) =>
        set((s) => ({ anchor: midnight(date), view: isCalendarView(s.view) ? s.view : 'diary' })),
      openWeek: (date) => set({ anchor: midnight(date), view: 'week', calView: 'week' }),
      toggleSidebar: () => set((s) => ({ sidebarCollapsed: !isSidebarCollapsed(s) })),
      setNarrow: (narrow) => set({ narrow }),
      openSettings: (page) =>
        set((s) => ({ view: 'settings', settingsPage: page ?? s.settingsPage })),
      setChord: (chord) => set({ chord }),
    }),
    {
      name: 'blotter.nav',
      // The anchor always starts at today; only layout choices persist.
      partialize: (s) => ({
        view: s.view,
        calView: s.calView,
        sidebarCollapsed: s.sidebarCollapsed,
      }),
    },
  ),
);

/** Effective sidebar state: the explicit choice, else collapsed when narrow. */
export function isSidebarCollapsed(s: Pick<NavState, 'sidebarCollapsed' | 'narrow'>): boolean {
  return s.sidebarCollapsed ?? s.narrow;
}

/** Width below which the sidebar collapses to its icon rail by default. */
export const SIDEBAR_AUTO_COLLAPSE_PX = 1180;
