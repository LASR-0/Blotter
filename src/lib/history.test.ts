import { describe, expect, it } from 'vitest';
import { daysText, groupByDay, matches, tagsOf, type Completion } from './history';

const done = (title: string, done_on: string, tags: string[] = [], o: Partial<Completion> = {}) =>
  ({
    id: title,
    title,
    notes: '',
    tags,
    done_on,
    done_by: 'user',
    completed_at: `${done_on}T09:00:00Z`,
    ...o,
  }) as Completion;

describe('history', () => {
  const today = new Date(2026, 9, 13);

  it('groups by day with friendly labels', () => {
    const groups = groupByDay(
      [
        done('Run', '2026-10-13'),
        done('Read', '2026-10-12'),
        done('Review', '2026-10-11'),
        done('Clean', '2026-10-01'),
        done('Bill', '2026-10-13'),
      ],
      today,
    );
    expect(groups.map((g) => [g.label, g.sub, g.items.length])).toEqual([
      ['Today', 'Tue 13 Oct', 2],
      ['Yesterday', 'Mon 12 Oct', 1],
      ['Sunday', 'Sun 11 Oct', 1],
      ['1 October', 'Thu 1 Oct', 1],
    ]);
  });

  it('filters by search and hidden tags', () => {
    const groceries = done('Groceries', '2026-10-12', ['shopping'], { notes: 'milk, eggs' });
    const untagged = done('Call Mum', '2026-10-12');
    expect(matches(groceries, 'MILK', new Set())).toBe(true);
    expect(matches(groceries, 'bread', new Set())).toBe(false);
    expect(matches(groceries, '', new Set(['shopping']))).toBe(false);
    // Untagged items never hide behind a tag filter.
    expect(matches(untagged, '', new Set(['shopping']))).toBe(true);
    // An item with another tag still shows.
    expect(
      matches(done('Mixed', '2026-10-12', ['shopping', 'home']), '', new Set(['shopping'])),
    ).toBe(true);
  });

  it('lists tags by use and words streaks', () => {
    expect(
      tagsOf([
        done('a', 'x', ['home']),
        done('b', 'x', ['work', 'home']),
        done('c', 'x', ['work', 'home']),
      ]),
    ).toEqual(['home', 'work']);
    expect([0, 1, 9].map(daysText)).toEqual(['—', '1 day', '9 days']);
  });
});
