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
- **Built-in presets:** **Blotter Light**, **Blotter Dark** (default), **Studio Light** and **Studio Dark** (Luke, 2026-10-17; these replaced Dark, Light, Paper and Gruvbox, whose saved ids map to them). No translucent/glass theme in v1. Users can duplicate, import and export themes.
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
- **Since 2026-10-15 the implementer commits each increment itself, as Luke (his git identity, no AI co-author trailer), once its test steps pass. It never pushes: Luke syncs from VS Code.** (Until then Luke made every commit himself.)
- Work is delivered in small increments. Each increment ends with a new entry in `COMMITS.md` containing:
  - the commit number and phase,
  - the **commit message** (Conventional Commits, e.g. `feat(core): …`),
  - the **complete list of files** to stage — added, modified, deleted — with repo-relative paths,
  - **test steps** (commands plus what to check by hand), run before committing,
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
- #18 (the quick-capture window, global shortcut and `--capture`) is committed (`0a70bd0`) and verified live on Windows. The Linux shortcut and Wayland paths still need a live check (Phase 13).

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

**Progress (2026-10-14):**
- #19 (block-out window expansion in the core, plus `list_blockout_windows`) is committed.
- #20 (the slot engine, `find_free_slots`, planning hours and buffer settings, and enforcement on every path that a flexible task never lands in a block-out) is committed.
- #21 (block-out settings page, `b l` lifting, and calendar shading from core windows) is committed.
- #22 (`auto_slot` with a result panel, plus conflict flags in the agenda, Week and Diary) is committed.
- #23 (end-of-day rollover: inbox or reslot mode, once a day after 04:00, startup and tray timer, banner) is committed. **Phase 5 is complete.**
- Next: Phase 6.

### Phase 6 — Recurrence and reminders

**Progress (2026-10-15):** complete. #24–#26 (with #24a and #25a–#25d) are committed.
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

**Progress (2026-10-15):** complete. #27 (undo/redo, with fixes #27a–#27d) and #28 (snapshots, restore, JSON export/import) are committed. Phase 8 (AI) is next and needs Luke's Anthropic key for live checks.
**Intent:**
- undo/redo from the change log,
- the snapshot system and restore UI,
- JSON export/import.

**AC:**
- Any sequence of edits can be undone and redone.
- Restoring a snapshot also snapshots the current state first.
- A JSON export re-imports to an identical state.

### Phase 8 — AI

**Plan (2026-10-15, D29):** #29 provider layer and keychain (API providers experimental) → #29a MCP server (from Phase 12) → #30 tool loop with the Claude Code provider → #31 chat panel → #32 proposals and auto mode → #33 daily briefing.

**Progress (2026-10-15):** #29 (providers, experimental) and #29a (MCP server, `blotter mcp`) are committed. #30 (chat backend: tool loop, threads, Claude Code provider, verified live on the subscription) is committed. #31 (chat panel, pop-out window, `--chat`) and #31a (Claude Code kept running between messages) are committed, as are #32 (proposals: Propose/Auto, remembered), #32a (Undo batch) and #33 (daily briefing). **Phase 8 is complete**; the API-key providers are still untested against real services (D29). Next: Phase 9.

**Open items (2026-10-15, end of session):**
- Linux CI for #29a–#33 was still running at hand-off (looking good); confirm it went green.
- #32 proposal cards were seen rendering in the demo by Luke, but not yet accepted or rejected there; #32a Undo batch and #33 briefing are untried in the window.
- #35 needs a published Outlook (or other ICS) calendar URL from Luke for its acceptance check.
- Before publishing the Claude Code route: check Anthropic's terms on driving Claude Code from another app (D29).
- API-key providers stay "Experimental — untested" until Luke provides a key (D29).
- Next session: start Phase 9 with #34.
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
**Progress (2026-10-16):** Live ICS checks (AC 1 and AC 2) are deferred to after Phase 12, before Phase 13 (Luke, 2026-10-16). #34 (ICS export and file import), #35 (subscriptions with the block-out option) and #35a (subscribed events drawn read-only in Diary, Week and Month) are committed. **Phase 9 is complete in code.** Still needing Luke: an exported file tried in Google Calendar and Apple Calendar (AC 1), and a published Outlook URL subscribed in the app (AC 2; the logic is tested against an Outlook-style feed and a local server). Open: subscriptions aren't in the JSON export. Next: Phase 10.
**Intent:** export, file import, and subscriptions with the block-out option.

**AC:**
- An exported `.ics` imports into Google Calendar and the Apple Calendar app.
- A published Outlook calendar URL subscribes successfully, and those events block flexible slotting when `as_blockout` is set.

### Phase 10 — Keybinding editor, theme engine, remaining settings
**Progress (2026-10-16):** #36 (keybinding editor, with #36a), #37 (theme engine), #38 (General settings) and #38a (first-run setup) are committed; every §5.9 page exists. **Phase 10 is complete in code**; its ACs need a live check by Luke. Next: Phase 11.
**Intent:** the keybinding editor (§5.4), the theme engine page with live preview (§5.5), all settings pages (§5.9), and first-run setup (§5.10).

**AC:**
- Rebinding takes effect immediately and persists.
- Conflicts are caught.
- Theme edits preview without affecting the rest of the app until applied.
- Contrast warnings appear when they should.

### Phase 11 — History and stats
**Progress (2026-10-16):** #39 (History view, list_completions, streaks) is committed. **Phase 11 is complete in code**; the AC needs a live check by Luke. Next: Phase 12.
**Intent:** the History view with completion timestamps, filters and streaks.

**AC:** completions from any path (UI, AI, MCP) appear, with correct local times.


### Before Phase 12 — polish backlog (Luke, 2026-10-16)
Done on 2026-10-16: #39a–#39e (presets, window size, About, Diary timeline, key hints, icon, one-page Settings, centring, sidebar chips) and #39f (Smart slot).
- **Sidebar Inbox count:** removed (uncommitted, with the Theme fix below).
- **Smart slot:** tried live with Claude Code by Luke; works.
- **New presets (Luke, 2026-10-17):** Blotter Light (his "Light copy 2": warm cream, orange accent, serif headings, larger text and corners), Blotter Dark (its dark counterpart, made to match; passes every AA check), Studio Light and Studio Dark (his "Light copy" and "Dark copy"). Blotter Dark is the default. Paper and Gruvbox are gone; saved ids `dark`, `light`, `paper`, `gruvbox` map to Studio Dark, Studio Light, Blotter Light and Blotter Dark. Contrast kept as is elsewhere (Luke: the hues are fine): Blotter Light accent text 2.00–2.18, text on accent 2.34; Studio Light text on accent 2.28, accent text 2.51–2.68; Studio Dark danger 3.43, text on accent 4.08. Uncommitted.
- **Other AI apps:** v1 ships with Claude Code as the only tested route; Codex CLI / Copilot CLI are post-v1 (§10).
- **Theme page with the chat docked:** fixed in code (uncommitted): below 760px of page width the preview stacks under the editor and the page scrolls. Needs Luke's check at the minimum window size.
- **Diary card focus ring:** fixed in code (uncommitted): 4px top padding on the day column list, so the 2px outline + 2px offset of a focused top card isn't clipped.
- **Logo colours apart from the accent (Luke):** new theme tokens `--logo-accent` (the calendar mark), `--logo-text` (the wordmark) and `--logo-shadow` (its solid extrusion) in the Theme editor's Logo group, with a Logo panel in the live preview. Themes saved earlier take their own text and accent. Uncommitted.
- **App icon in the new colours:** `public/blotter.svg` and `src-tauri/icons/app-icon.svg` recoloured to the Dark preset's logo (calendar #fe5858, page #fef6e1, fold #b73f3f, cells #fec3b5, done #56c49b, tile #121317); desktop icons regenerated with `pnpm tauri icon src-tauri/icons/app-icon.svg` (mobile/Store outputs removed). Uncommitted.
- **Sliding side panels (Luke):** the sidebar, the Diary's Inbox rail and the docked chat slide open and closed (200ms width transition, off with reduced motion) via `SlidePanel`. Uncommitted.
- **Styled hover hints (Luke):** every `title` attribute now shows as a themed tooltip (`TitleTips`, mounted once per window) instead of the browser's; the title is held back while hovered and put back after, for assistive tech. Uncommitted.
- **Calendars & tags filters (Luke, from the design reference):** a sidebar section listing every tag in use (new read op `list_tags`) and each enabled subscribed calendar, each with a checkbox in its colour that shows or hides it in the Diary, Week, Month and Year views; untagged items always show, and an item hides only when all its tags are hidden. Kept in local storage. The section's + opens Settings → Calendars. Uncommitted.
- **Inbox brain-dump box:** starts at 2 lines and grows a line at a time with the text, up to half the window, then scrolls. Uncommitted.
- **Title bar colours from the theme (Luke):** new tokens `--titlebar` and `--titlebar-text` (Theme editor, Window group; presets start at their background and text, older themes take theirs). Each window applies them through `set_titlebar` (Windows 11 DWM caption/border/text colour; light/dark via `set_theme` elsewhere). Uncommitted; not yet seen live.

Still to do before Phase 12:
- **Demo database:** delete the "Light copy", "Light copy 2" and "Dark copy" themes now that the presets carry them.
- **Luke: experiment with the preset colours**, then a UI testing pass, then Phase 12.

### Phase 12 — MCP server
**Progress (2026-10-17):** **Phase 12 is complete.** Checked with headless Claude Code (2.1.292) driving the release build's `blotter mcp` on a scratch database: it listed the agenda; in Propose mode (the default) its create_item came back "proposed, not applied" (one `mcp` proposal, no item, empty change log); with `ai_mode` set to auto the same call created the item, logged as `mcp`, after a "Before AI batch" snapshot. Changes reaching the running app within a second was verified live in #29a and that code is unchanged (not re-run: a second instance would hand off to Luke's open app). Claude Desktop wasn't tried (the AC takes either client). MCP setup is in the README (#44).

**Moved:** the MCP server is built in Phase 8 as #29a (D29). This phase keeps its acceptance checks with Claude Desktop and the README notes.
**Intent:** a stdio MCP server over the same operation registry (§4.8).

**AC:**
- Claude Desktop or Claude Code can list the agenda and propose or create items.
- Changes appear in the running app without a restart.
- The propose/auto setting is respected.

### Phase 13 — Packaging and release
**Progress (2026-10-17):** In code: Windows NSIS (per-user) and MSI installers build locally (`Blotter_1.0.0_x64-setup.exe`, `…_en-US.msi`); the MSI's contents were extracted and the packaged exe served MCP (v1.0.0, 28 tools). `.github/workflows/release.yml` builds both plus the AppImage, `.deb` and Flatpak bundle into a draft release on a `v*` tag. Flatpak manifest, AUR `PKGBUILD`, desktop entry, AppStream metadata, README and LICENSE are in `packaging/` and the root; the version is 1.0.0. Flatpak support in code: Claude Code through `flatpak-spawn --host` (opt-in override), `flatpak run` MCP commands, a hand-written autostart entry, and portal file pickers on Linux. **Still needs Luke:** push, then tag `v1.0.0` to run the release workflow (Linux bundles can't be built on this machine: no WSL distro or Docker); the AC's clean-VM install of every artifact and the Flatpak smoke checklist (notifications, tray, keychain, autostart, `--capture`, ICS fetch); the AUR checksum (`updpkgsums`) once the tag exists; Anthropic's terms on driving Claude Code from another app before publishing (D29); and the ICS checks deferred to Monday (Phase 9).
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
- Show the Blotter Light, Blotter Dark (default), Studio Light and Studio Dark presets.
- Show visible focus states throughout, since the app is keyboard-first.
- Components should map onto shadcn/Radix primitives wherever possible, so the design is implementable without custom primitives.

---

## 10. Post-v1 (parked)
- ~~**Self-hosted sync:** run a Blotter server on a home server and connect each device to it from settings.~~ **Moved into v1.2.1 (§14), with the server as its own project, Blotter Sync.**
- Two-way CalDAV sync (iCloud, Fastmail, Nextcloud) and the Google Calendar API.
- Auto-update via the Tauri updater, plus code signing.
- Splitting a task across multiple slots.
- Using the GlobalShortcuts portal for native hotkeys on Wayland.
- Driving other agent CLIs (GitHub Copilot CLI) headless with Blotter's MCP server, the way Claude Code is driven (Codex CLI moved into v1.1.0, §12). Desktop chat apps (ChatGPT, the Copilot app) can't be driven this way.
- **Propose a meeting by email** (Luke, 2026-10-20). Possible through public APIs: Gmail API `users.messages.send` (OAuth, `gmail.send` scope; a restricted scope, so a published app needs Google's verification) and Microsoft Graph `sendMail` (OAuth, `Mail.Send`). Both send an `.ics` `METHOD:REQUEST` attachment so the recipient gets a real invite. A no-account fallback: write the invite `.ics` and open a `mailto:` draft with it attached by hand, or SMTP with an app password.

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
| D15 | Planning hours concept (default 07:00–22:00) to bound flexible slotting | Confirmed |
| D16 | One slot per task in v1 (no splitting) | Confirmed |
| D17 | No Tauri updater or code signing in v1 | Confirmed |
| D18 | MIT licence | Confirmed |
| D19 | Name: **Blotter** | Confirmed |
| D20 | Diary view (day columns of task cards + today timeline rail) replaces the Today agenda as the home view | Confirmed |
| D21 | Visual direction: Linear's base design language, Notion Calendar's colour coordination and calendar layout, Sunsama-style diary view | Confirmed |
| D22 | No glass/translucent theme or window effects in v1 | Confirmed |
| D23 | Dark as default theme; monospace for times, durations and keyhints; AI proposals drawn "in pencil" | Confirmed |
| D24 | Ctrl+Z undoes the user's and the AI's changes, newest first; automatic system changes (rollover) are skipped. Theme changes aren't undoable (display preference, outside the change log) | Confirmed |
| D25 | Snapshots: keep the last 20 automatic plus all manual; stored in a `snapshots` folder next to the database by default, relocatable in settings | Confirmed |
| D26 | JSON import replaces everything, after an automatic snapshot (merge is post-v1) | Confirmed |
| D27 | AI providers authenticate with API keys (Anthropic Console, OpenAI), not consumer-app sign-in; Anthropic defaults to Claude Sonnet 5.5; other defaults picked from each provider's live model list at implementation. GitHub Models can be offered as an OpenAI-compatible preset | Confirmed |
| D28 | Daily briefing off by default; 07:30 when turned on | Confirmed |
| D29 | AI runs through two routes. **Claude Code** (the user's installed, signed-in `claude` CLI, driven headless with Blotter's MCP server) is the tested, primary route and uses the user's existing subscription. **API-key providers** (Anthropic, OpenAI, OpenAI-compatible/Ollama) are built and unit-tested against recorded responses but labelled "Experimental — untested" until checked with a real key. The MCP server moves from Phase 12 into Phase 8 (#29a), since the Claude Code route needs it. Check Anthropic's terms on driving Claude Code from another app before publishing that route | Confirmed |
| D30 | Live tests of the Claude Code route run on Luke's subscription. MCP and AI writes stay on before #32, applying directly with a snapshot before the first change of each session or turn, plus undo. Snapshot policy is the implementer's call (the test data isn't precious) | Confirmed |
| D31 | The Tauri updater ships in v1.2.0 (§13): its minisign key needs no OS code signing; package-manager installs (.deb, Flatpak, AUR) only get a notice. Supersedes the updater half of D17 | Confirmed |
| D32 | Self-hosted sync (§14) through **Blotter Sync**, a headless Blotter server in its own repo (`LASR-0/Blotter-Sync`, `../Blotter-sync` on Luke's machine) with its own version and roadmap. Blotter 1.2.1 makes the app and core sync-ready; Blotter Sync 1.0.0 is the first server. Data is no longer only per device once a user pairs a server; Local only stays the default. Amends D4 | Confirmed (Luke, 2026-10-22) |
| D33 | Undo and redo work **per field group**: an edit made since (on another device, by the server's rollover, or a later local edit undone out of order) is kept unless it touched a group the undone batch changed, in which case undo refuses as before. Creating and deleting a row, and settings, stay all or nothing. Amends D24 | Confirmed (Luke, 2026-10-23) |

---

## 12. v1.1.0 (Luke, 2026-10-20)

**Progress (2026-10-20):** **Phase 14 is complete in code** (#46–#64, with logo follow-ups #63a–#63g). Codex was tried live in the app on a scratch copy of the demo data (chat, a resumed follow-up that changed an item, and Smart slot); the other items pass their tests but most still await a look in the running app (each ledger entry lists its live check). Version 1.1.0; Luke tags `v1.1.0` and pushes to run the release workflow, then runs `updpkgsums` for the AUR.

Phase 14. Same protocol as v1 (§7): one ledger entry per increment in `COMMITS.md`, committed locally, Luke pushes. Items are grouped so the quick fixes land first and the scheduling-model changes, which share a data-model change, land together.

### 14a — Fixes and small features
- **Held arrows keep moving.** Holding an arrow (or `h j k l`, Shift+arrow, Alt+arrow) repeats focus moves, item moves and resizes. Today `useKeymap` drops every auto-repeated key. Only movement actions repeat; everything else still fires once per press.
- **Resize beats move.** Dragging the bottom edge of a Week or timeline card resizes it every time. Today the focus bar drawn over the 6px resize strip has no handler, so a press there starts a move. The resize strip grows to a comfortable hit area and sits above the focus bar.
- **Year view Shift+←/→.** Reported as skipping over a month. Reproduce first; the intended behaviour is that Shift+←/→ moves focus a month (same day, clamped to the month's length) and Shift+↑/↓ a row of months.
- **Brain-dump draft survives tab switches.** Text typed into the Inbox's brain-dump box (and the Diary rail's) is kept while switching views, until it is added or cleared. Session only, not persisted to disk.
- **Brain-dump preview.** Each line in the brain-dump box shows the same parse chips as quick add, including the event/fixed reading (dentist tue 3pm → event), so you can see what will be created before pressing Enter.
- **Week view add.** After Enter in the Week view's add box, the "new item here" indicator clears, so the created event is visible and it's clear it was made.
- **Setup screens use the logo.** First-run setup shows the Blotter mark instead of the "B" tile (the current mark until the new logo lands in 14f, then that).

### 14b — Tags and menus
- **Tags in Settings.** A Tags page lists every tag with its colour and use count: rename (merging into an existing tag), recolour, delete (removes it from items), create. New ops `rename_tag`, `set_tag_color`, `delete_tag`, logged and undoable like other mutations.
- **Themed right-click menu.** A context menu drawn with the theme (Radix ContextMenu) on cards, calendar items and days, replacing the browser's: open, edit, complete, schedule/unschedule, move to Inbox, delete, and for days "add here" and "lift block-out on this day". Text fields keep the native menu (copy/paste/spellcheck).

### 14c — Drag everywhere
- Tasks, events and reminders drag in the **Month view** (between days, keeping the time of day), the **Week view's all-day lane**, and to and from the Inbox rail where it is open. The same drop rules as now: flexible items are refused inside block-outs.

### 14d — Scheduling model
- **Task context.** Free text attached to a task that Smart slot and Consult me read when placing it. It reuses `notes` (no schema change). Quick-add/brain-dump syntax, proposed: everything after `//` on the line is the context, e.g. `groceries ~1h #home // need the car, shops shut 9pm`. Shown as a chip in the preview.
- **Holidays.** A new block-out kind for date ranges (e.g. 22 Dec – 4 Jan). Flexible tasks are never slotted inside, they're shown as a band across the days in every view, and by default other block-outs (work) are lifted on those days. Proposed quick setup in Settings → Block-outs.
- **Task windows.** A task can carry a window: earliest and latest date it may be done (e.g. "book holiday" between 1 and 15 Nov), optionally a time-of-day window. Every slotter (auto-slot, rollover reslot, Smart slot, drag) keeps it inside the window. If its slot passes undone, it is replanned to the next free slot inside the window; if the window has ended it goes back to the Inbox flagged overdue.
- **Weekly tasks.** A repeating flexible task (groceries, meal prep, laundry) with a home day and a usual time window, e.g. "Saturday, 9–12, once a week". Each week's occurrence is slotted on its home day in that window; if it passes undone it is replanned to the next suitable day and time that week (same time window first, then any planning hours), so it is done some time in the week. It's a task window that repeats, so it shares the 14d machinery. Quick-add syntax proposed: `laundry weekly sat 9-12 ~1h`.

### 14e — AI
- **Codex sessions.** OpenAI's Codex CLI driven headless with Blotter's MCP server, the way Claude Code is (§D29): status card in Settings → AI, chat, Smart slot, briefing. Needs Codex installed and signed in on the test machine (it isn't on this one yet).
- **Consult me.** A third slot mode beside Auto-slot and Smart slot. The AI goes through the Inbox one task at a time in the chat, asking the questions that matter for placing it (how long, by when, where, what it depends on, best time of day), writes the answers into the task's context and fields, then slots it. Propose mode applies as usual.

### 14f — Brand
- **New logo** from Luke's two concepts (`.claude/LogoRedesign1.png`, `LogoRedesign2.png`), traced to vectors and tried in every preset's logo colours (study: https://claude.ai/artifact/FJ37WR7jJ1hwoQntHT9vFE). After Luke picks: `public/blotter.svg`, `BlotterMark`, regenerated desktop icons, setup screens.
- **v1.1.0** release commit.

### Decisions (Luke, 2026-10-20)
1. Logo: concept A, traced closer to the reference: larger window, a centred spark whose lower point cuts into the B, a gradient from logo accent to logo page above a solid shaded underside (`--logo-shade`). The official colourway (from Luke's "Blotter Light copy") is in every preset and the icon: accent #fe5858, page #ffbf52, shade #f05656, wordmark #ffbf52 on #fe5858. The app/taskbar icon has no tile.
2. Context syntax: `//`, stored in the notes.
3. Holidays: a kind of block-out with its own table; making one lifts the block-outs it collides with; it is removed once its last day has passed; every task may go on a holiday except `#work` ones; `#holiday` tasks are placed first.
4. A task window that closes undone: back to the Inbox, overdue.
5. Codex: the user's ChatGPT plan through the Codex CLI (0.163.0 tested), signed in with `codex login`.

### Moved to post-v1.1 (§10)
- Home-server sync across devices (already §10).
- Sending a meeting proposal by email from Blotter.

---

## 13. v1.2.0 (Luke, 2026-10-21)

**Progress (2026-10-21):** **Phase 15 is complete** (#65–#71, with #70a). Each item was checked live on a scratch copy of the demo data (see #70a). Version 1.2.0; Luke added the updater signing secrets, tags `v1.2.0` and pushes, publishes the draft once the smoke checks pass, then sets the AUR checksum. 1.2.1 will be the first release installed copies can update to by themselves.

Phase 15. Same protocol as v1 (§7). Smallest first; the updater is last because it changes the release workflow and needs a signing key only Luke can make.

### 15a — New item button
- A **+** button in the Diary's Inbox rail, both open and collapsed, that opens the full item dialog (the `item.new` action) so every field and kind is in reach.
- The same button in the top bar for the Week and Month views (which have no rail), prefilled with the focused day.

### 15b — Tags that share time
- A tag setting, **"Tasks with this tag can be done simultaneously"** (Settings → Tags), e.g. laundry or shopping during a home task like cleaning.
- A task carrying such a tag may be slotted over other *tasks*, doesn't make its own time busy for other tasks, and isn't flagged as a conflict. It still stays out of block-outs and off events and other fixed items, so laundry never lands on the dentist.
- Every slotter honours it (auto-slot, rollover, drag, Smart slot's guidance), and the views lay overlapping cards side by side as they already do.

### 15c — Public holidays
- **Holidays stop lifting Sleep-type block-outs.** A holiday (made by hand or imported) lifts only the block-outs that permit fixed items; ones that warn about fixed items (Sleep) stay in place.
- **Import public holidays** from Nager.Date (free, no key, around 100 countries with subdivisions such as AU-VIC), fetched in Rust. The country, and the region where the zone tells it (Australia/Melbourne → AU-VIC), are guessed from the OS time zone and confirmed in a picker.
- Offered as a new first-run step and in Settings → Block-outs → Holidays. Imported days are ordinary holidays (lift block-outs, keep `#work` tasks off, the holiday tag in every view). A "keep up to date" option fetches next year's before this year's run out, since past holidays are pruned.

### 15d — In-app updates
- The Tauri updater. Its signing is a free minisign key pair, separate from OS code signing, so it doesn't need the signing infrastructure D17 was waiting on (D31).
- **Windows (NSIS) and AppImage:** "Blotter 1.2.1 is out" with **Restart to update** (download, install, relaunch).
- **.deb, Flatpak, AUR:** the package manager owns updates, so they get the notice and a link to the release notes only.
- A "Check for updates" setting (on by default) and a palette action. `release.yml` builds the updater artifacts and `latest.json`; it is only visible once Luke publishes the draft release.
- **Luke:** `pnpm tauri signer generate`, then the private key and its password as the `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` repo secrets; the public key goes in `tauri.conf.json`.

### Decisions (Luke, 2026-10-21)
1. Reverse block-outs ("only slot in this window") are dropped: planning hours already do this.
2. Holidays don't lift block-outs that warn about fixed items (Sleep), for hand-made and imported holidays alike.
3. Tasks that share time overlap other tasks, never events or block-outs.

---

## 14. v1.2.1 — Sync with Blotter Sync (Luke, 2026-10-22)

**Progress (2026-10-22):** planned (#72). Nothing built yet.

Phase 16. Blotter 1.2.1 makes every desktop able to sync through **Blotter Sync**, a headless Blotter that runs on a home server (Docker, or natively on Linux and Windows). The server is its own project in its own repo, `LASR-0/Blotter-Sync` (`../Blotter-sync` on this machine), with its own version (1.0.0 first), roadmap (`.claude/server-roadmap.md` there) and ledger. **This section is the source of truth for everything in this repo:** the sync model, the core groundwork and the app's side. The server roadmap covers the server itself (its API behaviour, pairing and tokens, storage, headless duties, deployment, the Home Assistant add-on). Same protocol as v1 (§7), in this ledger (#72 onward).

1.2.1 is also the first release that installed copies of 1.2.0 update to by themselves (§13), so it must be solid in Local only mode: nothing changes for users who never open Settings → Sync.

### 14.1 Goals
- **One planner across devices.** Every Blotter desktop (Windows, Linux) connected to the same server sees the same items, reminders, tags, block-outs, holidays, subscriptions, custom themes and keybindings, within seconds when online.
- **Offline-first stays true.** A device never needs the server. Edits made offline are kept and pushed when the server is reachable again.
- **Edits merge per field group.** Retitling a task on one device and rescheduling it on another, while both are offline, keeps both edits.
- **Two modes:** **Local only** (today's behaviour, the default) and **Home server** (pair once, then an ordered list of addresses tried in turn).
- **The server owns the automatic duties** (rollover, subscription refresh, holiday pruning and public-holiday upkeep) once a device is paired; daily local backups continue.
- Out of scope: phones, tablets, web UI, end-to-end encryption (the server must read the planner to run rollover), AI on the server, multiple users per server.

### 14.2 Compatibility with Blotter Sync
The two repos release separately, so compatibility is explicit:
- `blotter-core::sync` defines two constants: the **sync protocol** version (wire types and endpoint behaviour) and the **sync schema** version (synced tables and columns, pinned by a registry test so a migration that changes a synced table can't land without bumping it; D-S15 in the server roadmap). Every request carries both; the server refuses a mismatch with a typed error that names both sides, and the app says which side to update.
- The server depends on `blotter-core` from this repo as a git dependency pinned to a Blotter release tag. So a Blotter release that changes either version needs a matching Blotter Sync release; one that doesn't (UI, device-local tables) keeps working with the same server.
- `blotter-core` stays free of tokio, axum and reqwest: networking lives in `src-tauri` and in the server.
- **The compatibility table** lives in both READMEs, identical, and here. Release order: Blotter tags first, then Blotter Sync pins that tag and tags its own release.

| Blotter | Blotter Sync | Sync protocol | Sync schema |
|---|---|---|---|
| 1.2.1 and later 1.2.x | 1.0.x | 1 | 1 |

Blotter 1.2.0 and earlier have no sync.

### 14.3 What already supports this (verified in 1.2.0)
- `blotter-core` has **no Tauri dependency**; the server embeds it: same schema, migrations, operations, validation and slot engine.
- **Every id is a ULID** (`id.rs` was written "for post-v1 sync"), so devices create rows offline without collisions.
- **User, AI and MCP edits go through the operation layer**, in one transaction with change-log rows carrying the actor and a timestamp (`ops/mod.rs`, `Ctx::record`).
- **Undo already refuses** when an entity changed after the batch it would reverse (`ops/undo.rs`), so remote edits are protected once undo is scoped per device.
- The network-free halves of subscriptions (`subscriptions::due`, `record_fetch`) and public holidays (`public_holidays::select`) are already in the core; the scheduling around them is in `src-tauri`.
- `BLOTTER_DATA_DIR` lets two app instances run side by side for live tests (check the single-instance plugin doesn't block a second instance with a different data dir).

### 14.4 What breaks once data syncs (and where this plan fixes it)
| # | Problem today | Fix |
|---|---|---|
| H1 | The change log is an undo log: local rowid order, no device id, whole-entity snapshots. | A separate capture layer and outbox with per-field-group HLCs (16a). The change log stays local. |
| H2 | **Several writes bypass the operation layer**: `store::settings::set` from commands, first run, AI route settings and reminders; keybindings; themes; subscriptions; `alerts::mark_fired`; snapshot restore and JSON import (`export::replace_all`). The MCP server also writes from another process. | Capture at the database layer with SQLite triggers (D-S7), so every write path and every process is caught. |
| H3 | Rollover runs on every device; two devices would both move tasks, and `reslot` isn't deterministic across replicas. | In Home server mode only the server runs rollover. |
| H4 | Undo reverses the newest batch globally, so Ctrl+Z on one device could undo another device's edit. | Undo/redo consider only batches made on this device (16a; amends D24). |
| H5 | Snapshot restore and JSON import replace the whole database. | In Home server mode both become server actions that bump the epoch; devices re-bootstrap (16b). |
| H6 | Reminder state (`fired_at`, `snoozed_until`) is per device but in a shared table. | `fired_at` stays device-local; snooze syncs (D-S9). |
| H7 | Per-field merging can produce rows that break CHECK constraints (one device moves `start_at` later, another moves `end_at` earlier). | Coupled columns merge as field groups (§14.5, D-S8), and merged rows are validated. |
| H8 | `blockout_exceptions` has `UNIQUE (blockout_id, date)`; two devices lifting the same day make two rows with different ULIDs. | Deterministic resolution of natural-unique collisions (§14.5). |
| H9 | Natural-key tables (tags by name, settings by key, keybindings by action) would clash when a second device joins with its own copy. | First-join choice with explicit rules for natural keys (§14.7). |
| H10 | First run creates default block-outs and settings, so a second device that runs setup and then joins would duplicate them. | First run offers "Connect to a home server", which pairs and bootstraps instead (16b). |
| H11 | Devices on different versions would apply rows the other can't read. | Protocol and sync-schema gating at every request (§14.2). |
| H12 | Two devices both doing public-holiday upkeep import the same holidays twice. | The server owns upkeep; merge dedupes `public:*` holidays (§14.7). |

### 14.5 Sync model
**Shape (D-S3):** state-based sync with last-writer-wins per field group, ordered by HLC.
- Devices **push** captured changes (row id, field groups with their values and HLCs, tombstones) and **pull** everything after their cursor (the last server seq applied).
- The **server is a replica with authority over order.** It merges pushes with the same core merge function devices use, assigns each accepted change a seq, and broadcasts a notice. Devices merge pulled changes with the same function, so every replica converges.
- Replaying *operations* on other replicas is ruled out: operations read the clock and current state (slotting would land differently). CRDT libraries are more machinery than a single-user planner needs.

**Clocks:**
- One HLC per device (wall-clock ms + counter + device id), persisted, advanced on every local write and on every remote change received (`max(local, remote) + 1`). Ties break by device id.
- Every replica keeps, per row and field group, the HLC that last set it (a clocks table beside the data). A tombstone is a clock entry on the whole row.

**Field groups (D-S8):** columns related by a CHECK constraint, a foreign-key pair or an invariant an operation enforces form one group and merge together. The implementer derives the full list from the migrations and the ops and pins it in the sync registry, with a test that fails when a migration adds a column the registry doesn't classify. Expected groups:

| Table | Groups (others are groups of one) |
|---|---|
| `items` | **timing** {all_day, start_at, end_at, start_date, end_date, tz, duration_min}; **status** {status, completed_at}; **kind** {kind, anchoring}; **window** {not_before, due_at, window_start, window_end}; **series** {parent_id, recurrence_id, rrule}; `ics_uid` |
| `reminders` | **trigger** {offset_min, at}; `snoozed_until`; `fired_at` excluded (device-local, D-S9) |
| `blockouts` | **hours** {start_time, end_time, tz}; **active** {active_from, active_until}; `weekdays`; `permits_fixed`; … |
| `holidays` | **range** {start_date, end_date}; `source` |

Known, accepted limit: `items.tags` is one JSON column, so two devices changing the same item's tags offline keep the newer list, not both.

**Deletes, cascades and unique keys (D-S10):**
- **Delete wins** over concurrent edits to the same row. History on the editing device says the item was deleted on the other device. Undoing the delete on the device that made it restores the row as a new write, which then wins.
- **Cascades follow the parent.** A row whose parent is tombstoned is tombstoned when merged, matching the schema's `ON DELETE CASCADE`.
- **Natural-unique collisions** (two `blockout_exceptions` rows for the same block-out and date) keep the row with the earlier HLC and tombstone the other. Every UNIQUE constraint on a synced table gets a test.

**What syncs:**
| Table | Class | Notes |
|---|---|---|
| `items`, `reminders`, `tags`, `blockouts`, `blockout_exceptions`, `holidays`, `themes`, `keybindings` | **Synced** (LWW) | `reminders.fired_at` excluded. Keybindings are in-app overrides only; the OS-level quick-capture shortcut isn't stored there (O1). |
| `ics_subscriptions` | **Synced** definition (name, url, refresh, colour, as_blockout, enabled); **server-owned** status (last_fetch_at, last_ok_at, last_status, last_error) | |
| `ics_events` | **Server-owned**, server → device only, never edited on a device | The cache stays refetchable. |
| `settings` | **Per key** (below) | |
| `changes` | Device-local | Remote changes are recorded locally with their origin device, so History can show them. |
| `proposals`, `proposal_ops`, `chat_threads`, `chat_messages`, `providers`, `snapshots` | Device-local | Keys stay in each OS keychain. |

Settings (every key in `settings.rs` is classified; a test fails on an unclassified key):
- **Synced:** `planning_hours`, `week_start`, `clock` (12/24-hour), `timezone`, `rollover_mode`, `rollover_time`, `slot_buffer_min`, `default_task_duration_min`, `default_event_duration_min`, `default_event_reminder_min`, `snooze_options_min`, `public_holidays`.
- **Device-local:** `theme` (the selected theme, O1), `ai_mode` (auto-apply on one device's AI shouldn't turn it on everywhere; it was already kept per device through restore and import), `ai_route`, `claude_code_*`, `codex_*`, `briefing_enabled`, `briefing_time`, `last_briefing_date`, `close_to_tray`, `notifications_enabled`, `catch_up_enabled`, `catch_up_hours`, `snapshot_dir`, `snapshot_keep`, `update_check_enabled`, `setup_done`, `last_reminder_check`.
- **Server-owned:** `last_rollover_date`.

**Wire types** live in `blotter-core::sync` (serde types for hello, pair, bootstrap, push, pull, the change notice and every typed error, plus the merge function), so the app and the server can't drift. The endpoint list and their server-side behaviour are in the server roadmap (§5.2 there); changing a wire type bumps the sync protocol.

### 14.6 App: modes, sync engine and security
- **Local only:** today's behaviour and the default.
- **Home server:** paired once; an **ordered list of addresses** (e.g. LAN, then Tailscale, then a Caddy domain). The engine tries them in order, keeps the first that answers, and rechecks earlier ones periodically so a laptop returning home moves back to the LAN. Tailscale addresses (100.64.0.0/10, `*.ts.net`) are recognised: when only those fail, the app asks "Is Tailscale connected on this device?".
- **Sync engine** (in `src-tauri`): push the outbox, pull from the cursor, apply each batch of remote changes in one transaction, emit the existing `CHANGED_EVENT` so views refresh. Triggered by a local write (debounced), an SSE notice, reconnection and a periodic fallback poll. Backoff with jitter when unreachable; never blocks the UI; one status, not a toast per attempt. Applying remote changes never re-captures them into the outbox (a connection-scoped flag the triggers check, D-S7).
- **Token:** from pairing, stored in the OS keychain (`keychain.rs`), never in SQLite.
- **Plain HTTP rule (D-S6):** `http://` only to loopback, private LAN ranges (10/8, 172.16/12, 192.168/16, IPv6 ULA and link-local) and Tailscale's range; anything else must be `https://` with a certificate the OS trusts. No "ignore certificate errors" switch. Hostnames are judged by what they resolve to.

### 14.7 Joining and leaving
- **Empty server:** this device seeds it (pushes its whole planner).
- **Server has a planner:** the app asks:
  - **Use the server's planner** (recommended for a new device): local data is snapshotted, then replaced by a bootstrap.
  - **Merge:** ULID rows are unioned. Natural-key rows (tags, settings, keybindings, themes by id) take the server's values. Public holidays from the same `public:*` source and date are deduped.
- **First run** gains a choice at the start: set up this device, or **Connect to a home server** (pair, bootstrap, skip the setup steps that create data).
- **Disconnect** keeps the local copy, stops syncing, revokes this device's token on the server and clears the keychain entry.

### 14.8 Duties handed to the server
In Home server mode the app skips rollover, holiday pruning, public-holiday upkeep and subscription refreshes and shows the server's results instead. Local daily backups continue. Reminders and briefings stay on the device (every device notifies, D-S9). When the epoch changes, the app snapshots locally, re-bootstraps and tells the user; outbox changes that couldn't be pushed are kept in that snapshot. Known limit: a device that can't reach the server sees no rollover until it reconnects.

### 14.9 UI
- **Settings → Sync:** mode, Pair (address + code), the address list (add, reorder, remove, test), this device's name, the live route and last sync ("Synced via Tailscale · 2 min ago"), the server's version, Disconnect.
- **Status indicator** in the shell (sidebar or top bar, following the design language): synced, syncing, offline (with unsent count), needs attention (version mismatch with which side to update, revoked, re-bootstrap).
- **History** labels changes from other devices with the device name; Undo never offers them.
- **Snapshots** in Home server mode lists the server's snapshots; restore and JSON import go to the server. Local restore is disabled there, with a line explaining why.
- Every new screen has empty, loading and error states and full keyboard use (§6).

### 14.10 Quality bar (on top of §6)
- **Convergence tests in the core:** N in-memory replicas and an in-memory server, random interleavings of operations, offline periods, reordered and duplicated deliveries, then sync → every replica byte-identical in synced tables. Property-based (proptest), seeded so failures replay.
- **Constraint tests:** every field group and UNIQUE resolution has a test that would otherwise produce an invalid row.
- **Registry tests:** fail when a migration adds a table, column or setting the registry doesn't classify, or changes a synced one without bumping the sync schema.
- **Local only is unchanged:** every existing test passes as it is.
- **Live check before release:** two app instances with different `BLOTTER_DATA_DIR`s against a Blotter Sync build (natively on Windows; the Docker image in WSL).

### 16a — Sync groundwork in the core (no visible change)
**Intent:** everything a replica needs to take part in sync, with no networking: device identity, HLC, the sync registry, change capture, the merge function and wire types, device-scoped undo, and the network-free duty code the server reuses.

**AC:**
- A migration adds device identity, the HLC state, per-row/per-group clocks and the outbox. Existing databases upgrade without losing data; existing rows get clocks stamped at migration time.
- Every write path in H2, including a write from a second connection (the MCP server), lands in the outbox with correct groups and HLCs. Writes to device-local tables and columns never do.
- The merge function applies remote changes with LWW per group, tombstones, cascades and unique-collision rules; convergence and constraint tests pass (§14.10).
- Undo and redo work per field group (D33, #79d): an edit made since to another group of the same row stays; one to a group the batch changed refuses the undo.
- Undo and redo only touch batches made on this device; remote changes appear in History with their origin and are never undoable.
- Subscription refresh and public-holiday upkeep can be driven by a caller that does its own HTTP (the app now, the server later), with the decisions about what is due in the core.
- In Local only mode the app behaves exactly as 1.2.0.

**Leads:**
- Triggers vs a store-layer hook: triggers catch the MCP process and every direct `store::` call; confirm the cost on the demo data is negligible.
- HLC persistence must survive a crash between writing data and writing the clock (same transaction).
- `changes` needs a device column (or origin) for H4; check `newest_undoable` / `newest_redoable` and History's queries.
- The server takes this crate as a git dependency: keep its public API for sync deliberate (`blotter-core::sync`), since Blotter Sync builds against tagged releases.

### 16b — Sync in the app
**Needs:** Blotter Sync far enough along to pair, push and pull against (its ledger #03–#06), for live checks.

**Intent:** the app pairs with a server, syncs in the background, hands its automatic duties to the server and shows where things stand.

**AC:**
- Pair from Settings → Sync and from first run's "Connect to a home server"; the token is in the OS keychain.
- Address failover: with LAN, Tailscale and Caddy listed, the app syncs via whichever answers first and returns to LAN when it's back.
- The plain HTTP rule refuses public `http://` addresses with a clear message.
- Two instances edit offline, reconnect, and both show both edits (different groups) or the newer one (same group).
- In Home server mode the app doesn't run rollover, holiday pruning, public-holiday upkeep or subscription refreshes.
- Joining a server with a planner offers Use the server's / Merge, and both behave as §14.7.
- The status indicator shows each state, including a version mismatch naming the side to update; History shows device names; Snapshots uses the server's.
- Disconnect leaves a working Local only planner.

**Leads:**
- The single-instance plugin with two `BLOTTER_DATA_DIR`s on Windows and Linux, for the live tests.
- Flatpak: network access to LAN and tailnet addresses from the sandbox (`--share=network`) and the keychain portal storing the token.
- Reqwest SSE: a long-lived streaming GET with reconnect; the existing `ai/sse.rs` parser may be reusable.

### 16c — Release
- The README gains "Sync across devices", linking to Blotter Sync, with the compatibility table (§14.2).
- AppStream release note; version 1.2.1 everywhere; live checks recorded in the release entry.
- Blotter tags `v1.2.1` before Blotter Sync tags 1.0.0 (it pins this tag).

### After 1.2.1
- **Blotter Sync 1.1.0:** the Home Assistant add-on (server roadmap, Phase 3). No app change expected.

### Decisions (Luke, 2026-10-21 and 2026-10-22)
1. Sync is desktop-only (Windows and Linux).
2. The server is a headless Blotter, not just a relay, and is its own project and repo, Blotter Sync, versioned on its own (1.0.0 first, the HA add-on 1.1.0).
3. Conflicts resolve per field group (D-S3, D-S8); capture is by SQLite triggers (D-S7); every device fires reminders and snooze syncs (D-S9); delete wins, cascades follow the parent, natural-unique collisions keep the earlier row (D-S10).
4. Custom themes and keybindings sync; the selected theme stays per device (O1).
5. Docker is the server's primary deployment, with native Linux and Windows binaries too. The app has two sync modes, Local only and Home server, with an ordered address list.
6. A compatibility table in both repos says which versions work together.
