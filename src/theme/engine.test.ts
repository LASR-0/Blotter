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
import { findPreset, type ThemeVars } from './presets';

const dark = findPreset('dark')?.vars as ThemeVars;

describe('contrast', () => {
  it('measures WCAG ratios', () => {
    expect(contrastRatio('#000000', '#ffffff')).toBeCloseTo(21);
    expect(contrastRatio('#777777', '#ffffff')).toBeCloseTo(4.48, 2);
    expect(contrastRatio('red', '#ffffff')).toBeNull();
    expect(formatRatio(4.4999)).toBe('4.4:1');
  });

  it('passes Dark, Light and Paper; Gruvbox’s red is just under AA', () => {
    for (const id of ['dark', 'light', 'paper'])
      expect(contrastIssues(findPreset(id)?.vars as ThemeVars), id).toEqual([]);
    const gruvbox = contrastIssues(findPreset('gruvbox')?.vars as ThemeVars);
    expect(gruvbox.map((i) => i.label)).toEqual(['Danger on background']);
  });

  it('warns when a draft makes text hard to read, worst first', () => {
    const issues = contrastIssues({
      ...dark,
      '--color-text-muted': '#3a3b44',
      '--color-on-accent': '#7a6ce0',
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
    expect(currentChoice(HEADINGS, findPreset('paper')?.vars as ThemeVars)).toBe('serif');
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
    expect(vars['--color-text']).toBe(dark['--color-text']);
    expect(vars['--font-ui']).toBe('Comic Sans MS');
    expect(vars['--shadow-overlay']).toBe(dark['--shadow-overlay']);
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
