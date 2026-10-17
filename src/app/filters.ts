import { useMemo } from 'react';
import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { Item } from '@/lib/types';

/**
 * The sidebar's "Calendars & tags" filters: tags and subscribed calendars
 * hidden from the Diary and calendar views. A display choice, so it lives
 * in local storage, outside the change log.
 */
interface FilterState {
  /** Lowercased tag names. */
  hiddenTags: string[];
  /** Subscription ids. */
  hiddenCalendars: string[];
  toggleTag: (tag: string) => void;
  toggleCalendar: (id: string) => void;
}

const toggle = (list: string[], v: string) =>
  list.includes(v) ? list.filter((x) => x !== v) : [...list, v];

export const useFilters = create<FilterState>()(
  persist(
    (set) => ({
      hiddenTags: [],
      hiddenCalendars: [],
      toggleTag: (tag) => set((s) => ({ hiddenTags: toggle(s.hiddenTags, tag.toLowerCase()) })),
      toggleCalendar: (id) => set((s) => ({ hiddenCalendars: toggle(s.hiddenCalendars, id) })),
    }),
    { name: 'blotter.filters' },
  ),
);

/** Shown unless every one of its tags is hidden; untagged items always show. */
export function isItemShown(item: Pick<Item, 'tags'>, hiddenTags: readonly string[]): boolean {
  if (item.tags.length === 0 || hiddenTags.length === 0) return true;
  return item.tags.some((t) => !hiddenTags.includes(t.toLowerCase()));
}

/** `items` without those the tag filters hide. */
export function useShownItems<T extends Pick<Item, 'tags'>>(items: readonly T[] | undefined): T[] {
  const hiddenTags = useFilters((s) => s.hiddenTags);
  return useMemo(
    () => (items ?? []).filter((i) => isItemShown(i, hiddenTags)),
    [items, hiddenTags],
  );
}
