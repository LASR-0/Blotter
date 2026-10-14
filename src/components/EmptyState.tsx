import type { ReactNode } from 'react';
import type { Icon } from '@phosphor-icons/react';
import { cn } from '@/lib/cn';

/** Dashed empty-state panel from the design reference (E1, E4). */
export function EmptyState({
  icon: IconCmp,
  title,
  children,
  className,
}: {
  icon: Icon;
  title: string;
  children?: ReactNode;
  className?: string;
}) {
  return (
    <div
      className={cn(
        'flex flex-col items-start justify-center gap-2.5 rounded-md border border-dashed border-border-strong p-5',
        className,
      )}
    >
      <IconCmp size={22} className="text-text-muted" />
      <span className="font-semibold">{title}</span>
      {children && <div className="leading-normal text-text-muted">{children}</div>}
    </div>
  );
}

/** Error panel from the design reference (E3). */
export function ErrorPanel({
  title,
  message,
  onRetry,
}: {
  title: string;
  message: string;
  onRetry?: () => void;
}) {
  return (
    <div className="flex flex-col gap-2 rounded-md border border-danger bg-[color-mix(in_srgb,var(--color-danger)_6%,var(--color-bg))] p-3.5">
      <span className="font-semibold text-danger">{title}</span>
      <code className="font-mono text-xs break-words text-text-muted">{message}</code>
      {onRetry && (
        <button
          type="button"
          onClick={onRetry}
          className="h-7 cursor-pointer self-start rounded-md border border-border-strong px-2.5 text-sm hover:bg-hover"
        >
          Retry
        </button>
      )}
    </div>
  );
}
