import { ArrowsClockwise, Check, PushPin } from '@phosphor-icons/react';
import { useOverlay } from '@/app/overlay';
import { isOpError, runOp } from '@/lib/api';
import { cn } from '@/lib/cn';
import { cardTime, timeOf } from '@/lib/format';
import { tagColor } from '@/lib/tags';
import type { Item } from '@/lib/types';

/**
 * A Diary card (design reference §7, "Diary card states"). The box toggles
 * completion; `large` is the today column's bigger title.
 */
export function ItemCard({
  item,
  large,
  overdue,
  onError,
}: {
  item: Item;
  large?: boolean;
  overdue?: boolean;
  onError?: (message: string) => void;
}) {
  const done = item.status === 'done';
  const toggle = () =>
    runOp(done ? 'uncomplete_item' : 'complete_item', { id: item.id }).catch((e: unknown) =>
      onError?.(isOpError(e) ? e.message : String(e)),
    );
  const time = cardTime(item, overdue);

  return (
    <div
      onDoubleClick={() => useOverlay.getState().openItem({ id: item.id })}
      title="Double-click to edit"
      className={cn(
        'relative rounded-md border px-3 py-2.5',
        done
          ? 'border-border bg-transparent'
          : 'border-border bg-surface hover:border-border-strong',
      )}
    >
      <div className="flex items-start gap-2.5">
        <button
          type="button"
          role="checkbox"
          aria-checked={done}
          aria-label={done ? `Mark "${item.title}" open` : `Complete "${item.title}"`}
          title="Toggle complete"
          onClick={toggle}
          onDoubleClick={(e) => e.stopPropagation()}
          className={cn(
            'mt-px grid size-4 flex-none cursor-pointer place-items-center rounded-sm p-0 text-on-accent',
            done ? 'border border-accent bg-accent' : 'border-[1.5px] border-border-strong',
          )}
        >
          {done && <Check weight="bold" size={10} />}
        </button>
        <div className="flex min-w-0 flex-1 flex-col gap-[5px]">
          <span
            className={cn(
              'leading-[1.3] font-medium',
              large ? 'text-md' : 'text-base',
              done && 'text-text-muted line-through',
            )}
          >
            {item.title}
          </span>
          <div className="flex flex-wrap items-center gap-x-2.5 gap-y-1 text-xs text-text-muted">
            {time && <span className={cn('font-mono', overdue && 'text-danger')}>{time}</span>}
            {item.anchoring === 'fixed' && item.start_at && (
              <span title="Fixed: stays where you put it" className="flex items-center">
                <PushPin weight="fill" />
              </span>
            )}
            {item.rrule && (
              <span title="Recurring" className="flex items-center">
                <ArrowsClockwise />
              </span>
            )}
            {item.tags.map((t) => (
              <span key={t} className="flex items-center gap-1">
                <span className="size-1.5 rounded-full" style={{ background: tagColor(t) }} />
                {t}
              </span>
            ))}
          </div>
          {done && item.completed_at && (
            <span className="font-mono text-[10.5px] text-text-muted">
              ✓ {timeOf(item.completed_at)}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}
