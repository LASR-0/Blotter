import type { CSSProperties } from 'react';
import { Bell, MagnifyingGlass, Sparkle, Warning } from '@phosphor-icons/react';
import { ProposalCard } from '@/chat/ProposalCard';
import type { Proposal } from '@/chat/proposals';
import { ItemCard } from '@/components/ItemCard';
import { Button } from '@/components/ui/button';
import { KeyChips } from '@/components/ui/kbd';
import { Select } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import type { Item } from '@/lib/types';
import type { ThemeVars } from '@/theme/presets';

// Times for the samples: today, so cards read naturally.
const at = (h: number, m = 0) => {
  const d = new Date();
  d.setHours(h, m, 0, 0);
  return d.toISOString();
};

const sample = (o: Partial<Item>): Item => ({
  id: `preview-${o.title ?? 'item'}`,
  kind: 'task',
  anchoring: 'flexible',
  status: 'open',
  title: 'Item',
  notes: '',
  all_day: false,
  start_at: null,
  end_at: null,
  start_date: null,
  end_date: null,
  tz: null,
  duration_min: null,
  due_at: null,
  priority: 0,
  tags: [],
  color: null,
  sort_order: 0,
  rrule: null,
  parent_id: null,
  recurrence_id: null,
  completed_at: null,
  source: 'user',
  created_at: at(8),
  updated_at: at(8),
  ...o,
});

const CARDS: Item[] = [
  sample({
    title: 'Gym',
    start_at: at(17, 30),
    end_at: at(18, 15),
    duration_min: 45,
    tags: ['health'],
  }),
  sample({
    title: 'Dentist',
    kind: 'event',
    anchoring: 'fixed',
    start_at: at(15),
    end_at: at(15, 45),
    tags: ['personal'],
  }),
  sample({
    title: 'Send invoice',
    status: 'done',
    start_at: at(9),
    end_at: at(9, 20),
    completed_at: at(9, 10),
    tags: ['work'],
  }),
];

const PROPOSAL: Proposal = {
  id: 'preview-proposal',
  actor: 'ai:claude-code/opus',
  thread_id: null,
  status: 'pending',
  sealed: true,
  created_at: at(8),
  resolved_at: null,
  ops: [
    {
      id: 'p1',
      idx: 0,
      op: 'create_item',
      args: { title: 'Call Mum' },
      preview: {
        changes: [
          {
            entity: 'item',
            entity_id: 'x',
            before: null,
            after: { title: 'Call Mum', start_at: at(18, 30), end_at: at(18, 45), tags: [] },
          },
        ],
      },
      status: 'pending',
      error: null,
    },
  ],
};

function Panel({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="flex flex-col gap-2.5 rounded-lg border border-border bg-bg p-3.5">
      <h3 className="m-0 text-xs font-medium tracking-[.06em] text-text-muted uppercase">
        {title}
      </h3>
      {children}
    </section>
  );
}

/** A slice of the Week grid: an hour scale, a block-out and a conflict. */
function MiniWeek() {
  const hour = 30;
  const block = (top: number, h: number, label: string) => (
    <div
      className="absolute inset-x-0 border-y border-[color-mix(in_srgb,var(--color-blockout)_50%,transparent)] bg-hatch"
      style={{ top, height: h }}
    >
      <span className="absolute top-1 right-1 rounded-[3px] bg-bg px-1 font-mono text-[10px] text-text-muted">
        {label}
      </span>
    </div>
  );
  const event = (top: number, h: number, title: string, color: string, conflict = false) => (
    <div
      className="absolute right-[3px] left-[3px] flex items-center gap-1 overflow-hidden rounded-sm px-1.5 text-xs"
      style={{
        top: top + 1,
        height: h - 2,
        background: `color-mix(in srgb, ${color} 17%, var(--color-bg))`,
        ...(conflict
          ? {
              border: '1px solid var(--color-conflict)',
              borderLeft: '3px solid var(--color-conflict)',
            }
          : { borderLeft: `3px solid ${color}` }),
      }}
    >
      <span className="truncate font-medium">{title}</span>
      {conflict && <Warning size={11} weight="fill" className="flex-none text-conflict" />}
    </div>
  );
  return (
    <div className="grid grid-cols-[40px_repeat(3,minmax(0,1fr))] overflow-hidden rounded-md border border-border">
      <div className="relative" style={{ height: hour * 5 }}>
        {[9, 10, 11, 12].map((h, i) => (
          <span
            key={h}
            className="absolute right-1.5 -mt-1.5 font-mono text-[10px] text-text-muted"
            style={{ top: (i + 1) * hour }}
          >
            {h}:00
          </span>
        ))}
      </div>
      {[0, 1, 2].map((col) => (
        <div
          key={col}
          className={
            col === 1
              ? 'relative border-l border-border bg-[color-mix(in_srgb,var(--color-accent)_4%,var(--color-bg))]'
              : 'relative border-l border-border'
          }
        >
          {[1, 2, 3, 4].map((h) => (
            <div
              key={h}
              className="absolute inset-x-0 border-t border-border"
              style={{ top: h * hour }}
            />
          ))}
          {col === 0 && block(0, hour * 5, 'Work')}
          {col === 0 && event(hour, hour, 'Standup', 'var(--tag-work)')}
          {col === 1 && event(hour * 1.5, hour * 1.5, 'Review', 'var(--tag-work)', true)}
          {col === 1 && (
            <div
              className="absolute right-[3px] left-[45%] overflow-hidden rounded-sm border border-dashed border-pencil px-1.5 font-mono text-[10px] leading-6 text-pencil"
              style={{ top: hour * 2, height: hour - 2 }}
            >
              ✎ Call
            </div>
          )}
          {col === 2 && event(hour * 3, hour, 'Lunch', 'var(--tag-home)')}
          {col === 1 && (
            <div className="absolute right-0 -left-1 flex items-center" style={{ top: hour * 3.4 }}>
              <span className="size-2 rounded-full bg-danger" />
              <span className="h-0.5 flex-1 bg-danger" />
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

/**
 * The live preview gallery (roadmap §5.5): real components drawn with
 * `vars` set on this subtree only, so the rest of the app keeps its theme
 * until the draft is applied. It is inert: nothing in it can be clicked
 * or focused. Overlays that normally float (tooltip, dialog, toast,
 * palette) are drawn in place.
 */
export function ThemePreview({ vars }: { vars: ThemeVars }) {
  // Two columns when its pane is wide enough (a container query), else one.
  return (
    <div className="@container">
      <div
        inert
        aria-label="Theme preview"
        style={vars as CSSProperties}
        className="grid grid-cols-1 gap-3 rounded-lg bg-bg p-3 font-ui text-base text-text select-none @min-[640px]:grid-cols-2"
      >
        <Panel title="Controls">
          <div className="flex flex-wrap gap-1.5">
            <Button variant="primary">Apply</Button>
            <Button>Cancel</Button>
            <Button variant="ghost">Later</Button>
            <Button variant="danger">Delete</Button>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <input
              readOnly
              value="Dentist tue 3pm"
              className="h-[var(--density-row)] min-w-0 flex-1 rounded-md border border-accent bg-bg px-2.5 text-text outline-none"
            />
            <Switch checked aria-label="On" />
            <Switch checked={false} aria-label="Off" />
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Select
              ariaLabel="Example"
              value="60"
              onValueChange={() => {}}
              options={[{ value: '60', label: '1 hour' }]}
              className="min-w-[110px]"
            />
            <KeyChips binding="Ctrl+K" />
            <KeyChips binding="g w" />
            <span className="flex items-center gap-2 rounded-sm bg-text px-[9px] py-[5px] text-sm whitespace-nowrap text-bg">
              Quick add <span className="font-mono opacity-75">n</span>
            </span>
          </div>
        </Panel>

        <Panel title="Diary cards">
          <div className="flex flex-col gap-2">
            {CARDS.map((c, i) => (
              <ItemCard key={c.id} item={c} conflict={i === 1 ? 'Overlaps Review' : null} />
            ))}
          </div>
        </Panel>

        <Panel title="Week">
          <MiniWeek />
        </Panel>

        <Panel title="Chat">
          <div className="flex flex-col gap-2">
            <div className="max-w-[85%] self-end rounded-[10px_10px_3px_10px] bg-accent-tint px-[11px] py-2 leading-normal">
              Find me time to call Mum tonight
            </div>
            <span className="flex items-center gap-1.5 text-sm text-text-muted">
              <Sparkle size={13} /> I&rsquo;ve pencilled it in after dinner:
            </span>
            <ProposalCard proposal={PROPOSAL} />
          </div>
        </Panel>

        <Panel title="Dialog and toast">
          <div className="flex flex-col gap-3 rounded-lg bg-surface-raised p-[18px] shadow-overlay">
            <span className="text-md font-semibold">Delete Gym?</span>
            <p className="m-0 leading-normal text-text-muted">Ctrl+Z brings it back.</p>
            <div className="flex justify-end gap-2">
              <Button>Cancel</Button>
              <Button variant="danger">Delete</Button>
            </div>
          </div>
          <div className="flex gap-3 rounded-lg bg-surface-raised px-3.5 py-3 shadow-overlay">
            <span className="grid size-[30px] flex-none place-items-center rounded-md bg-accent-tint text-accent-text">
              <Bell />
            </span>
            <span className="flex min-w-0 flex-col gap-[3px]">
              <span className="truncate font-semibold">Dentist in 10 minutes</span>
              <span className="font-mono text-[11.5px] text-text-muted">15:00–15:45</span>
            </span>
          </div>
        </Panel>

        <Panel title="Command palette">
          <div className="flex flex-col overflow-hidden rounded-lg bg-surface-raised shadow-overlay">
            <div className="flex items-center gap-2 border-b border-border px-3 py-2.5">
              <MagnifyingGlass className="text-text-muted" />
              <span>go</span>
              <span className="h-4 w-px animate-pulse bg-accent" />
            </div>
            {[
              ['Week', 'g w'],
              ['Month', 'g m'],
              ['Settings', 'Ctrl+,'],
            ].map(([label, keys], i) => (
              <div
                key={label}
                className={
                  i === 0
                    ? 'flex items-center gap-2 bg-accent-tint px-3 py-2 text-accent-text'
                    : 'flex items-center gap-2 px-3 py-2'
                }
              >
                <span className="flex-1">{label}</span>
                <KeyChips binding={keys as string} />
              </div>
            ))}
          </div>
        </Panel>
      </div>
    </div>
  );
}
