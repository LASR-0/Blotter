import { describe, expect, it } from 'vitest';
import { formatDuration, parseDuration } from './duration';

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

describe('parseDuration', () => {
  it.each([
    ['45', 45],
    ['45m', 45],
    ['45 min', 45],
    ['1h', 60],
    ['1h30', 90],
    ['1h 30m', 90],
    ['1.5h', 90],
    ['2 hours', 120],
    ['90minutes', 90],
  ])('%s → %s', (input, expected) => {
    expect(parseDuration(input)).toBe(expected);
  });

  it.each(['', 'soon', '0', '-5', 'h', '1x'])('rejects %j', (input) => {
    expect(parseDuration(input)).toBeNull();
  });
});
