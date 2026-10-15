import { useEffect, useRef, useState, type KeyboardEvent } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { format, parseISO } from 'date-fns';
import { ArrowSquareOut, BellRinging, BellSimpleSlash, Check, X } from '@phosphor-icons/react';
import { Button } from '@/components/ui/button';
import { isOpError, runOp } from '@/lib/api';
import { cn } from '@/lib/cn';
import { useOp } from '@/lib/queries';
import { useNav } from './nav';
import { useOverlay } from './overlay';
import {
  missedTitle,
  parseSnooze,
  reminderStatus,
  snoozeLabel,
  useReminders,
  type Due,
  type Missed,
  type PendingNotices,
  type SnoozeRequest,
} from './reminders';

/** How long a toast lingers after it's been dealt with. */
const DONE_LINGER_MS = 2500;
const DEFAULT_SNOOZES = [5, 15, 60];

interface SettingsLite {
  snooze_options_min: number[];
}

/** Re-renders every 30 s so "in 10 min" stays current. */
function useNow(): Date {
  const [now, setNow] = useState(() => new Date());
  useEffect(() => {
    const id = window.setInterval(() => setNow(new Date()), 30_000);
    return () => window.clearInterval(id);
  }, []);
  return now;
}

const FOOT_BUTTON =
  'flex h-7 cursor-pointer items-center gap-1.5 rounded-md border border-border-strong bg-transparent px-2.5 text-sm text-text hover:bg-hover';

const errorText = (e: unknown) => (isOpError(e) ? e.message : String(e));

function ReminderCard({ toastKey, due }: { toastKey: string; due: Due }) {
  const dismiss = useReminders((s) => s.dismiss);
  const openItem = useOverlay((s) => s.openItem);
  const settings = useOp<SettingsLite>('get_settings');
  const snoozes = settings.data?.snooze_options_min ?? DEFAULT_SNOOZES;
  const now = useNow();
  const [custom, setCustom] = useState(false);
  const [customText, setCustomText] = useState('');
  const [done, setDone] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const card = useRef<HTMLDivElement>(null);

  // Gone a moment after it's been snoozed or completed.
  useEffect(() => {
    if (!done) return;
    const id = window.setTimeout(() => dismiss(toastKey), DONE_LINGER_MS);
    return () => window.clearTimeout(id);
  }, [done, dismiss, toastKey]);

  // Take focus when nothing else has it, so the keys work straight away
  // without pulling focus out of something the user is typing in.
  useEffect(() => {
    if (document.activeElement === document.body || document.activeElement === null) {
      card.current?.focus();
    }
  }, []);

  const snooze = async (req: SnoozeRequest, label: string) => {
    setError(null);
    try {
      const r = await runOp<{ snoozed_until: string }>('snooze_reminder', {
        reminder_id: due.reminder_id,
        ...req,
      });
      setDone(`Snoozed ${label} · fires again ${format(parseISO(r.snoozed_until), 'HH:mm')}`);
      setCustom(false);
    } catch (e) {
      setError(errorText(e));
    }
  };

  const snoozeCustom = () => {
    const req = parseSnooze(customText, new Date());
    if (!req) {
      setError(`Can't read "${customText}". Try 25m, 2h or 14:00.`);
      return;
    }
    void snooze(req, 'minutes' in req ? snoozeLabel(req.minutes) : `until ${req.until.slice(11)}`);
  };

  const markDone = async () => {
    setError(null);
    try {
      await runOp('complete_item', { id: due.item_id });
      setDone('Marked done');
    } catch (e) {
      setError(errorText(e));
    }
  };

  const open = () => {
    openItem({ id: due.item_id });
    dismiss(toastKey);
  };

  const onKeyDown = (e: KeyboardEvent) => {
    if (e.target instanceof HTMLInputElement) {
      if (e.key === 'Escape') {
        e.preventDefault();
        setCustom(false);
        card.current?.focus();
      }
      return;
    }
    const n = Number(e.key);
    let handled = true;
    if (Number.isInteger(n) && n >= 1 && n <= snoozes.length) {
      const m = snoozes[n - 1] ?? 0;
      void snooze({ minutes: m }, snoozeLabel(m));
    } else if (e.key === String(snoozes.length + 1)) setCustom(true);
    else if (e.key === 'Enter' && e.target === card.current) open();
    else if (e.key === 'x') void markDone();
    else if (e.key === 'Escape') dismiss(toastKey);
    else handled = false;
    if (handled) e.preventDefault();
  };

  return (
    <div
      ref={card}
      role="status"
      tabIndex={-1}
      aria-label={`Reminder: ${due.title}`}
      onKeyDown={onKeyDown}
      className="flex w-[380px] max-w-[calc(100vw-32px)] flex-col rounded-lg bg-surface-raised text-base text-text shadow-overlay focus:outline-none focus-visible:ring-2 focus-visible:ring-focus-ring"
    >
      <div className="flex gap-3 px-3.5 pt-3.5 pb-2.5">
        <span className="grid size-[30px] flex-none place-items-center rounded-md bg-accent-tint text-accent-text">
          <BellRinging weight="fill" size={16} />
        </span>
        <div className="flex min-w-0 flex-1 flex-col gap-[3px]">
          <span className="truncate font-semibold">{due.title}</span>
          <span className="font-mono text-[11.5px] text-text-muted">
            {done ?? reminderStatus(due, now)}
          </span>
        </div>
        <Button
          variant="bare"
          size="icon-sm"
          aria-label="Dismiss (Esc)"
          onClick={() => dismiss(toastKey)}
        >
          <X />
        </Button>
      </div>
      {!done && (
        <>
          <div className="flex flex-wrap items-center gap-1 pr-3.5 pb-3 pl-14">
            <span className="mr-1 text-xs text-text-muted">Snooze</span>
            {snoozes.map((m, i) => (
              <button
                key={m}
                type="button"
                onClick={() => void snooze({ minutes: m }, snoozeLabel(m))}
                className="flex h-[26px] cursor-pointer items-center gap-[5px] rounded-md border border-border-strong bg-surface px-[9px] font-mono text-sm hover:border-accent"
              >
                {snoozeLabel(m)}
                <span className="text-[9.5px] text-text-muted">{i + 1}</span>
              </button>
            ))}
            <button
              type="button"
              onClick={() => setCustom((c) => !c)}
              className={cn(
                'flex h-[26px] cursor-pointer items-center gap-[5px] rounded-md border px-[9px] font-mono text-sm hover:border-accent',
                custom ? 'border-accent bg-accent-tint' : 'border-border-strong bg-surface',
              )}
            >
              Custom…
              <span className="text-[9.5px] text-text-muted">{snoozes.length + 1}</span>
            </button>
          </div>
          {custom && (
            <form
              className="flex gap-1.5 pr-3.5 pb-3 pl-14"
              onSubmit={(e) => {
                e.preventDefault();
                snoozeCustom();
              }}
            >
              <input
                autoFocus
                value={customText}
                onChange={(e) => setCustomText(e.target.value)}
                placeholder="25m, 2h or 14:00"
                aria-label="Snooze for or until"
                className="h-7 min-w-0 flex-1 rounded-md border border-accent bg-bg px-2 font-mono text-sm text-text outline-none"
              />
              <Button type="submit" variant="primary" size="sm">
                Snooze
              </Button>
            </form>
          )}
          {error && <p className="m-0 pr-3.5 pb-3 pl-14 text-sm text-danger">{error}</p>}
          <div className="flex gap-1.5 border-t border-border px-3.5 py-2.5">
            <button type="button" className={FOOT_BUTTON} onClick={open}>
              <ArrowSquareOut />
              Open
              <span className="font-mono text-[10px] text-text-muted">↵</span>
            </button>
            <button type="button" className={FOOT_BUTTON} onClick={() => void markDone()}>
              <Check />
              Mark done
              <span className="font-mono text-[10px] text-text-muted">x</span>
            </button>
          </div>
        </>
      )}
    </div>
  );
}

/** Rows shown before "and n more". */
const MISSED_ROWS = 5;

function MissedCard({ toastKey, missed }: { toastKey: string; missed: Missed }) {
  const dismiss = useReminders((s) => s.dismiss);
  const nav = useNav();
  const rest = missed.items.length - MISSED_ROWS;
  return (
    <div
      role="status"
      tabIndex={-1}
      onKeyDown={(e) => {
        if (e.key === 'Escape') {
          e.preventDefault();
          dismiss(toastKey);
        }
      }}
      className="flex w-[380px] max-w-[calc(100vw-32px)] flex-col rounded-lg bg-surface-raised text-base text-text shadow-overlay focus:outline-none"
    >
      <div className="flex gap-3 px-3.5 pt-3.5 pb-2">
        <span className="grid size-[30px] flex-none place-items-center rounded-md bg-[color-mix(in_srgb,var(--color-warning)_16%,transparent)] text-warning">
          <BellSimpleSlash weight="fill" size={16} />
        </span>
        <div className="flex min-w-0 flex-1 flex-col gap-[3px]">
          <span className="font-semibold">{missedTitle(missed)}</span>
          <span className="text-sm text-text-muted">
            Since {format(parseISO(missed.since), 'EEE d MMM HH:mm')}
          </span>
        </div>
        <Button
          variant="bare"
          size="icon-sm"
          aria-label="Dismiss (Esc)"
          onClick={() => dismiss(toastKey)}
        >
          <X />
        </Button>
      </div>
      <ul className="m-0 flex list-none flex-col p-0 pr-3.5 pb-2.5 pl-14">
        {missed.items.slice(0, MISSED_ROWS).map((d) => (
          <li
            key={`${d.reminder_id}@${d.due}`}
            className="flex h-[30px] items-center gap-2 border-b border-border last:border-b-0"
          >
            <span className="w-[84px] flex-none font-mono text-xs text-text-muted">
              {format(parseISO(d.due), 'EEE HH:mm')}
            </span>
            <span className="min-w-0 flex-1 truncate">{d.title}</span>
          </li>
        ))}
        {rest > 0 && <li className="pt-1.5 text-sm text-text-muted">and {rest} more</li>}
      </ul>
      <div className="flex gap-1.5 border-t border-border px-3.5 py-2.5">
        <button
          type="button"
          className={FOOT_BUTTON}
          onClick={() => {
            nav.go('diary');
            nav.today();
            dismiss(toastKey);
          }}
        >
          Review in Diary
        </button>
        <span className="flex-1" />
        <Button variant="primary" size="sm" onClick={() => dismiss(toastKey)}>
          Mark all seen
        </Button>
      </div>
    </div>
  );
}

/**
 * The in-app reminder toasts (roadmap §4.7): snooze, open and done for each
 * reminder that fires, and one grouped toast for missed ones. They're the
 * way to snooze, since desktop notifications can't carry buttons here.
 */
export function ReminderToasts() {
  const toasts = useReminders((s) => s.toasts);

  useEffect(() => {
    const { addFired, addMissed } = useReminders.getState();
    const unFired = listen<Due>('reminder:fired', (e) => addFired(e.payload));
    const unMissed = listen<Missed>('reminder:missed', (e) => addMissed(e.payload));
    // Listening first, then collect what fired before the window loaded.
    void Promise.all([unFired, unMissed])
      .then(() => invoke<PendingNotices>('take_reminder_notices'))
      .then((p) => {
        p.missed.forEach(addMissed);
        p.fired.forEach(addFired);
      })
      .catch((e: unknown) => console.error('Could not read pending reminders', e));
    return () => {
      void unFired.then((stop) => stop());
      void unMissed.then((stop) => stop());
    };
  }, []);

  if (toasts.length === 0) return null;
  return (
    <div
      aria-label="Reminders"
      className="pointer-events-none fixed right-4 bottom-4 z-30 flex flex-col items-end gap-2.5 [&>*]:pointer-events-auto"
    >
      {toasts.map((t) =>
        t.kind === 'reminder' ? (
          <ReminderCard key={t.key} toastKey={t.key} due={t.due} />
        ) : (
          <MissedCard key={t.key} toastKey={t.key} missed={t.missed} />
        ),
      )}
    </div>
  );
}
