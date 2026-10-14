import { create } from 'zustand';

/** Modal overlays. One at a time; each traps focus and closes on Esc. */
export type Overlay = 'palette' | 'shortcuts';

interface OverlayState {
  overlay: Overlay | null;
  open: (overlay: Overlay) => void;
  /** Opens `overlay`, or closes it if it is already open. */
  toggle: (overlay: Overlay) => void;
  close: () => void;
}

export const useOverlay = create<OverlayState>((set) => ({
  overlay: null,
  open: (overlay) => set({ overlay }),
  toggle: (overlay) => set((s) => ({ overlay: s.overlay === overlay ? null : overlay })),
  close: () => set({ overlay: null }),
}));
