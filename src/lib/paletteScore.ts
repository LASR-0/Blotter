/** Splits a label into words: spaces, dashes, arrows, colons, brackets. */
const words = (s: string) => s.split(/[\s\-–—↔→:()/·]+/).filter(Boolean);

/** `needle`'s letters appear in order in `hay`, starting at a word start. */
function wordStartSubsequence(hay: string, needle: string): boolean {
  const starts = words(hay).map((w) => hay.indexOf(w));
  for (const start of starts) {
    if (hay[start] !== needle[0]) continue;
    let i = 1;
    for (let j = start + 1; j < hay.length && i < needle.length; j++) {
      if (hay[j] === needle[i]) i++;
    }
    if (i === needle.length) return true;
  }
  return false;
}

/**
 * The command palette's ranking (cmdk `filter`): 0 hides a row, higher
 * sorts first. Exact and prefix label matches win, so typing "redo" puts
 * Redo first rather than a loose match like "pREvious perioD".
 */
export function paletteScore(value: string, search: string, keywords: string[] = []): number {
  const s = search.trim().toLowerCase();
  if (!s) return 1;
  const v = value.toLowerCase();
  const kws = keywords.map((k) => k.toLowerCase());
  const ws = words(v);
  if (v === s) return 1;
  if (v.startsWith(s)) return 0.9;
  if (ws.some((w) => w.startsWith(s))) return 0.8;
  const parts = s.split(/\s+/);
  if (
    parts.length > 1 &&
    parts.every((p) => ws.some((w) => w.startsWith(p)) || kws.some((k) => k.startsWith(p)))
  ) {
    return 0.7;
  }
  if (v.includes(s)) return 0.6;
  if (kws.some((k) => k.startsWith(s))) return 0.5;
  if (kws.some((k) => k.includes(s))) return 0.4;
  return wordStartSubsequence(v, s) ? 0.1 : 0;
}
