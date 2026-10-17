import { useEffect, useState } from 'react';

/** `SlidePanel`'s transition. */
const DURATION_MS = 200;

/**
 * For a `SlidePanel` that mounts and unmounts: `mounted` stays true
 * through the closing slide, and `open` turns true a frame after mounting
 * so the opening slide runs.
 */
export function useSlidePresence(open: boolean): { mounted: boolean; open: boolean } {
  const [mounted, setMounted] = useState(open);
  const [shown, setShown] = useState(open);
  if (open && !mounted) setMounted(true);
  if (!open && shown) setShown(false);

  useEffect(() => {
    if (open) {
      // Two frames: the first paints the panel closed, the second slides it open.
      let frame = requestAnimationFrame(() => {
        frame = requestAnimationFrame(() => setShown(true));
      });
      return () => cancelAnimationFrame(frame);
    }
    const timer = setTimeout(() => setMounted(false), DURATION_MS);
    return () => clearTimeout(timer);
  }, [open]);

  return { mounted, open: shown };
}
