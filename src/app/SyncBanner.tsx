import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { ArrowsClockwise, X } from '@phosphor-icons/react';
import { Button } from '@/components/ui/button';
import { replacedText, type SyncNotice } from '@/lib/syncFormat';
import { useNav } from './nav';

/**
 * Tells the user the home server's planner was replaced and fetched again
 * (roadmap §14.8). The app holds the notice until a window takes it, like
 * the rollover banner's.
 */
export function SyncBanner() {
  const [notice, setNotice] = useState<SyncNotice | null>(null);

  useEffect(() => {
    const take = () =>
      invoke<SyncNotice | null>('take_sync_notice')
        .then((n) => n && setNotice(n))
        .catch((e: unknown) => console.error('Could not read the sync notice', e));
    void take();
    const unlisten = listen('sync:notice', () => void take());
    return () => {
      void unlisten.then((stop) => stop());
    };
  }, []);

  if (!notice) return null;
  return (
    <div
      role="status"
      className="fixed bottom-4 left-1/2 z-30 flex w-[640px] max-w-[calc(100vw-32px)] -translate-x-1/2 items-center gap-3 rounded-lg bg-surface-raised px-3.5 py-2.5 text-base text-text shadow-overlay"
    >
      <ArrowsClockwise className="flex-none text-accent-text" />
      <span className="min-w-0 flex-1 leading-normal">{replacedText(notice)}</span>
      <Button
        size="sm"
        onClick={() => {
          useNav.getState().openSettings('backups');
          setNotice(null);
        }}
      >
        Open Backups
      </Button>
      <Button variant="bare" size="icon-sm" aria-label="Dismiss" onClick={() => setNotice(null)}>
        <X />
      </Button>
    </div>
  );
}
