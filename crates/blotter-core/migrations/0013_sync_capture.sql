-- Change capture for sync (Blotter 1.2.1). Device-local, like 0012's tables.
-- The capture triggers themselves are generated from the sync registry and
-- installed when the database opens (sync::capture), not here.

-- Capture state: this replica's role ('device', or 'server' inside Blotter
-- Sync, which also captures server-owned data), whether remote changes are
-- being applied (set and cleared inside one transaction, so the triggers
-- skip those writes), and the fingerprint of the installed triggers.
CREATE TABLE sync_state (
    id       INTEGER PRIMARY KEY CHECK (id = 1),
    role     TEXT NOT NULL DEFAULT 'device' CHECK (role IN ('device', 'server')),
    applying INTEGER NOT NULL DEFAULT 0 CHECK (applying IN (0, 1)),
    triggers TEXT
);
INSERT INTO sync_state (id) VALUES (1);

-- The stamp that last set each field group of each synced row, and, under
-- the group '', when the row was last created or deleted (deleted = 1 is a
-- tombstone). pending = 1 marks local changes the server hasn't
-- acknowledged yet: together they are the outbox.
CREATE TABLE sync_clocks (
    tbl     TEXT NOT NULL,
    row_key TEXT NOT NULL,
    grp     TEXT NOT NULL,
    hlc     TEXT NOT NULL,
    deleted INTEGER NOT NULL DEFAULT 0 CHECK (deleted IN (0, 1)),
    pending INTEGER NOT NULL DEFAULT 0 CHECK (pending IN (0, 1)),
    PRIMARY KEY (tbl, row_key, grp)
) WITHOUT ROWID;
CREATE INDEX sync_clocks_pending ON sync_clocks (hlc) WHERE pending = 1;

-- The clock's current stamp as text, for the triggers (sync::hlc's format).
CREATE VIEW sync_now AS
SELECT printf('%015d.%06d.%s', c.wall_ms, c.counter, d.device_id) AS hlc
FROM sync_clock c, sync_device d
WHERE c.id = 1 AND d.id = 1;
