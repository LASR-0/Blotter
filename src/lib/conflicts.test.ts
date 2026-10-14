import { describe, expect, it } from 'vitest';
import { conflictMap, conflictText } from './conflicts';

const titleOf = (id: string) => ({ d: 'Dentist', p: 'Parent–teacher interview' })[id];

describe('conflictText', () => {
  it('names what an item overlaps', () => {
    expect(conflictText({ item_id: 'd', overlaps: ['p'], blockout: null }, titleOf)).toBe(
      'Overlaps Parent–teacher interview',
    );
    expect(conflictText({ item_id: 'x', overlaps: ['d', 'p', 'q'], blockout: null }, titleOf)).toBe(
      'Overlaps Dentist and 2 more',
    );
  });

  it('mentions a warning block-out, and nothing when there is no conflict', () => {
    expect(conflictText({ item_id: 'f', overlaps: [], blockout: 'Sleep' }, titleOf)).toBe(
      'Inside Sleep',
    );
    expect(conflictText({ item_id: 'f', overlaps: ['d'], blockout: 'Sleep' }, titleOf)).toBe(
      'Overlaps Dentist · Inside Sleep',
    );
    expect(conflictText(undefined, titleOf)).toBeNull();
  });

  it('keys conflicts by item', () => {
    expect(
      conflictMap([{ item_id: 'd', overlaps: ['p'], blockout: null }]).get('d')?.overlaps,
    ).toEqual(['p']);
    expect(conflictMap(undefined).size).toBe(0);
  });
});
