import type { Conflict, Item } from './types';

/** Conflicts keyed by item id. */
export function conflictMap(conflicts: readonly Conflict[] | undefined): Map<string, Conflict> {
  return new Map((conflicts ?? []).map((c) => [c.item_id, c]));
}

/**
 * One line for a conflict, as on the design's cards: "Overlaps Dentist",
 * "Overlaps Dentist and 2 more", "Inside Sleep", or both joined.
 */
export function conflictText(
  c: Conflict | undefined,
  titleOf: (id: string) => string | undefined,
): string | null {
  if (!c) return null;
  const parts: string[] = [];
  if (c.overlaps.length) {
    const first = titleOf(c.overlaps[0] as string) ?? 'another item';
    const more = c.overlaps.length - 1;
    parts.push(`Overlaps ${first}${more ? ` and ${more} more` : ''}`);
  }
  if (c.blockout) parts.push(`Inside ${c.blockout}`);
  return parts.length ? parts.join(' · ') : null;
}

/** `titleOf` over a list of items. */
export const titles = (items: readonly Item[]) => {
  const byId = new Map(items.map((i) => [i.id, i.title]));
  return (id: string) => byId.get(id);
};
