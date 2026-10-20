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

const blotterLight: ThemeVars = {
  '--color-bg': '#fff6e5',
  '--color-surface': '#fbefd5',
  '--color-surface-raised': '#fefaec',
  '--color-hover': '#ffcfa8',
  '--color-text': '#58481d',
  '--color-text-muted': '#63461c',
  '--color-border': '#e4bf95',
  '--color-border-strong': '#d7ae7e',
  '--color-accent': '#fe8a58',
  '--color-accent-text': '#fe8a58',
  '--color-accent-tint': '#feead7',
  '--color-on-accent': '#ffffff',
  '--color-danger': '#c4323f',
  '--color-warning': '#9a6200',
  '--color-blockout': '#988b67',
  '--color-conflict': '#c25a0f',
  '--color-pencil': '#8f7a5b',
  '--color-focus-ring': '#ff7e38',
  '--heat-0': '#fff6e5',
  '--heat-1': '#fbe7da',
  '--heat-2': '#fdc086',
  '--heat-3': '#ec955b',
  '--heat-4': '#f2841c',
  '--tag-work': '#2f6fdb',
  '--tag-health': '#1a9566',
  '--tag-home': '#c4661a',
  '--tag-shopping': '#c63f7c',
  '--tag-personal': '#7655d6',
  '--tag-ics': '#13889a',
  '--logo-accent': '#fe5858',
  '--logo-page': '#ffbf52',
  '--logo-shade': '#f05656',
  '--logo-text': '#ffbf52',
  '--logo-shadow': '#fe5858',
  '--logo-outline': '#d3d3da',
  '--titlebar': '#ffcfa8',
  '--titlebar-text': '#58481d',
  '--shadow-overlay': '0 0 0 1px #d3d3da, 0 12px 32px rgba(20,20,40,.14)',
  '--font-heading': FONT_SERIF,
  '--heading-weight': '500',
  '--radius-sm': '6px',
  '--radius-md': '8px',
  '--radius-lg': '12px',
  '--text-xs': '12px',
  '--text-sm': '13px',
  '--text-base': '14px',
  '--text-md': '15px',
  '--text-lg': '17px',
  '--text-xl': '22px',
  '--text-display': '43px',
};

const blotterDark: ThemeVars = {
  '--color-bg': '#1c1814',
  '--color-surface': '#221d18',
  '--color-surface-raised': '#2a241e',
  '--color-hover': '#352d25',
  '--color-text': '#f5e9d0',
  '--color-text-muted': '#bba88a',
  '--color-border': '#3a3129',
  '--color-border-strong': '#4d4136',
  '--color-accent': '#fe8a58',
  '--color-accent-text': '#ffa77e',
  '--color-accent-tint': '#3a2a20',
  '--color-on-accent': '#1c1814',
  '--color-danger': '#e5534b',
  '--color-warning': '#e9b45c',
  '--color-blockout': '#9a8d73',
  '--color-conflict': '#f0a35e',
  '--color-pencil': '#a8987c',
  '--color-focus-ring': '#ff9a66',
  '--heat-0': '#241f1a',
  '--heat-1': '#4a3424',
  '--heat-2': '#7a4a2c',
  '--heat-3': '#c06a36',
  '--heat-4': '#fe8a58',
  '--tag-work': '#6c9cf2',
  '--tag-health': '#56c49b',
  '--tag-home': '#e5995a',
  '--tag-shopping': '#e77aa9',
  '--tag-personal': '#a48ef2',
  '--tag-ics': '#55bdca',
  '--logo-accent': '#fe5858',
  '--logo-page': '#ffbf52',
  '--logo-shade': '#f05656',
  '--logo-text': '#ffbf52',
  '--logo-shadow': '#fe5858',
  '--logo-outline': 'transparent',
  '--titlebar': '#33271e',
  '--titlebar-text': '#f5e9d0',
  '--shadow-overlay': '0 0 0 1px #4d4136, 0 12px 32px rgba(0,0,0,.5)',
  '--font-heading': FONT_SERIF,
  '--heading-weight': '500',
  '--radius-sm': '6px',
  '--radius-md': '8px',
  '--radius-lg': '12px',
  '--text-xs': '12px',
  '--text-sm': '13px',
  '--text-base': '14px',
  '--text-md': '15px',
  '--text-lg': '17px',
  '--text-xl': '22px',
  '--text-display': '43px',
};

const studioLight: ThemeVars = {
  '--color-bg': '#fdfdfc',
  '--color-surface': '#f7f7f8',
  '--color-surface-raised': '#ffffff',
  '--color-hover': '#efeff2',
  '--color-text': '#1c1d22',
  '--color-text-muted': '#5e606b',
  '--color-border': '#e5e5ea',
  '--color-border-strong': '#d3d3da',
  '--color-accent': '#ec994b',
  '--color-accent-text': '#ff7300',
  '--color-accent-tint': '#fff4e5',
  '--color-on-accent': '#ffffff',
  '--color-danger': '#c4323f',
  '--color-warning': '#9a6200',
  '--color-blockout': '#8b8e99',
  '--color-conflict': '#c25a0f',
  '--color-pencil': '#6b6e79',
  '--color-focus-ring': '#ff7e38',
  '--heat-0': '#fffafa',
  '--heat-1': '#fbe7da',
  '--heat-2': '#f5d5b8',
  '--heat-3': '#e8a87d',
  '--heat-4': '#d68f4c',
  '--tag-work': '#2f6fdb',
  '--tag-health': '#1a9566',
  '--tag-home': '#c4661a',
  '--tag-shopping': '#c63f7c',
  '--tag-personal': '#7655d6',
  '--tag-ics': '#13889a',
  '--logo-accent': '#fe5858',
  '--logo-page': '#ffbf52',
  '--logo-shade': '#f05656',
  '--logo-text': '#ffbf52',
  '--logo-shadow': '#fe5858',
  '--logo-outline': '#d3d3da',
  '--titlebar': '#efeff2',
  '--titlebar-text': '#1c1d22',
  '--shadow-overlay': '0 0 0 1px #d3d3da, 0 12px 32px rgba(20,20,40,.14)',
  '--heading-weight': '600',
};

const studioDark: ThemeVars = {
  '--color-bg': '#121317',
  '--color-surface': '#18191e',
  '--color-surface-raised': '#202127',
  '--color-hover': '#25262d',
  '--color-text': '#e6e7ea',
  '--color-text-muted': '#989aa4',
  '--color-border': '#2a2b32',
  '--color-border-strong': '#3a3b44',
  '--color-accent': '#c06611',
  '--color-accent-text': '#ff7300',
  '--color-accent-tint': '#302f32',
  '--color-on-accent': '#ffffff',
  '--color-danger': '#c4323f',
  '--color-warning': '#e9b45c',
  '--color-blockout': '#7d808c',
  '--color-conflict': '#f0a35e',
  '--color-pencil': '#a5a8b3',
  '--color-focus-ring': '#ff7e38',
  '--heat-0': '#1b1c22',
  '--heat-1': '#694235',
  '--heat-2': '#724b36',
  '--heat-3': '#be7c60',
  '--heat-4': '#fe967c',
  '--tag-work': '#6c9cf2',
  '--tag-health': '#56c49b',
  '--tag-home': '#e5995a',
  '--tag-shopping': '#e77aa9',
  '--tag-personal': '#a48ef2',
  '--tag-ics': '#55bdca',
  '--logo-accent': '#fe5858',
  '--logo-page': '#ffbf52',
  '--logo-shade': '#f05656',
  '--logo-text': '#ffbf52',
  '--logo-shadow': '#fe5858',
  '--logo-outline': 'transparent',
  '--titlebar': '#121317',
  '--titlebar-text': '#e6e7ea',
  '--shadow-overlay': '0 0 0 1px #3a3b44, 0 12px 32px rgba(0,0,0,.5)',
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
  preset(
    'blotter-light',
    'Blotter Light',
    'Warm cream, orange accent, serif headings.',
    blotterLight,
  ),
  preset(
    'blotter-dark',
    'Blotter Dark',
    'Default. Warm espresso, orange accent, serif headings.',
    blotterDark,
  ),
  preset('studio-light', 'Studio Light', 'Clean light, calendar-first.', studioLight),
  preset('studio-dark', 'Studio Dark', 'Neutral dark, one orange accent.', studioDark),
];

export const DEFAULT_THEME_ID = 'blotter-dark';

/** Ids of earlier presets, mapped to their successors so a saved choice still resolves. */
const RENAMED: Readonly<Record<string, string>> = {
  dark: 'studio-dark',
  light: 'studio-light',
  paper: 'blotter-light',
  gruvbox: 'blotter-dark',
};

export function findPreset(id: string): Theme | undefined {
  const current = RENAMED[id] ?? id;
  return PRESETS.find((t) => t.id === current);
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
  ['Logo', ['--logo-accent', '--logo-page', '--logo-shade', '--logo-text', '--logo-shadow']],
  ['Window', ['--titlebar', '--titlebar-text']],
];
