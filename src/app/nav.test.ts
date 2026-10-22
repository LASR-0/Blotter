import { describe, expect, it } from 'vitest';
import { midnight, rolledAnchor, useNav } from './nav';

const day = (d: number, h = 0) => new Date(2026, 9, d, h, 30);

describe('rolledAnchor', () => {
  it('moves an anchor on yesterday to the new today', () => {
    expect(rolledAnchor(midnight(day(21)), day(21), day(22, 7))).toEqual(midnight(day(22)));
  });

  it('leaves an anchor the user moved elsewhere', () => {
    const elsewhere = midnight(day(28));
    expect(rolledAnchor(elsewhere, day(21), day(22))).toBe(elsewhere);
  });

  it('changes nothing within the same day', () => {
    const anchor = midnight(day(21));
    expect(rolledAnchor(anchor, day(21, 1), day(21, 23))).toBe(anchor);
  });

  it('catches up after days asleep', () => {
    expect(rolledAnchor(midnight(day(21)), day(21), day(24, 9))).toEqual(midnight(day(24)));
  });
});

describe('newDay', () => {
  it('moves the store anchor with the date', () => {
    useNav.setState({ anchor: midnight(day(21)) });
    useNav.getState().newDay(day(21), day(22));
    expect(useNav.getState().anchor).toEqual(midnight(day(22)));
  });
});
