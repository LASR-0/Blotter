import { useState, type FormEvent, type ReactNode } from 'react';
import {
  CalendarCheck,
  CalendarPlus,
  Flag,
  MagicWand,
  Sparkle,
  Timer,
  Trash,
  X,
  type Icon,
} from '@phosphor-icons/react';
import { deleteSelection, updateSelection } from '@/actions/bulkActions';
import { runAutoSlot } from '@/app/autoSlot';
import { runSmartSlot } from '@/app/smartSlot';
import { useSelection, type BulkEditor } from '@/app/selection';
import { Button } from '@/components/ui/button';
import { DatePicker } from '@/components/ui/date-picker';
import { Kbd } from '@/components/ui/kbd';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { TimeField } from '@/components/ui/time-field';
import { isOpError } from '@/lib/api';
import { parseDuration } from '@/lib/duration';
import { parseTime } from '@/lib/format';

const FIELD =
  'h-[30px] rounded-md border border-border-strong bg-bg px-2.5 font-mono text-sm text-text outline-none focus:border-accent';

function Editor({
  editor,
  icon: IconCmp,
  label,
  keyHint,
  children,
}: {
  editor: BulkEditor;
  icon: Icon;
  label: string;
  keyHint: string;
  children: ReactNode;
}) {
  const open = useSelection((s) => s.editor === editor);
  const openEditor = useSelection((s) => s.openEditor);
  return (
    <Popover open={open} onOpenChange={(o) => openEditor(o ? editor : null)}>
      <PopoverTrigger asChild>
        <Button variant="ghost" title={`${label} (${keyHint})`}>
          <IconCmp />
          {label}
          <Kbd>{keyHint}</Kbd>
        </Button>
      </PopoverTrigger>
      <PopoverContent side="top" align="start" className="flex w-[260px] flex-col gap-2.5">
        {children}
      </PopoverContent>
    </Popover>
  );
}

/** A small form that applies a patch to the selection and reports errors inline. */
function PatchForm({
  build,
  submitLabel,
  children,
}: {
  build: () => Record<string, unknown> | string;
  submitLabel: string;
  children: ReactNode;
}) {
  const [error, setError] = useState<string | null>(null);
  const openEditor = useSelection((s) => s.openEditor);
  const submit = (e: FormEvent) => {
    e.preventDefault();
    const patch = build();
    if (typeof patch === 'string') {
      setError(patch);
      return;
    }
    updateSelection(patch)
      .then(() => openEditor(null))
      .catch((err: unknown) => setError(isOpError(err) ? err.message : String(err)));
  };
  return (
    <form onSubmit={submit} className="flex flex-col gap-2.5">
      {children}
      {error && <p className="m-0 font-mono text-xs text-danger">{error}</p>}
      <Button type="submit" variant="primary" size="sm" className="self-end">
        {submitLabel}
      </Button>
    </form>
  );
}

function ScheduleForm() {
  const [date, setDate] = useState('');
  const [time, setTime] = useState('');
  return (
    <PatchForm
      submitLabel="Schedule"
      build={() => {
        if (!date) return 'Pick a date.';
        if (!time.trim()) return { date };
        const t = parseTime(time);
        return t ? { start: `${date}T${t}` } : `Can't read "${time}" as a time.`;
      }}
    >
      <span className="text-sm text-text-muted">
        Put the selected tasks on a day, at a time if you like.
      </span>
      <div className="flex gap-1.5">
        <DatePicker autoFocus value={date} onChange={setDate} ariaLabel="Date" className="flex-1" />
        <TimeField value={time} onChange={setTime} ariaLabel="Time" />
      </div>
    </PatchForm>
  );
}

function DurationForm() {
  const [value, setValue] = useState('');
  return (
    <PatchForm
      submitLabel="Set duration"
      build={() => {
        const minutes = parseDuration(value);
        return minutes ? { duration_min: minutes } : `Can't read "${value}"; try 30m or 1h30.`;
      }}
    >
      <input
        autoFocus
        value={value}
        onChange={(e) => setValue(e.target.value)}
        placeholder="30m"
        aria-label="Duration"
        className={FIELD}
      />
    </PatchForm>
  );
}

function PriorityPicker() {
  const openEditor = useSelection((s) => s.openEditor);
  const [error, setError] = useState<string | null>(null);
  const pick = (priority: number) =>
    updateSelection({ priority })
      .then(() => openEditor(null))
      .catch((e: unknown) => setError(isOpError(e) ? e.message : String(e)));
  return (
    <>
      <span className="text-sm text-text-muted">Priority for the selected tasks</span>
      <div className="flex gap-1.5">
        {['None', '!1', '!2', '!3'].map((label, p) => (
          <Button
            key={label}
            autoFocus={p === 0}
            size="sm"
            className="flex-1"
            onClick={() => void pick(p)}
          >
            {label}
          </Button>
        ))}
      </div>
      {error && <p className="m-0 font-mono text-xs text-danger">{error}</p>}
    </>
  );
}

function DueForm() {
  const [date, setDate] = useState('');
  const [time, setTime] = useState('');
  return (
    <PatchForm
      submitLabel={date ? 'Set due date' : 'Clear due date'}
      build={() => {
        if (!date) return { due: null };
        const t = time.trim() ? parseTime(time) : '17:00';
        return t ? { due: `${date}T${t}` } : `Can't read "${time}" as a time.`;
      }}
    >
      <span className="text-sm text-text-muted">Leave the date empty to clear it.</span>
      <div className="flex gap-1.5">
        <DatePicker
          autoFocus
          value={date}
          onChange={setDate}
          ariaLabel="Due date"
          placeholder="No due date"
          clearable
          className="flex-1"
        />
        <TimeField value={time} onChange={setTime} ariaLabel="Due time" placeholder="17:00" />
      </div>
    </PatchForm>
  );
}

/** Floating actions for the Inbox multi-selection (design V3). */
export function BulkBar() {
  const count = useSelection((s) => s.ids.size);
  const clear = useSelection((s) => s.clear);
  if (count === 0) return null;
  return (
    <div
      role="toolbar"
      aria-label="Bulk actions"
      className="absolute bottom-5 left-7 z-10 flex items-center gap-1 rounded-lg bg-surface-raised p-1.5 shadow-overlay"
    >
      <span className="px-2.5 font-medium">{count} selected</span>
      <span className="h-5 w-px bg-border" />
      <Button
        variant="primary"
        size="sm"
        title="Auto-slot selected (s)"
        onClick={() => void runAutoSlot([...useSelection.getState().ids])}
      >
        <MagicWand />
        Auto-slot <Kbd onAccent>s</Kbd>
      </Button>
      <Button
        size="sm"
        title="Smart slot selected with the AI: weighs what each task is, travel and breathing room"
        onClick={() => void runSmartSlot([...useSelection.getState().ids])}
      >
        <Sparkle />
        Smart slot
      </Button>
      <Editor editor="schedule" icon={CalendarPlus} label="Schedule…" keyHint="d">
        <ScheduleForm />
      </Editor>
      <Editor editor="duration" icon={Timer} label="Duration" keyHint="t">
        <DurationForm />
      </Editor>
      <Editor editor="priority" icon={Flag} label="Priority" keyHint="p">
        <PriorityPicker />
      </Editor>
      <Editor editor="due" icon={CalendarCheck} label="Due" keyHint="u">
        <DueForm />
      </Editor>
      <Button
        variant="ghost"
        className="text-danger"
        title="Delete (Del)"
        aria-label="Delete selected"
        onClick={() => deleteSelection()}
      >
        <Trash />
        <Kbd>Del</Kbd>
      </Button>
      <Button
        variant="bare"
        size="icon"
        title="Clear selection (Esc)"
        aria-label="Clear selection"
        onClick={clear}
      >
        <X />
      </Button>
    </div>
  );
}
