import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Info } from '@phosphor-icons/react';

/** Mirrors `ShortcutStatus` in src-tauri/src/capture.rs. */
type ShortcutStatus =
  | { state: 'registered'; shortcut: string }
  | { state: 'wayland' }
  | { state: 'failed'; shortcut: string; reason: string };

const CAPTURE_COMMAND = 'blotter --capture';

/**
 * Whether the system-wide capture shortcut works here. On Wayland it can't,
 * so this says how to bind the CLI flag instead of showing a dead shortcut.
 * Shown on Settings → Keybindings.
 */
export function QuickCaptureInfo() {
  const [status, setStatus] = useState<ShortcutStatus | null>(null);
  useEffect(() => {
    invoke<ShortcutStatus>('capture_shortcut_status')
      .then(setStatus)
      .catch(() => setStatus(null));
  }, []);
  // Shown only when the shortcut can't work, as in the design.
  if (!status || status.state === 'registered') return null;

  return (
    <div className="flex gap-2.5 rounded-md border border-warning bg-[color-mix(in_srgb,var(--color-warning)_7%,var(--color-bg))] px-3 py-2.5 text-sm leading-normal">
      <Info size={15} className="mt-px flex-none text-warning" />
      <span>
        {status.state === 'wayland' ? (
          <>
            Wayland session detected. Global shortcuts can&rsquo;t be registered here, so bind{' '}
            <code className="rounded-[3px] bg-surface-raised px-1 font-mono select-text">
              {CAPTURE_COMMAND}
            </code>{' '}
            in your compositor instead, e.g.{' '}
            <code className="rounded-[3px] bg-surface-raised px-1 font-mono select-text">
              bind = SUPER, SPACE, exec, {CAPTURE_COMMAND}
            </code>
            .
          </>
        ) : (
          <>
            {status.shortcut} couldn&rsquo;t be registered ({status.reason}). Another app may be
            using it; bind{' '}
            <code className="rounded-[3px] bg-surface-raised px-1 font-mono select-text">
              {CAPTURE_COMMAND}
            </code>{' '}
            to a shortcut of your own instead.
          </>
        )}
      </span>
    </div>
  );
}
