import { cn } from '@/lib/cn';

// The Blotter mark: a calendar page whose right edge forms a "B". The
// geometry matches public/blotter.svg (the brand icon, 1024 grid) minus its
// backdrop tile; keep the two in step. Colours come from theme tokens, and
// each part can be overridden with a --logo-* variable (themes set
// --logo-accent, --logo-page and --logo-outline). The ring holes are cut out, so whatever
// sits behind the mark shows through.
const ACCENT = 'var(--logo-accent, var(--color-accent))';
const PAGE = 'var(--logo-page, var(--color-text))';
const FOLD = `var(--logo-fold, color-mix(in srgb, ${ACCENT} 72%, #000))`;
const CELL = `var(--logo-cell, color-mix(in srgb, ${ACCENT} 32%, ${PAGE}))`;
const DONE = 'var(--logo-done, var(--color-tag-health))';
// Light themes give the page a thin edge; drawn behind the fill, so the shape stays the same.
const EDGED = {
  fill: PAGE,
  stroke: 'var(--logo-outline, transparent)',
  strokeWidth: 40,
  paintOrder: 'stroke',
  strokeLinejoin: 'round',
} as const;

export function BlotterMark({ className }: { className?: string }) {
  return (
    // Cropped tight to the artwork (its bottom edge is the drawing's), so a word set on
    // its baseline lines up; overflow shows the light themes' outline at the edges.
    <svg
      viewBox="210 184 628 628"
      overflow="visible"
      aria-hidden
      className={cn('block', className)}
    >
      <path
        style={{ fill: ACCENT }}
        fillRule="evenodd"
        d="M210 336a90 90 0 0 1 90-90h395a90 90 0 0 1 90 90v84a90 90 0 0 1-90 90H210Z M320 293a40 40 0 1 0 80 0a40 40 0 1 0-80 0Z M585 293a40 40 0 1 0 80 0a40 40 0 1 0-80 0Z"
      />
      <path
        style={{ fill: ACCENT }}
        d="M556 812V640l191-72c55 14 91 64 91 124c0 66-54 120-120 120Z"
      />
      <path
        style={EDGED}
        d="M210 396h487c49 0 88 40 88 89c0 32-14 62-38 83L586 716c-19 17-30 41-30 66v30H290a80 80 0 0 1-80-80Z"
      />
      <path
        style={{ fill: FOLD }}
        d="M747 568C705 660 650 760 580 806c-7 4-15 6-24 6v-30c0-25 11-49 30-66Z"
      />
      <rect x="331" y="190" width="58" height="125" rx="29" style={EDGED} />
      <rect x="596" y="190" width="58" height="125" rx="29" style={EDGED} />
      <rect x="266" y="448" width="116" height="114" rx="22" style={{ fill: CELL }} />
      <rect x="414" y="448" width="116" height="114" rx="22" style={{ fill: ACCENT }} />
      <rect x="562" y="448" width="116" height="114" rx="22" style={{ fill: CELL }} />
      <rect x="266" y="596" width="116" height="114" rx="22" style={{ fill: CELL }} />
      <rect x="414" y="596" width="116" height="114" rx="22" style={{ fill: DONE }} />
    </svg>
  );
}
