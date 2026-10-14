import { create } from 'zustand';
import { isOpError, runOp } from '@/lib/api';
import type { AutoSlotResult } from '@/lib/types';
import { useSelection } from './selection';

interface AutoSlotState {
  /** The last run's result, shown in the side panel until dismissed. */
  result: AutoSlotResult | null;
  at: Date | null;
  running: boolean;
  error: string | null;
  close: () => void;
}

export const useAutoSlot = create<AutoSlotState>((set) => ({
  result: null,
  at: null,
  running: false,
  error: null,
  close: () => set({ result: null, at: null, error: null }),
}));

/**
 * Runs `auto_slot` on `ids` (or the whole Inbox) and shows the result.
 * With `merge`, a re-run for a few tasks updates the panel already open:
 * newly placed tasks join the placed list and leave the unplaced one.
 */
export async function runAutoSlot(ids?: string[], merge = false): Promise<void> {
  useAutoSlot.setState({ running: true, error: null });
  try {
    const result = await runOp<AutoSlotResult>('auto_slot', ids ? { ids } : {});
    const prev = useAutoSlot.getState().result;
    if (merge && prev) {
      const redone = new Set([
        ...result.placed.map((p) => p.item.id),
        ...result.unplaced.map((u) => u.id),
      ]);
      useAutoSlot.setState({
        result: {
          ...result,
          placed: [...prev.placed, ...result.placed],
          unplaced: [...prev.unplaced.filter((u) => !redone.has(u.id)), ...result.unplaced],
        },
      });
    } else {
      useAutoSlot.setState({ result, at: new Date() });
    }
    useSelection.getState().clear();
  } catch (e) {
    useAutoSlot.setState({ error: isOpError(e) ? e.message : String(e) });
  } finally {
    useAutoSlot.setState({ running: false });
  }
}
