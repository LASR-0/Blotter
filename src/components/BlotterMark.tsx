import { useId } from 'react';
import { cn } from '@/lib/cn';

// The Blotter mark (v1.1): a calendar page whose right edge is a "B", with a
// window cut through it holding a spark and a diagonal sweep across its foot.
// The geometry matches public/blotter.svg and src-tauri/icons/app-icon.svg
// (1024 grid); keep the three in step. The page is a gradient from
// --logo-accent (top left) to --logo-page (bottom right); the rings are
// --logo-page. The window is cut out, so whatever sits behind the mark shows
// through, and the spark is cut the other way: filled inside the window, a
// hole where its tail crosses into the page.
const ACCENT = 'var(--logo-accent, var(--color-accent))';
const PAGE = 'var(--logo-page, var(--color-text))';

/** The page: two bowls on the right. */
const MARK_BODY =
  'M296 282H655C724 282 778 334 778 402C778 453 752 494 712 521C770 546 806 600 806 666' +
  'C806 752 739 821 653 821H296C268 821 246 799 246 771V332C246 304 268 282 296 282Z';
/** The window: its foot is the sweep's edge, rising to the right. */
const MARK_WINDOW =
  'M378 388H648C663 388 674 399 674 414V600L370 756C360 761 352 755 352 744V414C352 399 363 388 378 388Z';
/** The spark, its lower point long enough to cut through the window's foot. */
const MARK_SPARK =
  'M512 420C526 520 526 524 614 540C532 556 540 610 512 712C484 610 492 556 410 540C498 524 498 520 512 420Z';
/** The sweep: the page below the window's foot line, lightened. */
const MARK_SWEEP = 'M246 819L806 532V840H230Z';
const SHAPE = `${MARK_BODY} ${MARK_WINDOW} ${MARK_SPARK}`;

export function BlotterMark({ className }: { className?: string }) {
  const id = useId();
  return (
    // Cropped tight to the artwork (its bottom edge is the drawing's), so a word set on
    // its baseline lines up.
    <svg viewBox="219 206 615 615" aria-hidden className={cn('block', className)}>
      <defs>
        <linearGradient
          id={`${id}g`}
          gradientUnits="userSpaceOnUse"
          x1="246"
          y1="282"
          x2="806"
          y2="821"
        >
          <stop offset="0" style={{ stopColor: ACCENT }} />
          <stop offset="1" style={{ stopColor: PAGE }} />
        </linearGradient>
        <clipPath id={`${id}c`}>
          <path clipRule="evenodd" d={SHAPE} />
        </clipPath>
      </defs>
      {/* Page, window and spark in one even-odd shape: the window is a hole, the
          spark fills inside it, and its tail cuts a notch where it meets the page. */}
      <path fill={`url(#${id}g)`} fillRule="evenodd" d={SHAPE} />
      <path clipPath={`url(#${id}c)`} style={{ fill: PAGE, fillOpacity: 0.22 }} d={MARK_SWEEP} />
      <rect x="356" y="206" width="58" height="134" rx="29" style={{ fill: PAGE }} />
      <rect x="586" y="206" width="58" height="134" rx="29" style={{ fill: PAGE }} />
    </svg>
  );
}
