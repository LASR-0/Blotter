// Mirrors the JSON the core returns (crates/blotter-core/src/model).
// Instants are RFC 3339 UTC strings; dates are YYYY-MM-DD; times HH:MM.

export type ItemKind = 'task' | 'event' | 'reminder';
export type Anchoring = 'fixed' | 'flexible';
export type ItemStatus = 'open' | 'done' | 'cancelled';
export type Source = 'user' | 'ai' | 'import' | 'mcp';

export interface Item {
  id: string;
  kind: ItemKind;
  anchoring: Anchoring;
  status: ItemStatus;
  title: string;
  notes: string;
  all_day: boolean;
  start_at: string | null;
  end_at: string | null;
  start_date: string | null;
  end_date: string | null;
  tz: string | null;
  duration_min: number | null;
  due_at: string | null;
  priority: number;
  tags: string[];
  color: string | null;
  sort_order: number;
  rrule: string | null;
  parent_id: string | null;
  recurrence_id: string | null;
  completed_at: string | null;
  source: Source;
  /** The iCalendar UID it was imported with. */
  ics_uid?: string | null;
  created_at: string;
  updated_at: string;
}

export type Weekday = 'mon' | 'tue' | 'wed' | 'thu' | 'fri' | 'sat' | 'sun';

export interface BlockoutException {
  id: string;
  blockout_id: string;
  date: string;
  note: string;
  created_at: string;
}

export interface Blockout {
  id: string;
  label: string;
  weekdays: Weekday[];
  start_time: string;
  end_time: string;
  tz: string;
  active_from: string | null;
  active_until: string | null;
  permits_fixed: boolean;
  color: string | null;
  icon: string | null;
  sort_order: number;
  created_at: string;
  updated_at: string;
  exceptions: BlockoutException[];
}

/** One occurrence of a block-out (`list_blockout_windows`); instants are UTC. */
export interface BlockoutWindow {
  blockout_id: string;
  label: string;
  /** Local date the occurrence starts on; lift it with an exception on this date. */
  date: string;
  start: string;
  end: string;
  permits_fixed: boolean;
  color: string | null;
  /** Set for a busy event of a subscribed calendar treated as a block-out (can't be lifted). */
  subscription_id: string | null;
  /** That calendar's name. */
  calendar: string | null;
}

export interface BlockoutWindows {
  start: string;
  end: string;
  timezone: string;
  windows: BlockoutWindow[];
}

/** An item that overlaps others, or sits fixed in a block-out that warns. */
export interface Conflict {
  item_id: string;
  overlaps: string[];
  blockout: string | null;
}

export interface Agenda {
  start: string;
  end: string;
  timezone: string;
  items: Item[];
  conflicts: Conflict[];
}

/** `auto_slot`'s result. */
export interface AutoSlotResult {
  timezone: string;
  placed: {
    item: Item;
    start: string;
    end: string;
    previous_start: string | null;
    previous_date: string | null;
  }[];
  unplaced: {
    id: string;
    title: string;
    duration_min: number;
    due_at: string | null;
    reason: string;
    longest_gap: { start: string; end: string; minutes: number } | null;
  }[];
}
