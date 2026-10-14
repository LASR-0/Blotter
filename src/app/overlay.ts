import { create } from 'zustand';
import type { ItemDraft } from '@/lib/itemDraft';

/** Modal overlays. One at a time; each traps focus and closes on Esc. */
export type Overlay = 'palette' | 'shortcuts' | 'item';

/** What the item dialog is editing: an existing item, or a new one with defaults. */
export type ItemTarget = { id: string } | { defaults: Partial<ItemDraft> };

interface OverlayState {
  overlay: Overlay | null;
  item: ItemTarget | null;
  open: (overlay: Exclude<Overlay, 'item'>) => void;
  /** Opens `overlay`, or closes it if it is already open. */
  toggle: (overlay: Exclude<Overlay, 'item'>) => void;
  /** Opens the item dialog on an item, or blank for a new one. */
  openItem: (target: ItemTarget) => void;
  close: () => void;
}

export const useOverlay = create<OverlayState>((set) => ({
  overlay: null,
  item: null,
  open: (overlay) => set({ overlay }),
  toggle: (overlay) => set((s) => ({ overlay: s.overlay === overlay ? null : overlay })),
  openItem: (item) => set({ overlay: 'item', item }),
  close: () => set({ overlay: null, item: null }),
}));
