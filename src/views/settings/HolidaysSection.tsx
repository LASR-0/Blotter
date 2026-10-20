import { useState } from 'react';
import { format, parseISO } from 'date-fns';
import { Island, PencilSimple, Plus, Trash, X } from '@phosphor-icons/react';
import { useOverlay } from '@/app/overlay';
import { Button } from '@/components/ui/button';
import { DatePicker } from '@/components/ui/date-picker';
import { isOpError, runOp } from '@/lib/api';
import { useOp } from '@/lib/queries';
import type { Blockout, Holiday } from '@/lib/types';

const FIELD =
  'h-[30px] rounded-md border border-border-strong bg-bg px-2.5 text-base text-text outline-none focus:border-accent';

const errorText = (e: unknown) => (isOpError(e) ? e.message : String(e));

/** "Fri 16 – Mon 19 Oct 2026", or one day. */
function holidaySpan(h: Pick<Holiday, 'start_date' | 'end_date'>): string {
  const a = parseISO(h.start_date);
  const b = parseISO(h.end_date);
  if (h.start_date === h.end_date) return format(a, 'EEE d MMM yyyy');
  const sameYear = a.getFullYear() === b.getFullYear();
  return `${format(a, sameYear ? 'EEE d MMM' : 'EEE d MMM yyyy')} – ${format(b, 'EEE d MMM yyyy')}`;
}

/** The add / edit form: a name and a date range. */
function HolidayForm({ holiday, onDone }: { holiday?: Holiday; onDone: () => void }) {
  const [label, setLabel] = useState(holiday?.label ?? '');
  const [from, setFrom] = useState(holiday?.start_date ?? '');
  const [to, setTo] = useState(holiday?.end_date ?? '');
  const [error, setError] = useState<string | null>(null);
  const ready = label.trim() !== '' && from !== '';

  const save = async () => {
    const dates = { start_date: from, end_date: to || from };
    try {
      await (holiday
        ? runOp('update_holiday', { id: holiday.id, label, ...dates })
        : runOp('create_holiday', { label, ...dates }));
      onDone();
    } catch (e) {
      setError(errorText(e));
    }
  };

  return (
    <form
      className="flex flex-col gap-2 rounded-lg border border-border bg-surface p-3.5"
      onSubmit={(e) => {
        e.preventDefault();
        if (ready) void save();
      }}
    >
      <div className="flex flex-wrap items-center gap-1.5">
        <input
          aria-label="Holiday name"
          value={label}
          onChange={(e) => setLabel(e.target.value)}
          placeholder="Name, e.g. Japan trip"
          autoFocus
          className={`${FIELD} min-w-[180px] flex-1`}
        />
        <DatePicker ariaLabel="First day" value={from} onChange={setFrom} placeholder="From" />
        <span className="text-sm text-text-muted">to</span>
        <DatePicker
          ariaLabel="Last day"
          value={to}
          min={from || undefined}
          onChange={setTo}
          placeholder="Same day"
          clearable
        />
      </div>
      <div className="flex items-center gap-2">
        <span className="flex-1 text-xs text-text-muted">
          Block-outs on these days are lifted for you.
        </span>
        <Button size="sm" variant="ghost" onClick={onDone}>
          Cancel
        </Button>
        <Button type="submit" size="sm" variant="primary" disabled={!ready}>
          {holiday ? 'Save' : 'Add holiday'}
        </Button>
      </div>
      {error && (
        <p role="alert" className="m-0 text-sm text-danger">
          {error}
        </p>
      )}
    </form>
  );
}

/**
 * Settings → Block-outs → Holidays. A holiday lifts the block-outs on its
 * days (they show under each block-out's exceptions), keeps work tasks off
 * them and gives #holiday tasks first pick; it goes once its last day passes.
 */
export function HolidaysSection({ blockouts }: { blockouts: readonly Blockout[] }) {
  const holidays = useOp<Holiday[]>('list_holidays');
  const ask = useOverlay((s) => s.ask);
  const [editing, setEditing] = useState<string | 'new' | null>(null);
  const [error, setError] = useState<string | null>(null);
  const lifted = (id: string) =>
    blockouts.reduce((n, b) => n + b.exceptions.filter((x) => x.holiday_id === id).length, 0);
  const list = holidays.data ?? [];

  return (
    <section className="flex flex-col gap-2.5 pt-2">
      <div className="flex items-center gap-2.5">
        <h2 className="m-0 flex-1 font-heading text-lg font-heading-weight">Holidays</h2>
        {editing !== 'new' && (
          <Button onClick={() => setEditing('new')}>
            <Plus />
            New holiday
          </Button>
        )}
      </div>
      <p className="m-0 max-w-[560px] leading-normal text-text-muted">
        Days away from the usual week. Block-outs on those days are lifted, work tasks aren’t
        slotted in, and tasks tagged <span className="font-mono">#holiday</span> get first pick. A
        holiday disappears from here once its last day has passed.
      </p>
      {editing === 'new' && <HolidayForm onDone={() => setEditing(null)} />}
      {list.length === 0 && editing !== 'new' && holidays.isSuccess && (
        <p className="m-0 text-sm text-text-muted">No holidays coming up.</p>
      )}
      {list.length > 0 && (
        <ul className="m-0 flex list-none flex-col overflow-hidden rounded-lg border border-border p-0">
          {list.map((h) =>
            editing === h.id ? (
              <li key={h.id} className="border-b border-border last:border-b-0">
                <HolidayForm holiday={h} onDone={() => setEditing(null)} />
              </li>
            ) : (
              <li
                key={h.id}
                className="flex items-center gap-3 border-b border-border px-3.5 py-3 last:border-b-0"
              >
                <Island size={18} className="flex-none text-accent-text" />
                <div className="flex min-w-0 flex-1 flex-col gap-[3px]">
                  <span className="truncate font-medium">{h.label}</span>
                  <span className="truncate font-mono text-[11px] text-text-muted">
                    {holidaySpan(h)}
                  </span>
                </div>
                <span className="text-sm text-text-muted">
                  {lifted(h.id) === 0
                    ? 'Nothing to lift'
                    : `Lifts ${lifted(h.id)} block-out ${lifted(h.id) === 1 ? 'day' : 'days'}`}
                </span>
                <Button
                  variant="bare"
                  size="icon-sm"
                  aria-label={`Edit ${h.label}`}
                  title="Edit"
                  onClick={() => setEditing(h.id)}
                >
                  <PencilSimple />
                </Button>
                <Button
                  variant="bare"
                  size="icon-sm"
                  aria-label={`Delete ${h.label}`}
                  title="Delete"
                  onClick={() =>
                    ask({
                      title: `Delete ${h.label}?`,
                      body: 'The block-outs it lifted apply again on those days. Ctrl+Z undoes this.',
                      confirmLabel: 'Delete holiday',
                      danger: true,
                      onConfirm: () =>
                        runOp('delete_holiday', { id: h.id }).then(
                          () => setError(null),
                          (e: unknown) => setError(errorText(e)),
                        ),
                    })
                  }
                >
                  <Trash />
                </Button>
              </li>
            ),
          )}
        </ul>
      )}
      {error && (
        <p role="alert" className="m-0 flex items-center gap-2 text-sm text-danger">
          {error}
          <button type="button" aria-label="Dismiss" onClick={() => setError(null)}>
            <X />
          </button>
        </p>
      )}
    </section>
  );
}
