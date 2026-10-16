import { invoke } from '@tauri-apps/api/core';
import { useChat } from '@/chat/chatStore';
import { isOpError } from '@/lib/api';
import { useAutoSlot } from './autoSlot';

/**
 * Smart slot: the AI places `ids` (or every open Inbox task) with the
 * context auto-slot can't weigh — what each task is, travel, breathing
 * room. It runs as a chat turn in a new thread, which opens so the user
 * can follow it and accept its proposals. A failure to start shows in the
 * auto-slot panel.
 */
export async function runSmartSlot(ids?: string[]): Promise<void> {
  try {
    const thread = await invoke<string>('chat_smart_slot', { ids: ids ?? null });
    useChat.getState().showThread(thread);
    useChat.getState().setOpen(true);
  } catch (e) {
    useAutoSlot.setState({ error: isOpError(e) ? e.message : String(e) });
  }
}
