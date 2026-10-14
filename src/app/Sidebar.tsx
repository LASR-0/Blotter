import type { ReactNode } from 'react';
import {
  Calendar,
  CalendarDots,
  CheckCircle,
  Gear,
  MagnifyingGlass,
  Notebook,
  SidebarSimple,
  SquaresFour,
  Tray,
  type Icon,
} from '@phosphor-icons/react';
import { Kbd } from '@/components/ui/kbd';
import { Tooltip } from '@/components/ui/tooltip';
import { cn } from '@/lib/cn';
import type { View } from '@/lib/period';
import { useOp } from '@/lib/queries';
import type { Blockout, Item } from '@/lib/types';
import { isSidebarCollapsed, useNav } from './nav';
import { useOverlay } from './overlay';

interface NavEntry {
  view: View;
  label: string;
  icon: Icon;
  keys: string;
}

const NAV: readonly NavEntry[] = [
  { view: 'diary', label: 'Diary', icon: Notebook, keys: 'g d' },
  { view: 'week', label: 'Week', icon: Calendar, keys: 'g w' },
  { view: 'month', label: 'Month', icon: CalendarDots, keys: 'g m' },
  { view: 'year', label: 'Year', icon: SquaresFour, keys: 'g y' },
  { view: 'history', label: 'History', icon: CheckCircle, keys: 'g h' },
  { view: 'inbox', label: 'Inbox', icon: Tray, keys: 'i' },
];

const WEEKDAY_ABBR: Record<string, string> = {
  mon: 'M',
  tue: 'T',
  wed: 'W',
  thu: 'T',
  fri: 'F',
  sat: 'S',
  sun: 'S',
};

/** Compact schedule like `M–F 08:30–17` or `23:00–06:30` (every day). */
function blockoutWhen(b: Blockout): string {
  const short = (t: string) => (t.endsWith(':00') ? t.slice(0, 2) : t);
  const times = `${b.start_time}–${short(b.end_time)}`;
  if (b.weekdays.length === 7) return times;
  const weekdays = ['mon', 'tue', 'wed', 'thu', 'fri'];
  const days =
    b.weekdays.length === 5 &&
    weekdays.every((d) => b.weekdays.includes(d as Blockout['weekdays'][0]))
      ? 'M–F'
      : b.weekdays.map((d) => WEEKDAY_ABBR[d]).join('');
  return `${days} ${times}`;
}

function RailButton({
  collapsed,
  active,
  icon: IconCmp,
  label,
  keys,
  onClick,
  trailing,
}: {
  collapsed: boolean;
  active: boolean;
  icon: Icon;
  label: string;
  keys: string;
  onClick: () => void;
  trailing?: ReactNode;
}) {
  const button = (
    <button
      type="button"
      onClick={onClick}
      aria-current={active ? 'page' : undefined}
      aria-label={collapsed ? label : undefined}
      className={cn(
        'flex h-[30px] w-full cursor-pointer items-center gap-2.5 rounded-md text-text hover:bg-hover',
        collapsed ? 'justify-center' : 'px-2',
        active && 'bg-hover font-medium',
      )}
    >
      <IconCmp size={15} className={active ? 'text-accent-text' : 'text-text-muted'} />
      {!collapsed && (
        <>
          <span className="flex-1 text-left">{label}</span>
          {trailing}
          <span className="font-mono text-[10px] text-text-muted opacity-80">{keys}</span>
        </>
      )}
    </button>
  );
  return collapsed ? (
    <Tooltip label={label} shortcut={keys} side="right">
      {button}
    </Tooltip>
  ) : (
    button
  );
}

export function Sidebar() {
  const view = useNav((s) => s.view);
  const go = useNav((s) => s.go);
  const openSettings = useNav((s) => s.openSettings);
  const toggleSidebar = useNav((s) => s.toggleSidebar);
  const collapsed = useNav(isSidebarCollapsed);
  const openOverlay = useOverlay((s) => s.open);
  const inbox = useOp<Item[]>('list_inbox');
  const blockouts = useOp<Blockout[]>('list_blockouts');
  const inboxCount = inbox.data?.filter((i) => i.status === 'open').length ?? 0;

  return (
    <aside
      className={cn(
        'flex min-h-0 flex-none flex-col border-r border-border bg-surface',
        collapsed ? 'w-[52px]' : 'w-[232px]',
      )}
    >
      <div
        className={cn(
          'flex h-12 flex-none items-center gap-2',
          collapsed ? 'justify-center' : 'pr-2.5 pl-3',
        )}
      >
        {!collapsed && (
          <>
            <span className="grid size-6 flex-none place-items-center rounded-md bg-accent text-base font-semibold text-on-accent">
              B
            </span>
            <span className="flex-1 font-heading font-semibold">Blotter</span>
          </>
        )}
        <Tooltip
          label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
          shortcut="Ctrl B"
          side="right"
        >
          <button
            type="button"
            onClick={toggleSidebar}
            aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
            className="grid size-6 flex-none cursor-pointer place-items-center rounded-sm text-text-muted hover:bg-hover hover:text-text"
          >
            <SidebarSimple />
          </button>
        </Tooltip>
      </div>

      <div className="flex-none px-2 pb-2">
        <Tooltip label="Command palette" shortcut="Ctrl K" side="right">
          <button
            type="button"
            onClick={() => openOverlay('palette')}
            aria-label={collapsed ? 'Search or command' : undefined}
            className={cn(
              'flex h-[30px] w-full cursor-pointer items-center gap-2 rounded-md border border-border bg-bg text-text-muted hover:border-border-strong',
              collapsed ? 'justify-center' : 'px-2',
            )}
          >
            <MagnifyingGlass />
            {!collapsed && (
              <>
                <span className="flex-1 text-left">Search or command</span>
                <Kbd>Ctrl K</Kbd>
              </>
            )}
          </button>
        </Tooltip>
      </div>

      <nav aria-label="Views" className="flex flex-none flex-col gap-px px-2">
        {NAV.map((n) => (
          <RailButton
            key={n.view}
            collapsed={collapsed}
            active={view === n.view}
            icon={n.icon}
            label={n.label}
            keys={n.keys}
            onClick={() => go(n.view)}
            trailing={
              n.view === 'inbox' && inboxCount > 0 ? (
                <span className="rounded-full bg-hover px-1.5 font-mono text-xs text-text-muted">
                  {inboxCount}
                </span>
              ) : undefined
            }
          />
        ))}
      </nav>

      <div className="min-h-0 flex-1 overflow-auto px-2 pt-3.5 pb-2">
        {!collapsed && (blockouts.data?.length ?? 0) > 0 && (
          <section className="flex flex-col gap-px">
            <h2 className="px-2 pb-1 text-xs font-normal text-text-muted">Block-outs</h2>
            {blockouts.data?.map((b) => (
              <button
                key={b.id}
                type="button"
                title={`Edit ${b.label}`}
                onClick={() => useNav.getState().openBlockout(b.id)}
                className="flex h-7 cursor-pointer items-center gap-2.5 rounded-md px-2 text-left hover:bg-hover"
              >
                <span className="size-3 rounded-[3px] border border-[color-mix(in_srgb,var(--color-blockout)_50%,transparent)] bg-hatch" />
                <span className="flex-1 truncate">{b.label}</span>
                <span className="font-mono text-[10px] text-text-muted">{blockoutWhen(b)}</span>
              </button>
            ))}
          </section>
        )}
      </div>

      <div className="flex-none border-t border-border p-2">
        <RailButton
          collapsed={collapsed}
          active={view === 'settings'}
          icon={Gear}
          label="Settings"
          keys="Ctrl ,"
          onClick={() => openSettings()}
        />
      </div>
    </aside>
  );
}
