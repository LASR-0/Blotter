import type { ComponentProps } from 'react';
import { Popover as PopoverPrimitive } from 'radix-ui';
import { cn } from '@/lib/cn';

export const Popover = PopoverPrimitive.Root;
export const PopoverTrigger = PopoverPrimitive.Trigger;
export const PopoverClose = PopoverPrimitive.Close;

/** Raised overlay panel (design: radius-lg, surface-raised, overlay shadow). */
export function PopoverContent({
  className,
  sideOffset = 6,
  ...props
}: ComponentProps<typeof PopoverPrimitive.Content>) {
  return (
    <PopoverPrimitive.Portal>
      <PopoverPrimitive.Content
        sideOffset={sideOffset}
        className={cn(
          'z-50 rounded-lg bg-surface-raised p-3 text-base text-text shadow-overlay focus:outline-none',
          className,
        )}
        {...props}
      />
    </PopoverPrimitive.Portal>
  );
}
