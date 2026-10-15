import type { ReactNode } from 'react';
import { parseBlocks } from './mdParse';

/** `**bold**` and `` `code` `` within a line. */
function inline(text: string): ReactNode[] {
  const out: ReactNode[] = [];
  const re = /\*\*(.+?)\*\*|`([^`]+)`/g;
  let at = 0;
  let m: RegExpExecArray | null;
  while ((m = re.exec(text))) {
    if (m.index > at) out.push(text.slice(at, m.index));
    out.push(
      m[1] !== undefined ? (
        <strong key={m.index} className="font-semibold">
          {m[1]}
        </strong>
      ) : (
        <code key={m.index} className="rounded-sm bg-hover px-1 font-mono text-[0.92em]">
          {m[2]}
        </code>
      ),
    );
    at = m.index + m[0].length;
  }
  if (at < text.length) out.push(text.slice(at));
  return out;
}

/** A chat reply's Markdown as React elements (see mdParse), never as HTML. */
export function Markdown({ text }: { text: string }) {
  return (
    <>
      {parseBlocks(text).map((b, i) => {
        if (b.type === 'p') {
          return (
            <p key={i} className="m-0 [&+*]:mt-2">
              {b.lines.map((l, j) => (
                <span key={j}>
                  {j > 0 && <br />}
                  {inline(l)}
                </span>
              ))}
            </p>
          );
        }
        const List = b.type;
        return (
          <List
            key={i}
            className={`m-0 pl-5 [&+*]:mt-2 ${b.type === 'ul' ? 'list-disc' : 'list-decimal'}`}
          >
            {b.items.map((item, j) => (
              <li key={j}>{inline(item)}</li>
            ))}
          </List>
        );
      })}
    </>
  );
}
