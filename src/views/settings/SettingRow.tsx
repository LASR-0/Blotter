import type { ReactNode } from 'react';

/** One setting: its label and description on the left, its control on the right. */
export function SettingRow({
  label,
  description,
  children,
}: {
  label: string;
  description?: ReactNode;
  children: ReactNode;
}) {
  return (
    <div className="flex items-center gap-6 border-b border-border py-3">
      <div className="flex min-w-0 flex-1 flex-col gap-[3px]">
        <span className="font-medium">{label}</span>
        {description && (
          <span className="text-sm leading-[1.45] text-text-muted">{description}</span>
        )}
      </div>
      {children}
    </div>
  );
}
