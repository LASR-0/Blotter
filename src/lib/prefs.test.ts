import { afterEach, describe, expect, it } from 'vitest';
import { blockedIntervals, planningMinutes } from './availability';
import { clockTime, hourLabel, timeOf } from './format';
import { periodTitle } from './period';
import { DEFAULT_PLANNING, sharedSpan, usePrefs, weekStartsOn } from './prefs';

afterEach(() => usePrefs.setState({ weekStart: 'mon', clock: '24h', planning: DEFAULT_PLANNING }));

describe('display preferences', () => {
  it('shows times on a 24- or 12-hour clock', () => {
    const t = new Date(2026, 9, 16, 17, 30);
    expect(clockTime(t)).toBe('17:30');
    expect(hourLabel(9)).toBe('09:00');
    usePrefs.setState({ clock: '12h' });
    expect(clockTime(t)).toBe('5:30 pm');
    expect(timeOf(new Date(2026, 9, 16, 0, 5).toISOString())).toBe('12:05 am');
    expect([0, 9, 12, 13].map(hourLabel)).toEqual(['12 am', '9 am', '12 pm', '1 pm']);
  });

  it('starts weeks on the chosen day', () => {
    expect(weekStartsOn()).toBe(1);
    usePrefs.setState({ weekStart: 'sun' });
    expect(weekStartsOn()).toBe(0);
    // Fri 16 Oct 2026: a Sunday-first week runs 11–17 October.
    expect(periodTitle('week', new Date(2026, 9, 16)).title).toMatch(/^11 – 17 October/);
  });

  it('reads planning hours per weekday', () => {
    expect(planningMinutes(new Date(2026, 9, 17))).toEqual({ start: 420, end: 1320 });
    usePrefs.setState({
      planning: { ...DEFAULT_PLANNING, sat: null, mon: { start: '09:00', end: '17:30' } },
    });
    expect(planningMinutes(new Date(2026, 9, 17))).toBeNull();
    expect(planningMinutes(new Date(2026, 9, 19))).toEqual({ start: 540, end: 1050 });
    expect(blockedIntervals([], new Date(2026, 9, 17))).toEqual([]);
  });

  it('knows when every day shares one window', () => {
    expect(sharedSpan(DEFAULT_PLANNING)).toEqual({ start: '07:00', end: '22:00' });
    expect(sharedSpan({ ...DEFAULT_PLANNING, sun: null })).toBeNull();
    expect(sharedSpan({ ...DEFAULT_PLANNING, tue: { start: '08:00', end: '22:00' } })).toBeNull();
  });
});
