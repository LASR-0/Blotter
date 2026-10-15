import { useEffect, useState, type ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { BellRinging, Info, Plus, X } from '@phosphor-icons/react';
import { ErrorPanel } from '@/components/EmptyState';
import { Button } from '@/components/ui/button';
import { Select } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { TimeField } from '@/components/ui/time-field';
import { isOpError, runOp } from '@/lib/api';
import { formatDuration, parseDuration } from '@/lib/duration';
import { parseTime } from '@/lib/format';
import { useOp } from '@/lib/queries';

/** The `get_settings` fields this page shows. */
interface Settings {
  default_event_reminder_min: number;
  rollover_time: string;
  notifications_enabled: boolean;
  snooze_options_min: number[];
  catch_up_enabled: boolean;
  catch_up_hours: number;
  briefing_enabled: boolean;
  briefing_time: string;
}

type Patch = Partial<Settings>;

const REMINDER_CHOICES = [0, 5, 10, 15, 30, 60, 120, 1440];
const LOOKBACK_CHOICES = [6, 12, 24, 48, 168];
const MAX_SNOOZES = 6;

/** "6h", "24h", "48h", "7d": hours up to two days, then days. */
const lookbackLabel = (h: number) => (h > 48 && h % 24 === 0 ? `${h / 24}d` : `${h}h`);
const errorText = (e: unknown) => (isOpError(e) ? e.message : String(e));

function Row({
  label,
  description,
  children,
}: {
  label: string;
  description?: ReactNode;
  children: ReactNode;
}) {
  return (
    <div className="flex items-center gap-6 border-b border-border py-3">
      <div className="flex min-w-0 flex-1 flex-col gap-[3px]">
        <span className="font-medium">{label}</span>
        {description && (
          <span className="text-sm leading-[1.45] text-text-muted">{description}</span>
        )}
      </div>
      {children}
    </div>
  );
}

/** The snooze lengths offered on a reminder, as removable chips plus an add field. */
function SnoozeChips({ value, onChange }: { value: number[]; onChange: (next: number[]) => void }) {
  const [adding, setAdding] = useState(false);
  const [text, setText] = useState('');
  const [error, setError] = useState(false);
  const add = () => {
    const m = parseDuration(text);
    if (!m || m > 7 * 24 * 60) {
      setError(true);
      return;
    }
    onChange([...value, m]);
    setAdding(false);
    setText('');
  };
  return (
    <div className="flex flex-wrap items-center justify-end gap-1">
      {value.map((m) => (
        <span
          key={m}
          className="flex h-[26px] items-center gap-1 rounded-full border border-border-strong bg-surface-raised pr-1.5 pl-2.5 font-mono text-sm"
        >
          {formatDuration(m)}
          {value.length > 1 && (
            <button
              type="button"
              aria-label={`Remove ${formatDuration(m)}`}
              onClick={() => onChange(value.filter((x) => x !== m))}
              className="grid cursor-pointer place-items-center text-text-muted hover:text-text"
            >
              <X size={11} />
            </button>
          )}
        </span>
      ))}
      {adding ? (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            add();
          }}
        >
          <input
            autoFocus
            value={text}
            onChange={(e) => {
              setText(e.target.value);
              setError(false);
            }}
            onBlur={() => {
              if (!text.trim()) setAdding(false);
            }}
            onKeyDown={(e) => {
              if (e.key === 'Escape') {
                e.preventDefault();
                setAdding(false);
                setText('');
              }
            }}
            placeholder="30m"
            aria-label="Add a snooze length"
            aria-invalid={error}
            className="h-[26px] w-16 rounded-full border border-accent bg-bg px-2.5 font-mono text-sm text-text outline-none aria-[invalid=true]:border-danger"
          />
        </form>
      ) : (
        value.length < MAX_SNOOZES && (
          <button
            type="button"
            aria-label="Add a snooze length"
            onClick={() => setAdding(true)}
            className="grid size-[26px] cursor-pointer place-items-center rounded-full border border-dashed border-border-strong text-text-muted hover:text-text"
          >
            <Plus size={12} />
          </button>
        )
      )}
    </div>
  );
}

/** Settings → Notifications (design S5): how reminders reach you. */
export function NotificationsPage() {
  const settings = useOp<Settings>('get_settings');
  const [error, setError] = useState<string | null>(null);
  const [test, setTest] = useState<string | null>(null);
  const [autostart, setAutostart] = useState<boolean | null>(null);
  const [rollover, setRollover] = useState<string | null>(null);
  const [briefing, setBriefing] = useState<string | null>(null);

  useEffect(() => {
    invoke<boolean>('get_autostart')
      .then(setAutostart)
      .catch((e: unknown) => setError(errorText(e)));
  }, []);

  const save = (patch: Patch) => {
    setError(null);
    runOp('update_settings', patch).catch((e: unknown) => setError(errorText(e)));
  };

  if (settings.isError)
    return (
      <div className="px-8 pt-6">
        <ErrorPanel title="Couldn't load your settings" message={errorText(settings.error)} />
      </div>
    );
  const s = settings.data;
  if (!s) return null;
  const rolloverText = rollover ?? s.rollover_time;

  return (
    <div className="flex max-w-[720px] flex-col px-8 pt-6 pb-12">
      <div className="mb-3.5 flex items-center gap-2.5">
        <h1 className="m-0 flex-1 font-heading text-xl font-heading-weight">Notifications</h1>
        <Button
          onClick={() => {
            setTest(null);
            invoke('send_test_notification')
              .then(() =>
                setTest(
                  'Sent. If nothing appeared, check that notifications and Do Not Disturb allow them in your system settings.',
                ),
              )
              .catch((e: unknown) => setTest(errorText(e)));
          }}
        >
          <BellRinging />
          Send test
        </Button>
      </div>
      {test && <p className="m-0 mb-2 text-sm text-text-muted">{test}</p>}
      {error && <p className="m-0 mb-2 text-sm text-danger">{error}</p>}

      <Row
        label="Notifications"
        description="Desktop notifications for reminders. Reminders still show inside Blotter when this is off."
      >
        <Switch
          aria-label="Notifications"
          checked={s.notifications_enabled}
          onCheckedChange={(on) => save({ notifications_enabled: on })}
        />
      </Row>
      <Row
        label="Default event reminder"
        description="Events only. Tasks have no default reminder."
      >
        <Select
          ariaLabel="Default event reminder"
          value={String(s.default_event_reminder_min)}
          onValueChange={(v) => save({ default_event_reminder_min: Number(v) })}
          options={REMINDER_CHOICES.map((m) => ({
            value: String(m),
            label: m === 0 ? 'None' : `${formatDuration(m)} before`,
          }))}
          className="min-w-[130px] font-mono text-sm"
        />
      </Row>
      <Row
        label="Snooze options"
        description="The lengths offered on a reminder, on keys 1 to 6. Custom… takes any length or a time."
      >
        <SnoozeChips
          value={s.snooze_options_min}
          onChange={(next) => save({ snooze_options_min: next })}
        />
      </Row>
      <Row
        label="Catch up on missed reminders"
        description="Reminders missed while Blotter was closed or your computer was asleep arrive together as one notification."
      >
        <Switch
          aria-label="Catch up on missed reminders"
          checked={s.catch_up_enabled}
          onCheckedChange={(on) => save({ catch_up_enabled: on })}
        />
      </Row>
      <Row label="Catch-up lookback" description="Anything older is skipped quietly.">
        <Select
          ariaLabel="Catch-up lookback"
          value={String(s.catch_up_hours)}
          onValueChange={(v) => save({ catch_up_hours: Number(v) })}
          options={[...new Set([...LOOKBACK_CHOICES, s.catch_up_hours])]
            .sort((a, b) => a - b)
            .map((h) => ({ value: String(h), label: lookbackLabel(h) }))}
          className="min-w-[130px] font-mono text-sm"
        />
      </Row>
      <Row
        label="Daily briefing"
        description="Each morning the AI sums up your day and suggests times for Inbox tasks. It arrives as a notification and opens in the chat. Off by default."
      >
        <Switch
          aria-label="Daily briefing"
          checked={s.briefing_enabled}
          onCheckedChange={(on) => save({ briefing_enabled: on })}
        />
      </Row>
      {s.briefing_enabled && (
        <Row label="Briefing time" description="Or ask for one any time from the command palette.">
          <TimeField
            ariaLabel="Briefing time"
            value={briefing ?? s.briefing_time}
            onChange={(v) => {
              setBriefing(v);
              if (parseTime(v) === v && v !== s.briefing_time) save({ briefing_time: v });
            }}
          />
        </Row>
      )}
      <Row
        label="Rollover time"
        description="When unfinished tasks from earlier days are moved on."
      >
        <TimeField
          ariaLabel="Rollover time"
          value={rolloverText}
          onChange={(v) => {
            setRollover(v);
            // Save once it reads as a whole time (typed, picked or tidied on blur).
            if (parseTime(v) === v && v !== s.rollover_time) save({ rollover_time: v });
          }}
        />
      </Row>
      <Row
        label="Launch at login"
        description="Starts Blotter in the tray when you sign in, so reminders keep firing after a restart."
      >
        <Switch
          aria-label="Launch at login"
          disabled={autostart === null}
          checked={autostart ?? false}
          onCheckedChange={(on) => {
            setError(null);
            invoke<boolean>('set_autostart', { enabled: on })
              .then(setAutostart)
              .catch((e: unknown) => setError(errorText(e)));
          }}
        />
      </Row>

      <div className="mt-3.5 flex gap-2.5 rounded-md border border-border px-3.5 py-3 text-sm leading-normal text-text-muted">
        <Info size={15} className="mt-px flex-none" />
        <span>
          Desktop notifications can’t carry buttons here, so open Blotter from the tray to snooze:
          each reminder waits there as a toast with the snooze options.
        </span>
      </div>
    </div>
  );
}
