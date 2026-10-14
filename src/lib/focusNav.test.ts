import { describe, expect, it } from 'vitest';
import { moveInColumns, moveInWeek } from './focusNav';

describe('moveInColumns', () => {
  const cols = [['a1', 'a2', 'a3'], [], ['c1'], ['d1', 'd2']];

  it('focuses the first item when nothing is focused', () => {
    expect(moveInColumns(cols, null, 'down')).toBe('a1');
    expect(moveInColumns([[], []], null, 'down')).toBeNull();
  });

  it('steps within a column and stops at the ends', () => {
    expect(moveInColumns(cols, 'a1', 'down')).toBe('a2');
    expect(moveInColumns(cols, 'a3', 'down')).toBe('a3');
    expect(moveInColumns(cols, 'a1', 'up')).toBe('a1');
  });

  it('jumps over empty columns and clamps the row', () => {
    expect(moveInColumns(cols, 'a3', 'right')).toBe('c1');
    expect(moveInColumns(cols, 'c1', 'right')).toBe('d1');
    expect(moveInColumns(cols, 'd2', 'left')).toBe('c1');
    expect(moveInColumns(cols, 'd1', 'right')).toBe('d1');
  });
});

describe('moveInWeek', () => {
  const week = [
    [
      { id: 'run', start: 435 },
      { id: 'standup', start: 540 },
    ],
    [],
    [
      { id: 'lunch', start: 720 },
      { id: 'dinner', start: 1080 },
    ],
  ];

  it('steps by time within a day', () => {
    expect(moveInWeek(week, 'run', 'down')).toBe('standup');
    expect(moveInWeek(week, 'standup', 'up')).toBe('run');
  });

  it('jumps to the nearest time on the next day with items', () => {
    expect(moveInWeek(week, 'standup', 'right')).toBe('lunch');
    expect(moveInWeek(week, 'dinner', 'left')).toBe('standup');
    expect(moveInWeek(week, 'run', 'left')).toBe('run');
  });

  it('starts at the first item', () => {
    expect(moveInWeek(week, null, 'right')).toBe('run');
  });
});
