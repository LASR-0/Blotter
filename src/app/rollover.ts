import { create } from 'zustand';
import { isOpError, runOp } from '@/lib/api';
import type { AutoSlotResult, Item } from '@/lib/types';

/** `rollover`'s result. */
export interface RolloverResult {
  mode: 'inbox' | 'reslot';
  to_inbox: Item[];
  reslotted: AutoSlotResult['placed'];
  not_reslotted: AutoSlotResult['unplaced'];
}

interface RolloverState {
  /** The last rollover to tell the user about; a string for a manual run's message. */
  notice: RolloverResult | string | null;
  set: (notice: RolloverResult | string | null) => void;
}

export const useRollover = create<RolloverState>((set) => ({
  notice: null,
  set: (notice) => set({ notice }),
}));

/** Runs the rollover now (palette), whatever the time, and shows what moved. */
export async function rolloverNow(): Promise<void> {
  try {
    const result = await runOp<RolloverResult>('rollover');
    const moved = result.to_inbox.length + result.reslotted.length;
    useRollover
      .getState()
      .set(moved ? result : 'Nothing to roll over: no unfinished tasks on earlier days.');
  } catch (e) {
    useRollover.getState().set(isOpError(e) ? e.message : String(e));
  }
}
