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
| 07 | 2 | `feat(ui): shadcn/ui setup, theme tokens and built-in presets` | ready |
| 08 | 2 | `feat(ui): app layout, routing and action/keymap registry` | planned |
| 09 | 2 | `feat(ui): command palette` | planned |
| 10 | 3 | `feat(ui): week view time grid` | planned |
| 11 | 3 | `feat(ui): month and year views` | planned |
| 12 | 3 | `feat(ui): diary view with day columns and timeline rail` | planned |
| 13 | 3 | `feat(ui): item dialog and completion with timestamps` | planned |
| 14 | 3 | `feat(ui): keyboard navigation and keyboard move/resize` | planned |
| 15 | 3 | `feat(ui): drag-and-drop rescheduling in week view` | planned |
| 16 | 4 | `feat(ui): inbox panel and view` | planned |
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

### #07 — shadcn/ui setup, theme tokens and built-in presets            [status: ready]
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
