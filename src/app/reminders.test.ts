import { describe, expect, it } from 'vitest';
import { missedTitle, parseSnooze, reminderStatus, type Due } from './reminders';

// Local times, so the tests hold in any zone.
const at = (h: number, m = 0, day = 15) => new Date(2026, 9, day, h, m);
const iso = (d: Date) => d.toISOString();

const due = (over: Partial<Due> = {}): Due => ({
  reminder_id: 'r',
  item_id: 'i',
  title: 'Telehealth consult',
  start: iso(at(11, 30)),
  end: iso(at(12, 15)),
  all_day: false,
  due: iso(at(11, 20)),
  snoozed: false,
  ...over,
});

describe('reminderStatus', () => {
  it('says how soon, then the time', () => {
    expect(reminderStatus(due(), at(11, 20))).toBe('in 10 min · 11:30–12:15');
    expect(reminderStatus(due({ end: null }), at(11, 30))).toBe('now · 11:30');
    expect(reminderStatus(due(), at(11, 45))).toBe('started 15 min ago · 11:30–12:15');
  });

  it('marks snoozed firings and handles all-day and untimed items', () => {
    expect(reminderStatus(due({ snoozed: true }), at(11, 30))).toBe('now · 11:30–12:15 · snoozed');
    expect(reminderStatus(due({ all_day: true }), at(9))).toBe('All day');
    expect(reminderStatus(due({ start: null }), at(9))).toBe('Reminder');
  });
});

describe('parseSnooze', () => {
  const now = at(11, 20);

  it.each([
    ['25m', 25],
    ['2h', 120],
    ['1h30', 90],
    ['90min', 90],
    ['45', 45],
  ])('reads %s as a length', (input, minutes) => {
    expect(parseSnooze(input, now)).toEqual({ minutes });
  });

  it('reads a time of day as later today, or tomorrow once passed', () => {
    expect(parseSnooze('14:00', now)).toEqual({ until: '2026-10-15T14:00' });
    expect(parseSnooze('2pm', now)).toEqual({ until: '2026-10-15T14:00' });
    expect(parseSnooze('9:00', now)).toEqual({ until: '2026-10-16T09:00' });
  });

  it('rejects nonsense and over-long snoozes', () => {
    expect(parseSnooze('', now)).toBeNull();
    expect(parseSnooze('soon', now)).toBeNull();
    expect(parseSnooze('25:00', now)).toBeNull();
    expect(parseSnooze('200h', now)).toBeNull();
  });
});

describe('missedTitle', () => {
  it('counts and says why', () => {
    const one = { since: iso(at(1)), while_closed: true, items: [due()] };
    expect(missedTitle(one)).toBe('1 reminder missed while Blotter was closed');
    expect(missedTitle({ ...one, while_closed: false, items: [due(), due()] })).toBe(
      '2 reminders missed while your computer was asleep',
    );
  });
});
