# Blotter — Roadmap

> **Blotter** is a keyboard-first daily planner with an AI that pencils things in for you.
> The name comes from the desk blotter (the paper calendar pad on a desk) and from *blotting out* time, which is what block-out windows do.

This roadmap is the source of truth for the v1 one-shot. It is written as a **handover brief** for the implementing agent. Each phase gives intent, constraints, acceptance criteria and leads to verify. It does not prescribe file-level changes; the implementer decides structure within the constraints here.

**Workflow:** Roadmap (this document) → Design (Claude Design prompt, see §9) → Implementation (phases in §8, tracked commit by commit in `COMMITS.md`, with Luke as the feedback loop for bugs and visual errors).

---

## 1. Goals and scope

### v1 goals
- A **Diary** view (day columns of task cards) plus a calendar with **Week, Month and Year** views. All views are fully usable by keyboard alone.
- An **Inbox** for brain-dumping unorganised tasks. Inbox tasks can be auto-slotted by a deterministic scheduler, or by the AI.
- **Block-out windows** (e.g. work, 08:30–17:00 on weekdays) that flexible tasks can never land in. Fixed items such as appointments are allowed inside them.
- An **AI chat** where the user connects their own provider (Anthropic, OpenAI, or any OpenAI-compatible endpoint, including local models). The AI reads and changes the planner only through the app's own operation layer.
- **Guard rails** on AI changes:
  - Propose mode is the default, with auto-apply as a toggle.
  - A full change log with undo.
  - Snapshots that can be rolled back.
- **OS notifications** for reminders, with snooze. Reminders keep firing while the window is closed because the app stays in the tray.
- **Marking items complete**, with a timestamp, plus a completion history view.
- A **theme engine** with a live component preview, and a **keybinding editor**.
- **ICS export/import** and read-only ICS **subscriptions**. A subscription can optionally act as a block-out source, e.g. a published Outlook calendar.
- An **MCP server**, so external agents (Claude Desktop, Claude Code) can drive the planner through the same operations.
- Releases for **Windows and Linux**.

### Explicitly out of scope for v1
- **Multi-device sync via a self-hosted home server.** This is planned post-v1; see §10. Data stays **per device** in v1.
- Two-way CalDAV sync and the Google Calendar API (post-v1).
- Mobile apps.
- Splitting a task across several slots. A task occupies exactly one slot in v1.
- Auto-update via the Tauri updater, which needs signing infrastructure (post-v1).
- Code signing, accounts, and multi-user support.

### Audience
Built for Luke's personal use, structured so it can be open-sourced if it turns out well. That means:
- a clean README,
- no hard-coded personal data,
- MIT licence (proposed — confirm).

---

## 2. Platforms and packaging

| Target | Format | Notes |
|---|---|---|
| Windows 10/11 | NSIS installer (+ MSI) | This is the development platform. WebView2 ships with Windows. |
| Linux | **Flatpak** (Luke's preferred install) | Needs portals for notifications, tray (StatusNotifier), secrets and background/autostart. Verify each one inside the sandbox. |
| Linux | AppImage, `.deb` | Built with Tauri's bundler. |
| Linux (Arch) | AUR `PKGBUILD` | A `-bin` package built from the release artifacts is the simplest option. |

**CI:** GitHub Actions builds and tests on `windows-latest` and `ubuntu-latest` from the first phase onward. Development happens on Windows, so CI is what proves Linux releasability continuously, not just at the end.

### Known Linux caveats to design around from day one
1. **Global shortcuts don't work on Wayland** through Tauri's global-shortcut plugin, which is X11-only. Planned handling:
   - On Windows and X11, register the quick-capture shortcut natively.
   - On Wayland, quick capture is triggered by the CLI: `blotter --capture` (and `blotter --chat`). The running instance receives these via the single-instance plugin. The user binds the command in their compositor; the README documents a Hyprland `bind` example.
   - The settings page detects Wayland and explains this rather than showing a shortcut that silently does nothing.
   - *Lead:* check whether the xdg-desktop-portal **GlobalShortcuts** portal (supported by xdg-desktop-portal-hyprland) is usable from Rust. If it is, it's an improvement for later, not a v1 blocker.
2. **WebKitGTK on NVIDIA + Wayland** can render a blank or flickering window. Workaround: `WEBKIT_DISABLE_DMABUF_RENDERER=1`. The app should set this itself at startup on Linux, before the webview is created, unless the user has overridden it. Document it either way.
3. **Always-on-top** is not reliably enforceable on Wayland. The chat pop-out requests it; the README documents Hyprland `windowrule` float/pin rules.

---

## 3. Tech stack (decided)

| Layer | Choice | Why |
|---|---|---|
| Shell | **Tauri 2** | Small binaries, native installers, official plugins for everything needed. |
| Backend | **Rust** (stable) | The Rust core owns all state, scheduling, reminders and AI orchestration. |
| Frontend | **React + TypeScript + Vite** | |
| UI kit | **shadcn/ui** on **Radix** primitives, **Tailwind** | Accessible primitives (tooltip, popover, dialog, menu, toast, command) with keyboard support built in. Themed through CSS variables, which the theme engine edits directly. |
| Command palette | `cmdk` (shadcn `Command`) | |
| Calendar views | **Custom-built** (no FullCalendar) | Full keyboard control, and they consume theme tokens natively. |
| Dates (UI) | `date-fns` | |
| Quick-add parsing | `chrono-node` plus a custom token layer | See §5.7. |
| Frontend state | TanStack Query for backend data; Zustand for UI state | |
| Database | **SQLite**, WAL mode, embedded migrations | *Lead:* choose `sqlx` or `rusqlite` + `refinery`. Either is fine; prefer whichever keeps the pure core crate testable without the Tauri runtime. |
| Time (Rust) | `chrono` + `chrono-tz` (or `jiff`) | |
| Recurrence | RFC 5545 RRULE via the `rrule` crate | |
| ICS | `icalendar` crate (or equivalent) | |
| HTTP | `reqwest` (streaming) | |
| Secrets | `keyring` crate | Windows Credential Manager on Windows; Secret Service on Linux. |
| MCP | `rmcp` (official Rust MCP SDK) | *Lead:* verify the current crate name and API. |
| Tauri plugins | notification, single-instance, autostart, global-shortcut, dialog, fs, opener | |

*Lead (applies to all rows):* verify current major versions at scaffold time (Tauri 2.x plugin APIs, React 19, Tailwind v4 and its shadcn setup) and record the pinned versions in the README.

---

## 4. Architecture

### 4.1 Process and window model
- **One application process.** The Rust core holds the database, scheduler, reminder engine and AI orchestration. The React UI is a thin client: it calls Tauri commands and subscribes to Tauri events, and it never touches the database or API keys directly.
- **Closing the main window hides it to the tray.** The process keeps running so reminders fire. "Quit" lives in the tray menu and the command palette. Autostart on login is a setting, offered during first run.
- **Windows:**
  - **Main window:** normal, resizable, maximisable, with a sensible minimum size. The Year view is designed to use the full screen.
  - **Chat:** a collapsible panel docked inside the main window, **or** popped out into a slim separate window that requests always-on-top. That window has no maximise and is narrow by default — the "sticky note".
  - **Quick capture:** a small, undecorated, centred popup that writes straight to the Inbox and closes.
- **Single instance:** a second launch, or CLI flags such as `--capture` and `--chat`, focuses or acts on the running instance.

### 4.2 Code layout (constraint, not prescription)
- A Cargo workspace with a **pure core crate** (`blotter-core`). It contains models, the storage layer, the operation layer, the scheduler, recurrence and ICS logic, and has no Tauri dependency, so it is fully unit-testable.
- The **Tauri app crate** depends on the core and adds windows, tray, notifications, keychain and AI HTTP.
- The **MCP server** is a separate binary target (or a `blotter mcp` subcommand) that also depends on the core.
- The frontend lives in the same repo. The implementer decides the internal structure.

### 4.3 The operation layer (the heart of the app)
Every read and mutation goes through **one registry of typed operations**. Each operation has a name, a JSON Schema for its arguments, a description, and is flagged as read-only or mutating.

The same registry serves three consumers:
1. Tauri commands called by the UI.
2. The AI tool-calling loop (§4.6).
3. The MCP server (§4.8).

Every mutation writes to the **change log**: operation, entity, before/after JSON, actor (`user`, `ai:<provider>/<model>`, `mcp`, `system`), and a batch id. Undo/redo and AI proposals are built on this log.

The minimum set of operations:
- **Reads:** `list_agenda(range)`, `get_item`, `search_items`, `list_inbox`, `list_blockouts`, `find_free_slots(duration, window, constraints)`, `get_day_summary`.
- **Item mutations:** `create_item`, `update_item`, `delete_item`, `complete_item`, `uncomplete_item`, `schedule_item(id, start)`, `unschedule_item` (back to the Inbox), `auto_slot(ids | inbox)`.
- **Block-out and reminder mutations:** `create_blockout`, `update_blockout`, `delete_blockout`, `add_blockout_exception`, `set_reminder`, `clear_reminder`.

### 4.4 Data model
All timestamps are stored as UTC instants. Fixed items also store their IANA timezone. All-day items are stored as dates. Ids are ULIDs.

**`items`** — one table for tasks, events and reminders:

| Field | Meaning |
|---|---|
| `kind` | `task` \| `event` \| `reminder` |
| `anchoring` | `fixed` \| `flexible` (see §4.5). Defaults: events are fixed, tasks are flexible. A task can be set to fixed, e.g. a telehealth appointment. |
| `status` | `open` \| `done` \| `cancelled` |
| `title`, `notes` | |
| `start_at`, `end_at` | Null for unscheduled items. **Inbox = flexible task with no `start_at`.** |
| `duration_min` | Required to auto-slot. If missing, a default applies from settings. |
| `due_at` | Optional deadline. |
| `priority` | 0–3 |
| `tags`, `color` | |
| `all_day` | |
| `rrule`, `recurrence_id` / `parent_id` | Recurrence and per-occurrence overrides/exceptions. |
| `completed_at` | |
| `source` | `user` \| `ai` \| `import` \| `mcp` |
| `created_at`, `updated_at` | |

**Other tables:**
- **`reminders`:** item id, either an offset or an absolute time, `fired_at`, `snoozed_until`.
- **`blockouts`:** label, weekday set, start/end time-of-day, timezone, active date range, `permits_fixed` (default true), colour. Plus **`blockout_exceptions`** (dates on which the block-out is lifted, e.g. leave or public holidays).
- **`ics_subscriptions`:** URL, refresh interval, colour, `as_blockout` flag, last fetch status. Fetched events are cached read-only in their own table.
- **`changes`:** the change log. **`proposals`:** pending AI operation batches with status.
- **`snapshots`:** metadata only — reason, timestamp, file path. The snapshot files themselves are full copies of the database.
- **`chat_threads`, `chat_messages`:** including tool calls and results.
- **`settings`, `keybindings`, `themes`.**
- **`providers`:** name, kind, base URL, model, options. **API keys are never stored in SQLite**; only in the OS keychain.

### 4.5 Scheduling rules and the slot engine
This is pure Rust in the core crate, deterministic and heavily unit-tested.

**Concepts:**
- **Planning hours:** the window in which flexible tasks may be placed at all (default 07:00–22:00, configurable per weekday). This stops tasks landing at 3 a.m.
- **Busy time:**
  - events and scheduled items,
  - ICS subscription events (when the subscription is busy or set as a block-out),
  - block-outs,
  - a configurable buffer between items (default 0–10 minutes).

**Rules:**
1. A **flexible** item can never be placed in a block-out, outside planning hours, or overlapping busy time.
2. A **fixed** item is placed exactly where the user or AI says, *including inside block-outs*. If the block-out has `permits_fixed = false` (e.g. a "sleep" window), the app shows a warning but does not block it.
3. Overlapping fixed items are allowed but flagged as **conflicts**, both in the UI and in what tools return to the AI.

**`find_free_slots`** returns ranked candidate slots for a duration within a window.

**`auto_slot`** places a batch of tasks greedily:
- **Order:** by due date, then priority, then larger duration first.
- **Placement:** each task goes into the earliest valid slot before its due date.
- **Result:** placed items, plus unplaceable items with reasons (e.g. "no slot before due date"). Nothing is ever silently dropped.

**End-of-day rollover:** at a configured time, or on the first launch of a new day, unfinished flexible tasks from past days are either re-slotted or returned to the Inbox (a setting). Fixed items are never moved automatically.

### 4.6 AI integration
- **Providers:** a Rust trait with three adapters:
  - **Anthropic** (Messages API),
  - **OpenAI**,
  - **OpenAI-compatible** (base URL + optional key) for Ollama, LM Studio, llama.cpp server, OpenRouter, etc.

  Responses stream into the chat. A "test connection" button and a model list are fetched where the API allows. *Lead:* check each API's current tool-calling and streaming format at implementation time.
- **Credentials:** API keys are pasted into settings and stored in the OS keychain. All HTTP calls happen in Rust, never in the webview.
- **Context:** the system prompt carries:
  - current date/time and timezone,
  - the user's block-outs and planning hours,
  - today's agenda and the Inbox count,
  - the scheduling rules from §4.5 in plain language.

  Larger context is fetched through read tools on demand.
- **Tool loop:**
  - The model gets the operation registry as tools.
  - Tool arguments are **validated against the JSON Schema**. Validation errors go back to the model so it can retry; this matters for local models.
  - The loop is capped (e.g. 8 iterations per user turn).
- **Guard rails:**
  - **Propose mode (default):** mutating tools are not applied. They are recorded as a proposal batch, and the model is told "proposed, awaiting user approval". The chat shows a **proposal card** with a readable diff per operation, accept/reject per operation, and accept all/reject all.
  - **Auto mode (toggle):** before applying a batch, take a **snapshot** automatically, then apply. The chat shows what changed with a one-click **Undo batch**.
  - Read tools always run immediately.
- **Daily briefing:** from a command, or optionally at a set morning time. The AI summarises the day and proposes slots for Inbox items. The result appears in chat and as a notification.
- **Chat history** is persisted per thread. A new thread is one keypress away.

### 4.7 Reminders and notifications
- A Rust background task keeps a single timer for the **next** due reminder and recomputes it whenever the change log fires. Fired state is persisted so restarts never double-fire.
- **Catch-up:** reminders missed while the app wasn't running are delivered at the next start as one grouped "missed while closed" notification, within a configurable lookback.
- **Snooze** (5, 15 or 60 minutes, or a custom duration). *Lead:* check whether Tauri's notification plugin supports action buttons on Windows and Linux desktops. If it doesn't, clicking the notification opens the app on an in-app reminder toast with the snooze options, so snooze is always reachable.
- **Default reminder:** events get one at 10 minutes before, configurable. Tasks have no default reminder.

### 4.8 MCP server
- Exposes the operation registry over **stdio** for MCP clients.
- **Database access:**
  - It opens the same SQLite database in WAL mode.
  - Its writes go through the same operation layer, so they appear in the change log with actor `mcp`.
  - The running app detects external writes (e.g. by polling SQLite's `data_version` or watching the change log) and refreshes.
- Respects the same propose/auto setting. In propose mode, MCP mutations become proposals that appear in the app for approval.
- The README documents client configuration for Claude Desktop and Claude Code, including the Flatpak invocation (`flatpak run --command=blotter-mcp …`). *Lead:* verify.

### 4.9 Backups and rollback
- A **snapshot** is a full, consistent copy of the database made with SQLite's backup API or `VACUUM INTO`.
- **Snapshots are taken:**
  - before every auto-applied AI or MCP batch,
  - daily on first launch,
  - before every restore,
  - before ICS import,
  - and manually.
- **Retention is configurable**, e.g. keep the last 20 automatic snapshots plus all manual ones.
- **Restore UI:** lists snapshots with their reason and time. It needs confirmation and always snapshots the current state first, so a restore is itself reversible.
- **Undo/redo** (`Ctrl+Z` / `Ctrl+Shift+Z`) works from the change log for fine-grained reversal. Snapshots are the coarse safety net.
- **Full JSON export and import** for portability.

---

## 5. Features (v1)

### 5.1 Views
- **Diary (home view):** a planner-style board of day columns.
  - Today is first and widest; following days scroll to the right.
  - Each column has a large date header, a planned-vs-available time meter, and the day's items as individual **cards** (title, time or duration, tags, checkbox, proposed state).
  - Cards can be reordered within a day and dragged between days.
  - A docked **timeline rail** shows today's time grid, with block-outs and a now-line; a card can be dropped onto a time to schedule it.
  - Overdue items appear at the top of today's column.
  - The Inbox sits as a collapsible left rail.
  - One key toggles **Diary ↔ Calendar**.
- **Week:**
  - a time grid with block-outs shaded and labelled, conflicts marked, and a current-time line;
  - drag and drop to move or resize items;
  - the same moves by keyboard;
  - all-day items in a lane at the top.
- **Month:** a day grid with compact item chips, an overflow "+N" that opens a day popover, and a block-out indicator.
- **Year:**
  - 12 mini-months that fill the screen,
  - a per-day density heatmap (scheduled load, with a toggle to completed load),
  - pressing Enter on a day jumps to that day in Week view.
- **Inbox:** a side panel and a full view. Brain-dump input; multi-select; actions: auto-slot selected, schedule to a chosen slot, set duration/priority/due, delete, and "ask AI to sort".
- **History:** completed items with timestamps, filterable by date and tag, with simple streak and count stats.

### 5.2 Items
Items can be created, edited and deleted from dialogs, the command palette and quick add.

- **Completing** sets `completed_at`. Completed items stay visible with a muted style; uncompleting is allowed and logged.
- **Recurring items** support editing "this occurrence", "this and following" or "all", and deleting a single occurrence (stored as an exception).
- **Tags** use colours; `#tag` syntax works in quick add.

### 5.3 Block-outs
- Settings page for creating, editing and reordering block-outs.
- **Exceptions:** pick dates, or add one-off "lift this block-out on…" exceptions from the calendar via a context menu or keyboard.
- Block-outs are shaded in the Week and Today views.

### 5.4 Keyboard-first
Every action is reachable from the keyboard; the mouse is optional. Focus is always visible.

- **A single action registry:** each action has an id, a label, a default binding and a context (global or a specific view). It powers both the keymap and the command palette.
- **Bindings** support chords/sequences (e.g. `g` then `w`).
- **Proposed defaults** (finalise in implementation and list them in the README):

| Key | Action |
|---|---|
| `Ctrl+K` | Command palette |
| `n` | Quick add |
| `i` | Inbox |
| `Ctrl+/` | Toggle chat |
| `g d` / `g w` / `g m` / `g y` / `g h` | Diary / Week / Month / Year / History |
| `v` | Toggle Diary ↔ Calendar |
| `[` / `]` | Previous / next period |
| `.` | Jump to today |
| arrows or `h j k l` | Move focus between days and items |
| `Enter` | Open |
| `x` | Toggle complete |
| `e` | Edit |
| `Del` | Delete |
| `Shift` + arrows | Move the focused item |
| `Alt` + arrows | Resize the focused item |
| `s` | Schedule / auto-slot the focused item |
| `?` | Shortcut cheat-sheet overlay |

- **Keybinding editor:**
  - search actions,
  - click or press Enter, then type the new binding,
  - conflict detection with a "swap or cancel" choice,
  - reset one or all,
  - import/export.

### 5.5 Theme engine
- **A theme is a JSON set of tokens:**
  - colours (background, surface, text, muted, accent, border, danger, block-out, conflict, focus ring, heatmap ramp),
  - corner radius,
  - font family and size scale,
  - density.
- **Theme page:** a token editor on one side and a **live preview gallery** on the other. The gallery shows the real components with the draft theme applied *only inside the preview*: buttons, inputs, tooltip, dialog, toast, a diary card, a mini week grid with a block-out and a conflict, a chat bubble with a proposal card, and the command palette. "Apply" commits the theme app-wide.
- **Built-in presets:** **Dark** (default), **Light**, **Paper** and **Gruvbox**. No translucent/glass theme in v1. Users can duplicate, import and export themes.
- **Contrast checking:** text/background pairs below WCAG AA get a visible warning in the editor.

### 5.6 Notifications
See §4.7. The notification settings page covers: enable/disable, default reminder offsets, snooze options, catch-up lookback, daily briefing time, and rollover time.

### 5.7 Natural-language quick add
`chrono-node` parses dates and times. A custom token layer adds:
- duration (`~45m`, `~1h30`),
- priority (`!1`–`!3`),
- tags (`#shopping`),
- fixed anchoring (`@fixed`, or implied by an explicit time on an event-like phrase — finalise the rule and document it),
- recurrence phrases (`every monday`, `every weekday`, `every 2 weeks`), mapped to an RRULE.

As the user types, a **live parse preview** shows the item it will create. Everything works offline, without any AI.

**Example:** `dentist tue 3pm ~45m @fixed` → a fixed item on Tuesday at 15:00 lasting 45 minutes.

### 5.8 ICS
- **Export:** all items or a date range to a `.ics` file.
- **Import:** a `.ics` file. A snapshot is taken first, and an import summary is shown.
- **Subscriptions:** a URL with a refresh interval. Subscribed items are read-only, have their own colour, and can optionally be treated as block-outs.

### 5.9 Settings pages
1. General (week start, 12/24-hour clock, timezone, planning hours, default task duration, rollover behaviour, autostart)
2. Block-outs
3. Calendars (ICS)
4. AI providers (including the propose/auto toggle)
5. Notifications
6. Keybindings
7. Theme
8. Backups
9. About

### 5.10 First run
A short, keyboard-navigable setup with every step skippable:
1. Theme
2. Week start and planning hours
3. Add a work block-out
4. Connect an AI
5. Enable autostart

---

## 6. Quality bar
- The core crate has unit tests for:
  - the slot engine (block-out rules, fixed vs flexible, DST transitions, due dates, unplaceable reporting),
  - recurrence expansion and exceptions,
  - change-log undo/redo,
  - snapshot and restore,
  - ICS round-trip.
- Frontend: Vitest for the parser and keymap logic; component tests where cheap.
- `cargo clippy -D warnings`, `cargo fmt --check`, ESLint and Prettier are clean in CI.
- No `unwrap()` on user-data paths in Rust. Errors surface as toasts, not crashes.
- Every phase leaves the app **launchable and usable**. No half-wired features on the main branch.

---

## 7. Commit protocol (applies to every phase)
- **The implementer never commits or pushes. Luke makes every commit.**
- Work is delivered in small increments. Each increment ends with a new entry in `COMMITS.md` containing:
  - the commit number and phase,
  - the **commit message** (Conventional Commits, e.g. `feat(core): …`),
  - the **complete list of files** to stage — added, modified, deleted — with repo-relative paths,
  - **test steps** for Luke to run before committing (commands plus what to check by hand),
  - notes: known gaps, follow-ups.
- Changed files are delivered as **full files**, not diffs.
- Each increment should build and pass tests on its own (test before push).
- If Luke reports a bug or visual error, the fix gets its own ledger entry. Don't rewrite a previous entry; mark it as amended or superseded.
- `COMMITS.md` contains the planned sequence. The implementer may split or merge planned commits but must record the change.

---

## 8. Phases
Each phase states the intent, then the acceptance criteria (AC) and leads to verify. Order matters: later phases rely on earlier ones.

### Phase 0 — Scaffold and tooling
**Intent:** a running Tauri 2 + React app and a Cargo workspace with the core crate, linting and testing, and CI on Windows and Linux.

**AC:**
- `pnpm tauri dev` launches a window on Windows.
- CI builds and tests on both operating systems.
- Lint and format checks are enforced.

**Leads:**
- Pin current versions.
- Set the WebKitGTK environment-variable workaround in Linux startup (§2).

### Phase 1 — Core domain, storage and operation layer
**Intent:** the data model (§4.4), migrations, the operation registry with JSON Schemas (§4.3), and the change log.

**AC:**
- Every operation listed in §4.3 except `find_free_slots` and `auto_slot` (which arrive in Phase 5) is implemented in the core with tests.
- Every mutation produces change-log rows.
- The database is created in the OS app-data directory with WAL enabled.

### Phase 2 — App shell
**Intent:**
- tray, close-to-tray, single instance with CLI-flag handling,
- shadcn/ui set up with theme-token plumbing and the built-in themes,
- the app layout and routing,
- the action/keymap registry,
- the command palette.

**AC:**
- Closing the window keeps the app in the tray, and Quit really quits.
- A second launch focuses the first instance.
- The theme can be switched among the presets.
- `Ctrl+K` lists the registered actions and runs them.

### Phase 3 — Calendar views and items
**Intent:**
- the Diary view and the Week, Month and Year views (custom-built),
- item create/edit/delete,
- completion with timestamps,
- full keyboard navigation and keyboard moving/resizing,
- drag and drop in Week view.

**AC:**
- Every view works with the mouse unplugged.
- The Year view fills a maximised window legibly.
- Items persist across restarts.

### Phase 4 — Inbox and capture
**Intent:**
- the Inbox panel and view,
- the NL quick-add parser with live preview,
- the quick-capture window (native shortcut on Windows/X11; `--capture` flag everywhere).

**AC:**
- Typing a brain-dump line creates an Inbox item in under 2 seconds from the hotkey.
- `blotter --capture` works from a Hyprland bind.
- Parser tests cover the token grammar.

**Progress (2026-10-14):**
- #16 (Inbox) is committed.
- #17 (quick add) is committed (`22e3001`). Chips and preview are verified live; checking Enter, Ctrl+Enter and Alt+Enter is folded into #18's test steps.
- #18 (the quick-capture window, global shortcut and `--capture`) is ready to commit. The Linux shortcut and Wayland paths still need a live check (Phase 13).

### Phase 5 — Block-outs and scheduling
**Intent:**
- block-outs with exceptions and their settings page,
- block-out shading,
- the slot engine (§4.5),
- auto-slot from the Inbox,
- conflict flags,
- end-of-day rollover.

**AC:**
- A flexible task can never be placed in a block-out by any path (UI, auto-slot, AI, MCP).
- A fixed task can be.
- Unplaceable tasks report reasons.
- Engine unit tests cover DST and due dates.

### Phase 6 — Recurrence and reminders
**Intent:**
- RRULE items with occurrence editing,
- the reminder engine,
- OS notifications, snooze and catch-up,
- the autostart setting.

**AC:**
- A reminder fires with the window closed.
- No reminder double-fires after a restart.
- Snooze works through a notification action or the in-app fallback.
- **Notifications work in the Flatpak build** (verify in Phase 13, but design for it now).

### Phase 7 — Undo, snapshots and backups
**Intent:**
- undo/redo from the change log,
- the snapshot system and restore UI,
- JSON export/import.

**AC:**
- Any sequence of edits can be undone and redone.
- Restoring a snapshot also snapshots the current state first.
- A JSON export re-imports to an identical state.

### Phase 8 — AI
**Intent:**
- providers and keychain credentials,
- the tool loop over the operation registry,
- the chat panel and pop-out window,
- the proposal UI and auto mode with an automatic snapshot,
- the daily briefing.

**AC:**
- **Every provider kind works:** Anthropic, OpenAI, and a local Ollama model can each add an event, add Inbox items and auto-slot them.
- **Propose mode:** nothing changes until the user accepts.
- **Auto mode:** each batch is snapshotted and can be undone with one click.
- **Robustness:** invalid tool arguments from a model produce a retry, not a crash.
- **Key handling:** API keys never appear in SQLite, logs or the webview.

**Leads:** check each provider's current tool-call formats and streaming events.

### Phase 9 — ICS
**Intent:** export, file import, and subscriptions with the block-out option.

**AC:**
- An exported `.ics` imports into Google Calendar and the Apple Calendar app.
- A published Outlook calendar URL subscribes successfully, and those events block flexible slotting when `as_blockout` is set.

### Phase 10 — Keybinding editor, theme engine, remaining settings
**Intent:** the keybinding editor (§5.4), the theme engine page with live preview (§5.5), all settings pages (§5.9), and first-run setup (§5.10).

**AC:**
- Rebinding takes effect immediately and persists.
- Conflicts are caught.
- Theme edits preview without affecting the rest of the app until applied.
- Contrast warnings appear when they should.

### Phase 11 — History and stats
**Intent:** the History view with completion timestamps, filters and streaks.

**AC:** completions from any path (UI, AI, MCP) appear, with correct local times.

### Phase 12 — MCP server
**Intent:** a stdio MCP server over the same operation registry (§4.8).

**AC:**
- Claude Desktop or Claude Code can list the agenda and propose or create items.
- Changes appear in the running app without a restart.
- The propose/auto setting is respected.

### Phase 13 — Packaging and release
**Intent:**
- Windows NSIS/MSI installers,
- AppImage and `.deb`,
- a Flatpak manifest with the minimum permissions needed,
- an AUR `PKGBUILD`,
- the README (install, Hyprland binds and window rules, NVIDIA note, MCP setup, default keybindings),
- LICENSE,
- `v1.0.0`.

**AC:**
- Every artifact installs and launches on a clean VM or container.
- The Flatpak build passes the smoke checklist: notifications, tray, keychain, autostart, `--capture`, ICS subscription fetch.

---

## 9. Design handoff (next phase)
The Claude Design prompt (`design-prompt.md`, which also sets the visual direction) must produce visuals for the following screens and components, so implementation has a reference for everything visible.

**Screens:**
- the Diary view (day columns, cards, timeline rail),
- the Week, Month and Year views, with block-out shading, conflicts, completed items, the now-line and the Year heatmap,
- the Inbox (panel and full view),
- the item dialog, quick-add bar with live parse preview, and quick-capture popup,
- the command palette and `?` shortcut overlay,
- the chat panel and pop-out "sticky note" window, including a proposal card (diff, accept/reject) and an auto-mode "changed + undo" card,
- the reminder toast with snooze,
- every settings page, with emphasis on the **Theme engine** (editor plus live preview gallery) and the **Keybinding editor**,
- History,
- the first-run setup,
- empty, loading and error states.

**Other requirements:**
- Show the Dark (default), Light, Paper and Gruvbox presets.
- Show visible focus states throughout, since the app is keyboard-first.
- Components should map onto shadcn/Radix primitives wherever possible, so the design is implementable without custom primitives.

---

## 10. Post-v1 (parked)
- **Self-hosted sync:** run a Blotter server on a home server and connect each device to it from settings. The operation layer and change log are designed with this in mind: ULIDs, actor and timestamps on every change, and no reliance on local auto-increment ids.
- Two-way CalDAV sync (iCloud, Fastmail, Nextcloud) and the Google Calendar API.
- Auto-update via the Tauri updater, plus code signing.
- Splitting a task across multiple slots.
- Using the GlobalShortcuts portal for native hotkeys on Wayland.

---

## 11. Decisions log

| # | Decision | Status |
|---|---|---|
| D1 | Tauri 2 + Rust + React/TS/Vite | Confirmed |
| D2 | shadcn/ui (Radix) + Tailwind; theme via CSS variables | Confirmed |
| D3 | Custom calendar views (no FullCalendar) | Confirmed |
| D4 | Local-first SQLite, per-device data in v1 | Confirmed |
| D5 | One operation layer shared by UI, AI and MCP; every mutation logged | Confirmed |
| D6 | Deterministic slot engine; AI picks intent, not timing maths | Confirmed |
| D7 | Fixed vs flexible anchoring; flexible never in block-outs | Confirmed |
| D8 | AI propose mode by default, auto toggle, snapshots + undo | Confirmed |
| D9 | AI providers: Anthropic, OpenAI, OpenAI-compatible (local); keys in OS keychain; HTTP in Rust only | Confirmed |
| D10 | Main window resizable/maximisable; chat docked or popped out as a slim always-on-top window | Confirmed |
| D11 | Close-to-tray + optional autostart so reminders fire | Confirmed |
| D12 | Wayland: CLI-flag quick capture via compositor bind instead of a global shortcut | Confirmed |
| D13 | Packaging: NSIS/MSI, AppImage, .deb, Flatpak, AUR | Confirmed |
| D14 | MCP server in v1 (Phase 12) | Confirmed |
| D15 | Planning hours concept (default 07:00–22:00) to bound flexible slotting | **Proposed — confirm** |
| D16 | One slot per task in v1 (no splitting) | **Proposed — confirm** |
| D17 | No Tauri updater or code signing in v1 | **Proposed — confirm** |
| D18 | MIT licence | **Proposed — confirm** |
| D19 | Name: **Blotter** | **Proposed — confirm** |
| D20 | Diary view (day columns of task cards + today timeline rail) replaces the Today agenda as the home view | Confirmed |
| D21 | Visual direction: Linear's base design language, Notion Calendar's colour coordination and calendar layout, Sunsama-style diary view | Confirmed |
| D22 | No glass/translucent theme or window effects in v1 | Confirmed |
| D23 | Dark as default theme; monospace for times, durations and keyhints; AI proposals drawn "in pencil" | **Proposed — confirm** |
