// Actions on the keyboard-focused item, shared by every navigable view.

import { format } from 'date-fns';
import { deleteSelection } from '@/actions/bulkActions';
import { useFocus } from '@/app/focus';
import { useNav } from '@/app/nav';
import { useOverlay } from '@/app/overlay';
import { runOp } from '@/lib/api';
import { isRecurring, type OpCallSpec } from '@/lib/itemDraft';
import { isCalendarView } from '@/lib/period';
import type { Item } from '@/lib/types';

/** Surfaces failures until toasts land; nothing is lost because the op didn't apply. */
function report(e: unknown) {
  console.error('Blotter action failed', e);
}

export async function fetchItem(id: string): Promise<Item> {
  return runOp<Item>('get_item', { id });
}

/** Runs a single built call, if any. */
export function runSpec(spec: OpCallSpec | null): void {
  if (spec) runOp(spec.name, spec.args).catch(report);
}

/** Calls on the focused item still running, the oldest first. */
let queue: Promise<void> = Promise.resolve();
let queued = 0;

/**
 * Runs `build` against the focused item (fetched fresh) and applies the result.
 * Calls run one after another, so each builds on the last; with one already
 * waiting, further calls are dropped, so a held key can't pile up moves.
 */
export function withFocusedItem(build: (item: Item) => OpCallSpec | null): void {
  const id = useFocus.getState().itemId;
  if (!id || queued >= 2) return;
  queued += 1;
  queue = queue
    .then(async () => {
      const spec = build(await fetchItem(id));
      if (spec) await runOp(spec.name, spec.args);
    })
    .catch(report)
    .finally(() => {
      queued -= 1;
    });
}

export function toggleFocused(): void {
  withFocusedItem((item) => ({
    name: item.status === 'done' ? 'uncomplete_item' : 'complete_item',
    args: { id: item.id },
  }));
}

export function openFocused(): boolean {
  const id = useFocus.getState().itemId;
  if (!id) return false;
  useOverlay.getState().openItem({ id });
  return true;
}

/**
 * Opens the item dialog for a new item. In the calendar views it starts on
 * the focused day; in the Diary and Inbox it has no date, so a task made
 * there goes to the Inbox.
 */
export function openNewItem(): void {
  const day = isCalendarView(useNav.getState().view)
    ? useFocus.getState().controller?.day?.()
    : null;
  useOverlay.getState().openItem({ defaults: day ? { date: format(day, 'yyyy-MM-dd') } : {} });
}

export function deleteFocused(): void {
  // In the Inbox, Del acts on the multi-selection when there is one.
  if (deleteSelection()) return;
  const id = useFocus.getState().itemId;
  if (!id) return;
  fetchItem(id)
    .then((item) =>
      useOverlay.getState().ask({
        title: isRecurring(item)
          ? `Delete this occurrence of "${item.title}"?`
          : `Delete "${item.title}"?`,
        body: isRecurring(item)
          ? 'The rest of the series stays. Open it (Enter) to delete more occurrences.'
          : 'Its reminders go with it.',
        confirmLabel: isRecurring(item) ? 'Delete occurrence' : 'Delete',
        danger: true,
        onConfirm: async () => {
          await runOp('delete_item', { id: item.id });
          useFocus.getState().setItem(null);
        },
      }),
    )
    .catch(report);
}
