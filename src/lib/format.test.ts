import { describe, expect, it } from 'vitest';
import { parseTime } from './format';

describe('parseTime', () => {
  it.each([
    ['17:30', '17:30'],
    ['1730', '17:30'],
    ['9:05', '09:05'],
    ['930', '09:30'],
    ['7', '07:00'],
    ['5pm', '17:00'],
    ['5:30 PM', '17:30'],
    ['12am', '00:00'],
    ['12pm', '12:00'],
    ['0:00', '00:00'],
  ])('%s → %s', (input, expected) => {
    expect(parseTime(input)).toBe(expected);
  });

  it.each(['', 'noon', '24:00', '9:60', '13pm', '0am', '12345'])('rejects %j', (input) => {
    expect(parseTime(input)).toBeNull();
  });
});
