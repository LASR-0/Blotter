import type { ReactNode } from 'react';
import { cn } from '@/lib/cn';
import { Tooltip } from './tooltip';

export interface SegmentOption<T extends string> {
  value: T;
  label: ReactNode;
  /** Tooltip label and shortcut, e.g. `['Week', 'g w']`. */
  tip?: [string, string?];
  /** Accessible name when `label` is an icon or abbreviation. */
  ariaLabel?: string;
}

interface SegmentedProps<T extends string> {
  options: readonly SegmentOption<T>[];
  value: T | null;
  onChange: (value: T) => void;
  ariaLabel: string;
  mono?: boolean;
  className?: string;
}

/** The design's segmented switch (Diary | Calendar, W M Y). */
export function Segmented<T extends string>({
  options,
  value,
  onChange,
  ariaLabel,
  mono,
  className,
}: SegmentedProps<T>) {
  return (
    <div
      role="radiogroup"
      aria-label={ariaLabel}
      className={cn('flex rounded-md border border-border bg-surface p-0.5', className)}
    >
      {options.map((o) => {
        const on = o.value === value;
        const button = (
          <button
            key={o.value}
            type="button"
            role="radio"
            aria-checked={on}
            aria-label={o.ariaLabel}
            onClick={() => onChange(o.value)}
            className={cn(
              'flex h-6 cursor-pointer items-center gap-1.5 rounded-sm text-sm',
              mono ? 'px-[9px] font-mono' : 'px-2.5',
              on
                ? 'bg-surface-raised font-medium text-text shadow-[0_0_0_1px_var(--color-border-strong)]'
                : 'text-text-muted hover:text-text',
            )}
          >
            {o.label}
          </button>
        );
        return o.tip ? (
          <Tooltip key={o.value} label={o.tip[0]} shortcut={o.tip[1]}>
            {button}
          </Tooltip>
        ) : (
          button
        );
      })}
    </div>
  );
}
