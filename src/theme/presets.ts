// Built-in theme presets. Every key maps 1:1 onto a CSS custom property on
// the document root. Ported from the design reference (blotter-themes.js).

export type TokenName = `--${string}`;
export type ThemeVars = Record<TokenName, string>;

export interface Theme {
  id: string;
  label: string;
  note: string;
  builtin: boolean;
  vars: ThemeVars;
}

export const FONT_UI = '"Geist Variable", "Geist", "Segoe UI", system-ui, sans-serif';
export const FONT_MONO =
  '"Geist Mono Variable", "Geist Mono", "JetBrains Mono", ui-monospace, monospace';
export const FONT_SERIF = '"Newsreader Variable", "Newsreader", Georgia, serif';
const FONT_BRAND = '"Playfair Display Variable", "Playfair Display", Georgia, serif';

const shared: ThemeVars = {
  '--font-ui': FONT_UI,
  '--font-heading': FONT_UI,
  '--font-mono': FONT_MONO,
  '--font-brand': FONT_BRAND,
  '--brand-weight': '800',
  '--radius-sm': '4px',
  '--radius-md': '6px',
  '--radius-lg': '10px',
  '--space-1': '4px',
  '--space-2': '8px',
  '--space-3': '12px',
  '--space-4': '16px',
  '--space-5': '24px',
  '--space-6': '32px',
  '--text-xs': '11px',
  '--text-sm': '12px',
  '--text-base': '13px',
  '--text-md': '14px',
  '--text-lg': '16px',
  '--text-xl': '20px',
  '--text-display': '40px',
  '--density-row': '30px',
};

const dark: ThemeVars = {
  '--color-bg': '#121317',
  '--color-surface': '#18191e',
  '--color-surface-raised': '#202127',
  '--color-hover': '#25262d',
  '--color-text': '#e6e7ea',
  '--color-text-muted': '#989aa4',
  '--color-border': '#2a2b32',
  '--color-border-strong': '#3a3b44',
  '--color-accent': '#8b7cf6',
  '--color-accent-text': '#b2a8fb',
  '--color-accent-tint': '#221f37',
  '--color-on-accent': '#ffffff',
  '--color-danger': '#c4323f',
  '--color-warning': '#e9b45c',
  '--color-blockout': '#7d808c',
  '--color-conflict': '#f0a35e',
  '--color-pencil': '#a5a8b3',
  '--color-focus-ring': '#b2a8fb',
  '--heat-0': '#1b1c22',
  '--heat-1': '#2a2648',
  '--heat-2': '#3f3672',
  '--heat-3': '#6153b3',
  '--heat-4': '#9d90fa',
  '--tag-work': '#6c9cf2',
  '--tag-health': '#56c49b',
  '--tag-home': '#e5995a',
  '--tag-shopping': '#e77aa9',
  '--tag-personal': '#a48ef2',
  '--tag-ics': '#55bdca',
  '--logo-page': '#e6e7ea',
  '--logo-outline': 'transparent',
  '--shadow-overlay': '0 0 0 1px #3a3b44, 0 12px 32px rgba(0,0,0,.5)',
  '--heading-weight': '600',
};

const light: ThemeVars = {
  '--color-bg': '#ffffff',
  '--color-surface': '#f7f7f8',
  '--color-surface-raised': '#ffffff',
  '--color-hover': '#efeff2',
  '--color-text': '#1c1d22',
  '--color-text-muted': '#5e606b',
  '--color-border': '#e5e5ea',
  '--color-border-strong': '#d3d3da',
  '--color-accent': '#b7adff',
  '--color-accent-text': '#9086fd',
  '--color-accent-tint': '#efedfd',
  '--color-on-accent': '#ffffff',
  '--color-danger': '#c4323f',
  '--color-warning': '#9a6200',
  '--color-blockout': '#8b8e99',
  '--color-conflict': '#c25a0f',
  '--color-pencil': '#6b6e79',
  '--color-focus-ring': '#a094ff',
  '--heat-0': '#f1f1f4',
  '--heat-1': '#dfdafb',
  '--heat-2': '#bbb1f4',
  '--heat-3': '#8c7ce8',
  '--heat-4': '#5b4cd6',
  '--tag-work': '#2f6fdb',
  '--tag-health': '#1a9566',
  '--tag-home': '#c4661a',
  '--tag-shopping': '#c63f7c',
  '--tag-personal': '#7655d6',
  '--tag-ics': '#13889a',
  '--logo-page': '#ffffff',
  '--logo-outline': '#d3d3da',
  '--shadow-overlay': '0 0 0 1px #d3d3da, 0 12px 32px rgba(20,20,40,.14)',
  '--heading-weight': '600',
};

const paper: ThemeVars = {
  '--color-bg': '#f5f0e5',
  '--color-surface': '#ede6d6',
  '--color-surface-raised': '#fbf8f1',
  '--color-hover': '#e6dfcd',
  '--color-text': '#28241e',
  '--color-text-muted': '#665d4f',
  '--color-border': '#dbd1bc',
  '--color-border-strong': '#c7bba2',
  '--color-accent': '#6e99de',
  '--color-accent-text': '#2b4a7d',
  '--color-accent-tint': '#e1e2e3',
  '--color-on-accent': '#fbf8f1',
  '--color-danger': '#a3352a',
  '--color-warning': '#835608',
  '--color-blockout': '#8e836d',
  '--color-conflict': '#a14f17',
  '--color-pencil': '#6e675b',
  '--color-focus-ring': '#2b4a7d',
  '--heat-0': '#ece5d4',
  '--heat-1': '#d7d5cc',
  '--heat-2': '#aeb5c1',
  '--heat-3': '#6e82a4',
  '--heat-4': '#2b4a7d',
  '--tag-work': '#3a62a7',
  '--tag-health': '#3d7b52',
  '--tag-home': '#a5622a',
  '--tag-shopping': '#a6446a',
  '--tag-personal': '#6a4e9a',
  '--tag-ics': '#2c777e',
  '--logo-page': '#fbf8f1',
  '--logo-outline': '#c7bba2',
  '--font-heading': FONT_SERIF,
  '--shadow-overlay': '0 0 0 1px #c7bba2, 0 12px 28px rgba(60,45,20,.16)',
  '--heading-weight': '500',
};

const gruvbox: ThemeVars = {
  '--color-bg': '#282828',
  '--color-surface': '#2f2d2c',
  '--color-surface-raised': '#3c3836',
  '--color-hover': '#45403d',
  '--color-text': '#ebdbb2',
  '--color-text-muted': '#bdae93',
  '--color-border': '#45403d',
  '--color-border-strong': '#5a524c',
  '--color-accent': '#fabd2f',
  '--color-accent-text': '#fabd2f',
  '--color-accent-tint': '#3d3523',
  '--color-on-accent': '#282828',
  '--color-danger': '#fb4934',
  '--color-warning': '#fe8019',
  '--color-blockout': '#a89984',
  '--color-conflict': '#fe8019',
  '--color-pencil': '#a89984',
  '--color-focus-ring': '#fabd2f',
  '--heat-0': '#32302f',
  '--heat-1': '#4a4126',
  '--heat-2': '#76601f',
  '--heat-3': '#b38a1f',
  '--heat-4': '#fabd2f',
  '--tag-work': '#83a598',
  '--tag-health': '#b8bb26',
  '--tag-home': '#fe8019',
  '--tag-shopping': '#d3869b',
  '--tag-personal': '#8ec07c',
  '--tag-ics': '#76b0b4',
  '--logo-page': '#ebdbb2',
  '--logo-outline': 'transparent',
  '--shadow-overlay': '0 0 0 1px #5a524c, 0 12px 32px rgba(0,0,0,.45)',
  '--heading-weight': '600',
};

const preset = (id: string, label: string, note: string, vars: ThemeVars): Theme => ({
  id,
  label,
  note,
  builtin: true,
  vars: { ...shared, ...vars },
});

export const PRESETS: readonly Theme[] = [
  preset('dark', 'Dark', 'Default. Neutral dark, one violet accent.', dark),
  preset('light', 'Light', 'Clean light, calendar-first.', light),
  preset('paper', 'Paper', 'Warm off-white, ink text, serif headings.', paper),
  preset('gruvbox', 'Gruvbox', 'Warm retro dark.', gruvbox),
];

export const DEFAULT_THEME_ID = 'dark';

export function findPreset(id: string): Theme | undefined {
  return PRESETS.find((t) => t.id === id);
}

/** Groups shown in the theme editor (Phase 10). */
export const TOKEN_GROUPS: readonly (readonly [string, readonly TokenName[]])[] = [
  [
    'Surfaces',
    [
      '--color-bg',
      '--color-surface',
      '--color-surface-raised',
      '--color-hover',
      '--color-border',
      '--color-border-strong',
    ],
  ],
  ['Text', ['--color-text', '--color-text-muted', '--color-on-accent']],
  [
    'Accent & focus',
    ['--color-accent', '--color-accent-text', '--color-accent-tint', '--color-focus-ring'],
  ],
  [
    'Semantic',
    ['--color-danger', '--color-warning', '--color-blockout', '--color-conflict', '--color-pencil'],
  ],
  ['Heatmap ramp', ['--heat-0', '--heat-1', '--heat-2', '--heat-3', '--heat-4']],
  [
    'Tag palette',
    ['--tag-work', '--tag-health', '--tag-home', '--tag-shopping', '--tag-personal', '--tag-ics'],
  ],
  ['Logo', ['--logo-page']],
];
