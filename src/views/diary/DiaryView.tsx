import { useMemo } from 'react';
import { addDays, isSameDay, parseISO } from 'date-fns';
import { useNav } from '@/app/nav';
import { ErrorPanel } from '@/components/EmptyState';
import { isOpError } from '@/lib/api';
import { availableMinutes } from '@/lib/availability';
import { dayKey, itemsByDay } from '@/lib/days';
import { isoDate } from '@/lib/period';
import { useOp } from '@/lib/queries';
import type { Agenda, Blockout, Item } from '@/lib/types';
import { useNow } from '@/lib/useNow';
import { DayColumn } from './DayColumn';
import { InboxRail } from './InboxRail';
import { TimelineRail } from './TimelineRail';

/** Days shown from the anchor onwards; later ones scroll horizontally. */
const DAYS = 7;

/** Timed minutes of `items` falling within `day` (cancelled items excluded). */
function plannedMinutes(items: readonly Item[], day: Date): number {
  const lo = day.getTime();
  const hi = addDays(day, 1).getTime();
  return items.reduce((sum, i) => {
    if (i.all_day || !i.start_at || !i.end_at || i.status === 'cancelled') return sum;
    const start = Math.max(lo, parseISO(i.start_at).getTime());
    const end = Math.min(hi, parseISO(i.end_at).getTime());
    return sum + Math.max(0, (end - start) / 60_000);
  }, 0);
}

/**
 * The home view: an Inbox rail, day columns starting at the anchor (today
 * first and widest), and a timeline rail for the anchor day.
 */
export function DiaryView() {
  const anchor = useNav((s) => s.anchor);
  const now = useNow(60_000);
  const days = useMemo(() => Array.from({ length: DAYS }, (_, i) => addDays(anchor, i)), [anchor]);
  const agenda = useOp<Agenda>('list_agenda', {
    start: isoDate(anchor),
    end: isoDate(days[DAYS - 1] as Date),
  });
  const overdue = useOp<Item[]>('list_overdue');
  const blockouts = useOp<Blockout[]>('list_blockouts');
  const byDay = useMemo(() => itemsByDay(agenda.data?.items ?? []), [agenda.data]);
  const scheduledIds = useMemo(
    () => new Set((agenda.data?.items ?? []).map((i) => i.id)),
    [agenda.data],
  );
  const anchorItems = byDay.get(dayKey(anchor)) ?? [];

  return (
    <div className="relative flex h-full min-h-0">
      <InboxRail />
      <div className="flex min-w-0 flex-1 overflow-x-auto overflow-y-hidden">
        {agenda.isError && (
          <div className="absolute top-16 left-1/2 z-20 w-[352px] -translate-x-1/2">
            <ErrorPanel
              title="Couldn't load the Diary"
              message={isOpError(agenda.error) ? agenda.error.message : String(agenda.error)}
              onRetry={() => void agenda.refetch()}
            />
          </div>
        )}
        {days.map((day) => {
          const isToday = isSameDay(day, now);
          const items = byDay.get(dayKey(day)) ?? [];
          return (
            <DayColumn
              key={day.getTime()}
              day={day}
              today={isToday}
              items={items}
              // Overdue tasks surface on today's column only, unless already listed there.
              overdue={isToday ? (overdue.data ?? []).filter((i) => !scheduledIds.has(i.id)) : []}
              plannedMin={plannedMinutes(items, day)}
              availableMin={availableMinutes(blockouts.data ?? [], day)}
              loading={agenda.isPending}
            />
          );
        })}
      </div>
      <TimelineRail day={anchor} items={anchorItems} blockouts={blockouts.data ?? []} />
    </div>
  );
}
