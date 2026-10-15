import { useState, type ReactNode } from 'react';
import { Popover as PopoverPrimitive } from 'radix-ui';
import { cn } from '@/lib/cn';
import { parseTime } from '@/lib/format';

interface TimeFieldProps {
  /** `HH:mm`, or whatever the user is part-way through typing. */
  value: string;
  onChange: (value: string) => void;
  ariaLabel: string;
  placeholder?: string;
  disabled?: boolean;
  autoFocus?: boolean;
  /** Leading icon inside the field. */
  icon?: ReactNode;
  className?: string;
}

const STEP = 15;
const SLOTS = Array.from({ length: (24 * 60) / STEP }, (_, i) => toHm(i * STEP));

function toHm(min: number): string {
  const m = ((min % 1440) + 1440) % 1440;
  return `${String(Math.floor(m / 60)).padStart(2, '0')}:${String(m % 60).padStart(2, '0')}`;
}

function minutesOf(hm: string): number {
  const [h, m] = hm.split(':').map(Number);
  return (h ?? 0) * 60 + (m ?? 0);
}

/** Scrolls the slot list (not the dialog behind it) to centre the slot nearest `hm`. */
function centre(list: HTMLElement, hm: string | null) {
  const at = hm ? minutesOf(hm) : 9 * 60;
  const row = list.children[Math.min(SLOTS.length - 1, Math.round(at / STEP))] as
    HTMLElement | undefined;
  if (row) list.scrollTop = row.offsetTop - list.clientHeight / 2 + row.offsetHeight / 2;
}

/**
 * Typed time with a 15-minute slot list. Accepts anything `parseTime` reads
 * ("9", "930", "2pm") and tidies it on blur; Up/Down step by 15 minutes.
 */
export function TimeField({
  value,
  onChange,
  ariaLabel,
  placeholder = '--:--',
  disabled,
  autoFocus,
  icon,
  className,
}: TimeFieldProps) {
  const [open, setOpen] = useState(false);
  const parsed = parseTime(value);

  const step = (dir: 1 | -1) => {
    const at = parsed ? minutesOf(parsed) : 9 * 60 - dir * STEP;
    onChange(toHm(Math.round(at / STEP) * STEP + dir * STEP));
  };

  return (
    <PopoverPrimitive.Root open={open && !disabled} onOpenChange={setOpen}>
      <PopoverPrimitive.Anchor asChild>
        <label
          className={cn(
            'flex h-[30px] items-center gap-1.5 rounded-md border border-border-strong bg-bg px-2.5 font-mono text-sm text-text focus-within:border-accent',
            disabled && 'opacity-50',
            className,
          )}
        >
          {icon}
          <input
            value={value}
            disabled={disabled}
            autoFocus={autoFocus}
            aria-label={ariaLabel}
            placeholder={placeholder}
            onChange={(e) => onChange(e.target.value)}
            onClick={() => setOpen(true)}
            onBlur={() => {
              setOpen(false);
              if (parsed && parsed !== value) onChange(parsed);
            }}
            onKeyDown={(e) => {
              if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
                e.preventDefault();
                step(e.key === 'ArrowDown' ? 1 : -1);
              } else if (e.key === 'Escape' && open) {
                e.stopPropagation();
                setOpen(false);
              }
            }}
            className="w-12 min-w-0 bg-transparent outline-none placeholder:text-text-muted"
          />
        </label>
      </PopoverPrimitive.Anchor>
      <PopoverPrimitive.Portal>
        <PopoverPrimitive.Content
          align="start"
          sideOffset={4}
          // Focus stays in the input so typing keeps working.
          onOpenAutoFocus={(e) => e.preventDefault()}
          onCloseAutoFocus={(e) => e.preventDefault()}
          className="z-50 rounded-lg bg-surface-raised p-1 shadow-overlay"
        >
          <div
            // The list mounts in a portal; the callback ref centres it on open and as the value changes.
            ref={(el) => {
              if (el) centre(el, parsed);
            }}
            role="listbox"
            aria-label={ariaLabel}
            className="relative max-h-[220px] overflow-y-auto"
          >
            {SLOTS.map((t) => (
              <div
                key={t}
                role="option"
                aria-selected={t === parsed}
                // Keep focus in the input; mousedown would blur it first.
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => {
                  onChange(t);
                  setOpen(false);
                }}
                className={cn(
                  'cursor-pointer rounded-md px-2.5 py-1 font-mono text-sm hover:bg-hover',
                  t === parsed && 'bg-accent-tint text-accent-text',
                )}
              >
                {t}
              </div>
            ))}
          </div>
        </PopoverPrimitive.Content>
      </PopoverPrimitive.Portal>
    </PopoverPrimitive.Root>
  );
}
