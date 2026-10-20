-- Holidays: date ranges away from the usual week. Making one lifts the
-- block-outs that fall in it (exceptions carrying its id), work tasks are
-- kept off its days, and it is removed once its last day has passed.
CREATE TABLE holidays (
    id         TEXT PRIMARY KEY NOT NULL,
    label      TEXT NOT NULL,
    start_date TEXT NOT NULL,
    end_date   TEXT NOT NULL,
    color      TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    CHECK (end_date >= start_date)
);

-- The holiday that lifted a block-out on this date, if a holiday did.
ALTER TABLE blockout_exceptions ADD COLUMN holiday_id TEXT;
