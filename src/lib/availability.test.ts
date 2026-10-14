import { describe, expect, it } from 'vitest';
import { availableMinutes, blockedIntervals, coveredMinutes, windowTimes } from './availability';
import type { BlockoutWindow } from './types';

/** A window between two local wall-clock times (so tests pass in any zone). */
const win = (
  label: string,
  start: [number, number, number, number, number],
  end: [number, number, number, number, number],
): BlockoutWindow => {
  const [y, m, d, h, min] = start;
  const [y2, m2, d2, h2, min2] = end;
  return {
    blockout_id: label.toLowerCase(),
    label,
    date: `${y}-${String(m + 1).padStart(2, '0')}-${String(d).padStart(2, '0')}`,
    start: new Date(y, m, d, h, min).toISOString(),
    end: new Date(y2, m2, d2, h2, min2).toISOString(),
    permits_fixed: label !== 'Sleep',
    color: null,
  };
};

const tue = new Date(2026, 9, 13);
const sat = new Date(2026, 9, 17);
const work = win('Work', [2026, 9, 13, 8, 30], [2026, 9, 13, 17, 0]);
const sleepMon = win('Sleep', [2026, 9, 12, 23, 0], [2026, 9, 13, 6, 30]);
const sleepTue = win('Sleep', [2026, 9, 13, 23, 0], [2026, 9, 14, 6, 30]);

describe('availableMinutes', () => {
  it('matches the design: 15h planning minus Work 8h30 = 6h30 free', () => {
    expect(availableMinutes([sleepMon, work, sleepTue], tue)).toBe(390);
  });

  it('is the whole planning day when no window touches it', () => {
    expect(availableMinutes([sleepMon, work, sleepTue], sat)).toBe(900);
  });
});

describe('blockedIntervals', () => {
  it('shows a past-midnight window on both days it touches', () => {
    const parts = blockedIntervals([sleepTue, sleepMon], tue).map((i) => [i.start, i.end]);
    expect(parts).toEqual([
      [0, 390],
      [1380, 1440],
    ]);
  });

  it('keeps the window for labels', () => {
    const [i] = blockedIntervals([work], tue);
    expect(i?.window.label).toBe('Work');
    expect(windowTimes(work)).toBe('08:30–17:00');
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
