import { Island } from '@phosphor-icons/react';
import { cn } from '@/lib/cn';
import type { Holiday } from '@/lib/types';

/** A small "on holiday" marker for a day header. */
export function HolidayTag({ holiday, className }: { holiday: Holiday; className?: string }) {
  return (
    <span
      title={`Holiday: ${holiday.label}`}
      className={cn(
        'flex min-w-0 items-center gap-1 truncate rounded-full bg-accent-tint px-1.5 text-[10.5px] text-accent-text',
        className,
      )}
      style={holiday.color ? { color: holiday.color } : undefined}
    >
      <Island className="flex-none" />
      <span className="truncate">{holiday.label}</span>
    </span>
  );
}
