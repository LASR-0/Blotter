import { ArrowsClockwise, Check, PushPin, Warning } from '@phosphor-icons/react';
import { startPointerDrag, useDrag } from '@/app/drag';
import { useFocus } from '@/app/focus';
import { useOverlay } from '@/app/overlay';
import { isOpError, runOp } from '@/lib/api';
import { cn } from '@/lib/cn';
import { isRecurring } from '@/lib/itemDraft';
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
  conflict,
  onError,
}: {
  item: Item;
  large?: boolean;
  overdue?: boolean;
  /** A conflict line such as "Overlaps Dentist", shown in the conflict colour. */
  conflict?: string | null;
  onError?: (message: string) => void;
}) {
  const done = item.status === 'done';
  const toggle = () =>
    runOp(done ? 'uncomplete_item' : 'complete_item', { id: item.id }).catch((e: unknown) =>
      onError?.(isOpError(e) ? e.message : String(e)),
    );
  const time = cardTime(item, overdue);
  const focused = useFocus((s) => s.itemId === item.id);
  const dragged = useDrag((s) => s.item?.id === item.id && s.mode === 'move');

  return (
    <div
      data-focus-id={item.id}
      data-card-id={item.id}
      data-sort={item.sort_order}
      data-untimed={!item.start_at}
      aria-selected={focused}
      onPointerDown={(e) => startPointerDrag(e, item)}
      onClick={() => useFocus.getState().setItem(item.id)}
      onDoubleClick={() => useOverlay.getState().openItem({ id: item.id })}
      title="Double-click to edit"
      className={cn(
        'relative rounded-md border px-3 py-2.5',
        focused && 'outline-2 outline-offset-2 outline-focus-ring',
        dragged && 'border-dashed border-border-strong opacity-45',
        done
          ? 'border-border bg-transparent'
          : conflict
            ? 'border-conflict bg-surface shadow-[inset_0_0_0_1px_color-mix(in_srgb,var(--color-conflict)_25%,transparent)]'
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
          onClick={(e) => {
            e.stopPropagation();
            void toggle();
          }}
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
            {isRecurring(item) && (
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
          {conflict && !done && (
            <span className="flex items-center gap-[5px] text-[11px] text-conflict">
              <Warning weight="fill" />
              {conflict}
            </span>
          )}
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
