import { isOpError, runOp } from '@/lib/api';
import { useAutoSlot } from './autoSlot';
import { useNotice, type NoticeContent } from './noticeStore';

/** What undo or redo acted on (`ops::undo::BatchInfo`). */
export interface BatchInfo {
  batch_id: string;
  summary: string;
}

const UNDO_KEY = 'Ctrl+Z';
const REDO_KEY = 'Ctrl+Shift+Z';

const show = (n: NoticeContent) => useNotice.getState().show(n);

const failed = (e: unknown) => show({ text: isOpError(e) ? e.message : String(e), tone: 'error' });

/** Undoes the most recent change and says what it was, offering redo. */
export async function undoLast(): Promise<void> {
  try {
    const done = await runOp<BatchInfo | null>('undo');
    // The auto-slot panel describes placements that may no longer stand.
    if (done) useAutoSlot.getState().close();
    show(
      done
        ? {
            text: `Undone: ${done.summary}`,
            action: { label: 'Redo', shortcut: REDO_KEY, run: () => void redoLast() },
          }
        : { text: 'Nothing to undo' },
    );
  } catch (e) {
    failed(e);
  }
}

/** Redoes what the last undo reversed, offering to undo it again. */
export async function redoLast(): Promise<void> {
  try {
    const done = await runOp<BatchInfo | null>('redo');
    if (done) useAutoSlot.getState().close();
    show(
      done
        ? {
            text: `Redone: ${done.summary}`,
            action: { label: 'Undo', shortcut: UNDO_KEY, run: () => void undoLast() },
          }
        : { text: 'Nothing to redo' },
    );
  } catch (e) {
    failed(e);
  }
}
