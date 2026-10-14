import type { ComponentProps } from 'react';
import { Slot } from 'radix-ui';
import { cva, type VariantProps } from 'class-variance-authority';
import { cn } from '@/lib/cn';

// Matches the design reference component sheet (§7): primary / secondary /
// ghost / danger, and a 30px icon button that doubles as a toggle.
const buttonVariants = cva(
  'inline-flex shrink-0 cursor-pointer items-center justify-center gap-1.5 rounded-md whitespace-nowrap select-none disabled:pointer-events-none disabled:opacity-40 [&_svg]:shrink-0',
  {
    variants: {
      variant: {
        primary:
          'bg-accent font-medium text-on-accent hover:bg-[color-mix(in_srgb,var(--color-accent)_88%,var(--color-text))] active:bg-[color-mix(in_srgb,var(--color-accent)_80%,var(--color-bg))]',
        secondary:
          'border border-border-strong bg-surface-raised text-text hover:bg-hover active:bg-border-strong',
        ghost: 'text-text hover:bg-hover active:bg-border',
        danger:
          'bg-danger font-medium text-on-accent hover:bg-[color-mix(in_srgb,var(--color-danger)_88%,var(--color-text))]',
        icon: 'border border-border text-text-muted hover:bg-hover hover:text-text data-[state=on]:border-accent data-[state=on]:bg-accent-tint data-[state=on]:text-accent-text',
        bare: 'text-text-muted hover:bg-hover hover:text-text',
      },
      size: {
        md: 'h-[30px] px-3 text-base',
        sm: 'h-[26px] px-2.5 text-sm',
        icon: 'size-[30px] text-[15px]',
        'icon-sm': 'size-6 rounded-sm text-sm',
      },
    },
    defaultVariants: { variant: 'secondary', size: 'md' },
  },
);

export interface ButtonProps extends ComponentProps<'button'>, VariantProps<typeof buttonVariants> {
  /** Render the child element instead of a <button>, keeping the styles. */
  asChild?: boolean;
}

export function Button({ className, variant, size, asChild, type, ...props }: ButtonProps) {
  const Comp = asChild ? Slot.Root : 'button';
  return (
    <Comp
      type={asChild ? undefined : (type ?? 'button')}
      className={cn(buttonVariants({ variant, size }), className)}
      {...props}
    />
  );
}

// eslint-disable-next-line react-refresh/only-export-components
export { buttonVariants };
