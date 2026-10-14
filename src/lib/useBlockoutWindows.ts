import { addDays, endOfMonth, format, startOfMonth } from 'date-fns';
import { useNav } from '@/app/nav';
import { useOp } from './queries';
import type { BlockoutWindow, BlockoutWindows } from './types';

const ymd = (d: Date) => format(d, 'yyyy-MM-dd');

/**
 * Block-out windows from the core around the period being viewed: the
 * anchor's month, a week before it and six weeks after (the Diary scrolls
 * ahead, the Month grid shows spill-over days). The range only changes
 * month to month, so every view and the drag layer share one cached query.
 */
export function useBlockoutWindows(): BlockoutWindow[] {
  const anchor = useNav((s) => s.anchor);
  const month = startOfMonth(anchor);
  const windows = useOp<BlockoutWindows>('list_blockout_windows', {
    start: ymd(addDays(month, -7)),
    end: ymd(addDays(endOfMonth(month), 42)),
  });
  return windows.data?.windows ?? EMPTY;
}

const EMPTY: BlockoutWindow[] = [];
