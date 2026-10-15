import { describe, expect, it } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { Markdown } from './markdown';
import { parseBlocks } from './mdParse';

describe('parseBlocks', () => {
  it('splits paragraphs and lists', () => {
    expect(parseBlocks('Here you go:\n\n- One\n- Two\n\n1. First\n2) Second\nAfter')).toEqual([
      { type: 'p', lines: ['Here you go:'] },
      { type: 'ul', items: ['One', 'Two'] },
      { type: 'ol', items: ['First', 'Second'] },
      { type: 'p', lines: ['After'] },
    ]);
  });

  it('keeps soft line breaks inside a paragraph', () => {
    expect(parseBlocks('a\nb')).toEqual([{ type: 'p', lines: ['a', 'b'] }]);
  });
});

describe('Markdown', () => {
  it('renders bold and code, and escapes markup', () => {
    const html = renderToStaticMarkup(
      <Markdown text={'You have **8 items**, see `list_inbox`. <b>x</b>'} />,
    );
    expect(html).toContain('<strong class="font-semibold">8 items</strong>');
    expect(html).toContain('>list_inbox</code>');
    expect(html).toContain('&lt;b&gt;x&lt;/b&gt;');
  });
});
