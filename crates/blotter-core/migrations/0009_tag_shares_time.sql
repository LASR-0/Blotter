-- Tags whose tasks can be done at the same time as other tasks (laundry
-- during cleaning): such a task may be slotted over other tasks, doesn't
-- make its time busy for them, and isn't a conflict with them.
ALTER TABLE tags ADD COLUMN shares_time INTEGER NOT NULL DEFAULT 0;
