-- Sync groundwork (Blotter 1.2.1). Both tables are device-local: they never
-- sync, and a JSON import or snapshot restore leaves them as they were.

-- This install's identity. One row, made on first open (the id is a ULID,
-- generated in Rust); the name is what the user calls the device.
CREATE TABLE sync_device (
    id         INTEGER PRIMARY KEY CHECK (id = 1),
    device_id  TEXT NOT NULL,
    name       TEXT,
    created_at TEXT NOT NULL
);

-- This device's hybrid logical clock: the last stamp it handed out or saw.
CREATE TABLE sync_clock (
    id      INTEGER PRIMARY KEY CHECK (id = 1),
    wall_ms INTEGER NOT NULL CHECK (wall_ms >= 0),
    counter INTEGER NOT NULL CHECK (counter >= 0)
);
INSERT INTO sync_clock (id, wall_ms, counter) VALUES (1, 0, 0);
