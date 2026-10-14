import { invoke } from '@tauri-apps/api/core';
import {
  ArrowLeft,
  ArrowRight,
  Calendar,
  CalendarDots,
  CheckCircle,
  CrosshairSimple,
  Gear,
  Keyboard,
  MagnifyingGlass,
  NotePencil,
  Moon,
  Notebook,
  Palette,
  Power,
  Scroll,
  SidebarSimple,
  SquaresFour,
  Sun,
  ArrowsLeftRight,
  Tray,
  type Icon,
} from '@phosphor-icons/react';
import { useNav } from '@/app/nav';
import { useOverlay } from '@/app/overlay';
import { PRESETS } from '@/theme/presets';
import { useTheme } from '@/theme/theme';

/** Palette and cheat-sheet sections, in display order. */
export const ACTION_GROUPS = ['Go to', 'View', 'Actions', 'Theme'] as const;
export type ActionGroup = (typeof ACTION_GROUPS)[number];

/**
 * One user-facing action. The same list drives the keymap, the command
 * palette and the shortcut sheet; the keybinding editor (Phase 10)
 * overrides `binding` per id.
 */
export interface Action {
  id: string;
  label: string;
  group: ActionGroup;
  icon: Icon;
  /** Default binding, e.g. `g w` or `Ctrl+K`. */
  binding?: string;
  /** `global` (default) or a view id where the binding is active. */
  context?: string;
  /** Extra words the palette matches on. */
  keywords?: string[];
  /** Muted hint after the label in the palette. */
  hint?: string;
  /** The binding also works while an overlay (palette, sheet) is open. */
  inOverlay?: boolean;
  run: () => void;
}

const nav = () => useNav.getState();
const overlay = () => useOverlay.getState();

const THEME_ICONS: Record<string, Icon> = {
  dark: Moon,
  light: Sun,
  paper: Scroll,
  gruvbox: Palette,
};

/** Builds the action list. Actions read stores at run time, so the list is static. */
export function buildActions(): Action[] {
  const goTo = (
    view: 'diary' | 'week' | 'month' | 'year' | 'history',
    label: string,
    icon: Icon,
    binding: string,
  ): Action => ({
    id: `go.${view}`,
    label,
    group: 'Go to',
    icon,
    binding,
    run: () => nav().go(view),
  });

  return [
    goTo('diary', 'Diary', Notebook, 'g d'),
    goTo('week', 'Week', Calendar, 'g w'),
    goTo('month', 'Month', CalendarDots, 'g m'),
    goTo('year', 'Year', SquaresFour, 'g y'),
    goTo('history', 'History', CheckCircle, 'g h'),
    {
      id: 'go.inbox',
      label: 'Inbox',
      group: 'Go to',
      icon: Tray,
      binding: 'i',
      run: () => nav().go(nav().view === 'inbox' ? 'diary' : 'inbox'),
    },
    {
      id: 'go.settings',
      label: 'Settings',
      group: 'Go to',
      icon: Gear,
      binding: 'Ctrl+,',
      keywords: ['preferences', 'options'],
      run: () => nav().openSettings(),
    },

    {
      id: 'view.toggle',
      label: 'Diary ↔ Calendar',
      group: 'View',
      icon: ArrowsLeftRight,
      binding: 'v',
      keywords: ['switch', 'toggle'],
      run: () => nav().toggleDiaryCalendar(),
    },
    {
      id: 'period.prev',
      label: 'Previous period',
      group: 'View',
      icon: ArrowLeft,
      binding: '[',
      run: () => nav().step(-1),
    },
    {
      id: 'period.next',
      label: 'Next period',
      group: 'View',
      icon: ArrowRight,
      binding: ']',
      run: () => nav().step(1),
    },
    {
      id: 'period.today',
      label: 'Today',
      group: 'View',
      icon: CrosshairSimple,
      binding: '.',
      keywords: ['jump', 'now'],
      run: () => nav().today(),
    },
    {
      id: 'sidebar.toggle',
      label: 'Toggle sidebar',
      group: 'View',
      icon: SidebarSimple,
      binding: 'Ctrl+B',
      run: () => nav().toggleSidebar(),
    },

    {
      id: 'palette.toggle',
      label: 'Command palette',
      group: 'Actions',
      icon: MagnifyingGlass,
      binding: 'Ctrl+K',
      inOverlay: true,
      run: () => overlay().toggle('palette'),
    },
    {
      id: 'item.new',
      label: 'New item',
      group: 'Actions',
      icon: NotePencil,
      keywords: ['create', 'add', 'task', 'event'],
      run: () => overlay().openItem({ defaults: {} }),
    },
    {
      id: 'help.shortcuts',
      label: 'Keyboard shortcuts',
      group: 'Actions',
      icon: Keyboard,
      binding: '?',
      keywords: ['help', 'cheat sheet', 'keys'],
      inOverlay: true,
      run: () => overlay().toggle('shortcuts'),
    },
    {
      id: 'app.quit',
      label: 'Quit Blotter',
      group: 'Actions',
      icon: Power,
      binding: 'Ctrl+Q',
      keywords: ['exit', 'close'],
      inOverlay: true,
      run: () => void invoke('quit_app'),
    },

    ...PRESETS.map((t): Action => ({
      id: `theme.${t.id}`,
      label: `Theme: ${t.label}`,
      group: 'Theme',
      icon: THEME_ICONS[t.id] ?? Palette,
      keywords: ['appearance', 'colour', 'color'],
      run: () => useTheme.getState().setTheme(t.id),
    })),
  ];
}
