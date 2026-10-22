-- Sync (Blotter 1.2.1). Which device a change-log row came from: null for
-- this device, or the id of the device whose change a merge applied. Undo
-- and redo only ever reverse this device's own batches.
ALTER TABLE changes ADD COLUMN origin TEXT;

-- The other devices syncing through the same server, by the names the
-- server knows them by (device-local, refreshed by the app).
CREATE TABLE sync_peers (
    device_id TEXT PRIMARY KEY NOT NULL,
    name      TEXT NOT NULL
);
