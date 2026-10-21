import { describe, expect, it } from 'vitest';
import { autoTagColor, itemColor, tagColor, tagColorCss, tagVar } from './tags';

describe('autoTagColor', () => {
  it('uses theme tokens for known tags, case-insensitively', () => {
    expect(autoTagColor('health')).toBe('var(--tag-health)');
    expect(autoTagColor('Work')).toBe('var(--tag-work)');
  });

  it('maps unknown tags to a stable palette colour', () => {
    expect(autoTagColor('gardening')).toBe(autoTagColor('gardening'));
    expect(autoTagColor('gardening')).toMatch(
      /^var\(--tag-(work|health|home|shopping|personal)\)$/,
    );
  });
});

describe('tagColor', () => {
  it('reads the colour chosen in Settings first, then the automatic one', () => {
    expect(tagColor('Work')).toBe('var(--tagc-work, var(--tag-work))');
  });

  it('keeps variable names to identifier characters', () => {
    expect(tagVar('été')).toMatch(/^--tagc-u[0-9a-z]+$/);
    expect(tagVar('été')).toBe(tagVar('ÉTÉ'));
  });
});

describe('tagColorCss', () => {
  it('writes a rule only for tags with a colour', () => {
    expect(
      tagColorCss([
        { name: 'garden', items: 0, color: '#123456', shares_time: false },
        { name: 'home', items: 3, color: null, shares_time: false },
      ]),
    ).toBe(':root { --tagc-garden: #123456; }');
    expect(tagColorCss([])).toBe('');
  });
});

describe('itemColor', () => {
  it('prefers explicit colour, then first tag, then accent', () => {
    expect(itemColor({ color: '#ff0000', tags: ['work'] })).toBe('#ff0000');
    expect(itemColor({ color: null, tags: ['home', 'work'] })).toBe(
      'var(--tagc-home, var(--tag-home))',
    );
    expect(itemColor({ color: null, tags: [] })).toBe('var(--color-accent)');
  });
});
