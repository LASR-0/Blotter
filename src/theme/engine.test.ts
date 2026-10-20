import { describe, expect, it } from 'vitest';
import {
  contrastIssues,
  contrastRatio,
  copyName,
  currentChoice,
  DENSITY,
  formatRatio,
  fromUserTheme,
  HEADINGS,
  baseRadius,
  radiusVars,
  readTokens,
  TEXT_SIZES,
  TOKEN_NAMES,
} from './engine';
import { DEFAULT_THEME_ID, findPreset, type ThemeVars } from './presets';

/** The default preset, which fills tokens a stored theme lacks. */
const base = findPreset(DEFAULT_THEME_ID)?.vars as ThemeVars;
/** Studio Dark: default shape, text size and density. */
const dark = findPreset('studio-dark')?.vars as ThemeVars;

describe('contrast', () => {
  it('measures WCAG ratios', () => {
    expect(contrastRatio('#000000', '#ffffff')).toBeCloseTo(21);
    expect(contrastRatio('#777777', '#ffffff')).toBeCloseTo(4.48, 2);
    expect(contrastRatio('red', '#ffffff')).toBeNull();
    expect(formatRatio(4.4999)).toBe('4.4:1');
  });

  it('knows which preset pairs fall under AA (Blotter Dark passes all)', () => {
    const labels = (id: string) =>
      contrastIssues(findPreset(id)?.vars as ThemeVars).map((i) => i.label);
    expect(labels('blotter-light')).toEqual([
      'Accent text on accent tint',
      'Accent text on background',
      'Text on accent',
    ]);
    expect(labels('blotter-dark')).toEqual([]);
    expect(labels('studio-light')).toEqual([
      'Text on accent',
      'Accent text on accent tint',
      'Accent text on background',
    ]);
    expect(labels('studio-dark')).toEqual(['Danger on background', 'Text on accent']);
  });

  it('warns when a draft makes text hard to read, worst first', () => {
    const issues = contrastIssues({
      ...dark,
      '--color-text-muted': '#3a3b44',
      '--color-on-accent': '#7a6ce0',
      '--color-danger': '#f07178',
    });
    expect(issues.map((i) => i.label)).toEqual([
      'Text on accent',
      'Muted text on surface',
      'Muted text on background',
    ]);
    expect(issues[0]?.ratio).toBeLessThan(issues[2]?.ratio ?? 0);
  });
});

describe('editor choices', () => {
  it('recognises the presets’ shape, type and density', () => {
    expect(baseRadius(dark)).toBe(6);
    expect(currentChoice(TEXT_SIZES, dark)).toBe('default');
    expect(currentChoice(DENSITY, dark)).toBe('comfortable');
    expect(currentChoice(HEADINGS, findPreset('blotter-light')?.vars as ThemeVars)).toBe('serif');
    expect(currentChoice(DENSITY, { ...dark, '--density-row': '34px' })).toBeNull();
  });

  it('scales every text size together', () => {
    const large = TEXT_SIZES.find((c) => c.id === 'large')?.vars;
    expect(large?.['--text-base']).toBe('14px');
    expect(large?.['--text-display']).toBe('43px');
  });
});

describe('reading themes', () => {
  it('fills missing tokens and drops unknown or unsafe ones', () => {
    const { vars, dropped } = readTokens({
      '--color-bg': '#000000',
      '--color-text': 'red',
      '--made-up': '#ffffff',
      '--font-ui': 'Comic Sans MS',
      '--shadow-overlay': 'url(https://example.com/x.png)',
    });
    expect(vars['--color-bg']).toBe('#000000');
    expect(vars['--color-text']).toBe(base['--color-text']);
    expect(vars['--font-ui']).toBe('Comic Sans MS');
    expect(vars['--shadow-overlay']).toBe(base['--shadow-overlay']);
    expect(dropped).toBe(3);
    expect(Object.keys(vars).sort()).toEqual([...TOKEN_NAMES].sort());
  });

  it('turns a stored theme into one the app applies', () => {
    const t = fromUserTheme({
      id: 'user-1',
      name: 'Night',
      tokens: { '--color-accent': '#ff0000' },
      created_at: '',
      updated_at: '',
    });
    expect(t).toMatchObject({ id: 'user-1', label: 'Night', builtin: false });
    expect(t.vars['--color-accent']).toBe('#ff0000');
  });

  it('gives a theme saved before the logo and title bar tokens its own colours', () => {
    const { vars } = readTokens({ '--color-text': '#111111', '--color-accent': '#ff0000' });
    expect(vars['--logo-text']).toBe('#111111');
    expect(vars['--logo-shadow']).toBe('#ff0000');
    expect(vars['--logo-accent']).toBe('#ff0000');
    // The shade underneath follows its own accent: 72% of #ff0000.
    expect(vars['--logo-shade']).toBe('#b80000');
    expect(vars['--titlebar-text']).toBe('#111111');
    const set = readTokens({ '--color-text': '#111111', '--logo-text': '#222222' });
    expect(set.vars['--logo-text']).toBe('#222222');
  });

  it('names copies without clashing', () => {
    expect(copyName('Dark', ['Dark'])).toBe('Dark copy');
    expect(copyName('Dark', ['Dark copy'])).toBe('Dark copy 2');
    expect(copyName('Dark copy 2', ['Dark copy', 'Dark copy 2'])).toBe('Dark copy 3');
  });
});

describe('radius slider', () => {
  it('sets small, medium and large corners from one value', () => {
    expect(radiusVars(6)).toEqual({
      '--radius-sm': '4px',
      '--radius-md': '6px',
      '--radius-lg': '10px',
    });
    expect(radiusVars(0)['--radius-sm']).toBe('0px');
    expect(radiusVars(40)['--radius-md']).toBe('12px');
    expect(baseRadius(dark)).toBe(6);
  });
});
