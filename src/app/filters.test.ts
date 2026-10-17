import { describe, expect, it } from 'vitest';
import { isItemShown } from './filters';

describe('isItemShown', () => {
  it('hides an item only when every tag it has is hidden', () => {
    expect(isItemShown({ tags: ['Work'] }, ['work'])).toBe(false);
    expect(isItemShown({ tags: ['work', 'health'] }, ['work'])).toBe(true);
    expect(isItemShown({ tags: ['work', 'health'] }, ['work', 'health'])).toBe(false);
  });

  it('always shows untagged items', () => {
    expect(isItemShown({ tags: [] }, ['work'])).toBe(true);
  });
});
