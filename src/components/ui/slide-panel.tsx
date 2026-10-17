import type { ReactNode } from 'react';
import { cn } from '@/lib/cn';

interface SlidePanelProps {
  /** Width in px; changing it slides the panel open or closed. */
  width: number;
  /** Classes for the panel (border, background, …); it clips what it doesn't show. */
  className?: string;
  label?: string;
  children: ReactNode;
}

/**
 * A side panel whose width animates. Its content keeps the target width
 * from the start (less the 1px border), so it is revealed or covered by
 * the slide instead of reflowing through it.
 */
export function SlidePanel({ width, className, label, children }: SlidePanelProps) {
  return (
    <aside
      aria-label={label}
      style={{ width }}
      className={cn(
        'flex min-h-0 flex-none overflow-hidden transition-[width] duration-200 ease-out motion-reduce:transition-none',
        className,
      )}
    >
      <div style={{ minWidth: Math.max(width - 1, 0) }} className="flex min-h-0 w-full flex-col">
        {children}
      </div>
    </aside>
  );
}
