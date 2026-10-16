import { useEffect, useMemo, useRef, useState, type ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  ArrowCounterClockwise,
  DownloadSimple,
  LockSimple,
  MagnifyingGlass,
  UploadSimple,
  Warning,
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
    return `${formatBinding(binding)} is already bound to ${found.map((c) => c.action.label).join(' and ')}.`;
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
  conflict,
}: {
  action: Action;
  capturing: KeyToken[] | null;
  onCapture: () => void;
  onUnbind: () => void;
  onReset: () => void;
  overridden: boolean;
  bindings: string[];
  /** The clash to settle, drawn under the row. */
  conflict: ReactNode;
}) {
  const fixed = FIXED_ACTIONS.has(action.id);
  const context = fixed
    ? 'System'
    : action.context
      ? (CONTEXT_LABEL[action.context] ?? action.context)
      : 'Global';
  return (
    <div className="flex flex-col">
      <div
        role={fixed ? undefined : 'button'}
        tabIndex={fixed ? undefined : 0}
        aria-label={fixed ? undefined : `Change the keys for ${action.label}`}
        onClick={fixed ? undefined : onCapture}
        onKeyDown={(e) => {
          if (!fixed && e.key === 'Enter' && e.target === e.currentTarget) {
            e.preventDefault();
            onCapture();
          }
        }}
        title={
          fixed
            ? "Set with the operating system's shortcut, so it can't be changed here."
            : undefined
        }
        className={cn(
          'group flex h-[38px] items-center gap-3 rounded-md px-2.5',
          fixed ? 'cursor-default' : 'cursor-pointer hover:bg-hover',
          capturing && 'bg-hover',
        )}
      >
        <span className="flex min-w-0 flex-1 items-center gap-2">
          <span className="truncate">{action.label}</span>
          {overridden && (
            <span
              title="Changed from the default"
              className="size-1.5 flex-none rounded-full bg-accent"
            />
          )}
        </span>
        <span className="w-[90px] flex-none text-xs text-text-muted">{context}</span>
        {capturing ? (
          <span className="flex h-[26px] min-w-[170px] items-center justify-end gap-1.5 rounded-md border-[1.5px] border-dashed border-accent bg-accent-tint px-2 font-mono text-xs text-accent-text">
            {capturing.length ? (
              <KeyChips binding={tokensToBinding(capturing)} />
            ) : (
              <span className="animate-pulse">Press new keys… Esc to cancel</span>
            )}
          </span>
        ) : (
          <span className="flex min-w-[170px] items-center justify-end gap-1">
            {bindings.length ? (
              bindings.map((b) => <KeyChips key={b} binding={b} />)
            ) : (
              <span className="text-sm text-text-muted">Not set</span>
            )}
            {fixed && <LockSimple size={13} className="ml-1 text-text-muted" />}
          </span>
        )}
        <span className="flex w-[52px] flex-none justify-end gap-0.5">
          {!fixed && (
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                onReset();
              }}
              disabled={!overridden}
              aria-label={`Reset ${action.label} to its default`}
              title="Reset to default"
              className={cn(
                'grid size-6 cursor-pointer place-items-center rounded-sm text-text-muted hover:bg-hover',
                !overridden && 'invisible',
              )}
            >
              <ArrowCounterClockwise size={13} />
            </button>
          )}
          {!fixed && bindings.length > 0 && (
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                onUnbind();
              }}
              aria-label={`Remove the keys for ${action.label}`}
              title="Remove keys"
              className="grid size-6 cursor-pointer place-items-center rounded-sm text-text-muted opacity-0 group-focus-within:opacity-100 group-hover:opacity-100 hover:bg-hover"
            >
              <X size={13} />
            </button>
          )}
        </span>
      </div>
      {conflict}
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

      <label className="flex h-8 items-center gap-2 rounded-md border border-border-strong bg-surface px-2.5 focus-within:border-accent">
        <MagnifyingGlass className="text-text-muted" />
        <input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search actions or keys…"
          aria-label="Search actions or keys"
          className="min-w-0 flex-1 bg-transparent outline-none"
        />
        <span className="font-mono text-xs text-text-muted">
          {actions.filter(matches).length} actions
        </span>
      </label>

      {status && !pending && <p className="m-0 text-sm text-text-muted">{status}</p>}
      {error && <p className="m-0 text-sm text-danger">{error}</p>}

      {ACTION_GROUPS.map((group) => {
        const rows = actions.filter((a) => a.group === group && matches(a));
        if (rows.length === 0) return null;
        return (
          <section key={group} className="flex flex-col">
            <h2 className="m-0 px-2.5 py-1.5 text-xs font-normal tracking-[.06em] text-text-muted uppercase">
              {group}
            </h2>
            {rows.map((a) => {
              const clash = pending?.actionId === a.id ? pending : null;
              const canSwap = clash?.found.length === 1 && clash.found[0]?.exact;
              return (
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
                  conflict={
                    clash && (
                      <div
                        role="alert"
                        className="mt-1 mb-2 ml-2.5 flex items-center gap-2.5 rounded-md border border-conflict bg-[color-mix(in_srgb,var(--color-conflict)_8%,var(--color-bg))] px-3 py-2.5"
                      >
                        <Warning weight="fill" className="flex-none text-conflict" />
                        <span className="flex-1 leading-[1.45]">
                          {clashText(clash.binding, clash.found)}
                        </span>
                        {canSwap ? (
                          <Button
                            size="sm"
                            variant="primary"
                            autoFocus
                            title="The other action gets this one's old keys"
                            onClick={() => commit(clash.actionId, clash.binding, true)}
                          >
                            Swap
                          </Button>
                        ) : (
                          <Button
                            size="sm"
                            variant="primary"
                            autoFocus
                            title="The other actions lose these keys"
                            onClick={() => commit(clash.actionId, clash.binding, false)}
                          >
                            Take it
                          </Button>
                        )}
                        <Button size="sm" onClick={() => setPending(null)}>
                          Cancel
                        </Button>
                      </div>
                    )
                  }
                />
              );
            })}
          </section>
        );
      })}
    </div>
  );
}
