import { Calendar, CaretLeft, CaretRight, ChatCircleText, Notebook } from '@phosphor-icons/react';
import { Button } from '@/components/ui/button';
import { Segmented } from '@/components/ui/segmented';
import { Tooltip } from '@/components/ui/tooltip';
import { invoke } from '@tauri-apps/api/core';
import { useQuery } from '@tanstack/react-query';
import { PENDING_KEY, useChat } from '@/chat/chatStore';
import { hasPeriod, isCalendarView, periodTitle, type CalendarView } from '@/lib/period';
import { useNav } from './nav';

const CAL_OPTIONS = [
  { value: 'week', label: 'W', tip: ['Week', 'g w'], ariaLabel: 'Week' },
  { value: 'month', label: 'M', tip: ['Month', 'g m'], ariaLabel: 'Month' },
  { value: 'year', label: 'Y', tip: ['Year', 'g y'], ariaLabel: 'Year' },
] as const satisfies readonly {
  value: CalendarView;
  label: string;
  tip: [string, string];
  ariaLabel: string;
}[];

export function TopBar() {
  const { view, calView, anchor, chord, go, step, today, toggleDiaryCalendar } = useNav();
  const { title, sub } = periodTitle(view, anchor);
  const calendar = isCalendarView(view);
  const mode = view === 'diary' ? 'diary' : calendar ? 'calendar' : null;
  const chatOpen = useChat((s) => s.open);
  const setChatOpen = useChat((s) => s.setOpen);
  // Proposed changes waiting for a decision, from the chat or an MCP client.
  const pending =
    useQuery({ queryKey: PENDING_KEY, queryFn: () => invoke<number>('pending_proposals') }).data ??
    0;

  return (
    <header className="flex h-12 flex-none items-center gap-2 border-b border-border pr-3 pl-3.5">
      {hasPeriod(view) && (
        <div className="flex items-center gap-0.5">
          <Tooltip label="Previous" shortcut="[">
            <Button
              variant="bare"
              size="icon"
              className="size-7"
              aria-label="Previous"
              onClick={() => step(-1)}
            >
              <CaretLeft />
            </Button>
          </Tooltip>
          <Tooltip label="Today" shortcut=".">
            <Button size="sm" className="bg-transparent" onClick={today}>
              Today
            </Button>
          </Tooltip>
          <Tooltip label="Next" shortcut="]">
            <Button
              variant="bare"
              size="icon"
              className="size-7"
              aria-label="Next"
              onClick={() => step(1)}
            >
              <CaretRight />
            </Button>
          </Tooltip>
        </div>
      )}
      <h1 className="ml-1 font-heading text-lg font-heading-weight whitespace-nowrap">{title}</h1>
      {sub && <span className="font-mono text-xs whitespace-nowrap text-text-muted">{sub}</span>}
      <span className="flex-1" />

      {chord && (
        <span
          role="status"
          className="rounded-sm bg-accent-tint px-2 py-0.5 font-mono text-xs text-accent-text"
        >
          {chord.prefix.join(' ')} … {chord.continuations.join(' ')}
        </span>
      )}

      <Segmented
        ariaLabel="Diary or Calendar"
        value={mode}
        onChange={(m) =>
          m === 'diary' ? go('diary') : view === 'diary' ? toggleDiaryCalendar() : go(calView)
        }
        options={[
          {
            value: 'diary',
            label: (
              <>
                <Notebook />
                Diary
              </>
            ),
            tip: ['Toggle Diary ↔ Calendar', 'v'],
          },
          {
            value: 'calendar',
            label: (
              <>
                <Calendar />
                Calendar
              </>
            ),
            tip: ['Toggle Diary ↔ Calendar', 'v'],
          },
        ]}
      />
      {calendar && (
        <Segmented
          ariaLabel="Calendar view"
          mono
          value={view}
          onChange={go}
          options={CAL_OPTIONS}
        />
      )}
      <Tooltip label="Toggle AI chat" shortcut="Ctrl+/">
        <Button
          size="icon"
          variant="icon"
          aria-label={pending ? `Toggle AI chat (${pending} changes to review)` : 'Toggle AI chat'}
          data-state={chatOpen ? 'on' : 'off'}
          onClick={() => setChatOpen(!chatOpen)}
          className="relative"
        >
          <ChatCircleText />
          {pending > 0 && (
            <span className="absolute -top-1 -right-1 min-w-4 rounded-full bg-accent px-1 font-mono text-[10px] leading-4 text-on-accent">
              {pending}
            </span>
          )}
        </Button>
      </Tooltip>
    </header>
  );
}
