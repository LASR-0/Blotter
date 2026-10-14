import { useState } from 'react';
import { format, isToday } from 'date-fns';
import { SunHorizon } from '@phosphor-icons/react';
import { useNav } from '@/app/nav';
import { EmptyState, ErrorPanel } from '@/components/EmptyState';
import { ItemCard } from '@/components/ItemCard';
import { isOpError } from '@/lib/api';
import { cn } from '@/lib/cn';
import { formatDuration } from '@/lib/duration';
import { isoDate } from '@/lib/period';
import { useOp } from '@/lib/queries';
import type { Agenda } from '@/lib/types';

// A single day column for now. Commit #12 builds the full Diary: several
// day columns, the time meter against free time, drag and drop and the
// timeline rail.
export function DiaryView() {
  const anchor = useNav((s) => s.anchor);
  const agenda = useOp<Agenda>('list_agenda', { start: isoDate(anchor) });
  const [error, setError] = useState<string | null>(null);
  const items = agenda.data?.items ?? [];
  const planned = items.reduce((sum, i) => sum + (i.all_day ? 0 : (i.duration_min ?? 0)), 0);
  const today = isToday(anchor);

  return (
    <div className="flex h-full flex-col overflow-auto">
      <div className="flex w-[352px] flex-col gap-2.5 px-4 pt-[18px] pb-3">
        <div className="flex items-end gap-2.5">
          <span
            className={cn(
              'text-[44px] leading-[0.9] font-semibold',
              today ? 'text-accent-text' : 'text-text',
            )}
          >
            {format(anchor, 'd')}
          </span>
          <span className="flex flex-col">
            <span className="text-md font-semibold">{format(anchor, 'EEEE')}</span>
            <span className="text-sm text-text-muted">{format(anchor, 'MMMM yyyy')}</span>
          </span>
        </div>
        <div className="flex justify-between font-mono text-xs text-text-muted">
          <span>{formatDuration(planned)} planned</span>
          <span>
            {items.length} {items.length === 1 ? 'item' : 'items'}
          </span>
        </div>
      </div>

      <div className="flex w-[352px] flex-col gap-2 px-3 pb-3">
        {agenda.isPending && (
          <>
            {[1, 0.7, 0.45].map((o) => (
              <div
                key={o}
                style={{ opacity: o }}
                className="h-[62px] rounded-md border border-border bg-surface"
              />
            ))}
            <span className="font-mono text-xs text-text-muted">Loading agenda…</span>
          </>
        )}
        {agenda.isError && (
          <ErrorPanel
            title={today ? "Couldn't load today" : "Couldn't load this day"}
            message={isOpError(agenda.error) ? agenda.error.message : String(agenda.error)}
            onRetry={() => void agenda.refetch()}
          />
        )}
        {agenda.isSuccess && items.length === 0 && (
          <EmptyState icon={SunHorizon} title="Nothing planned yet" className="min-h-[300px]">
            Add tasks to the Inbox (<span className="font-mono">i</span>) and schedule them here.
          </EmptyState>
        )}
        {items.map((item) => (
          <ItemCard key={item.id} item={item} onError={setError} />
        ))}
        {error && <code className="font-mono text-xs text-danger">{error}</code>}
      </div>
    </div>
  );
}
