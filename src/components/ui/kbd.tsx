import type { ComponentProps } from 'react';
import { formatBinding } from '@/keymap/keymap';
import { cn } from '@/lib/cn';

interface KbdProps extends ComponentProps<'kbd'> {
  /** `sm` (10px) in menus and fields; `md` (11px) in palettes and sheets. */
  size?: 'sm' | 'md';
  /** Inverted chip for use on an accent fill. */
  onAccent?: boolean;
}

/** A keyboard key chip, e.g. `n` or `Ctrl` `K`. */
export function Kbd({ size = 'sm', onAccent, className, ...props }: KbdProps) {
  return (
    <kbd
      className={cn(
        'inline-block font-mono whitespace-nowrap',
        onAccent
          ? 'rounded-[3px] bg-[color-mix(in_srgb,var(--color-on-accent)_18%,var(--color-accent))] px-1 text-[10px] text-on-accent'
          : size === 'sm'
            ? 'rounded-[3px] border border-b-2 border-border-strong px-1 text-[10px] leading-[14px] text-text-muted'
            : 'rounded-sm border border-b-2 border-border-strong bg-surface px-1.5 text-xs leading-[18px] text-text',
        className,
      )}
      {...props}
    />
  );
}

/**
 * A binding as one chip per key, as in the palette and shortcut sheet:
 * `Ctrl+K` → [Ctrl] [K], `g w` → [g] [w].
 */
export function KeyChips({ binding, className }: { binding: string; className?: string }) {
  const keys = formatBinding(binding)
    .split(' ')
    .flatMap((step) => step.split(/\+(?!$)/));
  return (
    <span className={cn('inline-flex gap-[3px]', className)}>
      {keys.map((k, i) => (
        <kbd
          key={i}
          className="min-w-[18px] rounded-[3px] border border-b-2 border-border-strong bg-surface px-[5px] text-center font-mono text-[10.5px] text-text-muted"
        >
          {k}
        </kbd>
      ))}
    </span>
  );
}
