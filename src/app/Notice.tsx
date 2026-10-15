import { useEffect } from 'react';
import { useNotice } from './noticeStore';
import { useRollover } from './rollover';
import { KeyChips } from '@/components/ui/kbd';
import { cn } from '@/lib/cn';

const SHOW_MS = 5000;

/** Bottom-centre status line (undo/redo results and the like). */
export function Notice() {
  const notice = useNotice((s) => s.notice);
  const clear = useNotice((s) => s.clear);
  // Sit above the rollover banner when it's showing; both live at the bottom.
  const raised = useRollover((s) => s.notice !== null);

  useEffect(() => {
    if (!notice) return;
    const t = window.setTimeout(clear, SHOW_MS);
    return () => window.clearTimeout(t);
  }, [notice, clear]);

  return (
    <div
      aria-live="polite"
      className={cn(
        'pointer-events-none fixed left-1/2 z-30 -translate-x-1/2',
        raised ? 'bottom-[88px]' : 'bottom-4',
      )}
    >
      {notice && (
        <div
          key={notice.id}
          className={cn(
            'pointer-events-auto flex max-w-[min(560px,calc(100vw-32px))] items-center gap-3 rounded-lg bg-surface-raised py-2 pr-2 pl-3.5 text-base shadow-overlay',
            notice.tone === 'error' && 'text-danger',
          )}
        >
          <span className="min-w-0 truncate">{notice.text}</span>
          {notice.action && (
            <button
              type="button"
              onClick={() => {
                clear();
                notice.action?.run();
              }}
              className="flex h-[26px] flex-none cursor-pointer items-center gap-1.5 rounded-md px-2 text-sm text-accent-text hover:bg-hover"
            >
              {notice.action.label}
              {notice.action.shortcut && <KeyChips binding={notice.action.shortcut} />}
            </button>
          )}
        </div>
      )}
    </div>
  );
}
