// Pure layout for calendar grids: overlap lanes for timed items within a
// day, and rows for all-day items spanning several days.

export interface Span {
  id: string;
  /** Minutes from the start of the day (may be negative or > 1440 before clipping). */
  start: number;
  end: number;
}

export interface Placed {
  id: string;
  start: number;
  end: number;
  /** Zero-based lane within its overlap cluster. */
  lane: number;
  /** Number of lanes in the cluster; the item is 1/lanes wide. */
  lanes: number;
}

/** Minimum visual length so zero-length items still have a body to show. */
const MIN_MINUTES = 15;

/**
 * Lays out one day's timed items. Overlapping items are grouped into
 * clusters; each item takes the first free lane, and every item in a
 * cluster shares the cluster's lane count so widths line up.
 */
export function layoutDay(spans: readonly Span[]): Placed[] {
  const items = spans
    .map((s) => ({ ...s, end: Math.max(s.end, s.start + MIN_MINUTES) }))
    .sort((a, b) => a.start - b.start || b.end - a.end || a.id.localeCompare(b.id));

  const out: Placed[] = [];
  let cluster: Placed[] = [];
  let laneEnds: number[] = [];
  let clusterEnd = -Infinity;

  const flush = () => {
    for (const p of cluster) p.lanes = laneEnds.length;
    out.push(...cluster);
    cluster = [];
    laneEnds = [];
  };

  for (const s of items) {
    if (s.start >= clusterEnd) flush();
    let lane = laneEnds.findIndex((end) => end <= s.start);
    if (lane === -1) {
      lane = laneEnds.length;
      laneEnds.push(s.end);
    } else {
      laneEnds[lane] = s.end;
    }
    cluster.push({ id: s.id, start: s.start, end: s.end, lane, lanes: 0 });
    clusterEnd = Math.max(clusterEnd, s.end);
  }
  flush();
  return out;
}

export interface DayRange {
  id: string;
  /** Inclusive day indexes within the visible range (may extend past it). */
  first: number;
  last: number;
}

export interface PlacedRange {
  id: string;
  /** Clipped to [0, days − 1]. */
  first: number;
  last: number;
  row: number;
}

/** Stacks multi-day bars into rows so none overlap. Hidden ranges are dropped. */
export function layoutAllDay(ranges: readonly DayRange[], days: number): PlacedRange[] {
  const visible = ranges
    .filter((r) => r.last >= 0 && r.first < days)
    .map((r) => ({ id: r.id, first: Math.max(0, r.first), last: Math.min(days - 1, r.last) }))
    .sort((a, b) => a.first - b.first || b.last - a.last || a.id.localeCompare(b.id));
  const rowEnds: number[] = [];
  return visible.map((r) => {
    let row = rowEnds.findIndex((end) => end < r.first);
    if (row === -1) {
      row = rowEnds.length;
      rowEnds.push(r.last);
    } else {
      rowEnds[row] = r.last;
    }
    return { ...r, row };
  });
}
