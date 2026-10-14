import { describe, expect, it } from 'vitest';
import { buildChanges, draftFromItem, emptyDraft, type ItemDraft } from './itemDraft';
import type { Item } from './types';

const local = (s: string) => new Date(s).toISOString();

const gym: Item = {
  id: 'g1',
  kind: 'task',
  anchoring: 'flexible',
  status: 'open',
  title: 'Gym',
  notes: '',
  all_day: false,
  start_at: local('2026-10-13T17:30'),
  end_at: local('2026-10-13T18:15'),
  start_date: null,
  end_date: null,
  tz: null,
  duration_min: 45,
  due_at: null,
  priority: 0,
  tags: ['health'],
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

const edit = (
  item: Item,
  change: Partial<ItemDraft>,
  reminders = [] as Parameters<typeof draftFromItem>[1],
) => {
  const before = draftFromItem(item, reminders);
  return buildChanges({ ...before, ...change }, { item, draft: before });
};

describe('draftFromItem', () => {
  it('shows local date, time and compact duration', () => {
    expect(draftFromItem(gym)).toMatchObject({
      date: '2026-10-13',
      time: '17:30',
      duration: '45m',
    });
    expect(draftFromItem({ ...gym, duration_min: 90 }).duration).toBe('1h30m');
  });
});

describe('buildChanges (create)', () => {
  it('builds create_item args for each scheduling shape', () => {
    const timed = buildChanges(
      emptyDraft({
        title: ' Dentist ',
        kind: 'event',
        anchoring: 'fixed',
        date: '2026-10-20',
        time: '15:00',
        duration: '45m',
      }),
    );
    expect(timed).toMatchObject({
      ok: true,
      create: { title: 'Dentist', start: '2026-10-20T15:00', duration_min: 45 },
    });
    const allDay = buildChanges(emptyDraft({ title: 'Leave', date: '2026-10-20' }));
    expect(allDay).toMatchObject({ ok: true, create: { date: '2026-10-20' } });
    const inbox = buildChanges(
      emptyDraft({ title: 'Someday', reminders: [{ offset_min: 10, at: null }] }),
    );
    expect(inbox.ok && inbox.create).toEqual({
      title: 'Someday',
      kind: 'task',
      anchoring: 'flexible',
    });
    expect(inbox.ok && inbox.newReminders).toEqual([10]);
  });

  it('validates before building', () => {
    expect(buildChanges(emptyDraft())).toMatchObject({ ok: false, error: 'Give it a title.' });
    expect(buildChanges(emptyDraft({ title: 'x', time: '10:00' }))).toMatchObject({ ok: false });
    expect(buildChanges(emptyDraft({ title: 'x', duration: 'ages' }))).toMatchObject({ ok: false });
    expect(buildChanges(emptyDraft({ title: 'x', kind: 'event' }))).toMatchObject({ ok: false });
  });
});

describe('buildChanges (edit)', () => {
  it('sends only what changed', () => {
    expect(edit(gym, { title: 'Gym session', priority: 2 })).toEqual({
      ok: true,
      calls: [{ name: 'update_item', args: { id: 'g1', title: 'Gym session', priority: 2 } }],
      newReminders: [],
    });
    expect(edit(gym, {})).toEqual({ ok: true, calls: [], newReminders: [] });
  });

  it('reschedules, changes duration alone, or goes all-day', () => {
    expect(edit(gym, { time: '18:00' })).toMatchObject({
      calls: [{ args: { start: '2026-10-13T18:00', duration_min: 45 } }],
    });
    expect(edit(gym, { time: '6pm' })).toMatchObject({
      calls: [{ args: { start: '2026-10-13T18:00' } }],
    });
    expect(edit(gym, { time: '5:30 PM' })).toEqual({ ok: true, calls: [], newReminders: [] });
    expect(edit(gym, { duration: '1h' })).toMatchObject({
      calls: [{ args: { duration_min: 60 } }],
    });
    expect(edit(gym, { time: '' })).toMatchObject({ calls: [{ args: { date: '2026-10-13' } }] });
  });

  it('clearing the date unschedules', () => {
    expect(edit(gym, { date: '', time: '' })).toMatchObject({
      calls: [{ name: 'unschedule_item', args: { id: 'g1' } }],
    });
  });

  it('sets and clears the due date', () => {
    expect(edit(gym, { dueDate: '2026-10-30' })).toMatchObject({
      calls: [{ args: { due: '2026-10-30T17:00' } }],
    });
    const due = { ...gym, due_at: local('2026-10-30T09:00') };
    expect(edit(due, { dueDate: '', dueTime: '' })).toMatchObject({
      calls: [{ args: { due: null } }],
    });
  });

  it('adds and removes reminders', () => {
    const result = edit(gym, { reminders: [{ offset_min: 30, at: null }] }, [
      { id: 'r1', offset_min: 10, at: null },
    ]);
    expect(result).toMatchObject({
      calls: [
        { name: 'clear_reminder', args: { reminder_id: 'r1' } },
        { name: 'set_reminder', args: { item_id: 'g1', offset_min: 30 } },
      ],
    });
  });
});
