import type { ComponentProps } from 'react';
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

/** Renders a binding like `Ctrl+K` or a chord like `g w` as chips. */
export function KeyHint({ keys, size = 'sm' }: { keys: string; size?: 'sm' | 'md' }) {
  const steps = keys.split(' ');
  return (
    <span className="inline-flex items-center gap-1">
      {steps.map((step, i) => (
        <span key={i} className="inline-flex items-center gap-1">
          {i > 0 && <span className="font-mono text-xs text-text-muted">then</span>}
          {size === 'sm' ? (
            <Kbd size="sm">{step.replaceAll('+', ' ')}</Kbd>
          ) : (
            step.split('+').map((k) => (
              <Kbd key={k} size="md">
                {k}
              </Kbd>
            ))
          )}
        </span>
      ))}
    </span>
  );
}
