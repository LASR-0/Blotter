import { create } from 'zustand';
import type { ItemDraft } from '@/lib/itemDraft';

/** Modal overlays. One at a time; each traps focus and closes on Esc. */
export type Overlay = 'palette' | 'shortcuts' | 'quickadd' | 'item' | 'confirm' | 'lift';

/** What the item dialog is editing: an existing item, or a new one with defaults. */
export type ItemTarget = { id: string } | { defaults: Partial<ItemDraft> };

/** A yes/no question; Enter confirms, Esc cancels. */
export interface ConfirmRequest {
  title: string;
  body?: string;
  confirmLabel: string;
  danger?: boolean;
  onConfirm: () => void | Promise<void>;
}

interface OverlayState {
  overlay: Overlay | null;
  item: ItemTarget | null;
  confirm: ConfirmRequest | null;
  /** The day the lift-block-outs dialog is for. */
  liftDate: Date | null;
  open: (overlay: 'palette' | 'shortcuts' | 'quickadd') => void;
  /** Opens `overlay`, or closes it if it is already open. */
  toggle: (overlay: 'palette' | 'shortcuts' | 'quickadd') => void;
  /** Opens the item dialog on an item, or blank for a new one. */
  openItem: (target: ItemTarget) => void;
  ask: (request: ConfirmRequest) => void;
  /** Opens the dialog for lifting block-outs on `date`. */
  openLift: (date: Date) => void;
  close: () => void;
}

export const useOverlay = create<OverlayState>((set) => ({
  overlay: null,
  item: null,
  confirm: null,
  liftDate: null,
  open: (overlay) => set({ overlay }),
  toggle: (overlay) => set((s) => ({ overlay: s.overlay === overlay ? null : overlay })),
  openItem: (item) => set({ overlay: 'item', item }),
  ask: (confirm) => set({ overlay: 'confirm', confirm }),
  openLift: (liftDate) => set({ overlay: 'lift', liftDate }),
  close: () => set({ overlay: null, item: null, confirm: null, liftDate: null }),
}));
