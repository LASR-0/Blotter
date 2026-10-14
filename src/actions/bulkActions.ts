// Actions on the Inbox multi-selection. Each applies as one batch, so a
// bulk change undoes in one step.

import { useOverlay } from '@/app/overlay';
import { useSelection } from '@/app/selection';
import { runBatch } from '@/lib/api';

function selectedIds(): string[] {
  return [...useSelection.getState().ids];
}

/** Sends `update_item` with `patch` for every selected item. */
export async function updateSelection(patch: Record<string, unknown>): Promise<void> {
  const ids = selectedIds();
  if (!ids.length) return;
  await runBatch(ids.map((id) => ({ name: 'update_item', args: { id, ...patch } })));
}

/** Asks, then deletes every selected item in one batch. */
export function deleteSelection(): boolean {
  const ids = selectedIds();
  if (!ids.length) return false;
  useOverlay.getState().ask({
    title: `Delete ${ids.length} ${ids.length === 1 ? 'task' : 'tasks'}?`,
    body: 'Their reminders go with them.',
    confirmLabel: 'Delete',
    danger: true,
    onConfirm: async () => {
      await runBatch(ids.map((id) => ({ name: 'delete_item', args: { id } })));
      useSelection.getState().clear();
    },
  });
  return true;
}
