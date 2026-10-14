import type { ComponentProps } from 'react';
import { Switch as SwitchPrimitive } from 'radix-ui';
import { cn } from '@/lib/cn';

/** An on/off switch (design: 30×18 track, accent when on). Space toggles it. */
export function Switch({ className, ...props }: ComponentProps<typeof SwitchPrimitive.Root>) {
  return (
    <SwitchPrimitive.Root
      className={cn(
        'flex h-[18px] w-[30px] flex-none cursor-pointer items-center rounded-full border-0 bg-border-strong p-0.5 transition-colors data-[state=checked]:bg-accent',
        className,
      )}
      {...props}
    >
      <SwitchPrimitive.Thumb className="block size-3.5 rounded-full bg-bg transition-transform data-[state=checked]:translate-x-3 data-[state=checked]:bg-on-accent" />
    </SwitchPrimitive.Root>
  );
}
