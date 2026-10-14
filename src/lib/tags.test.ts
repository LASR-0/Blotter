import { describe, expect, it } from 'vitest';
import { itemColor, tagColor } from './tags';

describe('tagColor', () => {
  it('uses theme tokens for known tags, case-insensitively', () => {
    expect(tagColor('health')).toBe('var(--tag-health)');
    expect(tagColor('Work')).toBe('var(--tag-work)');
  });

  it('maps unknown tags to a stable palette colour', () => {
    expect(tagColor('gardening')).toBe(tagColor('gardening'));
    expect(tagColor('gardening')).toMatch(/^var\(--tag-(work|health|home|shopping|personal)\)$/);
  });
});

describe('itemColor', () => {
  it('prefers explicit colour, then first tag, then accent', () => {
    expect(itemColor({ color: '#ff0000', tags: ['work'] })).toBe('#ff0000');
    expect(itemColor({ color: null, tags: ['home', 'work'] })).toBe('var(--tag-home)');
    expect(itemColor({ color: null, tags: [] })).toBe('var(--color-accent)');
  });
});
