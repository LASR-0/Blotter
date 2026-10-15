# Blotter — Commit Ledger

Since 2026-10-15 the implementer commits each entry itself, as Luke (no AI co-author trailer), after its test steps pass, and **never pushes**: Luke syncs from VS Code. Before that, Luke staged and committed every entry himself.
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
| 16 | 4 | `feat(ui): inbox panel and view` | committed `48863c5` |
| 17 | 4 | `feat: natural-language quick add with live preview` | committed `22e3001` |
| 18 | 4 | `feat(app): quick-capture window, global shortcut and --capture flag` | committed `0a70bd0` |
| 19 | 5 | `feat(core): block-out windows with exceptions` | committed `0ae5626` |
| 20 | 5 | `feat(core): slot engine with fixed/flexible rules` | committed `73370bb` |
| 20a | 5 | `fix(ui): give quick-add errors their own row` | committed `5f7e140` |
| 21 | 5 | `feat(ui): block-out settings and calendar shading` | committed `72a0d77` |
| 22 | 5 | `feat: inbox auto-slot and conflict warnings` | committed `8777f83` |
| 23 | 5 | `feat: end-of-day rollover` | committed `25f3f7a` |
| 24 | 6 | `feat(core): recurring items with occurrence exceptions` | committed `6ecb88a` (UI split out as #24a) |
| 24a | 6 | `feat(ui): recurring item editing` | committed `afb6690` |
| 25 | 6 | `feat(app): reminder engine and OS notifications` | committed `7ba84fe` |
| 25a | 6 | `feat(ui): Blotter logo and wordmark` | committed `980e5cf` (#25a–#25d as one commit) |
| 25b | 6 | `feat(ui): themed select, date and time pickers` | committed `980e5cf` (#25a–#25d as one commit) |
| 25c | 6 | `feat(ui): theme-coloured scrollbars` | committed `980e5cf` (#25a–#25d as one commit) |
| 25d | 6 | `chore: add pnpm dev:demo for testing against demo data` | committed `980e5cf` (#25a–#25d as one commit) |
| 26 | 6 | `feat: snooze, missed-reminder catch-up and autostart setting` | committed `67206ac` |
| 27 | 7 | `feat: undo/redo from change log` | committed `575b2c9` |
| 27a | 7 | `fix(ui): rank exact palette matches first` | committed `53ee35a` |
| 27b | 7 | `fix(core): undo skips automatic system changes` | committed `b5b6341` |
| 27c | 7 | `fix(ui): unique keys on the block-out editor panels` | committed `e16efbd` |
| 27d | 7 | `feat(ui): error boundaries instead of a blank window` | committed `4c3e5b3` |
| 28 | 7 | `feat: snapshots, restore UI and JSON export/import` | committed `92c7afe` |
| 29 | 8 | `feat(ai): provider adapters and keychain credentials` (API providers marked experimental, D29) | committed `6d6442d` |
| 29a | 8 | `feat(mcp): MCP server over the operation registry` | committed (hash in the next entry; brought forward from #40, D29) |
| 30 | 8 | `feat(ai): tool-calling loop and Claude Code provider` | planned (Claude Code provider added, D29) |
| 31 | 8 | `feat(ui): chat panel and pop-out window` | planned |
| 32 | 8 | `feat(ai): proposal review and auto-apply mode` | planned |
| 33 | 8 | `feat(ai): daily briefing` | planned |
| 34 | 9 | `feat: ICS export and file import` | planned |
| 35 | 9 | `feat: ICS subscriptions with block-out option` | planned |
| 36 | 10 | `feat(ui): keybinding editor` | planned |
| 37 | 10 | `feat(ui): theme engine with live preview` | planned |
| 38 | 10 | `feat(ui): remaining settings pages and first-run setup` | planned |
| 39 | 11 | `feat(ui): completion history and streaks` | planned |
| 40 | 12 | ~~`feat(mcp): MCP server over the operation registry`~~ | superseded by #29a |
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
    M  pnpm-lock.yaml                (everything #25a left)
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
    M  src/styles.css                (the scrollbar hunk #25a left)
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
    M  src/styles.css                (the scrollbar hunk #25a left)
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
    M  pnpm-lock.yaml                (everything #25a left)
    M  vite.config.ts
    M  .prettierrc.json
    M  src/styles.css                (the scrollbar hunk #25a left)
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
    M  pnpm-lock.yaml                (everything #25a left)
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
    M  pnpm-lock.yaml                (everything #25a left)
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

### #16 — Inbox view with multi-select and bulk actions            [status: committed 48863c5]
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

### #17 — Natural-language quick add with live preview            [status: committed 22e3001]
Phase: 4
Message:
    feat: natural-language quick add with live preview

    A quick-add bar on n (design V8). Type a line such as
    "dentist tue 3pm ~45m @fixed" and chips show what was understood
    (date, time, duration, priority, tags, anchoring, repeat, due), with a
    preview of the item and where it will land (Inbox, Diary or Calendar).
    Enter creates, Ctrl+Enter creates and opens the details, Alt+Enter
    creates and keeps the bar open. Esc closes.

    The parser (src/lib/quickParse.ts) is pure and offline: a token layer
    reads ~duration, !1-!3, #tag, @fixed/@flex and "every ..." (mapped to an
    RRULE), then chrono-node (en-GB, day first) reads dates and times from
    the rest. "due <date>" sets the deadline (17:00 when no time is given).
    Fixed rule: @fixed/@flex always win; otherwise an explicit time on an
    event-like phrase (dentist, meeting, lunch, interview...) makes a fixed
    event, and anything else stays a flexible task.

    The Inbox brain-dump uses the same grammar per line and shows the token
    hints under the box.
Files:
    A  src/lib/quickParse.ts
    A  src/lib/quickParse.test.ts
    A  src/components/QuickAddPanel.tsx
    A  src/app/QuickAddDialog.tsx
    M  src/app/overlay.ts
    M  src/app/AppShell.tsx
    M  src/actions/actions.ts
    M  src/views/inbox/InboxView.tsx
    M  package.json             (chrono-node 2.10.3)
    M  pnpm-lock.yaml                (everything #25a left)
    M  .claude/COMMITS.md
    M  .claude/roadmap.md
    Stage everything in one go:  git add -A
Test steps:
    - pnpm check                → clean; Vitest 126 passed (+11 parser)
    - pnpm tauri dev (demo DB):
        · n → the quick-add bar opens with the hint line and an "Untitled · Inbox" preview
        · type: dentist tue 3pm ~45m @fixed #health → chips for duration, anchoring, tag,
          date, time; preview "Fixed event · Tue 20 Oct 15:00–15:45", pill Calendar
        · Enter → it appears on Tuesday at 15:00
        · n, "call plumber" → pill Inbox; Alt+Enter → created, bar stays open and clears
        · n, "pay rent fri !2", Ctrl+Enter → created and the item dialog opens on it
        · i → in the brain-dump type "fix gate ~30m #home" + Enter → row shows 30m and home
Notes:
    - Verified here: pnpm check clean, the parser tests, and the n bar opening with the
      empty state matching the design.
    - STILL TO VERIFY (next session, before #18): typing a line into the live bar and
      checking chips/preview, plus Ctrl+Enter and Alt+Enter. My test script sent "~" as
      Enter, so the live run went wrong. post.ps1 now has -Literal for that.
    - A date without a time becomes an all-day item on that day; auto-slotting it into
      free time arrives with #22.
    - A recurring line with no date starts today. Recurrence expansion arrives with #24.
    - Live check, 2026-10-14 (after commit): typing "dentist tue 3pm ~45m @fixed #health" into
      the n bar showed the chips Duration 45m, Anchoring fixed, Tag health, Date Tue 20 Oct and
      Time 15:00, plus the preview "Fixed event · Tue 20 Oct 15:00–15:45" with the Calendar pill.
      Enter / Ctrl+Enter / Alt+Enter weren't pressed: the running instance was Luke's own, on
      real data. They are in #18's test steps, so they get checked on the next run.

### #18 — Quick-capture window, global shortcut and --capture            [status: committed 0a70bd0]
Phase: 4
Message:
    feat(app): quick-capture window, global shortcut and --capture flag

    A quick-capture popup (design W6): an undecorated, always-on-top
    window, 600px wide, centred and kept out of the taskbar. It holds the
    quick-add panel in capture mode, sized to its content. It is created
    hidden at startup, so it appears instantly. Enter writes the line
    through the operation layer and hides it; Alt+Enter keeps it open;
    Esc discards and hides. Clicking elsewhere hides it but keeps the
    half-typed line.

    It opens from:
    - Ctrl+Shift+Space, system-wide (global-shortcut plugin; pressing it
      again hides the popup),
    - blotter --capture, on first launch or forwarded to the running
      instance (it no longer brings the main window forward),
    - the tray menu's new "Quick capture" item,
    - the command palette.

    On Wayland the shortcut isn't registered. Settings → About explains
    this and shows a Hyprland bind for blotter --capture. A shortcut
    that fails to register (e.g. already taken) is reported there too and
    never stops the app starting.
Files:
    A  src-tauri/src/capture.rs
    M  src-tauri/src/shell.rs
    M  src-tauri/src/lib.rs
    M  src-tauri/Cargo.toml              (tauri-plugin-global-shortcut 2.3.2)
    M  src-tauri/capabilities/default.json
    M  Cargo.lock
    A  src/app/CaptureApp.tsx
    M  src/main.tsx
    M  src/components/QuickAddPanel.tsx
    M  src/actions/actions.ts
    M  src/views/SettingsView.tsx
    M  .claude/COMMITS.md
    M  .claude/roadmap.md
    Stage everything in one go:  git add -A
Test steps:
    - pnpm check                → clean; Vitest 126 passed
    - cargo clippy --workspace --all-targets -- -D warnings → clean
    - cargo test --workspace    → core unchanged; app 6 passed (+3 capture: Wayland detection,
                                  height clamp, status JSON)
    - Quit any running Blotter first (single instance), then pnpm tauri dev (demo DB):
        #17 leftovers, in the main window:
        · n, "call plumber", Alt+Enter → "Created" status, bar stays open and empty
        · n, "pay rent fri !2", Ctrl+Enter → created, item dialog opens on it
        · n, "dentist tue 3pm ~45m @fixed", Enter → on Tuesday at 15:00
        Quick capture:
        · with another app focused, press Ctrl+Shift+Space → a 600px popup titled "Quick capture"
          appears centred, on top, cursor in the box, no taskbar button
        · type "fix gate ~30m #home", Enter → popup hides; the Inbox (main window) shows it with 30m
        · open it again: the box is empty. Type something and click another window → it hides;
          Ctrl+Shift+Space → the text is still there; Esc → hides; open again → empty
        · Ctrl+Shift+Space twice → opens, then hides
        · type a long line with many tokens (e.g. "x tue 3pm ~1h #a #b #c !2 every monday due fri")
          → the chips wrap and the window grows to fit, nothing is clipped
        · in a terminal: ./target/debug/blotter.exe --capture → the popup opens; the main window
          does not come forward
        · right-click tray → "Quick capture" → the popup opens
        · Ctrl+K → "Quick capture" → the popup opens
        · switch theme to Gruvbox in the main window, then open the popup → it's in Gruvbox
        · Settings → About → "Quick capture  Ctrl Shift Space" section
        · quit, start with --capture as the first launch → only the popup shows
Notes:
    - Destination: the design labels the popup "Inbox · scheduled on triage", but the popup uses
      the same grammar as the n bar, so a line with a date/time ("dentist tue 3pm") is scheduled
      and the pill says where it will land. Plain brain-dump lines go to the Inbox. Say if you'd
      rather force everything into the Inbox in capture mode.
    - Ctrl+Enter in the popup behaves like Enter (the design has no "open details" there).
    - Not verified here: the Linux paths (X11 shortcut, the Wayland message, a Hyprland bind).
      Linux CI compiles them; a live check comes in Phase 13 with the Flatpak.
    - The shortcut is fixed at Ctrl+Shift+Space until the keybinding editor (#36) can rebind it;
      the About section moves to Settings → Keybindings then.
    - The popup opens on the monitor it last used (Tauri centres it on its current monitor), not
      necessarily the one under the pointer.
    - First committed as 9352969 with a typo in the author email (1620670+…), which credited
      an unrelated GitHub account. Rewritten with the correct email and force-pushed as 0a70bd0
      (same tree, message and dates), with Luke's OK. The contributor list may stay cached until
      GitHub Support refreshes it.
    - Verified live after commit, on a scratch demo DB: n bar Alt+Enter (created, bar stays open
      and clears) and Ctrl+Enter (created, dialog opens). Ctrl+Shift+Space from Notepad opens the
      popup focused and centred. A long line wraps the chips onto three rows and the window grows
      to fit. Esc hides and the next open is empty. Clicking away hides and keeps the text.
      Pressing the shortcut again hides it. Enter on "fix gate ~30m #home" puts it in the Inbox
      with 30m. `blotter.exe --capture` opens the focused popup without the main window.

### #19 — Block-out windows with exceptions            [status: committed 0ae5626]
Phase: 5
Message:
    feat(core): block-out windows with exceptions

    Block-outs are rules (weekdays, a wall-clock window in their own
    timezone, an active date range, lifted dates). The new windows module
    expands them into concrete UTC windows for any span. It is pure and
    tested, and the slot engine (#20), calendar shading (#21) and the AI
    will all read block-outs through it instead of each re-deriving them.

    An occurrence belongs to the local date it starts on. Weekdays, the
    active range and lifted dates are checked against that date, so
    Sleep 23:00-06:30 on Friday runs into Saturday morning, and lifting
    Friday frees the whole night. Times stay wall-clock across DST: Sleep
    is an hour shorter on the spring-forward night, and times inside the
    gap move forward, as elsewhere in the core. Windows are half-open, so
    a task may end exactly when Work starts.

    New read op list_blockout_windows(start, end?) returns the windows
    for a range of days in the user's timezone, with permits_fixed for
    each. list_agenda and the new op share one range check.
    Blockout::applies_on covers the weekday and active-range test.
Files:
    A  crates/blotter-core/src/windows.rs
    M  crates/blotter-core/src/lib.rs
    M  crates/blotter-core/src/model/blockout.rs
    M  crates/blotter-core/src/ops/reads.rs
    M  .claude/COMMITS.md
    M  .claude/roadmap.md
    Stage everything in one go:  git add -A
Test steps:
    - cargo test --workspace    → 74 core (+9 windows, +1 op) + 6 app passed
    - cargo clippy --workspace --all-targets -- -D warnings → clean; cargo fmt --check → clean
    - pnpm check                → unchanged, clean
    - Nothing visible changes in the app: the calendar still shades block-outs with the
      frontend helper until #21 switches it to list_blockout_windows.
Notes:
    - Rules the tests pin down: weekday windows in local time; midnight wrap covering both
      days; a lifted date freeing the whole occurrence (including the next morning); active
      range bounds; DST spring-forward and fall-back lengths (Melbourne); times inside the gap;
      each block-out's own timezone; half-open edges; a corrupt timezone reported, not panicking.
    - The frontend helper (src/lib/availability.ts) reads block-out times in the machine's
      zone and ignores the block-out's own tz. That only matters if they differ, and #21
      replaces it with the core windows.
    - Exceptions are one date at a time (add_blockout_exception). The settings page (#21) can
      lift a range, such as two weeks of leave, as one run_batch, so it still undoes in one step.
    - Planning hours (07:00–22:00) are still a frontend constant; they join the core settings
      with the slot engine in #20.

### #20 — Slot engine with fixed/flexible rules            [status: committed 73370bb]
Phase: 5
Message:
    feat(core): slot engine with fixed/flexible rules

    The slot engine from roadmap §4.5, pure and deterministic. Free time
    is planning hours minus block-out windows and busy time (open, timed
    items, padded by the buffer on both sides). Slots take the earliest
    fit in each free gap, with starts rounded up to 5 minutes.

    Planning hours (per weekday, default 07:00-22:00, a day can be off)
    and the slot buffer (default 0 minutes) are now core settings.

    New read op find_free_slots(duration_min, from?, until?, limit?,
    exclude_item?) searches from now (never the past) up to 14 days by
    default, or up to a deadline.

    Rule 1 is enforced on every path: create_item, update_item and
    schedule_item refuse to put an open flexible item inside a block-out,
    with a message naming the block-out and its times. This covers the
    UI, AI and MCP alike. Fixed items may go anywhere. The check only
    runs when an item's time or anchoring changes, so an item caught by
    a block-out created later can still be renamed, completed or moved
    out. Lifted dates free the time. Day-only tasks have no time and
    aren't affected.
Files:
    A  crates/blotter-core/src/slots.rs
    A  crates/blotter-core/src/ops/slots.rs
    M  crates/blotter-core/src/lib.rs
    M  crates/blotter-core/src/settings.rs
    M  crates/blotter-core/src/time.rs           (HH:MM serde moved here from the block-out model)
    M  crates/blotter-core/src/model/blockout.rs
    M  crates/blotter-core/src/windows.rs        (windows::load reads and expands stored block-outs)
    M  crates/blotter-core/src/ops/mod.rs
    M  crates/blotter-core/src/ops/items.rs
    M  crates/blotter-core/src/ops/reads.rs
    M  crates/blotter-core/src/planner.rs        (find_ is a read-op prefix)
    M  .claude/COMMITS.md
    M  .claude/roadmap.md
    Stage everything in one go:  git add -A
Test steps:
    - cargo test --workspace    → 87 core (+8 engine, +5 ops) + 6 app passed
    - cargo clippy --workspace --all-targets -- -D warnings → clean; cargo fmt --check → clean
    - cargo run -p blotter-core --example seed -- <fresh db> → still seeds cleanly
    - pnpm tauri dev (demo DB):
        · n, "meal prep thu 10am ~1h", Enter → refused in red: "… would be inside the Work
          block-out (Thu 15 Oct 08:30–17:00) …", nothing created
        · n, "dentist thu 10am", Enter → created (an event is fixed, so allowed)
        · double-click a flexible evening task, set Time to 10:00 on a weekday, Save → the
          dialog shows the same refusal
Notes:
    - Verified here: the engine and op tests, plus the live quick-add refusal above (screenshot).
    - Decisions:
      · Default buffer is 0 (roadmap: "default 0–10 minutes"). It's a setting
        (slot_buffer_min) and gets a control with General settings in #38.
      · Manual placement (dialog, drag, keyboard) only enforces block-outs. Planning hours
        and overlapping items constrain the engine's choices, not where you put something
        yourself; overlaps get flagged as conflicts in #22.
      · Slots are ranked soonest-first, one per free gap. auto_slot (#22) adds due-date and
        priority ordering on top.
    - Keyboard moves (Shift+arrows) and drags into a block-out are refused with only a console
      error until toasts arrive in #27. Drags already refuse visibly in the UI (since #15).
    - The frontend still has its own PLANNING_HOURS constant for the Diary meter; #21 moves the
      meter and shading onto core data.
    - Error text keeps the core's "invalid input:" prefix, as every op error does today.

### #20a — Quick-add errors get their own row            [status: committed 5f7e140]
Phase: 5 (fix to #17, found during #20's live check)
Message:
    fix(ui): give quick-add errors their own row

    A long error, such as the new block-out refusal, shared the footer
    with the key hints, so the hints wrapped into narrow columns. Errors
    now get a full-width row above the footer. The hints stay on one line,
    and the short "Created …" note stays inline and truncates.
Files:
    M  src/components/QuickAddPanel.tsx
    M  .claude/COMMITS.md
    Stage everything in one go:  git add -A
Test steps:
    - pnpm check                → clean; Vitest 126 passed
    - pnpm tauri dev (demo DB): n, "meal prep thu 10am ~1h", Enter → the refusal shows in red
      on its own row under the preview; "↵ Create · Ctrl ↵ Create & open details · Alt ↵ Keep
      open" stays on one line. Alt+Enter on a valid line still shows "Created …" at the right.
Notes:
    - Verified here by screenshot. Applies to the quick-capture popup too, which uses the same panel.
    - Numbered 20a so the planned sequence keeps its numbers.

### #21 — Block-out settings and calendar shading            [status: committed 72a0d77]
Phase: 5
Message:
    feat(ui): block-out settings and calendar shading

    Settings → Block-outs (design S2):
    - The list shows a hatched swatch per block-out, its days and times
      (and timezone, when it isn't yours) and whether fixed items are
      allowed or warn. Reorder by dragging or with Alt+Up/Down.
    - The editor has label, day toggles, start and end (typed, e.g.
      "5:30pm"; it notes when a window runs past midnight), an "Allow
      fixed items" switch, Save (Ctrl+Enter), Revert and a two-click
      Delete. New block-out creates one from the same form.
    - Exceptions lists lifted dates and removes them with x. Adding takes
      one date or a range with a note; a range only lifts the days the
      block-out applies on, as one undoable batch (e.g. a week of leave).

    The calendar now draws block-outs from the core (list_blockout_windows
    via one shared, cached query) instead of re-deriving them in the UI,
    so each block-out's own timezone and lifted dates are respected:
    - Week: hatched, labelled windows in every day column, and a pin on
      fixed items sitting inside one.
    - Month: a hatched bar by the date on days with a block-out during
      planning hours, with the details as a tooltip.
    - Diary: the timeline rail, the day meters and drag refusal.

    b l (or the palette) opens "Block-outs on <day>" for the focused day
    in any calendar view: a switch per block-out lifts or restores it on
    that day. Sidebar block-outs open their settings.
Files:
    A  src/views/settings/BlockoutsPage.tsx
    A  src/app/LiftDialog.tsx
    A  src/components/ui/switch.tsx
    A  src/lib/blockoutDraft.ts
    A  src/lib/blockoutDraft.test.ts
    A  src/lib/useBlockoutWindows.ts
    M  src/lib/availability.ts
    M  src/lib/availability.test.ts
    M  src/lib/dnd.ts
    M  src/lib/dnd.test.ts
    M  src/lib/types.ts
    M  src/actions/actions.ts
    M  src/app/AppShell.tsx
    M  src/app/DragLayer.tsx
    M  src/app/Sidebar.tsx
    M  src/app/drag.ts
    M  src/app/focus.ts
    M  src/app/nav.ts
    M  src/app/overlay.ts
    M  src/views/SettingsView.tsx
    M  src/views/WeekView.tsx
    M  src/views/MonthView.tsx
    M  src/views/YearView.tsx
    M  src/views/diary/DiaryView.tsx
    M  src/views/diary/TimelineRail.tsx
    M  .claude/COMMITS.md
    M  .claude/roadmap.md
    Stage everything in one go:  git add -A
Test steps:
    - pnpm check                → clean; Vitest 134 passed (+8 block-out draft; availability and
                                  dnd tests now use core windows)
    - Rust unchanged since #20.
    - pnpm tauri dev (demo DB):
        · g w → Work hatched and labelled Mon–Fri 08:30–17:00, Sleep from 23:00; Standup,
          Telehealth, Dentist and Team lunch show a pin; the weekend is clear
        · g m → a small hatched bar after the date on weekdays only
        · b l (Week, today focused) → "Block-outs on Wed 14 Oct" with Work and Sleep; Space on
          Work → Lifted; Esc → Wednesday has no Work shading and Standup loses its pin
        · Ctrl+K "edit block" → Settings → Block-outs with Work selected
        · End "5:30pm", Ctrl+Enter → saved; the sidebar shows M-F 08:30-17:30
        · Exceptions: 19/10/2026 to 23/10/2026, note "Leave", Enter → five dated rows
        · click Work, Alt+↓ → it moves below Sleep (sidebar order too); drag a row by its
          handle to reorder with the mouse
        · New block-out → form; Create with no label → "Give the block-out a label."
        · click a sidebar block-out → its settings open
Notes:
    - Verified here with real keys and clicks, by screenshot: Week shading and pins, Month
      bars, b l lifting and restoring, the settings list and editor, the 17:30 save, a 5-day
      exception range and Alt+↓ reordering. Fixed while testing: the End field spilled out of
      its column, and the b l hint wrapped mid-chord.
    - Not built: the design's "lift from the day's context menu". b l and the settings page
      cover it; right-click menus could come with a later polish pass.
    - The Month bar counts windows that start that day and overlap planning hours, so Sleep
      doesn't mark every day. The Week grid still shades Sleep.
    - Block-out colour and icon (in the core model) have no controls yet; everything uses the
      theme's block-out hatch, as in the design.
    - Planning hours for the day meter are still the frontend constant (core setting since #20;
      UI in #38).

### #22 — Inbox auto-slot and conflict warnings            [status: committed 8777f83]
Phase: 5
Message:
    feat: inbox auto-slot and conflict warnings

    Core: auto_slot(ids? | Inbox, from?, until?) places flexible tasks
    with the slot engine. It orders them by due date, then priority, then
    longest first, and puts each in the earliest free slot before its due
    date, within planning hours and outside block-outs and busy time.
    A day-only task stays on its day. Each placed task also blocks the
    next. The whole run is one change-log batch. Nothing is dropped: each
    task that can't be placed comes back with a readable reason and the
    longest free gap, e.g. "No free 3h slot before its due date (Wed 14
    Oct 18:00). The longest free gap is 1h, Wed 14 Oct 17:00–18:00. It
    stays in the Inbox." Events, fixed, done, recurring and multi-day
    tasks are reported as not eligible.

    Conflicts (roadmap §4.5 rule 3): list_agenda and get_day_summary now
    return, per item, the open timed items it overlaps, and the
    block-out it sits in when that block-out warns about fixed items
    (permits_fixed off, e.g. Sleep). Overlaps are allowed, only flagged.

    UI: s auto-slots the Inbox selection or the focused task. The Inbox
    bulk bar gets a primary Auto-slot button, and the Diary's Inbox rail
    and the palette ("Auto-slot Inbox") do the whole Inbox. The result
    panel (design V3) lists what was placed ("Inbox → Thu 15 Oct
    17:00–17:30") and, for each task it couldn't place, the reason,
    with Move due date and Shorten to <longest gap> buttons; shortening
    re-runs that task in place. Week events with a conflict get the
    conflict border and a warning icon (tooltip says what they overlap).
    Diary cards get the border and an "Overlaps Dentist" line.
Files:
    A  src/app/AutoSlotPanel.tsx
    A  src/app/autoSlot.ts
    A  src/lib/conflicts.ts
    A  src/lib/conflicts.test.ts
    M  crates/blotter-core/src/slots.rs           (overlapping())
    M  crates/blotter-core/src/ops/slots.rs       (auto_slot, conflicts, shared search window)
    M  crates/blotter-core/src/ops/reads.rs       (agenda and day summary carry conflicts)
    M  src/actions/actions.ts
    M  src/app/AppShell.tsx
    M  src/components/ItemCard.tsx
    M  src/lib/types.ts
    M  src/views/WeekView.tsx
    M  src/views/diary/DayColumn.tsx
    M  src/views/diary/DiaryView.tsx
    M  src/views/diary/InboxRail.tsx
    M  src/views/inbox/BulkBar.tsx
    M  .claude/COMMITS.md
    M  .claude/roadmap.md
    Stage everything in one go:  git add -A
Test steps:
    - cargo test --workspace    → 92 core (+1 overlap, +4 auto-slot/conflict ops) + 6 app passed
    - cargo clippy --workspace --all-targets -- -D warnings → clean; cargo fmt --check → clean
    - pnpm check                → clean; Vitest 137 passed (+3 conflict text)
    - pnpm tauri dev (fresh demo DB):
        · g w → Dentist and Parent–teacher interview (Thu) have the orange conflict border
          and a ⚠; hover → "· conflict: Overlaps …"
        · g d, ] → Thursday's two cards say "⚠ Overlaps Parent–teacher interview" /
          "⚠ Overlaps Dentist"
        · Ctrl+K "auto-slot inbox" → side panel "7 placed …" (or 6 on a fresh seed), each
          "Inbox → Thu 15 Oct …"; none inside Work; the first one after Thursday's Morning
          run; Inbox shows Inbox zero; Esc or Done closes
        · i, add "Sort receipts ~3h due tomorrow 18:00", select it (Space), s → it's listed
          under Couldn't place with the reason and "Shorten to …"; click Shorten → it moves
          to Placed
Notes:
    - Verified here: the core rules by test (ordering, due-date bound, day-only tasks, one
      batch, unplaceable reasons, overlap and Sleep conflicts), and live by screenshot: Week
      and Diary conflict styling, and auto-slotting the whole Inbox from the palette.
      Also live: a 3h task due Thu 18:00 got the Couldn't place card ("… The longest free gap
      is 15m, Thu 15 Oct 08:15–08:30 …"), and Shorten to 15m re-ran it into Placed (Thu 07:00).
    - When several gaps tie for longest, the reason names the later one; placement still takes
      the earliest fit.
    - No "Undo batch" button in the panel yet: undo arrives in #27, and the run is already
      one batch, so it will undo in one step. "Ask AI" waits for the chat (#31).
    - Auto-slot searches 14 days ahead by default; planning hours are still the 07:00–22:00
      default until General settings (#38).
    - Month chips don't show conflicts (the design doesn't either).
    - My test script's first attempt at the palette missed and typed "auto-slot inbox" as
      shortcuts, which made a stray "box" task in the demo DB. That's a harness timing
      issue, not an app bug: Ctrl+K works when sent on its own.

### #23 — End-of-day rollover            [status: committed 25f3f7a]
Phase: 5
Message:
    feat: end-of-day rollover

    Core: a rollover op (roadmap §4.5) moves open, flexible tasks left on
    earlier days, timed or day-only. In inbox mode (the default) they go
    back to the Inbox. In reslot mode they get the next free slots
    through the same placement as auto_slot; whatever doesn't fit goes
    to the Inbox with the reason. Fixed items, events and recurring
    tasks never move, and tasks that are only past their deadline stay
    where they are. It's one change-log batch, and a second run does
    nothing.

    Planner::rollover_if_due runs it as the system actor once per local
    day, after the rollover time (default 04:00, so tasks worked on past
    midnight aren't swept away), and records the date. New settings:
    rollover_mode, rollover_time, last_rollover_date. auto_slot's
    placement loop is now a shared place() used by both.

    App: a background thread checks at startup and every minute, so the
    rollover happens on the first launch of a day or while Blotter sits
    in the tray. When something moved, it refreshes every window and
    holds a notice until the UI takes it.

    UI: a banner says what moved ("1 unfinished task from earlier days
    went back to the Inbox: Renew car registration") with Open Inbox and
    dismiss. The palette's "Roll over unfinished tasks now" runs it on
    demand.
Files:
    A  crates/blotter-core/src/ops/rollover.rs
    A  src-tauri/src/rollover.rs
    A  src/app/RolloverBanner.tsx
    A  src/app/rollover.ts
    M  crates/blotter-core/src/ops/mod.rs
    M  crates/blotter-core/src/ops/slots.rs        (place() shared with rollover)
    M  crates/blotter-core/src/planner.rs          (rollover_if_due)
    M  crates/blotter-core/src/settings.rs
    M  src-tauri/src/lib.rs
    M  src/actions/actions.ts
    M  src/app/AppShell.tsx
    M  .claude/COMMITS.md
    M  .claude/roadmap.md
    Stage everything in one go:  git add -A
Test steps:
    - cargo test --workspace    → 96 core (+3 rollover op, +1 once-a-day gate) + 6 app passed
    - cargo clippy --workspace --all-targets -- -D warnings → clean; cargo fmt --check → clean
    - pnpm check                → clean; Vitest 137 passed
    - Reseed the demo DB and pnpm tauri dev (after 04:00):
        · on launch a banner at the bottom: "1 unfinished task from earlier days went back to
          the Inbox: Renew car registration." The Inbox shows it; Open Inbox goes there; × dismisses
        · Ctrl+K "roll over" → "Nothing to roll over: no unfinished tasks on earlier days."
        · quit and relaunch the same day → no banner (it already ran today)
    - Reslot mode, until General settings (#38) has a control: set the setting by hand in
      DB Browser (settings: key rollover_mode, value "reslot") and relaunch the next day, or
      run the op from the palette after changing it.
Notes:
    - Verified here, live: the startup rollover and its banner on a fresh demo DB (screenshot),
      and the palette's manual run. Reslot mode, the 04:00 gate and once-a-day are covered by
      the core tests.
    - Decisions: inbox is the default mode (it never puts things somewhere you didn't choose);
      04:00 is the default rollover time. Both get controls with General settings in #38.
    - Behaviour change from #12: since unfinished flexible tasks now leave past days, the
      Diary's Overdue section mostly shows fixed items and tasks past their deadline.
    - The banner lives in memory: dismissing it is final; a rollover while the app is hidden
      shows when the window next opens.

### #24 — Recurring items with occurrence exceptions (core)            [status: committed 6ecb88a]
Phase: 6
Message:
    feat(core): recurring items with occurrence exceptions

    Recurring series are expanded on read. A series is one stored item
    with an RFC 5545 RRULE (rrule 0.14); its own start is the first
    occurrence. list_agenda, get_day_summary, the slot engine's busy time
    and conflicts now see every occurrence in their range. Occurrences
    keep their wall-clock time across DST: a fixed series in its own
    timezone, a flexible one in the user's. All-day series repeat by
    date.

    Each occurrence has a stable id, <series>@<original start>
    (2026-10-15T22:00:00Z, or 2026-10-15 for all-day). The item ops
    accept it:
    - update_item and delete_item take scope this (default) | following
      | all.
    - this: the occurrence is stored as an override (a child with
      parent_id and recurrence_id). A deleted occurrence is a cancelled
      override, i.e. an exception.
    - all: changes the series; a time change on one occurrence moves
      every occurrence by the same amount.
    - following: ends the series just before the occurrence (UNTIL) and
      starts a new series there, carrying a reduced COUNT, later
      overrides and reminders. Deleting following just ends the series.
    - complete/uncomplete and schedule_item on an occurrence store an
      override; the series stays open. Completing a whole series is
      refused, and occurrences can't go to the Inbox.
    - get_item on an occurrence returns its override or the expanded
      occurrence, with the series' reminders. set_reminder on an
      occurrence attaches to the series.

    RRULEs are now parsed on write (normalised, e.g. "rrule:freq=weekly"
    becomes FREQ=WEEKLY) and anything more frequent than daily is
    refused. The overdue and range queries leave series rows out, so an
    old series no longer looks overdue.
Files:
    A  crates/blotter-core/src/recurrence.rs
    A  crates/blotter-core/src/ops/occurrences.rs
    M  crates/blotter-core/Cargo.toml          (rrule 0.14.0)
    M  Cargo.lock
    M  crates/blotter-core/src/lib.rs
    M  crates/blotter-core/src/ops/items.rs
    M  crates/blotter-core/src/ops/mod.rs
    M  crates/blotter-core/src/ops/reads.rs
    M  crates/blotter-core/src/ops/reminders.rs
    M  crates/blotter-core/src/ops/slots.rs
    M  crates/blotter-core/src/store/items.rs
    M  .claude/COMMITS.md
    M  .claude/roadmap.md
    Stage everything in one go:  git add -A
Test steps:
    - cargo test --workspace    → 109 core (+5 recurrence, +8 occurrence ops) + 6 app passed
    - cargo clippy --workspace --all-targets -- -D warnings → clean; cargo fmt --check → clean
    - pnpm check                → unchanged, clean
    - pnpm tauri dev (demo DB): n, "gym session every weekday 6pm ~45m @fixed", Enter, g w →
      gym session on each weekday from its start, with the recurring icon; Friday's overlaps
      Dinner with Jess and shows the conflict mark
    - Dragging one occurrence, x on one, or Save/Delete in its dialog affect only that
      occurrence (the UI doesn't send a scope yet, so "this" applies)
Notes:
    - Split from the planned single commit: the UI (the this/following/all choice, a Repeats
      editor and the recurring icon on overrides) is #24a. Until then every UI change to an
      occurrence applies to that occurrence only, which is safe.
    - Verified live: the series above expanding in Week, with conflicts.
    - Decisions: "all" leaves existing overrides as they are (they stay exceptions). Recurring
      flexible tasks aren't checked against block-outs, and auto-slot and rollover skip them,
      as before.
    - Expansion is capped at 4000 occurrences per series per read.

### #24a — Recurring item editing            [status: committed afb6690]
Phase: 6
Message:
    feat(ui): recurring item editing

    The item dialog's recurring parts, from the design (V7):
    - A Repeats picker: Does not repeat, Every day, Every weekday, Every
      week on <day>, Every 2 weeks on <day>, Monthly on the <nth>, Every
      year, and Custom (edit the RRULE as text). The rule shows in mono
      beneath. Switching presets keeps how the series ends (COUNT or
      UNTIL). New items can repeat too.
    - For an occurrence, "This is a recurring item. Apply changes to:"
      This occurrence | This and following | All occurrences. Save and
      Delete use the choice; Delete reads "Delete occurrence", "Delete
      this and following" or "Delete series". Changing the repeat rule
      moves the choice off "This occurrence", since a rule belongs to
      the series.
    - Opening the series itself (e.g. from search) edits the whole
      series, and an override (an occurrence changed on its own) says
      so instead of offering a rule.

    Overrides keep the recurring icon on Diary cards and Week events.
    Del on a focused occurrence asks "Delete this occurrence of …?" and
    leaves the rest of the series.
Files:
    A  src/lib/repeat.ts
    A  src/lib/repeat.test.ts
    M  src/lib/itemDraft.ts
    M  src/lib/itemDraft.test.ts
    M  src/app/ItemDialog.tsx
    M  src/actions/itemActions.ts
    M  src/components/ItemCard.tsx
    M  src/views/WeekView.tsx
    M  .claude/COMMITS.md
    M  .claude/roadmap.md
    Stage everything in one go:  git add -A
Test steps:
    - pnpm check                → clean; Vitest 143 passed (+4 repeat, +2 recurring drafts)
    - Rust unchanged since #24.
    - pnpm tauri dev (demo DB):
        · n, "gym session every weekday 6pm ~45m @fixed", Enter; g w → it repeats on weekdays
        · double-click one → Recurring badge, Repeats "Every weekday" with
          RRULE:FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR, the Apply-changes-to bar, "Delete occurrence"
        · All occurrences, Time 18:30, Ctrl+Enter → every occurrence moves to 18:30
        · open Friday's, This occurrence, Time 19:30, Save → only Friday moves (still shows ↻)
        · open Thursday's, This and following, title "gym (heavy)", Save → Thursday on renamed
        · focus one, Del → "Delete this occurrence of …?" → Enter → only that one disappears
        · New item, date set, Repeats "Every week on …" → Save → it repeats
Notes:
    - Verified here, live: the dialog (screenshot) and an "All occurrences" time change moving
      Thursday's and Friday's sessions to 18:30. The rest are covered by the core tests (#24)
      and the draft tests.
    - Drags and Shift+arrow moves change only the occurrence you move (no prompt), as in
      #24. A prompt on drag could come later if it's missed.
    - The picker offers common rules only; anything else is Custom (RRULE text).

### #25 — Reminder engine and OS notifications            [status: committed 7ba84fe]
Phase: 6
Message:
    feat(app): reminder engine and OS notifications

    Core: a new alerts module answers "which reminders fall due between A
    and B" (roadmap §4.7). It covers:
    - offset reminders (before the start) and absolute ones;
    - recurring series, where an offset reminder fires for every
      occurrence (an override's own time wins; cancelled and done
      occurrences stay quiet);
    - all-day items, which count from 09:00 local;
    - snoozes, which replace the next firing.
    fired_at records the last firing's due time, so a restart never
    fires one twice and a rescheduled item fires again at its new time.

    App: a reminder thread sleeps until the next reminder is due (at
    most a minute), wakes early on every planner change, and shows an OS
    notification through tauri-plugin-notification 2.4: the title is the
    item, the body like "15:00–15:45 · in 10 min". The process stays in
    the tray, so reminders fire with the window closed. Each firing is
    also sent to the UI as a reminder:fired event (the in-app toast with
    snooze comes in #26). Reminders missed while Blotter wasn't running
    are left for #26's grouped catch-up rather than all firing at once.

    Default reminder: events get one 10 minutes before
    (default_event_reminder_min; 0 turns it off). create_item takes an
    explicit reminders list (minutes before), and [] means none. The item
    dialog now sends its list with the create (no follow-up batch, so no
    duplicate default), and a new item switched to Event starts with
    "10m before".
Files:
    A  crates/blotter-core/src/alerts.rs
    A  src-tauri/src/reminders.rs
    M  crates/blotter-core/src/lib.rs
    M  crates/blotter-core/src/settings.rs
    M  crates/blotter-core/src/store/reminders.rs   (list_all, set_fired)
    M  crates/blotter-core/src/ops/items.rs         (create_item reminders + default)
    M  crates/blotter-core/src/ops/reads.rs         (tests opt out of the default)
    M  crates/blotter-core/src/ops/reminders.rs     (tests opt out of the default)
    M  src-tauri/Cargo.toml                         (tauri-plugin-notification 2.4.0)
    M  src-tauri/src/lib.rs
    M  Cargo.lock
    M  src/lib/itemDraft.ts
    M  src/lib/itemDraft.test.ts
    M  src/app/ItemDialog.tsx
    M  .claude/COMMITS.md
    M  .claude/roadmap.md
    Stage everything in one go:  git add -A
Test steps:
    - cargo test --workspace    → 112 core (+3 alerts) + 7 app (+1 notification text) passed
    - cargo clippy --workspace --all-targets -- -D warnings → clean; cargo fmt --check → clean
    - pnpm check                → clean; Vitest 143 passed
    - pnpm tauri dev: n, "team meeting <a time 12 minutes from now>", Enter → about two minutes
      later a Windows notification: "team meeting · HH:MM–HH:MM · in 10 min"
    - Close the window to the tray first and repeat → it still appears
    - Quit and relaunch after it fired → it doesn't fire again
Notes:
    - Verified here: the engine logic by test (offsets, absolute, every occurrence of a
      series with a cancelled and a moved one, all-day from 09:00, done items quiet, no
      re-firing), the notification text by test, and live that a quick-added meeting got its
      default 10-minute reminder.
    - NOT verified live: I didn't see the Windows toast appear at 23:03 when the test
      reminder was due, and nothing was logged as an error. The likely causes are Focus
      Assist / Do Not Disturb, or Windows treating the unpackaged dev build as an unknown
      app. Please check this step yourself first; if no toast shows, check Settings →
      System → Notifications, and tell me what you see so I can add a fix entry.
    - Notification click and action buttons aren't available on desktop through the plugin;
      the in-app toast with snooze (#26) is the fallback the roadmap planned.
    - Missed reminders older than 2 minutes are deliberately left for #26's catch-up.

### #25a — Blotter logo and wordmark            [status: committed 980e5cf]
Phase: 6 (out-of-sequence brand work, requested before #26)
Message:
    feat(ui): Blotter logo and wordmark

    The placeholder icon is replaced by the Blotter logo, a calendar page
    whose right edge forms a "B", redrawn as a vector from
    .claude/Blotter-logo-reference.png, and the name is set in Playfair
    Display.

    - public/blotter.svg is the brand icon in its fixed colours (navy
      tile, indigo, green "done" cell). It is the favicon and the source
      for the app, taskbar and tray icons, regenerated with
      `pnpm tauri icon public/blotter.svg` (only the existing desktop
      sizes are kept).
    - BlotterMark is the same geometry drawn inline without the tile and
      coloured from the theme: accent for the header and bowls,
      --logo-page for the page and rings, an accent tint for the cells,
      tag-health for the done cell. The ring holes are cut out, so the
      background shows through.
    - Presets set --logo-page (the text colour on Dark and Gruvbox, a
      light page on Light and Paper) and --logo-outline, a thin edge that
      keeps a light page visible on a light sidebar. --logo-page is in a
      new "Logo" token group for the theme editor (#37).
    - Wordmark: Playfair Display (OFL, @fontsource-variable 5.3.0) at
      weight 800, from new --font-brand and --brand-weight tokens, so the
      theme engine can change it later. A solid offset shadow in the
      accent (wordmark-shadow; --brand-shadow overrides the colour) gives
      it a stamped, two-colour look.
    - The sidebar header (mark 24px, name 19px) and Settings → About
      (mark 48px, name 30px, version beside it) use both in place of the
      "B" tile, with the name on the mark's baseline.
Files:
    A  .claude/Blotter-logo-reference.png
    A  src/components/BlotterMark.tsx
    M  public/blotter.svg
    M  src-tauri/icons/32x32.png
    M  src-tauri/icons/64x64.png
    M  src-tauri/icons/128x128.png
    M  src-tauri/icons/128x128@2x.png
    M  src-tauri/icons/icon.png
    M  src-tauri/icons/icon.ico
    M  src-tauri/icons/icon.icns
    M  src/app/Sidebar.tsx
    M  src/views/SettingsView.tsx
    M  src/theme/presets.ts
    M  src/styles.css                (Playfair hunks only; the scrollbar hunk is #25c)
    M  package.json                  (the playfair-display hunk only)
    M  pnpm-lock.yaml                (the three playfair-display hunks only)
    #25a–#25d are all in the working tree. Stage this one with:
      git add .claude/Blotter-logo-reference.png src/components/BlotterMark.tsx public/blotter.svg src-tauri/icons src/app/Sidebar.tsx src/views/SettingsView.tsx src/theme/presets.ts
      git add -p src/styles.css package.json pnpm-lock.yaml
        → "y" for every hunk that mentions playfair, --font-brand, --brand-weight,
          wordmark-shadow or
          font-brand-weight; "n" for everything else (scrollbars, react-day-picker and
          its date-fns/tz, hijri, jalali deps, dev:demo)
      (VS Code: open each file's diff and "Stage Selected Ranges" on those lines.)
Test steps:
    - pnpm check                → clean; Vitest 143 passed
    - pnpm tauri dev:
        · the window title bar and taskbar show the new icon
        · sidebar header: the calendar-B mark beside "Blotter" in heavy Playfair with
          an accent-coloured offset shadow
        · the word sits level with the bottom of the mark, not centred on it
        · Settings → About: the same lockup at 48px / 30px, version on the same line
        · Theme → Light and Paper: white / cream page with a thin edge;
          Gruvbox: yellow header, cream page; Dark: light page
    - Tray icon shows the new logo (it uses the default window icon).
Notes:
    - Verified here by screenshot on all four presets (mark) and in the running app on Dark
      (Playfair wordmark), and by rendering public/blotter.svg next to the reference.
    - Boheld was considered; it's free for personal use only, so it can't ship in Blotter.
    - Playfair's licence reserves the name "Playfair Display": fine as-is; a modified
      font file would need a different name.
    - BlotterMark mirrors public/blotter.svg's paths (minus the tile); change both together.
    - The design prototypes in .claude/Prototype Screens still show the "B" tile; they're
      reference only and weren't changed.

### #25b — Themed select, date and time pickers            [status: committed 980e5cf]
Phase: 6 (housekeeping, requested before #26)
Message:
    feat(ui): themed select, date and time pickers

    Native <select> and <input type="date"> are replaced by components
    in the app's own style, built on Radix like the tooltips:

    - Select: Radix Select with a field-style trigger and a raised list
      (Repeats and "Remind me…" in the item dialog).
    - DatePicker: a field-style trigger ("Thu 15 Oct"; the year only
      when it isn't this one) opening a Radix popover with a
      react-day-picker 9.14 calendar, styled only with our tokens. Weeks
      start on Monday like the views; Today and (where the date is
      optional) Clear buttons; arrow keys move the day, Esc closes just
      the calendar. Used in the item dialog (date, due date), the Inbox
      bulk bar (schedule, due) and block-out exceptions (from, until).
    - TimeField: typing still works ("9", "930", "2pm", tidied on blur);
      clicking opens a list of 15-minute slots centred on the current
      time, and Up/Down step by 15 minutes. Used for every time beside
      a date picker.
Files:
    A  src/components/ui/select.tsx
    A  src/components/ui/date-picker.tsx
    A  src/components/ui/time-field.tsx
    M  src/app/ItemDialog.tsx
    M  src/views/inbox/BulkBar.tsx
    M  src/views/settings/BlockoutsPage.tsx
    M  package.json                  (react-day-picker 9.14.0 — this hunk only)
    M  pnpm-lock.yaml                (everything #25a left)
      git add src/components/ui/select.tsx src/components/ui/date-picker.tsx src/components/ui/time-field.tsx src/app/ItemDialog.tsx src/views/inbox/BulkBar.tsx src/views/settings/BlockoutsPage.tsx pnpm-lock.yaml
      git add -p package.json     → "y" for "react-day-picker", "n" for "dev:demo" (that is #25d)
      (or in VS Code: select the react-day-picker line → right-click → Stage Selected Ranges)
Test steps:
    - pnpm install; pnpm check  → clean; Vitest 143 passed
    - pnpm tauri dev, open an item (Enter on a card):
        · When: the date shows "Thu 15 Oct"; click → calendar with the day selected,
          Mon-first; pick another day → field updates, calendar closes; Esc closes only
          the calendar
        · Time: click → slot list centred on the time; ↓/↑ step 15 min; type "2pm",
          Tab → 14:00
        · Repeats → themed list with a check on the current option
        · Event + "Reminder" → the "Remind me…" list opens straight away
    - Inbox: select tasks → Schedule / Due use the same pickers; Due has Clear
    - Settings → Block-outs → a block-out → Exceptions: From / "Same day" pickers
Notes:
    - Verified here by screenshot (item dialog date, time and Repeats on Paper and Light,
      block-out exception fields) and by keyboard (Esc, ↓).
    - react-day-picker 10 is out; 9.14 is the line shadcn's calendar uses. Moving to 10
      later is small.
    - Block-out start/end times are plain text fields (no date beside them) and were
      left as they are; say if you want the slot list there too.

### #25c — Theme-coloured scrollbars            [status: committed 980e5cf]
Phase: 6 (housekeeping, requested before #26)
Message:
    feat(ui): theme-coloured scrollbars

    Scrollbars are now a slim rounded thumb in the theme's strong border
    colour (muted text on hover) on a clear track, with no arrow
    buttons. ::-webkit-scrollbar covers WebView2 and WebKitGTK.
Files:
    M  src/styles.css                (the scrollbar hunk #25a left)
      git add src/styles.css
Test steps:
    - pnpm check                → clean
    - pnpm tauri dev: Week view and the time slot list show the slim thumb on every
      preset; hovering darkens it
Notes:
    - Verified here by screenshot on Light (Week view, slot list).
    - Commit #25a → #25d in order; #25d carries the ledger.
    - Don't add `scrollbar-color` to the CSS: Chromium then ignores the ::-webkit-scrollbar
      rules.

### #25d — pnpm dev:demo            [status: committed 980e5cf]
Phase: 6 (housekeeping, requested before #26)
Message:
    chore: add pnpm dev:demo for testing against demo data

    `pnpm dev:demo` runs `pnpm tauri dev` with BLOTTER_DATA_DIR pointing
    at a demo database in <OS temp>/blotter-demo (BLOTTER_DEMO_DIR
    overrides it), so the real database is never touched. The demo is
    seeded with the seed example when it doesn't exist yet or was seeded
    in an earlier week (the seed is relative to the current week), and
    is otherwise kept between runs; --fresh reseeds it.
Files:
    A  scripts/dev-demo.mjs
    M  package.json                  (the "dev:demo" script — the hunk #25b left unstaged)
    M  .claude/COMMITS.md
      git add scripts/dev-demo.mjs package.json .claude/COMMITS.md
Test steps:
    - pnpm check                → clean
    - Close any running Blotter, then pnpm dev:demo
        → "seeding a new demo database (no demo database yet)", then the app opens with the
          demo week, 7 Inbox tasks and the Work / Sleep block-outs
    - Quit, pnpm dev:demo again → "reusing this week's demo database"
    - pnpm dev:demo --fresh     → "seeding a new demo database (--fresh)"
    - Plain pnpm tauri dev still opens your own (empty) database
Notes:
    - Verified here: all three paths (first seed, reuse, --fresh) on Windows, and the app
      showing the seeded data.
    - Only one Blotter can run at a time (single-instance): a running copy, including one
      in the tray, receives the launch and keeps its own database.
    - The WebView's localStorage (last view, theme cache) is shared between the demo and
      your real data, so a theme picked in the demo flashes briefly on your next normal
      launch before your own setting loads.

### #26 — Snooze, missed-reminder catch-up and autostart            [status: committed 67206ac]
Phase: 6
Message:
    feat: snooze, missed-reminder catch-up and autostart setting

    Core:
    - snooze_reminder: fires a reminder again `minutes` from now or at
      `until`; the snooze replaces its next firing and is change-logged.
      A snoozed firing on a series now reports the occurrence it was
      snoozed from (not the series' first), and stays quiet if that
      occurrence was completed or cancelled meanwhile.
    - get_settings / update_settings: the settings as the user sees them,
      and typed, validated changes to the notification ones (default event
      reminder, rollover time, notifications on/off, snooze options,
      catch-up on/off and lookback). Each changed key is change-logged as
      a `setting`. #38 extends these for the General page.

    App:
    - The engine now checks the time since its last check (saved), not a
      fixed two minutes. Reminders more than two minutes late were missed:
      those within the lookback (default 24h) arrive as one grouped
      notification, "3 reminders missed while Blotter was closed" (or
      "while your computer was asleep"); older ones are skipped quietly.
    - With notifications off, reminders still fire inside Blotter.
    - Firings before the window loads wait until the UI collects them
      (take_reminder_notices), so a catch-up at startup isn't lost.
    - Launch at login (tauri-plugin-autostart 2.7): the login launch
      passes --hidden, so Blotter starts in the tray. Plus a "Send test"
      notification command.

    UI:
    - Reminder toasts (design: Reminder Toast): title and "in 10 min ·
      11:30–12:15"; Snooze with the configured lengths on keys 1–n and
      Custom… ("25m", "2h" or "14:00"); Open (↵), Mark done (x), Esc to
      dismiss. A toast takes focus only when nothing else has it.
    - A missed toast lists what was missed with "Review in Diary" and
      "Mark all seen".
    - Settings → Notifications: notifications, default event reminder,
      snooze options (chips), catch-up and lookback, rollover time, launch
      at login, and Send test. Desktop notifications can't carry buttons
      here, so the page explains that snoozing happens on the toast.
Files:
    A  crates/blotter-core/src/ops/settings.rs
    A  src-tauri/src/autostart.rs
    A  src/app/ReminderToasts.tsx
    A  src/app/reminders.ts
    A  src/app/reminders.test.ts
    A  src/views/settings/NotificationsPage.tsx
    M  crates/blotter-core/src/alerts.rs
    M  crates/blotter-core/src/ops/mod.rs
    M  crates/blotter-core/src/ops/reminders.rs
    M  crates/blotter-core/src/planner.rs        (planner_at test helper)
    M  crates/blotter-core/src/settings.rs
    M  src-tauri/Cargo.toml                      (tauri-plugin-autostart 2.7.0)
    M  Cargo.lock
    M  src-tauri/src/commands.rs                 (CommandError::internal)
    M  src-tauri/src/lib.rs
    M  src-tauri/src/reminders.rs
    M  src/app/AppShell.tsx
    M  src/app/nav.ts
    M  src/views/SettingsView.tsx
    M  .claude/COMMITS.md
    M  .claude/roadmap.md
    Stage everything in one go:  git add -A
Test steps:
    - cargo test --workspace    → 116 core (+4: snooze, settings) + 9 app (+2: catch-up, missed text)
    - cargo clippy --workspace --all-targets -- -D warnings → clean; cargo fmt --check → clean
    - pnpm check                → clean; Vitest 153 passed (+10)
    - pnpm dev:demo (close any other Blotter first):
        · Settings → Notifications → Send test → a Windows notification appears
        · Quick-add "test ping <a time 11 minutes from now>" → a minute later a notification
          and a toast bottom-right: "in 10 min · …"; press 1 → "Snoozed 5m · fires again
          HH:MM", it closes; 5 minutes later it's back, "· snoozed"
        · On a toast: x marks it done; ↵ opens the item; Esc dismisses
        · Custom… → "2m" → Snooze → back in 2 minutes
        · Quit from the tray; quick-add another event 12 minutes ahead first; wait until
          its reminder time has passed by 3+ minutes; relaunch → one "1 reminder missed
          while Blotter was closed" notification and the missed toast
        · Notifications off → reminders still show as toasts, no Windows notification
        · Launch at login on → Task Manager → Startup apps lists Blotter; turn it off again
          (in dev it points at target\debug\blotter.exe)
Notes:
    - Verified live here (pnpm dev:demo, Windows): Settings → Notifications renders and
      saves; Send test reached Windows; an event's reminder fired as a notification and a
      toast; 1 snoozed 5m and it came back on time marked "snoozed"; Custom… 2m snoozed
      and returned; 1h snoozed; x marked the item done; quit before a reminder, relaunch
      after → one "1 reminder missed while Blotter was closed" notification and the
      missed toast (since the quit time); Mark all seen cleared it; notifications off →
      toast only, nothing sent to Windows; Launch at login on wrote the Run key
      (blotter.exe --hidden) and off removed it. Checked via the database and Windows'
      notification history.
    - Not checked live: "while your computer was asleep" (same code path, unit-tested).
    - The daily-briefing time on the design's Notifications page arrives with the briefing
      in Phase 8.
    - "Launch at login" sits on Notifications until the General page (#38), where the
      design puts it.
    - Flatpak: autostart and notifications need the Background and Notification portals;
      checked in Phase 13 as planned.
    - New operations beyond the roadmap's minimum set: snooze_reminder, get_settings,
      update_settings (all in the registry, so the AI and MCP can use them).

### #27 — Undo/redo from the change log            [status: committed 575b2c9]
Phase: 7
Message:
    feat: undo/redo from change log

    Core:
    - undo / redo / get_undo_state operations. Undo reverses the newest
      batch that hasn't been undone (any actor) by writing each entity
      back to its `before` state; redo re-applies what the last undo
      reversed, unless something has been edited since. `undo` also takes
      a batch_id, for the AI chat's "Undo batch" later.
    - Undo and redo write their own change rows (op undo/redo) with a new
      `reverts` column naming the batch (migration 0002), so the log stays
      append-only and History can show them.
    - Each entity must still be as the batch left it; otherwise the undo
      is refused ("“Gym” has changed since") instead of overwriting the
      later edit. Rows already removed by a cascade are skipped, foreign
      keys are checked at commit (so a restored reminder can precede its
      item), and a reminder's fired state is kept so it doesn't re-fire.
    - Batches get a short summary: "Create item “Gym”", "Auto-slot
      (8 items)", "Delete block-out “Work”".
    - The change log now sorts by insertion order: ULIDs made in the same
      millisecond don't sort by time, which could misorder a batch.

    UI:
    - Ctrl+Z undo and Ctrl+Shift+Z (or Ctrl+Y) redo, also in the palette.
      Text fields keep those keys for their own typing undo (new
      yieldsToText action flag).
    - A bottom-centre notice says what happened ("Undone: Auto-slot
      (8 items)") with the opposite action; it sits above the rollover
      banner when that's showing. Undo/redo closes the auto-slot result
      panel, whose placements may no longer stand.
Files:
    A  crates/blotter-core/migrations/0002_change_reverts.sql
    A  crates/blotter-core/src/ops/undo.rs
    A  src/app/Notice.tsx
    A  src/app/noticeStore.ts
    A  src/app/undo.ts
    M  crates/blotter-core/src/db.rs
    M  crates/blotter-core/src/model/change.rs
    M  crates/blotter-core/src/ops/mod.rs
    M  crates/blotter-core/src/store/changes.rs
    M  crates/blotter-core/src/store/settings.rs   (delete)
    M  src/actions/actions.ts
    M  src/app/AppShell.tsx
    M  src/keymap/useKeymap.ts
    M  .claude/COMMITS.md
    M  .claude/roadmap.md
Test steps:
    - cargo test --workspace    → 123 core (+7: undo/redo sequence, redo cleared by an edit,
      delete with reminder and override, conflict refusal, settings and block-outs, log rows,
      labels) + 9 app
    - cargo clippy --workspace --all-targets -- -D warnings → clean; cargo fmt --check → clean
    - pnpm check                → clean; Vitest 153 passed
    - pnpm dev:demo: Inbox → Auto-slot → Ctrl+Z → "Undone: Auto-slot (8 items)", the tasks
      are back in the Inbox; Ctrl+Shift+Z → placed again; type in a text field and Ctrl+Z
      undoes the typing only; About shows schema v2
Notes:
    - Verified live (pnpm dev:demo --fresh): the migration ran (About: schema v2); undoing an
      8-item auto-slot from the palette wrote 8 undo rows, returned the tasks to the Inbox and
      showed the notice with Redo; Redo from the palette re-placed them ("Redone: …").
      Ctrl+Z itself couldn't be pressed by my test harness (posted keys carry no Ctrl), so the
      shortcut path is covered by the keymap code, not a live press — please try it.
    - Ctrl+Z undoes system and AI batches too (e.g. the rollover), newest first, as the log
      is one timeline.
    - Undo of theme changes isn't covered: the theme is a UI setting outside the change log.

### #27a — Palette ranks exact matches first            [status: committed 53ee35a]
Phase: 7 (fix found while testing #27)
Message:
    fix(ui): rank exact palette matches first

    The palette used cmdk's default fuzzy score, so typing "redo" listed
    "Previous period", "Next period" and "Diary ↔ Calendar" (r…e…d…o in
    order) above Redo. The palette now ranks with its own score: exact
    label, then prefix, then a word starting with the search, then all
    words of a multi-word search ("theme dark"), then substring, then
    keywords. Loose letter-by-letter matches only count from a word
    start and rank last.
Files:
    A  src/lib/paletteScore.ts
    A  src/lib/paletteScore.test.ts
    M  src/app/CommandPalette.tsx
    M  .claude/COMMITS.md
Test steps:
    - pnpm check                → clean; Vitest 158 passed (+5)
    - Palette: "redo" → Redo first; "undo" → Undo first; "theme dark" → Theme: Dark
Notes:
    - Verified live: "redo" now lists Redo, then "Roll over unfinished tasks now" (a low,
      word-start match); the View rows no longer appear.

### #27b — Undo skips automatic system changes            [status: committed b5b6341]
Phase: 7 (Luke's call on #27, D24)
Message:
    fix(core): undo skips automatic system changes

    Ctrl+Z now undoes the user's and the AI's changes only; automatic
    system batches such as the rollover are skipped, and one made after
    an undo no longer clears redo. If a system change touched the same
    entity, undo's "changed since" check still refuses rather than
    overwriting it.

    Also confirms decisions D15–D19 and D23, and records D24–D28 (undo
    scope, snapshots, JSON import, AI keys and models, daily briefing).
Files:
    M  crates/blotter-core/src/store/changes.rs
    M  crates/blotter-core/src/ops/undo.rs
    M  .claude/roadmap.md
    M  .claude/COMMITS.md
Test steps:
    - cargo test --workspace    → 124 core (+1: system changes skipped) + 9 app
    - cargo clippy --workspace --all-targets -- -D warnings → clean
Notes:
    - Unit-tested only: the change is two SQL conditions.

### #27c — Unique keys on the block-out editor panels            [status: committed e16efbd]
Phase: 7 (parked fix, found 2026-10-15)
Message:
    fix(ui): unique keys on the block-out editor panels

    Settings → Block-outs keyed both the editor and the Exceptions panel
    beside it by the selected block-out's id. React needs sibling keys to
    be unique, so it logged "two children with the same key" whenever
    the page showed a block-out (and could mix up their state). The keys
    are now "editor-<id>" and "exceptions-<id>"; each panel still resets
    when another block-out is picked.
Files:
    M  src/views/settings/BlockoutsPage.tsx
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean
    - pnpm dev:demo → Settings → Block-outs: no "same key" warning in the dev log
Notes:
    - Verified live: the page with editor and exceptions logged no warnings (it logged one
      per visit before).

### #27d — Error boundaries instead of a blank window            [status: committed 4c3e5b3]
Phase: 7 (parked follow-up from #25b)
Message:
    feat(ui): error boundaries instead of a blank window

    A rendering error used to blank the whole window. Now the view
    area has its own boundary, keyed by view: a crash shows "This view
    hit an error and couldn't be shown" with the message and Retry, while
    the sidebar and top bar keep working, and switching views starts the
    broken one fresh. An outer boundary around the whole app offers
    Reload window as the last resort. Errors are logged to the console
    with their component stack.
Files:
    A  src/app/ErrorBoundary.tsx
    M  src/app/AppShell.tsx
    M  src/main.tsx
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 158 passed
Notes:
    - Verified live by making the Year view throw (temporarily, reverted): the panel showed,
      navigation kept working, and returning to Year after the fix rendered it normally.

### #28 — Snapshots, restore UI and JSON export/import            [status: committed 92c7afe]
Phase: 7
Message:
    feat: snapshots, restore UI and JSON export/import

    Core:
    - Snapshots are full VACUUM INTO copies in a `snapshots` folder beside
      the database (relocatable; moving it moves the files). Taken daily
      on the first check of the day, before every restore and import,
      and by hand with an optional note. The newest 20 automatic ones are
      kept (configurable 5–200 via update_settings `snapshot_keep`);
      manual ones are always kept. Missing files drop off the list.
    - Restore takes a "before restore" snapshot, copies the snapshot in
      with SQLite's backup API (the open connection stays valid) and
      migrates it if it's from an older version. The current snapshot
      list and this install's own settings (theme, snapshot folder,
      reminder/rollover bookkeeping) carry across, and the reminder
      check restarts from now so old reminders don't flood in.
    - JSON export (format "blotter", version 1): items, reminders,
      block-outs, exceptions and user settings. Import checks the file,
      then replaces everything after a "before import" snapshot; the
      change log is cleared with the data it described.
    - Migration 0003: a note on snapshots.

    App/UI:
    - Settings → Backups (design S8): Export JSON, Import JSON (with a
      confirm showing what the file holds), Snapshot now (with a note),
      retention, location (Change… / Use default), and the snapshot list
      with Restore… (confirmed). The window reloads after a restore or
      import, back on the same settings page.
    - tauri-plugin-dialog 2.8.1 for the native file and folder pickers.
Files:
    A  crates/blotter-core/migrations/0003_snapshot_note.sql
    A  crates/blotter-core/src/export.rs
    A  crates/blotter-core/src/snapshots.rs
    A  src-tauri/src/backups.rs
    A  src/lib/backupFormat.ts
    A  src/lib/backupFormat.test.ts
    A  src/views/settings/BackupsPage.tsx
    M  crates/blotter-core/Cargo.toml        (rusqlite "backup" feature)
    M  crates/blotter-core/src/db.rs          (migration 0003, restore_from)
    M  crates/blotter-core/src/lib.rs
    M  crates/blotter-core/src/ops/settings.rs (snapshot_keep)
    M  crates/blotter-core/src/planner.rs     (db_mut)
    M  crates/blotter-core/src/settings.rs    (snapshot keys, device keys)
    M  crates/blotter-core/src/store/items.rs (list_all)
    M  src-tauri/Cargo.toml                   (tauri-plugin-dialog 2.8.1)
    M  Cargo.lock
    M  src-tauri/src/lib.rs
    M  src-tauri/src/rollover.rs              (daily snapshot check)
    M  src/app/nav.ts                         (settings page persists)
    M  src/views/SettingsView.tsx
    M  .claude/COMMITS.md
    M  .claude/roadmap.md
Test steps:
    - cargo test --workspace    → 129 core (+5: restore round trip keeps the list and is
      itself undoable, pruning keeps manual, daily once per local day + relocation,
      export→import identical, rejects other files) + 9 app
    - cargo clippy / fmt → clean; pnpm check → clean; Vitest 160 passed (+2)
    - pnpm dev:demo: Settings → Backups shows a Daily snapshot; Snapshot now with a note →
      listed "Manual · note" and kept; add a task, Restore… the manual one → the task is
      gone and "Before restore" is listed; Export JSON → file saved; Import JSON that file
      → confirm lists the counts → replaced; Change… moves the snapshots
Notes:
    - Verified live: the daily snapshot at startup, Snapshot now with a note, Restore (probe
      task gone, "Before restore" listed, list carried across, no errors in the log).
    - Not driven live: the native Save/Open/Folder dialogs (my test harness can't operate
      them). Export→import is covered by the core round-trip test, the folder move by a core
      test; please try the three buttons.
    - Roadmap AC met: any edit sequence undoes and redoes (#27); a restore snapshots the
      current state first; a JSON export re-imports to an identical state (tested).

### #29 — AI provider adapters and keychain credentials (experimental)            [status: committed 6d6442d]
Phase: 8 (D29)
Message:
    feat(ai): provider adapters and keychain credentials

    Anthropic (Messages API, raw HTTP) and OpenAI-style chat completions
    (OpenAI and compatible servers such as Ollama), streamed, with tool
    calling. Tool arguments are parsed strictly and flagged, never run,
    when malformed; Anthropic thinking blocks are passed back unchanged;
    refusal fallbacks are on for the models that take them. Default
    Anthropic model: claude-sonnet-5-5.

    Keys live in the OS keychain (Windows Credential Manager / Secret
    Service), never in the database, logs or webview; the UI sees only
    the last four characters. Settings → AI providers: add Anthropic,
    OpenAI or OpenAI-compatible endpoints, save a key, load models, test
    the connection, pick the one in use.

    Experimental: unit-tested against recorded streams, not yet run
    against a real service with a real key (labelled in the UI).
Files:
    A  crates/blotter-core/src/store/providers.rs
    A  src-tauri/src/ai/{mod,sse,anthropic,openai,keychain,commands}.rs
    A  src/views/settings/AiProvidersPage.tsx
    M  crates/blotter-core/src/store/mod.rs
    M  src-tauri/Cargo.toml   (reqwest 0.13.5 json+stream, futures-util, keyring 4.2.0)
    M  Cargo.lock
    M  src-tauri/src/lib.rs
    M  src/app/nav.ts, src/views/SettingsView.tsx
    M  .claude/COMMITS.md, .claude/roadmap.md (D29, Phase 8 plan, #29a/#30 reorder)
Test steps:
    - cargo test --workspace → 131 core (+2 providers) + 20 app (+11: SSE parser, both
      adapters' recorded streams, truncated tool input, error classes, key hint)
    - clippy / fmt clean; pnpm check clean, Vitest 160
Notes:
    - Verified live: the page, the experimental badges, saving a made-up key into Windows
      Credential Manager (not in the DB, WAL or log), and Test connection reaching
      api.anthropic.com through the corporate TLS proxy, answering 401 "invalid x-api-key"
      shown as "couldn't sign in". The test key was deleted afterwards.
    - Fixed while testing: OpenAI/compatible providers couldn't be added without a model;
      the model may now be empty until chosen, and chat refuses until it is.
    - Not verified: a real reply from any provider (needs a key; D29), the OpenAI-compatible
      add flow after that fix, and Linux CI building aws-lc-rs (reqwest's default TLS).
    - Next: #29a MCP server, then #30 tool loop with the Claude Code provider.

### #29a — MCP server over the operation registry            [status: committed — hash in the next entry]
Phase: 8 (brought forward from #40, D29)
Message:
    feat(mcp): MCP server over the operation registry

    `blotter mcp` serves the operation registry over stdio (rmcp 3.5.1)
    for Claude Desktop, Claude Code and other MCP clients. It's a
    subcommand of the app's own program, handled before Tauri starts, so
    it never takes the single-instance lock, runs beside an open Blotter,
    and needs nothing extra packaged. It opens the same database as the
    app (WAL); changes are written as actor `mcp`, show in the change log
    and undo with Ctrl+Z. Blotter takes a "Before AI batch" snapshot
    before the first change of each MCP session. Validation and planner
    errors come back as tool errors, one problem per line, so the model
    can retry.

    Core: undo, redo, get_undo_state, rollover, update_settings and
    snooze_reminder are user-only. They're left out of agent tools and
    refused for AI and MCP actors. Also adds Planner::now.

    App:
    - Polls SQLite's data_version every second. A write from another
      process refreshes every window and wakes the reminder engine.
    - The app and `blotter mcp` find the database through one function:
      $BLOTTER_DATA_DIR, else the OS data dir plus the app id. This is the
      same path Tauri's app_data_dir gave, so existing data stays put.
    - Settings → AI providers has an MCP server section with copyable
      `claude mcp add` and claude_desktop_config.json setups. These pass
      BLOTTER_DATA_DIR on when the app runs on a non-default database.

    Build: MSRV 1.85 → 1.88 (rmcp needs it). Clippy's let-chain lint
    then collapses four nested ifs.
Files:
    A  crates/blotter-mcp/Cargo.toml
    A  crates/blotter-mcp/src/lib.rs
    A  src-tauri/src/external.rs
    A  src/lib/mcpSetup.ts
    A  src/lib/mcpSetup.test.ts
    A  src/views/settings/McpSection.tsx
    M  Cargo.toml                              (member, workspace dep, rust-version 1.88)
    M  Cargo.lock
    M  crates/blotter-core/src/ops/mod.rs      (USER_ONLY_OPS, agent_ops, actor check)
    M  crates/blotter-core/src/planner.rs      (now(), test)
    M  crates/blotter-core/src/ops/blockouts.rs, ops/slots.rs   (let chains)
    M  src-tauri/Cargo.toml                    (blotter-mcp, dirs 7.0.0)
    M  src-tauri/src/lib.rs                    (db_path, run_mcp, watcher, mcp_setup)
    M  src-tauri/src/cli.rs                    (mcp subcommand)
    M  src-tauri/src/commands.rs               (ChangedPayload pub(crate))
    M  src-tauri/src/capture.rs, rollover.rs   (let chains)
    M  src/views/settings/AiProvidersPage.tsx
    M  .claude/COMMITS.md, .claude/roadmap.md
Test steps:
    - cargo test --workspace → 132 core (+1 user-only ops) + 22 app (+2: mcp subcommand,
      app id matches tauri.conf.json) + 4 mcp (in-process client: tool list and hints,
      logged as mcp after one snapshot, invalid args come back, user-only/unknown refused)
    - cargo clippy --workspace --all-targets -- -D warnings → clean; cargo fmt --check → clean
    - pnpm check → clean; Vitest 164 passed (+4)
    - Raw JSON-RPC over stdio to target/debug/blotter.exe mcp on a scratch BLOTTER_DATA_DIR:
      initialize → serverInfo blotter 0.1.0, protocol 2025-06-18; tools/list → 25 tools;
      create_item / auto_slot work; bad args → isError with one line per problem;
      undo → -32602 unknown tool; one before_ai_batch snapshot in the folder
Notes:
    - Verified live (pnpm dev:demo --fresh, driven over WebView2 CDP): a `blotter mcp`
      create_item from another process moved the sidebar Inbox count 7 → 8 within the 1 s
      poll, and the item appeared in the Inbox view. The app's undo state named it
      ("Create item “Added over MCP”") and undo removed it. The settings section rendered
      with the demo folder passed through. No errors in the dev log.
    - Verified on the release build too (windows_subsystem = "windows", no console): the
      same stdio probe passes, so pipes from MCP clients work without a console window.
    - Propose mode isn't there yet: until #32 adds proposals and the Propose/Auto switch,
      MCP changes apply straight away (auto mode), with the session snapshot and undo as the
      guard rails. #32 should make MCP mutations become proposals when Propose is on (§4.8).
    - One snapshot per session rather than one per tool call: an agent making 10 calls
      would otherwise push 10 snapshots through the 20-snapshot retention.
    - Not verified: a real Claude Desktop / Claude Code session using the setup lines;
      Linux/Flatpak (`flatpak run --command=blotter io.github.lasr0.Blotter mcp`, Phase 13).
    - Next: #30 tool loop with the Claude Code provider (spawns `claude` with this server).
