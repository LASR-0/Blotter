-- Undo and redo write their own change rows (op 'undo' / 'redo'); each
-- names the batch it reverses, so redo can find what an undo undid.
ALTER TABLE changes ADD COLUMN reverts TEXT;
CREATE INDEX changes_reverts ON changes (reverts);
