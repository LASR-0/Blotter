import { useMemo } from 'react';
import { addDays, endOfMonth, format, startOfMonth } from 'date-fns';
import { useFilters } from '@/app/filters';
import { useNav } from '@/app/nav';
import { itemsByDay } from './days';
import { timeOf } from './format';
import { useOp } from './queries';

/** One cached event of a subscribed calendar (`list_calendar_events`); read-only. */
export interface CalendarEvent {
  id: string;
  subscription_id: string;
  uid: string;
  title: string;
  location: string;
  all_day: boolean;
  start_at: string | null;
  end_at: string | null;
  start_date: string | null;
  end_date: string | null;
  busy: boolean;
}

export interface CalendarInfo {
  id: string;
  name: string;
  color: string | null;
  as_blockout: boolean;
}

export interface CalendarEvents {
  start: string;
  end: string;
  timezone: string;
  calendars: CalendarInfo[];
  events: CalendarEvent[];
}

/** An event ready to draw: its calendar's name and colour resolved. */
export interface ShownEvent extends CalendarEvent {
  /** Prefixed, so it never collides with an item id in layouts. */
  key: string;
  calendar: string;
  color: string;
  /** For `itemsByDay`'s ordering. */
  sort_order: number;
}

/** Colour of a calendar without one of its own. */
export const DEFAULT_CALENDAR_COLOR = 'var(--tag-ics)';

/** Prefix of a calendar event's key in layouts shared with items. */
export const EVENT_KEY_PREFIX = 'cal:';

export const isEventKey = (id: string | null | undefined): boolean =>
  !!id && id.startsWith(EVENT_KEY_PREFIX);

/** The events of enabled calendars, with their calendar resolved. */
export function shownEvents(data: CalendarEvents | undefined): ShownEvent[] {
  if (!data) return [];
  const calendars = new Map(data.calendars.map((c) => [c.id, c]));
  return data.events.flatMap((e) => {
    const cal = calendars.get(e.subscription_id);
    if (!cal) return [];
    return [
      {
        ...e,
        key: `${EVENT_KEY_PREFIX}${e.id}`,
        calendar: cal.name,
        color: cal.color ?? DEFAULT_CALENDAR_COLOR,
        sort_order: 0,
      },
    ];
  });
}

/** "10:00–11:00", or "all day". */
export function eventTimes(e: Pick<CalendarEvent, 'all_day' | 'start_at' | 'end_at'>): string {
  if (e.all_day || !e.start_at) return 'all day';
  return e.end_at && e.end_at !== e.start_at
    ? `${timeOf(e.start_at)}–${timeOf(e.end_at)}`
    : timeOf(e.start_at);
}

/** Hover text: "Team sync · 10:00–11:00 · Room 4 · Work (read-only, free)". */
export function eventTitle(e: ShownEvent): string {
  const notes = ['read-only', !e.busy && 'free'].filter(Boolean).join(', ');
  return [e.title, eventTimes(e), e.location, `${e.calendar} (${notes})`]
    .filter(Boolean)
    .join(' · ');
}

const ymd = (d: Date) => format(d, 'yyyy-MM-dd');

function useEventsQuery() {
  const anchor = useNav((s) => s.anchor);
  const month = startOfMonth(anchor);
  return useOp<CalendarEvents>('list_calendar_events', {
    start: ymd(addDays(month, -7)),
    end: ymd(addDays(endOfMonth(month), 42)),
  });
}

/** The enabled subscribed calendars (for the sidebar's filters), from the same query. */
export function useCalendarList(): CalendarInfo[] {
  return useEventsQuery().data?.calendars ?? [];
}

/**
 * Subscribed events around the period being viewed: the same range as the
 * block-out windows (the anchor's month, a week before, six weeks after),
 * so every view shares one cached query. Calendars hidden in the sidebar are left out.
 */
export function useCalendarEvents(): ShownEvent[] {
  const q = useEventsQuery();
  const hidden = useFilters((s) => s.hiddenCalendars);
  return useMemo(
    () => shownEvents(q.data).filter((e) => !hidden.includes(e.subscription_id)),
    [q.data, hidden],
  );
}

/** Events bucketed by every local day they touch, as for items. */
export function eventsByDay(events: readonly ShownEvent[]): Map<string, ShownEvent[]> {
  return itemsByDay(events);
}
