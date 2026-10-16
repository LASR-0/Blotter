import { invoke } from '@tauri-apps/api/core';
import { create } from 'zustand';
import type { Overrides } from './overrides';

interface KeybindingState {
  /** Action id → binding (`""` unbound); missing ids use their defaults. */
  overrides: Overrides;
  /** Reads the saved overrides (call once after mount). */
  load: () => Promise<void>;
  /** Saves `next` as the whole set of overrides; takes effect at once. */
  replace: (next: Overrides) => Promise<void>;
}

export const useKeybindings = create<KeybindingState>((set, get) => ({
  overrides: {},
  load: async () => {
    set({ overrides: await invoke<Record<string, string>>('list_keybindings') });
  },
  replace: async (next) => {
    const before = get().overrides;
    // Optimistic, so a rebinding works on the very next key press.
    set({ overrides: next });
    try {
      await invoke('replace_keybindings', { bindings: next });
    } catch (e) {
      set({ overrides: before });
      throw e;
    }
  },
}));
