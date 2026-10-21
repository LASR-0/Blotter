import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { checkForUpdates } from '@/app/updates';
import { ErrorPanel } from '@/components/EmptyState';
import { Button } from '@/components/ui/button';
import { Segmented } from '@/components/ui/segmented';
import { Select } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { TimeField } from '@/components/ui/time-field';
import { isOpError, runOp } from '@/lib/api';
import { cn } from '@/lib/cn';
import { formatDuration } from '@/lib/duration';
import { parseTime } from '@/lib/format';
import {
  sharedSpan,
  type ClockStyle,
  type PlanningHours,
  type Span,
  type WeekStart,
} from '@/lib/prefs';
import { useOp } from '@/lib/queries';
import { SettingRow } from './SettingRow';

/** The `get_settings` fields this page shows. */
interface Settings {
  timezone: string;
  week_start: WeekStart;
  clock: ClockStyle;
  planning_hours: PlanningHours;
  default_task_duration_min: number;
  default_event_duration_min: number;
  slot_buffer_min: number;
  rollover_mode: 'inbox' | 'reslot';
  close_to_tray: boolean;
  update_check_enabled: boolean;
}

const DAYS = [
  ['mon', 'Monday'],
  ['tue', 'Tuesday'],
  ['wed', 'Wednesday'],
  ['thu', 'Thursday'],
  ['fri', 'Friday'],
  ['sat', 'Saturday'],
  ['sun', 'Sunday'],
] as const;
type DayKey = (typeof DAYS)[number][0];

const TASK_DURATIONS = [10, 15, 20, 30, 45, 60, 90, 120];
const EVENT_DURATIONS = [15, 30, 45, 60, 90, 120, 180];
const BUFFERS = [0, 5, 10, 15, 30];

const errorText = (e: unknown) => (isOpError(e) ? e.message : String(e));

/** A start–end pair of time fields that saves once both read as a valid window. */
function SpanFields({
  label,
  span,
  onSave,
  disabled,
}: {
  label: string;
  span: Span;
  onSave: (s: Span) => void;
  disabled?: boolean;
}) {
  const [start, setStart] = useState(span.start);
  const [end, setEnd] = useState(span.end);
  const [seen, setSeen] = useState(span);
  if (span.start !== seen.start || span.end !== seen.end) {
    setSeen(span);
    setStart(span.start);
    setEnd(span.end);
  }
  const commit = (s: string, e: string) => {
    if (parseTime(s) === s && parseTime(e) === e && s < e && (s !== span.start || e !== span.end))
      onSave({ start: s, end: e });
  };
  const backwards = parseTime(start) === start && parseTime(end) === end && start >= end;
  return (
    <span className="flex items-center gap-1.5">
      <TimeField
        ariaLabel={`${label} start`}
        value={start}
        disabled={disabled}
        onChange={(v) => {
          setStart(v);
          commit(v, end);
        }}
        className="w-[92px]"
      />
      <span className="text-text-muted">–</span>
      <TimeField
        ariaLabel={`${label} end`}
        value={end}
        disabled={disabled}
        onChange={(v) => {
          setEnd(v);
          commit(start, v);
        }}
        className={cn('w-[92px]', backwards && 'border-danger')}
      />
    </span>
  );
}

function PlanningEditor({
  hours,
  onSave,
}: {
  hours: PlanningHours;
  onSave: (h: PlanningHours) => void;
}) {
  const shared = sharedSpan(hours);
  const [perDay, setPerDay] = useState(shared === null);
  const fallback: Span = shared ??
    DAYS.map(([d]) => hours[d]).find(Boolean) ?? {
      start: '07:00',
      end: '22:00',
    };
  const setDay = (d: DayKey, span: Span | null) => onSave({ ...hours, [d]: span });

  return (
    <div className="flex flex-col gap-2 border-b border-border py-3">
      <div className="flex items-center gap-6">
        <div className="flex min-w-0 flex-1 flex-col gap-[3px]">
          <span className="font-medium">Planning hours</span>
          <span className="text-sm leading-[1.45] text-text-muted">
            Flexible tasks are only placed inside this window. Fixed items can go anywhere.
          </span>
        </div>
        {!perDay && (
          <SpanFields
            label="Planning hours"
            span={fallback}
            onSave={(s) => onSave(Object.fromEntries(DAYS.map(([d]) => [d, s])) as PlanningHours)}
          />
        )}
      </div>
      <label className="flex items-center gap-2 self-start text-sm text-text-muted">
        <Switch
          aria-label="Customise per weekday"
          checked={perDay}
          onCheckedChange={(on) => {
            setPerDay(on);
            // Back to one window: every day gets it.
            if (!on) onSave(Object.fromEntries(DAYS.map(([d]) => [d, fallback])) as PlanningHours);
          }}
        />
        Customise per weekday
      </label>
      {perDay && (
        <div className="flex flex-col gap-1.5 pl-1">
          {DAYS.map(([d, name]) => {
            const span = hours[d];
            return (
              <div key={d} className="flex h-[34px] items-center gap-3">
                <Switch
                  aria-label={`Plan on ${name}`}
                  checked={span !== null}
                  onCheckedChange={(on) => setDay(d, on ? fallback : null)}
                />
                <span className={cn('w-[96px]', !span && 'text-text-muted')}>{name}</span>
                {span ? (
                  <SpanFields label={name} span={span} onSave={(s) => setDay(d, s)} />
                ) : (
                  <span className="text-sm text-text-muted">No flexible tasks</span>
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

/** Settings → General (design S1, roadmap §5.9). */
export function GeneralPage() {
  const settings = useOp<Settings>('get_settings');
  const [error, setError] = useState<string | null>(null);
  const [autostart, setAutostart] = useState<boolean | null>(null);

  useEffect(() => {
    invoke<boolean>('get_autostart')
      .then(setAutostart)
      .catch((e: unknown) => setError(errorText(e)));
  }, []);

  const save = (patch: Partial<Settings>) => {
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
  const durations = (list: number[], current: number) =>
    [...new Set([...list, current])]
      .sort((a, b) => a - b)
      .map((m) => ({ value: String(m), label: formatDuration(m) }));

  return (
    <div className="flex max-w-[720px] flex-col px-8 pt-6 pb-12">
      <h1 className="m-0 mb-3.5 font-heading text-xl font-heading-weight">General</h1>
      {error && <p className="m-0 mb-2 text-sm text-danger">{error}</p>}

      <SettingRow label="Week starts on" description="Affects Week, Month and Year views.">
        <Select
          ariaLabel="Week starts on"
          value={s.week_start}
          onValueChange={(v) => save({ week_start: v })}
          options={[
            { value: 'mon', label: 'Monday' },
            { value: 'sun', label: 'Sunday' },
            { value: 'sat', label: 'Saturday' },
          ]}
          className="min-w-[130px]"
        />
      </SettingRow>
      <SettingRow label="Clock">
        <Segmented
          ariaLabel="Clock"
          value={s.clock}
          onChange={(v) => save({ clock: v })}
          options={[
            { value: '12h', label: '12-hour' },
            { value: '24h', label: '24-hour' },
          ]}
        />
      </SettingRow>
      <SettingRow
        label="Timezone"
        description="Blotter follows your computer's timezone, so it moves with you when you travel. Fixed items keep the timezone they were created in."
      >
        <span className="font-mono text-sm text-text-muted">{s.timezone}</span>
      </SettingRow>
      <PlanningEditor hours={s.planning_hours} onSave={(h) => save({ planning_hours: h })} />
      <SettingRow
        label="Default task duration"
        description="Used when a task has no duration and needs auto-slotting."
      >
        <Select
          ariaLabel="Default task duration"
          value={String(s.default_task_duration_min)}
          onValueChange={(v) => save({ default_task_duration_min: Number(v) })}
          options={durations(TASK_DURATIONS, s.default_task_duration_min)}
          className="min-w-[100px] font-mono text-sm"
        />
      </SettingRow>
      <SettingRow label="Default event length" description="When a new event has no end time.">
        <Select
          ariaLabel="Default event length"
          value={String(s.default_event_duration_min)}
          onValueChange={(v) => save({ default_event_duration_min: Number(v) })}
          options={durations(EVENT_DURATIONS, s.default_event_duration_min)}
          className="min-w-[100px] font-mono text-sm"
        />
      </SettingRow>
      <SettingRow
        label="Buffer between items"
        description="Free time kept around items when tasks are slotted."
      >
        <Select
          ariaLabel="Buffer between items"
          value={String(s.slot_buffer_min)}
          onValueChange={(v) => save({ slot_buffer_min: Number(v) })}
          options={[...new Set([...BUFFERS, s.slot_buffer_min])]
            .sort((a, b) => a - b)
            .map((m) => ({ value: String(m), label: m === 0 ? 'None' : formatDuration(m) }))}
          className="min-w-[100px] font-mono text-sm"
        />
      </SettingRow>
      <SettingRow
        label="Unfinished flexible tasks"
        description="At rollover, or on the first launch of a new day. Fixed items never move. The rollover time is under Notifications."
      >
        <Segmented
          ariaLabel="Unfinished flexible tasks"
          value={s.rollover_mode}
          onChange={(v) => save({ rollover_mode: v })}
          options={[
            { value: 'reslot', label: 'Re-slot' },
            { value: 'inbox', label: 'Return to Inbox' },
          ]}
        />
      </SettingRow>
      <SettingRow
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
      </SettingRow>
      <SettingRow
        label="Close to tray"
        description="Closing the window hides Blotter, so reminders still fire. Quit from the tray or the command palette. Off: closing quits."
      >
        <Switch
          aria-label="Close to tray"
          checked={s.close_to_tray}
          onCheckedChange={(on) => save({ close_to_tray: on })}
        />
      </SettingRow>
      <SettingRow
        label="Check for updates"
        description="Once a day, Blotter looks for a new release and says when one is out. Installer and AppImage copies update in place; package-manager installs get a note."
      >
        <span className="flex items-center gap-3">
          <Button size="sm" onClick={() => void checkForUpdates()}>
            Check now
          </Button>
          <Switch
            aria-label="Check for updates"
            checked={s.update_check_enabled}
            onCheckedChange={(on) => save({ update_check_enabled: on })}
          />
        </span>
      </SettingRow>
    </div>
  );
}
