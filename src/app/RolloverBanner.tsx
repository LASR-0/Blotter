import { useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { format, parseISO } from 'date-fns';
import { ArrowBendDownRight, X } from '@phosphor-icons/react';
import { Button } from '@/components/ui/button';
import { useNav } from './nav';
import { useRollover, type RolloverResult } from './rollover';
import { clockTime } from '@/lib/format';

const plural = (n: number, one: string, many = `${one}s`) => `${n} ${n === 1 ? one : many}`;

/** "Call bank, Fix gate and 2 more". */
function names(titles: string[]): string {
  if (titles.length <= 2) return titles.join(' and ');
  return `${titles.slice(0, 2).join(', ')} and ${titles.length - 2} more`;
}

function describe(r: RolloverResult): string {
  const parts: string[] = [];
  if (r.reslotted.length) {
    const first = r.reslotted[0];
    const when = first
      ? ` (from ${format(parseISO(first.start), 'EEE')} ${clockTime(parseISO(first.start))})`
      : '';
    parts.push(
      `${plural(r.reslotted.length, 'unfinished task')} re-slotted${when}: ${names(r.reslotted.map((p) => p.item.title))}`,
    );
  }
  if (r.to_inbox.length) {
    const what = r.reslotted.length ? 'didn’t fit and went' : 'from earlier days went';
    parts.push(
      `${plural(r.to_inbox.length, 'unfinished task')} ${what} back to the Inbox: ${names(r.to_inbox.map((i) => i.title))}`,
    );
  }
  return `${parts.join('. ')}.`;
}

/**
 * Tells the user what the end-of-day rollover moved. The app holds the
 * notice until a window takes it, so a rollover at startup still shows.
 */
export function RolloverBanner() {
  const notice = useRollover((s) => s.notice);
  const set = useRollover((s) => s.set);

  useEffect(() => {
    const take = () =>
      invoke<RolloverResult | null>('take_rollover_notice')
        .then((n) => n && useRollover.getState().set(n))
        .catch((e: unknown) => console.error('Could not read the rollover notice', e));
    void take();
    const unlisten = listen('planner:rollover', () => void take());
    return () => {
      void unlisten.then((stop) => stop());
    };
  }, []);

  if (!notice) return null;
  const text = typeof notice === 'string' ? notice : describe(notice);
  const toInbox = typeof notice !== 'string' && notice.to_inbox.length > 0;
  return (
    <div
      role="status"
      className="fixed bottom-4 left-1/2 z-30 flex w-[640px] max-w-[calc(100vw-32px)] -translate-x-1/2 items-center gap-3 rounded-lg bg-surface-raised px-3.5 py-2.5 text-base text-text shadow-overlay"
    >
      <ArrowBendDownRight className="flex-none text-accent-text" />
      <span className="min-w-0 flex-1 leading-normal">{text}</span>
      {toInbox && (
        <Button
          size="sm"
          onClick={() => {
            useNav.getState().go('inbox');
            set(null);
          }}
        >
          Open Inbox
        </Button>
      )}
      <Button variant="bare" size="icon-sm" aria-label="Dismiss" onClick={() => set(null)}>
        <X />
      </Button>
    </div>
  );
}
