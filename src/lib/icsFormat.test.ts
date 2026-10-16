import { describe, expect, it } from 'vitest';
import {
  exportedText,
  importNotes,
  importSpan,
  importWhat,
  refreshLabel,
  subscriptionStatus,
  type IcsSummary,
  type Subscription,
} from './icsFormat';

const base: IcsSummary = {
  scheduled: 0,
  inbox: 0,
  recurring: 0,
  occurrences: 0,
  reminders: 0,
  already: 0,
  skipped: 0,
  first: null,
  last: null,
  warnings: [],
};

describe('ICS import wording', () => {
  it('names what the file adds', () => {
    expect(importWhat({ ...base, scheduled: 12, inbox: 1 })).toBe(
      '12 calendar items and 1 Inbox task',
    );
    expect(importWhat({ ...base, scheduled: 1 })).toBe('1 calendar item');
    expect(importWhat({ ...base, occurrences: 2 })).toBe('2 changes to repeating items');
    expect(importWhat({ ...base, already: 4 })).toBe('nothing new');
  });

  it('gives the span, with years only when needed', () => {
    const today = new Date(2026, 9, 16);
    expect(importSpan({ ...base, first: '2026-10-20', last: '2026-10-20' }, today)).toBe(
      'on Tue 20 Oct',
    );
    expect(importSpan({ ...base, first: '2026-10-12', last: '2026-10-30' }, today)).toBe(
      'from Mon 12 Oct to Fri 30 Oct',
    );
    expect(importSpan({ ...base, first: '2026-12-28', last: '2027-01-04' }, today)).toBe(
      'from Mon 28 Dec 2026 to Mon 4 Jan 2027',
    );
    expect(importSpan(base, today)).toBe('');
  });

  it('lists repeats, reminders, what was left out and warnings', () => {
    expect(
      importNotes({
        ...base,
        recurring: 1,
        reminders: 3,
        already: 2,
        skipped: 1,
        warnings: ['Unknown time zone X.'],
      }),
    ).toEqual([
      '1 item repeats.',
      '3 reminders.',
      '2 items are already in Blotter and left out.',
      '1 cancelled or undated item was left out.',
      'Unknown time zone X.',
    ]);
  });

  it('reports an export', () => {
    expect(exportedText(5, 1, 'C:\\b.ics')).toBe(
      'Exported 5 calendar items and 1 Inbox task to C:\\b.ics',
    );
    expect(exportedText(1, 0, '/b.ics')).toBe('Exported 1 calendar item to /b.ics');
  });
});

describe('subscription wording', () => {
  const now = new Date(2026, 9, 16, 11, 0);
  const sub: Subscription = {
    id: 's',
    name: 'Work',
    url: 'https://outlook.office365.com/c.ics',
    refresh_min: 60,
    color: null,
    as_blockout: true,
    enabled: true,
    last_fetch_at: null,
    last_ok_at: null,
    last_status: null,
    last_error: null,
    created_at: '2026-10-16T00:00:00Z',
    updated_at: '2026-10-16T00:00:00Z',
    events: 0,
  };
  const local = (d: number, h: number, m = 0) => new Date(2026, 9, d, h, m).toISOString();

  it('labels refresh intervals', () => {
    expect([15, 60, 180, 1440, 2880, 90].map(refreshLabel)).toEqual([
      '15 min',
      '1 hour',
      '3 hours',
      'Daily',
      '2 days',
      '90 min',
    ]);
  });

  it('says when it last fetched', () => {
    expect(subscriptionStatus(sub, now)).toEqual({ tone: 'pending', text: 'Not fetched yet' });
    const ok = { ...sub, last_fetch_at: local(16, 10, 30), last_status: 'ok' as const, events: 48 };
    expect(subscriptionStatus(ok, now).text).toBe('Fetched 10:30 · 48 events');
    expect(subscriptionStatus({ ...ok, last_fetch_at: local(15, 9, 5) }, now).text).toBe(
      'Fetched yesterday 09:05 · 48 events',
    );
    expect(subscriptionStatus({ ...ok, enabled: false }, now).tone).toBe('off');
  });

  it('explains failures and the last success', () => {
    const failed = {
      ...sub,
      last_fetch_at: local(16, 10),
      last_ok_at: local(13, 10),
      last_status: 'error' as const,
      last_error: 'HTTP 404 Not Found',
    };
    expect(subscriptionStatus(failed, now)).toEqual({
      tone: 'error',
      text: 'HTTP 404 Not Found · last success 3 days ago',
    });
    expect(
      subscriptionStatus({ ...failed, last_ok_at: null, last_error: 'couldn’t connect' }, now).text,
    ).toBe('Couldn’t connect · never fetched');
  });
});
