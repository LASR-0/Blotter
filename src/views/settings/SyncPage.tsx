import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import {
  ArrowDown,
  ArrowsClockwise,
  ArrowUp,
  CheckCircle,
  CircleDashed,
  CloudSlash,
  Trash,
  WarningCircle,
  XCircle,
  type Icon,
} from '@phosphor-icons/react';
import { useOverlay } from '@/app/overlay';
import { ErrorPanel } from '@/components/EmptyState';
import { Button } from '@/components/ui/button';
import { isOpError } from '@/lib/api';
import { cn } from '@/lib/cn';
import {
  moveAddress,
  needsPairing,
  routeLabel,
  syncSummary,
  type AddressCheck,
  type Disconnected,
  type SyncInfo,
  type SyncStatus,
  type SyncTone,
} from '@/lib/syncFormat';
import { SettingRow } from './SettingRow';
import { FIELD, JoinChoice, PairForm } from './SyncPairing';

const INFO_KEY = ['sync-info'];
const STATUS_KEY = ['sync-status'];

const errorText = (e: unknown) => (isOpError(e) ? e.message : String(e));

const TONE: Record<SyncTone, { icon: Icon; className: string }> = {
  off: { icon: CloudSlash, className: 'text-text-muted' },
  busy: { icon: ArrowsClockwise, className: 'text-text-muted' },
  ok: { icon: CheckCircle, className: 'text-accent' },
  offline: { icon: CircleDashed, className: 'text-warning' },
  attention: { icon: WarningCircle, className: 'text-danger' },
};

/** Re-renders every 30 s, so "2 min ago" keeps up. */
function useNow(): Date {
  const [now, setNow] = useState(() => new Date());
  useEffect(() => {
    const t = window.setInterval(() => setNow(new Date()), 30_000);
    return () => window.clearInterval(t);
  }, []);
  return now;
}

function StatusCard({ status, now }: { status: SyncStatus; now: Date }) {
  const summary = syncSummary(status, now);
  const { icon: ToneIcon, className } = TONE[summary.tone];
  const version = status.state === 'synced' ? status.server_version : null;
  const tried = status.state === 'offline' ? status.tried : [];
  return (
    <div
      role="status"
      className={cn(
        'flex flex-col gap-1.5 rounded-lg border px-3.5 py-3',
        summary.tone === 'attention'
          ? 'border-danger bg-[color-mix(in_srgb,var(--color-danger)_5%,var(--color-bg))]'
          : 'border-border bg-bg',
      )}
    >
      <span className="flex items-center gap-2 font-medium">
        <ToneIcon size={16} weight="fill" className={cn('flex-none', className)} />
        {summary.text}
        <span className="flex-1" />
        {version && (
          <span className="font-mono text-xs font-normal text-text-muted">
            Blotter Sync {version}
          </span>
        )}
      </span>
      {summary.detail && <span className="text-sm text-text-muted">{summary.detail}</span>}
      {tried.length > 0 && (
        <ul className="m-0 flex list-none flex-col gap-0.5 p-0 font-mono text-xs text-text-muted">
          {tried.map(([a, why]) => (
            <li key={a}>
              {a} — {why}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

/** The addresses tried in order: reorder, test, remove, add. */
function AddressList({ addresses, onSaved }: { addresses: string[]; onSaved: () => void }) {
  const [adding, setAdding] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [checks, setChecks] = useState<Record<string, AddressCheck | 'testing'>>({});

  const save = async (next: string[]) => {
    setError(null);
    try {
      await invoke('sync_set_addresses', { addresses: next });
      onSaved();
      return true;
    } catch (e) {
      setError(errorText(e));
      return false;
    }
  };
  const test = async (address: string) => {
    setChecks((c) => ({ ...c, [address]: 'testing' }));
    try {
      const check = await invoke<AddressCheck>('sync_test_address', { address });
      setChecks((c) => ({ ...c, [address]: check }));
    } catch (e) {
      setChecks((c) => ({ ...c, [address]: { address, ok: false, message: errorText(e) } }));
    }
  };

  return (
    <section className="flex flex-col gap-2 border-b border-border py-3">
      <div className="flex flex-col gap-[3px]">
        <span className="font-medium">Addresses</span>
        <span className="text-sm leading-[1.45] text-text-muted">
          Tried in this order, for example your network first, then Tailscale, then a domain.
          Earlier ones are checked again every few minutes, so a laptop back home returns to the
          faster one.
        </span>
      </div>
      <ol className="m-0 flex list-none flex-col gap-1.5 p-0">
        {addresses.map((a, i) => {
          const check = checks[a];
          return (
            <li
              key={a}
              className="flex flex-col gap-1 rounded-md border border-border px-2.5 py-1.5"
            >
              <div className="flex items-center gap-2">
                <span className="w-4 text-right font-mono text-xs text-text-muted">{i + 1}</span>
                <span className="min-w-0 flex-1 truncate font-mono text-sm" title={a}>
                  {a}
                </span>
                <span className="text-xs text-text-muted">{routeLabel(a)}</span>
                <Button
                  size="sm"
                  variant="ghost"
                  disabled={check === 'testing'}
                  onClick={() => void test(a)}
                >
                  {check === 'testing' ? 'Testing…' : 'Test'}
                </Button>
                <Button
                  size="icon-sm"
                  variant="bare"
                  aria-label={`Move ${a} up`}
                  disabled={i === 0}
                  onClick={() => void save(moveAddress(addresses, i, -1))}
                >
                  <ArrowUp />
                </Button>
                <Button
                  size="icon-sm"
                  variant="bare"
                  aria-label={`Move ${a} down`}
                  disabled={i === addresses.length - 1}
                  onClick={() => void save(moveAddress(addresses, i, 1))}
                >
                  <ArrowDown />
                </Button>
                <Button
                  size="icon-sm"
                  variant="bare"
                  aria-label={`Remove ${a}`}
                  title={
                    addresses.length === 1 ? 'Disconnect instead to remove the last one' : undefined
                  }
                  disabled={addresses.length === 1}
                  onClick={() => void save(addresses.filter((x) => x !== a))}
                >
                  <Trash />
                </Button>
              </div>
              {check && check !== 'testing' && (
                <span
                  className={cn(
                    'flex items-center gap-1 pl-6 text-xs',
                    check.ok ? 'text-text-muted' : 'text-danger',
                  )}
                >
                  {check.ok ? (
                    <CheckCircle weight="fill" className="text-accent" />
                  ) : (
                    <XCircle weight="fill" />
                  )}
                  {check.message}
                </span>
              )}
            </li>
          );
        })}
      </ol>
      <form
        className="flex gap-2"
        onSubmit={(e) => {
          e.preventDefault();
          if (!adding.trim()) return;
          void save([...addresses, adding]).then((ok) => ok && setAdding(''));
        }}
      >
        <input
          value={adding}
          onChange={(e) => setAdding(e.target.value)}
          placeholder="Add an address (https://blotter.your-tailnet.ts.net)"
          aria-label="New server address"
          spellCheck={false}
          className={cn(FIELD, 'flex-1 font-mono')}
        />
        <Button type="submit" disabled={!adding.trim()}>
          Add
        </Button>
      </form>
      {error && <p className="m-0 text-sm text-danger">{error}</p>}
    </section>
  );
}

/** Settings → Sync (roadmap §14.9): Local only, or a paired home server. */
export function SyncPage() {
  const qc = useQueryClient();
  const ask = useOverlay((o) => o.ask);
  const now = useNow();
  const info = useQuery({ queryKey: INFO_KEY, queryFn: () => invoke<SyncInfo>('sync_info') });
  const status = useQuery({
    queryKey: STATUS_KEY,
    queryFn: () => invoke<SyncStatus | null>('sync_status'),
  });
  const [leftListed, setLeftListed] = useState(false);
  const [leaveError, setLeaveError] = useState<string | null>(null);

  const refresh = () => {
    void qc.invalidateQueries({ queryKey: INFO_KEY });
    void qc.invalidateQueries({ queryKey: STATUS_KEY });
  };
  useEffect(() => {
    const stop = listen('sync:status', () => void qc.invalidateQueries({ queryKey: STATUS_KEY }));
    return () => void stop.then((unlisten) => unlisten());
  }, [qc]);

  const disconnect = async () => {
    setLeaveError(null);
    try {
      const done = await invoke<Disconnected>('sync_disconnect');
      setLeftListed(!done.revoked);
      refresh();
    } catch (e) {
      setLeaveError(errorText(e));
    }
  };

  const header = (
    <>
      <h1 className="m-0 mb-1.5 font-heading text-xl font-heading-weight">Sync</h1>
      <p className="m-0 mb-3.5 max-w-[600px] leading-normal text-text-muted">
        Keep the same planner on every Blotter desktop through Blotter Sync, running on a home
        server. Without one, Blotter is Local only and your planner never leaves this device.
      </p>
    </>
  );

  if (info.isError)
    return (
      <div className="flex max-w-[720px] flex-col px-8 pt-6 pb-12">
        {header}
        <ErrorPanel
          title="Couldn't load sync settings"
          message={errorText(info.error)}
          onRetry={refresh}
        />
      </div>
    );
  if (!info.data)
    return (
      <div className="flex max-w-[720px] flex-col px-8 pt-6 pb-12">
        {header}
        <p className="m-0 text-text-muted">Loading…</p>
      </div>
    );

  const { server, device_name, joining } = info.data;
  const s = status.data ?? null;

  return (
    <div className="flex max-w-[720px] flex-col px-8 pt-6 pb-12">
      {header}
      <SettingRow
        label="Mode"
        description={
          server
            ? 'Home server: changes sync in the background, and every change waits here while the server is out of reach.'
            : 'Local only: nothing is sent anywhere.'
        }
      >
        <span className="rounded-full border border-border-strong px-2.5 py-0.5 text-sm">
          {server ? 'Home server' : 'Local only'}
        </span>
      </SettingRow>

      {leftListed && !server && (
        <p className="m-0 mt-3 rounded-md border border-warning px-3 py-2 text-sm leading-[1.45]">
          Disconnected. The server couldn’t be reached, so it still lists this device: revoke it
          there with <code className="font-mono">blotter-sync revoke</code>.
        </p>
      )}

      {!server && joining ? (
        <div className="mt-3">
          <JoinChoice address={joining} onDone={refresh} />
        </div>
      ) : !server ? (
        <div className="mt-3">
          <PairForm
            name={device_name}
            onPaired={() => {
              setLeftListed(false);
              refresh();
            }}
          />
        </div>
      ) : (
        <>
          <div className="mt-3 flex flex-col gap-3">
            {s && <StatusCard status={s} now={now} />}
            {needsPairing(s) && (
              <PairForm again address={server.addresses[0]} name={device_name} onPaired={refresh} />
            )}
          </div>
          <AddressList addresses={server.addresses} onSaved={refresh} />
          <SettingRow
            label="This device"
            description="Other devices’ History shows this name. It’s set when pairing."
          >
            <span className="text-sm">{device_name ?? '—'}</span>
          </SettingRow>
          <SettingRow
            label="Disconnect"
            description="Stop syncing and go back to Local only. This device keeps its planner, and the server forgets this device."
          >
            <Button
              variant="danger"
              onClick={() =>
                ask({
                  title: 'Disconnect from the home server?',
                  body: 'This device stops syncing and keeps its planner as it is now. The other devices carry on.',
                  confirmLabel: 'Disconnect',
                  danger: true,
                  onConfirm: disconnect,
                })
              }
            >
              Disconnect
            </Button>
          </SettingRow>
          {leaveError && <p className="m-0 mt-2 text-sm text-danger">{leaveError}</p>}
        </>
      )}
    </div>
  );
}
