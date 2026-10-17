// The theme engine's rules (roadmap §5.5): contrast checks, the shape,
// type and density choices the editor offers, and reading themes that
// come from the database or a file. Pure, so it is unit-tested.

import {
  DEFAULT_THEME_ID,
  findPreset,
  FONT_SERIF,
  FONT_UI,
  PRESETS,
  type Theme,
  type ThemeVars,
  type TokenName,
} from './presets';

/** Every token a theme sets: the default preset's, which has them all. */
const BASE: ThemeVars = (findPreset(DEFAULT_THEME_ID) ?? (PRESETS[0] as Theme)).vars;
export const TOKEN_NAMES = Object.keys(BASE) as TokenName[];

const NAMED_COLORS = [
  '--logo-accent',
  '--logo-page',
  '--logo-text',
  '--logo-shadow',
  '--titlebar',
  '--titlebar-text',
];

/** Colour tokens, edited with a picker and checked as `#rrggbb`. */
export const isColorToken = (name: string): boolean =>
  /^--(color|heat|tag)-/.test(name) || NAMED_COLORS.includes(name);

/** Tokens added after themes were first saved, and the theme's own token they started as. */
const DERIVED: readonly [TokenName, TokenName][] = [
  ['--logo-accent', '--color-accent'],
  ['--logo-text', '--color-text'],
  ['--logo-shadow', '--color-accent'],
  ['--titlebar', '--color-bg'],
  ['--titlebar-text', '--color-text'],
];

/** A user theme as stored (mirrors `store::themes::UserTheme`). */
export interface UserTheme {
  id: string;
  name: string;
  tokens: Record<string, string>;
  created_at: string;
  updated_at: string;
}

/** WCAG 2.x contrast ratio between two `#rrggbb` colours, or null. */
export function contrastRatio(a: string, b: string): number | null {
  const la = luminance(a);
  const lb = luminance(b);
  if (la === null || lb === null) return null;
  const [hi, lo] = la > lb ? [la, lb] : [lb, la];
  return (hi + 0.05) / (lo + 0.05);
}

/** AA for body text. */
export const AA_TEXT = 4.5;

export interface ContrastPair {
  fg: TokenName;
  bg: TokenName;
  label: string;
}

/** The text/background pairs the app actually draws. */
export const CONTRAST_PAIRS: readonly ContrastPair[] = [
  { fg: '--color-text', bg: '--color-bg', label: 'Text on background' },
  { fg: '--color-text', bg: '--color-surface', label: 'Text on surface' },
  { fg: '--color-text', bg: '--color-surface-raised', label: 'Text on raised surface' },
  { fg: '--color-text-muted', bg: '--color-bg', label: 'Muted text on background' },
  { fg: '--color-text-muted', bg: '--color-surface', label: 'Muted text on surface' },
  { fg: '--color-on-accent', bg: '--color-accent', label: 'Text on accent' },
  { fg: '--color-accent-text', bg: '--color-bg', label: 'Accent text on background' },
  { fg: '--color-accent-text', bg: '--color-accent-tint', label: 'Accent text on accent tint' },
  { fg: '--color-danger', bg: '--color-bg', label: 'Danger on background' },
];

export interface ContrastIssue extends ContrastPair {
  ratio: number;
}

/** Pairs below WCAG AA, worst first. */
export function contrastIssues(vars: ThemeVars): ContrastIssue[] {
  return CONTRAST_PAIRS.flatMap((p) => {
    const ratio = contrastRatio(vars[p.fg] ?? '', vars[p.bg] ?? '');
    return ratio !== null && ratio < AA_TEXT ? [{ ...p, ratio }] : [];
  }).sort((a, b) => a.ratio - b.ratio);
}

/** "3.9:1". */
export const formatRatio = (r: number): string => `${(Math.floor(r * 10) / 10).toFixed(1)}:1`;

/** A choice the editor offers: the tokens it sets. */
export interface Choice {
  id: string;
  label: string;
  vars: Partial<ThemeVars>;
}

const px = (n: number) => `${Math.round(n)}px`;

const TEXT_BASE: readonly [TokenName, number][] = [
  ['--text-xs', 11],
  ['--text-sm', 12],
  ['--text-base', 13],
  ['--text-md', 14],
  ['--text-lg', 16],
  ['--text-xl', 20],
  ['--text-display', 40],
];

export const TEXT_SIZES: readonly Choice[] = [
  ['small', 'Small', 0.92],
  ['default', 'Default', 1],
  ['large', 'Large', 1.08],
  ['larger', 'Larger', 1.16],
].map(([id, label, k]) => ({
  id: id as string,
  label: label as string,
  vars: Object.fromEntries(TEXT_BASE.map(([t, n]) => [t, px(n * (k as number))])),
}));

const SYSTEM_FONT = 'system-ui, "Segoe UI", Ubuntu, Cantarell, sans-serif';

export const UI_FONTS: readonly Choice[] = [
  { id: 'geist', label: 'Geist', vars: { '--font-ui': FONT_UI } },
  { id: 'system', label: 'System', vars: { '--font-ui': SYSTEM_FONT } },
];

/** Headings, as the editor offers them. */
export const HEADINGS: readonly Choice[] = [
  { id: 'sans', label: 'Sans', vars: { '--font-heading': FONT_UI, '--heading-weight': '600' } },
  {
    id: 'serif',
    label: 'Serif',
    vars: { '--font-heading': FONT_SERIF, '--heading-weight': '500' },
  },
];

/** Density: control height (`--density-row`). */
export const DENSITY: readonly Choice[] = [
  { id: 'compact', label: 'Compact', vars: { '--density-row': '26px' } },
  { id: 'comfortable', label: 'Comfortable', vars: { '--density-row': '30px' } },
];

/** The choice whose tokens `vars` has, or null when edited some other way. */
export function currentChoice(choices: readonly Choice[], vars: ThemeVars): string | null {
  const found = choices.find((c) =>
    Object.entries(c.vars).every(([k, v]) => vars[k as TokenName] === v),
  );
  return found?.id ?? null;
}

const HEX = /^#[0-9a-f]{6}$/i;
const UNSAFE = /[;{}<>\\]|url\(|@import|expression\(/i;

/**
 * A full set of tokens from a stored or imported theme: known tokens with
 * usable values are taken, the rest come from `base` (so a theme from an
 * older or newer Blotter still draws everything). `dropped` counts what
 * was left out.
 */
export function readTokens(
  raw: Readonly<Record<string, unknown>>,
  base: ThemeVars = BASE,
): { vars: ThemeVars; dropped: number } {
  const vars: ThemeVars = { ...base };
  let dropped = 0;
  for (const [name, value] of Object.entries(raw)) {
    const known = name in BASE;
    const ok =
      known &&
      typeof value === 'string' &&
      value.length <= 300 &&
      !UNSAFE.test(value) &&
      (!isColorToken(name) || HEX.test(value));
    if (ok) vars[name as TokenName] = (value as string).trim();
    else dropped++;
  }
  // An older theme keeps the look it was saved with, not the base's.
  for (const [token, from] of DERIVED) {
    const value = vars[from];
    if (!(token in raw) && value) vars[token] = value;
  }
  return { vars, dropped };
}

/** A stored user theme as a theme the app can apply. */
export function fromUserTheme(t: UserTheme): Theme {
  return {
    id: t.id,
    label: t.name,
    note: 'Your theme.',
    builtin: false,
    vars: readTokens(t.tokens).vars,
  };
}

/** "Dark copy", "Dark copy 2", … not clashing with `taken`. */
export function copyName(name: string, taken: readonly string[]): string {
  const base = `${name.replace(/ copy( \d+)?$/, '')} copy`;
  if (!taken.includes(base)) return base;
  for (let n = 2; ; n++) if (!taken.includes(`${base} ${n}`)) return `${base} ${n}`;
}

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

/** Corner tokens for a base radius `r` (the editor's slider, 0–12px). */
export function radiusVars(r: number): Partial<ThemeVars> {
  const n = Math.max(0, Math.min(12, Math.round(r)));
  return {
    '--radius-sm': `${Math.max(0, n - 2)}px`,
    '--radius-md': `${n}px`,
    '--radius-lg': `${n + 4}px`,
  };
}

/** The base radius a theme uses (its `--radius-md`), for the slider. */
export function baseRadius(vars: ThemeVars): number {
  const n = parseInt(vars['--radius-md'] ?? '', 10);
  return Number.isFinite(n) ? Math.max(0, Math.min(12, n)) : 6;
}
