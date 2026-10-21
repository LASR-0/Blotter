import { useState, type FormEvent } from 'react';
import { Hash, Plus, Trash } from '@phosphor-icons/react';
import { useOverlay } from '@/app/overlay';
import { Button } from '@/components/ui/button';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Switch } from '@/components/ui/switch';
import { Tooltip } from '@/components/ui/tooltip';
import { isOpError, runOp } from '@/lib/api';
import { cn } from '@/lib/cn';
import { useOp } from '@/lib/queries';
import { autoTagColor, TAG_SWATCHES, tagColor, type TagInfo } from '@/lib/tags';

const errorText = (e: unknown) => (isOpError(e) ? e.message : String(e));

/** The colour chip and its picker: the theme's tag colours, a custom hex, or automatic. */
function ColorPicker({ tag, onError }: { tag: TagInfo; onError: (e: unknown) => void }) {
  const [open, setOpen] = useState(false);
  const [hex, setHex] = useState(tag.color?.startsWith('#') ? tag.color : '');
  const pick = (color: string | null) => {
    setOpen(false);
    runOp('set_tag', { name: tag.name, color }).catch(onError);
  };
  const hexOk = /^#[0-9a-f]{6}$/i.test(hex);

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <button
          type="button"
          aria-label={`Colour of #${tag.name}`}
          className="size-5 flex-none cursor-pointer rounded-full border border-border-strong"
          style={{ background: tagColor(tag.name) }}
        />
      </PopoverTrigger>
      <PopoverContent align="start" className="flex w-[232px] flex-col gap-3">
        <div className="flex flex-wrap gap-2">
          {TAG_SWATCHES.map((c) => (
            <button
              key={c}
              type="button"
              aria-label={c.slice(10, -1)}
              aria-pressed={tag.color === c}
              onClick={() => pick(c)}
              className={cn(
                'size-6 cursor-pointer rounded-full border border-border-strong',
                tag.color === c && 'outline-2 outline-offset-2 outline-focus-ring',
              )}
              style={{ background: c }}
            />
          ))}
        </div>
        <form
          className="flex items-center gap-2"
          onSubmit={(e) => {
            e.preventDefault();
            if (hexOk) pick(hex.toLowerCase());
          }}
        >
          <input
            type="color"
            aria-label="Custom colour"
            value={hexOk ? hex : '#888888'}
            onChange={(e) => setHex(e.target.value)}
            className="size-7 flex-none cursor-pointer rounded-sm border border-border bg-transparent"
          />
          <input
            value={hex}
            onChange={(e) => setHex(e.target.value.trim())}
            placeholder="#rrggbb"
            aria-label="Custom colour (hex)"
            className="h-7 min-w-0 flex-1 rounded-sm border border-border bg-bg px-2 font-mono text-sm text-text outline-none focus:border-accent"
          />
          <Button type="submit" size="sm" disabled={!hexOk}>
            Set
          </Button>
        </form>
        <button
          type="button"
          onClick={() => pick(null)}
          className="flex cursor-pointer items-center gap-2 self-start text-sm text-text-muted hover:text-text"
        >
          <span
            className="size-3.5 rounded-full border border-border-strong"
            style={{ background: autoTagColor(tag.name) }}
          />
          Automatic
        </button>
      </PopoverContent>
    </Popover>
  );
}

function TagRow({ tag, onError }: { tag: TagInfo; onError: (e: unknown) => void }) {
  const ask = useOverlay((s) => s.ask);
  const [name, setName] = useState(tag.name);
  const rename = () => {
    const to = name.trim().replace(/^#/, '').toLowerCase();
    if (!to || to === tag.name) {
      setName(tag.name);
      return;
    }
    runOp('rename_tag', { from: tag.name, to }).catch((e: unknown) => {
      setName(tag.name);
      onError(e);
    });
  };
  const remove = () =>
    ask({
      title: `Delete #${tag.name}?`,
      body:
        tag.items > 0
          ? `It comes off ${tag.items === 1 ? 'the 1 item' : `all ${tag.items} items`} that carry it. Ctrl+Z puts it back.`
          : 'It isn’t on any items yet.',
      confirmLabel: 'Delete tag',
      danger: true,
      onConfirm: () => runOp('delete_tag', { name: tag.name }).then(() => undefined, onError),
    });

  return (
    <li className="flex items-center gap-3 border-b border-border py-2">
      <ColorPicker tag={tag} onError={onError} />
      <span className="flex min-w-0 flex-1 items-center text-text-muted">
        #
        <input
          value={name}
          onChange={(e) => setName(e.target.value)}
          onBlur={rename}
          onKeyDown={(e) => {
            if (e.key === 'Enter') e.currentTarget.blur();
            if (e.key === 'Escape') {
              setName(tag.name);
              e.currentTarget.blur();
            }
          }}
          aria-label={`Rename #${tag.name}`}
          className="h-7 min-w-0 flex-1 rounded-sm border border-transparent bg-transparent px-1 text-text outline-none hover:border-border focus:border-accent"
        />
      </span>
      <Tooltip label="Tasks with this tag can be done simultaneously with other tasks (never over events or fixed items)">
        <label className="flex cursor-pointer items-center gap-2 text-sm text-text-muted">
          Simultaneous
          <Switch
            aria-label={`Tasks tagged #${tag.name} can be done simultaneously`}
            checked={tag.shares_time}
            onCheckedChange={(on) =>
              runOp('set_tag', { name: tag.name, shares_time: on }).catch(onError)
            }
          />
        </label>
      </Tooltip>
      <span className="w-16 text-right font-mono text-sm text-text-muted">
        {tag.items} {tag.items === 1 ? 'item' : 'items'}
      </span>
      <Button
        variant="bare"
        size="icon-sm"
        aria-label={`Delete #${tag.name}`}
        title="Delete tag"
        onClick={remove}
      >
        <Trash />
      </Button>
    </li>
  );
}

/** Settings → Tags: every tag with its colour and use count; create, rename, recolour, delete. */
export function TagsPage() {
  const tags = useOp<TagInfo[]>('list_tags');
  const [draft, setDraft] = useState('');
  const [error, setError] = useState<string | null>(null);
  const fail = (e: unknown) => setError(errorText(e));

  const create = (e: FormEvent) => {
    e.preventDefault();
    const name = draft.trim().replace(/^#/, '');
    if (!name) return;
    runOp('set_tag', { name })
      .then(() => {
        setDraft('');
        setError(null);
      })
      .catch(fail);
  };

  return (
    <div className="flex max-w-[820px] flex-col gap-[18px] px-8 pt-6 pb-12">
      <h1 className="m-0 font-heading text-xl font-heading-weight">Tags</h1>
      <p className="m-0 max-w-[600px] leading-normal text-text-muted">
        Tags come from <span className="font-mono">#tag</span> in quick add and the Inbox, or from
        here. Renaming a tag onto another merges them; deleting one takes it off every item. Both
        undo with Ctrl+Z. Tasks with a simultaneous tag, like laundry, can be slotted over other
        tasks, such as cleaning.
      </p>
      <form onSubmit={create} className="flex max-w-[420px] items-center gap-2">
        <label className="flex h-[30px] flex-1 items-center gap-2 rounded-md border border-border bg-bg px-2 text-text-muted focus-within:border-accent">
          <Hash />
          <input
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            placeholder="New tag"
            aria-label="New tag"
            className="min-w-0 flex-1 bg-transparent text-text outline-none"
          />
        </label>
        <Button type="submit" disabled={!draft.trim()}>
          <Plus />
          Add
        </Button>
      </form>
      {error && <p className="m-0 text-sm text-danger">{error}</p>}
      {tags.data && tags.data.length === 0 ? (
        <p className="m-0 text-sm text-text-muted">No tags yet.</p>
      ) : (
        <ul className="m-0 flex list-none flex-col p-0">
          {(tags.data ?? []).map((t) => (
            // Keyed by name and colour, so a rename or recolour starts the row afresh.
            <TagRow key={`${t.name}:${t.color ?? ''}`} tag={t} onError={fail} />
          ))}
        </ul>
      )}
    </div>
  );
}
