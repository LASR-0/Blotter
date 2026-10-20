import { useState, type FormEvent, type KeyboardEvent, type ReactNode } from 'react';
import { Dialog } from 'radix-ui';
import { format, isToday, parseISO } from 'date-fns';
import {
  ArrowsClockwise,
  ArrowsOutLineHorizontal,
  Bell,
  Calendar,
  CalendarBlank,
  CheckSquare,
  Clock,
  FlagCheckered,
  Plus,
  PushPin,
  Timer,
  Trash,
  X,
} from '@phosphor-icons/react';
import { ErrorPanel } from '@/components/EmptyState';
import { Button } from '@/components/ui/button';
import { DatePicker } from '@/components/ui/date-picker';
import { Kbd } from '@/components/ui/kbd';
import { Select } from '@/components/ui/select';
import { Segmented } from '@/components/ui/segmented';
import { TimeField } from '@/components/ui/time-field';
import { isOpError, runBatch, runOp } from '@/lib/api';
import { cn } from '@/lib/cn';
import { formatDuration } from '@/lib/duration';
import { clockTime, parseTime, timeOf } from '@/lib/format';
import {
  buildChanges,
  draftFromItem,
  emptyDraft,
  isRecurring,
  type ItemDraft,
  type RepeatScope,
} from '@/lib/itemDraft';
import { describeRule, presetOf, ruleFor, withEnd, type RepeatPreset } from '@/lib/repeat';
import { useOp } from '@/lib/queries';
import { tagColor } from '@/lib/tags';
import type { Item, ItemKind } from '@/lib/types';
import { useFocus } from './focus';
import { useOverlay } from './overlay';

interface ItemDetail extends Item {
  reminders: { id: string; offset_min: number | null; at: string | null }[];
}

const REMINDER_OFFSETS = [5, 10, 15, 30, 60, 120, 1440];

const errorText = (e: unknown) =>
  isOpError(e) ? [e.message, ...e.details].filter(Boolean).join(' · ') : String(e);

function reminderLabel(r: { offset_min: number | null; at: string | null }): string {
  if (r.offset_min !== null)
    return r.offset_min === 0 ? 'at start' : `${formatDuration(r.offset_min)} before`;
  return r.at ? `${format(parseISO(r.at), 'EEE d MMM')} ${clockTime(parseISO(r.at))}` : '';
}

function Row({ label, children, top }: { label: string; children: ReactNode; top?: boolean }) {
  return (
    <>
      <span className={cn('text-text-muted', top && 'self-start pt-1.5')}>{label}</span>
      <div className="flex min-w-0 flex-wrap items-center gap-1.5">{children}</div>
    </>
  );
}

const FIELD =
  'flex h-[30px] items-center gap-1.5 rounded-md border border-border-strong bg-bg px-2.5 font-mono text-sm text-text focus-within:border-accent';

function TagEditor({ tags, onChange }: { tags: string[]; onChange: (t: string[]) => void }) {
  const [draft, setDraft] = useState('');
  const add = () => {
    const tag = draft.trim().replace(/^#/, '').toLowerCase();
    if (tag && !tags.includes(tag)) onChange([...tags, tag]);
    setDraft('');
  };
  const onKey = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault();
      add();
    } else if (e.key === 'Backspace' && !draft && tags.length) {
      onChange(tags.slice(0, -1));
    }
  };
  return (
    <>
      {tags.map((t) => (
        <span
          key={t}
          className="flex h-[26px] items-center gap-1.5 rounded-full border border-border-strong pr-1.5 pl-2.5 text-sm"
        >
          <span className="size-[7px] rounded-full" style={{ background: tagColor(t) }} />
          {t}
          <button
            type="button"
            aria-label={`Remove tag ${t}`}
            onClick={() => onChange(tags.filter((x) => x !== t))}
            className="grid cursor-pointer place-items-center text-text-muted hover:text-text"
          >
            <X size={11} />
          </button>
        </span>
      ))}
      <label className="flex h-[26px] items-center gap-1 rounded-full border border-dashed border-border-strong px-2.5 text-sm text-text-muted focus-within:border-accent">
        <Plus size={12} />
        <input
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={onKey}
          onBlur={add}
          placeholder="Tag"
          aria-label="Add tag"
          className="w-16 bg-transparent text-text outline-none"
        />
      </label>
    </>
  );
}

function ItemForm({
  item,
  initial,
  onDone,
}: {
  item?: ItemDetail;
  initial: ItemDraft;
  onDone: () => void;
}) {
  const [draft, setDraft] = useState(initial);
  const [showDue, setShowDue] = useState(Boolean(initial.dueDate));
  const [addingReminder, setAddingReminder] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const recurring = Boolean(item && isRecurring(item));
  // The series row itself (e.g. from search): every change applies to all of it.
  const isSeries = Boolean(item?.rrule && !item.parent_id);
  const asksScope = recurring && !isSeries;
  // Overrides of one occurrence carry no rule of their own.
  const isOverride = Boolean(item?.parent_id && !item.rrule);
  const [scope, setScope] = useState<RepeatScope>('this');
  const set = <K extends keyof ItemDraft>(key: K, value: ItemDraft[K]) =>
    setDraft((d) => ({ ...d, [key]: value }));

  const save = async () => {
    const result = buildChanges(draft, item ? { item, draft: initial, scope } : undefined);
    if (!result.ok) {
      setError(result.error);
      return;
    }
    setSaving(true);
    try {
      if (result.create) {
        const created = await runOp<Item>('create_item', result.create);
        // The new item takes keyboard focus, so it's clear where it landed.
        useFocus.getState().setItem(created.id);
      } else if (result.calls.length) {
        await runBatch(result.calls);
      }
      onDone();
    } catch (e) {
      setError(errorText(e));
      setSaving(false);
    }
  };

  const remove = async () => {
    if (!item) return;
    if (!confirmDelete) {
      setConfirmDelete(true);
      return;
    }
    try {
      await runOp('delete_item', asksScope ? { id: item.id, scope } : { id: item.id });
      onDone();
    } catch (e) {
      setError(errorText(e));
    }
  };

  const submit = (e: FormEvent) => {
    e.preventDefault();
    void save();
  };
  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      void save();
    }
  };

  const timed = Boolean(draft.date && parseTime(draft.time));
  const edited = item ? parseISO(item.updated_at) : null;

  return (
    <form onSubmit={submit} onKeyDown={onKeyDown} className="flex flex-col">
      <div className="flex items-center gap-2 px-[18px] pt-3.5">
        <Dialog.Title className="m-0 text-sm font-normal text-text-muted">
          {item ? 'Edit item' : 'New item'}
        </Dialog.Title>
        {item && recurring && (
          <span className="flex items-center gap-1 rounded-full border border-border-strong px-[7px] py-px text-xs text-text-muted">
            <ArrowsClockwise />
            Recurring
          </span>
        )}
        {item?.status === 'done' && item.completed_at && (
          <span className="rounded-full bg-accent-tint px-[7px] py-px font-mono text-xs text-accent-text">
            ✓ {timeOf(item.completed_at)}
          </span>
        )}
        <span className="flex-1" />
        <Dialog.Close
          type="button"
          aria-label="Close"
          className="grid size-[26px] cursor-pointer place-items-center rounded-md text-text-muted hover:bg-hover"
        >
          <X />
        </Dialog.Close>
      </div>

      <div className="px-[18px] pt-1.5">
        <input
          autoFocus
          value={draft.title}
          onChange={(e) => set('title', e.target.value)}
          placeholder="What needs doing?"
          aria-label="Title"
          className="h-10 w-full rounded-sm bg-transparent px-1 font-heading text-xl font-heading-weight text-text outline-none focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus-ring"
        />
      </div>

      <div className="grid grid-cols-[110px_minmax(0,1fr)] items-center gap-x-3.5 gap-y-2.5 px-[18px] py-4">
        <Row label="Kind">
          <Segmented<ItemKind>
            ariaLabel="Kind"
            value={draft.kind}
            onChange={(kind) =>
              setDraft((d) => ({
                ...d,
                kind,
                // Events default to fixed; switching to one pins it.
                anchoring: kind === 'event' ? 'fixed' : d.anchoring,
                // New events start with the default 10-minute reminder.
                reminders:
                  !item && kind === 'event' && d.reminders.length === 0
                    ? [{ offset_min: 10, at: null }]
                    : d.reminders,
              }))
            }
            options={[
              {
                value: 'task',
                label: (
                  <>
                    <CheckSquare />
                    Task
                  </>
                ),
              },
              {
                value: 'event',
                label: (
                  <>
                    <Calendar />
                    Event
                  </>
                ),
              },
              {
                value: 'reminder',
                label: (
                  <>
                    <Bell />
                    Reminder
                  </>
                ),
              },
            ]}
          />
        </Row>

        <Row label="Anchoring">
          <Segmented
            ariaLabel="Anchoring"
            value={draft.anchoring}
            onChange={(a) => set('anchoring', a)}
            options={[
              {
                value: 'flexible',
                label: (
                  <>
                    <ArrowsOutLineHorizontal />
                    Flexible
                  </>
                ),
              },
              {
                value: 'fixed',
                label: (
                  <>
                    <PushPin />
                    Fixed
                  </>
                ),
              },
            ]}
          />
          <span className="text-xs leading-[1.4] text-text-muted">
            {draft.anchoring === 'fixed'
              ? 'Stays exactly here, even inside block-outs.'
              : 'Can be moved by auto-slot. Never placed in block-outs.'}
          </span>
        </Row>

        <Row label="When">
          <DatePicker
            value={draft.date}
            onChange={(v) => set('date', v)}
            ariaLabel="Date"
            placeholder="No date"
            clearable
            icon={<CalendarBlank className="text-text-muted" />}
          />
          <TimeField
            value={draft.time}
            disabled={!draft.date}
            onChange={(v) => set('time', v)}
            ariaLabel="Time"
            icon={<Clock className="text-text-muted" />}
          />
          <label className={FIELD}>
            <Timer className="text-text-muted" />
            <input
              value={draft.duration}
              onChange={(e) => set('duration', e.target.value)}
              placeholder="30m"
              aria-label="Duration"
              className="w-14 bg-transparent outline-none placeholder:text-text-muted"
            />
          </label>
          {!showDue && (
            <button
              type="button"
              onClick={() => setShowDue(true)}
              className="flex h-[30px] cursor-pointer items-center gap-1.5 rounded-md border border-dashed border-border-strong px-2.5 text-sm text-text-muted hover:text-text"
            >
              <FlagCheckered />
              Due date
            </button>
          )}
          {!draft.date && (
            <span className="w-full text-xs text-text-muted">
              No date:{' '}
              {draft.kind === 'task' ? 'it goes to the Inbox.' : 'events and reminders need one.'}
            </span>
          )}
        </Row>

        {showDue && (
          <Row label="Due">
            <DatePicker
              value={draft.dueDate}
              onChange={(v) => set('dueDate', v)}
              ariaLabel="Due date"
              icon={<FlagCheckered className="text-text-muted" />}
            />
            <TimeField
              value={draft.dueTime}
              disabled={!draft.dueDate}
              onChange={(v) => set('dueTime', v)}
              ariaLabel="Due time"
              icon={<Clock className="text-text-muted" />}
            />
            <Button
              variant="bare"
              size="icon-sm"
              aria-label="Remove due date"
              onClick={() => {
                setDraft((d) => ({ ...d, dueDate: '', dueTime: '' }));
                setShowDue(false);
              }}
            >
              <X />
            </Button>
          </Row>
        )}

        <Row label="Priority">
          <Segmented
            ariaLabel="Priority"
            value={String(draft.priority)}
            onChange={(p) => set('priority', Number(p))}
            options={[
              { value: '0', label: 'None' },
              { value: '1', label: '!1' },
              { value: '2', label: '!2' },
              { value: '3', label: '!3' },
            ]}
          />
        </Row>

        <Row label="Tags">
          <TagEditor tags={draft.tags} onChange={(t) => set('tags', t)} />
        </Row>

        <Row label="Repeats">
          {isOverride ? (
            <span className="text-sm text-text-muted">
              One occurrence of a repeating series; pick All occurrences below to change the rest.
            </span>
          ) : (
            <RepeatPicker
              rule={draft.repeat}
              date={draft.date}
              onChange={(rule) => {
                set('repeat', rule);
                // A rule belongs to the series, never to one occurrence.
                if (asksScope && scope === 'this') setScope('all');
              }}
            />
          )}
        </Row>

        <Row label="Reminders">
          {draft.reminders.map((r, i) => (
            <span
              key={r.id ?? `new-${i}`}
              className="flex h-[26px] items-center gap-1.5 rounded-md border border-border-strong pr-1.5 pl-2.5 font-mono text-sm"
            >
              <Bell className="text-text-muted" />
              {reminderLabel(r)}
              <button
                type="button"
                aria-label="Remove reminder"
                onClick={() =>
                  set(
                    'reminders',
                    draft.reminders.filter((_, j) => j !== i),
                  )
                }
                className="grid cursor-pointer place-items-center text-text-muted hover:text-text"
              >
                <X size={11} />
              </button>
            </span>
          ))}
          {addingReminder ? (
            <Select
              defaultOpen
              ariaLabel="Remind me"
              placeholder="Remind me…"
              value=""
              onOpenChange={(open) => !open && setAddingReminder(false)}
              onValueChange={(v) => {
                set('reminders', [...draft.reminders, { offset_min: Number(v), at: null }]);
                setAddingReminder(false);
              }}
              options={REMINDER_OFFSETS.map((m) => ({
                value: String(m),
                label: `${formatDuration(m)} before`,
              }))}
              className="h-[26px] border-accent text-sm"
            />
          ) : (
            <button
              type="button"
              disabled={!timed}
              title={timed ? undefined : 'Give it a date and time first'}
              onClick={() => setAddingReminder(true)}
              className="flex h-[26px] cursor-pointer items-center gap-1 rounded-md border border-dashed border-border-strong px-2.5 text-sm text-text-muted hover:text-text disabled:cursor-default disabled:opacity-50"
            >
              <Plus size={12} />
              Reminder
            </button>
          )}
        </Row>

        <Row label="Notes" top>
          <textarea
            rows={2}
            value={draft.notes}
            onChange={(e) => set('notes', e.target.value)}
            placeholder="Notes, or context for Smart slot (when it can happen, what it needs)…"
            aria-label="Notes"
            className="w-full resize-y rounded-md border border-border-strong bg-bg px-2.5 py-[7px] text-text outline-none focus:border-accent"
          />
        </Row>
      </div>

      {asksScope && (
        <div className="mx-[18px] mt-1 flex flex-col gap-2 rounded-md border border-border-strong bg-surface p-3">
          <span className="text-sm font-medium">This is a recurring item. Apply changes to:</span>
          <Segmented
            ariaLabel="Apply changes to"
            value={scope}
            onChange={(v) => setScope(v as RepeatScope)}
            options={[
              { value: 'this', label: 'This occurrence' },
              { value: 'following', label: 'This and following' },
              { value: 'all', label: 'All occurrences' },
            ]}
          />
        </div>
      )}

      {error && (
        <p role="alert" className="mx-[18px] my-0 font-mono text-xs leading-normal text-danger">
          {error}
        </p>
      )}

      <div className="flex items-center gap-2 px-[18px] py-3.5">
        {item && (
          <Button
            variant="ghost"
            className={cn(
              'text-danger',
              confirmDelete && 'bg-[color-mix(in_srgb,var(--color-danger)_12%,transparent)]',
            )}
            onClick={() => void remove()}
            onBlur={() => setConfirmDelete(false)}
          >
            <Trash />
            {confirmDelete
              ? 'Click again to delete'
              : isSeries
                ? 'Delete series'
                : !recurring
                  ? 'Delete'
                  : scope === 'this'
                    ? 'Delete occurrence'
                    : scope === 'following'
                      ? 'Delete this and following'
                      : 'Delete series'}
          </Button>
        )}
        <span className="flex-1" />
        {item && edited && (
          <span className="font-mono text-[10.5px] text-text-muted">
            {item.source === 'user' ? 'created by you' : `created by ${item.source.toUpperCase()}`}{' '}
            · edited {isToday(edited) ? clockTime(edited) : format(edited, 'd MMM')}
          </span>
        )}
        <Dialog.Close asChild>
          <Button>Cancel</Button>
        </Dialog.Close>
        <Button type="submit" variant="primary" disabled={saving}>
          Save
          <Kbd onAccent>Ctrl ↵</Kbd>
        </Button>
      </div>
    </form>
  );
}

/** The Repeats row: preset picker, plus the raw rule (editable when custom). */
function RepeatPicker({
  rule,
  date,
  onChange,
}: {
  rule: string;
  date: string;
  onChange: (rule: string) => void;
}) {
  const preset = presetOf(rule, date);
  const [custom, setCustom] = useState(preset === 'custom');
  const day = date ? parseISO(date) : new Date();
  const options: [RepeatPreset, string][] = [
    ['none', 'Does not repeat'],
    ['daily', 'Every day'],
    ['weekdays', 'Every weekday'],
    ['weekly', `Every week on ${format(day, 'EEEE')}`],
    ['fortnightly', `Every 2 weeks on ${format(day, 'EEEE')}`],
    ['monthly', `Monthly on the ${format(day, 'do')}`],
    ['yearly', `Every year on ${format(day, 'd MMMM')}`],
    ['custom', 'Custom rule…'],
  ];
  return (
    <div className="flex min-w-0 flex-wrap items-center gap-2.5">
      <Select
        ariaLabel="Repeats"
        value={custom ? 'custom' : preset}
        onValueChange={(next) => {
          setCustom(next === 'custom');
          if (next === 'custom') return;
          const r = ruleFor(next, date);
          onChange(r ? withEnd(r, rule) : '');
        }}
        options={options.map(([value, label]) => ({ value, label }))}
        className="min-w-[150px]"
      />
      {custom ? (
        <input
          value={rule}
          onChange={(e) => onChange(e.target.value)}
          placeholder="FREQ=WEEKLY;BYDAY=MO,WE"
          aria-label="Custom repeat rule"
          className="h-[30px] min-w-0 flex-1 rounded-md border border-border-strong bg-bg px-2.5 font-mono text-sm text-text outline-none focus:border-accent"
        />
      ) : (
        rule && (
          <span title={describeRule(rule)} className="font-mono text-[10.5px] text-text-muted">
            RRULE:{rule}
          </span>
        )
      )}
    </div>
  );
}

function ExistingItem({ id, onDone }: { id: string; onDone: () => void }) {
  const detail = useOp<ItemDetail>('get_item', { id });
  if (detail.isPending)
    return (
      <div className="p-6 font-mono text-xs text-text-muted">
        <Dialog.Title className="sr-only">Edit item</Dialog.Title>
        Loading…
      </div>
    );
  if (detail.isError)
    return (
      <div className="p-4">
        <Dialog.Title className="sr-only">Edit item</Dialog.Title>
        <ErrorPanel title="Couldn't open this item" message={errorText(detail.error)} />
      </div>
    );
  const item = detail.data;
  return (
    <ItemForm
      key={item.updated_at}
      item={item}
      initial={draftFromItem(item, item.reminders)}
      onDone={onDone}
    />
  );
}

/** The item dialog: create, edit and delete one task, event or reminder. */
export function ItemDialog() {
  const target = useOverlay((s) => (s.overlay === 'item' ? s.item : null));
  const close = useOverlay((s) => s.close);
  return (
    <Dialog.Root open={target !== null} onOpenChange={(o) => !o && close()}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-40 bg-[color-mix(in_srgb,var(--color-bg)_62%,transparent)]" />
        <Dialog.Content
          aria-describedby={undefined}
          className="fixed top-[60px] left-1/2 z-41 max-h-[calc(100vh-80px)] w-[580px] max-w-[calc(100vw-32px)] -translate-x-1/2 overflow-auto rounded-lg bg-surface-raised text-base text-text shadow-overlay focus:outline-none"
        >
          {target &&
            ('id' in target ? (
              <ExistingItem id={target.id} onDone={close} />
            ) : (
              <ItemForm initial={emptyDraft(target.defaults)} onDone={close} />
            ))}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
