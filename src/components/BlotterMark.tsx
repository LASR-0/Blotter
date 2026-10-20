import { useId } from 'react';
import { cn } from '@/lib/cn';

// The Blotter mark (v1.1): a calendar page whose right edge is a "B", with a
// window cut through it holding a spark and a diagonal sweep across its foot.
// The geometry matches public/blotter.svg (the brand icon, 1024 grid) minus
// its backdrop tile; keep the two in step. Colours come from theme tokens,
// and each part can be overridden with a --logo-* variable (themes set
// --logo-accent, --logo-page and --logo-outline). The window is cut out, so
// whatever sits behind the mark shows through.
const ACCENT = 'var(--logo-accent, var(--color-accent))';
const PAGE = 'var(--logo-page, var(--color-text))';
const FOLD = `var(--logo-fold, color-mix(in srgb, ${ACCENT} 72%, #000))`;
// Light themes give the light parts a thin edge; drawn behind the fill, so the shape stays the same.
const EDGED = {
  fill: PAGE,
  stroke: 'var(--logo-outline, transparent)',
  strokeWidth: 28,
  paintOrder: 'stroke',
  strokeLinejoin: 'round',
} as const;

/** The page: two bowls on the right, with the window cut out. */
const MARK_BODY =
  'M296 282H655C724 282 778 334 778 402C778 453 752 494 712 521C770 546 806 600 806 666' +
  'C806 752 739 821 653 821H296C268 821 246 799 246 771V332C246 304 268 282 296 282Z ' +
  'M478 392H648C663 392 674 403 674 418V598L478 704C463 712 452 704 452 688V418C452 403 463 392 478 392Z';
/** The sweep, clipped to the page: everything below a line rising to the right. */
const MARK_SWEEP = 'M230 814L830 522V840H230Z';
const MARK_SPARK =
  'M563 424C578 505 578 505 659 520C578 535 578 535 563 616C548 535 548 535 467 520C548 505 548 505 563 424Z';

export function BlotterMark({ className }: { className?: string }) {
  const clip = useId();
  return (
    // Cropped tight to the artwork (its bottom edge is the drawing's), so a word set on
    // its baseline lines up; overflow shows the light themes' outline at the edges.
    <svg
      viewBox="219 206 615 615"
      overflow="visible"
      aria-hidden
      className={cn('block', className)}
    >
      <clipPath id={clip}>
        <path clipRule="evenodd" d={MARK_BODY} />
      </clipPath>
      <path style={{ fill: ACCENT }} fillRule="evenodd" d={MARK_BODY} />
      <path style={{ fill: FOLD }} clipPath={`url(#${clip})`} d={MARK_SWEEP} />
      <path style={EDGED} d={MARK_SPARK} />
      <rect x="356" y="206" width="58" height="134" rx="29" style={EDGED} />
      <rect x="586" y="206" width="58" height="134" rx="29" style={EDGED} />
    </svg>
  );
}
