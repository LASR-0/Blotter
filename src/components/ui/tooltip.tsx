import type { ComponentProps, ReactNode } from 'react';
import { Tooltip as TooltipPrimitive } from 'radix-ui';
import { cn } from '@/lib/cn';

export function TooltipProvider(props: ComponentProps<typeof TooltipPrimitive.Provider>) {
  return <TooltipPrimitive.Provider delayDuration={400} skipDelayDuration={200} {...props} />;
}

interface TooltipProps {
  /** Label text. */
  label: ReactNode;
  /** Optional shortcut shown after the label in mono, e.g. `Ctrl /`. */
  shortcut?: string;
  side?: ComponentProps<typeof TooltipPrimitive.Content>['side'];
  children: ReactNode;
}

/** Inverted tooltip (text colour on ground colour) with an optional shortcut. */
export function Tooltip({ label, shortcut, side = 'bottom', children }: TooltipProps) {
  return (
    <TooltipPrimitive.Root>
      <TooltipPrimitive.Trigger asChild>{children}</TooltipPrimitive.Trigger>
      <TooltipPrimitive.Portal>
        <TooltipPrimitive.Content
          side={side}
          sideOffset={6}
          className={cn(
            'z-50 flex items-center gap-2 rounded-sm bg-text px-[9px] py-[5px] text-sm whitespace-nowrap text-bg',
          )}
        >
          {label}
          {shortcut && <span className="font-mono opacity-75">{shortcut}</span>}
        </TooltipPrimitive.Content>
      </TooltipPrimitive.Portal>
    </TooltipPrimitive.Root>
  );
}
