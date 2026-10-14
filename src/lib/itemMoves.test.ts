import { describe, expect, it } from 'vitest';
import { moveByDays, moveByMinutes, resizeByMinutes } from './itemMoves';
import type { Item } from './types';

const local = (s: string) => new Date(s).toISOString();
const base: Omit<Item, 'all_day' | 'start_at' | 'end_at' | 'start_date' | 'end_date'> = {
  id: 'x',
  kind: 'task',
  anchoring: 'flexible',
  status: 'open',
  title: 'x',
  notes: '',
  tz: null,
  duration_min: 45,
  due_at: null,
  priority: 0,
  tags: [],
  color: null,
  sort_order: 0,
  rrule: null,
  parent_id: null,
  recurrence_id: null,
  completed_at: null,
  source: 'user',
  created_at: '',
  updated_at: '',
};
const timed: Item = {
  ...base,
  all_day: false,
  start_at: local('2026-10-13T17:30'),
  end_at: local('2026-10-13T18:15'),
  start_date: null,
  end_date: null,
};
const allDay: Item = {
  ...base,
  all_day: true,
  start_at: null,
  end_at: null,
  start_date: '2026-10-17',
  end_date: '2026-10-19',
};

describe('moveByDays', () => {
  it('keeps the local time when moving a timed item', () => {
    expect(moveByDays(timed, 1)?.args).toEqual({ id: 'x', start: '2026-10-14T17:30' });
  });
  it('shifts both dates of a multi-day all-day item', () => {
    expect(moveByDays(allDay, -1)?.args).toEqual({
      id: 'x',
      date: '2026-10-16',
      end_date: '2026-10-18',
    });
  });
  it('ignores unscheduled items', () => {
    expect(moveByDays({ ...timed, start_at: null, end_at: null }, 1)).toBeNull();
  });
});

describe('moveByMinutes', () => {
  it('moves start and end together', () => {
    expect(moveByMinutes(timed, -15)?.args).toEqual({
      id: 'x',
      start: '2026-10-13T17:15',
      end: '2026-10-13T18:00',
    });
  });
  it('ignores all-day items', () => {
    expect(moveByMinutes(allDay, 15)).toBeNull();
  });
});

describe('resizeByMinutes', () => {
  it('grows and shrinks the end', () => {
    expect(resizeByMinutes(timed, 15)?.args).toEqual({ id: 'x', end: '2026-10-13T18:30' });
    expect(resizeByMinutes(timed, -15)?.args).toEqual({ id: 'x', end: '2026-10-13T18:00' });
  });
  it('never goes below 15 minutes', () => {
    const short = { ...timed, end_at: local('2026-10-13T17:45') };
    expect(resizeByMinutes(short, -15)).toBeNull();
    expect(resizeByMinutes(timed, -60)?.args).toEqual({ id: 'x', end: '2026-10-13T17:45' });
  });
});
