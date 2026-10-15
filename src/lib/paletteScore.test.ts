import { describe, expect, it } from 'vitest';
import { paletteScore } from './paletteScore';

const rank = (labels: string[], search: string) =>
  labels
    .map((l) => ({ l, s: paletteScore(l, search) }))
    .filter((x) => x.s > 0)
    .sort((a, b) => b.s - a.s)
    .map((x) => x.l);

describe('paletteScore', () => {
  it('puts the exact command first and drops loose matches', () => {
    const labels = ['Previous period', 'Next period', 'Diary ↔ Calendar', 'Redo', 'Undo'];
    expect(rank(labels, 'redo')).toEqual(['Redo']);
    expect(rank(labels, 'undo')).toEqual(['Undo']);
  });

  it('ranks prefix, word start, then substring', () => {
    const labels = ['Week', 'Go to week', 'Weekly review', 'Midweek'];
    expect(rank(labels, 'week')).toEqual(['Week', 'Weekly review', 'Go to week', 'Midweek']);
  });

  it('matches several words in any order', () => {
    expect(paletteScore('Theme: Dark', 'theme dark')).toBeGreaterThan(0.5);
    expect(paletteScore('Theme: Dark', 'dark theme')).toBeGreaterThan(0.5);
  });

  it('uses keywords, and abbreviations from word starts', () => {
    expect(paletteScore('Settings', 'preferences', ['preferences', 'options'])).toBe(0.5);
    expect(paletteScore('Quick capture (system-wide)', 'qc')).toBe(0.1);
    expect(paletteScore('Quick capture', 'xyz')).toBe(0);
  });

  it('keeps every row for an empty search', () => {
    expect(paletteScore('Anything', '  ')).toBe(1);
  });
});
