import { describe, expect, it } from 'vitest';
import { addDays, subDays } from 'date-fns';
import { fileSize, takenAt } from './backupFormat';

describe('takenAt', () => {
  it('says today and yesterday, else the date', () => {
    const now = new Date();
    now.setHours(10, 41, 0, 0);
    expect(takenAt(now.toISOString())).toBe('Today 10:41');
    expect(takenAt(subDays(now, 1).toISOString())).toBe('Yesterday 10:41');
    expect(takenAt(new Date(2026, 9, 12, 20, 14).toISOString())).toMatch(/^Mon 12 Oct 20:14$/);
    expect(takenAt(addDays(now, 0).toISOString())).toMatch(/^Today/);
  });
});

describe('fileSize', () => {
  it('rounds to KB or one-decimal MB', () => {
    expect(fileSize(null)).toBe('—');
    expect(fileSize(200)).toBe('1 KB');
    expect(fileSize(840 * 1024)).toBe('840 KB');
    expect(fileSize(2.14 * 1024 * 1024)).toBe('2.1 MB');
  });
});
