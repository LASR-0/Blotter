import { useState, type ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';

import {
  ArrowCounterClockwise,
  ArrowsClockwise,
  Camera,
  DownloadSimple,
  FileArrowDown,
  FileArrowUp,
  Lightning,
  Sun,
  type Icon,
} from '@phosphor-icons/react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useOverlay } from '@/app/overlay';
import { ErrorPanel } from '@/components/EmptyState';
import { Button } from '@/components/ui/button';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Select } from '@/components/ui/select';
import { isOpError, runOp } from '@/lib/api';
import { fileSize, takenAt } from '@/lib/backupFormat';
import { useOp } from '@/lib/queries';

type Reason =
  | 'manual'
  | 'daily'
  | 'before_restore'
  | 'before_import'
  | 'before_ai_batch'
  | 'before_ics_import'
  | 'before_sync';

interface Snapshot {
  id: string;
  reason: Reason;
  manual: boolean;
  note: string | null;
  path: string;
  size_bytes: number | null;
  created_at: string;
}

interface BackupState {
  dir: string;
  default_dir: boolean;
  snapshots: Snapshot[];
}

interface ImportPreview {
  path: string;
  summary: {
    exported_at: string;
    items: number;
    reminders: number;
    blockouts: number;
    blockout_exceptions: number;
    settings: number;
  };
}

const REASONS: Record<Reason, { label: string; icon: Icon }> = {
  manual: { label: 'Manual', icon: Camera },
  daily: { label: 'Daily', icon: Sun },
  before_restore: { label: 'Before restore', icon: ArrowCounterClockwise },
  before_import: { label: 'Before JSON import', icon: FileArrowUp },
  before_ai_batch: { label: 'Before AI batch', icon: Lightning },
  before_ics_import: { label: 'Before ICS import', icon: DownloadSimple },
  before_sync: { label: 'Before the server’s planner', icon: ArrowsClockwise },
};

const KEEP_CHOICES = [10, 20, 50, 100];
const BACKUPS_KEY = ['backups'];

const errorText = (e: unknown) => (isOpError(e) ? e.message : String(e));

const plural = (n: number, one: string) => `${n} ${one}${n === 1 ? '' : 's'}`;

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

/** "Snapshot now", with an optional note. */
function SnapshotNow({ onTaken }: { onTaken: (s: Snapshot) => void }) {
  const [open, setOpen] = useState(false);
  const [note, setNote] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const take = async () => {
    setBusy(true);
    setError(null);
    try {
      const s = await invoke<Snapshot>('create_snapshot', { note: note.trim() || null });
      setOpen(false);
      setNote('');
      onTaken(s);
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button>
          <Camera />
          Snapshot now
        </Button>
      </PopoverTrigger>
      <PopoverContent align="end" className="flex w-[300px] flex-col gap-2">
        <form
          className="flex flex-col gap-2"
          onSubmit={(e) => {
            e.preventDefault();
            void take();
          }}
        >
          <label className="flex flex-col gap-1.5 text-sm text-text-muted">
            Note (optional)
            <input
              autoFocus
              value={note}
              onChange={(e) => setNote(e.target.value)}
              placeholder="e.g. before rearranging Q4"
              className="h-[30px] rounded-md border border-border-strong bg-bg px-2.5 text-base text-text outline-none focus:border-accent"
            />
          </label>
          <span className="text-xs text-text-muted">Manual snapshots are always kept.</span>
          {error && <span className="text-sm text-danger">{error}</span>}
          <Button type="submit" variant="primary" size="sm" disabled={busy} className="self-end">
            Take snapshot
          </Button>
        </form>
      </PopoverContent>
    </Popover>
  );
}

/** Settings → Backups (design S8): snapshots, restore and JSON. */
export function BackupsPage() {
  const qc = useQueryClient();
  const backups = useQuery({
    queryKey: BACKUPS_KEY,
    queryFn: () => invoke<BackupState>('list_snapshots'),
  });
  const settings = useOp<{ snapshot_keep: number }>('get_settings');
  const ask = useOverlay((s) => s.ask);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);

  const refresh = () => void qc.invalidateQueries({ queryKey: BACKUPS_KEY });
  const fail = (e: unknown) => {
    setStatus(null);
    setError(errorText(e));
  };
  const say = (text: string) => {
    setError(null);
    setStatus(text);
  };

  const restore = (s: Snapshot) =>
    ask({
      title: 'Restore this snapshot?',
      body: `${REASONS[s.reason].label}${s.note ? ` · ${s.note}` : ''}, ${takenAt(s.created_at)}. Everything in Blotter goes back to that moment. Blotter takes a snapshot of the current state first, so you can undo the restore.`,
      confirmLabel: 'Restore',
      danger: true,
      onConfirm: async () => {
        try {
          await invoke('restore_snapshot', { id: s.id });
          // Everything may have changed: start the window afresh.
          window.location.reload();
        } catch (e) {
          fail(e);
        }
      },
    });

  const startImport = async () => {
    try {
      const preview = await invoke<ImportPreview | null>('pick_import');
      if (!preview) return;
      const s = preview.summary;
      ask({
        title: 'Replace everything with this export?',
        body: `It holds ${plural(s.items, 'item')}, ${plural(s.reminders, 'reminder')}, ${plural(s.blockouts, 'block-out')} and ${plural(s.settings, 'setting')}, exported ${takenAt(s.exported_at)}. Everything in Blotter now is replaced, including the undo history. Blotter takes a snapshot first, so you can restore it if you change your mind.`,
        confirmLabel: 'Replace everything',
        danger: true,
        onConfirm: async () => {
          try {
            await invoke('apply_import', { path: preview.path });
            window.location.reload();
          } catch (e) {
            fail(e);
          }
        },
      });
    } catch (e) {
      fail(e);
    }
  };

  if (backups.isError) {
    return (
      <div className="px-8 pt-6">
        <ErrorPanel title="Couldn't load your snapshots" message={errorText(backups.error)} />
      </div>
    );
  }
  const data = backups.data;

  return (
    <div className="flex max-w-[820px] flex-col px-8 pt-6 pb-12">
      <div className="mb-3.5 flex flex-wrap items-center gap-2.5">
        <h1 className="m-0 flex-1 font-heading text-xl font-heading-weight">Backups</h1>
        <Button
          onClick={() =>
            invoke<string | null>('export_json')
              .then((p) => p && say(`Exported to ${p}`))
              .catch(fail)
          }
        >
          <FileArrowDown />
          Export JSON
        </Button>
        <Button onClick={() => void startImport()}>
          <FileArrowUp />
          Import JSON
        </Button>
        <SnapshotNow
          onTaken={() => {
            refresh();
            say('Snapshot taken.');
          }}
        />
      </div>
      {status && <p className="m-0 mb-2 text-sm break-all text-text-muted">{status}</p>}
      {error && <p className="m-0 mb-2 text-sm text-danger">{error}</p>}

      <Row
        label="Retention"
        description="Automatic snapshots beyond this are deleted. Manual snapshots are always kept."
      >
        <Select
          ariaLabel="Snapshots to keep"
          value={String(settings.data?.snapshot_keep ?? 20)}
          onValueChange={(v) =>
            runOp('update_settings', { snapshot_keep: Number(v) })
              .then(refresh)
              .catch(fail)
          }
          options={KEEP_CHOICES.map((n) => ({ value: String(n), label: `Keep last ${n}` }))}
          className="min-w-[150px] text-sm"
        />
      </Row>
      <Row
        label="Location"
        description={<span className="font-mono text-xs break-all">{data?.dir ?? '…'}</span>}
      >
        <div className="flex flex-none gap-1.5">
          {data && !data.default_dir && (
            <Button
              size="sm"
              variant="ghost"
              onClick={() =>
                invoke<string>('reset_snapshot_dir')
                  .then((d) => {
                    refresh();
                    say(`Snapshots moved to ${d}`);
                  })
                  .catch(fail)
              }
            >
              Use default
            </Button>
          )}
          <Button
            size="sm"
            onClick={() =>
              invoke<string | null>('choose_snapshot_dir')
                .then((d) => {
                  if (!d) return;
                  refresh();
                  say(`Snapshots moved to ${d}`);
                })
                .catch(fail)
            }
          >
            Change…
          </Button>
        </div>
      </Row>

      <div className="mt-5 overflow-hidden rounded-lg border border-border">
        <div className="grid grid-cols-[1fr_170px_80px_96px] items-center gap-3 border-b border-border bg-surface px-3.5 py-2 text-xs tracking-[.04em] text-text-muted uppercase">
          <span>Reason</span>
          <span>Taken</span>
          <span>Size</span>
          <span />
        </div>
        {data?.snapshots.length === 0 && (
          <p className="m-0 px-3.5 py-5 text-center text-sm text-text-muted">
            No snapshots yet. The first daily one is taken while Blotter runs; take one now with
            Snapshot now.
          </p>
        )}
        {data?.snapshots.map((s) => {
          const r = REASONS[s.reason];
          return (
            <div
              key={s.id}
              className="grid grid-cols-[1fr_170px_80px_96px] items-center gap-3 border-b border-border px-3.5 py-2 last:border-b-0 hover:bg-hover"
            >
              <span className="flex min-w-0 items-center gap-2">
                <r.icon className="flex-none text-text-muted" />
                <span className="truncate">
                  {r.label}
                  {s.note && <span className="text-text-muted"> · {s.note}</span>}
                </span>
                {s.manual && (
                  <span className="flex-none rounded-full bg-accent-tint px-1.5 text-[10.5px] text-accent-text">
                    kept
                  </span>
                )}
              </span>
              <span className="font-mono text-sm text-text-muted">{takenAt(s.created_at)}</span>
              <span className="font-mono text-sm text-text-muted">{fileSize(s.size_bytes)}</span>
              <Button
                size="sm"
                variant="ghost"
                onClick={() => restore(s)}
                className="justify-self-end"
              >
                Restore…
              </Button>
            </div>
          );
        })}
      </div>
    </div>
  );
}
