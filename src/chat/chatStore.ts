import { useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { create } from 'zustand';
import { isOpError } from '@/lib/api';
import { queryClient } from '@/lib/queries';
import { applyEvent, type ChatEvent, type LiveTurn } from './transcript';

export const THREADS_KEY = ['chat-threads'];
export const messagesKey = (threadId: string | null) => ['chat-messages', threadId];
/** Proposals (all of them; a thread's under ['chat-proposals', id], MCP clients' under null). */
export const PROPOSALS_KEY = ['chat-proposals'];
export const PENDING_KEY = ['pending-proposals'];

interface ChatState {
  /** The panel docked in the main window is showing. */
  open: boolean;
  /** The thread on screen; null for a new one. */
  threadId: string | null;
  /** Turns in progress (and just finished, until the saved copy loads). */
  live: Record<string, LiveTurn>;
  /** A send that failed before a turn started. */
  sendError: string | null;
  setOpen: (open: boolean) => void;
  showThread: (threadId: string | null) => void;
  send: (text: string) => Promise<void>;
  stop: () => Promise<void>;
  onEvent: (e: ChatEvent) => void;
}

export const useChat = create<ChatState>((set, get) => ({
  open: false,
  threadId: null,
  live: {},
  sendError: null,
  setOpen: (open) => set({ open }),
  showThread: (threadId) => set({ threadId, sendError: null }),
  send: async (text) => {
    // Drop what's left of the last turn (a "Stopped." notice).
    set((s) => {
      const live = { ...s.live };
      const current = s.threadId ? live[s.threadId] : undefined;
      if (s.threadId && current && !current.running) delete live[s.threadId];
      return { live, sendError: null };
    });
    try {
      const id = await invoke<string>('chat_send', { threadId: get().threadId, text });
      // The message is saved before the turn starts, so it shows from the
      // saved copy; the reply's events (maybe already here) follow it.
      set((s) => ({
        threadId: id,
        live: { ...s.live, [id]: s.live[id] ?? { items: [], running: true } },
      }));
      void queryClient.invalidateQueries({ queryKey: messagesKey(id) });
      void queryClient.invalidateQueries({ queryKey: THREADS_KEY });
    } catch (e) {
      set({ sendError: isOpError(e) ? e.message : String(e) });
    }
  },
  stop: async () => {
    const id = get().threadId;
    if (id) await invoke('chat_cancel', { threadId: id });
  },
  onEvent: (e) => {
    set((s) => ({
      live: {
        ...s.live,
        [e.thread_id]: applyEvent(s.live[e.thread_id] ?? { items: [], running: true }, e),
      },
    }));
    if (e.type === 'done' || e.type === 'error') void settle(e.thread_id, e);
  },
}));

/** Once a turn ends, loads its saved copy and drops the live one. A notice
 *  (e.g. "Stopped.") isn't saved, so it stays until the next turn. */
async function settle(threadId: string, e: ChatEvent) {
  await Promise.all([
    queryClient.invalidateQueries({ queryKey: messagesKey(threadId) }),
    queryClient.invalidateQueries({ queryKey: PROPOSALS_KEY }),
  ]);
  void queryClient.invalidateQueries({ queryKey: PENDING_KEY });
  void queryClient.invalidateQueries({ queryKey: THREADS_KEY });
  useChat.setState((s) => {
    const live = { ...s.live };
    const notices =
      e.type === 'done' && e.notice ? [{ kind: 'notice' as const, text: e.notice }] : [];
    if (notices.length) live[threadId] = { items: notices, running: false };
    else delete live[threadId];
    return { live };
  });
}

/** Feeds `chat:event`s into the store, and refreshes proposals when the
 *  planner changes (an MCP client may have proposed something). Mount once
 *  per window. */
export function useChatEvents() {
  useEffect(() => {
    const stop = listen<ChatEvent>('chat:event', (event) => {
      useChat.getState().onEvent(event.payload);
    });
    const stopChanged = listen('planner:changed', () => {
      void queryClient.invalidateQueries({ queryKey: PROPOSALS_KEY });
      void queryClient.invalidateQueries({ queryKey: PENDING_KEY });
    });
    return () => {
      void stop.then((unlisten) => unlisten());
      void stopChanged.then((unlisten) => unlisten());
    };
  }, []);
}
