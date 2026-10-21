import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useQuery } from '@tanstack/react-query';
import { isOpError } from './api';

/** Where imported public holidays come from (`get_public_holidays`). */
export interface PublicConfig {
  country: string;
  region: string | null;
  keep_updated: boolean;
  through_year: number;
}

export interface Country {
  countryCode: string;
  name: string;
}

interface Guess {
  country: string;
  region: string | null;
}

/** What `import_public_holidays` did. */
export interface Imported {
  added: number;
  removed: number;
  config: PublicConfig;
}

/** The region choice for national holidays only. */
export const NATIONAL = 'national';

/** `AU-VIC` → `VIC`. */
export const regionLabel = (code: string) => code.replace(/^[A-Z]{2}-/, '');

const errorText = (e: unknown) => (isOpError(e) ? e.message : String(e));

export function useCountries() {
  return useQuery({
    queryKey: ['public-holiday-countries'],
    queryFn: () => invoke<Country[]>('public_holiday_countries'),
    staleTime: Infinity,
    retry: false,
  });
}

/** Region names aren't known offline, so "VIC, Australia". */
export function placeName(
  config: Pick<PublicConfig, 'country' | 'region'>,
  countries?: readonly Country[],
): string {
  const country = countries?.find((c) => c.countryCode === config.country)?.name ?? config.country;
  return config.region ? `${regionLabel(config.region)}, ${country}` : country;
}

/**
 * The country and region choice, starting on a guess from the time zone,
 * and `run()`, which imports this year's and next year's public holidays
 * (from today on) for it.
 */
export function usePublicHolidayChoice(initial?: Pick<PublicConfig, 'country' | 'region'> | null) {
  const countries = useCountries();
  const guess = useQuery({
    queryKey: ['public-holiday-guess'],
    queryFn: () => invoke<Guess | null>('public_holiday_guess'),
    staleTime: Infinity,
    enabled: !initial,
  });
  // Until the user picks, the choice is the earlier one or the guess.
  const [chosen, setChosen] = useState<{ country: string; region: string } | null>(
    initial ? { country: initial.country, region: initial.region ?? '' } : null,
  );
  const country = chosen?.country ?? guess.data?.country ?? '';
  const region = chosen ? chosen.region : (guess.data?.region ?? '');

  const regions = useQuery({
    queryKey: ['public-holiday-regions', country],
    queryFn: () => invoke<string[]>('public_holiday_regions', { country }),
    enabled: country !== '',
    staleTime: Infinity,
    retry: false,
  });
  const regionList = regions.data ?? [];
  // No regions listed: national. A region the list lacks: pick again.
  const regionValue = !regionList.length ? NATIONAL : regionList.includes(region) ? region : '';
  const loadError = countries.error ?? regions.error;

  return {
    countries: countries.data ?? [],
    countriesLoading: countries.isLoading,
    regionList,
    country,
    regionValue,
    setCountry: (c: string) => setChosen({ country: c, region: '' }),
    setRegion: (r: string) => setChosen({ country, region: r }),
    ready: country !== '' && regions.isSuccess && regionValue !== '',
    loadError: loadError ? errorText(loadError) : null,
    run: () =>
      invoke<Imported>('import_public_holidays', {
        country,
        region: regionValue === NATIONAL ? null : regionValue,
      }),
  };
}

export type PublicHolidayChoice = ReturnType<typeof usePublicHolidayChoice>;
