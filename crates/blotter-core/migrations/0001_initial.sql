-- Blotter v1 schema.
--
-- Conventions:
--   * Ids are ULID strings (26 chars, lexicographically time-ordered).
--   * Instants are UTC text in RFC 3339 with a trailing Z and whole seconds,
--     e.g. 2026-10-14T08:30:00Z, so they sort and compare as text.
--   * Dates are YYYY-MM-DD; times of day are HH:MM (24-hour).
--   * Booleans are INTEGER 0/1. JSON columns hold UTF-8 JSON text.

CREATE TABLE items (
    id            TEXT PRIMARY KEY NOT NULL,
    kind          TEXT NOT NULL CHECK (kind IN ('task', 'event', 'reminder')),
    anchoring     TEXT NOT NULL CHECK (anchoring IN ('fixed', 'flexible')),
    status        TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'done', 'cancelled')),
    title         TEXT NOT NULL,
    notes         TEXT NOT NULL DEFAULT '',
    -- Timed items use start_at/end_at. All-day items use start_date/end_date
    -- (end inclusive). An item with neither is unscheduled; a flexible,
    -- unscheduled task is in the Inbox.
    all_day       INTEGER NOT NULL DEFAULT 0 CHECK (all_day IN (0, 1)),
    start_at      TEXT,
    end_at        TEXT,
    start_date    TEXT,
    end_date      TEXT,
    -- IANA zone the user meant for a fixed item (e.g. Australia/Melbourne).
    tz            TEXT,
    duration_min  INTEGER CHECK (duration_min IS NULL OR duration_min > 0),
    due_at        TEXT,
    priority      INTEGER NOT NULL DEFAULT 0 CHECK (priority BETWEEN 0 AND 3),
    tags          TEXT NOT NULL DEFAULT '[]',
    color         TEXT,
    -- Manual order within a Diary day column.
    sort_order    REAL NOT NULL DEFAULT 0,
    rrule         TEXT,
    -- For per-occurrence overrides: the series item and the original start
    -- of the occurrence being overridden.
    parent_id     TEXT REFERENCES items (id) ON DELETE CASCADE,
    recurrence_id TEXT,
    completed_at  TEXT,
    source        TEXT NOT NULL DEFAULT 'user' CHECK (source IN ('user', 'ai', 'import', 'mcp')),
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    CHECK (end_at IS NULL OR start_at IS NOT NULL),
    CHECK (end_at IS NULL OR end_at >= start_at),
    CHECK (all_day = 0 OR start_date IS NOT NULL),
    CHECK (end_date IS NULL OR end_date >= start_date),
    CHECK ((status = 'done') = (completed_at IS NOT NULL))
);
CREATE INDEX items_start_at ON items (start_at);
CREATE INDEX items_start_date ON items (start_date);
CREATE INDEX items_parent ON items (parent_id);
CREATE INDEX items_completed_at ON items (completed_at);
CREATE INDEX items_inbox ON items (anchoring, start_at, start_date) WHERE start_at IS NULL AND start_date IS NULL;

CREATE TABLE tags (
    name  TEXT PRIMARY KEY NOT NULL,
    color TEXT
);

CREATE TABLE reminders (
    id            TEXT PRIMARY KEY NOT NULL,
    item_id       TEXT NOT NULL REFERENCES items (id) ON DELETE CASCADE,
    -- Exactly one of: minutes before the item's start, or an absolute instant.
    offset_min    INTEGER,
    at            TEXT,
    fired_at      TEXT,
    snoozed_until TEXT,
    created_at    TEXT NOT NULL,
    CHECK ((offset_min IS NULL) <> (at IS NULL)),
    CHECK (offset_min IS NULL OR offset_min >= 0)
);
CREATE INDEX reminders_item ON reminders (item_id);

CREATE TABLE blockouts (
    id            TEXT PRIMARY KEY NOT NULL,
    label         TEXT NOT NULL,
    -- Bitmask, Monday = 1 ... Sunday = 64.
    weekdays      INTEGER NOT NULL CHECK (weekdays BETWEEN 1 AND 127),
    -- An end before the start wraps past midnight (e.g. Sleep 22:30-06:30).
    start_time    TEXT NOT NULL,
    end_time      TEXT NOT NULL,
    tz            TEXT NOT NULL,
    active_from   TEXT,
    active_until  TEXT,
    permits_fixed INTEGER NOT NULL DEFAULT 1 CHECK (permits_fixed IN (0, 1)),
    color         TEXT,
    icon          TEXT,
    sort_order    INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    CHECK (start_time <> end_time),
    CHECK (active_until IS NULL OR active_from IS NULL OR active_until >= active_from)
);

CREATE TABLE blockout_exceptions (
    id          TEXT PRIMARY KEY NOT NULL,
    blockout_id TEXT NOT NULL REFERENCES blockouts (id) ON DELETE CASCADE,
    date        TEXT NOT NULL,
    note        TEXT NOT NULL DEFAULT '',
    created_at  TEXT NOT NULL,
    UNIQUE (blockout_id, date)
);

CREATE TABLE ics_subscriptions (
    id            TEXT PRIMARY KEY NOT NULL,
    name          TEXT NOT NULL,
    url           TEXT NOT NULL,
    refresh_min   INTEGER NOT NULL DEFAULT 60 CHECK (refresh_min >= 5),
    color         TEXT,
    as_blockout   INTEGER NOT NULL DEFAULT 0 CHECK (as_blockout IN (0, 1)),
    enabled       INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    last_fetch_at TEXT,
    last_ok_at    TEXT,
    last_status   TEXT,
    last_error    TEXT,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

-- Read-only cache of fetched subscription events (recurrences expanded).
CREATE TABLE ics_events (
    id              TEXT PRIMARY KEY NOT NULL,
    subscription_id TEXT NOT NULL REFERENCES ics_subscriptions (id) ON DELETE CASCADE,
    uid             TEXT NOT NULL,
    title           TEXT NOT NULL,
    location        TEXT NOT NULL DEFAULT '',
    all_day         INTEGER NOT NULL DEFAULT 0 CHECK (all_day IN (0, 1)),
    start_at        TEXT,
    end_at          TEXT,
    start_date      TEXT,
    end_date        TEXT,
    busy            INTEGER NOT NULL DEFAULT 1 CHECK (busy IN (0, 1))
);
CREATE INDEX ics_events_sub ON ics_events (subscription_id);
CREATE INDEX ics_events_start_at ON ics_events (start_at);

-- Change log. Every mutation from every actor writes one row per entity
-- touched; rows sharing a batch_id are undone together.
CREATE TABLE changes (
    id          TEXT PRIMARY KEY NOT NULL,
    batch_id    TEXT NOT NULL,
    op          TEXT NOT NULL,
    entity      TEXT NOT NULL,
    entity_id   TEXT NOT NULL,
    before_json TEXT,
    after_json  TEXT,
    -- user | ai:<provider>/<model> | mcp | system
    actor       TEXT NOT NULL,
    created_at  TEXT NOT NULL,
    undone_at   TEXT
);
CREATE INDEX changes_batch ON changes (batch_id);
CREATE INDEX changes_entity ON changes (entity, entity_id);

-- Pending AI/MCP operation batches awaiting review (propose mode).
CREATE TABLE proposals (
    id          TEXT PRIMARY KEY NOT NULL,
    actor       TEXT NOT NULL,
    thread_id   TEXT REFERENCES chat_threads (id) ON DELETE SET NULL,
    status      TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'accepted', 'rejected', 'partial')),
    summary     TEXT NOT NULL DEFAULT '',
    created_at  TEXT NOT NULL,
    resolved_at TEXT
);

CREATE TABLE proposal_ops (
    id           TEXT PRIMARY KEY NOT NULL,
    proposal_id  TEXT NOT NULL REFERENCES proposals (id) ON DELETE CASCADE,
    idx          INTEGER NOT NULL,
    op           TEXT NOT NULL,
    args_json    TEXT NOT NULL,
    -- Rendered before/after for the proposal card diff.
    preview_json TEXT,
    status       TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'accepted', 'rejected', 'failed')),
    error        TEXT,
    UNIQUE (proposal_id, idx)
);

-- Metadata only; each snapshot is a full database copy on disk.
CREATE TABLE snapshots (
    id         TEXT PRIMARY KEY NOT NULL,
    reason     TEXT NOT NULL,
    manual     INTEGER NOT NULL DEFAULT 0 CHECK (manual IN (0, 1)),
    path       TEXT NOT NULL,
    size_bytes INTEGER,
    created_at TEXT NOT NULL
);

CREATE TABLE chat_threads (
    id          TEXT PRIMARY KEY NOT NULL,
    title       TEXT NOT NULL DEFAULT '',
    provider_id TEXT REFERENCES providers (id) ON DELETE SET NULL,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE chat_messages (
    id              TEXT PRIMARY KEY NOT NULL,
    thread_id       TEXT NOT NULL REFERENCES chat_threads (id) ON DELETE CASCADE,
    role            TEXT NOT NULL CHECK (role IN ('system', 'user', 'assistant', 'tool')),
    content         TEXT NOT NULL DEFAULT '',
    tool_calls_json TEXT,
    tool_call_id    TEXT,
    proposal_id     TEXT REFERENCES proposals (id) ON DELETE SET NULL,
    batch_id        TEXT,
    created_at      TEXT NOT NULL
);
CREATE INDEX chat_messages_thread ON chat_messages (thread_id, created_at);

-- API keys are never stored here; they live in the OS keychain keyed by id.
CREATE TABLE providers (
    id           TEXT PRIMARY KEY NOT NULL,
    name         TEXT NOT NULL,
    kind         TEXT NOT NULL CHECK (kind IN ('anthropic', 'openai', 'openai_compatible')),
    base_url     TEXT,
    model        TEXT NOT NULL,
    options_json TEXT NOT NULL DEFAULT '{}',
    is_default   INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL
);

CREATE TABLE settings (
    key        TEXT PRIMARY KEY NOT NULL,
    value_json TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Only overrides of the default keymap are stored.
CREATE TABLE keybindings (
    action_id  TEXT PRIMARY KEY NOT NULL,
    binding    TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- User themes; the four built-in presets ship with the frontend.
CREATE TABLE themes (
    id          TEXT PRIMARY KEY NOT NULL,
    name        TEXT NOT NULL,
    tokens_json TEXT NOT NULL,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
