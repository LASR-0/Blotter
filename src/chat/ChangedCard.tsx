import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { ArrowCounterClockwise, Database, Lightning } from '@phosphor-icons/react';
import { Button } from '@/components/ui/button';
import { isOpError, runOp, type Change } from '@/lib/api';
import { cn } from '@/lib/cn';
import { BATCHES_KEY } from './chatStore';
import { describe, type ProposalOp } from './proposals';

/** The entities a person cares about seeing in the list. */
const SHOWN = new Set(['item', 'blockout']);

/** A change-log row read as a card line (same wording as proposals). */
function asOp(c: Change, idx: number): ProposalOp {
  return {
    id: c.id,
    idx,
    op: c.op,
    args: {},
    preview: {
      changes: [
        {
          entity: c.entity,
          entity_id: c.entity_id,
          before: c.before as Record<string, unknown> | null,
          after: c.after as Record<string, unknown> | null,
        },
      ],
    },
    status: 'accepted',
    error: null,
  };
}

/**
 * What an Auto-mode turn changed (design: Chat Panel, "Changed · N items"),
 * with Undo batch. A snapshot was taken before the turn's first change.
 */
export function ChangedCard({ batches }: { batches: string[] }) {
  const qc = useQueryClient();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const changes = useQuery({
    queryKey: [...BATCHES_KEY, batches],
    queryFn: () => invoke<Change[]>('chat_batch_changes', { batchIds: batches }),
  });
  const all = changes.data ?? [];
  const shown = all.filter((c) => SHOWN.has(c.entity));
  const rows = shown.length ? shown : all;
  const undone = all.length > 0 && all.every((c) => c.undone_at);
  const items = new Set(rows.map((c) => c.entity_id)).size;

  const undo = async () => {
    setBusy(true);
    setError(null);
    try {
      // Newest first, as Ctrl+Z would.
      for (const batch of [...batches].reverse()) {
        if (all.some((c) => c.batch_id === batch && !c.undone_at)) {
          await runOp('undo', { batch_id: batch });
        }
      }
    } catch (e) {
      setError(isOpError(e) ? e.message : String(e));
    } finally {
      setBusy(false);
      void qc.invalidateQueries({ queryKey: BATCHES_KEY });
    }
  };

  if (!all.length) return null;
  return (
    <div className="flex flex-col rounded-lg border border-border-strong bg-bg">
      <div className="flex items-center gap-2 border-b border-border px-3 py-2.5">
        {undone ? (
          <ArrowCounterClockwise weight="fill" className="text-text-muted" />
        ) : (
          <Lightning weight="fill" className="text-warning" />
        )}
        <span className="font-semibold">
          {undone ? 'Batch undone' : `Changed · ${items} ${items === 1 ? 'item' : 'items'}`}
        </span>
      </div>
      <div className={cn('flex flex-col gap-1.5 px-3 py-2', undone && 'opacity-50')}>
        {rows.map((c, i) => {
          const line = describe(asOp(c, i));
          return (
            <div key={c.id} className="flex gap-2 text-sm">
              <span className="font-mono text-accent-text">{line.symbol}</span>
              <span className="flex min-w-0 flex-1 flex-col">
                <span className="font-medium">{line.title}</span>
                <span className="font-mono text-[10.5px] text-text-muted">
                  {line.from && `${line.from} → `}
                  {line.to}
                </span>
              </span>
            </div>
          );
        })}
      </div>
      {error && <p className="m-0 px-3 pb-2 text-sm text-danger">{error}</p>}
      <div className="flex items-center gap-2 border-t border-border py-2 pr-2.5 pl-3 text-[11px] text-text-muted">
        <Database />
        <span className="flex-1">
          {undone ? 'Ctrl+Shift+Z redoes it' : 'Snapshot saved before applying'}
        </span>
        {!undone && (
          <Button size="sm" disabled={busy} onClick={() => void undo()}>
            <ArrowCounterClockwise />
            Undo batch
          </Button>
        )}
      </div>
    </div>
  );
}
