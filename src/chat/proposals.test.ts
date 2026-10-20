import { describe, expect, it } from 'vitest';
import { actorLabel, describe as describeOp, itemWhen, type ProposalOp } from './proposals';

const op = (o: Partial<ProposalOp>): ProposalOp => ({
  id: 'o',
  idx: 0,
  op: 'create_item',
  args: {},
  preview: null,
  status: 'pending',
  error: null,
  ...o,
});

// Local times, so the expectations don't depend on the test machine's zone.
const at = (y: number, mo: number, d: number, h: number, mi: number) =>
  new Date(y, mo - 1, d, h, mi).toISOString();

describe('itemWhen', () => {
  it('reads the day and times, or the Inbox', () => {
    expect(itemWhen({ start_at: at(2026, 10, 13, 18, 30), end_at: at(2026, 10, 13, 18, 50) })).toBe(
      'Tue 13 Oct 18:30–18:50',
    );
    expect(itemWhen({ start_at: null })).toBe('Inbox');
    expect(itemWhen({ all_day: true, start_date: '2026-10-17' })).toBe('Sat 17 Oct · all day');
  });
});

describe('describe', () => {
  it('shows a new item with its time and tags', () => {
    const line = describeOp(
      op({
        preview: {
          changes: [
            {
              entity: 'item',
              entity_id: 'a',
              before: null,
              after: {
                title: 'Call Mum',
                start_at: at(2026, 10, 13, 18, 30),
                end_at: at(2026, 10, 13, 18, 50),
                tags: ['personal'],
              },
            },
          ],
        },
      }),
    );
    expect(line).toEqual({
      symbol: '+',
      title: 'Call Mum',
      from: '',
      to: 'new · Tue 13 Oct 18:30–18:50 · #personal',
    });
  });

  it('shows a move from where it was to where it goes', () => {
    const line = describeOp(
      op({
        op: 'schedule_item',
        preview: {
          changes: [
            {
              entity: 'item',
              entity_id: 'a',
              before: { title: 'Fix the tap', start_at: null },
              after: {
                title: 'Fix the tap',
                start_at: at(2026, 10, 17, 10, 30),
                end_at: at(2026, 10, 17, 11, 15),
              },
            },
          ],
        },
      }),
    );
    expect(line).toEqual({
      symbol: '~',
      title: 'Fix the tap',
      from: 'Inbox',
      to: 'Sat 17 Oct 10:30–11:15',
    });
  });

  it('handles completing, deleting and renaming', () => {
    const item = { title: 'Gym', start_at: null };
    expect(
      describeOp(
        op({
          op: 'complete_item',
          preview: { changes: [{ entity: 'item', entity_id: 'a', before: item, after: item }] },
        }),
      ),
    ).toMatchObject({ symbol: '✓', to: 'mark done' });
    expect(
      describeOp(
        op({
          op: 'delete_item',
          preview: { changes: [{ entity: 'item', entity_id: 'a', before: item, after: null }] },
        }),
      ),
    ).toMatchObject({ symbol: '−', from: 'Inbox', to: 'deleted' });
    expect(
      describeOp(
        op({
          op: 'update_item',
          preview: {
            changes: [
              { entity: 'item', entity_id: 'a', before: item, after: { ...item, title: 'Swim' } },
            ],
          },
        }),
      ),
    ).toMatchObject({ title: 'Swim', from: 'Gym', to: 'Swim' });
  });
});

describe('actorLabel', () => {
  it('names who proposed it', () => {
    expect(actorLabel('ai:claude-code/sonnet')).toBe('Claude Code · sonnet');
    expect(actorLabel('ai:codex/default')).toBe('Codex · default');
    expect(actorLabel('ai:anthropic/claude-sonnet-5-5')).toBe('anthropic · claude-sonnet-5-5');
    expect(actorLabel('mcp')).toBe('an MCP client');
  });
});
