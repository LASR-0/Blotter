import { describe, expect, it } from 'vitest';
import { describeParse, destination, parseQuick, toCreateArgs } from './quickParse';

// Wednesday 14 October 2026, 09:00 local.
const NOW = new Date(2026, 9, 14, 9, 0);
const p = (s: string) => parseQuick(s, NOW);

describe('parseQuick', () => {
  it('reads the design example', () => {
    const r = p('dentist tue 3pm ~45m @fixed');
    expect(r).toMatchObject({
      title: 'dentist',
      date: '2026-10-20',
      time: '15:00',
      durationMin: 45,
      anchoring: 'fixed',
      kind: 'event',
    });
    expect(r.chips.map((c) => c.kind)).toEqual(['duration', 'anchoring', 'date', 'time']);
  });

  it('pulls out tags, priority and duration forms', () => {
    const r = p('weekend shopping for presents sat ~1h30 #shopping !2 #Gifts');
    expect(r).toMatchObject({
      title: 'weekend shopping for presents',
      date: '2026-10-17',
      durationMin: 90,
      priority: 2,
      tags: ['shopping', 'gifts'],
    });
    expect(r.time).toBeUndefined();
    expect(p('think ~1.5h').durationMin).toBe(90);
    expect(p('think ~90').durationMin).toBe(90);
  });

  it('leaves a plain line in the Inbox', () => {
    const r = p('call plumber');
    expect(r).toMatchObject({ title: 'call plumber', kind: 'task', anchoring: 'flexible' });
    expect(destination(r)).toBe('inbox');
    expect(toCreateArgs(r)).toEqual({ title: 'call plumber', kind: 'task', anchoring: 'flexible' });
  });

  it('reads relative and numeric dates (day first)', () => {
    expect(p('pay rent 20/10').date).toBe('2026-10-20');
    expect(p('gym in 2 days').date).toBe('2026-10-16');
    expect(p('call mum tomorrow at 18:30')).toMatchObject({
      title: 'call mum',
      date: '2026-10-15',
      time: '18:30',
    });
  });

  it('moves a bare past time to tomorrow', () => {
    expect(p('thing at 7')).toMatchObject({ title: 'thing', date: '2026-10-15', time: '07:00' });
  });

  it('treats "due …" as the deadline, not the start', () => {
    const r = p('tax return due fri 5pm');
    expect(r).toMatchObject({ title: 'tax return', due: '2026-10-16T17:00' });
    expect(r.date).toBeUndefined();
    expect(p('report due 30/10').due).toBe('2026-10-30T17:00');
  });

  it('can have both a date and a deadline', () => {
    const r = p('draft essay tomorrow due fri');
    expect(r).toMatchObject({ title: 'draft essay', date: '2026-10-15', due: '2026-10-16T17:00' });
  });

  it('maps recurrence phrases to RRULEs', () => {
    expect(p('stand up every weekday 9am')).toMatchObject({
      rrule: 'FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR',
      time: '09:00',
    });
    expect(p('bins every monday').rrule).toBe('FREQ=WEEKLY;BYDAY=MO');
    expect(p('gym every mon, wed and fri').rrule).toBe('FREQ=WEEKLY;BYDAY=MO,WE,FR');
    expect(p('review every 2 weeks').rrule).toBe('FREQ=WEEKLY;INTERVAL=2');
    expect(p('water plants every other week').rrule).toBe('FREQ=WEEKLY;INTERVAL=2');
    expect(p('rent every month').rrule).toBe('FREQ=MONTHLY');
    // A recurring item needs a start, so it begins today.
    expect(p('bins every monday').date).toBe('2026-10-14');
  });

  it('applies the fixed rule', () => {
    expect(p('gym 6pm')).toMatchObject({ kind: 'task', anchoring: 'flexible' });
    expect(p('lunch with Sam at noon')).toMatchObject({
      kind: 'event',
      anchoring: 'fixed',
      time: '12:00',
    });
    expect(p('dentist tue')).toMatchObject({ kind: 'task', anchoring: 'flexible' });
    expect(p('meeting 3pm @flex')).toMatchObject({ kind: 'event', anchoring: 'flexible' });
    expect(p('gym 6pm @fixed').anchoring).toBe('fixed');
  });

  it('builds create args and destinations', () => {
    const timed = p('dentist tue 3pm ~45m #health');
    expect(destination(timed)).toBe('calendar');
    expect(toCreateArgs(timed)).toEqual({
      title: 'dentist',
      kind: 'event',
      anchoring: 'fixed',
      start: '2026-10-20T15:00',
      duration_min: 45,
      tags: ['health'],
    });
    const day = p('pay rent 20/10 !3');
    expect(destination(day)).toBe('day');
    expect(toCreateArgs(day)).toMatchObject({ date: '2026-10-20', priority: 3 });
  });

  it('ignores lookalikes', () => {
    expect(p('email bob@example.com').title).toBe('email bob@example.com');
    expect(p('fix issue #42').tags).toEqual(['42']);
    expect(p('buy 4 apples ~soon').title).toBe('buy 4 apples ~soon');
  });
});

describe('describeParse', () => {
  it('says what an event line will create', () => {
    expect(describeParse(parseQuick('dentist tue 3pm ~45m', NOW))).toBe(
      'Fixed event · Tue 20 Oct 15:00–15:45',
    );
  });

  it('keeps an undated task short', () => {
    expect(describeParse(parseQuick('laundry ~1h !2', NOW))).toBe('Flexible task · 1h · !2');
  });
});

describe('context', () => {
  it('keeps everything after // as notes, unparsed', () => {
    const p = parseQuick('groceries ~1h #home // need the car, shops shut 9pm #not-a-tag', NOW);
    expect(p.title).toBe('groceries');
    expect(p.tags).toEqual(['home']);
    expect(p.time).toBeUndefined();
    expect(p.notes).toBe('need the car, shops shut 9pm #not-a-tag');
    expect(toCreateArgs(p).notes).toBe(p.notes);
    expect(describeParse(p)).toBe(
      'Flexible task · 1h · // need the car, shops shut 9pm #not-a-tag',
    );
  });

  it('leaves a link alone', () => {
    const p = parseQuick('read https://example.com/post', NOW);
    expect(p.notes).toBeUndefined();
    expect(p.title).toBe('read https://example.com/post');
  });
});
