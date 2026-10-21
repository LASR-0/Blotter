import { useState } from 'react';
import { format, parseISO } from 'date-fns';
import { CaretDown, CaretRight, Flag, Trash } from '@phosphor-icons/react';
import { useOverlay } from '@/app/overlay';
import { Button } from '@/components/ui/button';
import { Select } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { isOpError, runOp } from '@/lib/api';
import {
  NATIONAL,
  placeName,
  regionLabel,
  useCountries,
  usePublicHolidayChoice,
  type Imported,
  type PublicConfig,
  type PublicHolidayChoice,
} from '@/lib/publicHolidays';
import { useOp } from '@/lib/queries';
import type { Holiday } from '@/lib/types';

const errorText = (e: unknown) => (isOpError(e) ? e.message : String(e));

/** The country and region selects for a choice. */
export function PublicHolidayFields({ choice }: { choice: PublicHolidayChoice }) {
  return (
    <>
      <Select
        ariaLabel="Country"
        placeholder={choice.countriesLoading ? 'Loading countries…' : 'Country'}
        value={choice.country}
        onValueChange={choice.setCountry}
        options={choice.countries.map((c) => ({ value: c.countryCode, label: c.name }))}
        className="min-w-[200px]"
      />
      {choice.regionList.length > 0 && (
        <Select
          ariaLabel="State or region"
          placeholder="State or region"
          value={choice.regionValue}
          onValueChange={choice.setRegion}
          options={[
            { value: NATIONAL, label: 'National holidays only' },
            ...choice.regionList.map((r) => ({ value: r, label: regionLabel(r) })),
          ]}
          className="min-w-[160px]"
        />
      )}
    </>
  );
}

/** The choice with an Import button, for Settings. */
function PublicHolidayPicker({
  initial,
  onImported,
  onCancel,
}: {
  initial?: Pick<PublicConfig, 'country' | 'region'> | null;
  onImported: (r: Imported) => void;
  onCancel?: () => void;
}) {
  const choice = usePublicHolidayChoice(initial);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const run = async () => {
    setBusy(true);
    setError(null);
    try {
      onImported(await choice.run());
    } catch (e) {
      setError(errorText(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="flex flex-col gap-2">
      <div className="flex flex-wrap items-center gap-1.5">
        <PublicHolidayFields choice={choice} />
        <span className="flex-1" />
        {onCancel && (
          <Button size="sm" variant="ghost" onClick={onCancel}>
            Cancel
          </Button>
        )}
        <Button
          size="sm"
          variant="primary"
          disabled={busy || !choice.ready}
          onClick={() => void run()}
        >
          {busy ? 'Importing…' : 'Import'}
        </Button>
      </div>
      {(error ?? choice.loadError) && (
        <p role="alert" className="m-0 text-sm text-danger">
          {error ?? choice.loadError}
        </p>
      )}
    </div>
  );
}

/** Settings → Block-outs → Holidays → Public holidays. */
export function PublicHolidaysCard({
  holidays,
  lifted,
}: {
  holidays: readonly Holiday[];
  lifted: (id: string) => number;
}) {
  const config = useOp<PublicConfig | null>('get_public_holidays');
  const countries = useCountries();
  const ask = useOverlay((s) => s.ask);
  const [changing, setChanging] = useState(false);
  const [open, setOpen] = useState(false);
  const [note, setNote] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const c = config.data;
  const imported = (r: Imported) => {
    setChanging(false);
    setNote(
      `Imported ${r.added} public ${r.added === 1 ? 'holiday' : 'holidays'} for ${placeName(r.config, countries.data)}.`,
    );
  };

  if (!config.isSuccess) return null;
  return (
    <div className="flex flex-col gap-2.5 rounded-lg border border-border bg-surface p-3.5">
      <div className="flex items-center gap-2.5">
        <Flag size={18} className="flex-none text-accent-text" />
        <span className="flex-1 font-medium">
          Public holidays
          {c && <span className="text-text-muted"> · {placeName(c, countries.data)}</span>}
        </span>
        {c && !changing && (
          <>
            <Button size="sm" onClick={() => setChanging(true)}>
              Change…
            </Button>
            <Button
              variant="bare"
              size="icon-sm"
              aria-label="Remove public holidays"
              title="Remove public holidays"
              onClick={() =>
                ask({
                  title: 'Remove public holidays?',
                  body: 'Every imported public holiday goes, and next year’s won’t be fetched. Your own holidays stay. Ctrl+Z undoes this.',
                  confirmLabel: 'Remove',
                  danger: true,
                  onConfirm: () =>
                    runOp('forget_public_holidays').then(
                      () => setNote(null),
                      (e: unknown) => setError(errorText(e)),
                    ),
                })
              }
            >
              <Trash />
            </Button>
          </>
        )}
      </div>
      {!c && !changing && (
        <p className="m-0 text-sm leading-normal text-text-muted">
          Bring in this year’s and next year’s public holidays for where you are, from
          date.nager.at. They work like your own holidays: work block-outs are lifted and work tasks
          stay off them.
        </p>
      )}
      {(!c || changing) && (
        <PublicHolidayPicker
          initial={c}
          onImported={imported}
          onCancel={changing ? () => setChanging(false) : undefined}
        />
      )}
      {c && !changing && (
        <>
          <label className="flex cursor-pointer items-center gap-2.5 text-sm text-text-muted">
            <Switch
              aria-label="Fetch next year’s public holidays automatically"
              checked={c.keep_updated}
              onCheckedChange={(on) =>
                runOp('set_public_holiday_updates', { keep_updated: on }).catch((e: unknown) =>
                  setError(errorText(e)),
                )
              }
            />
            Fetch each next year’s automatically (in through {c.through_year})
          </label>
          {holidays.length === 0 ? (
            <p className="m-0 text-sm text-text-muted">None coming up.</p>
          ) : (
            <>
              <button
                type="button"
                onClick={() => setOpen(!open)}
                aria-expanded={open}
                className="flex cursor-pointer items-center gap-1.5 self-start text-sm text-text-muted hover:text-text"
              >
                {open ? <CaretDown size={12} /> : <CaretRight size={12} />}
                Next: {holidays[0]?.label},{' '}
                {format(parseISO(holidays[0]?.start_date ?? ''), 'EEE d MMM')}
                {holidays.length > 1 && ` · ${holidays.length - 1} more`}
              </button>
              {open && (
                <ul className="m-0 flex list-none flex-col p-0">
                  {holidays.map((h) => (
                    <li
                      key={h.id}
                      className="flex items-center gap-3 border-t border-border py-1.5 text-sm"
                    >
                      <span className="w-[110px] flex-none font-mono text-xs text-text-muted">
                        {format(parseISO(h.start_date), 'EEE d MMM yyyy')}
                      </span>
                      <span className="min-w-0 flex-1 truncate">{h.label}</span>
                      <span className="text-xs text-text-muted">
                        {lifted(h.id) === 0 ? '' : `Lifts ${lifted(h.id)}`}
                      </span>
                      <Button
                        variant="bare"
                        size="icon-sm"
                        aria-label={`Delete ${h.label}`}
                        title="Delete"
                        onClick={() =>
                          runOp('delete_holiday', { id: h.id }).catch((e: unknown) =>
                            setError(errorText(e)),
                          )
                        }
                      >
                        <Trash />
                      </Button>
                    </li>
                  ))}
                </ul>
              )}
            </>
          )}
        </>
      )}
      {note && <p className="m-0 text-sm text-text-muted">{note}</p>}
      {error && (
        <p role="alert" className="m-0 text-sm text-danger">
          {error}
        </p>
      )}
    </div>
  );
}
