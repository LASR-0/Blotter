import { describe, expect, it } from 'vitest';
import { describeRule, presetOf, ruleFor, withEnd } from './repeat';

// Wednesday 14 October 2026.
const wed = '2026-10-14';

describe('ruleFor / presetOf', () => {
  it('builds rules from the item date', () => {
    expect(ruleFor('weekly', wed)).toBe('FREQ=WEEKLY;BYDAY=WE');
    expect(ruleFor('fortnightly', wed)).toBe('FREQ=WEEKLY;INTERVAL=2;BYDAY=WE');
    expect(ruleFor('monthly', wed)).toBe('FREQ=MONTHLY;BYMONTHDAY=14');
    expect(ruleFor('none', wed)).toBeNull();
  });

  it('recognises presets, ignoring order, case and how the series ends', () => {
    expect(presetOf('FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR', wed)).toBe('weekdays');
    expect(presetOf('byday=we;freq=weekly;count=5', wed)).toBe('weekly');
    expect(presetOf('FREQ=WEEKLY;BYDAY=MO', wed)).toBe('custom');
    expect(presetOf(null, wed)).toBe('none');
  });

  it('keeps the end when switching', () => {
    expect(withEnd('FREQ=DAILY', 'FREQ=WEEKLY;BYDAY=WE;COUNT=4')).toBe('FREQ=DAILY;COUNT=4');
    expect(withEnd('FREQ=DAILY', null)).toBe('FREQ=DAILY');
  });
});

describe('describeRule', () => {
  it('reads like the design', () => {
    expect(describeRule('FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR')).toBe('Every weekday');
    expect(describeRule('FREQ=WEEKLY;INTERVAL=2;BYDAY=MO')).toBe('Every 2 weeks on Monday');
    expect(describeRule('FREQ=MONTHLY;BYMONTHDAY=1;COUNT=6')).toBe('Monthly on the 1st, 6 times');
    expect(describeRule('FREQ=DAILY;UNTIL=20261031T000000Z')).toBe('Every day, until 31 Oct 2026');
    expect(describeRule(null)).toBe('Does not repeat');
    expect(describeRule('FREQ=SECONDLY')).toBe('FREQ=SECONDLY');
  });
});
