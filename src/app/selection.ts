import { create } from 'zustand';

/** Which bulk-action editor is open in the Inbox's bulk bar. */
export type BulkEditor = 'schedule' | 'duration' | 'priority' | 'due';

interface SelectionState {
  /** Selected item ids (Inbox multi-select). */
  ids: ReadonlySet<string>;
  editor: BulkEditor | null;
  toggle: (id: string) => void;
  setAll: (ids: Iterable<string>) => void;
  clear: () => void;
  openEditor: (editor: BulkEditor | null) => void;
}

export const useSelection = create<SelectionState>((set) => ({
  ids: new Set(),
  editor: null,
  toggle: (id) =>
    set((s) => {
      const ids = new Set(s.ids);
      if (ids.has(id)) ids.delete(id);
      else ids.add(id);
      return { ids, editor: ids.size ? s.editor : null };
    }),
  setAll: (ids) => set({ ids: new Set(ids) }),
  clear: () => set({ ids: new Set(), editor: null }),
  openEditor: (editor) => set((s) => ({ editor: s.ids.size ? editor : null })),
}));
