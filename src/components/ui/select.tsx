import type { ReactNode } from 'react';
import { Select as SelectPrimitive } from 'radix-ui';
import { CaretDown, Check } from '@phosphor-icons/react';
import { cn } from '@/lib/cn';

export interface SelectOption<T extends string> {
  value: T;
  label: ReactNode;
}

interface SelectProps<T extends string> {
  options: readonly SelectOption<T>[];
  /** Empty shows the placeholder. */
  value: T | '';
  onValueChange: (value: T) => void;
  ariaLabel: string;
  placeholder?: string;
  autoFocus?: boolean;
  /** Opens on mount (for "add" flows that swap a button for the select). */
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  className?: string;
}

/** Themed single-select over Radix Select; replaces native `<select>`. */
export function Select<T extends string>({
  options,
  value,
  onValueChange,
  ariaLabel,
  placeholder,
  autoFocus,
  defaultOpen,
  onOpenChange,
  className,
}: SelectProps<T>) {
  return (
    <SelectPrimitive.Root
      value={value || undefined}
      onValueChange={(v) => onValueChange(v as T)}
      defaultOpen={defaultOpen}
      onOpenChange={onOpenChange}
    >
      <SelectPrimitive.Trigger
        aria-label={ariaLabel}
        autoFocus={autoFocus}
        className={cn(
          'flex h-[30px] cursor-pointer items-center justify-between gap-2 rounded-md border border-border-strong bg-bg px-2.5 text-base text-text outline-none focus-visible:border-accent data-[placeholder]:text-text-muted data-[state=open]:border-accent',
          className,
        )}
      >
        <SelectPrimitive.Value placeholder={placeholder} />
        <SelectPrimitive.Icon className="text-text-muted">
          <CaretDown size={12} />
        </SelectPrimitive.Icon>
      </SelectPrimitive.Trigger>
      <SelectPrimitive.Portal>
        <SelectPrimitive.Content
          position="popper"
          sideOffset={4}
          className="z-50 max-h-[var(--radix-select-content-available-height)] min-w-[var(--radix-select-trigger-width)] overflow-hidden rounded-lg bg-surface-raised text-base text-text shadow-overlay"
        >
          <SelectPrimitive.Viewport className="p-1">
            {options.map((o) => (
              <SelectPrimitive.Item
                key={o.value}
                value={o.value}
                className="relative flex h-[28px] cursor-pointer items-center gap-2 rounded-md pr-2.5 pl-7 outline-none select-none data-[highlighted]:bg-hover data-[state=checked]:text-accent-text"
              >
                <SelectPrimitive.ItemIndicator className="absolute left-3">
                  <Check size={12} />
                </SelectPrimitive.ItemIndicator>
                <SelectPrimitive.ItemText>{o.label}</SelectPrimitive.ItemText>
              </SelectPrimitive.Item>
            ))}
          </SelectPrimitive.Viewport>
        </SelectPrimitive.Content>
      </SelectPrimitive.Portal>
    </SelectPrimitive.Root>
  );
}
