import { invoke } from '@tauri-apps/api/core';
import { create } from 'zustand';
import { DEFAULT_THEME_ID, findPreset, PRESETS, type Theme, type ThemeVars } from './presets';

const SETTING_KEY = 'theme';
const CACHE_KEY = 'blotter.theme';

/** Relative luminance of a `#rrggbb` colour (WCAG 2.x), or null if unparseable. */
export function luminance(hex: string): number | null {
  const m = /^#([0-9a-f]{6})$/i.exec(hex.trim());
  if (!m?.[1]) return null;
  const n = parseInt(m[1], 16);
  const channel = (c: number) => {
    const s = c / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  };
  return (
    0.2126 * channel((n >> 16) & 255) + 0.7152 * channel((n >> 8) & 255) + 0.0722 * channel(n & 255)
  );
}

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

function resolve(id: string | null | undefined): Theme {
  return (id && findPreset(id)) || (findPreset(DEFAULT_THEME_ID) ?? (PRESETS[0] as Theme));
}

function readCache(): string | null {
  try {
    return localStorage.getItem(CACHE_KEY);
  } catch {
    return null;
  }
}

function writeCache(id: string): void {
  try {
    localStorage.setItem(CACHE_KEY, id);
  } catch {
    // Storage can be unavailable; the database copy is authoritative anyway.
  }
}

interface ThemeState {
  theme: Theme;
  /** Switches theme app-wide and persists the choice. */
  setTheme: (id: string) => void;
  /** Reconciles with the persisted setting (call once after mount). */
  load: () => Promise<void>;
}

export const useTheme = create<ThemeState>((set, get) => ({
  theme: resolve(readCache()),
  setTheme: (id) => {
    const theme = resolve(id);
    applyTheme(theme);
    writeCache(theme.id);
    set({ theme });
    invoke('set_setting', { key: SETTING_KEY, value: theme.id }).catch((e: unknown) =>
      console.error('Could not save theme', e),
    );
  },
  load: async () => {
    const stored = await invoke<string | null>('get_setting', { key: SETTING_KEY });
    if (stored && stored !== get().theme.id) {
      const theme = resolve(stored);
      applyTheme(theme);
      writeCache(theme.id);
      set({ theme });
    }
  },
}));

/** Applies the cached theme synchronously, before React renders. */
export function bootTheme(): void {
  applyTheme(useTheme.getState().theme);
}
