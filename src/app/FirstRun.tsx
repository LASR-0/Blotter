import { useEffect, useState, type CSSProperties, type ReactNode } from 'react';
import { Dialog } from 'radix-ui';
import { invoke } from '@tauri-apps/api/core';
import { Check } from '@phosphor-icons/react';
import { Button } from '@/components/ui/button';
import { Segmented } from '@/components/ui/segmented';
import { Switch } from '@/components/ui/switch';
import { TimeField } from '@/components/ui/time-field';
import { isOpError, runOp } from '@/lib/api';
import { cn } from '@/lib/cn';
import { parseTime } from '@/lib/format';
import { DEFAULT_PLANNING, type WeekStart } from '@/lib/prefs';
import { PRESETS } from '@/theme/presets';
import { useTheme } from '@/theme/theme';
import { useNav } from './nav';
import { useOverlay } from './overlay';

const STEPS = ['Theme', 'Your week', 'Work hours', 'AI', 'Start with your computer'] as const;
const WEEKDAYS = [
  ['mon', 'M'],
  ['tue', 'T'],
  ['wed', 'W'],
  ['thu', 'T'],
  ['fri', 'F'],
  ['sat', 'S'],
  ['sun', 'S'],
] as const;

const errorText = (e: unknown) => (isOpError(e) ? e.message : String(e));
const isTime = (t: string) => parseTime(t) === t;

interface ClaudeStatus {
  path: string | null;
  logged_in: boolean | null;
}

function Times({
  label,
  start,
  end,
  onStart,
  onEnd,
}: {
  label: string;
  start: string;
  end: string;
  onStart: (v: string) => void;
  onEnd: (v: string) => void;
}) {
  return (
    <span className="flex items-center gap-1.5">
      <TimeField
        ariaLabel={`${label} start`}
        value={start}
        onChange={onStart}
        className="w-[92px]"
      />
      <span className="text-text-muted">–</span>
      <TimeField ariaLabel={`${label} end`} value={end} onChange={onEnd} className="w-[92px]" />
    </span>
  );
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="flex min-h-9 items-center gap-4">
      <span className="w-[120px] flex-none text-text-muted">{label}</span>
      {children}
    </div>
  );
}

/**
 * First-run setup (roadmap §5.10): five short steps, each skippable, all by
 * keyboard (Tab moves, Enter goes on, Esc ends the setup). Shown once, on a
 * fresh database; "Run setup again" in the palette brings it back.
 */
export function FirstRun() {
  const open = useOverlay((s) => s.overlay === 'setup');
  // Mounted only while open, so each run starts from the first step.
  return open ? <SetupDialog /> : null;
}

function SetupDialog() {
  const close = useOverlay((s) => s.close);
  const { theme, setTheme } = useTheme();
  const [step, setStep] = useState(0);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [weekStart, setWeekStart] = useState<WeekStart>('mon');
  const [planStart, setPlanStart] = useState('07:00');
  const [planEnd, setPlanEnd] = useState('22:00');
  const [work, setWork] = useState(true);
  const [workDays, setWorkDays] = useState<string[]>(['mon', 'tue', 'wed', 'thu', 'fri']);
  const [workStart, setWorkStart] = useState('08:30');
  const [workEnd, setWorkEnd] = useState('17:00');
  const [openAi, setOpenAi] = useState(true);
  const [claude, setClaude] = useState<ClaudeStatus | null>(null);
  const [autostart, setAutostart] = useState(true);

  useEffect(() => {
    invoke<ClaudeStatus>('claude_code_status')
      .then(setClaude)
      .catch(() => setClaude(null));
  }, []);

  const finish = async () => {
    try {
      await invoke('finish_first_run');
    } catch (e) {
      console.error('Could not record the setup as done', e);
    }
    close();
  };

  /** Applies the current step's choices (not when skipped), then moves on. */
  const next = async (apply: boolean) => {
    setError(null);
    setBusy(true);
    try {
      if (apply) {
        if (step === 1) {
          if (!isTime(planStart) || !isTime(planEnd) || planStart >= planEnd)
            throw new Error('Planning hours need a start before their end.');
          const span = { start: planStart, end: planEnd };
          await runOp('update_settings', {
            week_start: weekStart,
            planning_hours: Object.fromEntries(Object.keys(DEFAULT_PLANNING).map((d) => [d, span])),
          });
        }
        if (step === 2 && work) {
          if (!isTime(workStart) || !isTime(workEnd) || workDays.length === 0)
            throw new Error('Pick the days and times you work, or skip this step.');
          await runOp('create_blockout', {
            label: 'Work',
            weekdays: workDays,
            start_time: workStart,
            end_time: workEnd,
            icon: 'briefcase',
          });
        }
        if (step === 4) await invoke('set_autostart', { enabled: autostart });
      }
      if (step < STEPS.length - 1) {
        setStep(step + 1);
        return;
      }
      await finish();
      if (openAi && !(claude?.logged_in ?? false)) useNav.getState().openSettings('ai');
    } catch (e) {
      setError(e instanceof Error ? e.message : errorText(e));
    } finally {
      setBusy(false);
    }
  };

  const body = [
    <div key="theme" role="radiogroup" aria-label="Theme" className="grid grid-cols-2 gap-2.5">
      {PRESETS.map((t) => (
        <button
          key={t.id}
          type="button"
          role="radio"
          aria-checked={theme.id === t.id}
          onClick={() => setTheme(t.id)}
          style={t.vars as CSSProperties}
          className={cn(
            'flex cursor-pointer flex-col gap-1.5 rounded-lg border bg-bg p-3 text-left font-ui text-text',
            theme.id === t.id
              ? 'border-accent shadow-[0_0_0_1px_var(--color-accent)]'
              : 'border-border',
          )}
        >
          <span className="flex items-center font-heading font-heading-weight">
            <span className="flex-1">{t.label}</span>
            {theme.id === t.id && <Check weight="bold" className="text-accent-text" />}
          </span>
          <span className="rounded-md border border-border bg-surface px-2 py-1.5 text-sm">
            Gym <span className="font-mono text-xs text-text-muted">17:30</span>
          </span>
        </button>
      ))}
    </div>,
    <div key="week" className="flex flex-col gap-3">
      <Field label="Week starts on">
        <Segmented
          ariaLabel="Week starts on"
          value={weekStart}
          onChange={setWeekStart}
          options={[
            { value: 'mon', label: 'Monday' },
            { value: 'sun', label: 'Sunday' },
            { value: 'sat', label: 'Saturday' },
          ]}
        />
      </Field>
      <Field label="Planning hours">
        <Times
          label="Planning hours"
          start={planStart}
          end={planEnd}
          onStart={setPlanStart}
          onEnd={setPlanEnd}
        />
      </Field>
      <p className="m-0 text-sm leading-normal text-text-muted">
        Flexible tasks are only placed between these times. You can set each weekday on its own
        later, in Settings → General.
      </p>
    </div>,
    <div key="work" className="flex flex-col gap-3">
      <label className="flex items-center gap-2.5">
        <Switch aria-label="Add a Work block-out" checked={work} onCheckedChange={setWork} />
        Add a Work block-out
      </label>
      <Field label="Days">
        <span className="flex gap-1">
          {WEEKDAYS.map(([d, l]) => {
            const on = workDays.includes(d);
            return (
              <button
                key={d}
                type="button"
                aria-pressed={on}
                aria-label={d}
                disabled={!work}
                onClick={() => setWorkDays(on ? workDays.filter((x) => x !== d) : [...workDays, d])}
                className={cn(
                  'size-[30px] cursor-pointer rounded-md border text-xs font-medium disabled:cursor-default disabled:opacity-50',
                  on
                    ? 'border-accent bg-accent-tint text-accent-text'
                    : 'border-border-strong text-text-muted',
                )}
              >
                {l}
              </button>
            );
          })}
        </span>
      </Field>
      <Field label="Hours">
        <Times
          label="Work"
          start={workStart}
          end={workEnd}
          onStart={setWorkStart}
          onEnd={setWorkEnd}
        />
      </Field>
      <p className="m-0 text-sm leading-normal text-text-muted">
        Flexible tasks never land in a block-out; fixed things like meetings still can. Lift it on a
        day off with <span className="font-mono">b l</span>.
      </p>
    </div>,
    <div key="ai" className="flex flex-col gap-3">
      <p className="m-0 leading-normal">
        Blotter&rsquo;s AI can plan your day with you: ask it to fit things in, and it pencils them
        in for you to accept.
      </p>
      <p className="m-0 text-sm leading-normal text-text-muted">
        {claude?.logged_in
          ? 'Claude Code is installed and signed in on this computer, so the chat is ready to use.'
          : claude?.path
            ? 'Claude Code is installed but not signed in. Run "claude" in a terminal to sign in, or add an API key.'
            : 'Use Claude Code if you have it, or an API key from Anthropic, OpenAI or a local model.'}
      </p>
      {!claude?.logged_in && (
        <label className="flex items-center gap-2.5">
          <Switch
            aria-label="Open AI providers when done"
            checked={openAi}
            onCheckedChange={setOpenAi}
          />
          Open Settings → AI providers when I&rsquo;m done
        </label>
      )}
    </div>,
    <div key="autostart" className="flex flex-col gap-3">
      <label className="flex items-center gap-2.5">
        <Switch aria-label="Launch at login" checked={autostart} onCheckedChange={setAutostart} />
        Launch Blotter when I sign in
      </label>
      <p className="m-0 text-sm leading-normal text-text-muted">
        It starts quietly in the tray, so reminders fire even if you haven&rsquo;t opened it. You
        can change this in Settings → General.
      </p>
    </div>,
  ];

  return (
    <Dialog.Root open onOpenChange={(o) => !o && void finish()}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-40 bg-[color-mix(in_srgb,var(--color-bg)_80%,transparent)]" />
        <Dialog.Content
          aria-describedby={undefined}
          onOpenAutoFocus={(e) => {
            // Start on the step's own controls rather than the Skip button.
            e.preventDefault();
            (e.currentTarget as HTMLElement | null)
              ?.querySelector<HTMLElement>('[data-step] button, [data-step] input')
              ?.focus();
          }}
          onKeyDown={(e) => {
            // Enter goes on, unless a control that uses Enter has focus.
            const t = e.target as HTMLElement;
            if (
              e.key === 'Enter' &&
              t.tagName !== 'BUTTON' &&
              t.getAttribute('role') !== 'combobox'
            ) {
              e.preventDefault();
              void next(true);
            }
          }}
          className="fixed top-[90px] left-1/2 z-41 flex w-[540px] max-w-[calc(100vw-32px)] -translate-x-1/2 flex-col gap-4 rounded-lg bg-surface-raised p-6 text-base text-text shadow-overlay focus:outline-none"
        >
          <div className="flex items-center gap-1.5">
            {STEPS.map((s, i) => (
              <span
                key={s}
                aria-hidden
                className={cn('h-1 flex-1 rounded-full', i <= step ? 'bg-accent' : 'bg-border')}
              />
            ))}
          </div>
          <div className="flex items-baseline gap-2">
            <Dialog.Title className="m-0 flex-1 font-heading text-xl font-heading-weight">
              {step === 0 ? 'Welcome to Blotter' : STEPS[step]}
            </Dialog.Title>
            <span className="font-mono text-xs text-text-muted">
              {step + 1} / {STEPS.length}
            </span>
          </div>
          {step === 0 && (
            <p className="m-0 -mt-2 text-text-muted">
              A few choices to start with. Skip any of them; everything is in Settings later.
            </p>
          )}
          <div data-step className="min-h-[170px]">
            {body[step]}
          </div>
          {error && <p className="m-0 text-sm text-danger">{error}</p>}
          <div className="flex items-center gap-2">
            <Button variant="ghost" onClick={() => void finish()}>
              Skip setup
            </Button>
            <span className="flex-1" />
            {step > 0 && (
              <Button variant="ghost" disabled={busy} onClick={() => setStep(step - 1)}>
                Back
              </Button>
            )}
            {step > 0 && (
              <Button disabled={busy} onClick={() => void next(false)}>
                Skip
              </Button>
            )}
            <Button variant="primary" disabled={busy} onClick={() => void next(true)}>
              {step === STEPS.length - 1 ? 'Finish' : 'Next'}
            </Button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
