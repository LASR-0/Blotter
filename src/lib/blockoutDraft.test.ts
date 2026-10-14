import { describe, expect, it } from 'vitest';
import {
  appliesOn,
  blockoutSave,
  describeDays,
  draftFromBlockout,
  emptyBlockoutDraft,
  liftCalls,
  reorderCalls,
  validateBlockout,
} from './blockoutDraft';
import type { Blockout } from './types';

const blockout = (o: Partial<Blockout> = {}): Blockout => ({
  id: 'w',
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
  created_at: '',
  updated_at: '',
  exceptions: [],
  ...o,
});

describe('validateBlockout', () => {
  it('needs a label, a day and two different times', () => {
    const ok = { ...emptyBlockoutDraft(), label: 'Gym' };
    expect(validateBlockout(ok)).toBeNull();
    expect(validateBlockout({ ...ok, label: '  ' })).toMatch(/label/);
    expect(validateBlockout({ ...ok, weekdays: [] })).toMatch(/day/);
    expect(validateBlockout({ ...ok, start: 'soon' })).toMatch(/isn't a time/);
    expect(validateBlockout({ ...ok, start: '5pm', end: '17:00' })).toMatch(/differ/);
  });
});

describe('blockoutSave', () => {
  it('creates with normalised times and Monday-first days', () => {
    const save = blockoutSave(null, {
      label: ' Sleep ',
      weekdays: ['sun', 'mon'],
      start: '11pm',
      end: '6:30',
      permitsFixed: false,
    });
    expect(save).toEqual({
      name: 'create_blockout',
      args: {
        label: 'Sleep',
        weekdays: ['mon', 'sun'],
        start_time: '23:00',
        end_time: '06:30',
        permits_fixed: false,
      },
    });
  });

  it('updates only what changed, or nothing', () => {
    const b = blockout();
    expect(blockoutSave(b, draftFromBlockout(b))).toBeNull();
    expect(blockoutSave(b, { ...draftFromBlockout(b), end: '5:30pm' })).toEqual({
      name: 'update_blockout',
      args: { id: 'w', end_time: '17:30' },
    });
  });
});

describe('reorderCalls', () => {
  it('renumbers only the rows that moved', () => {
    const list = [
      blockout({ id: 'a', sort_order: 0 }),
      blockout({ id: 'b', sort_order: 1 }),
      blockout({ id: 'c', sort_order: 2 }),
    ];
    expect(reorderCalls(list, 'c', 0)).toEqual([
      { name: 'update_blockout', args: { id: 'c', sort_order: 0 } },
      { name: 'update_blockout', args: { id: 'a', sort_order: 1 } },
      { name: 'update_blockout', args: { id: 'b', sort_order: 2 } },
    ]);
    expect(reorderCalls(list, 'a', 0)).toEqual([]);
  });
});

describe('lifting', () => {
  it('knows which days a block-out applies on', () => {
    const b = blockout({ active_until: '2026-10-14' });
    expect(appliesOn(b, new Date(2026, 9, 14))).toBe(true);
    expect(appliesOn(b, new Date(2026, 9, 15))).toBe(false); // after the range
    expect(appliesOn(blockout(), new Date(2026, 9, 17))).toBe(false); // Saturday
  });

  it('lifts only applicable, not-yet-lifted dates in a range', () => {
    const b = blockout({
      exceptions: [{ id: 'x', blockout_id: 'w', date: '2026-10-13', note: '', created_at: '' }],
    });
    // Mon 12 – Sun 18 Oct: Tue is already lifted, the weekend doesn't apply.
    const dates = liftCalls(b, '2026-10-12', '2026-10-18', ' Leave ').map((c) => c.args.date);
    expect(dates).toEqual(['2026-10-12', '2026-10-14', '2026-10-15', '2026-10-16']);
    expect(liftCalls(b, '2026-10-12', '', 'x')[0]?.args.note).toBe('x');
  });

  it('rejects bad ranges', () => {
    expect(() => liftCalls(blockout(), '', '', '')).toThrow(/date/);
    expect(() => liftCalls(blockout(), '2026-10-12', '2026-10-01', '')).toThrow(/before/);
    expect(() => liftCalls(blockout(), '2026-01-01', '2027-06-01', '')).toThrow(/366/);
  });
});

describe('describeDays', () => {
  it('reads naturally', () => {
    expect(describeDays(['mon', 'tue', 'wed', 'thu', 'fri'])).toBe('Mon–Fri');
    expect(describeDays(['sun', 'sat'])).toBe('Weekends');
    expect(describeDays(['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun'])).toBe('Every day');
    expect(describeDays(['fri', 'mon', 'wed'])).toBe('Mon, Wed, Fri');
  });
});
