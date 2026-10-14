import { describe, expect, it } from 'vitest';
import { addedAgo, parseTime } from './format';

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

describe('addedAgo', () => {
  const now = new Date(2026, 9, 14, 10, 30);
  it.each([
    [new Date(2026, 9, 14, 10, 29, 30), 'now'],
    [new Date(2026, 9, 14, 7, 0), 'today'],
    [new Date(2026, 9, 13, 23, 0), 'yesterday'],
    [new Date(2026, 9, 11, 9, 0), '3d ago'],
    [new Date(2026, 9, 1, 9, 0), '1 Oct'],
  ])('%s → %s', (then, expected) => {
    expect(addedAgo(then.toISOString(), now)).toBe(expected);
  });
});
