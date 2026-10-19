import { useEffect } from 'react';
import { useOp } from '@/lib/queries';
import { tagColorCss, type TagInfo } from '@/lib/tags';

/**
 * Publishes the tag colours chosen in Settings → Tags as CSS variables, which
 * tagColor() reads, so a change shows everywhere at once. Mounted once per window.
 */
export function TagColors() {
  const tags = useOp<TagInfo[]>('list_tags');
  const css = tagColorCss(tags.data ?? []);
  useEffect(() => {
    let el = document.getElementById('blotter-tag-colors');
    if (!el) {
      el = document.createElement('style');
      el.id = 'blotter-tag-colors';
      document.head.appendChild(el);
    }
    el.textContent = css;
  }, [css]);
  return null;
}
