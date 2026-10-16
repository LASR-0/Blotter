import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { ChatPanel } from '@/chat/ChatPanel';
import { useChat, useChatEvents } from '@/chat/chatStore';
import { syncPrefs } from '@/lib/prefs';
import { usePlannerSync } from '@/lib/queries';
import { useTheme } from '@/theme/theme';

/** The chat pop-out window ("sticky note", D10): only the chat panel. */
export function ChatApp() {
  useChatEvents();
  usePlannerSync();

  useEffect(() => syncPrefs(), []);
  useEffect(() => {
    const thread = new URLSearchParams(window.location.search).get('thread');
    useChat.getState().showThread(thread);
    useTheme
      .getState()
      .load()
      .catch((e: unknown) => console.error('Could not load theme', e));
    const unlisten = listen<{ thread_id: string | null }>('chat:show-thread', (e) => {
      useChat.getState().showThread(e.payload.thread_id);
    });
    return () => {
      void unlisten.then((stop) => stop());
    };
  }, []);

  return <ChatPanel mode="popout" />;
}
