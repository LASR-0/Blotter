import {
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
  type CSSProperties,
  type MouseEvent,
} from 'react';
import {
  addDays,
  differenceInCalendarDays,
  format,
  getISOWeek,
  isSameDay,
  isWeekend,
  parseISO,
  startOfWeek,
} from 'date-fns';
import { ArrowsClockwise, PushPin } from '@phosphor-icons/react';
import { withFocusedItem } from '@/actions/itemActions';
import { startPointerDrag, useDrag } from '@/app/drag';
import { useFocus, useScrollFocusIntoView, useViewController } from '@/app/focus';
import { useNav } from '@/app/nav';
import { useOverlay } from '@/app/overlay';
import { DropPreview } from '@/components/DropPreview';
import { ErrorPanel } from '@/components/EmptyState';
import { Kbd } from '@/components/ui/kbd';
import { isOpError } from '@/lib/api';
import { blockedIntervals, windowTimes } from '@/lib/availability';
import { cn } from '@/lib/cn';
import { timeOf } from '@/lib/format';
import { moveInWeek, STEP_MIN } from '@/lib/focusNav';
import { moveByDays, moveByMinutes, resizeByMinutes } from '@/lib/itemMoves';
import { layoutAllDay, layoutDay, type Placed } from '@/lib/layout';
import { isoDate, WEEK_STARTS_ON } from '@/lib/period';
import { useOp } from '@/lib/queries';
import { itemColor } from '@/lib/tags';
import type { Agenda, Item } from '@/lib/types';
import { useBlockoutWindows } from '@/lib/useBlockoutWindows';
import { useNow } from '@/lib/useNow';

/** Height of one hour in the grid (design reference: 44px). */
const HOUR_PX = 44;
const GRID_PX = 24 * HOUR_PX;
const MS_PER_MIN = 60_000;
/** Hour the grid scrolls to when the week opens, unless "now" is later. */
const DEFAULT_SCROLL_HOUR = 7;

const minutesToPx = (min: number) => (min / 60) * HOUR_PX;
/** The grid cursor moves and sizes in half hours, as in the design. */
const CURSOR_MIN = 30;
const hhmm = (min: number) =>
  `${String(Math.floor(min / 60)).padStart(2, '0')}:${String(min % 60).padStart(2, '0')}`;

/** Minutes of `iso` after the start of `day`. */
function minutesInto(day: Date, iso: string): number {
  return (parseISO(iso).getTime() - day.getTime()) / MS_PER_MIN;
}

const DAY_MIN = 24 * 60;

/** Whether [start, end) touches the day; zero-length items count at their instant. */
function onDay(start: number, end: number): boolean {
  return start < DAY_MIN && (end > 0 || (end === start && start >= 0));
}

function isOverdue(item: Item, now: Date): boolean {
  if (item.status !== 'open' || item.kind !== 'task') return false;
  const end = item.end_at ?? item.due_at;
  return end !== null && parseISO(end) < now;
}

function EventBlock({
  item,
  placed,
  now,
  focused,
  pinned,
  onSelect,
}: {
  item: Item;
  placed: Placed;
  now: Date;
  focused: boolean;
  /** A fixed item sitting inside a block-out. */
  pinned: boolean;
  onSelect: (id: string) => void;
}) {
  const done = item.status === 'done';
  const color = itemColor(item);
  const height = Math.max(minutesToPx(placed.end - placed.start) - 2, 17);
  const drag = useDrag((s) => (s.item?.id === item.id ? s.mode : null));
  const compact = height < 30;
  const showTime = height > 34;
  const range =
    item.start_at && item.end_at ? `${timeOf(item.start_at)}–${timeOf(item.end_at)}` : '';
  const style: CSSProperties = {
    top: minutesToPx(placed.start) + 1,
    height,
    left: `calc(${(placed.lane / placed.lanes) * 100}% + ${placed.lane === 0 ? 3 : 1}px)`,
    width: `calc(${100 / placed.lanes}% - ${placed.lanes > 1 ? 4 : 6}px)`,
    background: `color-mix(in srgb, ${color} ${done ? 7 : 17}%, var(--color-bg))`,
    borderLeft: `3px solid ${color}`,
    lineHeight: compact ? `${height - 2}px` : 1.3,
  };

  return (
    <div
      title={`${item.title}${range ? ` · ${range}` : ''}`}
      data-focus-id={item.id}
      aria-selected={focused}
      onClick={(e) => {
        e.stopPropagation();
        onSelect(item.id);
      }}
      onDoubleClick={() => useOverlay.getState().openItem({ id: item.id })}
      onPointerDown={(e) => {
        const r = e.currentTarget.getBoundingClientRect();
        startPointerDrag(e, item, { grabMin: ((e.clientY - r.top) / HOUR_PX) * 60 });
      }}
      style={style}
      className={cn(
        'absolute cursor-grab rounded-sm',
        drag && 'opacity-45',
        focused ? 'z-5 outline-2 outline-offset-1 outline-focus-ring' : 'overflow-hidden',
        compact
          ? 'z-3 flex flex-row items-center gap-1 px-[5px] text-[10.5px]'
          : 'z-2 flex flex-col gap-px px-1.5 py-[3px] text-xs',
      )}
    >
      <span className="flex min-w-0 items-center gap-1">
        <span
          className={cn(
            'truncate font-medium',
            done
              ? 'text-text-muted line-through'
              : isOverdue(item, now)
                ? 'text-danger'
                : 'text-text',
          )}
        >
          {item.title}
        </span>
        {item.rrule && <ArrowsClockwise size={10} className="flex-none text-text-muted" />}
        {pinned && (
          <PushPin
            size={10}
            weight="fill"
            aria-label="Fixed inside block-out"
            className="ml-auto flex-none text-text-muted"
          />
        )}
      </span>
      {showTime && (
        <span className="truncate font-mono text-[10px] text-text-muted">
          {range}
          {done && item.completed_at && `  ✓ ${timeOf(item.completed_at)}`}
        </span>
      )}
      {item.start_at && !compact && (
        <span
          aria-hidden
          title="Drag to resize (or Alt+↑/↓)"
          onPointerDown={(e) => startPointerDrag(e, item, { mode: 'resize' })}
          className="absolute inset-x-0 bottom-0 h-1.5 cursor-ns-resize"
        />
      )}
      {focused && (
        <span
          aria-hidden
          title="Resize (Alt+↑/↓)"
          className="absolute -bottom-1 left-1/2 h-1 w-[18px] -translate-x-1/2 rounded-[2px] bg-focus-ring"
        />
      )}
    </div>
  );
}

export function WeekView() {
  const anchor = useNav((s) => s.anchor);
  const now = useNow();
  const days = useMemo(() => {
    const first = startOfWeek(anchor, { weekStartsOn: WEEK_STARTS_ON });
    return Array.from({ length: 7 }, (_, i) => addDays(first, i));
  }, [anchor]);
  const first = days[0] as Date;
  const agenda = useOp<Agenda>('list_agenda', {
    start: isoDate(first),
    end: isoDate(days[6] as Date),
  });
  const items = useMemo(() => agenda.data?.items ?? [], [agenda.data]);
  const windows = useBlockoutWindows();
  const byId = useMemo(() => new Map(items.map((i) => [i.id, i])), [items]);

  const allDay = useMemo(
    () =>
      layoutAllDay(
        items
          .filter((i) => i.all_day && i.start_date)
          .map((i) => {
            const firstIdx = differenceInCalendarDays(parseISO(i.start_date as string), first);
            const lastIdx = i.end_date
              ? differenceInCalendarDays(parseISO(i.end_date), first)
              : firstIdx;
            return { id: i.id, first: firstIdx, last: lastIdx };
          }),
        7,
      ),
    [items, first],
  );
  const allDayRows = Math.max(1, ...allDay.map((p) => p.row + 1));

  const columns = useMemo(
    () =>
      days.map((day) => {
        const spans = items
          .filter((i) => !i.all_day && i.start_at)
          .map((i) => ({
            id: i.id,
            start: minutesInto(day, i.start_at as string),
            end: minutesInto(day, i.end_at ?? (i.start_at as string)),
          }))
          .filter((s) => onDay(s.start, s.end))
          .map((s) => ({ ...s, start: Math.max(0, s.start), end: Math.min(DAY_MIN, s.end) }));
        return { day, placed: layoutDay(spans), blocks: blockedIntervals(windows, day) };
      }),
    [days, items, windows],
  );

  // Open each week scrolled to the morning, or to just before now.
  const scrollRef = useRef<HTMLDivElement>(null);
  const weekKey = first.getTime();
  useLayoutEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    const nowHour = new Date().getHours() + new Date().getMinutes() / 60;
    const showingToday = days.some((d) => isSameDay(d, new Date()));
    const hour = showingToday ? Math.max(DEFAULT_SCROLL_HOUR, nowHour - 3) : DEFAULT_SCROLL_HOUR;
    el.scrollTop = hour * HOUR_PX - 4;
    // Only when the week changes, not on every refetch.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [weekKey]);

  // Keyboard focus: an item, or (after Tab) a half-hour cursor on the grid.
  const focusedId = useFocus((s) => s.itemId);
  const [mode, setMode] = useState<'item' | 'cell'>('item');
  const [cursor, setCursor] = useState({ day: 0, minute: 9 * 60 });
  useScrollFocusIntoView(mode === 'cell' ? `cursor-${cursor.day}-${cursor.minute}` : focusedId);
  const timedByDay = columns.map(({ placed }) => placed.map((p) => ({ id: p.id, start: p.start })));
  const clampMinute = (m: number) => Math.max(0, Math.min(DAY_MIN - CURSOR_MIN, m));
  // While the cursor is active the item focus is parked, so Enter, x and Del
  // never act on an item you can't see; Tab back restores it.
  const parkedFocus = useRef<string | null>(null);
  const startCursorAt = (day: number, minute: number) => {
    setCursor({ day, minute: clampMinute(minute) });
    if (mode !== 'cell') parkedFocus.current = useFocus.getState().itemId;
    useFocus.getState().setItem(null);
    setMode('cell');
  };
  const todayColumn = () =>
    Math.max(
      0,
      days.findIndex((d) => isSameDay(d, new Date())),
    );

  useViewController({
    move: (dir) => {
      if (mode === 'cell') {
        const dDay = dir === 'right' ? 1 : dir === 'left' ? -1 : 0;
        const dMin = dir === 'down' ? CURSOR_MIN : dir === 'up' ? -CURSOR_MIN : 0;
        setCursor((c) => ({
          day: Math.max(0, Math.min(6, c.day + dDay)),
          minute: clampMinute(c.minute + dMin),
        }));
        return;
      }
      const next = moveInWeek(timedByDay, focusedId, dir);
      if (next) {
        useFocus.getState().setItem(next);
        setMode('item');
      } else startCursorAt(todayColumn(), 9 * 60);
    },
    shift: (dir) => {
      if (mode !== 'item') return;
      withFocusedItem((item) =>
        dir === 'left' || dir === 'right'
          ? moveByDays(item, dir === 'right' ? 1 : -1)
          : moveByMinutes(item, dir === 'down' ? STEP_MIN : -STEP_MIN),
      );
    },
    resize: (dir) => {
      if (mode === 'item')
        withFocusedItem((item) => resizeByMinutes(item, dir === 'down' ? STEP_MIN : -STEP_MIN));
    },
    tab: () => {
      if (mode === 'cell') {
        setMode('item');
        useFocus.getState().setItem(parkedFocus.current);
        return;
      }
      // Start the cursor on the focused item, or at the next half hour today.
      const col = columns.findIndex((c) => c.placed.some((p) => p.id === focusedId));
      const p = columns[col]?.placed.find((x) => x.id === focusedId);
      if (p) {
        startCursorAt(col, Math.floor(p.start / CURSOR_MIN) * CURSOR_MIN);
      } else {
        const n = new Date();
        const nowMin = n.getHours() * 60 + n.getMinutes();
        startCursorAt(todayColumn(), Math.ceil(nowMin / CURSOR_MIN) * CURSOR_MIN);
      }
    },
    enter: () => {
      if (mode !== 'cell') return false;
      useOverlay.getState().openItem({
        defaults: {
          date: isoDate(days[cursor.day] as Date),
          time: hhmm(cursor.minute),
          duration: '30m',
        },
      });
      return true;
    },
    day: () => {
      if (mode === 'cell') return days[cursor.day] ?? null;
      const start = focusedId ? byId.get(focusedId)?.start_at : null;
      if (start) return parseISO(start);
      return days.find((d) => isSameDay(d, new Date())) ?? first;
    },
  });

  const selectItem = (id: string) => {
    setMode('item');
    useFocus.getState().setItem(id);
  };
  const onColumnClick = (dayIdx: number) => (e: MouseEvent<HTMLDivElement>) => {
    const y = e.clientY - e.currentTarget.getBoundingClientRect().top;
    startCursorAt(dayIdx, Math.floor(((y / HOUR_PX) * 60) / CURSOR_MIN) * CURSOR_MIN);
    parkedFocus.current = null;
  };
  const focusedItem = focusedId ? byId.get(focusedId) : undefined;
  let status = '';
  if (mode === 'cell') {
    status = `cursor: ${format(days[cursor.day] as Date, 'EEE')} ${hhmm(cursor.minute)}`;
  } else if (focusedItem) {
    const when =
      focusedItem.start_at && focusedItem.end_at
        ? ` · ${format(parseISO(focusedItem.start_at), 'EEE')} ${timeOf(focusedItem.start_at)}–${timeOf(focusedItem.end_at)}`
        : '';
    status = `focus: ${focusedItem.title}${when}`;
  }

  const todayIdx = days.findIndex((d) => isSameDay(d, now));
  const nowTop = minutesToPx(now.getHours() * 60 + now.getMinutes());
  const grid = 'grid grid-cols-[56px_repeat(7,minmax(0,1fr))]';

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div ref={scrollRef} data-autoscroll className="relative min-h-0 flex-1 overflow-auto">
        {agenda.isError && (
          <div className="absolute top-3 left-1/2 z-20 w-[420px] -translate-x-1/2">
            <ErrorPanel
              title="Couldn't load this week"
              message={isOpError(agenda.error) ? agenda.error.message : String(agenda.error)}
              onRetry={() => void agenda.refetch()}
            />
          </div>
        )}

        {/* Day headers and the all-day lane stay put while the grid scrolls. */}
        <div className="sticky top-0 z-10 bg-bg">
          <div className={cn(grid, 'border-b border-border')}>
            <div className="pt-3.5 pr-2 text-right font-mono text-[10px] text-text-muted">
              W{getISOWeek(first)}
            </div>
            {days.map((d, i) => {
              const today = i === todayIdx;
              return (
                <div
                  key={i}
                  className="flex items-center gap-2 border-l border-border px-2.5 pt-2.5 pb-2"
                >
                  <span
                    className={cn(
                      'text-sm font-medium',
                      today ? 'text-accent-text' : 'text-text-muted',
                    )}
                  >
                    {format(d, 'EEE')}
                  </span>
                  <span
                    className={cn(
                      'text-base',
                      today
                        ? 'grid h-6 min-w-6 place-items-center rounded-full bg-accent px-1 font-semibold text-on-accent'
                        : 'font-medium',
                    )}
                  >
                    {format(d, 'd')}
                  </span>
                </div>
              );
            })}
          </div>
          <div
            className={cn(grid, 'gap-y-0.5 border-b border-border bg-surface py-1')}
            style={{ gridTemplateRows: `repeat(${allDayRows}, 24px)` }}
          >
            <div className="row-start-1 pt-[5px] pr-2 text-right text-[10px] text-text-muted">
              all-day
            </div>
            {allDay.map((p) => {
              const item = byId.get(p.id);
              if (!item) return null;
              const color = itemColor(item);
              const done = item.status === 'done';
              return (
                <div
                  key={p.id}
                  title={item.title}
                  style={{
                    gridRow: p.row + 1,
                    gridColumn: `${p.first + 2} / span ${p.last - p.first + 1}`,
                    background: `color-mix(in srgb, ${color} ${done ? 7 : 18}%, var(--color-bg))`,
                    borderLeft: `3px solid ${color}`,
                  }}
                  className={cn(
                    'mx-1 flex items-center gap-[5px] overflow-hidden rounded-sm px-1.5 text-xs',
                    done && 'text-text-muted line-through',
                  )}
                >
                  {item.rrule && <ArrowsClockwise className="flex-none text-text-muted" />}
                  <span className="truncate">{item.title}</span>
                </div>
              );
            })}
          </div>
        </div>

        <div className={cn(grid, 'relative')} style={{ height: GRID_PX }}>
          <div className="relative">
            {Array.from({ length: 23 }, (_, h) => h + 1).map((h) => (
              <span
                key={h}
                className="absolute right-2 -mt-1.5 font-mono text-[10px] text-text-muted"
                style={{ top: h * HOUR_PX }}
              >
                {String(h).padStart(2, '0')}:00
              </span>
            ))}
          </div>

          {columns.map(({ day, placed, blocks }, i) => (
            <div
              key={i}
              onClick={onColumnClick(i)}
              data-drop="grid"
              data-scope="week"
              data-date={isoDate(day)}
              data-hour-px={HOUR_PX}
              className={cn(
                'relative border-l border-border',
                i === todayIdx
                  ? 'bg-[color-mix(in_srgb,var(--color-accent)_4%,var(--color-bg))]'
                  : isWeekend(day) && 'bg-surface',
              )}
            >
              {Array.from({ length: 24 }, (_, h) => (
                <div key={h}>
                  {h > 0 && (
                    <div
                      className="absolute inset-x-0 border-t border-border"
                      style={{ top: h * HOUR_PX }}
                    />
                  )}
                  <div
                    className="absolute inset-x-0 border-t border-dashed border-[color-mix(in_srgb,var(--color-border)_55%,transparent)]"
                    style={{ top: h * HOUR_PX + HOUR_PX / 2 }}
                  />
                </div>
              ))}
              {blocks.map((b) => (
                <div
                  key={`${b.window.blockout_id}-${b.start}`}
                  title={`${b.window.label} ${windowTimes(b.window)} (block-out)`}
                  className="pointer-events-none absolute inset-x-0 border-y border-[color-mix(in_srgb,var(--color-blockout)_50%,transparent)] bg-hatch"
                  style={{ top: minutesToPx(b.start), height: minutesToPx(b.end - b.start) }}
                >
                  {b.end - b.start >= 45 && (
                    <span className="absolute top-1 right-1 rounded-[3px] bg-bg px-1 font-mono text-[10px] text-text-muted">
                      {b.window.label}
                    </span>
                  )}
                </div>
              ))}
              {placed.map((p) => {
                const item = byId.get(p.id);
                return item ? (
                  <EventBlock
                    key={p.id}
                    item={item}
                    placed={p}
                    now={now}
                    focused={mode === 'item' && focusedId === p.id}
                    pinned={
                      item.anchoring === 'fixed' &&
                      blocks.some((b) => p.start < b.end && p.end > b.start)
                    }
                    onSelect={selectItem}
                  />
                ) : null;
              })}
              <DropPreview scope="week" date={day} hourPx={HOUR_PX} className="inset-x-0.5" />
              {mode === 'cell' && cursor.day === i && (
                <div
                  data-focus-id={`cursor-${i}-${cursor.minute}`}
                  className="pointer-events-none absolute inset-x-0.5 z-6 flex items-center rounded-sm border-2 border-focus-ring bg-accent-tint px-1.5 font-mono text-[10px] text-accent-text"
                  style={{
                    top: minutesToPx(cursor.minute) + 1,
                    height: minutesToPx(CURSOR_MIN) - 2,
                  }}
                >
                  {hhmm(cursor.minute)} · ↵ to add
                </div>
              )}
              {i === todayIdx && (
                <div
                  aria-hidden
                  className="pointer-events-none absolute right-0 -left-1 z-4 flex items-center"
                  style={{ top: nowTop - 4 }}
                >
                  <span className="size-2 rounded-full bg-danger" />
                  <span className="h-0.5 flex-1 bg-danger" />
                </div>
              )}
            </div>
          ))}

          {todayIdx >= 0 && (
            <div
              className="pointer-events-none absolute left-2 z-4 rounded-[3px] bg-danger px-1 font-mono text-[10px] text-on-accent"
              style={{ top: nowTop - 7 }}
            >
              {format(now, 'HH:mm')}
            </div>
          )}
        </div>
      </div>
      <div className="flex flex-none items-center gap-4 border-t border-border px-3.5 py-1.5 text-xs text-text-muted">
        <span className="flex items-center gap-1.5">
          <Kbd>←↑↓→</Kbd>move focus
        </span>
        <span className="flex items-center gap-1.5">
          <Kbd>Shift</Kbd>+arrows move item
        </span>
        <span className="flex items-center gap-1.5">
          <Kbd>Alt</Kbd>+arrows resize
        </span>
        <span className="flex items-center gap-1.5">
          <Kbd>Tab</Kbd>cell ↔ item
        </span>
        <span className="flex-1" />
        <span className="truncate font-mono">{status}</span>
      </div>
    </div>
  );
}
