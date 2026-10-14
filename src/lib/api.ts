import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

/** Mirrors `CommandError` in src-tauri/src/commands.rs. */
export interface OpError {
  code:
    'not_found' | 'invalid' | 'invalid_args' | 'unknown_op' | 'database' | 'corrupt' | 'internal';
  message: string;
  details: string[];
}

export interface Change {
  id: string;
  batch_id: string;
  op: string;
  entity: string;
  entity_id: string;
  before: unknown;
  after: unknown;
  actor: string;
  created_at: string;
  undone_at: string | null;
}

export interface OpOutcome<T> {
  result: T;
  batch_id: string | null;
  changes: Change[];
}

export interface OpInfo {
  name: string;
  description: string;
  mutates: boolean;
  schema: Record<string, unknown>;
}

export function isOpError(e: unknown): e is OpError {
  return typeof e === 'object' && e !== null && 'code' in e && 'message' in e;
}

/** Runs one operation through the core registry, as the user. */
export async function runOp<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  const outcome = await invoke<OpOutcome<T>>('run_op', { name, args: args ?? null });
  return outcome.result;
}

export function listOps(): Promise<OpInfo[]> {
  return invoke<OpInfo[]>('list_ops');
}

export interface ChangedEvent {
  batch_id: string;
  ops: string[];
}

/** Fires after any committed mutation, from any window or actor. */
export function onPlannerChanged(handler: (e: ChangedEvent) => void): Promise<UnlistenFn> {
  return listen<ChangedEvent>('planner:changed', (event) => handler(event.payload));
}
