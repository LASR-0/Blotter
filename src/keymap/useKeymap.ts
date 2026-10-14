import { useEffect, useMemo, useRef } from 'react';
import type { Action } from '@/actions/actions';
import { useNav } from '@/app/nav';
import { eventToken, parseBinding, resolve, type Binding, type KeyToken } from './keymap';

/** How long an unfinished chord (e.g. `g`) waits for its next key. */
const CHORD_TIMEOUT_MS = 1200;

function isTyping(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  const tag = target.tagName;
  if (tag === 'TEXTAREA' || tag === 'SELECT') return true;
  if (tag !== 'INPUT') return false;
  const type = (target as HTMLInputElement).type;
  return !['checkbox', 'radio', 'button', 'submit', 'reset', 'range', 'color'].includes(type);
}

/**
 * Installs the global key handler. `contexts` are the active view contexts
 * (bindings with context `global` always apply). While an overlay is open
 * only actions marked `inOverlay` respond; the overlay handles the rest.
 */
export function useKeymap(
  actions: readonly Action[],
  contexts: readonly string[],
  overlayOpen: boolean,
): void {
  const bindings = useMemo<Binding[]>(
    () =>
      actions.flatMap((a) =>
        a.binding && (!overlayOpen || a.inOverlay)
          ? [{ actionId: a.id, sequence: parseBinding(a.binding), context: a.context ?? 'global' }]
          : [],
      ),
    [actions, overlayOpen],
  );
  const byId = useMemo(() => new Map(actions.map((a) => [a.id, a])), [actions]);
  const contextSet = useMemo(() => new Set(contexts), [contexts]);
  const pending = useRef<KeyToken[]>([]);
  const timer = useRef<number | undefined>(undefined);

  useEffect(() => {
    const setChord = useNav.getState().setChord;
    const clear = () => {
      pending.current = [];
      window.clearTimeout(timer.current);
      setChord(null);
    };

    const onKeyDown = (e: KeyboardEvent) => {
      if (e.defaultPrevented || e.isComposing || e.repeat) return;
      const token = eventToken(e);
      if (!token) return;
      const result = resolve(bindings, pending.current, token, contextSet, isTyping(e.target));
      if (result.type === 'none') {
        if (pending.current.length) clear();
        return;
      }
      e.preventDefault();
      if (result.type === 'pending') {
        pending.current = result.prefix;
        setChord({ prefix: result.prefix, continuations: result.continuations });
        window.clearTimeout(timer.current);
        timer.current = window.setTimeout(clear, CHORD_TIMEOUT_MS);
        return;
      }
      clear();
      byId.get(result.actionId)?.run();
    };

    window.addEventListener('keydown', onKeyDown);
    return () => {
      window.removeEventListener('keydown', onKeyDown);
      clear();
    };
  }, [bindings, byId, contextSet]);
}
