// The user's keybinding overrides on top of the actions' defaults
// (roadmap §5.4), and the conflict rules the editor checks them with.
// Pure, so it is unit-tested; the store and page sit on top.

import { NAV_CONTEXT, NAV_VIEWS, type Action } from '@/actions/actions';
import { parseBinding, type KeyToken } from './keymap';

/** Action id → binding; `""` means the user unbound it. */
export type Overrides = Readonly<Record<string, string>>;

/**
 * Actions whose keys aren't Blotter's to change: the capture shortcut is
 * registered with the operating system (and, on Wayland, by the user's
 * compositor).
 */
export const FIXED_ACTIONS: ReadonlySet<string> = new Set(['capture.open']);

/** The bindings an action answers to now: its override, else its defaults. */
export function effectiveBindings(action: Action, overrides: Overrides): string[] {
  const o = overrides[action.id];
  if (o !== undefined && !FIXED_ACTIONS.has(action.id)) return o ? [o] : [];
  return [action.binding, ...(action.altBindings ?? [])].filter((b): b is string => !!b);
}

export const isOverridden = (action: Action, overrides: Overrides): boolean =>
  overrides[action.id] !== undefined && !FIXED_ACTIONS.has(action.id);

/**
 * The actions with overrides applied, for the keymap, palette and sheet.
 * An overridden action drops its alternates and its family summary on the
 * shortcut sheet (that summary describes the defaults).
 */
export function applyOverrides(actions: readonly Action[], overrides: Overrides): Action[] {
  return actions.map((a) => {
    if (!isOverridden(a, overrides)) return a;
    const [binding] = effectiveBindings(a, overrides);
    return {
      ...a,
      binding,
      altBindings: [],
      sheet: a.sheet === false ? false : undefined,
    };
  });
}

/** Whether bindings in contexts `a` and `b` can both be live at once. */
export function contextsOverlap(a: string, b: string): boolean {
  if (a === b || a === 'global' || b === 'global') return true;
  // The roving-focus keys (nav) are live in every view that has them.
  const nav = (x: string, y: string) => x === NAV_CONTEXT && NAV_VIEWS.includes(y);
  return nav(a, b) || nav(b, a);
}

const startsWith = (seq: readonly KeyToken[], prefix: readonly KeyToken[]) =>
  prefix.length <= seq.length && prefix.every((t, i) => seq[i] === t);

export interface Clash {
  action: Action;
  binding: string;
  /** Same keys exactly (a swap makes sense), rather than one prefixing the other. */
  exact: boolean;
}

/**
 * What giving `actionId` the binding `binding` would clash with: another
 * action's binding equal to it, or one a chord prefix of the other (`g`
 * against `g w`), in a context live at the same time.
 */
export function clashes(
  actions: readonly Action[],
  overrides: Overrides,
  actionId: string,
  binding: string,
): Clash[] {
  const target = actions.find((a) => a.id === actionId);
  if (!target) return [];
  const seq = parseBinding(binding);
  const context = target.context ?? 'global';
  return actions.flatMap((a) => {
    if (a.id === actionId || !contextsOverlap(context, a.context ?? 'global')) return [];
    return effectiveBindings(a, overrides).flatMap((b) => {
      const other = parseBinding(b);
      if (!startsWith(seq, other) && !startsWith(other, seq)) return [];
      return [{ action: a, binding: b, exact: other.length === seq.length }];
    });
  });
}

/**
 * Overrides after giving `actionId` the binding `binding`. With `swap`, a
 * single exact clash takes over `actionId`'s old first binding; otherwise
 * every clashing action loses the clashing binding (its others stay).
 */
export function assign(
  actions: readonly Action[],
  overrides: Overrides,
  actionId: string,
  binding: string,
  swap: boolean,
): Record<string, string> {
  const target = actions.find((a) => a.id === actionId);
  const next: Record<string, string> = { ...overrides, [actionId]: binding };
  if (!target) return next;
  const found = clashes(actions, overrides, actionId, binding);
  const old = effectiveBindings(target, overrides)[0] ?? '';
  for (const c of found) {
    if (swap && found.length === 1 && c.exact) {
      next[c.action.id] = old;
      continue;
    }
    const left = effectiveBindings(c.action, overrides).filter((b) => b !== c.binding);
    next[c.action.id] = left[0] ?? '';
  }
  return next;
}

/** A captured key sequence as a stored binding (`["g", "w"]` → `"g w"`). */
export const tokensToBinding = (tokens: readonly KeyToken[]): string => tokens.join(' ');

/** Same keys, however they were written (`"Ctrl+K"` and `"ctrl+k"`). */
export function sameBinding(a: string, b: string): boolean {
  try {
    return parseBinding(a).join(' ') === parseBinding(b).join(' ');
  } catch {
    return false;
  }
}

/**
 * The usable part of an imported file: known, changeable actions with
 * bindings that parse. `skipped` counts the rest.
 */
export function cleanImport(
  actions: readonly Action[],
  bindings: Readonly<Record<string, string>>,
): { overrides: Record<string, string>; skipped: number } {
  const known = new Set(actions.map((a) => a.id));
  const overrides: Record<string, string> = {};
  let skipped = 0;
  for (const [id, binding] of Object.entries(bindings)) {
    let ok = known.has(id) && !FIXED_ACTIONS.has(id) && typeof binding === 'string';
    if (ok && binding) {
      try {
        parseBinding(binding);
      } catch {
        ok = false;
      }
    }
    if (ok) overrides[id] = binding;
    else skipped++;
  }
  return { overrides, skipped };
}
