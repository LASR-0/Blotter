import { useMemo, useState } from 'react';
import {
  addDays,
  format,
  isSameDay,
  isSameMonth,
  isWeekend,
  startOfMonth,
  startOfWeek,
} from 'date-fns';
import { X } from '@phosphor-icons/react';
import { useViewController } from '@/app/focus';
import { useNav } from '@/app/nav';
import { useOverlay } from '@/app/overlay';
import { ErrorPanel } from '@/components/EmptyState';
import { Kbd } from '@/components/ui/kbd';
import { Popover, PopoverClose, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { isOpError } from '@/lib/api';
import { blockedIntervals, PLANNING_HOURS, windowTimes } from '@/lib/availability';
import { cn } from '@/lib/cn';
import { dayKey, itemsByDay } from '@/lib/days';
import { timeOf } from '@/lib/format';
import { isoDate, WEEK_STARTS_ON } from '@/lib/period';
import { useOp } from '@/lib/queries';
import { itemColor } from '@/lib/tags';
import type { Agenda, Item } from '@/lib/types';
import { useBlockoutWindows } from '@/lib/useBlockoutWindows';
import { useNow } from '@/lib/useNow';

/** Chips shown per cell before collapsing into "+N more". */
const MAX_CHIPS = 3;
const CELLS = 42;

function chipTime(item: Item): string {
  return item.all_day ? 'all day' : item.start_at ? timeOf(item.start_at) : '';
}

function Chip({ item }: { item: Item }) {
  const done = item.status === 'done';
  const color = itemColor(item);
  return (
    <div
      title={`${chipTime(item)} ${item.title}`}
      onDoubleClick={() => useOverlay.getState().openItem({ id: item.id })}
      style={{
        background: `color-mix(in srgb, ${color} ${done ? 7 : 16}%, var(--color-bg))`,
        borderLeft: `3px solid ${color}`,
      }}
      className="flex h-[19px] min-w-0 flex-none items-center gap-[5px] rounded-[3px] border border-transparent px-[5px] text-xs"
    >
      <span className="flex-none font-mono text-[10px] text-text-muted">{chipTime(item)}</span>
      <span className={cn('truncate font-medium', done && 'text-text-muted line-through')}>
        {item.title}
      </span>
    </div>
  );
}

function DayPopover({ day, items, count }: { day: Date; items: Item[]; count: number }) {
  const openWeek = useNav((s) => s.openWeek);
  return (
    <Popover>
      <PopoverTrigger className="cursor-pointer self-start rounded-sm border-0 bg-transparent px-1.5 py-px text-xs font-medium text-text-muted hover:bg-hover hover:text-text">
        +{count} more
      </PopoverTrigger>
      <PopoverContent side="right" align="start" className="flex w-[280px] flex-col gap-2">
        <div className="flex items-baseline gap-2">
          <span className="font-heading text-[24px] font-heading-weight">{format(day, 'd')}</span>
          <span className="font-medium">{format(day, 'EEEE')}</span>
          <span className="text-sm text-text-muted">{format(day, 'MMMM')}</span>
          <span className="flex-1" />
          <PopoverClose
            aria-label="Close"
            className="grid size-[22px] cursor-pointer place-items-center rounded-sm text-text-muted hover:bg-hover"
          >
            <X />
          </PopoverClose>
        </div>
        <div className="flex max-h-[320px] flex-col gap-1 overflow-auto">
          {items.map((item) => {
            const color = itemColor(item);
            const done = item.status === 'done';
            return (
              <button
                type="button"
                key={item.id}
                onClick={() => useOverlay.getState().openItem({ id: item.id })}
                title="Open"
                style={{
                  background: `color-mix(in srgb, ${color} 14%, var(--color-surface-raised))`,
                  borderLeft: `3px solid ${color}`,
                }}
                className="flex cursor-pointer items-center gap-2 rounded-sm px-2 py-1.5 text-left text-sm hover:brightness-110"
              >
                <span className="w-[84px] flex-none font-mono text-xs text-text-muted">
                  {item.all_day
                    ? 'all day'
                    : item.start_at && item.end_at
                      ? `${timeOf(item.start_at)}–${timeOf(item.end_at)}`
                      : chipTime(item)}
                </span>
                <span className={cn('flex-1 font-medium', done && 'text-text-muted line-through')}>
                  {item.title}
                </span>
              </button>
            );
          })}
        </div>
        <button
          type="button"
          onClick={() => openWeek(day)}
          className="flex cursor-pointer items-center gap-1.5 border-t border-border pt-2 text-left text-xs text-text-muted hover:text-text"
        >
          <Kbd>↵</Kbd>Open in Week
        </button>
      </PopoverContent>
    </Popover>
  );
}

export function MonthView() {
  const anchor = useNav((s) => s.anchor);
  const openWeek = useNav((s) => s.openWeek);
  const now = useNow(60_000);
  const days = useMemo(() => {
    const first = startOfWeek(startOfMonth(anchor), { weekStartsOn: WEEK_STARTS_ON });
    return Array.from({ length: CELLS }, (_, i) => addDays(first, i));
  }, [anchor]);
  const windows = useBlockoutWindows();
  const agenda = useOp<Agenda>('list_agenda', {
    start: isoDate(days[0] as Date),
    end: isoDate(days[CELLS - 1] as Date),
  });
  const byDay = useMemo(() => itemsByDay(agenda.data?.items ?? []), [agenda.data]);

  // Keyboard focus is a day: the chosen one if it's in this month, else today
  // (when shown) or the 1st.
  const [chosen, setChosen] = useState<Date | null>(null);
  const focusDay =
    chosen && isSameMonth(chosen, anchor)
      ? chosen
      : isSameMonth(now, anchor)
        ? new Date(now.getFullYear(), now.getMonth(), now.getDate())
        : startOfMonth(anchor);
  useViewController({
    move: (dir) => {
      const next = addDays(focusDay, { left: -1, right: 1, up: -7, down: 7 }[dir]);
      setChosen(next);
      if (!isSameMonth(next, anchor)) useNav.getState().goToDate(next);
    },
    enter: () => {
      openWeek(focusDay);
      return true;
    },
    day: () => focusDay,
  });

  return (
    <div className="relative flex h-full min-h-0 flex-col">
      {agenda.isError && (
        <div className="absolute top-12 left-1/2 z-20 w-[420px] -translate-x-1/2">
          <ErrorPanel
            title="Couldn't load this month"
            message={isOpError(agenda.error) ? agenda.error.message : String(agenda.error)}
            onRetry={() => void agenda.refetch()}
          />
        </div>
      )}
      <div className="grid grid-cols-7 border-b border-border">
        {days.slice(0, 7).map((d) => (
          <div key={d.getDay()} className="px-2.5 py-2 text-sm font-medium text-text-muted">
            {format(d, 'EEE')}
          </div>
        ))}
      </div>
      <div className="grid min-h-0 flex-1 grid-cols-7 grid-rows-6">
        {days.map((day) => {
          const inMonth = isSameMonth(day, anchor);
          const today = isSameDay(day, now);
          const items = byDay.get(dayKey(day)) ?? [];
          const shown = items.slice(0, MAX_CHIPS);
          const focused = isSameDay(day, focusDay);
          const hidden = items.length - shown.length;
          const dayBlocks = blockedIntervals(
            windows.filter((w) => w.date === isoDate(day)),
            day,
          ).filter((b) => b.end > PLANNING_HOURS.start && b.start < PLANNING_HOURS.end);
          return (
            <div
              key={day.getTime()}
              aria-selected={focused}
              onClick={() => setChosen(day)}
              className={cn(
                'relative flex min-h-0 flex-col border-r border-b border-border',
                focused && 'z-1 outline-2 -outline-offset-2 outline-focus-ring',
                !inMonth
                  ? 'bg-surface'
                  : isWeekend(day)
                    ? 'bg-[color-mix(in_srgb,var(--color-surface)_50%,var(--color-bg))]'
                    : 'bg-bg',
              )}
            >
              <div className="flex h-[30px] flex-none items-center gap-1.5 px-2 pt-1">
                <button
                  type="button"
                  onClick={() => openWeek(day)}
                  title="Open in Week"
                  className={cn(
                    'cursor-pointer text-sm',
                    today
                      ? 'grid h-[22px] min-w-[22px] place-items-center rounded-full bg-accent px-1.5 font-semibold text-on-accent'
                      : cn(
                          'font-medium hover:underline',
                          inMonth ? 'text-text' : 'text-text-muted',
                        ),
                  )}
                >
                  {day.getDate() === 1 ? format(day, 'MMM d') : format(day, 'd')}
                </button>
                {dayBlocks.length > 0 && (
                  <span
                    title={dayBlocks
                      .map((b) => `${b.window.label} ${windowTimes(b.window)} (block-out)`)
                      .join('\n')}
                    className="h-1 w-7 rounded-[2px] bg-[repeating-linear-gradient(135deg,color-mix(in_srgb,var(--color-blockout)_70%,transparent)_0_1.5px,transparent_1.5px_4px)]"
                  />
                )}
              </div>
              <div className="flex min-h-0 flex-col gap-0.5 overflow-hidden px-1 pb-1">
                {shown.map((item) => (
                  <Chip key={item.id} item={item} />
                ))}
                {hidden > 0 && <DayPopover day={day} items={items} count={hidden} />}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
