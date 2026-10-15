/** Mirrors `store::chat::Thread`. */
export interface Thread {
  id: string;
  title: string;
  provider_id: string | null;
  external_session: string | null;
  created_at: string;
  updated_at: string;
}

/** Mirrors `store::chat::ChatMessage`. */
export interface SavedMessage {
  id: string;
  thread_id: string;
  role: 'user' | 'assistant' | 'tool';
  content: string;
  blocks: SavedBlock[] | null;
  batch_id: string | null;
  created_at: string;
}

export type SavedBlock =
  | { type: 'text'; text: string }
  | { type: 'tool_use'; id: string; name: string; input: unknown; invalid?: string }
  | { type: 'tool_result'; tool_use_id: string; content: string; is_error: boolean }
  | { type: 'error'; message: string }
  | { type: 'raw' };

/** Mirrors `ai::chat::ChatEvent` (with its thread id). */
export type ChatEvent = { thread_id: string } & (
  | { type: 'text'; text: string }
  | { type: 'tool_start'; id: string; name: string; input: unknown }
  | { type: 'tool_result'; id: string; name: string; is_error: boolean; preview: string }
  | { type: 'done'; notice: string | null }
  | { type: 'error'; message: string }
);

/** One row of the chat as shown. */
export type Item =
  | { kind: 'user'; text: string }
  | { kind: 'text'; text: string }
  | {
      kind: 'tool';
      id: string;
      name: string;
      args: string;
      status: 'running' | 'ok' | 'error';
      detail?: string;
    }
  | { kind: 'error'; message: string }
  | { kind: 'notice'; text: string }
  | { kind: 'proposal'; id: string }
  /** What an Auto-mode turn applied, with its Undo batch. */
  | { kind: 'changed'; batches: string[] };

const ARGS_MAX = 60;

/** A tool call's arguments in a few words: `"Water the plants", 30m`. */
export function summarizeArgs(input: unknown): string {
  if (typeof input !== 'object' || input === null) return '';
  // The title says most, so it goes first.
  const entries = Object.entries(input as Record<string, unknown>).sort(
    ([a], [b]) => Number(b === 'title') - Number(a === 'title'),
  );
  const parts = entries.map(([key, value]) => {
    if (key === 'title' && typeof value === 'string') return `"${value}"`;
    if (key === 'duration_min' && typeof value === 'number') return `${value}m`;
    if (typeof value === 'string')
      return `${key}: ${value.length > 14 ? `${value.slice(0, 6)}…` : value}`;
    if (Array.isArray(value)) return `${key}: [${value.length}]`;
    if (typeof value === 'object' && value !== null) return `${key}: {…}`;
    return `${key}: ${String(value)}`;
  });
  const text = parts.join(', ');
  return text.length > ARGS_MAX ? `${text.slice(0, ARGS_MAX - 1)}…` : text;
}

function resolveTool(items: Item[], id: string, isError: boolean, detail: string) {
  const tool = items.find((i) => i.kind === 'tool' && i.id === id);
  if (tool?.kind === 'tool') {
    tool.status = isError ? 'error' : 'ok';
    if (isError) tool.detail = detail;
  }
}

/**
 * A thread's saved messages as display rows. Each proposal goes after the
 * reply of the turn that made it: before the next message the user sent.
 */
export function transcript(
  messages: SavedMessage[],
  proposals: { id: string; created_at: string }[] = [],
): Item[] {
  const items: Item[] = [];
  const queue = [...proposals].sort((a, b) => Date.parse(a.created_at) - Date.parse(b.created_at));
  const flushBefore = (at: number) => {
    while (queue.length && Date.parse(queue[0]!.created_at) < at) {
      items.push({ kind: 'proposal', id: queue.shift()!.id });
    }
  };
  for (const m of messages) {
    if (m.role === 'user') {
      flushBefore(Date.parse(m.created_at));
      items.push({ kind: 'user', text: m.content });
      continue;
    }
    if (!m.blocks) {
      if (m.content) items.push({ kind: 'text', text: m.content });
    } else {
      for (const b of m.blocks) {
        if (b.type === 'text' && b.text.trim()) items.push({ kind: 'text', text: b.text });
        if (b.type === 'tool_use') {
          items.push({
            kind: 'tool',
            id: b.id,
            name: b.name,
            args: summarizeArgs(b.input),
            status: b.invalid ? 'error' : 'running',
            detail: b.invalid,
          });
        }
        if (b.type === 'tool_result') resolveTool(items, b.tool_use_id, b.is_error, b.content);
        if (b.type === 'error') items.push({ kind: 'error', message: b.message });
      }
    }
    // A turn that changed things (Auto mode) lists its batches on its reply.
    if (m.role === 'assistant' && m.batch_id) {
      items.push({ kind: 'changed', batches: m.batch_id.split(',').filter(Boolean) });
    }
  }
  flushBefore(Infinity);
  // A call without a result was cut short (e.g. the turn was stopped).
  for (const i of items) if (i.kind === 'tool' && i.status === 'running') i.status = 'error';
  return items;
}

/** The turn in progress, built from streamed events. */
export interface LiveTurn {
  items: Item[];
  running: boolean;
}

export function applyEvent(turn: LiveTurn, e: ChatEvent): LiveTurn {
  const items = turn.items.map((i) => ({ ...i }));
  switch (e.type) {
    case 'text': {
      const last = items[items.length - 1];
      if (last?.kind === 'text') last.text += e.text;
      else items.push({ kind: 'text', text: e.text });
      return { items, running: true };
    }
    case 'tool_start':
      items.push({
        kind: 'tool',
        id: e.id,
        name: e.name,
        args: summarizeArgs(e.input),
        status: 'running',
      });
      return { items, running: true };
    case 'tool_result':
      resolveTool(items, e.id, e.is_error, e.preview);
      return { items, running: true };
    case 'done':
      if (e.notice) items.push({ kind: 'notice', text: e.notice });
      return { items, running: false };
    case 'error':
      items.push({ kind: 'error', message: e.message });
      return { items, running: false };
  }
}
