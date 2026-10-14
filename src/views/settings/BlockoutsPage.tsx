import { useState, type DragEvent, type KeyboardEvent } from 'react';
import { format, isBefore, parseISO, startOfToday } from 'date-fns';
import {
  CalendarPlus,
  CaretRight,
  DotsSixVertical,
  Plus,
  Prohibit,
  PushPin,
  Trash,
  Warning,
  X,
} from '@phosphor-icons/react';
import { useNav } from '@/app/nav';
import { EmptyState, ErrorPanel } from '@/components/EmptyState';
import { Button } from '@/components/ui/button';
import { Kbd } from '@/components/ui/kbd';
import { Switch } from '@/components/ui/switch';
import { isOpError, runBatch, runOp } from '@/lib/api';
import {
  blockoutSave,
  describeDays,
  draftFromBlockout,
  emptyBlockoutDraft,
  liftCalls,
  reorderCalls,
  validateBlockout,
  WEEKDAYS,
  type BlockoutDraft,
} from '@/lib/blockoutDraft';
import { cn } from '@/lib/cn';
import { parseTime } from '@/lib/format';
import { useOp } from '@/lib/queries';
import type { Blockout } from '@/lib/types';

const errorText = (e: unknown) =>
  isOpError(e) ? e.message : e instanceof Error ? e.message : String(e);

const LOCAL_TZ = Intl.DateTimeFormat().resolvedOptions().timeZone;

const FIELD =
  'h-[30px] rounded-md border border-border-strong bg-bg px-2.5 text-base text-text outline-none focus:border-accent';

/** A small hatched swatch, as the block-out appears on the calendar. */
function Swatch({ className }: { className?: string }) {
  return (
    <span
      className={cn(
        'flex-none rounded-[4px] border border-[color-mix(in_srgb,var(--color-blockout)_50%,transparent)] bg-[repeating-linear-gradient(135deg,color-mix(in_srgb,var(--color-blockout)_55%,transparent)_0_1.5px,transparent_1.5px_5px)]',
        className,
      )}
    />
  );
}

function when(b: Blockout): string {
  const parts = [describeDays(b.weekdays), `${b.start_time}–${b.end_time}`];
  if (b.tz !== LOCAL_TZ) parts.push(b.tz);
  return parts.join(' · ');
}

/** Settings → Block-outs (design S2): the list, the editor and its exceptions. */
export function BlockoutsPage() {
  const query = useOp<Blockout[]>('list_blockouts');
  const list = query.data ?? [];
  const selectedId = useNav((s) => s.blockoutId);
  const select = (id: string | null) => useNav.setState({ blockoutId: id });
  const [creating, setCreating] = useState(false);
  const [dragId, setDragId] = useState<string | null>(null);
  const [dropIndex, setDropIndex] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);

  const selected = creating ? null : (list.find((b) => b.id === selectedId) ?? list[0] ?? null);

  const reorder = (id: string, index: number) => {
    const calls = reorderCalls(list, id, index);
    if (calls.length) runBatch(calls).catch((e: unknown) => setError(errorText(e)));
  };

  const onRowKey = (e: KeyboardEvent, b: Blockout, i: number) => {
    if (e.altKey && (e.key === 'ArrowUp' || e.key === 'ArrowDown')) {
      e.preventDefault();
      e.stopPropagation();
      reorder(b.id, i + (e.key === 'ArrowUp' ? -1 : 1));
    } else if (!e.altKey && (e.key === 'ArrowUp' || e.key === 'ArrowDown')) {
      e.preventDefault();
      e.stopPropagation();
      const next = list[i + (e.key === 'ArrowUp' ? -1 : 1)];
      if (next) {
        select(next.id);
        document.querySelector<HTMLElement>(`[data-blockout-row="${next.id}"]`)?.focus();
      }
    }
  };

  const onDragOver = (e: DragEvent<HTMLElement>, i: number) => {
    if (!dragId) return;
    e.preventDefault();
    const r = e.currentTarget.getBoundingClientRect();
    setDropIndex(e.clientY < r.top + r.height / 2 ? i : i + 1);
  };

  const onDrop = () => {
    if (dragId !== null && dropIndex !== null) {
      const from = list.findIndex((b) => b.id === dragId);
      // Removing the row first shifts later positions up by one.
      reorder(dragId, dropIndex > from ? dropIndex - 1 : dropIndex);
    }
    setDragId(null);
    setDropIndex(null);
  };

  return (
    <div className="flex max-w-[820px] flex-col gap-[18px] px-8 pt-6 pb-12">
      <div className="flex items-center gap-2.5">
        <h1 className="m-0 flex-1 font-heading text-xl font-heading-weight">Block-outs</h1>
        <Button
          onClick={() => {
            setCreating(true);
            setError(null);
          }}
        >
          <Plus />
          New block-out
        </Button>
      </div>
      <p className="m-0 max-w-[560px] leading-normal text-text-muted">
        Flexible tasks are never placed inside a block-out. Fixed items, such as appointments, can
        be, and show a pin. Drag to reorder, or <Kbd>Alt</Kbd> <Kbd>↑</Kbd>/<Kbd>↓</Kbd>.
      </p>

      {query.isError && (
        <ErrorPanel
          title="Couldn't load block-outs"
          message={errorText(query.error)}
          onRetry={() => void query.refetch()}
        />
      )}
      {error && (
        <p role="alert" className="m-0 text-sm text-danger">
          {error}
        </p>
      )}

      {list.length === 0 && !creating && query.isSuccess ? (
        <EmptyState icon={Prohibit} title="No block-outs yet">
          Add one for time flexible tasks should never take, such as Work or Sleep.
        </EmptyState>
      ) : (
        <div
          role="listbox"
          aria-label="Block-outs"
          className="flex flex-col overflow-hidden rounded-lg border border-border"
          onDragLeave={(e) => {
            if (!e.currentTarget.contains(e.relatedTarget as Node | null)) setDropIndex(null);
          }}
        >
          {list.map((b, i) => (
            <div
              key={b.id}
              role="option"
              tabIndex={0}
              aria-selected={selected?.id === b.id}
              data-blockout-row={b.id}
              draggable
              onDragStart={(e) => {
                e.dataTransfer.effectAllowed = 'move';
                e.dataTransfer.setData('text/plain', b.id);
                setDragId(b.id);
              }}
              onDragEnd={() => {
                setDragId(null);
                setDropIndex(null);
              }}
              onDragOver={(e) => onDragOver(e, i)}
              onDrop={onDrop}
              onClick={() => {
                setCreating(false);
                select(b.id);
              }}
              onKeyDown={(e) => onRowKey(e, b, i)}
              className={cn(
                'relative flex cursor-pointer items-center gap-3 border-b border-border px-3.5 py-3 outline-none last:border-b-0 focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-focus-ring',
                selected?.id === b.id && 'bg-accent-tint',
                dragId === b.id && 'opacity-45',
              )}
            >
              {dropIndex === i && <DropLine edge="top" />}
              {dropIndex === i + 1 && i === list.length - 1 && <DropLine edge="bottom" />}
              <DotsSixVertical className="cursor-grab text-text-muted" />
              <Swatch className="h-[22px] w-9" />
              <div className="flex min-w-0 flex-1 flex-col gap-[3px]">
                <span className="truncate font-medium">{b.label}</span>
                <span className="truncate font-mono text-[11px] text-text-muted">{when(b)}</span>
              </div>
              {b.permits_fixed ? (
                <span className="flex items-center gap-[5px] text-sm text-text-muted">
                  <PushPin />
                  Fixed allowed
                </span>
              ) : (
                <span className="flex items-center gap-[5px] text-sm text-warning">
                  <Warning />
                  Fixed warns
                </span>
              )}
              <CaretRight className="text-text-muted" />
            </div>
          ))}
          {creating && (
            <div
              aria-selected
              role="option"
              className="flex items-center gap-3 bg-accent-tint px-3.5 py-3"
            >
              <DotsSixVertical className="text-text-muted opacity-40" />
              <Swatch className="h-[22px] w-9" />
              <span className="flex-1 font-medium text-text-muted">New block-out</span>
            </div>
          )}
        </div>
      )}

      {(creating || selected) && (
        <div className="grid grid-cols-2 gap-5 rounded-lg border border-border bg-surface p-[18px]">
          <BlockoutEditor
            key={selected?.id ?? 'new'}
            blockout={selected}
            onSaved={(id) => {
              setCreating(false);
              select(id);
            }}
            onCancelNew={() => setCreating(false)}
            onDeleted={() => select(null)}
          />
          {selected ? (
            <Exceptions key={selected.id} blockout={selected} />
          ) : (
            <p className="m-0 self-center text-sm text-text-muted">
              Save the block-out to add exceptions.
            </p>
          )}
        </div>
      )}
    </div>
  );
}

function DropLine({ edge }: { edge: 'top' | 'bottom' }) {
  return (
    <span
      aria-hidden
      className={cn(
        'absolute inset-x-2 z-1 h-0.5 rounded-full bg-accent',
        edge === 'top' ? '-top-px' : '-bottom-px',
      )}
    />
  );
}

function BlockoutEditor({
  blockout,
  onSaved,
  onCancelNew,
  onDeleted,
}: {
  blockout: Blockout | null;
  onSaved: (id: string) => void;
  onCancelNew: () => void;
  onDeleted: () => void;
}) {
  const [draft, setDraft] = useState<BlockoutDraft>(() =>
    blockout ? draftFromBlockout(blockout) : emptyBlockoutDraft(),
  );
  const [error, setError] = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [busy, setBusy] = useState(false);
  const set = (patch: Partial<BlockoutDraft>) => {
    setDraft((d) => ({ ...d, ...patch }));
    setError(null);
  };

  const invalid = validateBlockout(draft);
  const change = invalid ? null : blockoutSave(blockout, draft);
  const dirty = !blockout || change !== null;

  const save = async () => {
    if (invalid) return setError(invalid);
    if (!change) return;
    setBusy(true);
    try {
      const saved = await runOp<Blockout>(change.name, change.args);
      onSaved(saved.id);
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  };

  const remove = async () => {
    if (!blockout) return;
    if (!confirmDelete) return setConfirmDelete(true);
    try {
      await runOp('delete_blockout', { id: blockout.id });
      onDeleted();
    } catch (e) {
      setError(errorText(e));
    }
  };

  const normalise = (key: 'start' | 'end') => {
    const t = parseTime(draft[key]);
    if (t) set({ [key]: t });
  };

  return (
    <form
      className="flex flex-col gap-3.5"
      onSubmit={(e) => {
        e.preventDefault();
        void save();
      }}
      onKeyDown={(e) => {
        if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
          e.preventDefault();
          void save();
        }
      }}
    >
      <span className="font-semibold">
        {blockout ? `Edit · ${blockout.label}` : 'New block-out'}
      </span>
      <label className="flex flex-col gap-1.5 text-sm text-text-muted">
        Label
        <input
          autoFocus={!blockout}
          value={draft.label}
          onChange={(e) => set({ label: e.target.value })}
          placeholder="e.g. Work"
          className={FIELD}
        />
      </label>
      <div className="flex flex-col gap-1.5 text-sm text-text-muted">
        Days
        <div role="group" aria-label="Days" className="flex gap-1">
          {WEEKDAYS.map((d) => {
            const on = draft.weekdays.includes(d);
            return (
              <button
                key={d}
                type="button"
                aria-pressed={on}
                aria-label={d}
                onClick={() =>
                  set({
                    weekdays: on ? draft.weekdays.filter((x) => x !== d) : [...draft.weekdays, d],
                  })
                }
                className={cn(
                  'size-[30px] cursor-pointer rounded-md border text-sm font-medium',
                  on
                    ? 'border-accent bg-accent-tint text-accent-text'
                    : 'border-border-strong text-text-muted hover:bg-hover',
                )}
              >
                {d[0]?.toUpperCase()}
              </button>
            );
          })}
        </div>
      </div>
      <div className="flex gap-2.5">
        {(['start', 'end'] as const).map((key) => (
          <label key={key} className="flex min-w-0 flex-1 flex-col gap-1.5 text-sm text-text-muted">
            {key === 'start' ? 'Start' : 'End'}
            <input
              value={draft[key]}
              onChange={(e) => set({ [key]: e.target.value })}
              onBlur={() => normalise(key)}
              placeholder="HH:MM"
              className={cn(FIELD, 'w-full min-w-0 font-mono')}
            />
          </label>
        ))}
      </div>
      {parseTime(draft.end) &&
        parseTime(draft.start) &&
        (parseTime(draft.end) as string) < (parseTime(draft.start) as string) && (
          <span className="-mt-2 text-sm text-text-muted">
            Ends the next morning (runs past midnight).
          </span>
        )}
      <div className="flex items-center gap-2.5">
        <span className="flex flex-1 flex-col gap-0.5">
          <span id="permits-fixed">Allow fixed items</span>
          <span className="text-sm text-text-muted">
            Turn off for windows like Sleep: fixed items then show a warning.
          </span>
        </span>
        <Switch
          aria-labelledby="permits-fixed"
          checked={draft.permitsFixed}
          onCheckedChange={(v) => set({ permitsFixed: v })}
        />
      </div>
      {error && (
        <p role="alert" className="m-0 text-sm text-danger">
          {error}
        </p>
      )}
      <div className="flex items-center gap-2 pt-1">
        {blockout ? (
          <Button
            variant="bare"
            size="sm"
            onClick={() => void remove()}
            onBlur={() => setConfirmDelete(false)}
            className="text-danger hover:text-danger"
          >
            <Trash />
            {confirmDelete ? 'Click again to delete' : 'Delete'}
          </Button>
        ) : (
          <Button variant="bare" size="sm" onClick={onCancelNew}>
            Cancel
          </Button>
        )}
        <span className="flex-1" />
        {blockout && dirty && (
          <Button variant="ghost" size="sm" onClick={() => setDraft(draftFromBlockout(blockout))}>
            Revert
          </Button>
        )}
        <Button variant="primary" size="sm" type="submit" disabled={!dirty || busy}>
          {blockout ? 'Save' : 'Create'} <Kbd onAccent>Ctrl ↵</Kbd>
        </Button>
      </div>
    </form>
  );
}

function Exceptions({ blockout }: { blockout: Blockout }) {
  const [from, setFrom] = useState('');
  const [to, setTo] = useState('');
  const [note, setNote] = useState('');
  const [error, setError] = useState<string | null>(null);
  const today = startOfToday();
  const exceptions = [...blockout.exceptions].sort((a, b) => a.date.localeCompare(b.date));

  const add = async () => {
    setError(null);
    try {
      const calls = liftCalls(blockout, from, to, note);
      if (calls.length === 0) {
        throw new Error(`${blockout.label} doesn't apply on ${to ? 'those dates' : 'that date'}.`);
      }
      // One batch, so a whole stretch of leave undoes in one step.
      await runBatch(calls);
      setFrom('');
      setTo('');
      setNote('');
    } catch (e) {
      setError(errorText(e));
    }
  };

  return (
    <div className="flex min-w-0 flex-col gap-2.5">
      <span className="font-semibold">Exceptions</span>
      <span className="text-sm leading-[1.45] text-text-muted">
        Dates when {blockout.label} is lifted. Also from any focused day with{' '}
        <span className="whitespace-nowrap">
          <Kbd>b</Kbd> <Kbd>l</Kbd>
        </span>
        .
      </span>
      <div className="flex max-h-[220px] flex-col gap-1.5 overflow-auto">
        {exceptions.length === 0 && <span className="text-sm text-text-muted">None yet.</span>}
        {exceptions.map((x) => {
          const past = isBefore(parseISO(x.date), today);
          return (
            <div
              key={x.id}
              className={cn(
                'flex h-8 flex-none items-center gap-2.5 rounded-md border border-border bg-bg px-2.5',
                past && 'opacity-60',
              )}
            >
              <span className="font-mono text-sm">
                {format(parseISO(x.date), 'EEE d MMM yyyy')}
              </span>
              <span className="min-w-0 flex-1 truncate text-sm text-text-muted">{x.note}</span>
              <Button
                variant="bare"
                size="icon-sm"
                aria-label={`Remove ${x.date}`}
                onClick={() =>
                  runOp('remove_blockout_exception', { exception_id: x.id }).catch((e: unknown) =>
                    setError(errorText(e)),
                  )
                }
              >
                <X />
              </Button>
            </div>
          );
        })}
      </div>
      <form
        className="flex flex-col gap-2"
        onSubmit={(e) => {
          e.preventDefault();
          void add();
        }}
      >
        <div className="flex items-center gap-1.5">
          <input
            type="date"
            aria-label="Lift from"
            value={from}
            onChange={(e) => setFrom(e.target.value)}
            className={cn(FIELD, 'min-w-0 flex-1 font-mono text-sm')}
          />
          <span className="text-sm text-text-muted">to</span>
          <input
            type="date"
            aria-label="Lift until (optional)"
            value={to}
            min={from || undefined}
            onChange={(e) => setTo(e.target.value)}
            className={cn(FIELD, 'min-w-0 flex-1 font-mono text-sm')}
          />
        </div>
        <div className="flex items-center gap-1.5">
          <input
            aria-label="Note"
            value={note}
            onChange={(e) => setNote(e.target.value)}
            placeholder="Note, e.g. Leave"
            className={cn(FIELD, 'min-w-0 flex-1 text-sm')}
          />
          <Button type="submit" size="sm" disabled={!from} className="border-dashed bg-transparent">
            <CalendarPlus />
            Add exception
          </Button>
        </div>
      </form>
      {error && (
        <p role="alert" className="m-0 text-sm text-danger">
          {error}
        </p>
      )}
    </div>
  );
}
