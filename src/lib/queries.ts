import { useEffect } from 'react';
import { QueryClient, useQuery, type UseQueryResult } from '@tanstack/react-query';
import { onPlannerChanged, runOp } from './api';

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // Data only changes through operations, which emit planner:changed;
      // that event invalidates everything, so nothing goes stale by time.
      staleTime: Infinity,
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});

const OP_KEY = 'op';

/** Runs a read operation as a cached query keyed by op name and args. */
export function useOp<T>(
  name: string,
  args?: Record<string, unknown>,
  options: { enabled?: boolean } = {},
): UseQueryResult<T> {
  return useQuery({
    queryKey: [OP_KEY, name, args ?? {}],
    queryFn: () => runOp<T>(name, args),
    enabled: options.enabled,
  });
}

/** Refetches every op query after any committed mutation, from any actor. */
export function usePlannerSync(): void {
  useEffect(() => {
    const unlisten = onPlannerChanged(() => {
      void queryClient.invalidateQueries({ queryKey: [OP_KEY] });
    });
    return () => {
      void unlisten.then((stop) => stop());
    };
  }, []);
}
