// Natural-language quick add: "dentist tue 3pm ~45m @fixed #health".
//
// A custom token layer runs first (so chrono never mistakes "~45m" for a
// time), then chrono-node reads dates and times from what remains. It
// works offline and is pure, so it is fully unit-tested.
//
// Grammar
//   ~45m  ~1h30  ~1.5h  ~90     duration
//   !1 !2 !3                    priority
//   #tag                        tag (letters, digits, - and _)
//   @fixed  @flex               anchoring
//   every day | weekday | weekend | week | month | year
//   every monday[, wed and fri] | every 2 weeks | every other week
//                               recurrence (RFC 5545 RRULE)
//   due <date [time]>           deadline
//   after <date> | from <date> to <date>
//                               do-window (not before; a range's end is the due date)
//   // anything to the end      context for Smart slot, kept in the notes
//   <date> <time>               when (chrono: tue, tomorrow, 20/10, next friday, 3pm, 18:30, noon…)
//
// Fixed anchoring rule: @fixed / @flex always win. Otherwise an explicit
// time on an event-like phrase (dentist, meeting, appointment, lunch,
// interview, class, flight…) makes it a fixed event; any other timed entry
// stays a flexible task.

import * as chrono from 'chrono-node';
import { format, parseISO } from 'date-fns';
import { formatDuration, parseDuration } from './duration';
import { parseTime } from './format';
import type { Anchoring, ItemKind } from './types';

export type ChipKind =
  | 'date'
  | 'time'
  | 'duration'
  | 'priority'
  | 'tag'
  | 'anchoring'
  | 'repeat'
  | 'due'
  | 'window'
  | 'context';

export interface Chip {
  kind: ChipKind;
  label: string;
  value: string;
  /** The text it came from. */
  source: string;
}

export interface QuickParse {
  title: string;
  kind: ItemKind;
  anchoring: Anchoring;
  /** YYYY-MM-DD */
  date?: string;
  /** HH:MM */
  time?: string;
  durationMin?: number;
  priority?: number;
  tags: string[];
  /** YYYY-MM-DDTHH:MM (local) */
  due?: string;
  rrule?: string;
  /** Usual time of day (HH:MM), as in "9-12". */
  windowStart?: string;
  windowEnd?: string;
  /** Do-window start (after / from …), YYYY-MM-DD. */
  notBefore?: string;
  /** Context after `//`, for the notes. */
  notes?: string;
  chips: Chip[];
}

/** Words that make a timed entry an event (fixed) rather than a task. */
const EVENT_WORDS =
  /\b(appointment|appt|meeting|meet|dentist|doctor|dr|gp|physio|vet|interview|lunch|dinner|breakfast|brunch|coffee|drinks|class|lesson|flight|train|concert|party|wedding|birthday|game|match|session|consult|consultation|telehealth|haircut|call with|catch[- ]up|standup|stand-up|webinar|conference)\b/i;

const WEEKDAYS: [RegExp, string, string][] = [
  [/^mon(day)?s?$/i, 'MO', 'Monday'],
  [/^tue(s(day)?)?s?$/i, 'TU', 'Tuesday'],
  [/^wed(nesday)?s?$/i, 'WE', 'Wednesday'],
  [/^thu(r(s(day)?)?)?s?$/i, 'TH', 'Thursday'],
  [/^fri(day)?s?$/i, 'FR', 'Friday'],
  [/^sat(urday)?s?$/i, 'SA', 'Saturday'],
  [/^sun(day)?s?$/i, 'SU', 'Sunday'],
];
const DAY_WORD = '(?:mon|tue|tues|wed|thu|thur|thurs|fri|sat|sun)[a-z]*';

function parseRecurrence(phrase: string): { rrule: string; label: string } | null {
  const p = phrase.toLowerCase().trim();
  const simple: Record<string, [string, string]> = {
    day: ['FREQ=DAILY', 'every day'],
    weekday: ['FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR', 'every weekday'],
    weekend: ['FREQ=WEEKLY;BYDAY=SA,SU', 'every weekend'],
    week: ['FREQ=WEEKLY', 'every week'],
    'other week': ['FREQ=WEEKLY;INTERVAL=2', 'every other week'],
    fortnight: ['FREQ=WEEKLY;INTERVAL=2', 'every fortnight'],
    month: ['FREQ=MONTHLY', 'every month'],
    year: ['FREQ=YEARLY', 'every year'],
  };
  const hit = simple[p];
  if (hit) return { rrule: hit[0], label: hit[1] };

  const n = /^(\d+)\s+(day|week|month|year)s?$/.exec(p);
  if (n) {
    const freq = { day: 'DAILY', week: 'WEEKLY', month: 'MONTHLY', year: 'YEARLY' }[n[2] as 'day'];
    return { rrule: `FREQ=${freq};INTERVAL=${n[1]}`, label: `every ${n[1]} ${n[2]}s` };
  }

  const days = p.split(/\s*(?:,|\band\b|&)\s*/).filter(Boolean);
  const codes: string[] = [];
  const names: string[] = [];
  for (const d of days) {
    const w = WEEKDAYS.find(([re]) => re.test(d));
    if (!w) return null;
    codes.push(w[1]);
    names.push(w[2]);
  }
  return codes.length
    ? { rrule: `FREQ=WEEKLY;BYDAY=${codes.join(',')}`, label: `every ${names.join(', ')}` }
    : null;
}

/** RRULE weekday codes from Sunday, as Date.getDay() counts. */
const RRULE_DAYS = ['SU', 'MO', 'TU', 'WE', 'TH', 'FR', 'SA'];
const fmtDay = (d: Date) => format(d, 'yyyy-MM-dd');
const fmtTime = (d: Date) => format(d, 'HH:mm');
const showDay = (d: Date) => format(d, 'EEE d MMM');

/** Parses one quick-add line relative to `now` (the user's local time). */
export function parseQuick(input: string, now: Date = new Date()): QuickParse {
  let rest = ` ${input} `;
  const chips: Chip[] = [];
  const out: QuickParse = { title: '', kind: 'task', anchoring: 'flexible', tags: [], chips };
  let anchoringSet: Anchoring | null = null;

  // Context first: everything after "//" is free text, so nothing in it is read
  // as a tag or a date. A "//" inside a word (https://…) doesn't count.
  const context = /\s\/\/(.*)$/.exec(rest);
  if (context) {
    rest = rest.slice(0, context.index) + ' ';
    const text = (context[1] as string).trim();
    if (text) {
      out.notes = text;
      chips.push({ kind: 'context', label: 'Context', value: text, source: '//' });
    }
  }

  // Removes a token from the text only when `fn` accepts it, so an
  // unreadable "~soon" stays part of the title.
  const take = (re: RegExp, fn: (m: RegExpExecArray) => boolean) => {
    const m = re.exec(rest);
    if (!m || !fn(m)) return;
    rest = rest.slice(0, m.index) + ' ' + rest.slice(m.index + m[0].length);
  };

  // "weekly sat" is "every sat"; "weekly" alone, "every week".
  rest = rest
    .replace(new RegExp(`\\sweekly(\\s+${DAY_WORD})(?=\\s)`, 'i'), ' every$1')
    .replace(/\sweekly(?=\s)/i, ' every week');

  // A usual time of day, "9-12" or "6pm–8pm", before chrono reads it as a date.
  take(
    /\s(\d{1,2}(?::\d{2})?\s*(?:am|pm)?)\s*[-–]\s*(\d{1,2}(?::\d{2})?\s*(?:am|pm)?)(?=\s)/i,
    (m) => {
      let from = parseTime(m[1] as string);
      const to = parseTime(m[2] as string);
      if (!from || !to) return false;
      // "1-3pm": the start takes the end's afternoon.
      if (/pm/i.test(m[2] as string) && !/[ap]m/i.test(m[1] as string) && from < '12:00') {
        const later = parseTime(`${m[1] as string}pm`);
        if (later && later < to) from = later;
      }
      if (to <= from) return false;
      out.windowStart = from;
      out.windowEnd = to;
      chips.push({ kind: 'window', label: 'Usually', value: `${from}–${to}`, source: m[0].trim() });
      return true;
    },
  );

  // Recurrence first: its weekday names would otherwise be read as dates.
  take(
    new RegExp(
      `\\severy\\s+((?:other\\s+week|fortnight|weekday|weekend|day|week|month|year)|\\d+\\s+(?:days?|weeks?|months?|years?)|${DAY_WORD}(?:\\s*(?:,|and|&)\\s*${DAY_WORD})*)(?=\\s)`,
      'i',
    ),
    (m) => {
      const r = parseRecurrence(m[1] as string);
      if (!r) return false;
      out.rrule = r.rrule;
      chips.push({ kind: 'repeat', label: 'Repeats', value: r.label, source: m[0].trim() });
      return true;
    },
  );
  take(/\s~(\S+)(?=\s)/, (m) => {
    const minutes = parseDuration(m[1] as string);
    if (!minutes) return false;
    out.durationMin = minutes;
    const h = Math.floor(minutes / 60);
    const mm = minutes % 60;
    chips.push({
      kind: 'duration',
      label: 'Duration',
      value: h ? `${h}h${mm ? ` ${mm}m` : ''}` : `${mm}m`,
      source: m[0].trim(),
    });
    return true;
  });
  take(/\s!([1-3])(?=\s)/, (m) => {
    out.priority = Number(m[1]);
    chips.push({ kind: 'priority', label: 'Priority', value: `!${m[1]}`, source: m[0].trim() });
    return true;
  });
  take(/\s@(fixed|flex|flexible)(?=\s)/i, (m) => {
    anchoringSet = (m[1] as string).toLowerCase() === 'fixed' ? 'fixed' : 'flexible';
    chips.push({ kind: 'anchoring', label: 'Anchoring', value: anchoringSet, source: m[0].trim() });
    return true;
  });
  for (let m = /\s#([\w-]+)(?=\s)/.exec(rest); m; m = /\s#([\w-]+)(?=\s)/.exec(rest)) {
    const tag = (m[1] as string).toLowerCase();
    if (!out.tags.includes(tag)) out.tags.push(tag);
    chips.push({ kind: 'tag', label: 'Tag', value: tag, source: m[0].trim() });
    rest = rest.slice(0, m.index) + ' ' + rest.slice(m.index + m[0].length);
  }

  // chrono reads "from 1 nov" as 1 o'clock; a "from" before something date-like
  // starts a do-window, so it is read as "between" (which chrono leaves alone).
  rest = rest.replace(
    /\bfrom(\s+)(?=(?:\d{1,2}(?:st|nd|rd|th)?\s+[a-z]{3}|\d{1,2}\/\d{1,2}|mon|tue|wed|thu|fri|sat|sun|next\b|tomorrow|today))/gi,
    'between$1',
  );

  // Dates and times. A match right after "due" is the deadline.
  // "weekend" on its own is almost always part of a title ("weekend
  // shopping"), not a date, so chrono's reading of it is ignored.
  const results = chrono.en.GB.parse(rest, now, { forwardDate: true }).filter(
    (r) => !/^(?:this\s+|the\s+)?weekend$/i.test(r.text.trim()),
  );
  const removals: [number, number][] = [];
  let when: chrono.ParsedResult | undefined;
  let windowEnds = false;
  for (const r of results) {
    const before = rest.slice(0, r.index);
    // The date after a window's start ("… to 15 nov") is its due date.
    const endWord = windowEnds ? /\b(?:to|and|until|till|-)\s*$/i.exec(before) : null;
    windowEnds = false;
    if (endWord && !out.due) {
      const e = r.start.date();
      out.due = `${fmtDay(e)}T17:00`;
      chips.push({
        kind: 'due',
        label: 'Due',
        value: showDay(e),
        source: `${endWord[0]}${r.text}`.trim(),
      });
      removals.push([endWord.index, r.index + r.text.length]);
      continue;
    }
    const dueWord = /\bdue\s*(?:on|by)?\s*$/i.exec(before);
    // "after 1 nov", "from 1 nov to 15 nov": a do-window (a range's end is the due date).
    const windowWord = /\b(?:after|between|not before)\s*$/i.exec(before);
    if (windowWord && !out.notBefore) {
      const d = r.start.date();
      out.notBefore = fmtDay(d);
      chips.push({
        kind: 'window',
        label: 'Not before',
        value: showDay(d),
        source: `${windowWord[0]}${r.text}`.trim(),
      });
      windowEnds = true;
      if (r.end && !out.due) {
        const e = r.end.date();
        out.due = `${fmtDay(e)}T17:00`;
        chips.push({ kind: 'due', label: 'Due', value: showDay(e), source: r.text });
      }
      removals.push([windowWord.index, r.index + r.text.length]);
    } else if (dueWord && !out.due) {
      const d = r.start.date();
      const time = r.start.isCertain('hour') ? fmtTime(d) : '17:00';
      out.due = `${fmtDay(d)}T${time}`;
      chips.push({
        kind: 'due',
        label: 'Due',
        value: `${showDay(d)}${r.start.isCertain('hour') ? ` ${time}` : ''}`,
        source: `${dueWord[0]}${r.text}`.trim(),
      });
      removals.push([dueWord.index, r.index + r.text.length]);
    } else if (!when) {
      when = r;
      removals.push([r.index, r.index + r.text.length]);
    }
  }
  for (const [a, b] of removals.sort((x, y) => y[0] - x[0]))
    rest = rest.slice(0, a) + ' ' + rest.slice(b);

  if (when) {
    const d = when.start.date();
    // A bare time ("9am") has already landed on today or tomorrow via forwardDate.
    const hasTime = when.start.isCertain('hour');
    out.date = fmtDay(d);
    chips.push({ kind: 'date', label: 'Date', value: showDay(d), source: when.text });
    if (hasTime) {
      out.time = fmtTime(d);
      chips.push({ kind: 'time', label: 'Time', value: out.time, source: when.text });
    }
  }

  // A usual time with a day but no time starts at the usual time.
  if (out.windowStart && out.date && !out.time) out.time = out.windowStart;

  // A recurring item needs a start: the first day its rule names (today if
  // it matches), else today.
  if (out.rrule && !out.date) {
    const codes = /BYDAY=([A-Z,]+)/.exec(out.rrule)?.[1]?.split(',') ?? [];
    const first = new Date(now);
    for (let i = 0; i < 7 && codes.length; i++) {
      if (codes.includes(RRULE_DAYS[first.getDay()] as string)) break;
      first.setDate(first.getDate() + 1);
    }
    out.date = fmtDay(codes.length ? first : now);
    // A usual time with no time given starts at the usual time.
    if (out.windowStart && !out.time) out.time = out.windowStart;
  }

  out.title = rest
    .replace(/\s+/g, ' ')
    .trim()
    .replace(/\s+(?:at|on|by|for|from|due)$/i, '')
    .replace(/^(?:at|on)\s+/i, '');

  const eventLike = Boolean(out.time && EVENT_WORDS.test(out.title));
  if (eventLike) out.kind = 'event';
  out.anchoring = anchoringSet ?? (eventLike ? 'fixed' : 'flexible');
  return out;
}

/** `create_item` arguments for a parsed line. */
export function toCreateArgs(p: QuickParse): Record<string, unknown> {
  const args: Record<string, unknown> = {
    title: p.title || 'Untitled',
    kind: p.kind,
    anchoring: p.anchoring,
  };
  if (p.date && p.time) args.start = `${p.date}T${p.time}`;
  else if (p.date) args.date = p.date;
  if (p.durationMin) args.duration_min = p.durationMin;
  if (p.priority) args.priority = p.priority;
  if (p.tags.length) args.tags = p.tags;
  if (p.due) args.due = p.due;
  if (p.rrule) args.rrule = p.rrule;
  if (p.notes) args.notes = p.notes;
  if (p.notBefore) args.not_before = p.notBefore;
  if (p.windowStart && p.windowEnd) {
    args.window_start = p.windowStart;
    args.window_end = p.windowEnd;
  }
  return args;
}

/** Where a parsed line will land, for the preview pill. */
export function destination(p: QuickParse): 'inbox' | 'day' | 'calendar' {
  if (p.date && p.time) return 'calendar';
  if (p.date) return 'day';
  return 'inbox';
}

/** One-line description of what will be created, as in the design preview. */
export function describeParse(p: QuickParse): string {
  const parts: string[] = [`${p.anchoring === 'fixed' ? 'Fixed' : 'Flexible'} ${p.kind}`];
  if (p.date) {
    const day = format(parseISO(p.date), 'EEE d MMM');
    if (p.time) {
      const [h, m] = p.time.split(':').map(Number);
      const end = (h ?? 0) * 60 + (m ?? 0) + (p.durationMin ?? (p.kind === 'event' ? 60 : 30));
      const endText = `${String(Math.floor(end / 60) % 24).padStart(2, '0')}:${String(end % 60).padStart(2, '0')}`;
      parts.push(`${day} ${p.time}–${endText}`);
    } else {
      parts.push(day);
      if (p.durationMin) parts.push(formatDuration(p.durationMin));
    }
  } else if (p.durationMin) {
    parts.push(formatDuration(p.durationMin));
  }
  const repeat = p.chips.find((c) => c.kind === 'repeat');
  if (repeat) parts.push(repeat.value);
  if (p.priority) parts.push(`!${p.priority}`);
  if (p.windowStart) parts.push(`usually ${p.windowStart}–${p.windowEnd}`);
  if (p.notBefore) parts.push(`from ${format(parseISO(p.notBefore), 'EEE d MMM')}`);
  if (p.due) parts.push(`due ${format(parseISO(p.due), 'EEE d MMM')}`);
  if (p.notes) parts.push(`// ${p.notes.length > 40 ? `${p.notes.slice(0, 39)}…` : p.notes}`);
  return parts.join(' · ');
}
