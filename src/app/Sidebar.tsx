import {
  Calendar,
  CalendarDots,
  Check,
  CheckCircle,
  Gear,
  LinkSimple,
  MagnifyingGlass,
  Notebook,
  Plus,
  SidebarSimple,
  SquaresFour,
  Tray,
  type Icon,
} from '@phosphor-icons/react';
import { BlotterMark } from '@/components/BlotterMark';
import { KeyChips } from '@/components/ui/kbd';
import { SlidePanel } from '@/components/ui/slide-panel';
import { Tooltip } from '@/components/ui/tooltip';
import { DEFAULT_CALENDAR_COLOR, useCalendarList } from '@/lib/calendarEvents';
import { cn } from '@/lib/cn';
import type { View } from '@/lib/period';
import { useOp } from '@/lib/queries';
import { tagColor } from '@/lib/tags';
import type { Blockout } from '@/lib/types';
import { useFilters } from './filters';
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
}: {
  collapsed: boolean;
  active: boolean;
  icon: Icon;
  label: string;
  keys: string;
  onClick: () => void;
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
          {/* One key chip per key, as in the palette: [Ctrl] [,], [g] [d]. */}
          <KeyChips binding={keys} compact />
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

/** A tag or calendar filter row: a checkbox in its colour (design: Calendars & tags). */
function FilterRow({
  label,
  color,
  shown,
  ics,
  onToggle,
}: {
  label: string;
  color: string;
  shown: boolean;
  ics?: boolean;
  onToggle: () => void;
}) {
  return (
    <button
      type="button"
      role="checkbox"
      aria-checked={shown}
      onClick={onToggle}
      className={cn(
        'flex h-7 cursor-pointer items-center gap-2.5 rounded-md px-2 text-left hover:bg-hover',
        !shown && 'opacity-60',
      )}
    >
      <span
        className="grid size-[13px] flex-none place-items-center rounded-[4px] border-[1.5px] text-bg"
        style={{ borderColor: color, background: shown ? color : 'transparent' }}
      >
        {shown && <Check size={8} weight="bold" />}
      </span>
      <span className="flex-1 truncate">{label}</span>
      {ics && (
        <Tooltip label="ICS subscription" side="right">
          <LinkSimple className="flex-none text-text-muted" />
        </Tooltip>
      )}
    </button>
  );
}

/** Tags in use and subscribed calendars, each shown or hidden in the Diary and calendar views. */
function CalendarsAndTags() {
  const tags = useOp<{ name: string; items: number }[]>('list_tags');
  const calendars = useCalendarList();
  const { hiddenTags, hiddenCalendars, toggleTag, toggleCalendar } = useFilters();
  const openSettings = useNav((s) => s.openSettings);
  if (!tags.data?.length && !calendars.length) return null;
  return (
    <section aria-label="Calendars and tags" className="flex flex-col gap-px">
      <h2 className="flex items-center px-2 pb-1 text-xs font-normal text-text-muted">
        <span className="flex-1">Calendars &amp; tags</span>
        <Tooltip label="Subscribe to a calendar" side="right">
          <button
            type="button"
            aria-label="Subscribe to a calendar"
            onClick={() => openSettings('calendars')}
            className="grid size-4 cursor-pointer place-items-center rounded-sm hover:bg-hover hover:text-text"
          >
            <Plus size={11} />
          </button>
        </Tooltip>
      </h2>
      {tags.data?.map((t) => (
        <FilterRow
          key={t.name}
          label={t.name.charAt(0).toUpperCase() + t.name.slice(1)}
          color={tagColor(t.name)}
          shown={!hiddenTags.includes(t.name)}
          onToggle={() => toggleTag(t.name)}
        />
      ))}
      {calendars.map((c) => (
        <FilterRow
          key={c.id}
          label={c.name}
          color={c.color ?? DEFAULT_CALENDAR_COLOR}
          shown={!hiddenCalendars.includes(c.id)}
          ics
          onToggle={() => toggleCalendar(c.id)}
        />
      ))}
    </section>
  );
}
export function Sidebar() {
  const view = useNav((s) => s.view);
  const go = useNav((s) => s.go);
  const openSettings = useNav((s) => s.openSettings);
  const toggleSidebar = useNav((s) => s.toggleSidebar);
  const collapsed = useNav(isSidebarCollapsed);
  const openOverlay = useOverlay((s) => s.open);
  const blockouts = useOp<Blockout[]>('list_blockouts');

  return (
    <SlidePanel width={collapsed ? 52 : 232} className="border-r border-border bg-surface">
      <div
        className={cn(
          'flex h-12 flex-none items-center gap-2',
          collapsed ? 'justify-center' : 'pr-2.5 pl-3',
        )}
      >
        {!collapsed && (
          // Baseline lockup: the word sits on the mark's bottom edge.
          <span className="flex flex-1 items-baseline gap-2">
            <BlotterMark className="size-6 flex-none" />
            <span className="wordmark-shadow font-brand text-[25px] leading-none font-brand-weight">
              Blotter
            </span>
          </span>
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
              'flex h-[30px] w-full cursor-pointer items-center gap-1.5 rounded-md border border-border bg-bg text-text-muted hover:border-border-strong',
              collapsed ? 'justify-center' : 'px-2',
            )}
          >
            <MagnifyingGlass />
            {!collapsed && (
              <>
                <span className="min-w-0 flex-1 truncate text-left">Search or command</span>
                <KeyChips binding="Ctrl+K" compact />
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
          />
        ))}
      </nav>

      <div className="flex min-h-0 flex-1 flex-col gap-3.5 overflow-auto px-2 pt-3.5 pb-2">
        {!collapsed && <CalendarsAndTags />}
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
          keys="Ctrl+,"
          onClick={() => openSettings()}
        />
      </div>
    </SlidePanel>
  );
}
