import { create } from 'zustand';

/** A one-line, self-dismissing message with an optional follow-up action. */
export interface NoticeContent {
  text: string;
  tone?: 'info' | 'error';
  action?: { label: string; shortcut?: string; run: () => void };
}

interface NoticeState {
  notice: (NoticeContent & { id: number }) | null;
  show: (n: NoticeContent) => void;
  clear: () => void;
}

let nextId = 0;

export const useNotice = create<NoticeState>((set) => ({
  notice: null,
  show: (n) => set({ notice: { ...n, id: ++nextId } }),
  clear: () => set({ notice: null }),
}));
