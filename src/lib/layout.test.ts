import { describe, expect, it } from 'vitest';
import { layoutAllDay, layoutDay } from './layout';

const byId = <T extends { id: string }>(xs: T[]) => Object.fromEntries(xs.map((x) => [x.id, x]));

describe('layoutDay', () => {
  it('gives non-overlapping items the full width', () => {
    const out = byId(
      layoutDay([
        { id: 'a', start: 420, end: 450 },
        { id: 'b', start: 450, end: 480 },
      ]),
    );
    expect(out.a).toMatchObject({ lane: 0, lanes: 1 });
    expect(out.b).toMatchObject({ lane: 0, lanes: 1 });
  });

  it('splits an overlap into lanes (Dentist vs interview)', () => {
    const out = byId(
      layoutDay([
        { id: 'dentist', start: 900, end: 945 },
        { id: 'interview', start: 930, end: 950 },
      ]),
    );
    expect(out.dentist).toMatchObject({ lane: 0, lanes: 2 });
    expect(out.interview).toMatchObject({ lane: 1, lanes: 2 });
  });

  it('reuses a freed lane and shares the cluster lane count', () => {
    const out = byId(
      layoutDay([
        { id: 'long', start: 600, end: 720 },
        { id: 'x', start: 600, end: 630 },
        { id: 'y', start: 640, end: 700 },
        { id: 'later', start: 800, end: 830 },
      ]),
    );
    expect(out.long).toMatchObject({ lane: 0, lanes: 2 });
    expect(out.x).toMatchObject({ lane: 1, lanes: 2 });
    expect(out.y).toMatchObject({ lane: 1, lanes: 2 });
    expect(out.later).toMatchObject({ lane: 0, lanes: 1 });
  });

  it('gives zero-length items a minimum body', () => {
    const [p] = layoutDay([{ id: 'r', start: 600, end: 600 }]);
    expect(p?.end).toBe(615);
  });
});

describe('layoutAllDay', () => {
  it('stacks overlapping ranges and clips to the week', () => {
    const out = byId(
      layoutAllDay(
        [
          { id: 'visit', first: 5, last: 9 },
          { id: 'bin', first: 1, last: 1 },
          { id: 'leave', first: -2, last: 2 },
          { id: 'gone', first: -5, last: -1 },
        ],
        7,
      ),
    );
    expect(out.leave).toMatchObject({ first: 0, last: 2, row: 0 });
    expect(out.bin).toMatchObject({ row: 1 });
    expect(out.visit).toMatchObject({ first: 5, last: 6, row: 0 });
    expect(out.gone).toBeUndefined();
  });
});
