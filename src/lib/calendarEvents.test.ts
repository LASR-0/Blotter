import { describe, expect, it } from 'vitest';
import {
  DEFAULT_CALENDAR_COLOR,
  eventsByDay,
  eventTimes,
  eventTitle,
  isEventKey,
  shownEvents,
  type CalendarEvent,
  type CalendarEvents,
} from './calendarEvents';

const local = (d: number, h: number, m = 0) => new Date(2026, 9, d, h, m).toISOString();

const event = (o: Partial<CalendarEvent>): CalendarEvent => ({
  id: 'e1',
  subscription_id: 'work',
  uid: 'u1',
  title: 'Team sync',
  location: '',
  all_day: false,
  start_at: local(14, 10),
  end_at: local(14, 11),
  start_date: null,
  end_date: null,
  busy: true,
  ...o,
});

const data = (events: CalendarEvent[]): CalendarEvents => ({
  start: '2026-10-01',
  end: '2026-10-31',
  timezone: 'Australia/Melbourne',
  calendars: [
    { id: 'work', name: 'Work', color: '#6c9cf2', as_blockout: true },
    { id: 'afl', name: 'AFL', color: null, as_blockout: false },
  ],
  events,
});

describe('calendar events', () => {
  it('resolves each event’s calendar and keys it apart from items', () => {
    const [work, afl] = shownEvents(data([event({}), event({ id: 'e2', subscription_id: 'afl' })]));
    expect(work).toMatchObject({ key: 'cal:e1', calendar: 'Work', color: '#6c9cf2' });
    expect(afl?.color).toBe(DEFAULT_CALENDAR_COLOR);
    expect(isEventKey(work?.key)).toBe(true);
    expect(isEventKey('01JABCDEF')).toBe(false);
    // An event whose calendar isn't listed (e.g. just removed) isn't shown.
    expect(shownEvents(data([event({ subscription_id: 'gone' })]))).toEqual([]);
  });

  it('describes times and hover text', () => {
    expect(eventTimes(event({}))).toBe('10:00–11:00');
    expect(eventTimes(event({ end_at: local(14, 10) }))).toBe('10:00');
    expect(eventTimes(event({ all_day: true, start_at: null }))).toBe('all day');
    const [shown] = shownEvents(data([event({ location: 'Room 4', busy: false })]));
    expect(eventTitle(shown!)).toBe('Team sync · 10:00–11:00 · Room 4 · Work (read-only, free)');
  });

  it('buckets by local day like items', () => {
    const days = eventsByDay(
      shownEvents(
        data([
          event({}),
          event({
            id: 'e2',
            all_day: true,
            start_at: null,
            end_at: null,
            start_date: '2026-10-15',
            end_date: '2026-10-16',
          }),
        ]),
      ),
    );
    expect([...days.keys()]).toEqual(['2026-10-14', '2026-10-15', '2026-10-16']);
  });
});
