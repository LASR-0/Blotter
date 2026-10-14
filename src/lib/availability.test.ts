import { describe, expect, it } from 'vitest';
import { availableMinutes, blockedIntervals, coveredMinutes } from './availability';
import type { Blockout } from './types';

const blockout = (o: Partial<Blockout>): Blockout => ({
  id: 'b',
  label: 'Work',
  weekdays: ['mon', 'tue', 'wed', 'thu', 'fri'],
  start_time: '08:30',
  end_time: '17:00',
  tz: 'Australia/Melbourne',
  active_from: null,
  active_until: null,
  permits_fixed: true,
  color: null,
  icon: null,
  sort_order: 0,
  created_at: '2026-10-01T00:00:00Z',
  updated_at: '2026-10-01T00:00:00Z',
  exceptions: [],
  ...o,
});

const tue = new Date(2026, 9, 13);
const sat = new Date(2026, 9, 17);
const work = blockout({});
const sleep = blockout({
  id: 's',
  label: 'Sleep',
  weekdays: ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun'],
  start_time: '23:00',
  end_time: '06:30',
  permits_fixed: false,
});

describe('availableMinutes', () => {
  it('matches the design: 15h planning minus Work 8h30 = 6h30 free', () => {
    expect(availableMinutes([work, sleep], tue)).toBe(390);
  });

  it('ignores weekday block-outs at the weekend', () => {
    expect(availableMinutes([work, sleep], sat)).toBe(900);
  });

  it('honours lifted dates and active ranges', () => {
    const lifted = blockout({
      exceptions: [{ id: 'e', blockout_id: 'b', date: '2026-10-13', note: '', created_at: '' }],
    });
    expect(availableMinutes([lifted], tue)).toBe(900);
    expect(availableMinutes([blockout({ active_until: '2026-10-12' })], tue)).toBe(900);
    expect(availableMinutes([blockout({ active_from: '2026-10-13' })], tue)).toBe(390);
  });
});

describe('blockedIntervals', () => {
  it('splits a wrap-around window across midnight', () => {
    const parts = blockedIntervals([sleep], tue).map((i) => [i.start, i.end]);
    expect(parts).toEqual([
      [0, 390],
      [1380, 1440],
    ]);
  });
});

describe('coveredMinutes', () => {
  it('counts overlaps once and clips to the window', () => {
    expect(
      coveredMinutes(
        [
          { start: 500, end: 600 },
          { start: 550, end: 700 },
          { start: 1300, end: 1400 },
        ],
        420,
        1320,
      ),
    ).toBe(200 + 20);
  });
});
