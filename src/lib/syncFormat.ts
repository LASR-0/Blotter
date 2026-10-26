// Settings → Sync's wording: where syncing goes, how long ago, and what
// each status means. Mirrors `SyncStatus` and `pairing` in src-tauri/src/sync.

export interface Versions {
  protocol: number;
  schema: number;
}

export type Attention =
  | { reason: 'update_blotter'; server: Versions }
  | { reason: 'update_server'; server: Versions }
  | { reason: 'revoked' }
  | { reason: 'unauthorized' }
  | { reason: 'rebootstrap' }
  | { reason: 'wrong_server' };

export type SyncState =
  | { state: 'local_only' }
  | { state: 'syncing' }
  | { state: 'synced'; address: string; tailscale: boolean; server_version: string | null }
  | { state: 'offline'; tailscale: boolean; tried: [string, string][]; retry_in_s: number }
  | ({ state: 'attention' } & Attention);

export type SyncStatus = SyncState & {
  /** Local changes the server doesn't have yet. */
  unsent: number;
  last_sync_at: string | null;
};

export interface SyncInfo {
  device_name: string | null;
  server: { server_id: string; addresses: string[]; paired_at: string } | null;
  /** A pairing waiting on Use the server's planner or Merge: its address. */
  joining: string | null;
}

export interface AddressCheck {
  address: string;
  ok: boolean;
  message: string;
}

export interface Disconnected {
  revoked: boolean;
}

/**
 * What pairing did with the planners: this device's went up to an empty
 * server, it took the server's, it paired again, or both have one and the
 * user chooses.
 */
export type Joined = 'seeded' | 'fetched' | 'again' | 'choose';

export type JoinChoiceValue = 'server' | 'merge';

/** The server's planner was replaced (restored or imported there) and fetched again. */
export interface SyncNotice {
  snapshot_id: string;
  unsent: number;
}

export function replacedText(n: SyncNotice): string {
  const kept =
    n.unsent > 0
      ? ` It had ${unsentText(n.unsent)} the server never got; they’re in the snapshot in Backups.`
      : ' This device’s planner from before is in Backups.';
  return `The home server’s planner was restored or imported there, so Blotter fetched it again.${kept}`;
}

function hostOf(address: string): string {
  try {
    return new URL(address).hostname.replace(/^\[|\]$/g, '').toLowerCase();
  } catch {
    return address;
  }
}

function v4(host: string): [number, number, number, number] | null {
  const m = /^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$/.exec(host);
  return m ? [Number(m[1]), Number(m[2]), Number(m[3]), Number(m[4])] : null;
}

/** Tailscale: a `*.ts.net` name, 100.64.0.0/10 or fd7a:115c:a1e0::/48. */
function isTailscale(host: string): boolean {
  const ip = v4(host);
  if (ip) return ip[0] === 100 && ip[1] >= 64 && ip[1] < 128;
  return host.endsWith('.ts.net') || host.startsWith('fd7a:115c:a1e0:');
}

/** On the local network, as far as the address alone tells. */
function isLocal(host: string): boolean {
  const ip = v4(host);
  if (ip) {
    const [a, b] = ip;
    return (
      a === 10 ||
      a === 127 ||
      (a === 172 && b >= 16 && b < 32) ||
      (a === 192 && b === 168) ||
      (a === 169 && b === 254)
    );
  }
  return (
    host === 'localhost' ||
    host === '::1' ||
    /^f[cd][0-9a-f]{2}:/.test(host) ||
    host.startsWith('fe80:') ||
    /\.(local|lan|home\.arpa|internal)$/.test(host) ||
    !host.includes('.')
  );
}

/** How an address reaches the server: `Tailscale`, `your network`, or its host name. */
export function routeLabel(address: string): string {
  const host = hostOf(address);
  if (isTailscale(host)) return 'Tailscale';
  if (isLocal(host)) return 'your network';
  return host;
}

/** `just now`, `5 min ago`, `3 h ago`, `yesterday`, `4 days ago`. */
export function syncedAgo(iso: string, now: Date = new Date()): string {
  const min = Math.floor((now.getTime() - new Date(iso).getTime()) / 60_000);
  if (min < 1) return 'just now';
  if (min < 60) return `${min} min ago`;
  const h = Math.floor(min / 60);
  if (h < 24) return `${h} h ago`;
  const days = Math.floor(h / 24);
  return days === 1 ? 'yesterday' : `${days} days ago`;
}

export function unsentText(n: number): string {
  return n === 1 ? '1 unsent change' : `${n} unsent changes`;
}

export type SyncTone = 'off' | 'busy' | 'ok' | 'offline' | 'attention';

export interface SyncSummary {
  tone: SyncTone;
  text: string;
  /** What to do about it, when there's something to do. */
  detail?: string;
}

const ATTENTION: Record<Attention['reason'], SyncSummary> = {
  update_blotter: {
    tone: 'attention',
    text: 'Update Blotter',
    detail: 'The server syncs a newer version than this Blotter. Update Blotter on this device.',
  },
  update_server: {
    tone: 'attention',
    text: 'Update Blotter Sync',
    detail:
      'The server runs an older Blotter Sync than this Blotter needs. Update it on the server.',
  },
  revoked: {
    tone: 'attention',
    text: 'Disconnected by the server',
    detail: 'This device was revoked on the server. Pair again to keep syncing.',
  },
  unauthorized: {
    tone: 'attention',
    text: 'Pair again',
    detail: "The server doesn't recognise this device any more. Pair again to keep syncing.",
  },
  rebootstrap: {
    tone: 'attention',
    text: "The server's planner was replaced",
    detail: 'It was restored or imported on the server, so this device needs to fetch it again.',
  },
  wrong_server: {
    tone: 'attention',
    text: 'A different server answers',
    detail:
      'Every address reaches a Blotter Sync server other than the one this device paired with. Check the addresses.',
  },
};

/** One line for the status, plus what to do about it. */
export function syncSummary(status: SyncStatus, now: Date = new Date()): SyncSummary {
  const unsent = status.unsent > 0 ? ` · ${unsentText(status.unsent)}` : '';
  switch (status.state) {
    case 'local_only':
      return { tone: 'off', text: 'Local only' };
    case 'syncing':
      return { tone: 'busy', text: `Syncing…${unsent}` };
    case 'synced': {
      const when = status.last_sync_at ? ` · ${syncedAgo(status.last_sync_at, now)}` : '';
      return { tone: 'ok', text: `Synced via ${routeLabel(status.address)}${when}${unsent}` };
    }
    case 'offline':
      return {
        tone: 'offline',
        text: `Offline${unsent}`,
        detail: status.tailscale
          ? "Can't reach the server. Is Tailscale connected on this device?"
          : `Can't reach the server. Trying again in ${retryText(status.retry_in_s)}; changes wait here until then.`,
      };
    case 'attention':
      return ATTENTION[status.reason];
  }
}

function retryText(s: number): string {
  return s < 60 ? `${Math.max(1, s)} s` : `${Math.round(s / 60)} min`;
}

/** Statuses the Pair form fixes. */
export function needsPairing(status: SyncStatus | null | undefined): boolean {
  return status?.state === 'attention' && ['revoked', 'unauthorized'].includes(status.reason);
}

/** `list` with the entry at `i` moved by `delta`, or unchanged at either end. */
export function moveAddress(list: readonly string[], i: number, delta: -1 | 1): string[] {
  const j = i + delta;
  const out = [...list];
  if (i < 0 || i >= out.length || j < 0 || j >= out.length) return out;
  const [moved] = out.splice(i, 1);
  if (moved !== undefined) out.splice(j, 0, moved);
  return out;
}
