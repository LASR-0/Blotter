import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { QuickAddPanel } from '@/components/QuickAddPanel';
import { useTheme } from '@/theme/theme';

/**
 * The whole UI of the quick-capture window (design W6): the quick-add panel
 * in capture mode. The window is created hidden at startup and shown by the
 * system-wide shortcut, `blotter --capture` or the tray.
 */
export function CaptureApp() {
  // A new key remounts the panel with an empty line. Esc and creating start
  // fresh; clicking away only hides the window, so a half-typed line survives.
  const [session, setSession] = useState(0);
  const root = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const unlisten = listen('capture:open', () => {
      root.current?.querySelector('input')?.focus();
      // Pick up a theme switched in the main window while this one was hidden.
      useTheme
        .getState()
        .load()
        .catch((e: unknown) => console.error('Could not load theme', e));
    });
    return () => {
      void unlisten.then((stop) => stop());
    };
  }, []);

  // Keep the window exactly as tall as the panel (chips can wrap).
  useEffect(() => {
    const el = root.current;
    if (!el) return;
    const fit = () => void invoke('fit_capture', { height: el.offsetHeight });
    const observer = new ResizeObserver(fit);
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  const finish = () => {
    setSession((s) => s + 1);
    void invoke('hide_capture');
  };

  return (
    <div ref={root}>
      <QuickAddPanel key={session} mode="capture" onClose={finish} onCreated={finish} />
    </div>
  );
}
