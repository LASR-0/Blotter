import { Calendar, CalendarDots, CheckCircle, SquaresFour, type Icon } from '@phosphor-icons/react';
import { EmptyState } from '@/components/EmptyState';

const VIEWS: Record<string, { icon: Icon; title: string; body: string }> = {
  week: {
    icon: Calendar,
    title: 'Week view',
    body: 'The time grid with block-outs, conflicts and drag and drop.',
  },
  month: {
    icon: CalendarDots,
    title: 'Month view',
    body: 'A day grid with item chips and block-out indicators.',
  },
  year: {
    icon: SquaresFour,
    title: 'Year view',
    body: 'Twelve mini-months with a scheduled-load heatmap.',
  },
  history: {
    icon: CheckCircle,
    title: 'History',
    body: 'Completed items with timestamps, filters and streaks.',
  },
};

/** Placeholder for views that arrive in later commits. */
export function PendingView({ view }: { view: string }) {
  const v = VIEWS[view];
  if (!v) return null;
  return (
    <div className="grid h-full place-items-center p-8">
      <EmptyState icon={v.icon} title={`${v.title} is on its way`} className="w-[420px]">
        {v.body} Navigation and shortcuts already work; the view itself lands in an upcoming build.
      </EmptyState>
    </div>
  );
}
