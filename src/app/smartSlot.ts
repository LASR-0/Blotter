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
  return startSlotThread('chat_smart_slot', ids);
}

/**
 * Consult me: the AI asks about each task in turn (how long, by when,
 * where, what time suits), records the answers on it, then places it.
 * A conversation in a new chat thread.
 */
export async function runConsultMe(ids?: string[]): Promise<void> {
  return startSlotThread('chat_consult_slot', ids);
}

async function startSlotThread(command: string, ids?: string[]): Promise<void> {
  try {
    const thread = await invoke<string>(command, { ids: ids ?? null });
    useChat.getState().showThread(thread);
    useChat.getState().setOpen(true);
  } catch (e) {
    useAutoSlot.setState({ error: isOpError(e) ? e.message : String(e) });
  }
}
