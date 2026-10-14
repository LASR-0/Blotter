import { Check } from '@phosphor-icons/react';
import { isOpError, runOp } from '@/lib/api';
import { cn } from '@/lib/cn';
import { itemMeta, timeOf } from '@/lib/format';
import type { Item } from '@/lib/types';

/**
 * A Diary card (design reference §7, "Diary card states": default, hover,
 * focus, completed). Clicking the box toggles completion.
 */
export function ItemCard({ item, onError }: { item: Item; onError?: (message: string) => void }) {
  const done = item.status === 'done';
  const toggle = () =>
    runOp(done ? 'uncomplete_item' : 'complete_item', { id: item.id }).catch((e: unknown) =>
      onError?.(isOpError(e) ? e.message : String(e)),
    );

  return (
    <div
      className={cn(
        'flex items-start gap-2.5 rounded-md border px-3 py-2.5',
        done
          ? 'border-border bg-transparent'
          : 'border-border bg-surface hover:border-border-strong hover:bg-hover',
      )}
    >
      <button
        type="button"
        role="checkbox"
        aria-checked={done}
        aria-label={done ? `Mark "${item.title}" open` : `Complete "${item.title}"`}
        onClick={toggle}
        className={cn(
          'mt-px grid size-4 flex-none cursor-pointer place-items-center rounded-sm text-on-accent',
          done ? 'border border-accent bg-accent' : 'border-[1.5px] border-border-strong',
        )}
      >
        {done && <Check weight="bold" size={10} />}
      </button>
      <div className="flex min-w-0 flex-1 flex-col gap-[5px]">
        <span className={cn('font-medium', done && 'text-text-muted line-through')}>
          {item.title}
        </span>
        <span className="font-mono text-xs text-text-muted">{itemMeta(item)}</span>
        {done && item.completed_at && (
          <span className="font-mono text-xs text-text-muted">✓ {timeOf(item.completed_at)}</span>
        )}
      </div>
    </div>
  );
}
