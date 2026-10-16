-- The iCalendar UID an item was imported with (or exported under), so the
-- same file imported twice doesn't duplicate it and a re-export keeps the
-- UID other calendars know it by.
ALTER TABLE items ADD COLUMN ics_uid TEXT;
CREATE INDEX items_ics_uid ON items (ics_uid) WHERE ics_uid IS NOT NULL;
