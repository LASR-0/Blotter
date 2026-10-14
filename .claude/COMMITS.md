# Blotter — Commit Ledger

The implementer prepares each commit; **Luke stages, commits and pushes. The implementer never does.**
Each entry lists the exact files to stage and the message to use. Files are delivered as full files.

## How to use an entry
1. Copy or replace the listed files into the repo at the given repo-relative paths.
2. Run the **Test steps**. Don't commit if anything fails; report it back instead.
3. Stage exactly the listed files (`git add <paths>`; `git rm <paths>` for deletions).
4. Commit with the given message verbatim.
5. Update the entry's status to `committed`, and note the short hash if you like.

## Entry template
```
### #NN — <short title>            [status: planned | ready | committed | superseded by #NN]
Phase: N
Message:
    <type>(<scope>): <summary>

    <optional body>
Files:
    A  path/to/new-file
    M  path/to/changed-file
    D  path/to/deleted-file
Test steps:
    - <command>  → <expected>
    - <manual check>
Notes:
    - <known gaps / follow-ups>
```

Commit types: `feat`, `fix`, `refactor`, `test`, `docs`, `build`, `ci`, `chore`.
Bug and visual fixes from Luke's feedback get their own entries (`fix(...)`), added in order. Earlier entries are never rewritten.

---

## Planned sequence
The file lists are filled in when an entry moves to `ready`. Planned commits may be split or merged; record any change here.

| # | Phase | Planned message | Status |
|---|---|---|---|
| 01 | 0 | `chore: scaffold Tauri 2 + React + Vite app with Cargo workspace` | committed `2ce6f7e` |
| 02 | 0 | `chore: add rustfmt, clippy, eslint, prettier and vitest tooling` | committed `467e1ca` |
| 03 | 0 | `ci: build and test on Windows and Linux` | committed `e9760fe` (CI green) |
| 04 | 1 | `feat(core): domain models, SQLite schema and migrations` | committed `bc79321` |
| 05 | 1 | `feat(core): operation registry with JSON schemas and change log` | committed `f405964` (CI green) |
| 06 | 2 | `feat(app): tray, close-to-tray and single-instance CLI handling` | committed `de724f7` |
| 07 | 2 | `feat(ui): shadcn/ui setup, theme tokens and built-in presets` | committed `7051ac3` (CI green) |
| 08 | 2 | `feat(ui): app layout, routing and action/keymap registry` | committed `0fbe467` |
| 09 | 2 | `feat(ui): command palette` | committed `bd6509b` |
| 10 | 3 | `feat(ui): week view time grid` | committed `9de2693` |
| 11 | 3 | `feat(ui): month and year views` | committed `e63069b` |
| 12 | 3 | `feat(ui): diary view with day columns and timeline rail` | committed `c0e2b59` |
| 13 | 3 | `feat(ui): item dialog and completion with timestamps` | committed `9c17d90` |
| 14 | 3 | `feat(ui): keyboard navigation and keyboard move/resize` | committed `a5913b0` |
| 15 | 3 | `feat(ui): drag-and-drop rescheduling in week view` | committed `e883650` (scope widened: Diary + Inbox rail too) |
| 16 | 4 | `feat(ui): inbox panel and view` | ready |
| 17 | 4 | `feat: natural-language quick add with live preview` | planned |
| 18 | 4 | `feat(app): quick-capture window, global shortcut and --capture flag` | planned |
| 19 | 5 | `feat(core): block-out windows with exceptions` | planned |
| 20 | 5 | `feat(core): slot engine with fixed/flexible rules` | planned |
| 21 | 5 | `feat(ui): block-out settings and calendar shading` | planned |
| 22 | 5 | `feat: inbox auto-slot and conflict warnings` | planned |
| 23 | 5 | `feat: end-of-day rollover` | planned |
| 24 | 6 | `feat(core): recurring items with occurrence exceptions` | planned |
| 25 | 6 | `feat(app): reminder engine and OS notifications` | planned |
| 26 | 6 | `feat: snooze, missed-reminder catch-up and autostart setting` | planned |
| 27 | 7 | `feat: undo/redo from change log` | planned |
| 28 | 7 | `feat: snapshots, restore UI and JSON export/import` | planned |
| 29 | 8 | `feat(ai): provider adapters and keychain credentials` | planned |
| 30 | 8 | `feat(ai): tool-calling loop over the operation registry` | planned |
| 31 | 8 | `feat(ui): chat panel and pop-out window` | planned |
| 32 | 8 | `feat(ai): proposal review and auto-apply mode` | planned |
| 33 | 8 | `feat(ai): daily briefing` | planned |
| 34 | 9 | `feat: ICS export and file import` | planned |
| 35 | 9 | `feat: ICS subscriptions with block-out option` | planned |
| 36 | 10 | `feat(ui): keybinding editor` | planned |
| 37 | 10 | `feat(ui): theme engine with live preview` | planned |
| 38 | 10 | `feat(ui): remaining settings pages and first-run setup` | planned |
| 39 | 11 | `feat(ui): completion history and streaks` | planned |
| 40 | 12 | `feat(mcp): MCP server over the operation registry` | planned |
| 41 | 13 | `build: Windows installers and Linux AppImage/deb bundles` | planned |
| 42 | 13 | `build: Flatpak manifest` | planned |
| 43 | 13 | `build: AUR PKGBUILD` | planned |
| 44 | 13 | `docs: README, Hyprland notes, MCP setup and keybindings` | planned |
| 45 | 13 | `chore(release): v1.0.0` | planned |

---

## Entries
<!-- Ready and committed entries go below, newest last. -->

### #01 — Scaffold Tauri 2 + React + Vite with Cargo workspace            [status: committed 2ce6f7e]
Phase: 0
Message:
    chore: scaffold Tauri 2 + React + Vite app with Cargo workspace

    Cargo workspace with the pure blotter-core crate and the Tauri app
    crate (binary `blotter`, id io.github.lasr0.Blotter). React 19 + Vite 8
    frontend on pnpm. Sets WEBKIT_DISABLE_DMABUF_RENDERER=1 on Linux unless
    the user overrides it. Also tracks the roadmap, commit ledger and
    design prototypes under .claude/.
Files:
    A  .gitattributes
    A  .gitignore
    A  Cargo.toml
    A  Cargo.lock
    A  package.json
    A  pnpm-lock.yaml
    A  index.html
    A  vite.config.ts
    A  tsconfig.json
    A  tsconfig.app.json
    A  tsconfig.node.json
    A  public/blotter.svg
    A  src/main.tsx
    A  src/App.tsx
    A  src/styles.css
    A  crates/blotter-core/Cargo.toml
    A  crates/blotter-core/src/lib.rs
    A  src-tauri/Cargo.toml
    A  src-tauri/build.rs
    A  src-tauri/tauri.conf.json
    A  src-tauri/capabilities/default.json
    A  src-tauri/src/main.rs
    A  src-tauri/src/lib.rs
    A  src-tauri/icons/32x32.png
    A  src-tauri/icons/64x64.png
    A  src-tauri/icons/128x128.png
    A  src-tauri/icons/128x128@2x.png
    A  src-tauri/icons/icon.png
    A  src-tauri/icons/icon.ico
    A  src-tauri/icons/icon.icns
    A  .claude/roadmap.md
    A  .claude/COMMITS.md
    A  .claude/Prototype Screens/   (all 19 files in the folder)
    Stage everything in one go:  git add -A
    (The tree contains only these files; `git status` should list nothing else.)
Test steps:
    - pnpm install                     → completes, no errors
    - pnpm build                       → tsc passes, vite writes dist/
    - cargo test --workspace           → 1 passed (blotter-core version_is_set)
    - pnpm tauri dev                   → a 1440×900 dark window titled "Blotter" opens,
                                         showing "Blotter / A keyboard-first daily planner"
                                         and "core v0.1.0" (proves the Rust command bridge works)
    - Try to shrink the window         → it stops at 1024×700
Notes:
    - Pinned: Tauri 2.12.3 (CLI 2.12.2, API 2.12.3), React 19.3.0, Vite 8.3.4,
      TypeScript 6.0.3 (not 7.x: typescript-eslint 8.71 supports TS < 6.1), Rust 1.98.1.
      These go into the README in #44.
    - On the KSB network, npm/pnpm need NODE_OPTIONS=--use-system-ca and cargo
      needs CARGO_HTTP_CHECK_REVOKE=false (corporate TLS inspection). CI is unaffected.
    - Icons were generated from public/blotter.svg with `pnpm tauri icon`.

### #02 — Lint, format and test tooling            [status: committed 467e1ca]
Phase: 0
Message:
    chore: add rustfmt, clippy, eslint, prettier and vitest tooling

    Workspace lint policy: clippy::all warn, unwrap/expect warn outside
    tests, unsafe_code denied (one audited exception for the Linux WebKit
    env var). ESLint 10 flat config with typescript-eslint and react-hooks,
    Prettier, Vitest 5. `pnpm check` runs typecheck, lint, format check and
    tests. run() now reports a startup failure instead of panicking.
Files:
    M  Cargo.toml
    M  crates/blotter-core/Cargo.toml
    M  src-tauri/Cargo.toml
    M  src-tauri/src/lib.rs
    M  package.json
    M  pnpm-lock.yaml
    M  tsconfig.node.json
    A  .editorconfig
    A  .prettierrc.json
    A  .prettierignore
    A  clippy.toml
    A  rustfmt.toml
    A  eslint.config.js
    A  vitest.config.ts
    A  src/lib/duration.ts
    A  src/lib/duration.test.ts
    M  .claude/COMMITS.md
    Stage everything in one go:  git add -A
Test steps:
    - pnpm install                                        → completes
    - pnpm check                                          → tsc, eslint, prettier all clean; Vitest 8 passed
    - cargo fmt --all --check                             → no output, exit 0
    - cargo clippy --workspace --all-targets -- -D warnings → finishes with no warnings
    - cargo test --workspace                              → 1 passed
    - pnpm tauri dev                                      → same window as #01 still opens
Notes:
    - Cargo repository URL corrected to github.com/LASR-0/Blotter (capital B, matching the remote).
    - src/lib/duration.ts (formatDuration → "45m", "1h 30m") is the first real util;
      the Diary time meter and cards will use it.
    - The Linux-only cfg block is not compiled on Windows; #03's ubuntu CI job is
      what proves it is clippy-clean.

### #03 — CI on Windows and Linux            [status: committed e9760fe, CI green]
Phase: 0
Message:
    ci: build and test on Windows and Linux

    GitHub Actions on push to main, PRs and manual dispatch. A frontend
    job runs `pnpm check` and the Vite build. A Rust matrix on
    windows-latest and ubuntu-latest builds the frontend (generate_context
    embeds dist/), then runs clippy -D warnings and the workspace tests;
    rustfmt runs once on Linux. Installs the WebKitGTK 4.1, appindicator
    and xdo dev packages on Ubuntu.
Files:
    A  .github/workflows/ci.yml
    M  .claude/COMMITS.md
    Stage everything in one go:  git add -A
Test steps:
    - pnpm format:check        → clean (Prettier also checks the workflow YAML)
    - Commit and push, then open the repo's Actions tab
                               → the "CI" run shows three jobs: Frontend, Rust (windows-latest),
                                 Rust (ubuntu-latest), all green
    - The first run is slow (~10–15 min for the Rust jobs); rust-cache makes
      later runs much faster.
Notes:
    - Actions pinned to majors: checkout@v7, setup-node@v7, pnpm/action-setup@v6
      (reads pnpm 12.3.4 from package.json "packageManager"), Swatinem/rust-cache@v2,
      dtolnay/rust-toolchain@stable.
    - The Ubuntu job is the first time the Linux-only WebKit workaround in
      src-tauri/src/lib.rs gets compiled and linted. If it fails, send me the log
      and I'll add a fix entry.
    - Bundling (installers) is not built in CI yet; that arrives in Phase 13.

### #04 — Domain models, SQLite schema and migrations            [status: committed bc79321]
Phase: 1
Message:
    feat(core): domain models, SQLite schema and migrations

    Initial migration with every v1 table from roadmap §4.4 (items, tags,
    reminders, block-outs + exceptions, ICS subscriptions + cached events,
    change log, proposals, snapshots, chat, providers, settings,
    keybindings, themes). CHECK constraints guard invariants such as
    done <=> completed_at and exclusive reminder offset/at.

    blotter-core gains models (Item, Reminder, Blockout, Weekdays, Change,
    Actor), canonical UTC text encoding for instants, ULID ids, and a
    storage layer of plain functions over a Connection. The app opens the
    database in the OS app-data directory in WAL mode on startup.
Files:
    A  crates/blotter-core/migrations/0001_initial.sql
    M  crates/blotter-core/Cargo.toml
    M  crates/blotter-core/src/lib.rs
    A  crates/blotter-core/src/db.rs
    A  crates/blotter-core/src/error.rs
    A  crates/blotter-core/src/id.rs
    A  crates/blotter-core/src/time.rs
    A  crates/blotter-core/src/model/mod.rs
    A  crates/blotter-core/src/model/item.rs
    A  crates/blotter-core/src/model/reminder.rs
    A  crates/blotter-core/src/model/blockout.rs
    A  crates/blotter-core/src/model/change.rs
    A  crates/blotter-core/src/store/mod.rs
    A  crates/blotter-core/src/store/items.rs
    A  crates/blotter-core/src/store/reminders.rs
    A  crates/blotter-core/src/store/blockouts.rs
    A  crates/blotter-core/src/store/changes.rs
    A  crates/blotter-core/src/store/settings.rs
    M  src-tauri/src/lib.rs
    M  src/App.tsx
    M  src/styles.css
    M  Cargo.lock
    M  .claude/COMMITS.md
    Stage everything in one go:  git add -A
Test steps:
    - cargo test --workspace                                  → 28 passed in blotter-core
    - cargo clippy --workspace --all-targets -- -D warnings  → no warnings
    - cargo fmt --all --check                                 → clean
    - pnpm check                                              → clean, 8 passed
    - pnpm tauri dev → the placeholder screen now shows a second line:
          db C:\Users\ROXBLUK\AppData\Roaming\io.github.lasr0.Blotter\blotter.db · wal · schema v1
      and that folder contains blotter.db, blotter.db-wal and blotter.db-shm.
    - Close and relaunch → same line, no error (migrations are idempotent).
Notes:
    - rusqlite 0.40 (bundled SQLite) + rusqlite_migration 2.6 chosen over sqlx:
      synchronous, no async runtime in the core, trivially testable in memory.
    - Instants are stored as `YYYY-MM-DDTHH:MM:SSZ` text so SQL string comparison
      equals time order; all-day items use start_date/end_date (end inclusive).
    - Writes use INSERT … ON CONFLICT DO UPDATE, never REPLACE, so restoring an
      item on undo can't cascade-delete its reminders (covered by a test).
    - ulid 3 renamed Ulid::new() to Ulid::generate().
    - Recurring-series expansion in range queries arrives in Phase 6; for now a
      series item appears at its first occurrence only.
    - The db status line on the placeholder screen is temporary; the real app
      layout replaces it in #08.

### #05 — Operation registry, JSON Schemas and change log            [status: committed f405964, CI green]
Phase: 1
Message:
    feat(core): operation registry with JSON schemas and change log

    One registry of typed operations serves the UI now and the AI loop and
    MCP server later. Each op has a name, description, read/mutate flag and
    a JSON Schema generated from its argument type (schemars); arguments are
    validated with jsonschema before the handler runs, and every violation
    is reported so a model can correct its call.

    Mutations run in one transaction with their change-log rows (full
    before/after JSON, actor, shared batch id). Deletes log cascaded
    reminders, overrides and exceptions so undo can restore them.

    Ops: list_agenda, get_item, search_items, list_inbox, list_blockouts,
    get_day_summary, create/update/delete/complete/uncomplete/schedule/
    unschedule_item, create/update/delete_blockout, add/remove_blockout_
    exception, set_reminder, clear_reminder.

    The app holds a Planner (OS timezone as default), exposes list_ops and
    run_op, and emits planner:changed after each committed mutation.
Files:
    M  crates/blotter-core/Cargo.toml
    M  crates/blotter-core/src/lib.rs
    M  crates/blotter-core/src/error.rs
    M  crates/blotter-core/src/model/mod.rs
    M  crates/blotter-core/src/model/blockout.rs
    A  crates/blotter-core/src/settings.rs
    A  crates/blotter-core/src/planner.rs
    A  crates/blotter-core/src/ops/mod.rs
    A  crates/blotter-core/src/ops/input.rs
    A  crates/blotter-core/src/ops/items.rs
    A  crates/blotter-core/src/ops/reads.rs
    A  crates/blotter-core/src/ops/reminders.rs
    A  crates/blotter-core/src/ops/blockouts.rs
    M  src-tauri/Cargo.toml
    M  src-tauri/src/lib.rs
    A  src-tauri/src/commands.rs
    A  src/lib/api.ts
    M  src/App.tsx
    M  src/styles.css
    M  Cargo.lock
    M  .claude/COMMITS.md
    Stage everything in one go:  git add -A
Test steps:
    - cargo test --workspace                                  → 62 passed in blotter-core
    - cargo clippy --workspace --all-targets -- -D warnings  → no warnings
    - cargo fmt --all --check                                 → clean
    - pnpm check                                              → clean, 8 passed
    - pnpm tauri dev:
        · first line reads "core v0.1.0 · 20 operations"
        · type "Call plumber" in the Inbox box, press Enter → it appears in the list
        · tick it → struck through; untick → back to normal
        · type only spaces and press Enter → nothing happens
        · close and relaunch → the item is still there (persistence)
    - Optional, to see the change log: open %APPDATA%\io.github.lasr0.Blotter\blotter.db in
      DB Browser for SQLite → table `changes` has one row per action above, actor `user`,
      with before/after JSON.
Notes:
    - Acceptance for Phase 1 (roadmap §8): every §4.3 op except find_free_slots and
      auto_slot (Phase 5) is implemented and tested; every mutation writes change rows
      (one test drives all 11 mutating ops); DB lives in app-data with WAL.
    - Added remove_blockout_exception (not in §4.3) so a lifted date can be un-lifted
      without deleting the whole block-out.
    - Naive times in arguments ("2026-10-20T15:00") are read in the user's timezone;
      DST gaps move forward, overlaps take the earlier instant (tested on Melbourne).
    - The "flexible never inside a block-out" rule is enforced in Phase 5 (#20/#22),
      when the slot engine lands. Until then schedule_item accepts any time.
    - rrule is only shape-checked (FREQ=…); real parsing arrives in Phase 6.
    - The Inbox box on the placeholder screen is temporary and goes away in #08.

### #06 — Tray, close-to-tray and single-instance CLI            [status: committed de724f7]
Phase: 2
Message:
    feat(app): tray, close-to-tray and single-instance CLI handling

    Closing the main window hides it; the process stays in the tray so
    reminders can keep firing. The tray icon opens Blotter on left-click and
    its menu has Open and Quit. A second launch hands its arguments to the
    running instance (single-instance plugin) and exits.

    CLI flags: --capture and --chat bring the window forward and emit an
    app:cli event (the capture popup and chat consume it in #18 and #31);
    --hidden starts in the tray without showing a window, for autostart.
    The main window now starts hidden and is shown in setup, so hidden
    launches never flash.
Files:
    A  src-tauri/src/cli.rs
    A  src-tauri/src/shell.rs
    M  src-tauri/src/lib.rs
    M  src-tauri/Cargo.toml
    M  src-tauri/tauri.conf.json
    M  Cargo.lock
    M  .claude/COMMITS.md
    Stage everything in one go:  git add -A
Test steps:
    - cargo test --workspace                                  → 62 core + 3 app (cli) passed
    - cargo clippy --workspace --all-targets -- -D warnings  → no warnings
    - pnpm tauri dev, then by hand:
        · the window opens as before, and a Blotter icon appears in the tray
          (may be under the ^ overflow arrow)
        · click the window's X → window disappears, tray icon stays, process still in Task Manager
        · left-click the tray icon → window comes back, focused
        · close it again, right-click tray → "Open Blotter" restores it
        · right-click tray → "Quit Blotter" → app exits fully (gone from Task Manager)
    - Single instance (while `pnpm tauri dev` is running), in a second terminal:
          ./target/debug/blotter.exe
        → no second window; the existing one comes to the front; the second process exits
    - Hidden start: quit, then run  ./target/debug/blotter.exe --hidden  (with `pnpm dev`
      running in another terminal for the frontend) → only the tray icon appears; click it
      to open the window.
Notes:
    - Verified here: a second launch with --chat leaves exactly one blotter.exe running.
    - Quick capture (#18) and chat (#31) don't exist yet, so --capture/--chat currently
      just bring the window forward; the app:cli event is already emitted for them.
    - Linux tray uses libayatana-appindicator (already in CI deps); the Flatpak tray
      (StatusNotifier) is verified in Phase 13.
    - "Quit" in the command palette arrives with the palette in #09.

### #07 — shadcn/ui setup, theme tokens and built-in presets            [status: committed 7051ac3, CI green]
Phase: 2
Message:
    feat(ui): shadcn/ui setup, theme tokens and built-in presets

    Tailwind v4 with the design reference's token set as the theme: the
    token names (--color-surface, --radius-md, --text-sm, --font-mono…) are
    Tailwind namespaces, so utilities like bg-surface read the live CSS
    variable and a theme switch is just new variable values. Setting tokens
    on any subtree re-themes only that subtree (for the Phase 10 preview).
    shadcn/ui names (background, primary, ring…) are aliased onto them.

    Dark, Light, Paper and Gruvbox presets ported from the design, applied
    before first paint from a local cache and persisted in settings through
    allowlisted get_setting/set_setting commands. Geist, Geist Mono and
    Newsreader are bundled (offline). First shadcn-style components:
    Button, Kbd/KeyHint and Tooltip on Radix.
Files:
    M  package.json
    M  pnpm-lock.yaml
    M  vite.config.ts
    M  .prettierrc.json
    M  src/styles.css
    M  src/main.tsx
    M  src/App.tsx
    A  src/lib/cn.ts
    A  src/theme/presets.ts
    A  src/theme/theme.ts
    A  src/theme/theme.test.ts
    A  src/components/ui/button.tsx
    A  src/components/ui/kbd.tsx
    A  src/components/ui/tooltip.tsx
    M  src-tauri/src/commands.rs
    M  src-tauri/src/lib.rs
    M  src-tauri/Cargo.toml
    M  Cargo.lock
    M  .claude/COMMITS.md
    Stage everything in one go:  git add -A
Test steps:
    - pnpm install                → completes
    - pnpm check                  → clean; Vitest 14 passed (8 duration + 6 theme)
    - cargo test --workspace      → 62 core + 3 app passed; clippy clean
    - pnpm tauri dev:
        · the placeholder screen is restyled: big "Blotter" heading, a Dark/Light/Paper/Gruvbox
          switch, a Save/Cancel/Skip button row, a sparkle icon button, key chips
        · click each preset → the whole window recolours instantly; Paper switches the heading
          to a serif
        · hover the sparkle button → inverted tooltip "Toggle AI chat  Ctrl /"
        · pick Gruvbox, close the window from the tray (Quit), relaunch → it opens straight in
          Gruvbox with no dark flash
        · Tab through the controls → every one shows the 2px focus ring
    - Inbox box still works as in #05 (add, tick, untick).
Notes:
    - Verified by screenshot here: Paper renders with Newsreader headings, Geist Mono for
      the mono lines, accent-filled primary button, and correct chips and input styling.
    - The preset switch on this screen is temporary; #09's command palette gets
      "Theme: …" commands and #08 replaces this screen with the real layout.
    - Only Button, Kbd and Tooltip so far; further shadcn components (Dialog, Command,
      Switch, Select, Popover…) are added in the commits that first use them.
    - prettier-plugin-tailwindcss now sorts class names; `pnpm format` may reorder classes.

### #08 — App layout, routing and action/keymap registry            [status: committed 0fbe467]
Phase: 2
Message:
    feat(ui): app layout, routing and action/keymap registry

    The real app shell from the design: a 232px sidebar (view nav with
    shortcut hints and Inbox count, block-out list, Settings) that
    collapses to a 52px icon rail below 1180px or with Ctrl+B, and a 48px
    top bar with period navigation, the period title, a chord hint chip and
    the Diary|Calendar and W|M|Y switches.

    A single action registry (id, label, group, default binding, context)
    drives a pure, tested keymap engine: modifier bindings, chords such as
    "g w" with a 1.2s timeout, view contexts, and plain keys ignored while
    typing. The same list will feed the command palette in #09.

    Routing is a small persisted nav store (view, calendar view, period
    anchor, sidebar). Backend reads go through TanStack Query and refetch on
    every planner:changed event. Diary shows the day's items, Inbox has the
    brain-dump input and list, Settings has Theme (preset cards rendered in
    their own theme) and About. Week, Month, Year and History show a
    placeholder until their commits land. Replaces the scaffold screen.
Files:
    D  src/App.tsx
    M  src/main.tsx
    M  package.json
    M  pnpm-lock.yaml
    A  src/app/AppShell.tsx
    A  src/app/Sidebar.tsx
    A  src/app/TopBar.tsx
    A  src/app/nav.ts
    A  src/actions/actions.ts
    A  src/keymap/keymap.ts
    A  src/keymap/keymap.test.ts
    A  src/keymap/useKeymap.ts
    A  src/lib/period.ts
    A  src/lib/period.test.ts
    A  src/lib/queries.ts
    A  src/lib/types.ts
    A  src/lib/format.ts
    A  src/components/EmptyState.tsx
    A  src/components/ItemCard.tsx
    A  src/components/ui/segmented.tsx
    A  src/views/DiaryView.tsx
    A  src/views/InboxView.tsx
    A  src/views/PendingView.tsx
    A  src/views/SettingsView.tsx
    M  .claude/COMMITS.md
    Stage everything in one go:  git add -A   (picks up the src/App.tsx deletion too)
Test steps:
    - pnpm install               → completes
    - pnpm check                 → clean; Vitest 31 passed (13 keymap, 4 period, 6 theme, 8 duration)
    - cargo test --workspace     → unchanged, all pass
    - pnpm tauri dev, keyboard only:
        · g then d / w / m / y / h → Diary / Week / Month / Year / History; while waiting for
          the second key a "g … d w m y h" chip shows in the top bar
        · i → Inbox (i again → back to Diary); type a task, Enter → it appears; Diary sidebar
          badge shows the open count
        · v → toggles Diary ↔ the last calendar view
        · [ and ] → previous/next day (Diary), week (Week)… title updates; . → back to today
        · Ctrl+, → Settings; click Light/Paper/Gruvbox cards → app recolours; About shows the db path
        · Ctrl+B → sidebar collapses to icons (hover shows label + shortcut); Ctrl+B again expands
        · Narrow the window below ~1180px → sidebar collapses on its own
        · Click in the Inbox input and type "g w" → it types; no navigation
    - Close and relaunch → it reopens on the view you left (period resets to today)
Notes:
    - Verified here with real keystrokes and screenshots: g d, g w and Ctrl+, all navigate;
      preset cards each render in their own theme (scoped tokens work).
    - Week/Month/Year/History bodies are placeholders until #10, #11 and #39; their
      navigation, titles and shortcuts are real now.
    - Not yet in the shell (arrive with their features): search/palette button (#09),
      Quick add (#17), chat toggle (#31), Calendars & tags list (#21/#35).
    - Diary is a single day list for now; the multi-column Diary with the time meter
      and timeline rail is #12.

### #09 — Command palette and shortcut sheet            [status: committed bd6509b]
Phase: 2
Message:
    feat(ui): command palette

    Ctrl+K opens a cmdk palette in a Radix dialog, styled to the design:
    grouped actions (Go to, View, Actions, Theme) with icons and key chips,
    fuzzy matching on labels and keywords, and an Items group that searches
    titles, notes and tags through the search_items operation (debounced).
    Choosing an item jumps to its day, or to the Inbox.

    ? opens the keyboard-shortcut sheet, generated from the action
    registry's live bindings. New actions: Command palette (Ctrl+K),
    Keyboard shortcuts (?), Quit Blotter (Ctrl+Q, via a quit_app command).
    While an overlay is open only overlay-safe bindings respond. The
    sidebar gains the "Search or command" button.
Files:
    A  src/app/CommandPalette.tsx
    A  src/app/ShortcutSheet.tsx
    A  src/app/overlay.ts
    M  src/app/AppShell.tsx
    M  src/app/Sidebar.tsx
    M  src/app/nav.ts
    M  src/actions/actions.ts
    M  src/keymap/useKeymap.ts
    M  src/components/ui/kbd.tsx
    M  src-tauri/src/commands.rs
    M  src-tauri/src/lib.rs
    M  package.json
    M  pnpm-lock.yaml
    M  .claude/COMMITS.md
    Stage everything in one go:  git add -A
Test steps:
    - pnpm install              → completes
    - pnpm check                → clean; Vitest 31 passed
    - cargo clippy --workspace --all-targets -- -D warnings → clean
    - pnpm tauri dev, keyboard only:
        · Ctrl+K → palette opens with the cursor in the search box
        · type "wee" → Week is first and selected; Enter → Week view, palette closes
        · Ctrl+K, type "pap", Enter → Paper theme applied
        · Ctrl+K, ↑/↓ moves the highlight (wraps at the ends); Esc closes
        · Ctrl+K twice → opens then closes
        · add an Inbox task "Call plumber"; Ctrl+K, type "plum" → it appears under Items;
          Enter → Inbox view
        · while the palette is open, typing "g" or "i" just types (no navigation)
        · ? → shortcut sheet with Go to / View / Actions columns; Esc closes
        · Ctrl+Q → Blotter quits completely (gone from the tray and Task Manager)
    - Click "Search or command" in the sidebar → palette opens
Notes:
    - Verified here with real keystrokes and screenshots: Ctrl+K opens the palette,
      "wee" ranks Week first, Esc/? open the sheet, Ctrl+Q exits the process.
    - Fixed during testing: rows first matched on "group + label", which made
      "wee" fuzzy-match unrelated rows; they now match on label + keywords only.
      The sheet no longer opens with a focus ring on its Esc button.
    - cmdk moves the highlight to whatever row the mouse is over; that is normal
      behaviour, but it means a resting mouse over the palette can change the
      selection while you type.
    - The design's palette also lists Quick add, Daily briefing, Auto-slot, Undo,
      Snapshot and Toggle chat; they join the registry with their features (#17, #33,
      #22, #27, #28, #31). "Rebind in Settings → Keybindings" on the sheet comes with #36.

### #10 — Week view time grid            [status: committed 9de2693]
Phase: 3
Message:
    feat(ui): week view time grid

    The Week view from the design: day headers with today's accent pill, an
    all-day lane where multi-day items span columns and stack into rows, and
    a 24-hour grid (44px per hour, dashed half-hours) that opens at 07:00 or
    just before now. Items are tinted by tag colour with a 3px edge, done
    items are muted and struck through, overdue tasks show in danger red,
    short items collapse to one line, and overlapping items share the width
    in lanes. Weekends use the surface tone, today's column is tinted, and a
    red now-line with a time chip tracks the current minute.

    Overlap lanes and all-day rows are computed by pure, tested layout
    functions; tag colours map known tags to theme tokens and others to a
    stable palette pick.

    Dev tooling: BLOTTER_DATA_DIR points the app at another data folder, and
    `cargo run -p blotter-core --example seed -- <db>` fills a database with
    a demo week through the operation layer.
Files:
    A  src/views/WeekView.tsx
    A  src/lib/layout.ts
    A  src/lib/layout.test.ts
    A  src/lib/tags.ts
    A  src/lib/tags.test.ts
    A  src/lib/useNow.ts
    M  src/lib/period.ts
    M  src/app/AppShell.tsx
    A  crates/blotter-core/examples/seed.rs
    M  crates/blotter-core/Cargo.toml
    M  src-tauri/src/lib.rs
    M  Cargo.lock
    M  .claude/COMMITS.md
    Stage everything in one go:  git add -A
Test steps:
    - pnpm check                → clean; Vitest 39 passed (+5 layout, +3 tags)
    - cargo test --workspace    → 62 core + 3 app passed; clippy clean
    - Demo data on a scratch database (leaves your real data alone), in Git Bash:
          export BLOTTER_DATA_DIR="$TEMP/blotter-demo"
          cargo run -p blotter-core --example seed -- "$BLOTTER_DATA_DIR/blotter.db"
          pnpm tauri dev
      then g w:
        · Mon–Sun headers with today's date in an accent circle; "Bin night" on Wednesday and
          "Mum visiting" spanning Saturday–Sunday in the all-day lane
        · Thursday 15:00 Dentist and 15:30 Parent–teacher interview sit side by side
        · tasks before today are struck through; today's 07:15 Morning run (if past) is red
        · a red line with a time chip marks now in today's column; Sat/Sun are slightly lighter
        · [ and ] move by a week; the grid keeps its morning scroll position
      (Run `unset BLOTTER_DATA_DIR` afterwards, or open a fresh terminal, to get your real data back.)
    - Without the env var, your own items appear in the week they're scheduled.
Notes:
    - Verified here by screenshot against the seeded week: lanes, all-day spanning, done,
      overdue, compact items and the now-line all render as designed.
    - Display only in this commit: keyboard focus and moving (#14), drag and drop (#15),
      opening items (#13), block-out shading (#21) and conflict flags (#22) come next.
    - The grid assumes 24-hour days; on the two DST-change days an hour is drawn slightly off
      (items still sit on the correct day). Revisit if it matters in practice.
    - Week starts on Monday (WEEK_STARTS_ON); it becomes a setting with General in #38.

### #11 — Month and Year views            [status: committed e63069b]
Phase: 3
Message:
    feat(ui): month and year views

    Month: a 6×7 grid from the design with up to three chips per day (mono
    time or "all day", tag-tinted, done items struck through), "+N more"
    opening a day popover that lists every item with its time range and an
    Open in Week link. Out-of-month days use the surface tone, weekends a
    half tint, the first of a month reads "Oct 1", today gets the accent
    pill, and clicking a day number opens its week.

    Year: twelve mini-months that fill the window, each day a heatmap cell
    (design thresholds 0 / <3 / <5 / <7 / 7+), with a Scheduled load |
    Completed toggle, a Less–More legend, per-month counts and a yearly
    total. Today is outlined, the current month is accented, and clicking a
    day (tooltip: "Tue 20 Oct · 6 items, 4h 10m") opens its week.

    Shared, tested day bucketing places timed items on every local day they
    touch and all-day items on each of their dates. Adds a Radix popover
    component and an openWeek(date) navigation action.
Files:
    A  src/views/MonthView.tsx
    A  src/views/YearView.tsx
    A  src/lib/days.ts
    A  src/lib/days.test.ts
    A  src/components/ui/popover.tsx
    M  src/app/AppShell.tsx
    M  src/app/nav.ts
    M  src/views/SettingsView.tsx
    M  .claude/COMMITS.md
    Stage everything in one go:  git add -A
Test steps:
    - pnpm check                → clean; Vitest 43 passed (+4 days)
    - With the demo database from #10 (export BLOTTER_DATA_DIR=..., pnpm tauri dev):
        · g m → October grid; this week's days show chips; "Oct 1" and "Nov 1" labelled;
          today in an accent circle; 28–30 Sep and 2–8 Nov dimmed
        · click "+3 more" on Tuesday → popover lists all 6 items with time ranges;
          "Open in Week" jumps to that week; Esc or × closes it
        · click a day number → that week opens
        · [ / ] → previous/next month
        · g y → 12 mini-months; this week's cells shaded; today outlined; October accented;
          "30 scheduled in 2026" top right
        · toggle Completed → only days with completions stay shaded
        · hover a day → tooltip with count and planned time; click → its week
        · maximise the window → the year grid grows to fill it
    - Settings → About: the "B" tile is the full 24px size again.
Notes:
    - Verified here by screenshot (Month, Year, and the day popover via a posted click).
    - Fixed while testing: today's pill pushed its cell's chips 5px lower than its
      neighbours (cell headers now have a fixed height); `text-2xl` silently did nothing
      because Blotter's type scale replaces Tailwind's, which also affected the About
      logo tile from #08 — both now use the design's explicit 24px.
    - Completed mode counts items scheduled in the shown year by their completion day;
      completions of unscheduled Inbox items join when History (#39) adds its query.
    - Keyboard navigation of days (arrows, Enter → Week) comes with #14; block-out strips
      and conflict icons in Month with #21/#22.

### #12 — Diary view with day columns and timeline rail            [status: committed c0e2b59]
Phase: 3
Message:
    feat(ui): diary view with day columns and timeline rail

    The home view from the design. A collapsible Inbox rail with a
    brain-dump input and the open Inbox tasks. Day columns from the anchor
    day onwards, scrolling sideways: today is first and widest with a 44px
    accent date, a Today badge and a raised ground; each column has a
    planned-vs-free meter (free = planning hours 07:00–22:00 minus that
    day's block-outs, red above 90%), design cards and an inline "Add task"
    that creates a task on that day. Today's column opens with an Overdue
    section. A timeline rail shows the anchor day's hours with block-out
    hatching and labels, timed items (pinned when fixed inside a block-out)
    and the now-line.

    Core: new list_overdue read op (open tasks left on an earlier day or
    past their deadline); get_day_summary's overdue_count now uses the same
    definition. Free-time maths (block-outs wrapping midnight, lifted dates,
    active ranges) is a pure, tested module. Cards now follow the design's
    meta grammar (07:15 · 30m, 09:00–09:15, due Mon 12 · 20m, pin,
    recurring, tag dots). The seed leaves one task overdue for the demo.
Files:
    A  src/views/diary/DiaryView.tsx
    A  src/views/diary/DayColumn.tsx
    A  src/views/diary/InboxRail.tsx
    A  src/views/diary/TimelineRail.tsx
    D  src/views/DiaryView.tsx
    A  src/lib/availability.ts
    A  src/lib/availability.test.ts
    M  src/lib/format.ts
    M  src/components/ItemCard.tsx
    M  src/app/AppShell.tsx
    M  src/app/nav.ts
    M  crates/blotter-core/src/ops/reads.rs
    M  crates/blotter-core/src/store/items.rs
    M  crates/blotter-core/examples/seed.rs
    M  .claude/COMMITS.md
    Stage everything in one go:  git add -A
Test steps:
    - pnpm check                → clean; Vitest 48 passed (+5 availability)
    - cargo test --workspace    → 63 core (+1 overdue rules) + 3 app passed; clippy clean
    - Reseed the demo database (the seed changed):
          export BLOTTER_DATA_DIR="$TEMP/blotter-demo"; rm -rf "$BLOTTER_DATA_DIR"
          cargo run -p blotter-core --example seed -- "$BLOTTER_DATA_DIR/blotter.db"
          pnpm tauri dev      then  g d
        · today's column: Today badge, OVERDUE "Renew car registration · due Mon 12 · 20m" in red,
          then TODAY cards; meter reads planned vs free (a weekday shows ≤ 6h 30m free)
        · following days scroll horizontally; ] moves the start day forward, . back to today
        · Timeline rail: Sleep hatched at both ends, Work 08:30–17:00 hatched and labelled,
          Standup has a pin, red now-line on today
        · "Add task" under a column → type → Enter → card appears in that column
        · Inbox rail: type in "Brain-dump a task…" → appears in the rail and the sidebar count;
          « hides the rail to a slim strip (stays hidden after restart), click it to reopen
        · tick a card → struck through with ✓ time; untick restores it
Notes:
    - Verified here by screenshot with the demo data (it also showed the Light theme
      correctly): overdue section, meters matching the design's demo numbers, rail hatching.
    - Dragging cards between days / onto the timeline joins drag and drop in #15;
      card focus, x/e/Enter keys in #14; "in Work" pin labels and conflict lines in #21/#22;
      the rail's Auto-slot button arrives with auto-slot in #22.
    - "Add task" makes an all-day task on that day (no time yet), which is how a day-only
      task is stored. Planning hours are fixed at 07:00–22:00 until the General settings (#38).

### #13 — Item dialog and completion with timestamps            [status: committed 9c17d90]
Phase: 3
Message:
    feat(ui): item dialog and completion with timestamps

    The item dialog from the design: heading-font title; Kind (task, event,
    reminder) and Anchoring (flexible, fixed, with an explanation); When as
    date, typed time and duration (no date = Inbox, date only = all-day);
    an optional due date; priority; tag chips; reminders (offsets before
    the start); notes. The header shows Recurring and the completion time
    for done items; the footer has a two-click Delete, who created it and
    when it was last edited, Cancel and Save (Ctrl+Enter).

    Edits become a minimal batch (only changed fields; unschedule when the
    date is cleared; reminder adds/removes) sent through a new run_batch
    command, so one Save is one atomic, undoable change-log batch. Draft
    building, validation and time/duration parsing ("5:30pm", "1h30") are
    pure, tested modules.

    Double-click opens an item from Diary cards, Week events, Month chips,
    the timeline and Inbox rails; Month popover rows and palette item
    results open it on click/Enter; "New item" is in the palette.
Files:
    A  src/app/ItemDialog.tsx
    A  src/lib/itemDraft.ts
    A  src/lib/itemDraft.test.ts
    A  src/lib/format.test.ts
    M  src/lib/format.ts
    M  src/lib/duration.ts
    M  src/lib/duration.test.ts
    M  src/lib/api.ts
    M  src/app/overlay.ts
    M  src/app/AppShell.tsx
    M  src/app/CommandPalette.tsx
    M  src/actions/actions.ts
    M  src/components/ItemCard.tsx
    M  src/views/WeekView.tsx
    M  src/views/MonthView.tsx
    M  src/views/diary/InboxRail.tsx
    M  src/views/diary/TimelineRail.tsx
    M  src-tauri/src/commands.rs
    M  src-tauri/src/lib.rs
    M  .claude/COMMITS.md
    Stage everything in one go:  git add -A
Test steps:
    - pnpm check                → clean; Vitest 88 passed (+11 duration, +10 time, +8 draft)
    - cargo test --workspace / clippy → unchanged, clean
    - pnpm tauri dev (demo DB or your own):
        · double-click a Diary card → dialog opens with title focused and fields filled
        · change the title, press Ctrl+Enter → saved, card updates everywhere
        · set Time to "5:30pm", tab away → shows 17:30; type "1h30" in duration → Save →
          the card reads 17:30 · 1h 30m
        · clear the date → hint "it goes to the Inbox"; Save → it moves to the Inbox
        · add a reminder ("10m before") on a timed item → Save → reopen → it's listed; × removes it
        · add tags by typing and pressing Enter; Backspace in the empty tag box removes the last
        · Ctrl+K → "New item" → fill in → Save creates it
        · Delete → button says "Click again to delete" → click → gone
        · try saving an event with no date → inline error, nothing saved
    - Ctrl+K, type part of a title, Enter on the result → opens that item's dialog.
Notes:
    - Verified here: opened Gym via the palette, edited the title, saved; the Week view
      refreshed with the new title. The design screenshots matched in Gruvbox and Dark.
    - Time is a typed 24-hour field rather than <input type="time">, because WebView2
      follows the Windows locale (it showed "05:30 PM"). The date keeps the native picker,
      which uses your regional dd/mm/yyyy.
    - Repeats editing and the "this / following / all occurrences" choice come with
      recurrence in #24; for now a recurring item shows its rule read-only.
    - Undo of a Save (or Delete) arrives with undo/redo in #27; each Save is already a
      single batch, so it will undo in one step.

### #14 — Keyboard navigation and keyboard move/resize            [status: committed a5913b0]
Phase: 3
Message:
    feat(ui): keyboard navigation and keyboard move/resize

    Roving keyboard focus in every navigable view, per roadmap §5.4 and the
    design annotations. Arrows or h j k l move focus: Diary card to card
    and column to column; Week to the nearest item by time, with Tab
    switching to a half-hour grid cursor (Enter there opens a new item at
    that slot); Month and Year by day and week, following into the next
    month or year, Enter opening the day's week (Year shows the focused
    day's tooltip); Inbox up and down.

    On the focused item: Enter or e opens it, x toggles completion, Del
    deletes after a confirm dialog (Enter confirms, Esc cancels).
    Shift+arrows move it by a day or 15 minutes and Alt+↑/↓ resize it, all
    through update_item so moves are validated and change-logged. A single
    click focuses, a double click opens. The Week view gains the design's
    key-hint and status footer, and focused events show the resize handle.

    Each view registers a small controller; the shared actions live in the
    registry under a new "Focused item" group, active only in navigable
    views. Actions can carry alternate bindings (ArrowUp / k), Enter and
    Space keep their native meaning on buttons, and the shortcut sheet
    shows each key family as one row. Focus movement and move/resize
    argument building are pure, tested modules.
Files:
    A  src/app/focus.ts
    A  src/app/ConfirmDialog.tsx
    A  src/actions/itemActions.ts
    A  src/lib/focusNav.ts
    A  src/lib/focusNav.test.ts
    A  src/lib/itemMoves.ts
    A  src/lib/itemMoves.test.ts
    M  src/actions/actions.ts
    M  src/keymap/useKeymap.ts
    M  src/app/AppShell.tsx
    M  src/app/overlay.ts
    M  src/app/CommandPalette.tsx
    M  src/app/ShortcutSheet.tsx
    M  src/components/ItemCard.tsx
    M  src/views/diary/DiaryView.tsx
    M  src/views/InboxView.tsx
    M  src/views/WeekView.tsx
    M  src/views/MonthView.tsx
    M  src/views/YearView.tsx
    M  .claude/COMMITS.md
    Stage everything in one go:  git add -A
Test steps:
    - pnpm check                → clean; Vitest 101 passed (+6 focus, +7 move/resize)
    - With the demo DB, mouse untouched:
        Diary (g d): j/k/h/l (or arrows) move the ring between cards and columns; x ticks the
          focused card; e or Enter opens it; Shift+→ moves it to the next day (keeps its time);
          Shift+↓ makes a timed card 15m later; Del → "Delete …?" → Esc keeps it
        Week (g w): j / l move by time and day; footer shows "focus: … · Tue 07:15–07:45";
          Shift+↓ moves 15m, Shift+→ a day; Alt+↓ / Alt+↑ resize (never below 15m);
          Tab → cursor box "HH:MM · ↵ to add"; arrows move it by 30m / a day; Enter → New item
          pre-filled with that day and time; Tab again → back to the item you had
        Month (g m): arrows move the ring by day/week, crossing into the next month; Enter → that week
        Year (g y): arrows move the ring with a tooltip ("Thu 22 Oct · … ↵ Week"); Enter → week
        Inbox (i): j/k move between tasks; x / e / Del work
    - Typing in any text field (Inbox input, dialog) never moves focus or triggers x/e/Del.
    - Tab on a focused button outside the Week grid still moves browser focus normally; Enter on
      a focused button presses it.
    - ? → the sheet has a "Focused item" column with one row each for Move focus, Move item,
      Resize item, plus Open, Edit, Toggle complete, Delete.
Notes:
    - Verified here with real and posted key events and screenshots: Diary column jumps, Week
      nearest-by-time, x toggle, Tab cursor and Enter → prefilled New item, Shift+↓ ×2 moved an
      item 07:15 → 07:45, Year focus with tooltip.
    - Fixed while testing: in cursor mode Enter opened the previously focused item instead of
      creating at the cursor; item focus is now parked while the cursor is active.
    - Diary Shift+↑/↓ shifts timed cards by 15m; reordering untimed cards within a day comes
      with drag and drop in #15.
    - Focus is logical (store + ring), not DOM focus, so screen readers don't yet follow it;
      aria-activedescendant wiring is a follow-up worth doing before v1.

### #15 — Drag-and-drop rescheduling            [status: committed e883650]
Phase: 3
Message:
    feat(ui): drag-and-drop rescheduling in week view and diary

    Pointer-driven drag and drop following the design's drag annotations.
    A press becomes a drag after 4px, so clicks and double clicks still
    work; the source stays as a 45% dashed placeholder and a tilted ghost
    card follows the pointer; Esc cancels and scroll areas follow the
    pointer near their edges.

    Week: drag an event to a new time or day (dashed slot snapped to 15
    minutes with its time range) or drag its bottom edge to resize. Diary:
    drag cards between day columns (2px accent insert line; timed cards
    keep their time, untimed ones are reordered via sort_order), onto the
    timeline rail to schedule at a time, and between the Inbox rail and the
    days in both directions. A flexible item over a block-out shows the
    refused slot ("⦸ Work — flexible") and snaps back; fixed items may drop
    inside. Every drop is one update_item / unschedule_item call.

    Core: update_item accepts sort_order. The timeline rail now lays out
    overlapping items in lanes like the Week grid. Slot snapping, refusal,
    midpoint ordering and drop-to-operation mapping are pure, tested rules.
Files:
    A  src/app/drag.ts
    A  src/app/DragLayer.tsx
    A  src/components/DropPreview.tsx
    A  src/lib/dnd.ts
    A  src/lib/dnd.test.ts
    M  src/app/AppShell.tsx
    M  src/components/ItemCard.tsx
    M  src/views/WeekView.tsx
    M  src/views/diary/DayColumn.tsx
    M  src/views/diary/DiaryView.tsx
    M  src/views/diary/InboxRail.tsx
    M  src/views/diary/TimelineRail.tsx
    M  crates/blotter-core/src/ops/items.rs
    M  .claude/COMMITS.md
    Stage everything in one go:  git add -A
    (Message differs slightly from the planned one: "...in week view and diary".)
Test steps:
    - pnpm check                → clean; Vitest 110 passed (+9 dnd)
    - cargo test --workspace    → 64 core (+1 sort_order) + 3 app; clippy clean
    - With the demo DB (reseed for a clean slate):
        Week (g w):
          · drag an evening task down an hour → dashed slot shows e.g. "20:15–20:30"; release → moved
          · drag the bottom edge of an event → it grows/shrinks in 15m steps
          · drag a flexible task (e.g. Meal prep) into Mon–Fri 09:00–17:00 → red hatched
            "⦸ Work — flexible" slot; release → it snaps back unchanged
          · drag a fixed event (e.g. Dentist) into work hours → allowed
          · start a drag, press Esc → cancelled
        Diary (g d):
          · drag a card from Thursday to Friday → insert line shows; release → it's on Friday, same time
          · drag a card onto the Timeline rail → slot with time range; release → scheduled today then
          · drag an Inbox-rail task onto a day column → becomes a day task there; Inbox count drops
          · drag a day task back onto the Inbox rail (it highlights) → back in the Inbox
          · drag an event onto the Inbox rail → no highlight, nothing happens (only tasks go there)
        · a plain click still focuses, a double click still opens the dialog
Notes:
    - Verified here with posted pointer events and mid-drag screenshots: Week move with slot
      preview, refused block-out drop snapping back, Diary cross-column insert line, timeline
      drop (Dinner moved to 21:00 and the day meter turned red), Inbox rail → day.
    - Fixed while testing: overlapping items on the timeline rail drew on top of each other;
      they now share the width in lanes.
    - Untimed day tasks always list above timed cards (they have no time to sort by), so the
      insert line's position only reorders among the untimed ones. A refinement for later:
      show the line only where an untimed card can actually go.
    - A refused drop is silent for now; it gets a toast ("Work is a block-out; flexible tasks
      can't go there") when toasts land with undo in #27.
    - Core enforcement of "flexible never in a block-out" for every path (AI, MCP, keyboard)
      arrives with the slot engine in #20.

### #16 — Inbox view with multi-select and bulk actions            [status: ready]
Phase: 4
Message:
    feat(ui): inbox panel and view

    The full Inbox from the design (V3): a header with the unscheduled
    count, a brain-dump box where Enter adds every line as its own task in
    one batch (Shift+Enter for a new line; text stays if saving fails), and
    a task table with select box, title and tags, duration, priority (!2
    amber, !3 red), due and when it was added.

    Multi-select by click or Space, Esc to clear. A floating bulk bar acts
    on the selection, each change as one undoable batch: Schedule… (d, to
    a day or a time), Duration (t), Priority (p), Due (u, set or clear) and
    Delete (Del, with a confirm). Rows keep keyboard focus (j/k), double
    click opens the dialog and rows can be dragged onto days. The Diary's
    Inbox rail from #12 remains the side panel.
Files:
    A  src/views/inbox/InboxView.tsx
    A  src/views/inbox/BulkBar.tsx
    A  src/app/selection.ts
    A  src/actions/bulkActions.ts
    D  src/views/InboxView.tsx
    M  src/actions/actions.ts
    M  src/actions/itemActions.ts
    M  src/app/AppShell.tsx
    M  src/lib/format.ts
    M  src/lib/format.test.ts
    M  .claude/COMMITS.md
    Stage everything in one go:  git add -A
Test steps:
    - pnpm check                → clean; Vitest 115 passed (+5 added-ago)
    - pnpm tauri dev (demo DB), press i:
        · type two lines in the box (Shift+Enter between), Enter → both appear, box clears
        · j to focus a row, Space selects it (accent tint, ticked box); select another
        · bulk bar shows "2 selected"; p → pick !2 → both rows show !2 in amber
        · t → type 1h → Set duration → both show 1h
        · u → pick a date → both show it under Due; u again with no date → cleared
        · d → pick a date → both leave the Inbox and appear on that day in the Diary
        · select one, Del → "Delete 1 task?" → Enter → gone
        · Esc clears the selection; leaving the Inbox clears it too
        · click a row to select it, double click to open the dialog
Notes:
    - Verified here: keyboard selection, the bulk bar, bulk priority applied to two rows in
      one batch, and a brain-dump line added from the box.
    - Not yet in the bulk bar: Auto-slot and its result panel (#22) and Ask AI (#31). The
      token hints under the box (~45m #tag !2 due fri) arrive with the parser in #17; until
      then each line is a plain title.
    - Completed Inbox tasks are hidden here; they'll show in History (#39).
