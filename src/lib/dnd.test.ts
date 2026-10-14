import { describe, expect, it } from 'vitest';
import { between, dropCall, refusedBy, resizeEnd, slotStart } from './dnd';
import type { BlockoutWindow, Item } from './types';

const local = (s: string) => new Date(s).toISOString();
const item = (o: Partial<Item>): Item => ({
  id: 'i',
  kind: 'task',
  anchoring: 'flexible',
  status: 'open',
  title: 'x',
  notes: '',
  all_day: false,
  start_at: null,
  end_at: null,
  start_date: null,
  end_date: null,
  tz: null,
  duration_min: 30,
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
  ...o,
});
const work: BlockoutWindow = {
  blockout_id: 'w',
  label: 'Work',
  date: '2026-10-13',
  start: new Date(2026, 9, 13, 8, 30).toISOString(),
  end: new Date(2026, 9, 13, 17, 0).toISOString(),
  permits_fixed: true,
  color: null,
};
const tue = new Date(2026, 9, 13);

describe('slotStart', () => {
  it('snaps to 15 minutes, keeping the grab offset', () => {
    expect(slotStart(17 * 60 + 7, 0, 30)).toBe(17 * 60);
    expect(slotStart(17 * 60 + 8, 0, 30)).toBe(17 * 60 + 15);
    expect(slotStart(17 * 60 + 20, 20, 45)).toBe(17 * 60);
  });
  it('stays within the day', () => {
    expect(slotStart(-30, 0, 30)).toBe(0);
    expect(slotStart(23 * 60 + 50, 0, 60)).toBe(23 * 60);
  });
});

describe('resizeEnd', () => {
  it('snaps and never goes before start + 15', () => {
    expect(resizeEnd(18 * 60 + 22, 17 * 60)).toBe(18 * 60 + 15);
    expect(resizeEnd(16 * 60, 17 * 60)).toBe(17 * 60 + 15);
  });
});

describe('refusedBy', () => {
  it('refuses a flexible item overlapping a block-out', () => {
    expect(
      refusedBy({ anchoring: 'flexible' }, tue, 16 * 60 + 45, 17 * 60 + 15, [work])?.label,
    ).toBe('Work');
  });
  it('allows a flexible item just outside, and any fixed item', () => {
    expect(refusedBy({ anchoring: 'flexible' }, tue, 17 * 60, 17 * 60 + 15, [work])).toBeNull();
    expect(refusedBy({ anchoring: 'fixed' }, tue, 11 * 60, 12 * 60, [work])).toBeNull();
  });
});

describe('between', () => {
  it('finds a midpoint or extends past the ends', () => {
    expect(between(1, 2)).toBe(1.5);
    expect(between(undefined, 5)).toBe(4);
    expect(between(5, undefined)).toBe(6);
    expect(between(undefined, undefined)).toBe(0);
  });
});

describe('dropCall', () => {
  const timed = item({ start_at: local('2026-10-13T17:30'), end_at: local('2026-10-13T18:15') });

  it('schedules at a slot or resizes', () => {
    expect(dropCall(timed, { kind: 'slot', date: tue, startMin: 19 * 60 })).toEqual({
      name: 'update_item',
      args: { id: 'i', start: '2026-10-13T19:00' },
    });
    expect(dropCall(timed, { kind: 'resize', date: tue, endMin: 18 * 60 + 30 })).toEqual({
      name: 'update_item',
      args: { id: 'i', end: '2026-10-13T18:30' },
    });
  });

  it('moves timed items to a day keeping the time, untimed ones as day tasks with order', () => {
    const wed = new Date(2026, 9, 14);
    expect(dropCall(timed, { kind: 'day', date: wed, sortOrder: 3 })).toMatchObject({
      args: { start: '2026-10-14T17:30' },
    });
    expect(dropCall(item({}), { kind: 'day', date: wed, sortOrder: 3 })).toEqual({
      name: 'update_item',
      args: { id: 'i', date: '2026-10-14', sort_order: 3 },
    });
  });

  it('sends tasks back to the Inbox but not events', () => {
    expect(dropCall(timed, { kind: 'inbox' })).toEqual({
      name: 'unschedule_item',
      args: { id: 'i' },
    });
    expect(dropCall(item({ kind: 'event' }), { kind: 'inbox' })).toHaveProperty('error');
  });
});
