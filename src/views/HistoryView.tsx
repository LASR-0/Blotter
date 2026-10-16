import { useMemo, useState } from 'react';
import { addDays, format, parseISO } from 'date-fns';
import { Check, CheckCircle, MagnifyingGlass } from '@phosphor-icons/react';
import { useOverlay } from '@/app/overlay';
import { EmptyState, ErrorPanel } from '@/components/EmptyState';
import { Select } from '@/components/ui/select';
import { isOpError } from '@/lib/api';
import { cn } from '@/lib/cn';
import { clockTime } from '@/lib/format';
import { daysText, groupByDay, matches, tagsOf, type History } from '@/lib/history';
import { isoDate } from '@/lib/period';
import { useOp } from '@/lib/queries';
import { tagColor } from '@/lib/tags';

/** Ranges offered, in days back from today. */
const RANGES = [
  { value: '7', label: 'Last 7 days' },
  { value: '30', label: 'Last 30 days' },
  { value: '90', label: 'Last 90 days' },
  { value: '365', label: 'Last year' },
] as const;
type Range = (typeof RANGES)[number]['value'];

/** History (design: History View): completed items, with streaks and filters. */
export function HistoryView() {
  const [range, setRange] = useState<Range>('30');
  const [query, setQuery] = useState('');
  const [hidden, setHidden] = useState<ReadonlySet<string>>(new Set());
  const today = useMemo(() => new Date(), []);
  const history = useOp<History>('list_completions', {
    start: isoDate(addDays(today, 1 - Number(range))),
    end: isoDate(today),
  });

  const all = useMemo(() => history.data?.completions ?? [], [history.data]);
  const tags = useMemo(() => tagsOf(all), [all]);
  const shown = useMemo(() => all.filter((c) => matches(c, query, hidden)), [all, query, hidden]);
  const groups = useMemo(() => groupByDay(shown, today), [shown, today]);

  if (history.isError)
    return (
      <div className="p-8">
        <ErrorPanel
          title="Couldn't load your history"
          message={isOpError(history.error) ? history.error.message : String(history.error)}
          onRetry={() => void history.refetch()}
        />
      </div>
    );
  const h = history.data;
  const peak = Math.max(1, ...(h?.recent.map((r) => r.count) ?? [1]));
  const stats: [string, string][] = h
    ? [
        ['Current streak', daysText(h.streaks.current)],
        ['This week', String(h.this_week)],
        ['This month', String(h.this_month)],
        ['Best streak', daysText(h.streaks.best)],
      ]
    : [];

  return (
    <div data-autoscroll className="h-full overflow-auto">
      <div className="flex max-w-[880px] flex-col gap-5 px-7 pt-5 pb-10">
        <div className="flex items-baseline gap-2.5">
          <h1 className="m-0 font-heading text-xl font-heading-weight">History</h1>
          <span className="font-mono text-sm text-text-muted">completed items</span>
        </div>

        <div className="grid grid-cols-[repeat(4,minmax(0,1fr))_2fr] overflow-hidden rounded-lg border border-border">
          {stats.map(([label, value]) => (
            <div key={label} className="flex flex-col gap-1 border-r border-border px-4 py-3.5">
              <span className="text-xs text-text-muted">{label}</span>
              <span className="font-heading text-[24px] font-heading-weight tracking-[-.01em]">
                {value}
              </span>
            </div>
          ))}
          {h && (
            <div className="flex flex-col gap-1.5 px-4 py-3.5">
              <span className="text-xs text-text-muted">Last 14 days</span>
              <div className="flex min-h-[34px] flex-1 items-end gap-[3px]">
                {h.recent.map((r, i) => (
                  <span
                    key={r.date}
                    title={`${format(parseISO(r.date), 'EEE d MMM')}: ${r.count} completed`}
                    className={cn(
                      'flex-1 rounded-[2px]',
                      i === h.recent.length - 1 ? 'bg-accent' : 'bg-heat-3',
                    )}
                    style={{ height: r.count ? `${(r.count / peak) * 100}%` : 2 }}
                  />
                ))}
              </div>
            </div>
          )}
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <label className="flex h-[30px] w-[220px] items-center gap-2 rounded-md border border-border-strong bg-surface px-2.5 focus-within:border-accent">
            <MagnifyingGlass className="text-text-muted" />
            <input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search completed…"
              aria-label="Search completed"
              className="min-w-0 flex-1 bg-transparent outline-none"
            />
          </label>
          <Select
            ariaLabel="Date range"
            value={range}
            onValueChange={setRange}
            options={RANGES}
            className="min-w-[140px] bg-surface"
          />
          {tags.length > 0 && <span className="mx-1 h-[18px] w-px bg-border" />}
          {tags.map((t) => {
            const on = !hidden.has(t);
            return (
              <button
                key={t}
                type="button"
                aria-pressed={on}
                onClick={() => {
                  const next = new Set(hidden);
                  if (on) next.add(t);
                  else next.delete(t);
                  setHidden(next);
                }}
                className={cn(
                  'flex h-[26px] cursor-pointer items-center gap-1.5 rounded-full border px-2.5 text-sm',
                  on
                    ? 'border-border-strong bg-surface-raised text-text'
                    : 'border-border text-text-muted opacity-70',
                )}
              >
                <span className="size-[7px] rounded-full" style={{ background: tagColor(t) }} />
                {t}
              </button>
            );
          })}
        </div>

        {h && groups.length === 0 && (
          <EmptyState
            icon={CheckCircle}
            title={all.length ? 'Nothing matches' : 'Nothing done yet'}
          >
            {all.length
              ? 'Try another search, or show the tags you hid.'
              : 'Completed items show here with the time you finished them. Press x on a focused item to complete it.'}
          </EmptyState>
        )}

        <div className="flex flex-col gap-[18px]">
          {groups.map((g) => (
            <section
              key={g.date}
              aria-label={`${g.label}, ${g.sub}`}
              className="flex flex-col gap-0.5"
            >
              <div className="flex items-baseline gap-2 border-b border-border px-2 pb-1.5">
                <span className="font-semibold">{g.label}</span>
                <span className="font-mono text-xs text-text-muted">{g.sub}</span>
                <span className="flex-1" />
                <span className="font-mono text-xs text-text-muted">{g.items.length} done</span>
              </div>
              {g.items.map((c) => {
                const tag = c.tags[0];
                return (
                  <button
                    type="button"
                    key={c.id}
                    onClick={() => useOverlay.getState().openItem({ id: c.id })}
                    title="Open"
                    className="grid h-[34px] cursor-pointer grid-cols-[20px_minmax(0,1fr)_auto_64px] items-center gap-2.5 rounded-md px-2 text-left hover:bg-hover"
                  >
                    <span className="grid size-4 place-items-center rounded-[4px] bg-accent text-on-accent">
                      <Check weight="bold" size={10} />
                    </span>
                    <span className="flex min-w-0 items-center gap-2.5">
                      <span className="truncate text-text-muted line-through">{c.title}</span>
                      {tag && (
                        <span className="flex flex-none items-center gap-1 text-xs text-text-muted">
                          <span
                            className="size-1.5 rounded-full"
                            style={{ background: tagColor(tag) }}
                          />
                          {tag}
                        </span>
                      )}
                    </span>
                    <span
                      title={c.done_by === 'ai' ? 'Completed by the AI' : 'Completed over MCP'}
                      className={cn(
                        'rounded-[3px] border border-border px-[5px] font-mono text-[10px] text-text-muted',
                        (c.done_by === 'user' || c.done_by === 'system') && 'invisible',
                      )}
                    >
                      {c.done_by}
                    </span>
                    <span className="text-right font-mono text-sm text-text-muted">
                      ✓ {c.completed_at ? clockTime(parseISO(c.completed_at)) : ''}
                    </span>
                  </button>
                );
              })}
            </section>
          ))}
        </div>
        {h?.truncated && (
          <p className="m-0 text-sm text-text-muted">
            Showing the newest 2,000. Pick a shorter range to see the rest.
          </p>
        )}
      </div>
    </div>
  );
}
