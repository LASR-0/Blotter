import { useMemo, useState } from 'react';
import {
  addDays,
  endOfYear,
  format,
  getDaysInMonth,
  isSameDay,
  isSameYear,
  startOfYear,
} from 'date-fns';
import { useViewController } from '@/app/focus';
import { useNav } from '@/app/nav';
import { ErrorPanel } from '@/components/EmptyState';
import { Kbd } from '@/components/ui/kbd';
import { Segmented } from '@/components/ui/segmented';
import { isOpError } from '@/lib/api';
import { cn } from '@/lib/cn';
import { completionsByDay, dayKey, heatLevel, itemsByDay } from '@/lib/days';
import { formatDuration } from '@/lib/duration';
import { isoDate, WEEK_STARTS_ON } from '@/lib/period';
import { useOp } from '@/lib/queries';
import type { Agenda, Item } from '@/lib/types';
import { useNow } from '@/lib/useNow';

type Mode = 'scheduled' | 'completed';

const WEEKDAY_INITIALS = ['M', 'T', 'W', 'T', 'F', 'S', 'S'];

/** Minutes of timed work in a day's items (all-day items don't count). */
function plannedMinutes(items: readonly Item[]): number {
  return items.reduce((sum, i) => sum + (i.all_day ? 0 : (i.duration_min ?? 0)), 0);
}

export function YearView() {
  const anchor = useNav((s) => s.anchor);
  const openWeek = useNav((s) => s.openWeek);
  const now = useNow(60_000);
  const [mode, setMode] = useState<Mode>('scheduled');
  const year = anchor.getFullYear();
  const agenda = useOp<Agenda>('list_agenda', {
    start: isoDate(startOfYear(anchor)),
    end: isoDate(endOfYear(anchor)),
  });
  const scheduled = useMemo(() => itemsByDay(agenda.data?.items ?? []), [agenda.data]);
  const completed = useMemo(() => completionsByDay(agenda.data?.items ?? []), [agenda.data]);
  const byDay = mode === 'scheduled' ? scheduled : completed;

  const months = useMemo(
    () =>
      Array.from({ length: 12 }, (_, m) => {
        const first = new Date(year, m, 1);
        // Leading blanks so day 1 sits under its weekday (Monday first).
        const lead = (first.getDay() - WEEK_STARTS_ON + 7) % 7;
        const days = Array.from({ length: getDaysInMonth(first) }, (_, d) => addDays(first, d));
        return { first, lead, days };
      }),
    [year],
  );

  // Keyboard focus is a day in the shown year (today when it's this year).
  const [chosen, setChosen] = useState<Date | null>(null);
  const focusDay =
    chosen && isSameYear(chosen, anchor)
      ? chosen
      : isSameYear(now, anchor)
        ? new Date(now.getFullYear(), now.getMonth(), now.getDate())
        : startOfYear(anchor);
  useViewController({
    move: (dir) => {
      const next = addDays(focusDay, { left: -1, right: 1, up: -7, down: 7 }[dir]);
      setChosen(next);
      if (!isSameYear(next, anchor)) useNav.getState().goToDate(next);
    },
    enter: () => {
      openWeek(focusDay);
      return true;
    },
    day: () => focusDay,
  });

  let total = 0;
  for (const [key, list] of byDay) if (key.startsWith(String(year))) total += list.length;

  return (
    <div className="relative flex h-full min-h-0 flex-col">
      <div className="flex items-center gap-4 border-b border-border px-5 py-2.5">
        <Segmented
          ariaLabel="Heatmap"
          value={mode}
          onChange={setMode}
          options={[
            { value: 'scheduled', label: 'Scheduled load' },
            { value: 'completed', label: 'Completed' },
          ]}
        />
        <div className="flex items-center gap-1.5 text-xs text-text-muted" aria-hidden>
          <span>Less</span>
          {[0, 1, 2, 3, 4].map((l) => (
            <span
              key={l}
              className={cn('size-3 rounded-[3px]', l === 0 && 'border border-border')}
              style={{ background: `var(--heat-${l})` }}
            />
          ))}
          <span>More</span>
        </div>
        <span className="flex-1" />
        <span className="font-mono text-xs text-text-muted">
          {total} {mode} in {year}
        </span>
      </div>

      {agenda.isError && (
        <div className="absolute top-16 left-1/2 z-20 w-[420px] -translate-x-1/2">
          <ErrorPanel
            title="Couldn't load this year"
            message={isOpError(agenda.error) ? agenda.error.message : String(agenda.error)}
            onRetry={() => void agenda.refetch()}
          />
        </div>
      )}

      <div className="grid min-h-0 flex-1 grid-cols-4 grid-rows-3 gap-x-7 gap-y-[18px] px-6 pt-[18px] pb-[22px]">
        {months.map(({ first, lead, days }) => {
          const monthCount = days.reduce((n, d) => n + (byDay.get(dayKey(d))?.length ?? 0), 0);
          const current = first.getMonth() === now.getMonth() && year === now.getFullYear();
          return (
            <section key={first.getMonth()} className="flex min-h-0 flex-col gap-1.5">
              <div className="flex items-baseline gap-2">
                <h2
                  className={cn(
                    'm-0 font-heading text-lg font-heading-weight',
                    current ? 'text-accent-text' : 'text-text',
                  )}
                >
                  {format(first, 'MMMM')}
                </h2>
                <span className="font-mono text-[10px] text-text-muted">
                  {monthCount} {mode === 'scheduled' ? 'items' : 'done'}
                </span>
              </div>
              <div className="grid min-h-0 flex-1 grid-cols-7 grid-rows-[auto_repeat(6,minmax(0,1fr))] gap-[3px]">
                {WEEKDAY_INITIALS.map((w, i) => (
                  <span key={i} className="text-center font-mono text-[9.5px] text-text-muted">
                    {w}
                  </span>
                ))}
                {Array.from({ length: lead }, (_, i) => (
                  <span key={`lead-${i}`} />
                ))}
                {days.map((d) => {
                  const items = byDay.get(dayKey(d)) ?? [];
                  const level = heatLevel(items.length);
                  const minutes = plannedMinutes(items);
                  const label = `${format(d, 'EEE d MMM')} · ${items.length} ${
                    mode === 'scheduled' ? 'items' : 'done'
                  }${mode === 'scheduled' && minutes ? `, ${formatDuration(minutes)}` : ''}`;
                  const focused = isSameDay(d, focusDay);
                  return (
                    <button
                      key={d.getDate()}
                      type="button"
                      aria-selected={focused}
                      onClick={() => (focused ? openWeek(d) : setChosen(d))}
                      title={`${label} · open in Week`}
                      aria-label={label}
                      style={{ background: `var(--heat-${level})` }}
                      className={cn(
                        'grid min-h-0 cursor-pointer place-items-center rounded-[3px] font-mono text-[10px]',
                        level >= 3 ? 'font-medium text-on-accent' : 'text-text-muted',
                        isSameDay(d, now) && 'shadow-[inset_0_0_0_1.5px_var(--color-text)]',
                        focused && 'relative z-4 outline-2 outline-offset-1 outline-focus-ring',
                      )}
                    >
                      {d.getDate()}
                      {focused && (
                        <span className="pointer-events-none absolute bottom-[calc(100%+8px)] left-1/2 z-5 flex -translate-x-1/2 items-center gap-2 rounded-md bg-surface-raised px-[9px] py-1.5 font-ui text-xs font-normal whitespace-nowrap text-text shadow-overlay">
                          <span className="font-medium">{label}</span>
                          <Kbd>↵ Week</Kbd>
                        </span>
                      )}
                    </button>
                  );
                })}
              </div>
            </section>
          );
        })}
      </div>
    </div>
  );
}
