import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import {
  ArrowClockwise,
  CheckCircle,
  CircleDashed,
  PauseCircle,
  Trash,
  XCircle,
} from '@phosphor-icons/react';
import { useOverlay } from '@/app/overlay';
import { Button } from '@/components/ui/button';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Select } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { isOpError, onPlannerChanged } from '@/lib/api';
import { DEFAULT_CALENDAR_COLOR } from '@/lib/calendarEvents';
import { cn } from '@/lib/cn';
import {
  REFRESH_CHOICES,
  refreshLabel,
  subscriptionStatus,
  type Subscription,
} from '@/lib/icsFormat';

const SUBSCRIPTIONS_KEY = ['subscriptions'];

const errorText = (e: unknown) => (isOpError(e) ? e.message : String(e));

const FIELD =
  'h-[30px] min-w-0 flex-1 rounded-md border border-border-strong bg-bg px-2.5 font-mono text-sm text-text outline-none focus:border-accent';

const STATUS_ICON = {
  ok: CheckCircle,
  error: XCircle,
  pending: CircleDashed,
  off: PauseCircle,
} as const;

/** Colours a calendar can take (the tag palette); none uses the theme's calendar colour. */
const CALENDAR_COLORS = ['#6c9cf2', '#56c49b', '#e5995a', '#e77aa9', '#a48ef2'];

function ColorPicker({
  name,
  value,
  onChange,
}: {
  name: string;
  value: string | null;
  onChange: (color: string | null) => void;
}) {
  const [open, setOpen] = useState(false);
  const swatch = (color: string | null, label: string) => (
    <button
      key={label}
      type="button"
      aria-label={label}
      aria-pressed={value === color}
      onClick={() => {
        setOpen(false);
        if (color !== value) onChange(color);
      }}
      style={{ background: color ?? DEFAULT_CALENDAR_COLOR }}
      className={cn(
        'size-5 cursor-pointer rounded-full border-2',
        value === color ? 'border-text' : 'border-transparent',
      )}
    />
  );
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger
        aria-label={`Colour of ${name}`}
        style={{ background: value ?? DEFAULT_CALENDAR_COLOR }}
        className="size-3 flex-none cursor-pointer rounded-full border-0"
      />
      <PopoverContent align="start" className="flex w-auto gap-1.5 p-2">
        {swatch(null, 'Theme colour')}
        {CALENDAR_COLORS.map((c) => swatch(c, c))}
      </PopoverContent>
    </Popover>
  );
}

function SubscriptionCard({ s, onChanged }: { s: Subscription; onChanged: () => void }) {
  const ask = useOverlay((o) => o.ask);
  const [name, setName] = useState(s.name);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const status = subscriptionStatus(s);
  const Icon = STATUS_ICON[status.tone];
  const failed = status.tone === 'error';

  const call = async (command: string, args: Record<string, unknown>) => {
    setBusy(true);
    setError(null);
    try {
      await invoke(command, args);
      onChanged();
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  };
  const patch = (p: Record<string, unknown>) => call('update_subscription', { id: s.id, patch: p });
  const saveName = () => {
    const next = name.trim();
    if (!next) setName(s.name);
    else if (next !== s.name) void patch({ name: next });
  };

  return (
    <div
      className={cn(
        'flex flex-col gap-2 rounded-lg border px-3.5 py-3',
        failed
          ? 'border-danger bg-[color-mix(in_srgb,var(--color-danger)_5%,var(--color-bg))]'
          : 'border-border bg-bg',
      )}
    >
      <div className="flex items-center gap-2">
        <ColorPicker name={s.name} value={s.color} onChange={(color) => void patch({ color })} />
        <input
          value={name}
          onChange={(e) => setName(e.target.value)}
          onBlur={saveName}
          onKeyDown={(e) => {
            if (e.key === 'Enter') e.currentTarget.blur();
            if (e.key === 'Escape') setName(s.name);
          }}
          aria-label="Calendar name"
          className="min-w-0 flex-1 bg-transparent text-md font-semibold outline-none"
        />
        <span
          className={cn(
            'flex items-center gap-1 text-xs',
            failed ? 'text-danger' : 'text-text-muted',
          )}
        >
          <Icon size={13} weight={status.tone === 'pending' ? 'regular' : 'fill'} />
          {busy ? 'Fetching…' : status.text}
        </span>
      </div>
      <span className="truncate font-mono text-xs text-text-muted" title={s.url}>
        {s.url}
      </span>
      {status.tone === 'ok' && s.last_error && (
        <span className="text-xs text-text-muted">{s.last_error}</span>
      )}
      <div className="flex flex-wrap items-center gap-x-4 gap-y-2 text-sm text-text-muted">
        <label className="flex items-center gap-2">
          Refresh
          <Select
            ariaLabel="Refresh every"
            value={String(s.refresh_min)}
            onValueChange={(v) => void patch({ refresh_min: Number(v) })}
            options={[...new Set<number>([...REFRESH_CHOICES, s.refresh_min])]
              .sort((a, b) => a - b)
              .map((m) => ({ value: String(m), label: refreshLabel(m) }))}
            className="min-w-[100px] font-mono text-sm"
          />
        </label>
        <label
          className="flex items-center gap-2"
          title="Busy events keep flexible tasks out, like a block-out. Fixed items may still go there."
        >
          Treat as block-out
          <Switch
            aria-label="Treat as block-out"
            checked={s.as_blockout}
            onCheckedChange={(on) => void patch({ as_blockout: on })}
          />
        </label>
        <span className="flex-1" />
        <span className="text-xs">Read-only</span>
        <Button
          size="sm"
          disabled={busy}
          onClick={() => void call('refresh_subscription', { id: s.id })}
        >
          <ArrowClockwise />
          {failed ? 'Retry' : 'Refresh'}
        </Button>
        <Button
          size="sm"
          variant="ghost"
          aria-label={`Unsubscribe from ${s.name}`}
          onClick={() =>
            ask({
              title: `Unsubscribe from ${s.name}?`,
              body: 'Its events leave Blotter. The calendar itself isn’t touched.',
              confirmLabel: 'Unsubscribe',
              danger: true,
              onConfirm: () => call('remove_subscription', { id: s.id }),
            })
          }
        >
          <Trash />
        </Button>
      </div>
      {error && <span className="text-sm text-danger">{error}</span>}
    </div>
  );
}

/** Calendar subscriptions (design S3): paste a URL, then refresh and block-out options. */
export function SubscriptionsSection() {
  const qc = useQueryClient();
  const subs = useQuery({
    queryKey: SUBSCRIPTIONS_KEY,
    queryFn: () => invoke<Subscription[]>('list_subscriptions'),
  });
  const [url, setUrl] = useState('');
  const [adding, setAdding] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = () => void qc.invalidateQueries({ queryKey: SUBSCRIPTIONS_KEY });
  // The minute check fetches in the background; show what it found.
  useEffect(() => {
    const stop = onPlannerChanged((e) => {
      if (e.ops.includes('subscriptions')) refresh();
    });
    return () => void stop.then((unlisten) => unlisten());
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const subscribe = async () => {
    if (!url.trim()) return;
    setAdding(true);
    setError(null);
    try {
      await invoke('add_subscription', { subscription: { url: url.trim() } });
      setUrl('');
      refresh();
    } catch (e) {
      setError(errorText(e));
    } finally {
      setAdding(false);
    }
  };

  return (
    <section className="flex flex-col gap-3">
      <h2 className="m-0 text-md font-semibold">Subscriptions</h2>
      <p className="m-0 max-w-[600px] leading-normal text-text-muted">
        Show a calendar published as a link, such as your Outlook work calendar. It stays read-only
        and refreshes on its own. Turn on Treat as block-out and its busy times keep flexible tasks
        out.
      </p>
      <form
        className="flex gap-2"
        onSubmit={(e) => {
          e.preventDefault();
          void subscribe();
        }}
      >
        <input
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          placeholder="Paste a calendar URL (webcal:// or https://…ics)"
          aria-label="Calendar URL"
          spellCheck={false}
          className={FIELD}
        />
        <Button type="submit" variant="primary" disabled={adding || !url.trim()}>
          {adding ? 'Subscribing…' : 'Subscribe'}
        </Button>
      </form>
      {error && <p className="m-0 text-sm text-danger">{error}</p>}
      {subs.isError && <p className="m-0 text-sm text-danger">{errorText(subs.error)}</p>}
      {subs.data?.map((s) => (
        // Keyed by name too, so a name the calendar gives itself replaces the field.
        <SubscriptionCard key={`${s.id}:${s.name}`} s={s} onChanged={refresh} />
      ))}
    </section>
  );
}
