import { describe, expect, it } from 'vitest';
import {
  eventToken,
  findConflicts,
  formatBinding,
  parseBinding,
  resolve,
  type Binding,
  type KeyLike,
} from './keymap';

const ev = (key: string, mods: Partial<KeyLike> = {}): KeyLike => ({
  key,
  ctrlKey: false,
  altKey: false,
  shiftKey: false,
  metaKey: false,
  ...mods,
});

describe('eventToken', () => {
  it('normalises letters, modifiers and named keys', () => {
    expect(eventToken(ev('k', { ctrlKey: true }))).toBe('ctrl+k');
    expect(eventToken(ev('K', { ctrlKey: true, shiftKey: true }))).toBe('ctrl+shift+k');
    expect(eventToken(ev('ArrowUp', { shiftKey: true }))).toBe('shift+arrowup');
    expect(eventToken(ev(' '))).toBe('space');
    expect(eventToken(ev('Escape'))).toBe('escape');
  });

  it('drops Shift from printable symbols', () => {
    expect(eventToken(ev('?', { shiftKey: true }))).toBe('?');
    expect(eventToken(ev('/', { ctrlKey: true }))).toBe('ctrl+/');
  });

  it('ignores bare modifier presses', () => {
    expect(eventToken(ev('Shift', { shiftKey: true }))).toBeNull();
    expect(eventToken(ev('Control', { ctrlKey: true }))).toBeNull();
  });
});

describe('parseBinding', () => {
  it('parses single keys, modifiers and chords', () => {
    expect(parseBinding('Ctrl+K')).toEqual(['ctrl+k']);
    expect(parseBinding('g w')).toEqual(['g', 'w']);
    expect(parseBinding('Del')).toEqual(['delete']);
    expect(parseBinding('Shift+ArrowUp')).toEqual(['shift+arrowup']);
    expect(parseBinding('?')).toEqual(['?']);
    expect(parseBinding('Ctrl+,')).toEqual(['ctrl+,']);
    expect(parseBinding('+')).toEqual(['+']);
  });

  it('matches what eventToken produces', () => {
    expect(parseBinding('Ctrl+/')[0]).toBe(eventToken(ev('/', { ctrlKey: true })));
    expect(parseBinding('Shift+?')[0]).toBe(eventToken(ev('?', { shiftKey: true })));
  });

  it('rejects junk', () => {
    expect(() => parseBinding('')).toThrow();
    expect(() => parseBinding('Hyper+K')).toThrow();
  });
});

describe('formatBinding', () => {
  it('renders for chips and tooltips', () => {
    expect(formatBinding('ctrl+k')).toBe('Ctrl+K');
    expect(formatBinding('g w')).toBe('g w');
    expect(formatBinding('Shift+ArrowUp')).toBe('Shift+↑');
    expect(formatBinding('Del')).toBe('Del');
  });
});

describe('resolve', () => {
  const B = (actionId: string, binding: string, context = 'global'): Binding => ({
    actionId,
    sequence: parseBinding(binding),
    context,
  });
  const bindings = [
    B('palette', 'Ctrl+K'),
    B('quick', 'n'),
    B('go.week', 'g w'),
    B('go.diary', 'g d'),
    B('complete', 'x', 'diary'),
    B('week.complete', 'x', 'week'),
  ];
  const ctx = new Set(['diary']);

  it('runs single-key and modified bindings', () => {
    expect(resolve(bindings, [], 'n', ctx, false)).toEqual({ type: 'run', actionId: 'quick' });
    expect(resolve(bindings, [], 'ctrl+k', ctx, false)).toEqual({
      type: 'run',
      actionId: 'palette',
    });
  });

  it('chords go pending, then run', () => {
    const first = resolve(bindings, [], 'g', ctx, false);
    expect(first).toEqual({ type: 'pending', prefix: ['g'], continuations: ['w', 'd'] });
    expect(resolve(bindings, ['g'], 'w', ctx, false)).toEqual({
      type: 'run',
      actionId: 'go.week',
    });
  });

  it('a broken chord retries the key on its own', () => {
    expect(resolve(bindings, ['g'], 'n', ctx, false)).toEqual({ type: 'run', actionId: 'quick' });
    expect(resolve(bindings, ['g'], 'z', ctx, false)).toEqual({ type: 'none' });
  });

  it('respects contexts', () => {
    expect(resolve(bindings, [], 'x', ctx, false)).toEqual({ type: 'run', actionId: 'complete' });
    expect(resolve(bindings, [], 'x', new Set(['week']), false)).toEqual({
      type: 'run',
      actionId: 'week.complete',
    });
    expect(resolve(bindings, [], 'x', new Set(['month']), false)).toEqual({ type: 'none' });
  });

  it('ignores plain keys while typing but not command keys', () => {
    expect(resolve(bindings, [], 'n', ctx, true)).toEqual({ type: 'none' });
    expect(resolve(bindings, [], 'g', ctx, true)).toEqual({ type: 'none' });
    expect(resolve(bindings, [], 'ctrl+k', ctx, true)).toEqual({
      type: 'run',
      actionId: 'palette',
    });
  });
});

describe('findConflicts', () => {
  it('flags equal and prefix sequences in overlapping contexts', () => {
    const B = (actionId: string, binding: string, context = 'global'): Binding => ({
      actionId,
      sequence: parseBinding(binding),
      context,
    });
    const conflicts = findConflicts([
      B('a', 'g'),
      B('b', 'g w'),
      B('c', 'x', 'diary'),
      B('d', 'x', 'week'),
      B('e', 'Ctrl+K'),
      B('f', 'Ctrl+K', 'diary'),
    ]);
    expect(conflicts.map(([x, y]) => `${x.actionId}/${y.actionId}`)).toEqual(['a/b', 'e/f']);
  });
});
