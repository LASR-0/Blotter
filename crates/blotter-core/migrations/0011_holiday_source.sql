-- Where a holiday came from: null when the user (or the AI) made it, or
-- `public:<country or region>` (e.g. `public:AU-VIC`) when it was imported
-- as a public holiday, so a re-import can replace what it brought in.
ALTER TABLE holidays ADD COLUMN source TEXT;
