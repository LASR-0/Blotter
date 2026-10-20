import { useEffect, useLayoutEffect, useRef, useState, type KeyboardEvent } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import {
  ArrowSquareIn,
  ArrowSquareOut,
  ArrowUp,
  ChatCircleText,
  ClockCounterClockwise,
  Info,
  Lightning,
  NotePencil,
  PencilSimple,
  PushPin,
  SunHorizon,
  Stop,
  Trash,
  Tray,
  Warning,
  WarningCircle,
  Wrench,
  X,
  XCircle,
} from '@phosphor-icons/react';
import { Button } from '@/components/ui/button';
import { Kbd } from '@/components/ui/kbd';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Tooltip } from '@/components/ui/tooltip';
import { AI_SETTINGS_KEY, type AiSettings } from '@/lib/aiSettings';
import { addedAgo } from '@/lib/format';
import { cn } from '@/lib/cn';
import { messagesKey, PROPOSALS_KEY, THREADS_KEY, useChat } from './chatStore';
import { ChangedCard } from './ChangedCard';
import { ProposalCard } from './ProposalCard';
import type { Proposal } from './proposals';
import { Markdown } from './markdown';
import { transcript, type Item, type SavedMessage, type Thread } from './transcript';

interface ProviderView {
  name: string;
  model: string;
  is_default: boolean;
}

const SUGGESTIONS = [
  { icon: SunHorizon, text: 'Brief me on today' },
  { icon: Tray, text: 'Sort my inbox into this week' },
  { icon: Warning, text: 'Any conflicts this week?' },
];

const ICON_BTN = 'size-[26px] rounded-md';

/** "Claude Code · sonnet" or the default API provider and model. */
function useProviderLabel(): string {
  const ai = useQuery({
    queryKey: AI_SETTINGS_KEY,
    queryFn: () => invoke<AiSettings>('get_ai_settings'),
  });
  const providers = useQuery({
    queryKey: ['ai-providers'],
    queryFn: () => invoke<ProviderView[]>('list_providers'),
    enabled: ai.data?.route === 'api',
  });
  if (!ai.data) return '…';
  if (ai.data.route === 'claude_code') return `Claude Code · ${ai.data.claude_code_model}`;
  if (ai.data.route === 'codex') return `Codex · ${ai.data.codex_model || 'default'}`;
  const p = providers.data?.find((x) => x.is_default);
  return p ? `${p.name} · ${p.model || 'no model'}` : 'No provider set up';
}

function ThreadList({ onPick }: { onPick: () => void }) {
  const qc = useQueryClient();
  const threads = useQuery({
    queryKey: THREADS_KEY,
    queryFn: () => invoke<Thread[]>('chat_threads'),
  });
  const { threadId, showThread } = useChat();
  const remove = async (id: string) => {
    await invoke('chat_delete_thread', { threadId: id });
    if (id === threadId) showThread(null);
    void qc.invalidateQueries({ queryKey: THREADS_KEY });
  };
  const list = threads.data ?? [];
  return (
    <div className="flex max-h-[360px] w-[280px] flex-col gap-0.5 overflow-auto">
      {list.length === 0 && <span className="p-2 text-sm text-text-muted">No threads yet.</span>}
      {list.map((t) => (
        <div
          key={t.id}
          className={cn(
            'group flex items-center gap-1 rounded-md pr-1 hover:bg-hover',
            t.id === threadId && 'bg-hover',
          )}
        >
          <button
            className="flex min-w-0 flex-1 cursor-pointer flex-col items-start gap-0.5 px-2 py-1.5 text-left"
            onClick={() => {
              showThread(t.id);
              onPick();
            }}
          >
            <span className="w-full truncate text-sm">{t.title || 'Untitled'}</span>
            <span className="font-mono text-xs text-text-muted">{addedAgo(t.updated_at)}</span>
          </button>
          <Button
            size="icon-sm"
            variant="bare"
            aria-label={`Delete “${t.title}”`}
            className="opacity-0 group-hover:opacity-100 focus-visible:opacity-100"
            onClick={() => void remove(t.id)}
          >
            <Trash />
          </Button>
        </div>
      ))}
    </div>
  );
}

function Row({
  item,
  streaming,
  proposals,
}: {
  item: Item;
  streaming: boolean;
  proposals: Map<string, Proposal>;
}) {
  switch (item.kind) {
    case 'user':
      return (
        <div className="max-w-[85%] self-end rounded-[10px_10px_3px_10px] bg-accent-tint px-[11px] py-2 leading-normal whitespace-pre-wrap">
          {item.text}
        </div>
      );
    case 'text':
      return (
        <div className="pr-2 leading-[1.55]">
          <Markdown text={item.text} />
          {streaming && (
            <span className="ml-0.5 inline-block h-3.5 w-[7px] animate-[bl-blink_1s_steps(1)_infinite] bg-accent align-[-2px]" />
          )}
        </div>
      );
    case 'tool': {
      const failed = item.status === 'error';
      return (
        <div className="flex flex-col gap-1">
          <div
            className={cn(
              'flex items-center gap-1.5 font-mono text-[10.5px]',
              failed ? 'text-danger' : 'text-text-muted',
            )}
          >
            {failed ? <WarningCircle /> : <Wrench />}
            <span className="truncate">
              {item.name}({item.args})
            </span>
            {item.status === 'running' && <span className="animate-pulse">…</span>}
          </div>
          {failed && item.detail && (
            <code className="ml-4 font-mono text-[10.5px] leading-normal whitespace-pre-wrap text-text-muted">
              {item.detail}
            </code>
          )}
        </div>
      );
    }
    case 'error':
      return (
        <div className="flex flex-col gap-1.5 rounded-md border border-danger bg-[color-mix(in_srgb,var(--color-danger)_7%,var(--color-surface))] px-3 py-2.5">
          <span className="flex items-center gap-1.5 font-medium text-danger">
            <XCircle weight="fill" />
            Couldn&apos;t answer
          </span>
          <span className="text-sm leading-normal whitespace-pre-wrap">{item.message}</span>
        </div>
      );
    case 'notice':
      return (
        <span className="flex items-center gap-1.5 text-xs text-text-muted">
          <Info />
          {item.text}
        </span>
      );
    case 'changed':
      return <ChangedCard batches={item.batches} />;
    case 'proposal': {
      const p = proposals.get(item.id);
      return p ? <ProposalCard proposal={p} /> : null;
    }
  }
}

/** The chat (design: Chat Panel), docked in the main window or popped out. */
export function ChatPanel({ mode }: { mode: 'docked' | 'popout' }) {
  const { threadId, live, sendError, send, stop, showThread, setOpen } = useChat();
  const provider = useProviderLabel();
  const qc = useQueryClient();
  const ai = useQuery({
    queryKey: AI_SETTINGS_KEY,
    queryFn: () => invoke<AiSettings>('get_ai_settings'),
  });
  const auto = ai.data?.mode === 'auto';
  const toggleMode = () =>
    void invoke<AiSettings>('set_ai_settings', { input: { mode: auto ? 'propose' : 'auto' } }).then(
      (s) => qc.setQueryData(AI_SETTINGS_KEY, s),
    );
  const [draft, setDraft] = useState('');
  const [threadsOpen, setThreadsOpen] = useState(false);
  const [onTop, setOnTop] = useState(true);
  const input = useRef<HTMLTextAreaElement>(null);
  const scroller = useRef<HTMLDivElement>(null);

  const threads = useQuery({
    queryKey: THREADS_KEY,
    queryFn: () => invoke<Thread[]>('chat_threads'),
  });
  const saved = useQuery({
    queryKey: messagesKey(threadId),
    queryFn: () => invoke<SavedMessage[]>('chat_messages', { threadId }),
    enabled: threadId !== null,
  });
  const threadProposals = useQuery({
    queryKey: [...PROPOSALS_KEY, threadId],
    queryFn: () => invoke<Proposal[]>('chat_proposals', { threadId }),
    enabled: threadId !== null,
  });
  // Waiting changes from MCP clients (Claude Desktop, …), shown in every thread.
  const external = useQuery({
    queryKey: [...PROPOSALS_KEY, null],
    queryFn: () => invoke<Proposal[]>('chat_proposals', { threadId: null }),
  });
  const proposalMap = new Map((threadProposals.data ?? []).map((p) => [p.id, p]));
  const turn = threadId ? live[threadId] : undefined;
  const running = turn?.running ?? false;
  const items = [
    ...transcript(saved.data ?? [], threadProposals.data ?? []),
    ...(turn?.items ?? []),
  ];
  const title = threads.data?.find((t) => t.id === threadId)?.title ?? 'New thread';
  const empty = threadId === null && items.length === 0;

  useEffect(() => input.current?.focus(), [threadId]);
  // Have Claude Code start up now, so it's ready when a message is sent.
  useEffect(() => {
    invoke('chat_prewarm', { threadId }).catch((e: unknown) =>
      console.warn('Chat warm-up failed', e),
    );
  }, [threadId]);
  // Follow the reply as it streams.
  useLayoutEffect(() => {
    const el = scroller.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [items.length, turn]);

  const submit = (text = draft) => {
    const t = text.trim();
    if (!t || running) return;
    setDraft('');
    void send(t);
  };
  const onKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey && !e.nativeEvent.isComposing) {
      e.preventDefault();
      submit();
    }
    if (e.key === 'Escape' && mode === 'popout') void getCurrentWindow().close();
  };

  const popOut = () => {
    setOpen(false);
    void invoke('open_chat_window', { threadId });
  };
  const toggleOnTop = () => {
    const next = !onTop;
    setOnTop(next);
    void invoke('set_chat_on_top', { onTop: next });
  };

  return (
    <div className="flex h-full min-h-0 flex-col bg-surface text-base text-text">
      <div className="flex flex-col gap-2 border-b border-border px-3 py-2.5">
        <div className="flex items-center gap-1.5">
          <span className="min-w-0 flex-1 truncate font-semibold">{title}</span>
          <Tooltip label="New thread">
            <Button
              size="icon"
              variant="bare"
              className={ICON_BTN}
              aria-label="New thread"
              onClick={() => showThread(null)}
            >
              <NotePencil />
            </Button>
          </Tooltip>
          <Popover open={threadsOpen} onOpenChange={setThreadsOpen}>
            <Tooltip label="Threads">
              <PopoverTrigger asChild>
                <Button size="icon" variant="bare" className={ICON_BTN} aria-label="Threads">
                  <ClockCounterClockwise />
                </Button>
              </PopoverTrigger>
            </Tooltip>
            <PopoverContent align="end" className="p-1.5">
              <ThreadList onPick={() => setThreadsOpen(false)} />
            </PopoverContent>
          </Popover>
          {mode === 'docked' ? (
            <>
              <Tooltip label="Pop out" shortcut="Ctrl+Shift+/">
                <Button
                  size="icon"
                  variant="bare"
                  className={ICON_BTN}
                  aria-label="Pop out"
                  onClick={popOut}
                >
                  <ArrowSquareOut />
                </Button>
              </Tooltip>
              <Tooltip label="Close chat" shortcut="Ctrl+/">
                <Button
                  size="icon"
                  variant="bare"
                  className={ICON_BTN}
                  aria-label="Close chat"
                  onClick={() => setOpen(false)}
                >
                  <X />
                </Button>
              </Tooltip>
            </>
          ) : (
            <>
              <Tooltip label={onTop ? 'Stop keeping on top' : 'Keep on top'}>
                <Button
                  size="icon"
                  variant="bare"
                  className={cn(ICON_BTN, onTop && 'bg-accent-tint text-accent-text')}
                  aria-label="Keep on top"
                  aria-pressed={onTop}
                  onClick={toggleOnTop}
                >
                  <PushPin weight={onTop ? 'fill' : 'regular'} />
                </Button>
              </Tooltip>
              <Tooltip label="Dock into the main window">
                <Button
                  size="icon"
                  variant="bare"
                  className={ICON_BTN}
                  aria-label="Dock into the main window"
                  onClick={() => void invoke('dock_chat', { threadId })}
                >
                  <ArrowSquareIn />
                </Button>
              </Tooltip>
            </>
          )}
        </div>
        <div className="flex items-center gap-1.5 text-xs">
          <span
            title="Provider and model (Settings → AI providers)"
            className="flex h-[22px] min-w-0 items-center gap-1.5 rounded-full border border-border px-2 text-text-muted"
          >
            <span className="size-1.5 flex-none rounded-full bg-tag-health" />
            <span className="truncate font-mono">{provider}</span>
          </span>
          <button
            onClick={toggleMode}
            aria-label={auto ? 'Auto mode: switch to Propose' : 'Propose mode: switch to Auto'}
            title={
              auto
                ? 'Auto: changes apply straight away, after a safety snapshot, and can be undone. Click for Propose.'
                : 'Propose: changes wait for you to accept them. Click for Auto.'
            }
            className={cn(
              'flex h-[22px] cursor-pointer items-center gap-1 rounded-full px-2 font-medium whitespace-nowrap',
              auto
                ? 'bg-[color-mix(in_srgb,var(--color-warning)_14%,transparent)] text-warning'
                : 'bg-accent-tint text-accent-text',
            )}
          >
            {auto ? <Lightning /> : <PencilSimple />}
            {auto ? 'Auto' : 'Propose'}
          </button>
        </div>
      </div>

      <div
        ref={scroller}
        className="flex min-h-0 flex-1 flex-col gap-3 overflow-auto px-3 py-3.5"
        aria-live="polite"
      >
        {(external.data ?? []).map((p) => (
          <ProposalCard key={p.id} proposal={p} showActor />
        ))}
        {empty && (
          <div className="my-auto flex flex-col gap-3 p-2">
            <ChatCircleText size={24} className="text-text-muted" />
            <span className="text-md font-semibold">New thread</span>
            <span className="leading-normal text-text-muted">
              Ask about your plans or tell Blotter what to schedule.{' '}
              {auto
                ? 'Changes apply straight away and can be undone.'
                : 'Changes arrive as proposals you accept or reject.'}
            </span>
            <div className="flex flex-col gap-1">
              {SUGGESTIONS.map(({ icon: Icon, text }) => (
                <button
                  key={text}
                  onClick={() => submit(text)}
                  className="flex h-8 cursor-pointer items-center gap-2 rounded-md border border-border bg-bg px-2.5 text-left hover:border-border-strong hover:bg-hover"
                >
                  <Icon className="text-text-muted" />
                  {text}
                </button>
              ))}
            </div>
          </div>
        )}
        {saved.isLoading && threadId && (
          <span className="font-mono text-xs text-text-muted">Loading thread…</span>
        )}
        {items.map((item, i) => (
          <Row
            key={i}
            item={item}
            streaming={running && i === items.length - 1}
            proposals={proposalMap}
          />
        ))}
        {running && items[items.length - 1]?.kind !== 'text' && (
          <span className="animate-pulse font-mono text-xs text-text-muted">Thinking…</span>
        )}
      </div>

      <div className="border-t border-border px-3 pt-2.5 pb-3">
        {sendError && <p className="m-0 mb-2 text-sm text-danger">{sendError}</p>}
        <div className="flex flex-col rounded-md border border-border-strong bg-bg focus-within:border-accent">
          <textarea
            ref={input}
            rows={2}
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            onKeyDown={onKeyDown}
            placeholder="Ask Blotter, or tell it what to plan…"
            aria-label="Message"
            className="resize-none border-0 bg-transparent px-2.5 py-[9px] text-text outline-none placeholder:text-text-muted"
          />
          <div className="flex items-center gap-1.5 px-1.5 pt-1 pb-1.5 pl-2.5 text-xs text-text-muted">
            <span>context: today + inbox</span>
            <span className="flex-1" />
            {running ? (
              <Button size="sm" onClick={() => void stop()} aria-label="Stop">
                <Stop weight="fill" />
                Stop
              </Button>
            ) : (
              <>
                <Kbd>↵</Kbd>
                <Button
                  size="icon"
                  variant="primary"
                  className={ICON_BTN}
                  aria-label="Send"
                  disabled={!draft.trim()}
                  onClick={() => submit()}
                >
                  <ArrowUp weight="bold" />
                </Button>
              </>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
