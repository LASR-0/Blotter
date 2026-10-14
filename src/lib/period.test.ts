import { describe, expect, it } from 'vitest';
import { isoDate, periodTitle, stepPeriod } from './period';

const d = (y: number, m: number, day: number) => new Date(y, m - 1, day);

describe('periodTitle', () => {
  it('matches the design headers', () => {
    expect(periodTitle('diary', d(2026, 10, 13))).toEqual({
      title: 'Tuesday 13 October',
      sub: 'week 42',
    });
    expect(periodTitle('week', d(2026, 10, 14))).toEqual({
      title: '12 – 18 October 2026',
      sub: 'W42',
    });
    expect(periodTitle('month', d(2026, 10, 14)).title).toBe('October 2026');
    expect(periodTitle('year', d(2026, 10, 14)).title).toBe('2026');
  });

  it('spans months and years in week titles', () => {
    expect(periodTitle('week', d(2026, 10, 1)).title).toBe('28 Sep – 4 Oct 2026');
    expect(periodTitle('week', d(2026, 12, 30)).title).toBe('28 Dec 2026 – 3 Jan 2027');
  });

  it('honours a Sunday week start', () => {
    expect(periodTitle('week', d(2026, 10, 14), 0).title).toBe('11 – 17 October 2026');
  });
});

describe('stepPeriod', () => {
  it('steps by the view period', () => {
    const a = d(2026, 1, 31);
    expect(isoDate(stepPeriod('diary', a, 1))).toBe('2026-02-01');
    expect(isoDate(stepPeriod('week', a, -1))).toBe('2026-01-24');
    expect(isoDate(stepPeriod('month', a, 1))).toBe('2026-02-28');
    expect(isoDate(stepPeriod('year', a, 1))).toBe('2027-01-31');
    expect(stepPeriod('inbox', a, 1)).toBe(a);
  });
});
