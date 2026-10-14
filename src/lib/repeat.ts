// Repeat presets for the item dialog's "Repeats" picker, and readable
// descriptions of RRULEs. Anything the presets can't express stays as a
// custom rule, shown and edited as text.

import { format, parseISO } from 'date-fns';

export type RepeatPreset =
  'none' | 'daily' | 'weekdays' | 'weekly' | 'fortnightly' | 'monthly' | 'yearly' | 'custom';

const BYDAY = ['SU', 'MO', 'TU', 'WE', 'TH', 'FR', 'SA'] as const;
const DAY_NAMES: Record<string, string> = {
  MO: 'Monday',
  TU: 'Tuesday',
  WE: 'Wednesday',
  TH: 'Thursday',
  FR: 'Friday',
  SA: 'Saturday',
  SU: 'Sunday',
};

const day = (date: string) => (date ? parseISO(date) : new Date());

/** The rule a preset means for an item starting on `date` (`YYYY-MM-DD`). */
export function ruleFor(preset: RepeatPreset, date: string): string | null {
  const d = day(date);
  switch (preset) {
    case 'none':
    case 'custom':
      return null;
    case 'daily':
      return 'FREQ=DAILY';
    case 'weekdays':
      return 'FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR';
    case 'weekly':
      return `FREQ=WEEKLY;BYDAY=${BYDAY[d.getDay()]}`;
    case 'fortnightly':
      return `FREQ=WEEKLY;INTERVAL=2;BYDAY=${BYDAY[d.getDay()]}`;
    case 'monthly':
      return `FREQ=MONTHLY;BYMONTHDAY=${d.getDate()}`;
    case 'yearly':
      return 'FREQ=YEARLY';
  }
}

/** Splits a rule into its parts, e.g. { FREQ: 'WEEKLY', BYDAY: 'MO' }. */
function parts(rule: string): Record<string, string> {
  const out: Record<string, string> = {};
  for (const p of rule
    .toUpperCase()
    .replace(/^RRULE:/, '')
    .split(';')) {
    const [k, v] = p.split('=');
    if (k && v) out[k.trim()] = v.trim();
  }
  return out;
}

/** Which preset a rule matches for an item on `date`, or `custom`. */
export function presetOf(rule: string | null | undefined, date: string): RepeatPreset {
  if (!rule) return 'none';
  const norm = (r: string) => {
    const p = parts(r);
    return Object.keys(p)
      .sort()
      .map((k) => `${k}=${p[k]}`)
      .join(';');
  };
  const presets: RepeatPreset[] = [
    'daily',
    'weekdays',
    'weekly',
    'fortnightly',
    'monthly',
    'yearly',
  ];
  // Ignore how the series ends when matching; that's kept separately.
  const body = (r: string) =>
    norm(
      r
        .split(';')
        .filter((p) => !/^(COUNT|UNTIL)=/i.test(p.trim()))
        .join(';'),
    );
  return presets.find((p) => body(ruleFor(p, date) ?? '') === body(rule)) ?? 'custom';
}

/** Keeps a rule's COUNT/UNTIL when switching presets. */
export function withEnd(rule: string, previous: string | null | undefined): string {
  const end = (previous ?? '').split(';').filter((p) => /^(COUNT|UNTIL)=/i.test(p.trim()));
  return [rule, ...end].join(';');
}

const ordinal = (n: number) => {
  const s = ['th', 'st', 'nd', 'rd'];
  const v = n % 100;
  return `${n}${s[(v - 20) % 10] ?? s[v] ?? s[0]}`;
};

/** A readable rule: "Every weekday", "Every 2 weeks on Monday", "Monthly on the 15th, 6 times". */
export function describeRule(rule: string | null | undefined): string {
  if (!rule) return 'Does not repeat';
  const p = parts(rule);
  const interval = Number(p.INTERVAL ?? '1');
  const days = (p.BYDAY ?? '').split(',').filter(Boolean);
  let text: string;
  switch (p.FREQ) {
    case 'DAILY':
      text = interval > 1 ? `Every ${interval} days` : 'Every day';
      break;
    case 'WEEKLY': {
      const names = days.map((d) => DAY_NAMES[d] ?? d);
      const weekdays = ['MO', 'TU', 'WE', 'TH', 'FR'];
      if (interval === 1 && days.length === 5 && weekdays.every((d) => days.includes(d))) {
        text = 'Every weekday';
      } else {
        const on = names.length ? ` on ${names.join(', ')}` : '';
        text = interval > 1 ? `Every ${interval} weeks${on}` : `Every week${on}`;
      }
      break;
    }
    case 'MONTHLY':
      text =
        (interval > 1 ? `Every ${interval} months` : 'Monthly') +
        (p.BYMONTHDAY ? ` on the ${ordinal(Number(p.BYMONTHDAY))}` : '');
      break;
    case 'YEARLY':
      text = interval > 1 ? `Every ${interval} years` : 'Yearly';
      break;
    default:
      return rule;
  }
  if (p.COUNT) text += `, ${p.COUNT} times`;
  if (p.UNTIL) {
    const m = /^(\d{4})(\d{2})(\d{2})/.exec(p.UNTIL);
    if (m) text += `, until ${format(parseISO(`${m[1]}-${m[2]}-${m[3]}`), 'd MMM yyyy')}`;
  }
  return text;
}
