import { useOp } from './queries';
import type { Holiday } from './types';

/** The holiday covering a `YYYY-MM-DD` date, if any; first match wins. */
export function useHolidayOn(): (ymd: string) => Holiday | undefined {
  const holidays = useOp<Holiday[]>('list_holidays').data ?? [];
  return (ymd) => holidays.find((h) => h.start_date <= ymd && ymd <= h.end_date);
}
