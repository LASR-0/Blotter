import { useEffect } from 'react';
import { midnight, useNav } from './nav';

/** How often the date is looked at while Blotter stays open. */
const CHECK_MS = 60_000;

/**
 * Keeps "today" current while Blotter stays open past midnight (or wakes
 * from sleep on a later day): the anchor moves on with the date, so the
 * Diary and its timeline don't stay on yesterday until a restart.
 */
export function useNewDay() {
  useEffect(() => {
    let today = midnight();
    const check = () => {
      const now = midnight();
      if (now.getTime() !== today.getTime()) {
        useNav.getState().newDay(today, now);
        today = now;
      }
    };
    const timer = window.setInterval(check, CHECK_MS);
    const onVisible = () => {
      if (document.visibilityState === 'visible') check();
    };
    window.addEventListener('focus', check);
    document.addEventListener('visibilitychange', onVisible);
    return () => {
      window.clearInterval(timer);
      window.removeEventListener('focus', check);
      document.removeEventListener('visibilitychange', onVisible);
    };
  }, []);
}
