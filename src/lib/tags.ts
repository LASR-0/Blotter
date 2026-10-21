import type { Item } from './types';

/** Tags with a dedicated theme token (--tag-<name>). */
const THEMED_TAGS = ['work', 'health', 'home', 'shopping', 'personal'] as const;

/** Colours offered for a tag: the theme's tag tokens, so they follow the theme. */
export const TAG_SWATCHES = [...THEMED_TAGS, 'ics'].map((t) => `var(--tag-${t})`);

/** A tag as listed by `list_tags`. */
export interface TagInfo {
  name: string;
  items: number;
  /** Chosen colour, or null for the automatic one. */
  color: string | null;
  /** Its tasks can be done at the same time as other tasks. */
  shares_time: boolean;
}

/** The CSS variable carrying a tag's chosen colour (set by TagColors). */
export function tagVar(tag: string): string {
  const t = tag.toLowerCase();
  // Custom property names are identifiers; anything unusual falls back to a hash.
  if (/^[a-z0-9_-]+$/.test(t)) return `--tagc-${t}`;
  let hash = 0;
  for (const ch of t) hash = (hash * 31 + (ch.codePointAt(0) ?? 0)) >>> 0;
  return `--tagc-u${hash.toString(36)}`;
}

/** A tag's automatic colour: its theme token if it has one, else a stable pick. */
export function autoTagColor(tag: string): string {
  const t = tag.toLowerCase();
  if ((THEMED_TAGS as readonly string[]).includes(t)) return `var(--tag-${t})`;
  let hash = 0;
  for (const ch of t) hash = (hash * 31 + ch.charCodeAt(0)) >>> 0;
  return `var(--tag-${THEMED_TAGS[hash % THEMED_TAGS.length]})`;
}

/**
 * CSS colour for a tag: the colour chosen for it in Settings → Tags, else
 * its automatic one, so the same tag always gets the same colour.
 */
export function tagColor(tag: string): string {
  return `var(${tagVar(tag)}, ${autoTagColor(tag)})`;
}

/** An item's colour: explicit colour, else its first tag, else the accent. */
export function itemColor(item: Pick<Item, 'color' | 'tags'>): string {
  if (item.color) return item.color;
  const first = item.tags[0];
  return first ? tagColor(first) : 'var(--color-accent)';
}

/** The stylesheet that gives each coloured tag its variable. */
export function tagColorCss(tags: readonly TagInfo[]): string {
  const rules = tags.filter((t) => t.color).map((t) => `${tagVar(t.name)}: ${t.color};`);
  return rules.length ? `:root { ${rules.join(' ')} }` : '';
}
