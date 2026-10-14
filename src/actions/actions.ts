import { useNav } from '@/app/nav';
import { PRESETS } from '@/theme/presets';
import { useTheme } from '@/theme/theme';

export type ActionGroup = 'Navigate' | 'View' | 'Theme' | 'App';

/**
 * One user-facing action. The same list drives the keymap and (from #09)
 * the command palette; the keybinding editor (Phase 10) overrides
 * `binding` per id.
 */
export interface Action {
  id: string;
  label: string;
  group: ActionGroup;
  /** Default binding, e.g. `g w` or `Ctrl+K`. */
  binding?: string;
  /** `global` (default) or a view id where the binding is active. */
  context?: string;
  /** Extra words the palette matches on. */
  keywords?: string[];
  run: () => void;
}

const nav = () => useNav.getState();

/** Builds the action list. Actions read stores at run time, so the list is static. */
export function buildActions(): Action[] {
  return [
    {
      id: 'go.diary',
      label: 'Go to Diary',
      group: 'Navigate',
      binding: 'g d',
      run: () => nav().go('diary'),
    },
    {
      id: 'go.week',
      label: 'Go to Week',
      group: 'Navigate',
      binding: 'g w',
      run: () => nav().go('week'),
    },
    {
      id: 'go.month',
      label: 'Go to Month',
      group: 'Navigate',
      binding: 'g m',
      run: () => nav().go('month'),
    },
    {
      id: 'go.year',
      label: 'Go to Year',
      group: 'Navigate',
      binding: 'g y',
      run: () => nav().go('year'),
    },
    {
      id: 'go.history',
      label: 'Go to History',
      group: 'Navigate',
      binding: 'g h',
      run: () => nav().go('history'),
    },
    {
      id: 'go.inbox',
      label: 'Toggle Inbox',
      group: 'Navigate',
      binding: 'i',
      run: () => nav().go(nav().view === 'inbox' ? 'diary' : 'inbox'),
    },
    {
      id: 'go.settings',
      label: 'Open Settings',
      group: 'Navigate',
      binding: 'Ctrl+,',
      keywords: ['preferences'],
      run: () => nav().openSettings(),
    },
    {
      id: 'view.toggle',
      label: 'Toggle Diary ↔ Calendar',
      group: 'View',
      binding: 'v',
      run: () => nav().toggleDiaryCalendar(),
    },
    {
      id: 'period.prev',
      label: 'Previous period',
      group: 'View',
      binding: '[',
      run: () => nav().step(-1),
    },
    {
      id: 'period.next',
      label: 'Next period',
      group: 'View',
      binding: ']',
      run: () => nav().step(1),
    },
    {
      id: 'period.today',
      label: 'Jump to today',
      group: 'View',
      binding: '.',
      run: () => nav().today(),
    },
    {
      id: 'sidebar.toggle',
      label: 'Toggle sidebar',
      group: 'View',
      binding: 'Ctrl+B',
      run: () => nav().toggleSidebar(),
    },
    ...PRESETS.map((t): Action => ({
      id: `theme.${t.id}`,
      label: `Theme: ${t.label}`,
      group: 'Theme',
      keywords: ['appearance', 'colour', 'color'],
      run: () => useTheme.getState().setTheme(t.id),
    })),
  ];
}
