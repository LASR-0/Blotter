import {
  addDays,
  addMonths,
  addWeeks,
  addYears,
  endOfWeek,
  format,
  getISOWeek,
  startOfWeek,
  type Day,
} from 'date-fns';

export type CalendarView = 'week' | 'month' | 'year';
export type View = 'diary' | CalendarView | 'inbox' | 'history' | 'settings';

/** Views with a date period (and prev/next/today). */
export const PERIOD_VIEWS: readonly View[] = ['diary', 'week', 'month', 'year'];

export function hasPeriod(view: View): boolean {
  return PERIOD_VIEWS.includes(view);
}

export function isCalendarView(view: View): view is CalendarView {
  return view === 'week' || view === 'month' || view === 'year';
}

/** Moves the anchor one period in `dir` (−1 or +1) for the given view. */
export function stepPeriod(view: View, anchor: Date, dir: 1 | -1): Date {
  switch (view) {
    case 'diary':
      return addDays(anchor, dir);
    case 'week':
      return addWeeks(anchor, dir);
    case 'month':
      return addMonths(anchor, dir);
    case 'year':
      return addYears(anchor, dir);
    default:
      return anchor;
  }
}

export interface PeriodTitle {
  title: string;
  /** Secondary mono label, e.g. `week 42` or `W42`. */
  sub: string;
}

/** Header title for a view at `anchor`, e.g. `12 – 18 October 2026` / `W42`. */
export function periodTitle(view: View, anchor: Date, weekStartsOn: Day = 1): PeriodTitle {
  switch (view) {
    case 'diary':
      return { title: format(anchor, 'EEEE d MMMM'), sub: `week ${getISOWeek(anchor)}` };
    case 'week': {
      const start = startOfWeek(anchor, { weekStartsOn });
      const end = endOfWeek(anchor, { weekStartsOn });
      const sameMonth = start.getMonth() === end.getMonth();
      const sameYear = start.getFullYear() === end.getFullYear();
      const left = format(start, sameMonth ? 'd' : sameYear ? 'd MMM' : 'd MMM yyyy');
      const right = format(end, sameMonth ? 'd MMMM yyyy' : 'd MMM yyyy');
      return { title: `${left} – ${right}`, sub: `W${getISOWeek(anchor)}` };
    }
    case 'month':
      return { title: format(anchor, 'MMMM yyyy'), sub: '' };
    case 'year':
      return { title: format(anchor, 'yyyy'), sub: '' };
    case 'inbox':
      return { title: 'Inbox', sub: '' };
    case 'history':
      return { title: 'History', sub: '' };
    case 'settings':
      return { title: 'Settings', sub: '' };
  }
}

/** `YYYY-MM-DD` for a local date, the format operations expect. */
export function isoDate(d: Date): string {
  return format(d, 'yyyy-MM-dd');
}
