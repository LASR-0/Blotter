import { create } from 'zustand';
import type { Day } from 'date-fns';
import { onPlannerChanged, runOp } from './api';

export type WeekStart = 'mon' | 'sun' | 'sat';
export type ClockStyle = '24h' | '12h';

/** A wall-clock window, `HH:mm`. */
export interface Span {
  start: string;
  end: string;
}

/** Mirrors `slots::PlanningHours`: null means no planning that day. */
export type PlanningHours = Record<
  'mon' | 'tue' | 'wed' | 'thu' | 'fri' | 'sat' | 'sun',
  Span | null
>;

export const DEFAULT_PLANNING: PlanningHours = {
  mon: { start: '07:00', end: '22:00' },
  tue: { start: '07:00', end: '22:00' },
  wed: { start: '07:00', end: '22:00' },
  thu: { start: '07:00', end: '22:00' },
  fri: { start: '07:00', end: '22:00' },
  sat: { start: '07:00', end: '22:00' },
  sun: { start: '07:00', end: '22:00' },
};

const WEEK_DAY: Record<WeekStart, Day> = { mon: 1, sun: 0, sat: 6 };

interface PrefsState {
  weekStart: WeekStart;
  clock: ClockStyle;
  planning: PlanningHours;
  /** Reads them from `get_settings`, and again whenever settings change. */
  load: () => Promise<void>;
}

interface SettingsSlice {
  week_start: WeekStart;
  clock: ClockStyle;
  planning_hours: PlanningHours;
}

/**
 * Display preferences the date helpers read synchronously (week start,
 * 12/24-hour clock, planning hours). They live in the core's settings;
 * this keeps a copy so pure helpers needn't take them as arguments.
 */
export const usePrefs = create<PrefsState>((set) => ({
  weekStart: 'mon',
  clock: '24h',
  planning: DEFAULT_PLANNING,
  load: async () => {
    const s = await runOp<SettingsSlice>('get_settings');
    set({ weekStart: s.week_start, clock: s.clock, planning: s.planning_hours });
  },
}));

/** date-fns' `weekStartsOn` for the user's week start. */
export const weekStartsOn = (): Day => WEEK_DAY[usePrefs.getState().weekStart];

/** Loads the prefs now and after every settings change. Returns a stop function. */
export function syncPrefs(): () => void {
  const load = () =>
    usePrefs
      .getState()
      .load()
      .catch((e: unknown) => console.error('Could not load preferences', e));
  void load();
  const stop = onPlannerChanged((e) => {
    const reloads = ['update_settings', 'undo', 'redo', 'restore', 'import'];
    if (e.ops.some((op) => reloads.includes(op))) void load();
  });
  return () => void stop.then((unlisten) => unlisten());
}

const PLANNING_DAYS = ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun'] as const;

/** The one span every day shares, or null when the days differ (or one is off). */
export function sharedSpan(hours: PlanningHours): Span | null {
  const first = hours.mon;
  if (!first) return null;
  return PLANNING_DAYS.every((d) => {
    const s = hours[d];
    return s !== null && s.start === first.start && s.end === first.end;
  })
    ? first
    : null;
}
