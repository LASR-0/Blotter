import { useState } from 'react';
import { format } from 'date-fns';
import { Dialog } from 'radix-ui';
import { Button } from '@/components/ui/button';
import { Kbd } from '@/components/ui/kbd';
import { Switch } from '@/components/ui/switch';
import { isOpError, runOp } from '@/lib/api';
import { appliesOn, liftedOn } from '@/lib/blockoutDraft';
import { useOp } from '@/lib/queries';
import type { Blockout } from '@/lib/types';
import { useNav } from './nav';
import { useOverlay } from './overlay';

/**
 * `b l` on a focused day: the block-outs that apply that day, each with a
 * switch that lifts it (adds an exception) or restores it. Changes apply
 * at once; Esc closes.
 */
export function LiftDialog() {
  const date = useOverlay((s) => (s.overlay === 'lift' ? s.liftDate : null));
  const close = useOverlay((s) => s.close);
  const blockouts = useOp<Blockout[]>('list_blockouts', undefined, { enabled: date !== null });
  const [error, setError] = useState<string | null>(null);
  const relevant = date ? (blockouts.data ?? []).filter((b) => appliesOn(b, date)) : [];

  const toggle = async (b: Blockout, lift: boolean) => {
    if (!date) return;
    setError(null);
    const key = format(date, 'yyyy-MM-dd');
    try {
      if (lift) {
        await runOp('add_blockout_exception', { blockout_id: b.id, date: key });
      } else {
        const x = b.exceptions.find((e) => e.date === key);
        if (x) await runOp('remove_blockout_exception', { exception_id: x.id });
      }
    } catch (e) {
      setError(isOpError(e) ? e.message : String(e));
    }
  };

  return (
    <Dialog.Root
      open={date !== null}
      onOpenChange={(o) => {
        if (!o) {
          setError(null);
          close();
        }
      }}
    >
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-40 bg-[color-mix(in_srgb,var(--color-bg)_62%,transparent)]" />
        <Dialog.Content
          aria-describedby="lift-body"
          className="fixed top-[140px] left-1/2 z-41 flex w-[400px] max-w-[calc(100vw-32px)] -translate-x-1/2 flex-col gap-3 rounded-lg bg-surface-raised p-[18px] text-base text-text shadow-overlay focus:outline-none"
        >
          <Dialog.Title className="m-0 text-md font-semibold">
            Block-outs on {date ? format(date, 'EEE d MMM') : ''}
          </Dialog.Title>
          <p id="lift-body" className="m-0 leading-normal text-text-muted">
            Lift one to free that time for flexible tasks on this day only.
          </p>
          {relevant.length === 0 && blockouts.isSuccess && (
            <p className="m-0 text-sm text-text-muted">No block-outs apply on this day.</p>
          )}
          <div className="flex flex-col gap-1.5">
            {relevant.map((b, i) => {
              const lifted = date ? liftedOn(b, date) : false;
              return (
                <label
                  key={b.id}
                  className="flex h-10 cursor-pointer items-center gap-2.5 rounded-md border border-border bg-bg px-3"
                >
                  <span className="size-3 rounded-[3px] border border-[color-mix(in_srgb,var(--color-blockout)_50%,transparent)] bg-hatch" />
                  <span className="flex-1">
                    <span className={lifted ? 'text-text-muted line-through' : ''}>{b.label}</span>
                    <span className="ml-2 font-mono text-xs text-text-muted">
                      {b.start_time}–{b.end_time}
                    </span>
                  </span>
                  <span className="text-sm text-text-muted">{lifted ? 'Lifted' : 'Active'}</span>
                  <Switch
                    autoFocus={i === 0}
                    aria-label={`Lift ${b.label}`}
                    checked={lifted}
                    onCheckedChange={(v) => void toggle(b, v)}
                  />
                </label>
              );
            })}
          </div>
          {error && <p className="m-0 font-mono text-xs text-danger">{error}</p>}
          <div className="flex items-center gap-2 pt-1">
            <Button
              variant="bare"
              size="sm"
              onClick={() => {
                close();
                useNav.getState().openBlockout(relevant[0]?.id ?? null);
              }}
            >
              Edit block-outs…
            </Button>
            <span className="flex-1" />
            <Dialog.Close asChild>
              <Button>
                Done <Kbd>Esc</Kbd>
              </Button>
            </Dialog.Close>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
