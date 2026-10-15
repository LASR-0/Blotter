import { describe, expect, it } from 'vitest';
import {
  applyEvent,
  summarizeArgs,
  transcript,
  type LiveTurn,
  type SavedMessage,
} from './transcript';

const msg = (m: Partial<SavedMessage>): SavedMessage => ({
  id: 'm',
  thread_id: 't',
  role: 'assistant',
  content: '',
  blocks: null,
  batch_id: null,
  created_at: '2026-10-15T00:00:00Z',
  ...m,
});

describe('summarizeArgs', () => {
  it('reads like a call', () => {
    expect(summarizeArgs({ title: 'Water the plants', duration_min: 30 })).toBe(
      '"Water the plants", 30m',
    );
    expect(summarizeArgs({ id: '01M4ZJMVTFDKZ43KBJA2B14CWZ' })).toBe('id: 01M4ZJ…');
    expect(summarizeArgs({ ids: ['a', 'b'] })).toBe('ids: [2]');
    expect(summarizeArgs({ duration_min: 15, title: 'Stretch' })).toBe('"Stretch", 15m');
    expect(summarizeArgs({})).toBe('');
    expect(summarizeArgs(null)).toBe('');
  });
});

describe('transcript', () => {
  it('pairs tool calls with results across messages', () => {
    const items = transcript([
      msg({ role: 'user', content: 'Add milk' }),
      msg({
        blocks: [{ type: 'tool_use', id: 't1', name: 'create_item', input: { title: 'Milk' } }],
      }),
      msg({
        role: 'tool',
        blocks: [{ type: 'tool_result', tool_use_id: 't1', content: '{}', is_error: false }],
      }),
      msg({ content: 'Added.', blocks: [{ type: 'text', text: 'Added.' }] }),
    ]);
    expect(items).toEqual([
      { kind: 'user', text: 'Add milk' },
      { kind: 'tool', id: 't1', name: 'create_item', args: '"Milk"', status: 'ok' },
      { kind: 'text', text: 'Added.' },
    ]);
  });

  it('shows failed calls with their error, saved errors, and unanswered calls as failed', () => {
    const items = transcript([
      msg({
        blocks: [
          { type: 'tool_use', id: 'a', name: 'auto_slot', input: {} },
          { type: 'tool_result', tool_use_id: 'a', content: 'no slot', is_error: true },
          { type: 'tool_use', id: 'b', name: 'list_inbox', input: {} },
        ],
      }),
      msg({ blocks: [{ type: 'error', message: 'Claude Code stopped' }] }),
      msg({ content: 'Plain reply' }),
    ]);
    expect(items[0]).toMatchObject({ kind: 'tool', status: 'error', detail: 'no slot' });
    expect(items[1]).toMatchObject({ kind: 'tool', id: 'b', status: 'error' });
    expect(items[2]).toEqual({ kind: 'error', message: 'Claude Code stopped' });
    expect(items[3]).toEqual({ kind: 'text', text: 'Plain reply' });
  });
});

describe('transcript with proposals', () => {
  it('puts each proposal after its turn', () => {
    const items = transcript(
      [
        msg({ role: 'user', content: 'Plan tonight', created_at: '2026-10-15T08:00:00Z' }),
        msg({ content: 'Here you go.', created_at: '2026-10-15T08:00:20Z' }),
        msg({ role: 'user', content: 'Thanks', created_at: '2026-10-15T08:05:00Z' }),
        msg({ content: 'Any time.', created_at: '2026-10-15T08:05:05Z' }),
      ],
      [{ id: 'p1', created_at: '2026-10-15T08:00:10Z' }],
    );
    expect(items.map((i) => i.kind)).toEqual(['user', 'text', 'proposal', 'user', 'text']);
  });
});

describe('applyEvent', () => {
  it('streams text, tools and the end of a turn', () => {
    let turn: LiveTurn = { items: [], running: true };
    const t = 'x';
    turn = applyEvent(turn, { thread_id: t, type: 'text', text: 'Let me ' });
    turn = applyEvent(turn, { thread_id: t, type: 'text', text: 'check.' });
    turn = applyEvent(turn, {
      thread_id: t,
      type: 'tool_start',
      id: '1',
      name: 'list_inbox',
      input: {},
    });
    expect(turn.items[1]).toMatchObject({ status: 'running' });
    turn = applyEvent(turn, {
      thread_id: t,
      type: 'tool_result',
      id: '1',
      name: 'list_inbox',
      is_error: false,
      preview: '[]',
    });
    turn = applyEvent(turn, { thread_id: t, type: 'text', text: 'Empty.' });
    turn = applyEvent(turn, { thread_id: t, type: 'done', notice: 'Stopped.' });
    expect(turn.running).toBe(false);
    expect(turn.items.map((i) => i.kind)).toEqual(['text', 'tool', 'text', 'notice']);
    expect(turn.items[0]).toEqual({ kind: 'text', text: 'Let me check.' });
    expect(turn.items[1]).toMatchObject({ status: 'ok' });
  });

  it('ends on an error', () => {
    const turn = applyEvent(
      { items: [], running: true },
      { thread_id: 'x', type: 'error', message: 'boom' },
    );
    expect(turn).toEqual({ items: [{ kind: 'error', message: 'boom' }], running: false });
  });
});
