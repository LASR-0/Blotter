import { describe, expect, it } from 'vitest';
import { completionsByDay, heatLevel, itemsByDay } from './days';
import type { Item } from './types';

const base: Item = {
  id: '',
  kind: 'task',
  anchoring: 'flexible',
  status: 'open',
  title: '',
  notes: '',
  all_day: false,
  start_at: null,
  end_at: null,
  start_date: null,
  end_date: null,
  tz: null,
  duration_min: null,
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
  created_at: '2026-10-01T00:00:00Z',
  updated_at: '2026-10-01T00:00:00Z',
};
const item = (o: Partial<Item>): Item => ({ ...base, ...o });
// Local times, so the test holds in any timezone.
const local = (s: string) => new Date(s).toISOString();

describe('itemsByDay', () => {
  it('places timed items on each day they touch', () => {
    const map = itemsByDay([
      item({ id: 'run', start_at: local('2026-10-14T07:15'), end_at: local('2026-10-14T07:45') }),
      item({
        id: 'flight',
        start_at: local('2026-10-14T22:00'),
        end_at: local('2026-10-15T02:00'),
      }),
      item({ id: 'late', start_at: local('2026-10-14T23:00'), end_at: local('2026-10-15T00:00') }),
      item({ id: 'ping', start_at: local('2026-10-16T09:00'), end_at: local('2026-10-16T09:00') }),
    ]);
    expect(map.get('2026-10-14')?.map((i) => i.id)).toEqual(['run', 'flight', 'late']);
    expect(map.get('2026-10-15')?.map((i) => i.id)).toEqual(['flight']);
    expect(map.get('2026-10-16')?.map((i) => i.id)).toEqual(['ping']);
  });

  it('spreads all-day items over their dates and lists them first', () => {
    const map = itemsByDay([
      item({ id: 'gym', start_at: local('2026-10-17T07:00'), end_at: local('2026-10-17T08:00') }),
      item({ id: 'visit', all_day: true, start_date: '2026-10-17', end_date: '2026-10-19' }),
    ]);
    expect(map.get('2026-10-17')?.map((i) => i.id)).toEqual(['visit', 'gym']);
    expect(map.get('2026-10-19')?.map((i) => i.id)).toEqual(['visit']);
    expect(map.has('2026-10-20')).toBe(false);
  });
});

describe('completionsByDay', () => {
  it('keys done items by completion day', () => {
    const map = completionsByDay([
      item({ id: 'a', status: 'done', completed_at: local('2026-10-14T18:21') }),
      item({ id: 'b', status: 'open' }),
    ]);
    expect([...map.keys()]).toEqual(['2026-10-14']);
  });
});

describe('heatLevel', () => {
  it('uses the design thresholds', () => {
    expect([0, 1, 2, 3, 4, 5, 6, 7, 12].map(heatLevel)).toEqual([0, 1, 1, 2, 2, 3, 3, 4, 4]);
  });
});
