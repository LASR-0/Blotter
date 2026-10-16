import { describe, expect, it } from 'vitest';
import { Keyboard } from '@phosphor-icons/react';
import type { Action } from '@/actions/actions';
import {
  applyOverrides,
  assign,
  clashes,
  cleanImport,
  contextsOverlap,
  effectiveBindings,
  sameBinding,
  tokensToBinding,
} from './overrides';

const act = (id: string, binding?: string, o: Partial<Action> = {}): Action => ({
  id,
  label: id,
  group: 'Actions',
  icon: Keyboard,
  binding,
  run: () => {},
  ...o,
});

const actions = [
  act('palette.toggle', 'Ctrl+K'),
  act('quick.add', 'n'),
  act('go.week', 'g w'),
  act('go.month', 'g m'),
  act('focus.up', 'ArrowUp', {
    altBindings: ['k'],
    context: 'nav',
    sheet: { label: 'Move', keys: 'h j k l' },
  }),
  act('week.cursor', 'Tab', { context: 'week' }),
  act('inbox.select', 'Space', { context: 'inbox' }),
  act('theme.light'),
  act('capture.open', 'Ctrl+Shift+Space'),
];
const byId = (id: string) => actions.find((a) => a.id === id) as Action;

describe('keybinding overrides', () => {
  it('replaces defaults, or unbinds with ""', () => {
    expect(effectiveBindings(byId('focus.up'), {})).toEqual(['ArrowUp', 'k']);
    expect(effectiveBindings(byId('focus.up'), { 'focus.up': 'w' })).toEqual(['w']);
    expect(effectiveBindings(byId('quick.add'), { 'quick.add': '' })).toEqual([]);
    // The OS-level capture shortcut can't be changed here.
    expect(effectiveBindings(byId('capture.open'), { 'capture.open': 'c' })).toEqual([
      'Ctrl+Shift+Space',
    ]);
  });

  it('applies to the actions the keymap, palette and sheet read', () => {
    const out = applyOverrides(actions, { 'focus.up': 'w', 'theme.light': 't l' });
    const up = out.find((a) => a.id === 'focus.up');
    expect(up).toMatchObject({ binding: 'w', altBindings: [], sheet: undefined });
    expect(out.find((a) => a.id === 'theme.light')?.binding).toBe('t l');
    expect(out.find((a) => a.id === 'quick.add')).toBe(byId('quick.add'));
  });

  it('knows which contexts are live together', () => {
    expect(contextsOverlap('global', 'week')).toBe(true);
    expect(contextsOverlap('nav', 'inbox')).toBe(true);
    expect(contextsOverlap('week', 'inbox')).toBe(false);
    expect(contextsOverlap('nav', 'settings')).toBe(false);
  });

  it('finds exact and chord-prefix clashes', () => {
    expect(clashes(actions, {}, 'theme.light', 'n')).toMatchObject([
      { action: { id: 'quick.add' }, binding: 'n', exact: true },
    ]);
    // "g" would swallow both g-chords.
    expect(clashes(actions, {}, 'theme.light', 'g').map((c) => c.action.id)).toEqual([
      'go.week',
      'go.month',
    ]);
    expect(clashes(actions, {}, 'theme.light', 'g').every((c) => !c.exact)).toBe(true);
    // An alternate counts; contexts that are never live together don't.
    expect(clashes(actions, {}, 'quick.add', 'k')[0]?.action.id).toBe('focus.up');
    expect(clashes(actions, {}, 'week.cursor', 'Space')).toEqual([]);
    expect(clashes(actions, {}, 'quick.add', 'ctrl+k')[0]?.action.id).toBe('palette.toggle');
  });

  it('swaps a single exact clash, or takes the keys', () => {
    expect(assign(actions, {}, 'palette.toggle', 'n', true)).toEqual({
      'palette.toggle': 'n',
      'quick.add': 'Ctrl+K',
    });
    expect(assign(actions, {}, 'palette.toggle', 'n', false)).toEqual({
      'palette.toggle': 'n',
      'quick.add': '',
    });
    // Taking "k" leaves focus.up its arrow.
    expect(assign(actions, {}, 'quick.add', 'k', false)).toEqual({
      'quick.add': 'k',
      'focus.up': 'ArrowUp',
    });
    // A prefix clash can't swap; both chords lose their keys.
    expect(assign(actions, {}, 'theme.light', 'g', true)).toEqual({
      'theme.light': 'g',
      'go.week': '',
      'go.month': '',
    });
  });

  it('stores captured keys and compares them however written', () => {
    expect(tokensToBinding(['g', 'w'])).toBe('g w');
    expect(sameBinding('Ctrl+K', 'ctrl+k')).toBe(true);
    expect(sameBinding('g w', 'g m')).toBe(false);
  });

  it('keeps only usable bindings from an import', () => {
    expect(
      cleanImport(actions, {
        'quick.add': 'a',
        'go.week': '',
        'no.such.action': 'z',
        'capture.open': 'c',
        'palette.toggle': 'Hyper+K',
      }),
    ).toEqual({ overrides: { 'quick.add': 'a', 'go.week': '' }, skipped: 3 });
  });
});
