import { isSameDay } from 'date-fns';
import { Prohibit } from '@phosphor-icons/react';
import { useDrag } from '@/app/drag';
import { cn } from '@/lib/cn';
import { hhmm } from '@/lib/dnd';

/**
 * The drop slot on a time grid (design: "Timeline slot · valid / refused"):
 * a dashed accent box with the snapped time range, or a dashed danger box
 * over hatching when a flexible item would land in a block-out.
 */
export function DropPreview({
  scope,
  date,
  hourPx,
  className,
}: {
  scope: 'week' | 'timeline';
  date: Date;
  hourPx: number;
  className?: string;
}) {
  const hover = useDrag((s) => s.hover);
  const item = useDrag((s) => s.item);
  if (!item || hover?.kind !== 'grid' || hover.scope !== scope || !isSameDay(hover.date, date))
    return null;
  const top = (hover.startMin / 60) * hourPx;
  const height = Math.max(((hover.endMin - hover.startMin) / 60) * hourPx, 14);
  const refused = hover.refused;
  return (
    <div
      aria-hidden
      className={cn(
        'pointer-events-none absolute z-6 flex items-start gap-1 overflow-hidden rounded-sm border-[1.5px] border-dashed px-1.5 font-mono text-[10px] leading-[1.25]',
        refused
          ? 'border-danger bg-hatch text-danger'
          : 'border-accent bg-accent-tint text-accent-text',
        className,
      )}
      style={{ top, height }}
    >
      {refused ? (
        <>
          <Prohibit className="mt-px flex-none" />
          <span className="truncate">{refused.label} — flexible</span>
        </>
      ) : (
        <span className="truncate">
          {hhmm(hover.startMin)}–{hhmm(Math.min(hover.endMin, 24 * 60))}
        </span>
      )}
    </div>
  );
}
