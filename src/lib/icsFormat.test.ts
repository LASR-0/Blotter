import { describe, expect, it } from 'vitest';
import { exportedText, importNotes, importSpan, importWhat, type IcsSummary } from './icsFormat';

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
