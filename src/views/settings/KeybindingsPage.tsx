import { useEffect, useMemo, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  ArrowCounterClockwise,
  DownloadSimple,
  LockSimple,
  MagnifyingGlass,
  UploadSimple,
  X,
} from '@phosphor-icons/react';
import { ACTION_GROUPS, buildActions, NAV_CONTEXT, type Action } from '@/actions/actions';
import { useOverlay } from '@/app/overlay';
import { Button } from '@/components/ui/button';
import { KeyChips } from '@/components/ui/kbd';
import { useKeybindings } from '@/keymap/keybindingStore';
import { eventToken, formatBinding, type KeyToken } from '@/keymap/keymap';
import {
  assign,
  clashes,
  cleanImport,
  effectiveBindings,
  FIXED_ACTIONS,
  isOverridden,
  tokensToBinding,
  type Clash,
} from '@/keymap/overrides';
import { isOpError } from '@/lib/api';
import { cn } from '@/lib/cn';
import { QuickCaptureInfo } from './QuickCaptureInfo';

/** A chord's next key may follow within this long; then the capture ends. */
const CAPTURE_PAUSE_MS = 1000;
/** Longest chord accepted. */
const MAX_STEPS = 3;

const errorText = (e: unknown) => (isOpError(e) ? e.message : String(e));

const CONTEXT_LABEL: Record<string, string> = {
  [NAV_CONTEXT]: 'Calendar views',
  week: 'Week',
  inbox: 'Inbox',
};

/** What a list of clashes means, e.g. "n is already Quick add". */
function clashText(binding: string, found: readonly Clash[]): string {
  const names = found.map((c) => `${c.action.label} (${formatBinding(c.binding)})`);
  if (found.every((c) => c.exact)) {
    return `${formatBinding(binding)} is already ${names.join(' and ')}.`;
  }
  return `${formatBinding(binding)} would get in the way of ${names.join(' and ')}: one starts with the other.`;
}

interface Pending {
  actionId: string;
  binding: string;
  found: Clash[];
}

/**
 * Listens for a new binding while `active`: keys are collected into a
 * chord until a pause, then `onDone` gets them. Esc cancels. Nothing else
 * in the app sees these keys.
 */
function useCapture(active: boolean, onDone: (tokens: KeyToken[] | null) => void) {
  const [tokens, setTokens] = useState<KeyToken[]>([]);
  const done = useRef(onDone);
  useEffect(() => {
    done.current = onDone;
  });
  useEffect(() => {
    if (!active) return;
    let seq: KeyToken[] = [];
    let timer: number | undefined;
    const finish = (result: KeyToken[] | null) => {
      window.clearTimeout(timer);
      seq = [];
      setTokens([]);
      done.current(result);
    };
    const onKey = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopImmediatePropagation();
      if (e.repeat) return;
      const token = eventToken(e);
      if (!token) return;
      if (token === 'escape') {
        finish(null);
        return;
      }
      seq = [...seq, token];
      setTokens(seq);
      window.clearTimeout(timer);
      if (seq.length >= MAX_STEPS) finish(seq);
      else timer = window.setTimeout(() => finish(seq), CAPTURE_PAUSE_MS);
    };
    // Capture phase, ahead of the app's own key handler.
    window.addEventListener('keydown', onKey, true);
    return () => {
      window.removeEventListener('keydown', onKey, true);
      window.clearTimeout(timer);
    };
  }, [active]);
  return tokens;
}

function Row({
  action,
  capturing,
  onCapture,
  onUnbind,
  onReset,
  overridden,
  bindings,
}: {
  action: Action;
  capturing: KeyToken[] | null;
  onCapture: () => void;
  onUnbind: () => void;
  onReset: () => void;
  overridden: boolean;
  bindings: string[];
}) {
  const fixed = FIXED_ACTIONS.has(action.id);
  const context = action.context ? (CONTEXT_LABEL[action.context] ?? action.context) : null;
  return (
    <div
      className={cn(
        'group flex min-h-[38px] items-center gap-3 border-b border-border px-3 py-1.5',
        capturing && 'bg-accent-tint',
      )}
    >
      <action.icon size={15} className="flex-none text-text-muted" />
      <span className="min-w-0 flex-1 truncate">
        {action.label}
        {context && <span className="ml-2 text-xs text-text-muted">{context}</span>}
      </span>
      {overridden && !capturing && (
        <span title="Changed from the default" className="size-1.5 rounded-full bg-accent" />
      )}
      {fixed ? (
        <span
          title="Set with the operating system's shortcut, so it can't be changed here."
          className="flex items-center gap-2"
        >
          {bindings.map((b) => (
            <KeyChips key={b} binding={b} />
          ))}
          <LockSimple size={13} className="text-text-muted" />
        </span>
      ) : (
        <button
          type="button"
          onClick={onCapture}
          aria-label={`Change the keys for ${action.label}`}
          className={cn(
            'flex min-h-[26px] min-w-[120px] cursor-pointer items-center justify-end gap-2 rounded-md border px-2 outline-none focus-visible:border-accent',
            capturing ? 'border-accent bg-bg' : 'border-transparent hover:border-border-strong',
          )}
        >
          {capturing ? (
            capturing.length ? (
              <KeyChips binding={tokensToBinding(capturing)} />
            ) : (
              <span className="text-sm text-accent-text">Press the new keys… Esc cancels</span>
            )
          ) : bindings.length ? (
            bindings.map((b) => <KeyChips key={b} binding={b} />)
          ) : (
            <span className="text-sm text-text-muted">Not set</span>
          )}
        </button>
      )}
      {!fixed && (
        <span className="flex w-[52px] flex-none justify-end gap-0.5 opacity-0 group-focus-within:opacity-100 group-hover:opacity-100">
          {bindings.length > 0 && (
            <button
              type="button"
              onClick={onUnbind}
              aria-label={`Remove the keys for ${action.label}`}
              title="Remove keys"
              className="grid size-6 cursor-pointer place-items-center rounded-sm text-text-muted hover:bg-hover"
            >
              <X size={13} />
            </button>
          )}
          {overridden && (
            <button
              type="button"
              onClick={onReset}
              aria-label={`Reset ${action.label} to its default`}
              title="Reset to default"
              className="grid size-6 cursor-pointer place-items-center rounded-sm text-text-muted hover:bg-hover"
            >
              <ArrowCounterClockwise size={13} />
            </button>
          )}
        </span>
      )}
    </div>
  );
}

/** Settings → Keybindings (roadmap §5.4, design S10). */
export function KeybindingsPage() {
  const actions = useMemo(() => buildActions(), []);
  const overrides = useKeybindings((s) => s.overrides);
  const replace = useKeybindings((s) => s.replace);
  const ask = useOverlay((s) => s.ask);
  const [query, setQuery] = useState('');
  const [capturing, setCapturing] = useState<string | null>(null);
  const [pending, setPending] = useState<Pending | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const save = (next: Record<string, string>, message: string) => {
    setError(null);
    replace(next)
      .then(() => setStatus(message))
      .catch((e: unknown) => setError(errorText(e)));
  };
  const labelOf = (id: string) => actions.find((a) => a.id === id)?.label ?? id;

  const commit = (actionId: string, binding: string, swap: boolean) => {
    setPending(null);
    save(
      assign(actions, overrides, actionId, binding, swap),
      `${labelOf(actionId)} is now ${formatBinding(binding)}.`,
    );
  };

  const captured = useCapture(capturing !== null, (tokens) => {
    const actionId = capturing;
    setCapturing(null);
    if (!actionId || !tokens) return;
    const binding = tokensToBinding(tokens);
    const found = clashes(actions, overrides, actionId, binding);
    if (found.length) setPending({ actionId, binding, found });
    else commit(actionId, binding, false);
  });

  const words = query.trim().toLowerCase().split(/\s+/).filter(Boolean);
  const matches = (a: Action) => {
    const text = [a.label, a.id, a.group, ...(a.keywords ?? [])]
      .concat(effectiveBindings(a, overrides).map(formatBinding))
      .join(' ')
      .toLowerCase();
    return words.every((w) => text.includes(w));
  };
  const changed = Object.keys(overrides).length;

  const startImport = async () => {
    setError(null);
    try {
      const file = await invoke<Record<string, string> | null>('pick_keybindings_import');
      if (!file) return;
      const { overrides: next, skipped } = cleanImport(actions, file);
      const count = Object.keys(next).length;
      ask({
        title: 'Import these keybindings?',
        body: [
          `They replace your current changes with ${count} from the file.`,
          skipped > 0 ? `${skipped} that don't fit this version of Blotter are left out.` : '',
        ]
          .filter(Boolean)
          .join(' '),
        confirmLabel: 'Import',
        onConfirm: () => save(next, `Imported ${count} keybindings.`),
      });
    } catch (e) {
      setError(errorText(e));
    }
  };

  const startExport = async () => {
    setError(null);
    try {
      const path = await invoke<string | null>('export_keybindings');
      if (path) setStatus(`Saved your keybindings to ${path}`);
    } catch (e) {
      setError(errorText(e));
    }
  };

  return (
    <div className="flex max-w-[820px] flex-col gap-[18px] px-8 pt-6 pb-12">
      <div className="flex flex-wrap items-center gap-2">
        <h1 className="m-0 flex-1 font-heading text-xl font-heading-weight">Keybindings</h1>
        <Button onClick={() => void startImport()}>
          <DownloadSimple />
          Import…
        </Button>
        <Button onClick={() => void startExport()}>
          <UploadSimple />
          Export…
        </Button>
        <Button
          disabled={changed === 0}
          onClick={() =>
            ask({
              title: 'Reset all keybindings?',
              body: `Your ${changed} ${changed === 1 ? 'change goes' : 'changes go'} back to the defaults.`,
              confirmLabel: 'Reset all',
              danger: true,
              onConfirm: () => save({}, 'Every keybinding is back to its default.'),
            })
          }
        >
          <ArrowCounterClockwise />
          Reset all
        </Button>
      </div>
      <p className="m-0 max-w-[600px] leading-normal text-text-muted">
        Click a binding, or Tab to it and press Enter, then press the new keys. For a chord like g
        then w, press them one after the other. Changes work straight away.
      </p>

      <QuickCaptureInfo />

      <label className="flex h-[34px] items-center gap-2 rounded-md border border-border-strong bg-bg px-2.5 focus-within:border-accent">
        <MagnifyingGlass className="text-text-muted" />
        <input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search actions or keys"
          aria-label="Search actions or keys"
          className="min-w-0 flex-1 bg-transparent outline-none"
        />
      </label>

      {pending && (
        <div
          role="alert"
          className="flex flex-col gap-2.5 rounded-lg border border-conflict bg-[color-mix(in_srgb,var(--color-conflict)_8%,var(--color-bg))] px-3.5 py-3"
        >
          <span className="text-sm">
            {clashText(pending.binding, pending.found)} Give it to {labelOf(pending.actionId)}?
          </span>
          <div className="flex flex-wrap gap-1.5">
            {pending.found.length === 1 && pending.found[0]?.exact && (
              <Button
                size="sm"
                variant="primary"
                autoFocus
                onClick={() => commit(pending.actionId, pending.binding, true)}
              >
                Swap
              </Button>
            )}
            <Button
              size="sm"
              variant={
                pending.found.length === 1 && pending.found[0]?.exact ? 'secondary' : 'primary'
              }
              onClick={() => commit(pending.actionId, pending.binding, false)}
            >
              Take it
            </Button>
            <Button size="sm" variant="ghost" onClick={() => setPending(null)}>
              Cancel
            </Button>
          </div>
          <span className="text-xs text-text-muted">
            Swap gives the other action this one&rsquo;s old keys; Take it leaves it without these.
          </span>
        </div>
      )}
      {status && !pending && <p className="m-0 text-sm text-text-muted">{status}</p>}
      {error && <p className="m-0 text-sm text-danger">{error}</p>}

      {ACTION_GROUPS.map((group) => {
        const rows = actions.filter((a) => a.group === group && matches(a));
        if (rows.length === 0) return null;
        return (
          <section key={group} className="flex flex-col">
            <h2 className="m-0 px-3 pb-1.5 text-xs font-medium tracking-[.06em] text-text-muted uppercase">
              {group}
            </h2>
            <div className="overflow-hidden rounded-lg border border-b-0 border-border">
              {rows.map((a) => (
                <Row
                  key={a.id}
                  action={a}
                  bindings={effectiveBindings(a, overrides)}
                  overridden={isOverridden(a, overrides)}
                  capturing={capturing === a.id ? captured : null}
                  onCapture={() => {
                    setPending(null);
                    setStatus(null);
                    setCapturing(a.id);
                  }}
                  onUnbind={() => save({ ...overrides, [a.id]: '' }, `${a.label} has no keys now.`)}
                  onReset={() => {
                    const next = { ...overrides };
                    delete next[a.id];
                    save(next, `${a.label} is back to its default.`);
                  }}
                />
              ))}
            </div>
          </section>
        );
      })}
    </div>
  );
}
