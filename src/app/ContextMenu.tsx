// The right-click menu, drawn with the theme instead of the browser's. One
// listener for the whole window reads what was clicked from the markup:
//   [data-focus-id] / [data-menu-item]   an item (cards, calendar blocks, chips)
//   [data-date]                          a day (Diary column, Week/timeline grid, Month cell)
// Text fields keep the browser's menu (copy, paste, spelling); so does
// Shift+right-click, for the developer tools.

import { useEffect, useState, type ReactNode } from 'react';
import { DropdownMenu } from 'radix-ui';
import { format, parseISO } from 'date-fns';
import {
  ArrowCounterClockwise,
  CalendarPlus,
  Check,
  NotePencil,
  Prohibit,
  Trash,
  Tray,
  type Icon,
} from '@phosphor-icons/react';
import { deleteFocused, fetchItem } from '@/actions/itemActions';
import { runOp } from '@/lib/api';
import { isEventKey } from '@/lib/calendarEvents';
import { hhmm } from '@/lib/dnd';
import type { Item } from '@/lib/types';
import { useFocus } from './focus';
import { useOverlay } from './overlay';

type Target = { kind: 'item'; item: Item } | { kind: 'day'; date: string; minute: number | null };

interface Open {
  x: number;
  y: number;
  target: Target;
}

/** Half-hour slot under the pointer on a time grid, or null off one. */
function gridMinute(zone: HTMLElement, y: number): number | null {
  const hourPx = Number(zone.dataset.hourPx);
  if (!hourPx) return null;
  const min = ((y - zone.getBoundingClientRect().top) / hourPx) * 60;
  return Math.max(0, Math.min(23 * 60 + 30, Math.floor(min / 30) * 30));
}

function isTextField(el: HTMLElement): boolean {
  return el.isContentEditable || el.closest('input, textarea, select') !== null;
}

async function targetAt(el: HTMLElement, y: number): Promise<Target | null> {
  const itemEl = el.closest<HTMLElement>('[data-menu-item], [data-focus-id]');
  const id = itemEl?.dataset.menuItem ?? itemEl?.dataset.focusId;
  if (id && !id.startsWith('cursor-') && !isEventKey(id)) {
    return { kind: 'item', item: await fetchItem(id) };
  }
  const day = el.closest<HTMLElement>('[data-date]');
  if (day?.dataset.date) return { kind: 'day', date: day.dataset.date, minute: gridMinute(day, y) };
  return null;
}

const ITEM_CLASS =
  'flex h-[28px] cursor-pointer items-center gap-2 rounded-md px-2.5 outline-none select-none data-[highlighted]:bg-hover data-[disabled]:cursor-default data-[disabled]:opacity-40';

function Entry({
  icon: EntryIcon,
  children,
  danger,
  onSelect,
}: {
  icon: Icon;
  children: ReactNode;
  danger?: boolean;
  onSelect: () => void;
}) {
  return (
    <DropdownMenu.Item
      onSelect={onSelect}
      className={`${ITEM_CLASS} ${danger ? 'text-danger' : ''}`}
    >
      <EntryIcon className={danger ? '' : 'text-text-muted'} />
      {children}
    </DropdownMenu.Item>
  );
}

function ItemEntries({ item }: { item: Item }) {
  const done = item.status === 'done';
  const scheduled = Boolean(item.start_at || item.start_date);
  // Actions go through the focused item, as their keys do.
  const focus = () => useFocus.getState().setItem(item.id);
  return (
    <>
      <DropdownMenu.Label className="max-w-[240px] truncate px-2.5 pt-1 pb-1.5 text-xs text-text-muted">
        {item.title}
      </DropdownMenu.Label>
      <Entry icon={NotePencil} onSelect={() => useOverlay.getState().openItem({ id: item.id })}>
        Open
      </Entry>
      <Entry
        icon={done ? ArrowCounterClockwise : Check}
        onSelect={() =>
          void runOp(done ? 'uncomplete_item' : 'complete_item', { id: item.id }).catch(
            console.error,
          )
        }
      >
        {done ? 'Mark not done' : 'Mark done'}
      </Entry>
      {scheduled && item.kind === 'task' && item.anchoring === 'flexible' && !item.rrule && (
        <Entry
          icon={Tray}
          onSelect={() => void runOp('unschedule_item', { id: item.id }).catch(console.error)}
        >
          Move to Inbox
        </Entry>
      )}
      <DropdownMenu.Separator className="my-1 h-px bg-border" />
      <Entry
        icon={Trash}
        danger
        onSelect={() => {
          focus();
          deleteFocused();
        }}
      >
        Delete…
      </Entry>
    </>
  );
}

function DayEntries({ date, minute }: { date: string; minute: number | null }) {
  const day = parseISO(date);
  return (
    <>
      <DropdownMenu.Label className="px-2.5 pt-1 pb-1.5 text-xs text-text-muted">
        {format(day, 'EEEE d MMMM')}
        {minute !== null && ` · ${hhmm(minute)}`}
      </DropdownMenu.Label>
      <Entry
        icon={CalendarPlus}
        onSelect={() =>
          useOverlay.getState().openItem({
            defaults: minute === null ? { date } : { date, time: hhmm(minute), duration: '30m' },
          })
        }
      >
        {minute === null ? 'Add item on this day…' : `Add item at ${hhmm(minute)}…`}
      </Entry>
      <Entry icon={Prohibit} onSelect={() => useOverlay.getState().openLift(day)}>
        Lift block-outs on this day…
      </Entry>
    </>
  );
}

/** Mounted once in the main window. */
export function ContextMenu() {
  const [open, setOpen] = useState<Open | null>(null);

  useEffect(() => {
    const onMenu = (e: MouseEvent) => {
      const el = e.target instanceof HTMLElement ? e.target : null;
      if (!el || e.shiftKey || isTextField(el)) return;
      e.preventDefault();
      const { clientX: x, clientY: y } = e;
      targetAt(el, y)
        .then((target) => setOpen(target ? { x, y, target } : null))
        .catch(console.error);
    };
    window.addEventListener('contextmenu', onMenu);
    return () => window.removeEventListener('contextmenu', onMenu);
  }, []);

  return (
    <DropdownMenu.Root open={open !== null} onOpenChange={(o) => !o && setOpen(null)} modal={false}>
      {/* A point at the pointer for the menu to open from. */}
      <DropdownMenu.Trigger asChild>
        <span
          aria-hidden
          className="pointer-events-none fixed size-0"
          style={{ left: open?.x ?? 0, top: open?.y ?? 0 }}
        />
      </DropdownMenu.Trigger>
      <DropdownMenu.Portal>
        <DropdownMenu.Content
          align="start"
          sideOffset={2}
          collisionPadding={8}
          // Focus returns to where it was, not to the invisible trigger.
          onCloseAutoFocus={(e) => e.preventDefault()}
          className="z-50 min-w-[200px] rounded-lg border border-border bg-surface-raised p-1 text-base text-text shadow-overlay"
        >
          {open?.target.kind === 'item' && <ItemEntries item={open.target.item} />}
          {open?.target.kind === 'day' && (
            <DayEntries date={open.target.date} minute={open.target.minute} />
          )}
        </DropdownMenu.Content>
      </DropdownMenu.Portal>
    </DropdownMenu.Root>
  );
}
