// Pointer-driven drag and drop. A drag starts after a small movement (so
// clicks and double clicks still work), hit-tests drop zones marked with
// data-drop attributes, and on release turns the drop into one operation.
//
// Drop zones:
//   data-drop="grid"  data-date="YYYY-MM-DD" data-hour-px="44" data-scope="week|timeline"
//   data-drop="day"   data-date="YYYY-MM-DD"   (cards inside carry data-card-id,
//                                               data-sort and data-untimed)
//   data-drop="inbox"
// Scroll containers that should follow the pointer carry data-autoscroll.

import type { PointerEvent as ReactPointerEvent } from 'react';
import { create } from 'zustand';
import { format, parseISO } from 'date-fns';
import { runOp } from '@/lib/api';
import {
  between,
  dropCall,
  hhmm,
  itemLength,
  refusedBy,
  resizeEnd,
  slotStart,
  type Drop,
} from '@/lib/dnd';
import type { BlockoutWindow, Item } from '@/lib/types';
import { useFocus } from './focus';

export type DragMode = 'move' | 'resize';

export type Hover =
  | {
      kind: 'grid';
      scope: string;
      date: Date;
      startMin: number;
      endMin: number;
      refused: BlockoutWindow | null;
    }
  | { kind: 'day'; date: Date; index: number; sortOrder?: number }
  | { kind: 'inbox' };

interface DragState {
  item: Item | null;
  mode: DragMode;
  /** Minutes into the item where it was grabbed (grid moves). */
  grabMin: number;
  x: number;
  y: number;
  hover: Hover | null;
  /** Block-out windows used to refuse flexible drops; set by the shell. */
  windows: BlockoutWindow[];
}

export const useDrag = create<DragState>(() => ({
  item: null,
  mode: 'move',
  grabMin: 0,
  x: 0,
  y: 0,
  hover: null,
  windows: [],
}));

/** Movement before a press becomes a drag. */
const THRESHOLD_PX = 4;

const parseDate = (s: string) => parseISO(s);

export function computeHover(x: number, y: number): Hover | null {
  const { item, mode, grabMin, windows } = useDrag.getState();
  if (!item) return null;

  if (mode === 'resize') {
    // Resizing always tracks the item's own column, wherever the pointer is.
    const date = item.start_at ? format(parseISO(item.start_at), 'yyyy-MM-dd') : null;
    const col = date
      ? document.querySelector<HTMLElement>(
          `[data-drop="grid"][data-scope="week"][data-date="${date}"]`,
        )
      : null;
    if (!col || !item.start_at) return null;
    const hourPx = Number(col.dataset.hourPx);
    const day = parseDate(col.dataset.date as string);
    const start = (parseISO(item.start_at).getTime() - day.getTime()) / 60_000;
    const pointerMin = ((y - col.getBoundingClientRect().top) / hourPx) * 60;
    return {
      kind: 'grid',
      scope: 'week',
      date: day,
      startMin: start,
      endMin: resizeEnd(pointerMin, start),
      refused: null,
    };
  }

  // The topmost zone under the pointer, looking through elements that sit over a
  // zone without being inside it (the Week view's all-day bars).
  let zone: HTMLElement | null = null;
  for (const el of document.elementsFromPoint(x, y)) {
    zone = el.closest<HTMLElement>('[data-drop]');
    if (zone) break;
  }
  if (!zone) return null;
  const kind = zone.dataset.drop;

  if (kind === 'inbox') return item.kind === 'task' ? { kind: 'inbox' } : null;

  if (kind === 'grid') {
    const hourPx = Number(zone.dataset.hourPx);
    const date = parseDate(zone.dataset.date as string);
    const length = itemLength(item);
    const pointerMin = ((y - zone.getBoundingClientRect().top) / hourPx) * 60;
    const startMin = slotStart(pointerMin, grabMin, length);
    const endMin = startMin + length;
    return {
      kind: 'grid',
      scope: zone.dataset.scope ?? 'week',
      date,
      startMin,
      endMin,
      refused: refusedBy(item, date, startMin, endMin, windows),
    };
  }

  if (kind === 'day') {
    const cards = [...zone.querySelectorAll<HTMLElement>('[data-card-id]')].filter(
      (c) => c.dataset.cardId !== item.id,
    );
    let index = 0;
    for (const c of cards) {
      const r = c.getBoundingClientRect();
      if (y > r.top + r.height / 2) index += 1;
    }
    const untimed = (c: HTMLElement | undefined) => c?.dataset.untimed === 'true';
    const before = cards.slice(0, index).reverse().find(untimed);
    const after = cards.slice(index).find(untimed);
    // Zones without cards (Month cells, the all-day lane) leave the order alone.
    const sortOrder = cards.length
      ? between(
          before ? Number(before.dataset.sort) : undefined,
          after ? Number(after.dataset.sort) : undefined,
        )
      : undefined;
    return { kind: 'day', date: parseDate(zone.dataset.date as string), index, sortOrder };
  }
  return null;
}

/** Whether a drop would change nothing (e.g. released where it started). */
function isNoop(item: Item, drop: Drop): boolean {
  if (drop.kind === 'slot' && item.start_at) {
    const target = `${format(drop.date, 'yyyy-MM-dd')}T${hhmm(drop.startMin)}`;
    return format(parseISO(item.start_at), "yyyy-MM-dd'T'HH:mm") === target;
  }
  if (drop.kind === 'inbox') return !item.start_at && !item.start_date;
  return false;
}

function finish(commit: boolean) {
  const { item, hover, mode } = useDrag.getState();
  useDrag.setState({ item: null, hover: null });
  if (!commit || !item || !hover) return;

  let drop: Drop;
  if (hover.kind === 'grid') {
    if (hover.refused) return; // Snaps back: flexible items never land in block-outs.
    drop =
      mode === 'resize'
        ? { kind: 'resize', date: hover.date, endMin: hover.endMin }
        : { kind: 'slot', date: hover.date, startMin: hover.startMin };
  } else if (hover.kind === 'day') {
    drop = { kind: 'day', date: hover.date, sortOrder: hover.sortOrder };
  } else {
    drop = { kind: 'inbox' };
  }
  if (isNoop(item, drop)) return;
  const call = dropCall(item, drop);
  if ('error' in call) {
    console.error(call.error);
    return;
  }
  runOp(call.name, call.args).catch((e: unknown) => console.error('Drop failed', e));
}

/** A click fires after pointerup; swallow it so a drop doesn't also click. */
function swallowNextClick() {
  const stop = (e: Event) => {
    e.stopPropagation();
    e.preventDefault();
  };
  window.addEventListener('click', stop, { capture: true, once: true });
  window.setTimeout(() => window.removeEventListener('click', stop, { capture: true }), 0);
}

/**
 * Call from a draggable's onPointerDown. `grabMin` is how far into the item
 * (in minutes) the press happened, for grid moves.
 */
export function startPointerDrag(
  e: ReactPointerEvent,
  item: Item,
  options: { mode?: DragMode; grabMin?: number } = {},
): void {
  if (e.button !== 0) return;
  // Buttons and fields inside a card keep their own behaviour.
  if ((e.target as HTMLElement).closest('button, input, textarea, select, a')) return;
  if (options.mode === 'resize') e.stopPropagation();

  const startX = e.clientX;
  const startY = e.clientY;
  let dragging = false;

  const onMove = (ev: PointerEvent) => {
    if (!dragging) {
      if (Math.hypot(ev.clientX - startX, ev.clientY - startY) < THRESHOLD_PX) return;
      dragging = true;
      useFocus.getState().setItem(item.id);
      useDrag.setState({ item, mode: options.mode ?? 'move', grabMin: options.grabMin ?? 0 });
    }
    useDrag.setState({ x: ev.clientX, y: ev.clientY, hover: computeHover(ev.clientX, ev.clientY) });
  };
  const cleanup = () => {
    window.removeEventListener('pointermove', onMove);
    window.removeEventListener('pointerup', onUp);
    window.removeEventListener('keydown', onKey, true);
  };
  const onUp = () => {
    cleanup();
    if (dragging) {
      swallowNextClick();
      finish(true);
    }
  };
  const onKey = (ev: KeyboardEvent) => {
    if (ev.key === 'Escape' && dragging) {
      ev.preventDefault();
      ev.stopPropagation();
      cleanup();
      finish(false);
    }
  };
  window.addEventListener('pointermove', onMove);
  window.addEventListener('pointerup', onUp);
  window.addEventListener('keydown', onKey, true);
}
