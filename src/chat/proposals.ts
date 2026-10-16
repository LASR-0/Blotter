import { format, parseISO } from 'date-fns';
import { summarizeArgs } from './transcript';
import { clockTime } from '@/lib/format';

/** Mirrors `proposals::ProposalOp`. */
export interface ProposalOp {
  id: string;
  idx: number;
  op: string;
  args: Record<string, unknown>;
  preview: { changes?: PreviewChange[] } | null;
  status: 'pending' | 'accepted' | 'rejected' | 'failed';
  error: string | null;
}

export interface PreviewChange {
  entity: string;
  entity_id: string;
  before: Record<string, unknown> | null;
  after: Record<string, unknown> | null;
}

/** Mirrors `proposals::Proposal`. */
export interface Proposal {
  id: string;
  actor: string;
  thread_id: string | null;
  status: 'pending' | 'accepted' | 'rejected' | 'partial';
  sealed: boolean;
  created_at: string;
  resolved_at: string | null;
  ops: ProposalOp[];
}

/** One proposed change as the card shows it: `~ Review insurance · Wed 19:00 → Tue 19:00`. */
export interface ChangeLine {
  symbol: '+' | '~' | '−' | '✓';
  title: string;
  /** Where it was (struck through), if anywhere. */
  from: string;
  /** Where it goes, or what happens to it. */
  to: string;
}

const str = (v: unknown) => (typeof v === 'string' ? v : '');

/** When an item happens, e.g. `Tue 13 Oct 18:30–18:50`, or `Inbox`. */
export function itemWhen(item: Record<string, unknown> | null): string {
  if (!item) return '';
  const start = str(item.start_at);
  const end = str(item.end_at);
  if (item.all_day && str(item.start_date)) {
    return `${format(parseISO(str(item.start_date)), 'EEE d MMM')} · all day`;
  }
  if (!start) return 'Inbox';
  const s = parseISO(start);
  return end
    ? `${format(s, 'EEE d MMM')} ${clockTime(s)}–${clockTime(parseISO(end))}`
    : `${format(s, 'EEE d MMM')} ${clockTime(s)}`;
}

function symbolFor(op: string): ChangeLine['symbol'] {
  if (op.startsWith('create_') || op === 'add_blockout_exception' || op === 'set_reminder')
    return '+';
  if (op.startsWith('delete_') || op === 'clear_reminder' || op === 'remove_blockout_exception') {
    return '−';
  }
  if (op === 'complete_item') return '✓';
  return '~';
}

/** Describes one proposed operation from what its rehearsal changed. */
export function describe(op: ProposalOp): ChangeLine {
  const symbol = symbolFor(op.op);
  const changes = op.preview?.changes ?? [];
  const main = changes.find((c) => c.entity === 'item') ?? changes[0];
  const before = main?.before ?? null;
  const after = main?.after ?? null;
  const title =
    str(after?.title) ||
    str(before?.title) ||
    str(after?.label) ||
    str(before?.label) ||
    str(op.args.title) ||
    op.op.replace(/_/g, ' ');

  if (main?.entity === 'item') {
    if (op.op === 'complete_item') return { symbol, title, from: '', to: 'mark done' };
    if (op.op === 'uncomplete_item') return { symbol, title, from: 'done', to: 'not done' };
    if (!before) {
      const tags =
        Array.isArray(after?.tags) && after.tags.length ? ` · #${after.tags.join(' #')}` : '';
      return { symbol, title, from: '', to: `new · ${itemWhen(after)}${tags}` };
    }
    if (!after) return { symbol, title, from: itemWhen(before), to: 'deleted' };
    const was = itemWhen(before);
    const now = itemWhen(after);
    if (was !== now) return { symbol, title, from: was, to: now };
    if (str(before.title) !== str(after.title))
      return { symbol, title, from: str(before.title), to: str(after.title) };
    return { symbol, title, from: '', to: 'details changed' };
  }
  if (main?.entity === 'blockout' && after) {
    return { symbol, title, from: '', to: `${str(after.start_time)}–${str(after.end_time)}` };
  }
  return { symbol, title, from: '', to: changes.length ? summarizeArgs(op.args) : 'no change' };
}

/** "From Claude Code · sonnet", "From an MCP client". */
export function actorLabel(actor: string): string {
  if (actor === 'mcp') return 'an MCP client';
  const m = /^ai:([^/]+)\/(.+)$/.exec(actor);
  if (!m) return actor;
  const provider = m[1] === 'claude-code' ? 'Claude Code' : m[1]!;
  return `${provider} · ${m[2]}`;
}
