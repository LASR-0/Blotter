import { describe, expect, it } from 'vitest';
import { DEFAULT_THEME_ID, findPreset, PRESETS, TOKEN_GROUPS } from './presets';
import { isDark, luminance } from './theme';

describe('presets', () => {
  it('ship the Blotter and Studio pairs', () => {
    expect(PRESETS.map((t) => t.id)).toEqual([
      'blotter-light',
      'blotter-dark',
      'studio-light',
      'studio-dark',
    ]);
  });

  it('still find a theme saved under an earlier preset id', () => {
    expect(findPreset('dark')?.id).toBe('studio-dark');
    expect(findPreset('paper')?.id).toBe('blotter-light');
    expect(findPreset(DEFAULT_THEME_ID)?.label).toBe('Blotter Dark');
  });

  it('all define exactly the same tokens', () => {
    const keys = (i: number) => Object.keys(PRESETS[i]?.vars ?? {}).sort();
    for (let i = 1; i < PRESETS.length; i++) expect(keys(i)).toEqual(keys(0));
  });

  it('define every token the editor groups list', () => {
    for (const theme of PRESETS)
      for (const [, tokens] of TOKEN_GROUPS)
        for (const t of tokens)
          expect(theme.vars[t], `${theme.id} ${t}`).toMatch(/^#[0-9a-f]{6}$/i);
  });

  it('the Blotter pair alone use serif headings', () => {
    for (const t of PRESETS)
      expect(t.vars['--font-heading']?.includes('Newsreader')).toBe(t.id.startsWith('blotter'));
  });
});

describe('luminance', () => {
  it('matches WCAG reference points', () => {
    expect(luminance('#000000')).toBe(0);
    expect(luminance('#ffffff')).toBeCloseTo(1);
    expect(luminance('blue')).toBeNull();
  });

  it('classifies preset grounds', () => {
    const dark = Object.fromEntries(PRESETS.map((t) => [t.id, isDark(t.vars)]));
    expect(dark).toEqual({
      'blotter-light': false,
      'blotter-dark': true,
      'studio-light': false,
      'studio-dark': true,
    });
  });
});
