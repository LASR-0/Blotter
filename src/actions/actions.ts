import { invoke } from '@tauri-apps/api/core';
import {
  ArrowBendDownRight,
  ArrowClockwise,
  ArrowCounterClockwise,
  ArrowDown,
  ArrowLeft,
  ArrowRight,
  ArrowsOutLineVertical,
  ArrowSquareOut,
  ChatCircleText,
  ArrowUp,
  CheckSquare,
  NotePencil as EditIcon,
  Trash,
  ArrowsDownUp,
  Rows,
  Calendar,
  CalendarDots,
  CheckCircle,
  CrosshairSimple,
  Gear,
  Keyboard,
  Lightning,
  MagicWand,
  MagnifyingGlass,
  NotePencil,
  Moon,
  Notebook,
  Palette,
  Plus,
  Power,
  Prohibit,
  Scroll,
  SidebarSimple,
  SquaresFour,
  Sun,
  SunHorizon,
  ArrowsLeftRight,
  Tray,
  type Icon,
} from '@phosphor-icons/react';
import { deleteFocused, openFocused, toggleFocused } from '@/actions/itemActions';
import { useChat } from '@/chat/chatStore';
import { runAutoSlot } from '@/app/autoSlot';
import { rolloverNow } from '@/app/rollover';
import { redoLast, undoLast } from '@/app/undo';
import { useFocus } from '@/app/focus';
import { useNav } from '@/app/nav';
import { useSelection, type BulkEditor } from '@/app/selection';
import { useOverlay } from '@/app/overlay';
import { PRESETS } from '@/theme/presets';
import { useTheme } from '@/theme/theme';

/** Palette and cheat-sheet sections, in display order. */
export const ACTION_GROUPS = ['Go to', 'View', 'Focused item', 'Actions', 'Theme'] as const;
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
  /** Further default bindings, e.g. `k` alongside ArrowUp. */
  altBindings?: string[];
  /** `global` (default) or a view id where the binding is active. */
  context?: string;
  /** Extra words the palette matches on. */
  keywords?: string[];
  /** Muted hint after the label in the palette. */
  hint?: string;
  /** Text fields keep this key for themselves (Ctrl+Z undoes typing there). */
  yieldsToText?: boolean;
  /** The binding also works while an overlay (palette, sheet) is open. */
  inOverlay?: boolean;
  /** Hidden from the command palette (pure navigation keys). */
  paletteHidden?: boolean;
  /** Shortcut-sheet override: hide (false) or show one summary row for a family. */
  sheet?: false | { label: string; keys: string };
  run: () => void;
}

const nav = () => useNav.getState();
const overlay = () => useOverlay.getState();

const controller = () => useFocus.getState().controller;

/** Context active in views with roving focus (Diary, Week, Month, Year, Inbox). */
export const NAV_CONTEXT = 'nav';

/** Views with roving keyboard focus, where `nav` bindings are live. */
export const NAV_VIEWS: readonly string[] = ['diary', 'week', 'month', 'year', 'inbox'];

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

    ...(
      [
        ['left', 'ArrowLeft', 'h', ArrowLeft],
        ['right', 'ArrowRight', 'l', ArrowRight],
        ['up', 'ArrowUp', 'k', ArrowUp],
        ['down', 'ArrowDown', 'j', ArrowDown],
      ] as const
    ).map(([dir, key, vim, icon]): Action => ({
      id: `focus.${dir}`,
      label: `Move focus ${dir}`,
      group: 'Focused item',
      icon,
      binding: key,
      altBindings: [vim],
      context: NAV_CONTEXT,
      paletteHidden: true,
      sheet: dir === 'left' ? { label: 'Move focus (or arrows)', keys: 'h j k l' } : false,
      run: () => controller()?.move(dir),
    })),
    ...(
      [
        ['left', 'Shift+ArrowLeft', 'Move item a day earlier'],
        ['right', 'Shift+ArrowRight', 'Move item a day later'],
        ['up', 'Shift+ArrowUp', 'Move item 15m earlier'],
        ['down', 'Shift+ArrowDown', 'Move item 15m later'],
      ] as const
    ).map(([dir, key, label]): Action => ({
      id: `item.move.${dir}`,
      label,
      group: 'Focused item',
      icon: ArrowsDownUp,
      binding: key,
      context: NAV_CONTEXT,
      paletteHidden: true,
      sheet: dir === 'left' ? { label: 'Move item (day / 15m)', keys: 'Shift+←↑↓→' } : false,
      run: () => controller()?.shift?.(dir),
    })),
    {
      id: 'item.resize.longer',
      label: 'Make item 15m longer',
      group: 'Focused item',
      icon: ArrowsOutLineVertical,
      binding: 'Alt+ArrowDown',
      context: NAV_CONTEXT,
      paletteHidden: true,
      sheet: { label: 'Resize item', keys: 'Alt+↑↓' },
      run: () => controller()?.resize?.('down'),
    },
    {
      id: 'item.resize.shorter',
      label: 'Make item 15m shorter',
      group: 'Focused item',
      icon: ArrowsOutLineVertical,
      binding: 'Alt+ArrowUp',
      context: NAV_CONTEXT,
      paletteHidden: true,
      sheet: false,
      run: () => controller()?.resize?.('up'),
    },
    {
      id: 'item.open',
      label: 'Open',
      group: 'Focused item',
      icon: EditIcon,
      binding: 'Enter',
      context: NAV_CONTEXT,
      paletteHidden: true,
      run: () => {
        if (!openFocused()) controller()?.enter?.();
      },
    },
    {
      id: 'item.edit',
      label: 'Edit focused item',
      group: 'Focused item',
      icon: EditIcon,
      binding: 'e',
      context: NAV_CONTEXT,
      run: () => void openFocused(),
    },
    {
      id: 'item.autoslot',
      label: 'Auto-slot the selected or focused task',
      group: 'Focused item',
      icon: MagicWand,
      binding: 's',
      context: NAV_CONTEXT,
      keywords: ['schedule', 'slot', 'place', 'plan'],
      paletteHidden: true,
      run: () => {
        const selected = [...useSelection.getState().ids];
        const focused = useFocus.getState().itemId;
        const ids = selected.length ? selected : focused ? [focused] : [];
        if (ids.length) void runAutoSlot(ids);
      },
    },
    {
      id: 'rollover.now',
      label: 'Roll over unfinished tasks now',
      group: 'Actions',
      icon: ArrowBendDownRight,
      keywords: ['rollover', 'yesterday', 'overdue', 'unfinished', 'end of day'],
      run: () => void rolloverNow(),
    },
    {
      id: 'inbox.autoslot',
      label: 'Auto-slot Inbox',
      group: 'Actions',
      icon: MagicWand,
      keywords: ['schedule', 'slot', 'place', 'plan', 'inbox'],
      run: () => void runAutoSlot(),
    },
    {
      id: 'item.toggle',
      label: 'Toggle complete',
      group: 'Focused item',
      icon: CheckSquare,
      binding: 'x',
      context: NAV_CONTEXT,
      keywords: ['done', 'complete', 'finish'],
      run: toggleFocused,
    },
    {
      id: 'item.delete',
      label: 'Delete focused item',
      group: 'Focused item',
      icon: Trash,
      binding: 'Delete',
      context: NAV_CONTEXT,
      run: deleteFocused,
    },
    {
      id: 'inbox.select',
      label: 'Select / deselect task',
      group: 'Focused item',
      icon: CheckSquare,
      binding: 'Space',
      context: 'inbox',
      paletteHidden: true,
      run: () => {
        const id = useFocus.getState().itemId;
        if (id) useSelection.getState().toggle(id);
      },
    },
    {
      id: 'inbox.clear',
      label: 'Clear selection',
      group: 'Focused item',
      icon: CheckSquare,
      binding: 'Escape',
      context: 'inbox',
      paletteHidden: true,
      sheet: false,
      run: () => useSelection.getState().clear(),
    },
    ...(
      [
        ['schedule', 'd', 'Schedule selected tasks'],
        ['duration', 't', 'Set duration of selected tasks'],
        ['priority', 'p', 'Set priority of selected tasks'],
        ['due', 'u', 'Set due date of selected tasks'],
      ] as const
    ).map(([editor, key, label]): Action => ({
      id: `inbox.bulk.${editor}`,
      label,
      group: 'Focused item',
      icon: CheckSquare,
      binding: key,
      context: 'inbox',
      paletteHidden: true,
      sheet:
        editor === 'schedule'
          ? { label: 'Bulk schedule / duration / priority / due', keys: 'd t p u' }
          : false,
      run: () => useSelection.getState().openEditor(editor as BulkEditor),
    })),
    {
      id: 'week.cursor',
      label: 'Switch item ↔ time cursor',
      group: 'Focused item',
      icon: Rows,
      binding: 'Tab',
      context: 'week',
      paletteHidden: true,
      run: () => controller()?.tab?.(),
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
      id: 'quick.add',
      label: 'Quick add',
      group: 'Actions',
      icon: Plus,
      binding: 'n',
      keywords: ['create', 'add', 'task', 'capture', 'natural'],
      run: () => overlay().open('quickadd'),
    },
    {
      // The OS-level shortcut is registered by the backend; this entry lists it
      // on the sheet and opens the popup from the palette (e.g. on Wayland).
      id: 'capture.open',
      label: 'Quick capture (system-wide)',
      group: 'Actions',
      icon: Lightning,
      binding: 'Ctrl+Shift+Space',
      keywords: ['capture', 'popup', 'inbox', 'brain dump'],
      run: () => void invoke('show_capture'),
    },
    {
      id: 'blockout.lift',
      label: 'Lift block-outs on the focused day',
      group: 'Actions',
      icon: Prohibit,
      binding: 'b l',
      keywords: ['block-out', 'blockout', 'exception', 'leave', 'holiday', 'day off'],
      run: () => overlay().openLift(controller()?.day?.() ?? useNav.getState().anchor),
    },
    {
      id: 'blockouts.settings',
      label: 'Edit block-outs',
      group: 'Go to',
      icon: Prohibit,
      keywords: ['block-out', 'blockout', 'work hours', 'settings'],
      run: () => useNav.getState().openSettings('blockouts'),
    },
    {
      id: 'keybindings.settings',
      label: 'Edit keybindings',
      group: 'Go to',
      icon: Keyboard,
      keywords: ['shortcuts', 'keys', 'rebind', 'settings'],
      run: () => useNav.getState().openSettings('keybindings'),
    },
    {
      id: 'setup.run',
      label: 'Run setup again',
      group: 'Actions',
      icon: MagicWand,
      keywords: ['first run', 'welcome', 'onboarding'],
      run: () => overlay().open('setup'),
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
      id: 'edit.undo',
      label: 'Undo',
      group: 'Actions',
      icon: ArrowCounterClockwise,
      binding: 'Ctrl+Z',
      yieldsToText: true,
      keywords: ['revert', 'back'],
      run: () => void undoLast(),
    },
    {
      id: 'edit.redo',
      label: 'Redo',
      group: 'Actions',
      icon: ArrowClockwise,
      binding: 'Ctrl+Shift+Z',
      altBindings: ['Ctrl+Y'],
      yieldsToText: true,
      run: () => void redoLast(),
    },
    {
      id: 'chat.toggle',
      label: 'Toggle chat',
      group: 'View',
      icon: ChatCircleText,
      binding: 'Ctrl+/',
      keywords: ['ai', 'assistant', 'claude', 'ask'],
      run: () => useChat.getState().setOpen(!useChat.getState().open),
    },
    {
      id: 'chat.popout',
      label: 'Pop out chat',
      group: 'View',
      icon: ArrowSquareOut,
      binding: 'Ctrl+Shift+/',
      keywords: ['ai', 'window', 'sticky', 'on top'],
      run: () => {
        const chat = useChat.getState();
        chat.setOpen(false);
        void invoke('open_chat_window', { threadId: chat.threadId });
      },
    },
    {
      id: 'chat.briefing',
      label: 'Daily briefing',
      group: 'Actions',
      icon: SunHorizon,
      keywords: ['ai', 'brief', 'morning', 'summary', 'today', 'plan my day'],
      run: () =>
        void invoke<string>('chat_briefing').then((id) => {
          useChat.getState().showThread(id);
          useChat.getState().setOpen(true);
        }),
    },
    {
      id: 'chat.new',
      label: 'New chat thread',
      group: 'Actions',
      icon: NotePencil,
      keywords: ['ai', 'assistant', 'conversation'],
      run: () => {
        useChat.getState().showThread(null);
        useChat.getState().setOpen(true);
      },
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
