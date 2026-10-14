// Pure focus movement for the views' roving focus: column lists (Diary,
// Inbox) and time-ordered day lists (Week).

export type Dir = 'left' | 'right' | 'up' | 'down';

/**
 * Moves through columns of ids (Diary day columns, or one Inbox column).
 * Up/down step within a column; left/right jump to the nearest non-empty
 * column in that direction, keeping the row where possible. With nothing
 * focused, focuses the first id.
 */
export function moveInColumns(
  columns: readonly (readonly string[])[],
  current: string | null,
  dir: Dir,
): string | null {
  const first = columns.find((c) => c.length > 0)?.[0] ?? null;
  const col = current ? columns.findIndex((c) => c.includes(current)) : -1;
  if (col < 0) return first;
  const column = columns[col] as readonly string[];
  const row = column.indexOf(current as string);

  if (dir === 'up') return column[Math.max(0, row - 1)] ?? current;
  if (dir === 'down') return column[Math.min(column.length - 1, row + 1)] ?? current;

  const step = dir === 'right' ? 1 : -1;
  for (let c = col + step; c >= 0 && c < columns.length; c += step) {
    const target = columns[c] as readonly string[];
    if (target.length > 0) return target[Math.min(row, target.length - 1)] ?? current;
  }
  return current;
}

export interface Timed {
  id: string;
  /** Minutes from midnight (or any comparable start). */
  start: number;
}

/**
 * Moves through a week of time-ordered items. Up/down step to the previous
 * or next item that day; left/right jump to the item nearest in start time
 * on the closest day in that direction that has items.
 */
export function moveInWeek(
  days: readonly (readonly Timed[])[],
  current: string | null,
  dir: Dir,
): string | null {
  const sorted = days.map((d) =>
    [...d].sort((a, b) => a.start - b.start || a.id.localeCompare(b.id)),
  );
  const first = sorted.find((d) => d.length > 0)?.[0]?.id ?? null;
  const dayIdx = current ? sorted.findIndex((d) => d.some((t) => t.id === current)) : -1;
  if (dayIdx < 0) return first;
  const day = sorted[dayIdx] as Timed[];
  const idx = day.findIndex((t) => t.id === current);
  const here = day[idx] as Timed;

  if (dir === 'up') return day[Math.max(0, idx - 1)]?.id ?? current;
  if (dir === 'down') return day[Math.min(day.length - 1, idx + 1)]?.id ?? current;

  const step = dir === 'right' ? 1 : -1;
  for (let d = dayIdx + step; d >= 0 && d < sorted.length; d += step) {
    const target = sorted[d] as Timed[];
    if (target.length === 0) continue;
    let best = target[0] as Timed;
    for (const t of target)
      if (Math.abs(t.start - here.start) < Math.abs(best.start - here.start)) best = t;
    return best.id;
  }
  return current;
}

/** Keyboard move/resize step for timed items (design: 15 minutes). */
export const STEP_MIN = 15;
