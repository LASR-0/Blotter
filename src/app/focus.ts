import { useEffect, useLayoutEffect, useRef } from 'react';
import { create } from 'zustand';
import type { Dir } from '@/lib/focusNav';

/**
 * How the current view responds to the shared navigation keys. Each
 * navigable view registers one while it is mounted.
 */
export interface ViewController {
  /** Arrows / h j k l. */
  move(dir: Dir): void;
  /** Shift+arrows: move the focused item. */
  shift?(dir: Dir): void;
  /** Alt+↑/↓: resize the focused item. */
  resize?(dir: 'up' | 'down'): void;
  /** Tab (Week): switch between item focus and the grid cursor. */
  tab?(): void;
  /** Enter with no focused item (e.g. open a day). True if handled. */
  enter?(): boolean;
  /** The day keyboard focus is on, for day actions such as lifting block-outs. */
  day?(): Date | null;
}

interface FocusState {
  /** The keyboard-focused item, shown with the focus ring. */
  itemId: string | null;
  controller: ViewController | null;
  setItem: (id: string | null) => void;
  setController: (c: ViewController | null) => void;
}

export const useFocus = create<FocusState>((set) => ({
  itemId: null,
  controller: null,
  setItem: (itemId) => set({ itemId }),
  setController: (controller) => set({ controller }),
}));

/** Registers `controller` for the mounted view, always calling its latest version. */
export function useViewController(controller: ViewController): void {
  const latest = useRef(controller);
  useLayoutEffect(() => {
    latest.current = controller;
  });
  useEffect(() => {
    const proxy: ViewController = {
      move: (d) => latest.current.move(d),
      shift: (d) => latest.current.shift?.(d),
      resize: (d) => latest.current.resize?.(d),
      tab: () => latest.current.tab?.(),
      enter: () => latest.current.enter?.() ?? false,
      day: () => latest.current.day?.() ?? null,
    };
    useFocus.getState().setController(proxy);
    return () => {
      if (useFocus.getState().controller === proxy) useFocus.getState().setController(null);
    };
  }, []);
}

/** Scrolls the element marked `data-focus-id={id}` into view after it renders. */
export function useScrollFocusIntoView(id: string | null): void {
  useEffect(() => {
    if (!id) return;
    const el = document.querySelector(`[data-focus-id="${CSS.escape(id)}"]`);
    el?.scrollIntoView({ block: 'nearest', inline: 'nearest' });
  }, [id]);
}
