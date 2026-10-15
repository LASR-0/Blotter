/**
 * The little Markdown models write in chat: paragraphs, bullet and numbered
 * lists, **bold** and `code`. Built as React elements, never as HTML, so a
 * reply can't inject markup.
 */
export type MdBlock = { type: 'p'; lines: string[] } | { type: 'ul' | 'ol'; items: string[] };

const BULLET = /^\s*[-*•]\s+(.*)$/;
const NUMBERED = /^\s*\d+[.)]\s+(.*)$/;

export function parseBlocks(text: string): MdBlock[] {
  const blocks: MdBlock[] = [];
  for (const raw of text.split('\n')) {
    const line = raw.trimEnd();
    const bullet = BULLET.exec(line);
    const numbered = bullet ? null : NUMBERED.exec(line);
    const last = blocks[blocks.length - 1];
    const listed = bullet ?? numbered;
    if (listed) {
      const type = bullet ? 'ul' : 'ol';
      const item = listed[1] ?? '';
      if (last?.type === type) last.items.push(item);
      else blocks.push({ type, items: [item] });
    } else if (!line.trim()) {
      blocks.push({ type: 'p', lines: [] });
    } else if (last?.type === 'p') {
      last.lines.push(line);
    } else {
      blocks.push({ type: 'p', lines: [line] });
    }
  }
  return blocks.filter((b) => (b.type === 'p' ? b.lines.length > 0 : true));
}
