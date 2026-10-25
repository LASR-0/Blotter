import { describe, expect, it } from 'vitest';
import {
  moveAddress,
  needsPairing,
  routeLabel,
  syncSummary,
  syncedAgo,
  type SyncStatus,
} from './syncFormat';

const NOW = new Date('2026-10-26T09:00:00Z');
const status = (s: Partial<SyncStatus> & Pick<SyncStatus, 'state'>) =>
  ({ unsent: 0, last_sync_at: null, ...s }) as SyncStatus;

describe('routeLabel', () => {
  it('names Tailscale, the local network, or the host', () => {
    expect(routeLabel('https://blotter.tail1234.ts.net')).toBe('Tailscale');
    expect(routeLabel('http://100.101.102.103:7458')).toBe('Tailscale');
    expect(routeLabel('http://[fd7a:115c:a1e0::1]:7458')).toBe('Tailscale');
    expect(routeLabel('http://192.168.1.10:7458')).toBe('your network');
    expect(routeLabel('http://172.20.0.1:7458')).toBe('your network');
    expect(routeLabel('http://nas.local:7458')).toBe('your network');
    expect(routeLabel('http://nas:7458')).toBe('your network');
    expect(routeLabel('http://[fd00::1]:7458')).toBe('your network');
    expect(routeLabel('https://sync.example.com')).toBe('sync.example.com');
    expect(routeLabel('http://100.128.0.1:7458')).toBe('100.128.0.1');
  });
});

describe('syncedAgo', () => {
  it('counts up from just now', () => {
    expect(syncedAgo('2026-10-26T08:59:30Z', NOW)).toBe('just now');
    expect(syncedAgo('2026-10-26T08:58:00Z', NOW)).toBe('2 min ago');
    expect(syncedAgo('2026-10-26T06:00:00Z', NOW)).toBe('3 h ago');
    expect(syncedAgo('2026-10-25T08:00:00Z', NOW)).toBe('yesterday');
    expect(syncedAgo('2026-10-22T09:00:00Z', NOW)).toBe('4 days ago');
  });
});

describe('syncSummary', () => {
  it('reads as the roadmap words it', () => {
    expect(
      syncSummary(
        status({
          state: 'synced',
          address: 'https://blotter.tail1234.ts.net',
          tailscale: true,
          server_version: '1.0.0',
          last_sync_at: '2026-10-26T08:58:00Z',
        }),
        NOW,
      ),
    ).toEqual({ tone: 'ok', text: 'Synced via Tailscale · 2 min ago' });
    expect(syncSummary(status({ state: 'local_only' })).text).toBe('Local only');
    expect(syncSummary(status({ state: 'syncing', unsent: 1 })).text).toBe(
      'Syncing… · 1 unsent change',
    );
  });

  it('says what to do when offline or stuck', () => {
    const off = syncSummary(
      status({ state: 'offline', tailscale: false, tried: [], retry_in_s: 90, unsent: 3 }),
    );
    expect(off.text).toBe('Offline · 3 unsent changes');
    expect(off.detail).toContain('Trying again in 2 min');
    const tailnet = syncSummary(
      status({ state: 'offline', tailscale: true, tried: [], retry_in_s: 4 }),
    );
    expect(tailnet.detail).toContain('Is Tailscale connected');

    const newer = status({
      state: 'attention',
      reason: 'update_blotter',
      server: { protocol: 1, schema: 2 },
    });
    expect(syncSummary(newer).text).toBe('Update Blotter');
    const older = status({
      state: 'attention',
      reason: 'update_server',
      server: { protocol: 0, schema: 1 },
    });
    expect(syncSummary(older).text).toBe('Update Blotter Sync');
  });
});

describe('needsPairing', () => {
  it('is for a revoked or unknown token only', () => {
    expect(needsPairing(status({ state: 'attention', reason: 'revoked' }))).toBe(true);
    expect(needsPairing(status({ state: 'attention', reason: 'unauthorized' }))).toBe(true);
    expect(needsPairing(status({ state: 'attention', reason: 'rebootstrap' }))).toBe(false);
    expect(needsPairing(status({ state: 'syncing' }))).toBe(false);
    expect(needsPairing(null)).toBe(false);
  });
});

describe('moveAddress', () => {
  it('swaps neighbours and stops at the ends', () => {
    expect(moveAddress(['a', 'b', 'c'], 2, -1)).toEqual(['a', 'c', 'b']);
    expect(moveAddress(['a', 'b', 'c'], 0, 1)).toEqual(['b', 'a', 'c']);
    expect(moveAddress(['a', 'b'], 0, -1)).toEqual(['a', 'b']);
    expect(moveAddress(['a', 'b'], 1, 1)).toEqual(['a', 'b']);
  });
});
