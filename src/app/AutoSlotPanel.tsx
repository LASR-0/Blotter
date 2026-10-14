import { useEffect } from 'react';
import { format, parseISO } from 'date-fns';
import { MagicWand, WarningCircle } from '@phosphor-icons/react';
import { Button } from '@/components/ui/button';
import { Kbd } from '@/components/ui/kbd';
import { runOp } from '@/lib/api';
import { formatDuration } from '@/lib/duration';
import type { AutoSlotResult } from '@/lib/types';
import { runAutoSlot, useAutoSlot } from './autoSlot';
import { useOverlay } from './overlay';

type Placed = AutoSlotResult['placed'][number];
type Unplaced = AutoSlotResult['unplaced'][number];

const day = (iso: string) => format(parseISO(iso), 'EEE d MMM');
const time = (iso: string) => format(parseISO(iso), 'HH:mm');

/** "Inbox → Sat 17 Oct 11:30–13:00", or where it was moved from. */
function movedText(p: Placed): string {
  const from = p.previous_start
    ? `${day(p.previous_start)} ${time(p.previous_start)}`
    : p.previous_date
      ? day(p.previous_date)
      : 'Inbox';
  return `${from} → ${day(p.start)} ${time(p.start)}–${time(p.end)}`;
}

function summary(r: AutoSlotResult): string {
  const placed = `${r.placed.length} placed`;
  const unplaced = r.unplaced.length ? `, ${r.unplaced.length} couldn't be placed` : '';
  if (!r.placed.length && !r.unplaced.length) return 'Nothing to place: the Inbox is empty.';
  return `${placed}${unplaced}. Ordered by due date, then priority, then longest first. Block-outs were avoided.`;
}

function UnplacedCard({ u }: { u: Unplaced }) {
  const gap = u.longest_gap && u.longest_gap.minutes >= 5 ? u.longest_gap.minutes : null;
  return (
    <div className="flex flex-col gap-1.5 rounded-md border border-warning bg-[color-mix(in_srgb,var(--color-warning)_8%,var(--color-bg))] px-3 py-2.5">
      <div className="flex items-center gap-1.5">
        <WarningCircle weight="fill" className="flex-none text-warning" />
        <span className="truncate font-medium">{u.title}</span>
        <span className="ml-auto font-mono text-[11px] whitespace-nowrap text-text-muted">
          ~{formatDuration(u.duration_min)}
          {u.due_at && ` · due ${format(parseISO(u.due_at), 'EEE')}`}
        </span>
      </div>
      <span className="leading-normal">{u.reason}</span>
      <div className="flex flex-wrap gap-1.5 pt-0.5">
        {u.due_at && (
          <Button size="sm" onClick={() => useOverlay.getState().openItem({ id: u.id })}>
            Move due date
          </Button>
        )}
        {gap !== null && gap < u.duration_min && (
          <Button
            size="sm"
            onClick={() =>
              void runOp('update_item', { id: u.id, duration_min: gap }).then(() =>
                runAutoSlot([u.id], true),
              )
            }
          >
            Shorten to {formatDuration(gap)}
          </Button>
        )}
        {!u.due_at && gap === null && (
          <Button size="sm" onClick={() => useOverlay.getState().openItem({ id: u.id })}>
            Open
          </Button>
        )}
      </div>
    </div>
  );
}

/** The auto-slot result (design V3): what was placed, and why the rest wasn't. */
export function AutoSlotPanel() {
  const { result, at, error, close } = useAutoSlot();
  const open = result !== null || error !== null;

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && useOverlay.getState().overlay === null) close();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [open, close]);

  if (!open) return null;
  return (
    <aside
      aria-label="Auto-slot result"
      className="fixed top-[60px] right-3 bottom-3 z-30 flex w-[360px] flex-col gap-3.5 overflow-auto rounded-lg bg-surface-raised p-4 text-base text-text shadow-overlay"
    >
      <div className="flex items-center gap-2">
        <MagicWand className="text-accent-text" />
        <span className="font-semibold">Auto-slot result</span>
        <span className="flex-1" />
        {at && <span className="font-mono text-[11px] text-text-muted">{format(at, 'HH:mm')}</span>}
      </div>
      {error && (
        <p role="alert" className="m-0 text-danger">
          {error}
        </p>
      )}
      {result && <p className="m-0 leading-normal text-text-muted">{summary(result)}</p>}
      {result && result.placed.length > 0 && (
        <section className="flex flex-col gap-1.5">
          <span className="text-[11px] tracking-[.05em] text-text-muted uppercase">Placed</span>
          {result.placed.map((p) => (
            <button
              key={p.item.id}
              type="button"
              onClick={() => useOverlay.getState().openItem({ id: p.item.id })}
              className="flex cursor-pointer flex-col gap-1 rounded-md border border-border bg-bg px-3 py-2.5 text-left hover:bg-hover"
            >
              <span className="font-medium">{p.item.title}</span>
              <span className="font-mono text-[11px] text-text-muted">{movedText(p)}</span>
            </button>
          ))}
        </section>
      )}
      {result && result.unplaced.length > 0 && (
        <section className="flex flex-col gap-1.5">
          <span className="text-[11px] tracking-[.05em] text-warning uppercase">
            Couldn't place
          </span>
          {result.unplaced.map((u) => (
            <UnplacedCard key={u.id} u={u} />
          ))}
        </section>
      )}
      <span className="flex-1" />
      <Button variant="primary" autoFocus onClick={close}>
        Done <Kbd onAccent>Esc</Kbd>
      </Button>
    </aside>
  );
}
