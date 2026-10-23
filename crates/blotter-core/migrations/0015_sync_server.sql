-- Sync (Blotter 1.2.1). The home server this device syncs with, device-local
-- and kept across a snapshot restore. No row means Local only. The token
-- lives in the OS keychain, never here.
CREATE TABLE sync_server (
    id           INTEGER PRIMARY KEY CHECK (id = 1),
    server_id    TEXT NOT NULL,
    -- JSON array of base addresses, tried in order.
    addresses    TEXT NOT NULL DEFAULT '[]',
    -- The epoch last bootstrapped from, and the last seq applied from it.
    epoch        TEXT NOT NULL,
    seq          INTEGER NOT NULL DEFAULT 0,
    paired_at    TEXT NOT NULL,
    last_sync_at TEXT
);
