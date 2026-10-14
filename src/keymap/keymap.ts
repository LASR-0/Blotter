// Pure keymap logic: normalising key events, parsing binding strings and
// resolving chord sequences. No DOM or timers, so it is fully unit-tested.
//
// Binding syntax (shown to users and stored in settings):
//   "Ctrl+K"   one keystroke with modifiers, joined by +
//   "g w"      a chord: g, then w (space-separated steps)
//   "?" "." "[" "Del" "Shift+ArrowUp" "Alt+ArrowDown"

/** A normalised keystroke, e.g. `ctrl+k`, `shift+arrowup`, `?`, `g`. */
export type KeyToken = string;

export interface KeyLike {
  key: string;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  metaKey: boolean;
}

const MODIFIER_KEYS = new Set(['control', 'shift', 'alt', 'meta', 'altgraph', 'os']);

const KEY_ALIASES: Record<string, string> = {
  ' ': 'space',
  spacebar: 'space',
  esc: 'escape',
  del: 'delete',
  up: 'arrowup',
  down: 'arrowdown',
  left: 'arrowleft',
  right: 'arrowright',
  return: 'enter',
};

const MOD_ORDER = ['ctrl', 'alt', 'shift', 'meta'] as const;
type Mod = (typeof MOD_ORDER)[number];

const MOD_ALIASES: Record<string, Mod> = {
  ctrl: 'ctrl',
  control: 'ctrl',
  alt: 'alt',
  option: 'alt',
  shift: 'shift',
  meta: 'meta',
  cmd: 'meta',
  super: 'meta',
  win: 'meta',
};

function normalizeKey(key: string): string {
  const lower = key.length === 1 ? key.toLowerCase() : key.toLowerCase();
  return KEY_ALIASES[lower] ?? lower;
}

function compose(mods: Set<Mod>, key: string): KeyToken {
  // A printable non-letter (e.g. "?", "[", "/") already reflects Shift, so
  // Shift is dropped to make "?" match regardless of keyboard layout.
  if (key.length === 1 && !/[a-z]/.test(key)) mods.delete('shift');
  const parts: string[] = MOD_ORDER.filter((m) => mods.has(m));
  parts.push(key);
  return parts.join('+');
}

/** Normalises a keyboard event; returns null for bare modifier presses. */
export function eventToken(e: KeyLike): KeyToken | null {
  const key = normalizeKey(e.key);
  if (MODIFIER_KEYS.has(key) || key === 'dead' || key === 'unidentified') return null;
  const mods = new Set<Mod>();
  if (e.ctrlKey) mods.add('ctrl');
  if (e.altKey) mods.add('alt');
  if (e.shiftKey) mods.add('shift');
  if (e.metaKey) mods.add('meta');
  return compose(mods, key);
}

function parseStep(step: string): KeyToken {
  // Split on "+" but keep a literal "+" key ("Ctrl++" or "+").
  const parts = step === '+' ? ['+'] : step.split(/\+(?!$)/);
  const key = normalizeKey(parts.pop() ?? '');
  const mods = new Set<Mod>();
  for (const p of parts) {
    const mod = MOD_ALIASES[p.toLowerCase()];
    if (!mod) throw new Error(`Unknown modifier "${p}" in "${step}"`);
    mods.add(mod);
  }
  if (!key) throw new Error(`Empty key in "${step}"`);
  return compose(mods, key);
}

/** Parses `"g w"` into `["g", "w"]`, `"Ctrl+K"` into `["ctrl+k"]`. */
export function parseBinding(binding: string): KeyToken[] {
  const steps = binding.trim().split(/\s+/).filter(Boolean);
  if (steps.length === 0) throw new Error('Empty binding');
  return steps.map(parseStep);
}

/** True if the token includes Ctrl, Alt or Meta (usable while typing). */
export function hasCommandModifier(token: KeyToken): boolean {
  return /^(ctrl|alt|meta)\+/.test(token);
}

const DISPLAY: Record<string, string> = {
  ctrl: 'Ctrl',
  alt: 'Alt',
  shift: 'Shift',
  meta: 'Super',
  arrowup: '↑',
  arrowdown: '↓',
  arrowleft: '←',
  arrowright: '→',
  enter: '↵',
  escape: 'Esc',
  delete: 'Del',
  backspace: '⌫',
  space: 'Space',
  tab: 'Tab',
};

/** Human form of a binding: `"ctrl+k"` → `"Ctrl+K"`, chords stay space-separated. */
export function formatBinding(binding: string): string {
  return parseBinding(binding)
    .map((token) => {
      const parts = token.split(/\+(?!$)/);
      const key = parts.pop() ?? '';
      const mods = parts.map((m) => DISPLAY[m] ?? m);
      // Letters are uppercase after a modifier (Ctrl+K) but stay as typed
      // on their own, since "n" and "N" (Shift+n) differ.
      const shownKey =
        DISPLAY[key] ??
        (key.length === 1
          ? mods.length > 0
            ? key.toUpperCase()
            : key
          : key.charAt(0).toUpperCase() + key.slice(1));
      return [...mods, shownKey].join('+');
    })
    .join(' ');
}

export interface Binding {
  actionId: string;
  sequence: KeyToken[];
  /** `global`, or a view/context id that must be active. */
  context: string;
}

export type Resolution =
  | { type: 'run'; actionId: string }
  | { type: 'pending'; prefix: KeyToken[]; continuations: KeyToken[] }
  | { type: 'none' };

const startsWith = (seq: KeyToken[], prefix: KeyToken[]) =>
  prefix.length <= seq.length && prefix.every((t, i) => seq[i] === t);

/**
 * Resolves one keystroke against the bindings, given any pending chord
 * prefix. Context-specific bindings win over global ones for the same keys.
 */
export function resolve(
  bindings: readonly Binding[],
  pending: readonly KeyToken[],
  token: KeyToken,
  activeContexts: ReadonlySet<string>,
  typing: boolean,
): Resolution {
  if (typing && pending.length === 0 && !hasCommandModifier(token)) return { type: 'none' };
  const live = bindings.filter((b) => b.context === 'global' || activeContexts.has(b.context));

  const attempt = (seq: KeyToken[]): Resolution => {
    const exact = live.filter(
      (b) => b.sequence.length === seq.length && startsWith(b.sequence, seq),
    );
    if (exact.length > 0) {
      const best = exact.find((b) => b.context !== 'global') ?? exact[0];
      return best ? { type: 'run', actionId: best.actionId } : { type: 'none' };
    }
    const longer = live.filter(
      (b) => b.sequence.length > seq.length && startsWith(b.sequence, seq),
    );
    if (longer.length > 0) {
      const continuations = [...new Set(longer.map((b) => b.sequence[seq.length] as KeyToken))];
      return { type: 'pending', prefix: seq, continuations };
    }
    return { type: 'none' };
  };

  const result = attempt([...pending, token]);
  // A key that breaks a chord starts fresh, so "g" then "n" still opens quick add.
  if (result.type === 'none' && pending.length > 0 && !(typing && !hasCommandModifier(token)))
    return attempt([token]);
  return result;
}

/** Bindings whose sequences collide (one equals or prefixes another) in overlapping contexts. */
export function findConflicts(bindings: readonly Binding[]): [Binding, Binding][] {
  const out: [Binding, Binding][] = [];
  for (let i = 0; i < bindings.length; i++)
    for (let j = i + 1; j < bindings.length; j++) {
      const a = bindings[i] as Binding;
      const b = bindings[j] as Binding;
      const overlap = a.context === b.context || a.context === 'global' || b.context === 'global';
      if (overlap && (startsWith(a.sequence, b.sequence) || startsWith(b.sequence, a.sequence)))
        out.push([a, b]);
    }
  return out;
}
