import { useState, type KeyboardEvent } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useQueryClient } from '@tanstack/react-query';
import { Check, PencilSimple, X } from '@phosphor-icons/react';
import { Button } from '@/components/ui/button';
import { Kbd } from '@/components/ui/kbd';
import { isOpError } from '@/lib/api';
import { cn } from '@/lib/cn';
import { PENDING_KEY, PROPOSALS_KEY } from './chatStore';
import { actorLabel, describe, type Proposal, type ProposalOp } from './proposals';

const STATUS_TEXT: Record<ProposalOp['status'], string> = {
  pending: '',
  accepted: 'Accepted · now in ink',
  rejected: 'Rejected',
  failed: 'Couldn’t apply',
};

/** A batch of proposed changes (design: Chat Panel, "Proposal"), drawn in pencil. */
export function ProposalCard({ proposal, showActor }: { proposal: Proposal; showActor?: boolean }) {
  const qc = useQueryClient();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const pending = proposal.ops.filter((o) => o.status === 'pending');

  const decide = async (accept: string[], reject: string[]) => {
    if (busy || (!accept.length && !reject.length)) return;
    setBusy(true);
    setError(null);
    try {
      await invoke('decide_proposal', { proposalId: proposal.id, accept, reject });
      await qc.invalidateQueries({ queryKey: PROPOSALS_KEY });
      void qc.invalidateQueries({ queryKey: PENDING_KEY });
    } catch (e) {
      setError(isOpError(e) ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };
  const ids = pending.map((o) => o.id);
  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter' && e.ctrlKey && ids.length) {
      e.preventDefault();
      void decide(ids, []);
    }
  };

  return (
    <div
      className="flex flex-col rounded-lg border border-dashed border-pencil bg-bg"
      onKeyDown={onKeyDown}
      role="group"
      aria-label="Proposed changes"
    >
      <div className="flex items-center gap-2 border-b border-dashed border-border-strong px-3 py-2.5">
        <PencilSimple className="text-pencil" />
        <span className="font-semibold">Proposal</span>
        <span className="truncate font-mono text-[11px] text-text-muted">
          {showActor && `from ${actorLabel(proposal.actor)} · `}
          {pending.length ? `${pending.length} of ${proposal.ops.length} pending` : 'all decided'}
        </span>
      </div>
      {proposal.ops.map((op) => {
        const line = describe(op);
        const rejected = op.status === 'rejected' || op.status === 'failed';
        return (
          <div
            key={op.id}
            className={cn(
              'flex items-start gap-2.5 border-b border-dashed border-border px-3 py-2.5',
              op.status === 'accepted' &&
                'bg-[color-mix(in_srgb,var(--color-accent)_6%,transparent)]',
            )}
          >
            <span
              className={cn(
                'w-3.5 font-mono text-xs font-medium',
                rejected ? 'text-text-muted' : 'text-accent-text',
              )}
            >
              {line.symbol}
            </span>
            <div className="flex min-w-0 flex-1 flex-col gap-[3px]">
              <span className={cn('font-medium', rejected && 'text-text-muted line-through')}>
                {line.title}
              </span>
              <span className="font-mono text-[10.5px] leading-normal text-text-muted">
                {line.from && <span className="line-through">{line.from}</span>}
                {line.from && ' → '}
                <span className={rejected ? undefined : 'text-text'}>{line.to}</span>
              </span>
              {op.status !== 'pending' && (
                <span
                  className={cn(
                    'flex items-center gap-1 text-[11px]',
                    op.status === 'accepted' ? 'text-accent-text' : 'text-text-muted',
                    op.status === 'failed' && 'text-danger',
                  )}
                >
                  {op.status === 'accepted' ? <Check weight="bold" /> : <X weight="bold" />}
                  {op.error ?? STATUS_TEXT[op.status]}
                </span>
              )}
            </div>
            {op.status === 'pending' && (
              <div className="flex gap-0.5">
                <Button
                  size="icon"
                  className="size-[26px]"
                  aria-label={`Accept: ${line.title}`}
                  disabled={busy}
                  onClick={() => void decide([op.id], [])}
                >
                  <Check weight="bold" />
                </Button>
                <Button
                  size="icon"
                  variant="bare"
                  className="size-[26px] hover:text-danger"
                  aria-label={`Reject: ${line.title}`}
                  disabled={busy}
                  onClick={() => void decide([], [op.id])}
                >
                  <X weight="bold" />
                </Button>
              </div>
            )}
          </div>
        );
      })}
      {error && <p className="m-0 px-3 py-2 text-sm text-danger">{error}</p>}
      {pending.length > 0 && (
        <div className="flex items-center gap-1.5 px-2.5 py-2">
          <Button size="sm" disabled={busy} onClick={() => void decide([], ids)}>
            Reject all
          </Button>
          <span className="flex-1" />
          <Button size="sm" variant="primary" disabled={busy} onClick={() => void decide(ids, [])}>
            Accept all
            <Kbd onAccent>Ctrl ↵</Kbd>
          </Button>
        </div>
      )}
    </div>
  );
}
