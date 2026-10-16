import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { addDays, format } from 'date-fns';
import { DownloadSimple, UploadSimple } from '@phosphor-icons/react';
import { useQueryClient } from '@tanstack/react-query';
import { useOverlay } from '@/app/overlay';
import { Button } from '@/components/ui/button';
import { DatePicker } from '@/components/ui/date-picker';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Segmented } from '@/components/ui/segmented';
import { isOpError } from '@/lib/api';
import {
  exportedText,
  importNotes,
  importSpan,
  importWhat,
  type IcsSummary,
} from '@/lib/icsFormat';

interface ExportResult {
  path: string;
  events: number;
  todos: number;
}

interface IcsPreview {
  path: string;
  summary: IcsSummary;
}

interface Imported {
  summary: IcsSummary;
  snapshot: { id: string } | null;
  batch_id: string | null;
}

const errorText = (e: unknown) => (isOpError(e) ? e.message : String(e));
const ymd = (d: Date) => format(d, 'yyyy-MM-dd');

/** "Export…": everything, or the items in a date range. */
function ExportPopover({
  onDone,
  onError,
}: {
  onDone: (text: string) => void;
  onError: (e: unknown) => void;
}) {
  const [open, setOpen] = useState(false);
  const [scope, setScope] = useState<'all' | 'range'>('all');
  const [from, setFrom] = useState(() => ymd(new Date()));
  const [to, setTo] = useState(() => ymd(addDays(new Date(), 30)));
  const [busy, setBusy] = useState(false);
  const badRange = scope === 'range' && (!from || !to || to < from);

  const run = async () => {
    setBusy(true);
    try {
      const out = await invoke<ExportResult | null>('export_ics', {
        range: scope === 'range' ? { from, to } : null,
      });
      if (out) {
        setOpen(false);
        onDone(exportedText(out.events, out.todos, out.path));
      }
    } catch (e) {
      onError(e);
    } finally {
      setBusy(false);
    }
  };

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button>
          <UploadSimple />
          Export…
        </Button>
      </PopoverTrigger>
      <PopoverContent align="end" className="flex w-[320px] flex-col gap-3">
        <Segmented
          ariaLabel="What to export"
          value={scope}
          onChange={setScope}
          options={[
            { value: 'all', label: 'Everything' },
            { value: 'range', label: 'Date range' },
          ]}
          className="self-start"
        />
        {scope === 'range' && (
          <div className="flex items-center gap-2 text-sm text-text-muted">
            <DatePicker ariaLabel="From" value={from} onChange={setFrom} />
            to
            <DatePicker ariaLabel="To" value={to} onChange={setTo} min={from} />
          </div>
        )}
        <span className="text-xs leading-normal text-text-muted">
          {scope === 'all'
            ? 'Every scheduled item, plus Inbox tasks as to-dos (Google Calendar leaves those out).'
            : 'Items on those days, and any repeating item that falls in them, with all its repeats.'}
        </span>
        <Button
          variant="primary"
          size="sm"
          disabled={busy || badRange}
          onClick={() => void run()}
          className="self-end"
        >
          Export .ics
        </Button>
      </PopoverContent>
    </Popover>
  );
}

/** Settings → Calendars (ICS), design S3: import and export. */
export function CalendarsPage() {
  const qc = useQueryClient();
  const ask = useOverlay((s) => s.ask);
  const [status, setStatus] = useState<string | null>(null);
  const [notes, setNotes] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);

  const fail = (e: unknown) => {
    setStatus(null);
    setNotes([]);
    setError(errorText(e));
  };
  const say = (text: string, details: string[] = []) => {
    setError(null);
    setStatus(text);
    setNotes(details);
  };

  const startImport = async () => {
    try {
      const preview = await invoke<IcsPreview | null>('pick_ics_import');
      if (!preview) return;
      const s = preview.summary;
      const what = importWhat(s);
      const span = importSpan(s);
      const details = importNotes(s);
      if (what === 'nothing new') {
        say('Nothing new to import from that file.', details);
        return;
      }
      ask({
        title: 'Import this calendar?',
        body: [
          `It adds ${what}${span ? ` ${span}` : ''}.`,
          ...details,
          'Blotter takes a snapshot first, and Ctrl+Z undoes the whole import.',
        ].join(' '),
        confirmLabel: 'Import',
        onConfirm: async () => {
          try {
            const done = await invoke<Imported>('apply_ics_import', { path: preview.path });
            say(`Imported ${importWhat(done.summary)}.`, importNotes(done.summary));
            void qc.invalidateQueries({ queryKey: ['backups'] });
          } catch (e) {
            fail(e);
          }
        },
      });
    } catch (e) {
      fail(e);
    }
  };

  return (
    <div className="flex max-w-[820px] flex-col gap-[18px] px-8 pt-6 pb-12">
      <div className="flex flex-wrap items-center gap-2">
        <h1 className="m-0 flex-1 font-heading text-xl font-heading-weight">Calendars (ICS)</h1>
        <Button onClick={() => void startImport()}>
          <DownloadSimple />
          Import .ics
        </Button>
        <ExportPopover onDone={(t) => say(t)} onError={fail} />
      </div>
      <p className="m-0 max-w-[600px] leading-normal text-text-muted">
        Bring in an .ics file from Google Calendar, Apple Calendar or Outlook, or export Blotter for
        them. An import only adds: items it has seen before are left out, and the whole import
        undoes in one step.
      </p>
      {status && (
        <div className="flex flex-col gap-1.5 rounded-lg border border-border px-3.5 py-3">
          <span className="text-sm break-all">{status}</span>
          {notes.map((n) => (
            <span key={n} className="text-sm text-text-muted">
              {n}
            </span>
          ))}
        </div>
      )}
      {error && <p className="m-0 text-sm text-danger">{error}</p>}
    </div>
  );
}
