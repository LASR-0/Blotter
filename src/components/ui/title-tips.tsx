import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';

const DELAY_MS = 400;
/** Moving to another titled element within this long shows its tip at once. */
const SKIP_DELAY_MS = 300;
const GAP = 6;
const MARGIN = 8;

interface Tip {
  text: string;
  rect: DOMRect;
}

/**
 * Every `title` attribute as a themed tooltip, styled like `Tooltip`,
 * instead of the browser's own. While an element is hovered its title is
 * held back (so the native popup never appears) and put back on leave, so
 * assistive tech still reads it. Mounted once per window.
 */
export function TitleTips() {
  const [tip, setTip] = useState<Tip | null>(null);

  useEffect(() => {
    let target: HTMLElement | null = null;
    let timer: ReturnType<typeof setTimeout> | undefined;
    let lastHidden = 0;

    const restore = () => {
      clearTimeout(timer);
      if (target?.dataset.tip !== undefined) {
        target.setAttribute('title', target.dataset.tip);
        delete target.dataset.tip;
      }
      target = null;
      setTip((t) => {
        if (t) lastHidden = Date.now();
        return null;
      });
    };

    const over = (e: PointerEvent) => {
      // `[data-tip]` is the hovered element, its title held back.
      const el = (e.target as Element | null)?.closest<HTMLElement>('[title], [data-tip]');
      if (el === target || (!el && target?.contains(e.target as Node))) return;
      restore();
      if (!el || el.closest('[inert]')) return;
      const text = el.getAttribute('title') ?? '';
      el.dataset.tip = text;
      el.removeAttribute('title');
      target = el;
      if (!text.trim()) return;
      const show = () => setTip({ text, rect: el.getBoundingClientRect() });
      if (Date.now() - lastHidden < SKIP_DELAY_MS) show();
      else timer = setTimeout(show, DELAY_MS);
    };

    const out = (e: PointerEvent) => {
      if (target && !target.contains(e.relatedTarget as Node | null)) restore();
    };

    document.addEventListener('pointerover', over, true);
    document.addEventListener('pointerout', out, true);
    // Anything that moves or acts on the page hides the tip.
    for (const ev of ['pointerdown', 'keydown', 'wheel', 'blur'] as const)
      window.addEventListener(ev, restore, true);
    return () => {
      restore();
      document.removeEventListener('pointerover', over, true);
      document.removeEventListener('pointerout', out, true);
      for (const ev of ['pointerdown', 'keydown', 'wheel', 'blur'] as const)
        window.removeEventListener(ev, restore, true);
    };
  }, []);

  return tip ? createPortal(<TipBubble tip={tip} />, document.body) : null;
}

/** Below the element, or above it when there's no room; kept inside the window. */
function TipBubble({ tip }: { tip: Tip }) {
  const ref = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState<{ left: number; top: number } | null>(null);

  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    const { width, height } = el.getBoundingClientRect();
    const r = tip.rect;
    const below = r.bottom + GAP + height <= window.innerHeight - MARGIN;
    const top = below ? r.bottom + GAP : r.top - GAP - height;
    const left = Math.min(
      Math.max(r.left + r.width / 2 - width / 2, MARGIN),
      window.innerWidth - MARGIN - width,
    );
    setPos({ left, top: Math.max(top, MARGIN) });
  }, [tip]);

  return (
    <div
      ref={ref}
      role="tooltip"
      style={pos ?? { left: 0, top: 0, visibility: 'hidden' }}
      className="pointer-events-none fixed z-[100] max-w-[320px] rounded-sm bg-text px-[9px] py-[5px] text-sm whitespace-pre-line text-bg"
    >
      {tip.text}
    </div>
  );
}
