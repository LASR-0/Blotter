import type { Item } from './types';

/** Tags with a dedicated theme token (--tag-<name>). */
const THEMED_TAGS = ['work', 'health', 'home', 'shopping', 'personal'] as const;

/**
 * CSS colour for a tag: its theme token if it has one, otherwise a stable
 * pick from the tag palette so the same tag always gets the same colour.
 */
export function tagColor(tag: string): string {
  const t = tag.toLowerCase();
  if ((THEMED_TAGS as readonly string[]).includes(t)) return `var(--tag-${t})`;
  let hash = 0;
  for (const ch of t) hash = (hash * 31 + ch.charCodeAt(0)) >>> 0;
  return `var(--tag-${THEMED_TAGS[hash % THEMED_TAGS.length]})`;
}

/** An item's colour: explicit colour, else its first tag, else the accent. */
export function itemColor(item: Pick<Item, 'color' | 'tags'>): string {
  if (item.color) return item.color;
  const first = item.tags[0];
  return first ? tagColor(first) : 'var(--color-accent)';
}
