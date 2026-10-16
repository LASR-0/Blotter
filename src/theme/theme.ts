import { invoke } from '@tauri-apps/api/core';
import { create } from 'zustand';
import { DEFAULT_THEME_ID, findPreset, PRESETS, type Theme, type ThemeVars } from './presets';

const SETTING_KEY = 'theme';
const CACHE_KEY = 'blotter.theme';

export { luminance } from './engine';
import { fromUserTheme, luminance, readTokens, type UserTheme } from './engine';

/** Whether a theme's ground is dark, so native controls and scrollbars match. */
export function isDark(vars: ThemeVars): boolean {
  return (luminance(vars['--color-bg'] ?? '') ?? 0) < 0.2;
}

/** Writes every token onto the document root. */
export function applyTheme(theme: Theme, root: HTMLElement = document.documentElement): void {
  for (const [name, value] of Object.entries(theme.vars)) root.style.setProperty(name, value);
  root.style.colorScheme = isDark(theme.vars) ? 'dark' : 'light';
  root.dataset.theme = theme.id;
}

const fallback = (): Theme => findPreset(DEFAULT_THEME_ID) ?? (PRESETS[0] as Theme);

/** The theme `id` names among the presets and `user` themes, else the default. */
function resolve(id: string | null | undefined, user: readonly Theme[]): Theme {
  if (!id) return fallback();
  return findPreset(id) ?? user.find((t) => t.id === id) ?? fallback();
}

/**
 * The cached active theme, so the first paint is already right. It holds
 * the whole theme (a user theme isn't known until the database is read);
 * older builds cached just a preset id.
 */
function readCache(): Theme {
  try {
    const raw = localStorage.getItem(CACHE_KEY);
    if (!raw) return fallback();
    if (!raw.startsWith('{')) return resolve(raw, []);
    const cached = JSON.parse(raw) as Partial<Theme>;
    if (typeof cached.id !== 'string' || typeof cached.vars !== 'object' || !cached.vars)
      return fallback();
    if (findPreset(cached.id)) return resolve(cached.id, []);
    return {
      id: cached.id,
      label: String(cached.label ?? 'Theme'),
      note: '',
      builtin: false,
      vars: readTokens(cached.vars as Record<string, unknown>).vars,
    };
  } catch {
    return fallback();
  }
}

function writeCache(theme: Theme): void {
  try {
    localStorage.setItem(CACHE_KEY, JSON.stringify(theme));
  } catch {
    // Storage can be unavailable; the database copy is authoritative anyway.
  }
}

function activate(theme: Theme): void {
  applyTheme(theme);
  writeCache(theme);
}

/** A theme to save: a new one has no id yet. */
export interface ThemeDraft {
  id?: string;
  name: string;
  vars: ThemeVars;
}

interface ThemeState {
  theme: Theme;
  /** The user's own themes, after the presets. */
  userThemes: Theme[];
  /** Switches theme app-wide and persists the choice. */
  setTheme: (id: string) => void;
  /** Reads the user's themes and the persisted choice (call once after mount). */
  load: () => Promise<void>;
  /** Saves a user theme; `apply` also makes it the active one. */
  saveTheme: (draft: ThemeDraft, apply: boolean) => Promise<Theme>;
  /** Deletes a user theme; if it was active, Dark takes over. */
  deleteTheme: (id: string) => Promise<void>;
}

export const useTheme = create<ThemeState>((set, get) => ({
  theme: readCache(),
  userThemes: [],
  setTheme: (id) => {
    const theme = resolve(id, get().userThemes);
    activate(theme);
    set({ theme });
    invoke('set_setting', { key: SETTING_KEY, value: theme.id }).catch((e: unknown) =>
      console.error('Could not save theme', e),
    );
  },
  load: async () => {
    const [stored, saved] = await Promise.all([
      invoke<string | null>('get_setting', { key: SETTING_KEY }),
      invoke<UserTheme[]>('list_themes'),
    ]);
    const userThemes = saved.map(fromUserTheme);
    const theme = resolve(stored ?? get().theme.id, userThemes);
    activate(theme);
    set({ theme, userThemes });
  },
  saveTheme: async (draft, apply) => {
    const saved = fromUserTheme(
      await invoke<UserTheme>('save_theme', {
        theme: { id: draft.id ?? null, name: draft.name, tokens: draft.vars },
      }),
    );
    const others = get().userThemes.filter((t) => t.id !== saved.id);
    const userThemes = draft.id
      ? get().userThemes.map((t) => (t.id === saved.id ? saved : t))
      : [...others, saved];
    set({ userThemes });
    if (apply || get().theme.id === saved.id) {
      // Re-applied when it's the active one, so edits to it show at once.
      set({ theme: saved });
      activate(saved);
      if (apply) await invoke('set_setting', { key: SETTING_KEY, value: saved.id });
    }
    return saved;
  },
  deleteTheme: async (id) => {
    await invoke('delete_theme', { id });
    set({ userThemes: get().userThemes.filter((t) => t.id !== id) });
    if (get().theme.id === id) get().setTheme(DEFAULT_THEME_ID);
  },
}));

/** Applies the cached theme synchronously, before React renders. */
export function bootTheme(): void {
  applyTheme(useTheme.getState().theme);
}
