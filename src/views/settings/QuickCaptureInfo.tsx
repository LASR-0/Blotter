import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Kbd } from '@/components/ui/kbd';

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
  if (!status) return null;

  return (
    <section className="flex flex-col gap-2.5 rounded-lg border border-border px-3.5 py-3">
      <h2 className="m-0 flex items-center gap-2 text-base font-medium">
        Quick capture
        {status.state === 'registered' && <Kbd>{status.shortcut.split('+').join(' ')}</Kbd>}
      </h2>
      <p className="m-0 leading-normal text-text-muted">
        {status.state === 'registered' &&
          'Press the shortcut anywhere to open a small popup that writes straight to Blotter.'}
        {status.state === 'wayland' &&
          "Wayland doesn't let apps register system-wide shortcuts. Bind the command below in your compositor instead; for Hyprland:"}
        {status.state === 'failed' &&
          `${status.shortcut} couldn't be registered (${status.reason}). Another app may be using it; you can bind the command below instead.`}
      </p>
      <code className="self-start rounded-sm bg-surface px-2 py-1 font-mono text-sm select-text">
        {status.state === 'wayland'
          ? `bind = SUPER, SPACE, exec, ${CAPTURE_COMMAND}`
          : CAPTURE_COMMAND}
      </code>
    </section>
  );
}
