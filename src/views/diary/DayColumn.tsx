import { useState, type FormEvent } from 'react';
import { format } from 'date-fns';
import { Plus, SunHorizon } from '@phosphor-icons/react';
import { EmptyState } from '@/components/EmptyState';
import { ItemCard } from '@/components/ItemCard';
import { isOpError, runOp } from '@/lib/api';
import { cn } from '@/lib/cn';
import { formatDuration } from '@/lib/duration';
import { isoDate } from '@/lib/period';
import type { Item } from '@/lib/types';

export interface DayColumnProps {
  day: Date;
  today: boolean;
  items: Item[];
  /** Open tasks from earlier days, shown at the top of today's column. */
  overdue: Item[];
  plannedMin: number;
  availableMin: number;
  loading: boolean;
}

function SectionLabel({ children, danger }: { children: string; danger?: boolean }) {
  return (
    <div
      className={cn(
        'px-0.5 pt-1.5 text-xs font-medium tracking-[.06em] uppercase',
        danger ? 'text-danger' : 'text-text-muted',
      )}
    >
      {children}
    </div>
  );
}

/** Inline "Add task" at the foot of a column: creates a task on that day. */
function AddTask({ day, onError }: { day: Date; onError: (m: string) => void }) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState('');

  const submit = (e: FormEvent) => {
    e.preventDefault();
    const title = draft.trim();
    if (!title) {
      setEditing(false);
      return;
    }
    runOp('create_item', { title, date: isoDate(day) })
      .then(() => setDraft(''))
      .catch((err: unknown) => onError(isOpError(err) ? err.message : String(err)));
  };

  if (!editing) {
    return (
      <button
        type="button"
        onClick={() => setEditing(true)}
        className="flex h-[30px] flex-none cursor-pointer items-center gap-2 rounded-md px-2.5 text-left text-text-muted hover:bg-hover"
      >
        <Plus />
        Add task
      </button>
    );
  }
  return (
    <form onSubmit={submit} className="flex-none">
      <label className="flex h-[30px] items-center gap-2 rounded-md border border-accent bg-surface px-2.5 shadow-[0_0_0_3px_color-mix(in_srgb,var(--color-accent)_22%,transparent)]">
        <Plus className="text-text-muted" />
        <input
          autoFocus
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onBlur={() => !draft.trim() && setEditing(false)}
          onKeyDown={(e) => e.key === 'Escape' && setEditing(false)}
          placeholder={`Task on ${format(day, 'EEE d')}`}
          aria-label={`New task on ${format(day, 'EEEE d MMMM')}`}
          className="min-w-0 flex-1 bg-transparent outline-none"
        />
      </label>
    </form>
  );
}

export function DayColumn({
  day,
  today,
  items,
  overdue,
  plannedMin,
  availableMin,
  loading,
}: DayColumnProps) {
  const [error, setError] = useState<string | null>(null);
  const freeMin = Math.max(0, availableMin - plannedMin);
  const pct =
    availableMin > 0 ? Math.min(100, (plannedMin / availableMin) * 100) : plannedMin > 0 ? 100 : 0;

  return (
    <section
      aria-label={format(day, 'EEEE d MMMM')}
      className={cn(
        'flex min-h-0 flex-none flex-col border-r border-border',
        today
          ? 'w-[352px] bg-[color-mix(in_srgb,var(--color-surface)_40%,var(--color-bg))]'
          : 'w-[262px] bg-bg',
      )}
    >
      <header className="flex flex-col gap-2.5 px-4 pt-[18px] pb-3">
        <div className="flex items-end gap-2.5">
          <span
            className={cn(
              'font-heading leading-[0.9] font-heading-weight tracking-[-0.02em]',
              today ? 'text-[44px] text-accent-text' : 'text-[32px] text-text',
            )}
          >
            {format(day, 'd')}
          </span>
          <div className="flex flex-col pb-0.5 leading-[1.25]">
            <span className="text-md font-semibold">{format(day, 'EEEE')}</span>
            <span className="text-sm text-text-muted">
              {format(day, today ? 'MMMM yyyy' : 'MMMM')}
            </span>
          </div>
          <span className="flex-1" />
          {today && (
            <span className="rounded-full bg-accent-tint px-2 py-0.5 text-xs font-medium text-accent-text">
              Today
            </span>
          )}
        </div>
        <div className="flex flex-col gap-[5px]">
          <div
            role="meter"
            aria-label="Planned time against free time"
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={Math.round(pct)}
            className="flex h-1 overflow-hidden rounded-[2px] bg-border"
          >
            <span
              className={cn('rounded-[2px]', pct > 90 ? 'bg-danger' : 'bg-accent')}
              style={{ width: `${pct}%` }}
            />
          </div>
          <div className="flex justify-between font-mono text-xs text-text-muted">
            <span>{formatDuration(plannedMin)} planned</span>
            <span>{formatDuration(freeMin)} free</span>
          </div>
        </div>
      </header>

      <div className="flex min-h-0 flex-1 flex-col gap-2 overflow-auto px-3 pb-4">
        {loading &&
          [1, 0.7, 0.45].map((o) => (
            <div
              key={o}
              style={{ opacity: o }}
              className="h-[62px] flex-none rounded-md border border-border bg-surface"
            />
          ))}

        {overdue.length > 0 && (
          <>
            <SectionLabel danger>Overdue</SectionLabel>
            {overdue.map((item) => (
              <ItemCard key={item.id} item={item} large overdue onError={setError} />
            ))}
            {items.length > 0 && <SectionLabel>Today</SectionLabel>}
          </>
        )}

        {!loading && items.length === 0 && overdue.length === 0 && today && (
          <EmptyState
            icon={SunHorizon}
            title="Nothing planned yet"
            className="min-h-[220px] flex-none"
          >
            Add a task below, or brain-dump into the Inbox and schedule it here.
          </EmptyState>
        )}

        {items.map((item) => (
          <ItemCard key={item.id} item={item} large={today} onError={setError} />
        ))}

        <AddTask day={day} onError={setError} />
        {error && <code className="font-mono text-xs text-danger">{error}</code>}
      </div>
    </section>
  );
}
