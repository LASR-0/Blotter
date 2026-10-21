import { describe, expect, it } from 'vitest';
import { placeName, regionLabel } from './publicHolidays';

describe('public holiday places', () => {
  const countries = [{ countryCode: 'AU', name: 'Australia' }];

  it('names the region code and the country', () => {
    expect(regionLabel('AU-VIC')).toBe('VIC');
    expect(placeName({ country: 'AU', region: 'AU-VIC' }, countries)).toBe('VIC, Australia');
    expect(placeName({ country: 'AU', region: null }, countries)).toBe('Australia');
  });

  it('falls back to the code before the country list loads', () => {
    expect(placeName({ country: 'DE', region: null })).toBe('DE');
  });
});
