import { useLayoutEffect, useRef } from 'react';
import { format, isSameDay, parseISO } from 'date-fns';
import { Clock, PushPin } from '@phosphor-icons/react';
import { startPointerDrag } from '@/app/drag';
import { useOverlay } from '@/app/overlay';
import { DropPreview } from '@/components/DropPreview';
import { isoDate } from '@/lib/period';
import { blockedIntervals } from '@/lib/availability';
import { cn } from '@/lib/cn';
import { layoutDay } from '@/lib/layout';
import type { Blockout, Item } from '@/lib/types';
import { itemColor } from '@/lib/tags';
import { useNow } from '@/lib/useNow';

/** Height of an hour on the rail (design: 34px). */
const HOUR_PX = 34;
const GUTTER_PX = 44;
const MS_PER_MIN = 60_000;
const px = (min: number) => (min / 60) * HOUR_PX;

/** Horizontal position inside the rail for an item's overlap lane. */
function laneStyle(p: { lane: number; lanes: number } | undefined) {
  const lane = p?.lane ?? 0;
  const count = p?.lanes ?? 1;
  return {
    left: `calc(${GUTTER_PX + 4}px + (100% - ${GUTTER_PX + 8}px) * ${lane / count})`,
    width: `calc((100% - ${GUTTER_PX + 8}px) / ${count} - ${count > 1 ? 2 : 0}px)`,
  };
}

/** The docked timeline for one day: hour grid, block-outs, timed items, now-line. */
export function TimelineRail({
  day,
  items,
  blockouts,
}: {
  day: Date;
  items: Item[];
  blockouts: Blockout[];
}) {
  const now = useNow();
  const scrollRef = useRef<HTMLDivElement>(null);
  const isToday = isSameDay(day, now);
  const nowMin = now.getHours() * 60 + now.getMinutes();

  // Open at 07:00, or a couple of hours before now on today's rail.
  const dayKey = day.getTime();
  useLayoutEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    const n = new Date();
    const startMin = isSameDay(day, n)
      ? Math.max(7 * 60, n.getHours() * 60 + n.getMinutes() - 120)
      : 7 * 60;
    el.scrollTop = px(startMin) - 8;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [dayKey]);

  const blocks = blockedIntervals(blockouts, day);
  const timed = items
    .filter((i) => !i.all_day && i.start_at)
    .map((i) => {
      const start = (parseISO(i.start_at as string).getTime() - day.getTime()) / MS_PER_MIN;
      const end =
        (parseISO(i.end_at ?? (i.start_at as string)).getTime() - day.getTime()) / MS_PER_MIN;
      return {
        item: i,
        start: Math.max(0, start),
        end: Math.min(24 * 60, Math.max(end, start + 15)),
      };
    })
    .filter((t) => t.end > 0 && t.start < 24 * 60);
  // Overlapping items share the rail's width in lanes, as in the Week grid.
  const lanes = new Map(
    layoutDay(timed.map((t) => ({ id: t.item.id, start: t.start, end: t.end }))).map((p) => [
      p.id,
      p,
    ]),
  );
  const inBlock = (s: number, e: number) => blocks.some((b) => s >= b.start && e <= b.end);

  return (
    <aside className="flex min-h-0 w-[236px] flex-none flex-col border-l border-border bg-bg">
      <div className="flex items-center gap-2 px-3.5 pt-3.5 pb-2.5">
        <Clock className="text-text-muted" />
        <span className="font-semibold">Timeline</span>
        <span className="font-mono text-xs text-text-muted">{format(day, 'EEE d')}</span>
      </div>
      <div ref={scrollRef} data-autoscroll className="min-h-0 flex-1 overflow-auto pt-1 pr-3 pb-4">
        <div
          data-drop="grid"
          data-scope="timeline"
          data-date={isoDate(day)}
          data-hour-px={HOUR_PX}
          className="relative"
          style={{ height: 24 * HOUR_PX }}
        >
          {Array.from({ length: 24 }, (_, h) => (
            <div
              key={h}
              className="absolute inset-x-0 flex items-start"
              style={{ top: h * HOUR_PX }}
            >
              <span className="-mt-1.5 w-11 pr-2 text-right font-mono text-[10px] text-text-muted">
                {h > 0 ? `${String(h).padStart(2, '0')}:00` : ''}
              </span>
              <span className="flex-1 border-t border-border" />
            </div>
          ))}

          {blocks.map((b) => (
            <div
              key={`${b.blockout.id}-${b.start}`}
              className="absolute right-0 border-y border-[color-mix(in_srgb,var(--color-blockout)_45%,transparent)] bg-hatch"
              style={{ left: GUTTER_PX, top: px(b.start), height: px(b.end - b.start) }}
            >
              <span className="absolute top-1 right-1.5 rounded-[3px] bg-bg px-1 font-mono text-[10px] text-text-muted">
                {b.blockout.label} {b.blockout.start_time}–{b.blockout.end_time}
              </span>
            </div>
          ))}

          {timed.map(({ item, start, end }) => {
            const color = itemColor(item);
            const done = item.status === 'done';
            const pinned = item.anchoring === 'fixed' && inBlock(start, end);
            return (
              <div
                key={item.id}
                title={item.title}
                onDoubleClick={() => useOverlay.getState().openItem({ id: item.id })}
                onPointerDown={(e) => {
                  const r = e.currentTarget.getBoundingClientRect();
                  startPointerDrag(e, item, { grabMin: ((e.clientY - r.top) / HOUR_PX) * 60 });
                }}
                className="absolute flex cursor-grab items-center gap-1 overflow-hidden rounded-[3px] px-1.5 text-[10.5px]"
                style={{
                  ...laneStyle(lanes.get(item.id)),
                  top: px(start) + 1,
                  height: Math.max(px(end - start) - 2, 14),
                  background: `color-mix(in srgb, ${color} ${done ? 7 : 18}%, var(--color-bg))`,
                  borderLeft: `2px solid ${color}`,
                }}
              >
                <span
                  className={cn('truncate font-medium', done && 'text-text-muted line-through')}
                >
                  {item.title}
                </span>
                {pinned && (
                  <PushPin weight="fill" size={10} className="ml-auto flex-none text-text-muted" />
                )}
              </div>
            );
          })}

          <DropPreview scope="timeline" date={day} hourPx={HOUR_PX} className="right-1 left-12" />

          {isToday && (
            <div
              aria-hidden
              className="pointer-events-none absolute right-0 flex items-center"
              style={{ left: GUTTER_PX - 6, top: px(nowMin) - 3.5 }}
            >
              <span className="ml-[3px] size-[7px] rounded-full bg-danger" />
              <span className="h-[1.5px] flex-1 bg-danger" />
            </div>
          )}
        </div>
      </div>
    </aside>
  );
}
