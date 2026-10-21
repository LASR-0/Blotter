import { useEffect, useState, type ReactNode } from 'react';
import { Dialog } from 'radix-ui';
import { invoke } from '@tauri-apps/api/core';
import { LockKey, Power, PushPin } from '@phosphor-icons/react';
import { BlotterMark } from '@/components/BlotterMark';
import { Button } from '@/components/ui/button';
import { Switch } from '@/components/ui/switch';
import { TimeField } from '@/components/ui/time-field';
import { isOpError, runOp } from '@/lib/api';
import { cn } from '@/lib/cn';
import { parseTime } from '@/lib/format';
import { DEFAULT_PLANNING, type WeekStart } from '@/lib/prefs';
import { PRESETS } from '@/theme/presets';
import { useTheme } from '@/theme/theme';
import { usePublicHolidayChoice } from '@/lib/publicHolidays';
import { PublicHolidayFields } from '@/views/settings/PublicHolidays';
import { useOverlay } from './overlay';

const STEPS: readonly [string, string][] = [
  ['Pick a theme', 'You can edit every colour later in Settings → Theme.'],
  ['Your week', 'When does your week start, and when are you willing to do things?'],
  [
    'Block out your work hours',
    'Flexible tasks will never be scheduled inside a block-out. Add more later.',
  ],
  [
    'Public holidays',
    'Imported for where you are, from date.nager.at. Work block-outs are lifted on them and work tasks stay off.',
  ],
  [
    'Connect an AI',
    'Optional. Use Claude Code if you have it, bring your own provider, or a local model through any OpenAI-compatible endpoint.',
  ],
  [
    'Keep reminders running',
    'Closing the window hides Blotter to the tray. Quit from the tray menu.',
  ],
];

const WEEKDAYS = [
  ['mon', 'M', 'Monday'],
  ['tue', 'T', 'Tuesday'],
  ['wed', 'W', 'Wednesday'],
  ['thu', 'T', 'Thursday'],
  ['fri', 'F', 'Friday'],
  ['sat', 'S', 'Saturday'],
  ['sun', 'S', 'Sunday'],
] as const;

type ApiKind = 'anthropic' | 'openai' | 'openai_compatible';
type Provider = 'claude_code' | ApiKind;

const PROVIDERS: Record<ApiKind, { name: string; desc: string; model: string }> = {
  anthropic: { name: 'Anthropic', desc: 'Claude models', model: 'claude-sonnet-5-5' },
  openai: { name: 'OpenAI', desc: 'GPT models', model: '' },
  openai_compatible: { name: 'OpenAI-compatible', desc: 'Ollama, LM Studio…', model: '' },
};
const PROVIDER_NAME: Record<ApiKind, string> = {
  anthropic: 'Anthropic',
  openai: 'OpenAI',
  openai_compatible: 'Ollama',
};

const errorText = (e: unknown) =>
  isOpError(e) ? e.message : e instanceof Error ? e.message : String(e);
const isTime = (t: string) => parseTime(t) === t;
const minutes = (t: string) => {
  const [h, m] = t.split(':').map(Number);
  return (h ?? 0) * 60 + (m ?? 0);
};

interface ClaudeStatus {
  path: string | null;
  logged_in: boolean | null;
}

function Labelled({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="flex flex-col gap-2">
      <span className="text-sm text-text-muted">{label}</span>
      {children}
    </div>
  );
}

function ProviderCard({
  on,
  name,
  desc,
  onPick,
}: {
  on: boolean;
  name: string;
  desc: string;
  onPick: () => void;
}) {
  return (
    <button
      type="button"
      role="radio"
      aria-checked={on}
      onClick={onPick}
      className={cn(
        'flex cursor-pointer flex-col gap-[3px] rounded-md border p-3 text-left text-text',
        on ? 'border-accent bg-accent-tint' : 'border-border bg-transparent',
      )}
    >
      <span className="font-medium">{name}</span>
      <span className="text-xs text-text-muted">{desc}</span>
    </button>
  );
}

/**
 * First-run setup (roadmap §5.10, design: First Run): six short steps,
 * each skippable, all by keyboard. Enter continues, Esc skips the step.
 * Shown once, on a fresh database; "Run setup again" in the palette brings
 * it back.
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
  const [label, setLabel] = useState('Work');
  const [workDays, setWorkDays] = useState<string[]>(['mon', 'tue', 'wed', 'thu', 'fri']);
  const [workStart, setWorkStart] = useState('08:30');
  const [workEnd, setWorkEnd] = useState('17:00');
  const [claude, setClaude] = useState<ClaudeStatus | null>(null);
  const [provider, setProvider] = useState<Provider | null>(null);
  const [secret, setSecret] = useState('');
  const [providerId, setProviderId] = useState<string | null>(null);
  const [tested, setTested] = useState<string | null>(null);
  const [autostart, setAutostart] = useState(true);
  const holidayChoice = usePublicHolidayChoice();

  useEffect(() => {
    invoke<ClaudeStatus>('claude_code_status')
      .then((s) => {
        setClaude(s);
        // Claude Code, when it's ready, is the tested route (D29).
        setProvider((p) => p ?? (s.logged_in ? 'claude_code' : 'anthropic'));
      })
      .catch(() => setProvider((p) => p ?? 'anthropic'));
  }, []);

  const finish = async () => {
    try {
      await invoke('finish_first_run');
    } catch (e) {
      console.error('Could not record the setup as done', e);
    }
    close();
  };

  /** Saves (or updates) the chosen API provider with the typed key or URL. */
  const saveProvider = async (kind: ApiKind): Promise<string> => {
    const compat = kind === 'openai_compatible';
    const name = PROVIDER_NAME[kind];
    const saved = await invoke<{ id: string; model: string }>('save_provider', {
      input: {
        id: providerId,
        name,
        kind,
        base_url: compat ? secret.trim() || 'http://localhost:11434/v1' : null,
        model: PROVIDERS[kind].model || null,
        api_key: compat ? null : secret.trim(),
        make_default: true,
      },
    });
    setProviderId(saved.id);
    if (!saved.model) {
      // OpenAI and local endpoints: take the first model they list.
      const models = await invoke<{ id: string }[]>('fetch_models', { id: saved.id });
      const first = models[0]?.id;
      if (first) {
        await invoke('save_provider', { input: { id: saved.id, name, kind, model: first } });
      }
    }
    await invoke('set_ai_settings', { input: { route: 'api' } });
    return saved.id;
  };

  const test = async (kind: ApiKind) => {
    setError(null);
    setTested(null);
    setBusy(true);
    try {
      const id = await saveProvider(kind);
      const r = await invoke<{ ms: number }>('test_provider', { id });
      setTested(`Connected · ${r.ms} ms`);
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
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
        if (step === 2) {
          if (!isTime(workStart) || !isTime(workEnd) || workDays.length === 0)
            throw new Error('Pick the days and hours you work, or skip this step.');
          await runOp('create_blockout', {
            label: label.trim() || 'Work',
            weekdays: workDays,
            start_time: workStart,
            end_time: workEnd,
            icon: 'briefcase',
          });
        }
        if (step === 3 && holidayChoice.ready) await holidayChoice.run();
        if (step === 4) {
          if (provider === 'claude_code') {
            await invoke('set_ai_settings', { input: { route: 'claude_code' } });
          } else if (provider && secret.trim() && !tested) {
            await saveProvider(provider);
          }
        }
        if (step === 5) await invoke('set_autostart', { enabled: autostart });
      }
      if (step < STEPS.length - 1) {
        setStep(step + 1);
        return;
      }
      await finish();
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  };

  const [title, sub] = STEPS[step] as [string, string];
  const planLeft = (minutes(planStart) / (24 * 60)) * 100;
  const planRight = 100 - (minutes(planEnd) / (24 * 60)) * 100;
  const claudeFound = !!claude?.path;

  const body = [
    <div key="theme" role="radiogroup" aria-label="Theme" className="grid grid-cols-4 gap-2.5">
      {PRESETS.map((t) => {
        const on = theme.id === t.id;
        const v = t.vars;
        return (
          <button
            key={t.id}
            type="button"
            role="radio"
            aria-checked={on}
            onClick={() => setTheme(t.id)}
            className={cn(
              'flex cursor-pointer flex-col gap-2 rounded-md border bg-transparent p-2 text-left text-text',
              on ? 'border-accent outline-2 outline-offset-2 outline-focus-ring' : 'border-border',
            )}
          >
            <span
              className="flex h-[84px] flex-col gap-1 rounded-[6px] border p-2"
              style={{ background: v['--color-bg'], borderColor: v['--color-border'] }}
            >
              <span
                className="h-1.5 w-3/5 rounded-[2px]"
                style={{ background: v['--color-text'] }}
              />
              <span
                className="h-3.5 rounded-[3px]"
                style={{
                  background: v['--color-surface'],
                  borderLeft: `3px solid ${v['--color-accent']}`,
                }}
              />
              <span
                className="h-3.5 rounded-[3px] border border-dashed opacity-50"
                style={{ borderColor: v['--color-text'] }}
              />
            </span>
            <span className="font-medium">{t.label}</span>
          </button>
        );
      })}
    </div>,

    <div key="week" className="flex flex-col gap-4">
      <Labelled label="Week starts on">
        <div role="radiogroup" aria-label="Week starts on" className="flex gap-1.5">
          {(['mon', 'sun', 'sat'] as const).map((d) => (
            <button
              key={d}
              type="button"
              role="radio"
              aria-checked={weekStart === d}
              onClick={() => setWeekStart(d)}
              className={cn(
                'flex h-8 cursor-pointer items-center rounded-md border px-3.5 text-text',
                weekStart === d ? 'border-accent bg-accent-tint' : 'border-border-strong',
              )}
            >
              {{ mon: 'Monday', sun: 'Sunday', sat: 'Saturday' }[d]}
            </button>
          ))}
        </div>
      </Labelled>
      <Labelled label="Planning hours — flexible tasks are only placed in this window">
        <span className="flex items-center gap-2.5">
          <TimeField
            ariaLabel="Planning hours start"
            value={planStart}
            onChange={setPlanStart}
            className="w-[92px]"
          />
          <span className="text-text-muted">to</span>
          <TimeField
            ariaLabel="Planning hours end"
            value={planEnd}
            onChange={setPlanEnd}
            className="w-[92px]"
          />
        </span>
        <div
          aria-hidden
          className="relative h-[18px] rounded-[4px] border border-border bg-surface"
        >
          {planStart < planEnd && (
            <span
              className="absolute inset-y-0 border-x-2 border-accent bg-accent-tint"
              style={{ left: `${planLeft}%`, right: `${planRight}%` }}
            />
          )}
          <span className="absolute top-px left-1 font-mono text-[10px] text-text-muted">00</span>
          <span className="absolute top-px right-1 font-mono text-[10px] text-text-muted">24</span>
        </div>
      </Labelled>
    </div>,

    <div
      key="work"
      className="flex flex-col gap-3.5 rounded-md border border-border bg-[repeating-linear-gradient(135deg,color-mix(in_srgb,var(--color-blockout)_14%,transparent)_0_1.5px,transparent_1.5px_7px)] p-3.5"
    >
      <label className="flex items-center gap-2.5">
        <span className="w-[60px] text-sm text-text-muted">Label</span>
        <input
          value={label}
          onChange={(e) => setLabel(e.target.value)}
          aria-label="Block-out label"
          className="h-[30px] min-w-[160px] rounded-md border border-border-strong bg-surface-raised px-2.5 text-text outline-none focus:border-accent"
        />
      </label>
      <div className="flex items-center gap-2.5">
        <span className="w-[60px] text-sm text-text-muted">Days</span>
        <span className="flex gap-1">
          {WEEKDAYS.map(([d, l, name]) => {
            const on = workDays.includes(d);
            return (
              <button
                key={d}
                type="button"
                aria-pressed={on}
                aria-label={name}
                onClick={() => setWorkDays(on ? workDays.filter((x) => x !== d) : [...workDays, d])}
                className={cn(
                  'grid size-[30px] cursor-pointer place-items-center rounded-md border text-xs font-medium',
                  on
                    ? 'border-accent bg-accent-tint text-accent-text'
                    : 'border-border-strong bg-surface-raised text-text-muted',
                )}
              >
                {l}
              </button>
            );
          })}
        </span>
      </div>
      <div className="flex items-center gap-2.5">
        <span className="w-[60px] text-sm text-text-muted">Hours</span>
        <span className="flex items-center gap-2">
          <TimeField
            ariaLabel="Work start"
            value={workStart}
            onChange={setWorkStart}
            className="w-[92px]"
          />
          <span className="text-text-muted">–</span>
          <TimeField
            ariaLabel="Work end"
            value={workEnd}
            onChange={setWorkEnd}
            className="w-[92px]"
          />
        </span>
      </div>
      <span className="flex items-center gap-1.5 text-sm text-text-muted">
        <PushPin weight="fill" />
        Fixed appointments, like a telehealth consult, can still go here.
      </span>
    </div>,

    <div key="holidays" className="flex flex-col gap-3">
      <Labelled label="Where you are (guessed from your time zone)">
        <span className="flex flex-wrap items-center gap-1.5">
          <PublicHolidayFields choice={holidayChoice} />
        </span>
      </Labelled>
      {holidayChoice.loadError && (
        <p className="m-0 text-sm text-text-muted">
          {holidayChoice.loadError} Skip this step and import them later in Settings → Block-outs.
        </p>
      )}
    </div>,

    <div key="ai" className="flex flex-col gap-3">
      <div
        role="radiogroup"
        aria-label="AI provider"
        className={cn('grid gap-2', claudeFound ? 'grid-cols-4' : 'grid-cols-3')}
      >
        {claudeFound && (
          <ProviderCard
            on={provider === 'claude_code'}
            name="Claude Code"
            desc={claude?.logged_in ? 'Signed in · your subscription' : 'Installed, not signed in'}
            onPick={() => setProvider('claude_code')}
          />
        )}
        {(Object.keys(PROVIDERS) as ApiKind[]).map((k) => (
          <ProviderCard
            key={k}
            on={provider === k}
            name={PROVIDERS[k].name}
            desc={PROVIDERS[k].desc}
            onPick={() => {
              setProvider(k);
              setProviderId(null);
              setTested(null);
              setSecret('');
            }}
          />
        ))}
      </div>
      {provider === 'claude_code' ? (
        <p className="m-0 text-sm leading-normal text-text-muted">
          {claude?.logged_in
            ? 'The chat will use Claude Code on this computer. Nothing else to set up.'
            : 'Run "claude" in a terminal to sign in, then the chat can use it.'}
        </p>
      ) : (
        provider && (
          <div className="flex gap-2">
            <input
              type={provider === 'openai_compatible' ? 'text' : 'password'}
              value={secret}
              onChange={(e) => {
                setSecret(e.target.value);
                setTested(null);
              }}
              placeholder={
                provider === 'openai_compatible' ? 'http://localhost:11434/v1' : 'Paste API key'
              }
              aria-label={provider === 'openai_compatible' ? 'Endpoint URL' : 'API key'}
              autoComplete="off"
              spellCheck={false}
              className="h-8 min-w-0 flex-1 rounded-md border border-border-strong bg-bg px-2.5 font-mono text-sm text-text outline-none focus:border-accent"
            />
            <Button
              disabled={busy || (provider !== 'openai_compatible' && !secret.trim())}
              onClick={() => void test(provider)}
            >
              Test
            </Button>
          </div>
        )
      )}
      {tested && <span className="text-sm text-text-muted">{tested}</span>}
      <span className="flex items-center gap-1.5 text-sm text-text-muted">
        <LockKey />
        Keys go to your OS keychain. AI changes start in Propose mode.
      </span>
    </div>,

    <div key="autostart" className="flex items-center gap-3 rounded-md border border-border p-3.5">
      <Power size={20} className="flex-none text-text-muted" />
      <span className="flex min-w-0 flex-1 flex-col gap-0.5">
        <span className="font-medium">Launch Blotter at login</span>
        <span className="text-sm text-text-muted">
          Blotter starts in the tray, so reminders fire even if you never open the window.
        </span>
      </span>
      <Switch aria-label="Launch at login" checked={autostart} onCheckedChange={setAutostart} />
    </div>,
  ];

  return (
    <Dialog.Root open onOpenChange={(o) => !o && void finish()}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-40 bg-[color-mix(in_srgb,var(--color-bg)_80%,transparent)]" />
        <Dialog.Content
          aria-describedby="setup-sub"
          onEscapeKeyDown={(e) => {
            // Esc skips the step, as the design's hint says.
            e.preventDefault();
            if (!busy) void next(false);
          }}
          onOpenAutoFocus={(e) => {
            // Start on the step's own controls.
            e.preventDefault();
            (e.currentTarget as HTMLElement | null)
              ?.querySelector<HTMLElement>('[data-step] button, [data-step] input')
              ?.focus();
          }}
          onKeyDown={(e) => {
            // Enter continues, unless a control that uses Enter has focus.
            const t = e.target as HTMLElement;
            if (
              e.key === 'Enter' &&
              t.tagName !== 'BUTTON' &&
              t.getAttribute('role') !== 'combobox'
            ) {
              e.preventDefault();
              if (!busy) void next(true);
            }
          }}
          className="fixed top-[80px] left-1/2 z-41 flex w-[620px] max-w-[calc(100vw-32px)] -translate-x-1/2 flex-col rounded-lg bg-surface-raised text-base text-text shadow-overlay focus:outline-none"
        >
          <div className="flex items-center gap-2.5 px-[22px] pt-4">
            <BlotterMark className="size-6 flex-none" />
            <span className="font-medium">Set up Blotter</span>
            <span className="flex-1" />
            <div aria-hidden className="flex gap-1">
              {STEPS.map(([t], i) => (
                <span
                  key={t}
                  className={cn(
                    'h-1.5 rounded-[3px]',
                    i === step ? 'w-[18px]' : 'w-1.5',
                    i <= step ? 'bg-accent' : 'bg-border-strong',
                  )}
                />
              ))}
            </div>
            <span className="font-mono text-xs text-text-muted">
              {step + 1} / {STEPS.length}
            </span>
          </div>
          <div className="flex flex-col gap-1.5 px-[22px] pt-[22px] pb-2">
            <Dialog.Title className="m-0 font-heading text-[24px] font-heading-weight tracking-[-.01em]">
              {title}
            </Dialog.Title>
            <span id="setup-sub" className="max-w-[480px] leading-normal text-text-muted">
              {sub}
            </span>
          </div>
          <div data-step className="min-h-[210px] px-[22px] pt-3.5 pb-[22px]">
            {body[step]}
            {error && <p className="m-0 mt-3 text-sm text-danger">{error}</p>}
          </div>
          <div className="flex items-center gap-2 border-t border-border px-[22px] py-3.5">
            <Button
              variant="ghost"
              disabled={busy || step === 0}
              onClick={() => setStep(step - 1)}
              className="disabled:opacity-40"
            >
              Back
            </Button>
            <button
              type="button"
              onClick={() => void finish()}
              className="cursor-pointer text-sm text-text-muted hover:text-text hover:underline"
            >
              Set up later
            </button>
            <span className="flex-1" />
            <Button disabled={busy} onClick={() => void next(false)}>
              Skip
              <span className="font-mono text-[10px] text-text-muted">Esc</span>
            </Button>
            <Button variant="primary" disabled={busy} onClick={() => void next(true)}>
              {step === STEPS.length - 1 ? 'Finish' : 'Continue'}
              <span className="font-mono text-[10px] opacity-75">↵</span>
            </Button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
