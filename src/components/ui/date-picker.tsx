import { useState, type ReactNode } from 'react';
import { DayPicker } from 'react-day-picker';
import { format, isThisYear, parseISO } from 'date-fns';
import { CaretLeft, CaretRight } from '@phosphor-icons/react';
import { cn } from '@/lib/cn';
import { WEEK_STARTS_ON } from '@/lib/period';
import { Popover, PopoverContent, PopoverTrigger } from './popover';

interface DatePickerProps {
  /** `yyyy-MM-dd`, or empty for no date. */
  value: string;
  onChange: (value: string) => void;
  ariaLabel: string;
  placeholder?: string;
  /** Earliest pickable day, `yyyy-MM-dd`. */
  min?: string;
  /** Shows a Clear button (for optional dates). */
  clearable?: boolean;
  /** Leading icon inside the trigger. */
  icon?: ReactNode;
  autoFocus?: boolean;
  className?: string;
}

const ymd = (d: Date) => format(d, 'yyyy-MM-dd');

const FOOT_BUTTON =
  'h-[26px] cursor-pointer rounded-md px-2 text-sm text-text-muted hover:bg-hover hover:text-text focus-visible:outline-2 focus-visible:outline-focus-ring';

/** Date field: a trigger in the app's field style that opens a calendar popover. */
export function DatePicker({
  value,
  onChange,
  ariaLabel,
  placeholder = 'Pick a date',
  min,
  clearable,
  icon,
  autoFocus,
  className,
}: DatePickerProps) {
  const [open, setOpen] = useState(false);
  const selected = value ? parseISO(value) : undefined;
  const pick = (v: string) => {
    onChange(v);
    setOpen(false);
  };
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger
        aria-label={ariaLabel}
        autoFocus={autoFocus}
        className={cn(
          'flex h-[30px] cursor-pointer items-center gap-1.5 rounded-md border border-border-strong bg-bg px-2.5 font-mono text-sm whitespace-nowrap text-text outline-none focus-visible:border-accent data-[state=open]:border-accent',
          className,
        )}
      >
        {icon}
        {selected ? (
          format(selected, isThisYear(selected) ? 'EEE d MMM' : 'EEE d MMM yyyy')
        ) : (
          <span className="text-text-muted">{placeholder}</span>
        )}
      </PopoverTrigger>
      <PopoverContent align="start" className="p-2">
        <DayPicker
          mode="single"
          autoFocus
          showOutsideDays
          weekStartsOn={WEEK_STARTS_ON}
          selected={selected}
          defaultMonth={selected}
          disabled={min ? { before: parseISO(min) } : undefined}
          onSelect={(d) => d && pick(ymd(d))}
          components={{
            Chevron: ({ orientation }) =>
              orientation === 'left' ? <CaretLeft size={14} /> : <CaretRight size={14} />,
          }}
          classNames={{
            root: 'relative select-none',
            months: 'flex',
            month: 'flex flex-col gap-2',
            month_caption: 'flex h-[28px] items-center pl-1.5 font-semibold',
            nav: 'absolute top-0 right-0 flex gap-0.5',
            button_previous:
              'grid size-[28px] cursor-pointer place-items-center rounded-md text-text-muted hover:bg-hover hover:text-text disabled:opacity-40',
            button_next:
              'grid size-[28px] cursor-pointer place-items-center rounded-md text-text-muted hover:bg-hover hover:text-text disabled:opacity-40',
            month_grid: 'border-collapse',
            weekdays: '',
            weekday: 'h-[26px] w-[32px] text-xs font-normal text-text-muted',
            week: '',
            day: 'p-0 text-center',
            day_button:
              'grid size-[32px] cursor-pointer place-items-center rounded-md font-mono text-sm outline-none hover:bg-hover focus-visible:outline-2 focus-visible:outline-focus-ring disabled:cursor-default disabled:hover:bg-transparent',
            today: 'text-accent-text font-semibold',
            selected: '[&>button]:bg-accent [&>button]:text-on-accent [&>button]:hover:bg-accent',
            outside: 'text-text-muted opacity-60',
            disabled: 'opacity-35',
          }}
        />
        <div className="mt-1 flex justify-between border-t border-border pt-1.5">
          <button
            type="button"
            disabled={!!min && ymd(new Date()) < min}
            className={cn(FOOT_BUTTON, 'disabled:pointer-events-none disabled:opacity-40')}
            onClick={() => pick(ymd(new Date()))}
          >
            Today
          </button>
          {clearable && value && (
            <button type="button" className={FOOT_BUTTON} onClick={() => pick('')}>
              Clear
            </button>
          )}
        </div>
      </PopoverContent>
    </Popover>
  );
}
