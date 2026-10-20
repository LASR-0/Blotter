-- A task's do-window: not slotted before `not_before` (its end is the
-- existing due date), and a usual time of day to slot it in. Weekly tasks
-- use the time window too.
ALTER TABLE items ADD COLUMN not_before TEXT;
ALTER TABLE items ADD COLUMN window_start TEXT;
ALTER TABLE items ADD COLUMN window_end TEXT;
