// The drag ghost and auto-scroll; the drag state machine lives in drag.ts.

import { useEffect } from 'react';
import { createPortal } from 'react-dom';
import { DotsSixVertical } from '@phosphor-icons/react';
import { cardTime } from '@/lib/format';
import type { BlockoutWindow } from '@/lib/types';
import { computeHover, useDrag } from './drag';

/** Distance from a scroll container's edge that starts auto-scrolling. */
const EDGE_PX = 48;
const SCROLL_STEP = 14;

/** Scrolls the container under the pointer while it's near an edge. */
function useAutoScroll(active: boolean) {
  useEffect(() => {
    if (!active) return;
    let frame = 0;
    const tick = () => {
      const { x, y } = useDrag.getState();
      const el = document.elementFromPoint(x, y)?.closest<HTMLElement>('[data-autoscroll]');
      if (el) {
        const r = el.getBoundingClientRect();
        const dy = y < r.top + EDGE_PX ? -SCROLL_STEP : y > r.bottom - EDGE_PX ? SCROLL_STEP : 0;
        const dx = x < r.left + EDGE_PX ? -SCROLL_STEP : x > r.right - EDGE_PX ? SCROLL_STEP : 0;
        if (dy) el.scrollTop += dy;
        if (dx) el.scrollLeft += dx;
        if (dx || dy) useDrag.setState({ hover: computeHover(x, y) });
      }
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }, [active]);
}

/** The ghost card that follows the pointer while dragging (design: "Dragging"). */
export function DragLayer({ windows }: { windows: BlockoutWindow[] }) {
  const item = useDrag((s) => s.item);
  const mode = useDrag((s) => s.mode);
  const x = useDrag((s) => s.x);
  const y = useDrag((s) => s.y);
  useAutoScroll(item !== null);
  useEffect(() => {
    useDrag.setState({ windows });
  }, [windows]);

  useEffect(() => {
    if (!item) return;
    document.body.style.cursor = mode === 'resize' ? 'ns-resize' : 'grabbing';
    document.body.style.userSelect = 'none';
    return () => {
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
    };
  }, [item, mode]);

  if (!item || mode === 'resize') return null;
  return createPortal(
    <div
      aria-hidden
      className="pointer-events-none fixed z-50 flex w-[212px] -rotate-2 flex-col gap-1 rounded-md border border-accent bg-surface-raised px-[11px] py-[9px] shadow-overlay"
      style={{ left: x + 12, top: y + 8 }}
    >
      <div className="flex items-center gap-2">
        <DotsSixVertical className="text-text-muted" />
        <span className="truncate font-medium">{item.title}</span>
      </div>
      <span className="pl-[21px] font-mono text-xs text-text-muted">{cardTime(item)}</span>
    </div>,
    document.body,
  );
}
