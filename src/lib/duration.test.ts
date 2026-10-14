import { describe, expect, it } from 'vitest';
import { formatDuration } from './duration';

describe('formatDuration', () => {
  it.each([
    [0, '0m'],
    [-5, '0m'],
    [Number.NaN, '0m'],
    [15, '15m'],
    [60, '1h'],
    [90, '1h 30m'],
    [390, '6h 30m'],
    [44.6, '45m'],
  ])('%s → %s', (input, expected) => {
    expect(formatDuration(input)).toBe(expected);
  });
});
