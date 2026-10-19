import { create } from 'zustand';

/** Text typed into a brain-dump box but not added yet. */
export type DraftBox = 'inbox' | 'rail';

interface DraftState {
  drafts: Record<DraftBox, string>;
  set: (box: DraftBox, text: string) => void;
}

/**
 * Brain-dump drafts, kept here rather than in the boxes so switching views
 * doesn't lose them. Session only: they're gone when the app quits.
 */
export const useDrafts = create<DraftState>((set) => ({
  drafts: { inbox: '', rail: '' },
  set: (box, text) => set((s) => ({ drafts: { ...s.drafts, [box]: text } })),
}));

/** `useState`-style access to one box's draft. */
export function useDraft(box: DraftBox): [string, (text: string) => void] {
  const text = useDrafts((s) => s.drafts[box]);
  const set = useDrafts((s) => s.set);
  return [text, (t: string) => set(box, t)];
}
