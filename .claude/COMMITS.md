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

Commit types: `feat`, `fix`, `perf`, `refactor`, `test`, `docs`, `build`, `ci`, `chore`.
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
| 29a | 8 | `feat(mcp): MCP server over the operation registry` | committed `e43fd31` (brought forward from #40, D29) |
| 30 | 8 | `feat(ai): tool-calling loop and Claude Code provider` | committed `aafb799` |
| 31 | 8 | `feat(ui): chat panel and pop-out window` | committed `4edf2d6` |
| 31a | 8 | `perf(ai): keep Claude Code running between messages` | committed `bd48332` |
| 32 | 8 | `feat(ai): proposal review and auto-apply mode` | committed `0a2b64a` (Undo batch split out as #32a) |
| 32a | 8 | `feat(ui): undo batch for auto-applied AI changes` | committed `d8bc8d7` |
| 33 | 8 | `feat(ai): daily briefing` | committed `e43ac71` |
| 33a | 8 | `docs: Phase 8 wrap-up and open items` | committed `9af9c7c` |
| 34 | 9 | `feat: ICS export and file import` | committed `2551a8e` |
| 35 | 9 | `feat: ICS subscriptions with block-out option` | committed `73d43a7` |
| 35a | 9 | `feat(ui): subscribed events in the calendar views` | committed `05cceab` |
| 36 | 10 | `feat(ui): keybinding editor` | committed `aa8c036` |
| 36a | 10 | `refactor(ui): quick capture info on the Keybindings page` | committed `aa98dad` |
| 37 | 10 | `feat(ui): theme engine with live preview` | committed `daf1ff4` |
| 38 | 10 | `feat(ui): General settings page` | committed `f6b8d44` (split: first-run setup is #38a) |
| 38a | 10 | `feat(ui): first-run setup` | committed `95b63fe` |
| 39 | 11 | `feat(ui): completion history and streaks` | committed `4703da8` |
| 36b | 10 | `fix(ui): Keybindings page to its design` | committed `97bed23` |
| 37a | 10 | `fix(ui): Theme page to its design` | committed `7a1f99f` |
| 38b | 10 | `fix(ui): first-run setup to its design` | committed `b1efb94` |
| 39a | polish | `style(theme): Luke's accents for Dark, Light and Paper` | committed `1b32ffd` |
| 39b | polish | `style(ui): polish from Luke's list` | committed `0a92ee0` |
| 39c | polish | `feat(ui): Settings on one page in wide windows` | committed `5243959` |
| 39d | polish | `style(ui): centred content, sidebar key chip, keybinding row` | committed `86f2a48` |
| 39e | polish | `style(ui): one key chip per key in the sidebar` | committed `26d1f45` |
| 39f | polish | `feat(ai): smart slot` | committed `daff565` |
| 39g | polish | `feat(theme): Blotter and Studio presets, logo and title bar colours` | committed `6c97806` |
| 39h | polish | `feat(ui): calendar and tag filters, sliding panels, themed hover hints` | committed `876637a` |
| 39i | polish | `fix(ui): palette theme icons for the new presets` | committed `1d1bb76` |
| 40 | 12 | ~~`feat(mcp): MCP server over the operation registry`~~ | superseded by #29a |
| 41 | 13 | `build: Windows installers, Linux bundles and the release workflow` | committed `6d39450` |
| 42 | 13 | `build: Flatpak manifest and sandbox support` | committed `686a4c2` |
| 43 | 13 | `build: AUR PKGBUILD` | committed `cfc2e8f` |
| 44 | 13 | `docs: README, Hyprland notes, MCP setup and keybindings` | committed `d84fe70` |
| 45 | 13 | `chore(release): v1.0.0` | committed `c263ecb` (untagged; Luke tags after the smoke checks) |
| 45a | 13 | `feat(app): no icon or title in the Windows title bar` | committed `e957972` |
| 45b | 13 | `fix(app): plain Windows title bar from the first frame` | committed `d3fb700` |
| 46 | 14a | `docs: v1.1.0 plan` | committed `674340a` |
| 47 | 14a | `fix(ui): held arrows keep moving focus and items` | committed `4502a85` |
| 48 | 14a | `fix(ui): resize handle wins over move` | committed `24eddd5` |
| 49 | 14a | `fix(ui): Shift+arrows in the Year view` | committed `ab21b49` |
| 50 | 14a | `feat(ui): brain-dump draft survives switching views` | committed `1ecc376` |
| 51 | 14a | `feat(ui): parse preview in the brain-dump box` | committed `08ec23e` |
| 52 | 14a | `fix(ui): Week view add indicator clears after Enter` | committed `9c1078e` |
| 53 | 14a | `fix(ui): Blotter mark on the setup screens` | committed `5b36d32` |
| 54 | 14b | `feat: tag management in Settings` | committed `ab38e15` |
| 55 | 14b | `feat(ui): themed context menu` | committed `a836678` |
| 56 | 14c | `feat(ui): drag items in the Month view and all-day lane` | committed `5b957f5` |
| 57 | 14d | `feat: task context in quick add and Smart slot` | committed `99707a6` |
| 58 | 14d | `feat(core): holidays` | committed `cfc4052` |
| 59 | 14d | `feat(core): task windows with replanning` | committed `0579005` |
| 60 | 14d | `feat: weekly tasks` | committed `b2d6ad2` |
| 61 | 14e | `feat(ai): Codex CLI sessions` | committed `26ecac0` |
| 62 | 14e | `feat(ai): Consult me slot mode` | committed `ad1e018` |
| 63 | 14f | `feat(ui): new Blotter logo` | committed `690777d` |
| 63a | 14f | `fix(ui): no grey edge on the logo's spark and rings` | committed `0e139df` |
| 63b | 14f | `fix(ui): logo closer to its design, gradient, no taskbar tile` | committed `7359512` |
| 63c | 14f | `fix(ui): logo keeps a shaded underside below the gradient` | committed `521ee95` |
| 63d | 14f | `feat(theme): a colour for the logo's shaded underside` | committed `8aa6f9a` |
| 63e | 14f | `feat(theme): the official logo colours in every preset and the icon` | committed `4ac476c` |
| 63f | 14f | `fix(ui): logo spark moved down instead of stretched` | committed `36ad210` |
| 63g | 14f | `fix(ui): logo spark centred and larger, cutting into the B` | committed `6ec8b01` |
| 64 | 14f | `chore(release): v1.1.0` | committed `a1ff0a1` |
| 64a | 14f | `build: AUR checksum for v1.1.0` | committed (hash in the next entry) |
| 65 | 15 | `docs: v1.2.0 plan` | committed `74ae4f7` |
| 66 | 15a | `feat(ui): new item button in the Inbox rail and top bar` | committed `0c828ab` |
| 67 | 15b | `feat(core): tags whose tasks can share time` | committed `cabc654` |
| 68 | 15c | `fix(core): holidays leave Sleep-type block-outs in place` | committed `57ade5b` |
| 69 | 15c | `feat: public holiday import` | committed `f63b0a4` |
| 70 | 15d | `feat(app): in-app updates` | committed `46c48ed` |
| 70a | 15d | `fix(ui): plain update errors and the holiday list's empty line` | committed `2d94a55` |
| 71 | 15d | `chore(release): v1.2.0` | committed `4cb1d74` (untagged) |
| 72 | 16 | `docs: v1.2.1 sync plan` | committed `7cb6137` |
| 72a | 16 | `docs: sync schema gate confirmed, Blotter-Sync repo name` | committed `78626ec` |
| 73 | 16a | `feat(core): device identity and hybrid logical clock` | committed `eda091d` |
| 74 | 16a | `feat(core): sync registry for tables, columns and settings` | committed `12a01e5` |
| 75 | 16a | `feat(core): capture synced writes into an outbox` | committed `bbaf848` |
| 76 | 16a | `feat(core): wire types and merging remote changes per field group` | committed `b875e15` |
| 77 | 16a | `test(core): multi-replica convergence tests` | committed `d5ec285` |
| 78 | 16a | `feat(core): undo and redo scoped to this device` | committed `4dd7ae4` |
| 79 | 16a | `refactor(core): subscription and holiday upkeep a caller can drive` | committed `3b87cab` |
| 79a | 16a | `build: rust-version 1.95` | committed `aa31b69` |
| 79b | 16 | `fix(ui): the Diary moves on to today when the date changes` | committed `bd0017b` |
| 79c | 16a | `test(core): the clock test no longer expires` | committed `6b33cb5` |
| 79d | 16a | `feat(core): undo per field group` | committed `661cbeb` |
| 79e | 16a | `feat(core): Blotter Sync's request and response types` | committed `2c89f08` |
| 80 | 16b | `feat(app): sync engine with push, pull and address failover` | committed `f22b2ae` |
| 81 | 16b | `feat(app): pairing and Settings → Sync` | committed `7673cef` (page confirmed by Luke) |
| 82 | 16b | `feat(app): connect to a home server from first run, and joining` | committed (hash in the next entry) |
| 83 | 16b | `feat(app): hand automatic duties to the server` | planned — after the live check of the server's duties (Luke's home server, roadmap §14 progress) |
| 84 | 16b | `feat(ui): sync status, device names in History, server snapshots` | planned |
| 85 | 16c | `chore(release): v1.2.1` | planned |

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

### #29a — MCP server over the operation registry            [status: committed e43fd31]
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

### #30 — Tool loop, chat threads and the Claude Code provider            [status: committed aafb799]
Phase: 8 (D29, D30)
Message:
    feat(ai): tool-calling loop and Claude Code provider

    The chat backend. A turn runs through Claude Code (the default, D29)
    or the default API provider, streams to the UI as chat:event, and is
    saved in its thread. Turns can be cancelled, and a thread only runs
    one turn at a time. The chat panel itself comes in #31.

    Claude Code: Blotter runs the user's installed, signed-in `claude -p`
    on their subscription. Blotter's MCP server is its only tool source:
    no built-in tools, no other MCP servers, and permission for Blotter's
    tools only. Changes are logged as ai:claude-code/<model> (new
    BLOTTER_MCP_ACTOR), with a snapshot before the first change of each
    turn. Each thread resumes its Claude session, starting a fresh one if
    that session is gone. `claude` is found on PATH or in its install
    folders (WinGet, ~/.local/bin, …), or at a path the user sets. The
    prompt goes on stdin, and no console window flashes on Windows.

    API providers (experimental): the tool loop offers the agent ops as
    tools. Malformed or invalid calls go back as errors for a retry;
    cut-off or refused replies don't run tools. It stops after 8 tool
    rounds, and snapshots before a turn's first change.

    Core:
    - The system prompt: now, timezone, planning hours, block-outs,
      today's agenda with ids, Inbox, overdue and conflict counts, and
      the scheduling rules.
    - Chat thread and message storage. Migration 0004 adds
      chat_threads.external_session.
    - AI route and Claude Code model/path settings, kept per device.
    - CoreError::for_agent, shared with the MCP server.

    UI: Settings → AI providers gets a Claude Code card (status, model,
    program path, Use for chat). API providers show "In use" only when
    the API route is on.
Files:
    A  crates/blotter-core/migrations/0004_chat_session.sql
    A  crates/blotter-core/src/context.rs
    A  crates/blotter-core/src/store/chat.rs
    A  src-tauri/src/ai/agent.rs
    A  src-tauri/src/ai/chat.rs
    A  src-tauri/src/ai/claude_code.rs
    A  src/lib/aiSettings.ts
    A  src/views/settings/ClaudeCodeCard.tsx
    M  crates/blotter-core/src/db.rs           (migration 0004, schema v4 in tests)
    M  crates/blotter-core/src/error.rs        (for_agent)
    M  crates/blotter-core/src/lib.rs          (context)
    M  crates/blotter-core/src/settings.rs     (AI keys, device keys)
    M  crates/blotter-core/src/store/mod.rs    (chat)
    M  crates/blotter-mcp/src/lib.rs           (with_actor, for_agent, chat snapshot note)
    M  src-tauri/Cargo.toml                    (tokio: process, io-util, time)
    M  Cargo.lock
    M  src-tauri/src/ai/mod.rs, ai/commands.rs (modules; ai_err pub(crate))
    M  src-tauri/src/lib.rs                    (MCP_ACTOR_ENV, ChatRuns, commands)
    M  src/views/settings/AiProvidersPage.tsx
    M  .claude/COMMITS.md, .claude/roadmap.md  (D30, Phase 8 progress)
Test steps:
    - cargo test --workspace → 136 core (+4: system prompt ×2, chat store ×2) + 32 app
      (+10: tool loop ×4 with a scripted model, stream-json parser ×2 on recorded lines,
      args, MCP config, thread replay, previews) + 5 mcp (+1 actor)
    - cargo clippy --workspace --all-targets -- -D warnings → clean; cargo fmt --check → clean
    - pnpm check → clean; Vitest 164
    - pnpm dev:demo, chat driven over WebView2 CDP (no panel until #31): chat_send "Add a
      30 minute task called Water the plants…" → Claude Code calls create_item, then
      list_inbox → "You now have 8 items"; the sidebar Inbox moved 7 → 8 live
Notes:
    - Verified live on Luke's subscription (D30), Claude Code 2.1.292 from WinGet, found
      through the install-folder fallback (the app's PATH predated the install). Status:
      "signed in with your Claude subscription". The change was logged as
      ai:claude-code/sonnet with a before_ai_batch snapshot. The second message resumed
      the thread's session. Cancelling mid-turn left no claude.exe or blotter mcp behind,
      and the thread took a new message straight away.
    - Found and fixed live: `--allowedTools blotter` doesn't match MCP tools. The rule is
      `mcp__blotter`. Reads still worked without it; writes were refused under dontAsk.
    - Not verified: the API route against a real provider (no key; experimental, D29).
      Linux/Flatpak: inside the sandbox, `claude` would need `flatpak-spawn --host`
      (Phase 13).
    - Until #32 there are no proposals: changes apply at once (D30). Undo batch, the
      Propose/Auto switch and proposal cards come in #32. The batch ids written by a
      Claude Code turn aren't recorded on its message yet; #32 needs them.
    - Dev gotcha: a Claude Code session registered with target\debug\blotter.exe keeps the
      exe open, which blocks relinking. Rename the running exe aside (Windows allows it)
      rather than killing the session.
    - Next: #31 chat panel and pop-out window.

### #31 — Chat panel and pop-out window            [status: committed 4edf2d6]
Phase: 8 (design: Chat Panel)
Message:
    feat(ui): chat panel and pop-out window

    The chat, following the Chat Panel design. It docks on the right of
    the main window (Ctrl+/ or the new top-bar button) or pops out into
    a slim always-on-top "sticky note" window (Ctrl+Shift+/) that docks
    back. `blotter --chat` opens it in the running app.

    - Header: thread title, new thread, a thread list (switch or delete),
      pop out / close (docked) or keep on top / dock back (pop-out).
      Chips show the AI in use ("Claude Code · sonnet") and the mode.
      Mode reads "Auto" until proposals arrive in #32 (D30).
    - Messages: user bubbles; tool calls as mono lines with their
      arguments, red with the error when one fails; replies stream with
      a cursor and render light Markdown (paragraphs, lists, bold, code)
      as React elements, never HTML; error cards for failed turns.
    - Composer: Enter sends, Shift+Enter adds a line, Stop cancels a
      running turn. A new thread offers the design's three suggestions.
    - Backend: chat_window.rs opens, pins and docks the pop-out. Tool
      starts now carry their input.
Files:
    A  src-tauri/src/chat_window.rs
    A  src/app/ChatApp.tsx
    A  src/chat/ChatPanel.tsx
    A  src/chat/chatStore.ts
    A  src/chat/markdown.tsx
    A  src/chat/markdown.test.tsx
    A  src/chat/mdParse.ts
    A  src/chat/transcript.ts
    A  src/chat/transcript.test.ts
    M  src-tauri/capabilities/default.json  (chat window)
    M  src-tauri/src/ai/agent.rs, ai/chat.rs (tool input on tool_start)
    M  src-tauri/src/lib.rs
    M  src/actions/actions.ts               (chat.toggle, chat.popout, chat.new)
    M  src/app/AppShell.tsx                 (docked panel, --chat and dock events)
    M  src/app/TopBar.tsx                   (Toggle AI chat button)
    M  src/main.tsx                         (chat window renders ChatApp)
    M  src/styles.css                       (cursor blink)
    M  src/views/settings/ClaudeCodeCard.tsx (tag-health colour; there is no success token)
    M  .claude/COMMITS.md, .claude/roadmap.md
Test steps:
    - cargo test --workspace → 136 core + 33 app (+1 pop-out URL) + 5 mcp; clippy / fmt clean
    - pnpm check → clean; Vitest 172 (+8: transcript, live events, argument summaries,
      Markdown parse and escaping)
    - pnpm dev:demo: top-bar chat button → "Brief me on today" → tool lines, a streamed
      brief with headings and lists; type a request → one bubble, create_item line,
      Inbox count updates; Pop out → sticky window on the thread; Dock → back in the
      main window; close it, run `blotter --chat` → it opens again
Notes:
    - Verified live (WebView2 CDP, Claude Code on the subscription): everything in the test
      steps, with screenshots of the docked panel mid-stream and finished, and of the pop-out.
    - Found and fixed live: the first message showed twice (a live copy plus the saved
      copy). It now shows only the saved copy, which is written before the turn starts.
      Also: creating the pop-out in a sync command left it blank (a WebView2 deadlock on
      Windows). open_chat_window is async now.
    - Not verified: real keypresses for Ctrl+/ and Ctrl+Shift+/ (my harness can't send
      Ctrl; the top-bar button and actions run the same code); always-on-top on Wayland.
    - Mishap while testing: `taskkill /IM blotter.exe` also stopped the `blotter mcp`
      server Luke's VS Code Claude session had started. A new session starts a new one.
      I now stop processes by PID only.
    - Next: #32 proposal review and auto-apply mode.

### #31a — Keep Claude Code running between messages            [status: committed bd48332]
Phase: 8 (Luke: "basic prompts take quite a while")
Message:
    perf(ai): keep Claude Code running between messages

    A basic chat request took about 15 s, and only about 3.5 s of that was
    the model. The rest was starting `claude` fresh for every message:
    about 2.5 s for Windows to start the program and 8 s for Claude Code's
    own startup, most of it waiting on its MCP connection. Blotter's
    server answers in 0.35 s, and turning off hooks or user settings made
    no difference.

    Now each thread keeps a Claude Code process running. Messages go on
    its stdin as stream-json, and replies stream back. Opening the chat
    or a new thread starts a process ahead of time; that makes no model
    call. Processes stop after 10 minutes idle, at most two threads keep
    one, and a model change starts a new one. If a process ends (Stop,
    or a crash), the next message starts a new one that resumes the
    thread's session. Blotter exiting closes their stdin, which ends
    them.

    Because a long-running process keeps its system prompt, the prompt
    is now only the fixed instructions. A planner snapshot (now, planning
    hours, block-outs, today's agenda, and now the Inbox items with ids)
    travels with each message. Listing the Inbox saves the model a
    list_inbox round trip on Inbox requests.

    Spawned `claude` no longer inherits the CLAUDECODE / CLAUDE_CODE_*
    variables of a Claude session Blotter was started from. They made it
    act as that session's child.
Files:
    M  crates/blotter-core/src/context.rs   (instructions + snapshot, Inbox listed)
    M  src-tauri/src/ai/claude_code.rs      (Session over stream-json, env cleaning)
    M  src-tauri/src/ai/chat.rs             (ClaudeSessions pool, chat_prewarm, reaper)
    M  src-tauri/src/lib.rs                 (state, reaper, command)
    M  src-tauri/Cargo.toml                 (tokio sync)
    M  src/chat/ChatPanel.tsx               (warm up on open and thread change)
    M  .claude/COMMITS.md
Test steps:
    - cargo test --workspace → 136 core + 35 app (+2: stream-json input lines, inherited
      variables dropped) + 5 mcp; clippy / fmt clean
    - pnpm check → clean; Vitest 172
    - Timed by hand with the exact arguments (claude 2.1.292, WinGet): one process per
      message → init at 10.8 s, total ~15 s; one kept process → messages 2 and 3 in
      2.7 s and 2.3 s (create_item, update_item, list_inbox)
Notes:
    - Verified live by Luke in his dev app (rebuilt on the change): "way better now".
      I couldn't drive it myself, since his instance held the single-instance lock.
    - Startup breakdown measured from Claude Code's startup timing (CLAUDE_CODE_EMIT_
      STARTUP_TIMING): hooks_init 7.1 s waiting alongside mcp_connect 4.7 s. --bare starts
      faster but never reads OAuth, so it can't use the subscription.
    - Memory: each warm `claude` process is a few hundred MB, hence the cap and idle stop.
    - Next: #32 proposal review and auto-apply mode.

### #32 — Proposal review and the Propose/Auto switch            [status: committed 0a2b64a]
Phase: 8 (D8; Luke: "make the auto/propose remembered")
Message:
    feat(ai): proposal review and auto-apply mode

    Propose mode (the default, D8): a change the AI or an MCP client asks
    for is not made. It is rehearsed (run in a transaction that is rolled
    back) and kept as a proposed operation with the before/after the
    rehearsal showed. The model gets the rehearsal's result, marked
    "Proposed, not applied", so it can chain proposals and tells the user
    nothing has changed yet. Bad arguments and rule breaks (a flexible
    task in a block-out) still come back as errors for a retry.

    Ids stay stable: each rehearsal replays the proposal's earlier pending
    operations first. The entity ids a rehearsal creates are recorded and
    handed out again on later replays and on accept, so "create X, then
    schedule X" works and an id the model was told about stays the id.
    Operations now get entity ids from their context (Ctx::new_id).

    Accepting runs the chosen operations for real, in order, as the AI
    that proposed them, under one undoable batch, each in its own
    savepoint. One that can't apply any more (it needed a change that was
    rejected) is marked failed with a reason; the rest still apply. A
    proposal is pending, accepted, rejected or partial.

    A chat turn's operations form one proposal (card). The app seals it
    when the turn ends; those Claude Code made through Blotter's MCP
    server are claimed for the thread. Proposals from other MCP clients
    stay unthreaded until decided.

    The mode is a setting (ai_mode). It's kept per device and is
    remembered as it was left. Auto applies changes at once, as before,
    after a safety snapshot. The snapshot tells the model which mode is
    on.

    UI: proposal cards in pencil (design: Chat Panel), with accept/reject
    per change and Accept all (Ctrl+Enter) / Reject all. Each change reads
    like "~ Fix the tap · Inbox → Sat 17 Oct 10:30–11:15". The mode chip
    in the chat header switches Propose/Auto, and Settings → AI providers
    has the same switch. A badge on the chat button counts changes
    waiting, including proposals from MCP clients, which show at the top
    of the chat.
Files:
    A  crates/blotter-core/migrations/0005_proposal_ids.sql
    A  crates/blotter-core/src/proposals.rs
    A  src/chat/ProposalCard.tsx
    A  src/chat/proposals.ts
    A  src/chat/proposals.test.ts
    M  crates/blotter-core/src/ops/mod.rs        (Ctx::new_id, run_each with savepoints)
    M  crates/blotter-core/src/ops/{blockouts,items,occurrences,reminders}.rs (ctx.new_id)
    M  crates/blotter-core/src/planner.rs        (run_each)
    M  crates/blotter-core/src/context.rs        (mode line in the snapshot)
    M  crates/blotter-core/src/settings.rs       (KEY_AI_MODE, device key)
    M  crates/blotter-core/src/db.rs, lib.rs     (migration 0005, schema v5)
    M  crates/blotter-mcp/src/lib.rs, Cargo.toml (propose in Propose mode; tests)
    M  src-tauri/src/ai/agent.rs                 (Place, propose in the tool loop)
    M  src-tauri/src/ai/chat.rs                  (mode setting, seal, proposal commands)
    M  src-tauri/src/lib.rs
    M  src/chat/ChatPanel.tsx, chatStore.ts, transcript.ts (+test)
    M  src/app/TopBar.tsx                        (pending badge)
    M  src/lib/aiSettings.ts, src/views/settings/AiProvidersPage.tsx
    M  Cargo.lock, .claude/COMMITS.md
Test steps:
    - cargo test --workspace → 141 core (+5 proposals: rehearsal changes nothing and chained
      ids survive accept; a rejected prerequisite fails what needed it; partial accept and bad
      calls; sealing; the mode is remembered) + 36 app (+1 propose in the tool loop) + 6 mcp
      (+1 propose mode records instead of changing); clippy / fmt clean
    - pnpm check → clean; Vitest 178 (+6: change lines, actor labels, cards after their turn)
    - Headless Claude Code through `blotter mcp` on the demo DB (Propose): "create Chain test,
      then schedule it tomorrow 18:30" → create_item then schedule_item with the rehearsal's
      id, both recorded, no item created, and the reply says nothing is applied yet. 10:00
      tomorrow → the Work block-out error came back and the model asked instead
Notes:
    - Not driven live by me: the cards, the mode chip, the Settings switch and the badge.
      Luke's dev app held the single-instance lock while I worked. His demo database has 3
      pending test changes from the checks above (an MCP card at the top of the chat), which
      exercise the UI: please accept or reject them.
    - Concurrent turns in two threads with the same Claude Code model could claim each
      other's MCP proposals (sealing matches on actor). It's rare; fixing it would need a
      thread id per process.
    - Pencil items on the calendar for pending proposals (D23) aren't drawn yet; the card is
      in pencil.
    - Next: #32a Undo batch for auto-applied changes.

### #32a — Undo batch for auto-applied AI changes            [status: committed d8bc8d7]
Phase: 8 (design: Chat Panel, auto-mode card)
Message:
    feat(ui): undo batch for auto-applied AI changes

    In Auto mode, a chat reply that changed things is followed by a
    "Changed · N items" card, worded like the proposal cards ("~ Fix the
    tap · Inbox → Sat 17 Oct 10:30–11:15"). It notes the snapshot taken
    first and has Undo batch, which undoes the turn's batches newest
    first. Undoing a batch whose items were edited since is refused with
    the reason, as Ctrl+Z is. Once undone, the card reads "Batch undone"
    and points to Ctrl+Shift+Z.

    The batches are found the same way for both routes. The turn notes
    where the change log is when it starts; afterwards it collects the
    batches its actor wrote since (leaving out undo/redo rows) and records
    them on the turn's reply. That covers Claude Code too, whose changes
    are written by the MCP server's own process.
Files:
    A  src/chat/ChangedCard.tsx
    M  crates/blotter-core/src/store/changes.rs  (last_rowid, batches_since)
    M  crates/blotter-core/src/store/chat.rs     (set_turn_batches)
    M  src-tauri/src/ai/chat.rs                  (record the turn's batches; chat_batch_changes)
    M  src-tauri/src/lib.rs
    M  src/chat/ChatPanel.tsx, chatStore.ts, transcript.ts (+test)
    M  .claude/COMMITS.md
Test steps:
    - cargo test --workspace → 143 core (+2: batches_since finds one actor's turn and skips
      undo rows; a turn's batches go on its latest reply) + 36 app + 6 mcp; clippy / fmt clean
    - pnpm check → clean; Vitest 179 (+1: the Changed card follows its reply)
Notes:
    - Not driven live by me: Luke's dev app held the single-instance lock. Please try:
      Auto mode, ask the chat to add two tasks, then Undo batch on the card, which should
      remove them.
    - Redo has no batch argument, so the card offers Undo only; Ctrl+Shift+Z redoes.
    - Next: #33 daily briefing.

### #33 — Daily briefing            [status: committed e43ac71]
Phase: 8 (D28)
Message:
    feat(ai): daily briefing

    The AI briefs the user on their day: today's fixed commitments,
    conflicts and overdue tasks, and the free time left. It then finds
    slots for Inbox tasks, proposed or applied according to the
    Propose/Auto switch.

    - On demand: "Daily briefing" in the command palette starts it in a
      new thread titled "Briefing · Thu 15 Oct" and opens the chat there.
    - Each morning, when turned on (D28: off by default; 07:30): the
      minute check runs it once a day after its time, marking the day
      first so a failure can't repeat it. When it's done, an OS
      notification ("Your daily briefing") shows the start of the reply,
      and the chat opens on it in the main window.
    - Settings → Notifications: a Daily briefing switch and its time.
      Both are in get_settings / update_settings as briefing_enabled and
      briefing_time. The last run date is per device.

    Also: chat_send's turn start is now start_turn, used by the briefing
    too (with a thread title and the notification).
Files:
    A  crates/blotter-core/src/briefing.rs
    M  crates/blotter-core/src/lib.rs
    M  crates/blotter-core/src/settings.rs      (briefing keys; last run is a device key)
    M  crates/blotter-core/src/ops/settings.rs  (briefing_enabled, briefing_time)
    M  src-tauri/src/ai/chat.rs                 (start_turn, chat_briefing, briefing_if_due, announce)
    M  src-tauri/src/chat_window.rs             (DOCK_EVENT shared)
    M  src-tauri/src/rollover.rs                (minute check calls the briefing)
    M  src-tauri/src/lib.rs
    M  src/actions/actions.ts                   (Daily briefing)
    M  src/views/settings/NotificationsPage.tsx
    M  .claude/COMMITS.md, .claude/roadmap.md
Test steps:
    - cargo test --workspace → 145 core (+2: due once a day after its time when on, titles;
      update_settings takes the briefing fields and rejects a bad time) + 36 app + 6 mcp;
      clippy / fmt clean
    - pnpm check → clean; Vitest 179
Notes:
    - Not driven live by me: Luke's dev app held the single-instance lock (it rebuilt on
      these changes at 22:25). The turn itself is the same path as chat_send, which was
      verified live in #30/#31a. Please try: Ctrl+K → "Daily briefing". Then switch it on
      in Settings → Notifications with a time a minute or two ahead and leave the window
      closed to the tray: a notification should arrive and the chat should be open on it.
    - Notification clicks aren't wired (the plugin has no reliable click event on
      desktop); the chat is opened on the thread in the main window instead.
    - Phase 8 is complete with this (#29–#33, #29a, #31a, #32a). Remaining from the AC:
      the API providers against real services (experimental, D29).
    - Next: Phase 9, #34 ICS export and file import.

### #33a — Phase 8 wrap-up and open items            [status: committed 9af9c7c]
Phase: 8
Message:
    docs: Phase 8 wrap-up and open items

    Records #33's hash, and in the roadmap the items still open at the
    end of Phase 8: Linux CI to confirm, proposal UI only partly tried, a
    published calendar URL needed for #35, Anthropic's terms to check
    before publishing the Claude Code route, and the untested API-key
    providers.
Files:
    M  .claude/COMMITS.md
    M  .claude/roadmap.md
Test steps:
    - none (docs only)
Notes:
    - Next: #34 ICS export and file import.

### #34 — ICS export and file import            [status: committed 2551a8e]
Phase: 9 (roadmap §5.8; design: Settings, Calendars (ICS))
Message:
    feat: ICS export and file import

    Core (blotter-core::ics, no new dependencies):
    - An RFC 5545 text layer: unfolding, quoted parameters, TEXT escaping,
      lenient component parsing, and output folded at 75 octets.
    - Export of everything or a date range. Scheduled items become
      VEVENTs (so tasks show in Google and Apple Calendar), Inbox tasks
      VTODOs. Plain items are written in UTC. A series is written in its
      zone with a VTIMEZONE, so its wall-clock time survives DST; changed
      occurrences follow with RECURRENCE-ID, deleted ones as EXDATE.
      Reminders become VALARMs, tags CATEGORIES. Kind, anchoring,
      completion, colour and due go in X-BLOTTER-* properties, so a
      Blotter export imports back whole.
    - Import reads VEVENT/VTODO, DTSTART/DTEND/DURATION in UTC, floating,
      IANA, prefixed or Windows (Outlook) zones, RRULE (UNTIL made UTC),
      EXDATE, RECURRENCE-ID, VALARMs before the start, CATEGORIES,
      PRIORITY and LOCATION (into notes). Rules Blotter can't expand
      (hourly, broken) import once with a warning; unknown zones are read
      in the user's with a warning; cancelled events are left out.
    - Items keep their UID (migration 0006: items.ics_uid). Importing the
      same file twice, or a Blotter export into the Blotter it came from,
      adds nothing.
    - A preview says what a file adds before anything is written. The
      import itself is the user-only operation import_ics: one batch, so
      Ctrl+Z undoes it whole ("Import ICS (12 items)"), after a "Before
      ICS import" snapshot (none when nothing is new). A flexible item
      that would land in a block-out is made fixed.

    App/UI: Settings → Calendars (ICS) with Import .ics (native picker,
    then a confirm listing what it adds, its date span and warnings) and
    Export… (Everything or a date range). Subscriptions join this page in
    #35.
Files:
    A  crates/blotter-core/migrations/0006_item_ics_uid.sql
    A  crates/blotter-core/src/ics/mod.rs        (import with snapshot; round-trip tests)
    A  crates/blotter-core/src/ics/text.rs
    A  crates/blotter-core/src/ics/zones.rs
    A  crates/blotter-core/src/ics/export.rs
    A  crates/blotter-core/src/ics/import.rs
    A  crates/blotter-core/src/ops/ics.rs        (import_ics)
    A  src-tauri/src/ics.rs                      (export_ics, pick_ics_import, apply_ics_import)
    A  src/lib/icsFormat.ts, src/lib/icsFormat.test.ts
    A  src/views/settings/CalendarsPage.tsx
    M  crates/blotter-core/src/db.rs             (migration 0006, schema v6)
    M  crates/blotter-core/src/lib.rs
    M  crates/blotter-core/src/model/item.rs     (ics_uid)
    M  crates/blotter-core/src/store/items.rs    (ics_uid column)
    M  crates/blotter-core/src/ops/mod.rs        (registry; import_ics is user-only)
    M  crates/blotter-core/src/ops/undo.rs       ("Import ICS" label)
    M  src-tauri/src/lib.rs
    M  src/app/nav.ts, src/lib/types.ts, src/views/SettingsView.tsx
    M  .claude/COMMITS.md, .claude/roadmap.md
Test steps:
    - cargo test --workspace → 159 core (+14: text layer, zones and VTIMEZONEs, durations,
      UNTIL, export→import gives the same items incl. series overrides and reminders,
      re-import adds nothing, one undoable batch and agents refused, date ranges, an
      Outlook/Google-style file, snapshot only when something is new) + 36 app + 6 mcp;
      clippy / fmt clean
    - pnpm check → clean; Vitest 183 (+4 wording)
    - A copy of the demo database: exported 26 calendar items and 9 Inbox tasks (no line over
      75 octets); imported into an empty database → the same 35, 14 reminders, no warnings;
      previewed into its own database → 35 already there, nothing new
Notes:
    - Not driven live by me: the page, the native Save/Open dialogs and the confirm.
      Luke's dev:demo was running while I worked. Please try Settings → Calendars: Export…
      both ways, then Import .ics that file (expect "nothing new"), and an export from
      Google or Outlook.
    - Roadmap AC 1 still needs Luke: import an exported file into Google Calendar and the
      Apple Calendar app.
    - Google Calendar ignores VTODOs, so Inbox tasks only reach calendars that read to-dos.
    - Next: #35 subscriptions (needs a published calendar URL).

### #35 — ICS subscriptions with block-out option            [status: committed 73d43a7]
Phase: 9 (roadmap §5.8, §4.5; design: Settings, Calendars (ICS))
Message:
    feat: ICS subscriptions with block-out option

    Subscribe to a calendar published as a link (webcal:// or https://),
    such as an Outlook work calendar. It stays read-only, refreshes on
    its own, and with "Treat as block-out" on, its busy times keep
    flexible tasks out.

    Core:
    - blotter-core::subscriptions: add, update, remove and list
      subscriptions, and record a fetch. webcal:// is fetched over https;
      other schemes are refused. Refresh every 5 minutes to 7 days
      (default 1 hour). Like AI providers they are the user's settings,
      outside the operation registry and undo, so agents can't make the
      app fetch URLs.
    - ics::feed reads a feed with the import's reader (Windows zones,
      RRULE, EXDATE, RECURRENCE-ID) and expands each series from a year
      ago to two years ahead into the existing ics_events cache. Events
      marked free (TRANSP:TRANSPARENT or Outlook's FREE) never block. The
      calendar's own name (X-WR-CALNAME) replaces the host name it starts
      with. A failed fetch, or a page that isn't a calendar, keeps the
      previous events and records why.
    - windows::load (now given the user's zone) adds the busy timed
      events of block-out calendars as windows with `calendar` set. So
      find_free_slots, auto_slot, the placement check, drag refusal, the
      Diary meter and the shading all respect them; fixed items may sit
      in them without a conflict. All-day events never block (on
      Outlook they're usually free, or leave).
    - New read operation list_calendar_events, so the AI sees subscribed
      events too.

    App: a fetch loop checks once a minute for subscriptions due (HTTP
    outside the planner lock, 20 MB cap, timeouts), and a Refresh never
    runs twice at once. Commands list/add/update/remove/refresh_subscription.

    UI: Settings → Calendars gets Subscriptions: paste a URL and
    Subscribe (fetched at once), then per calendar its name (editable),
    status ("Fetched 10:30 · 48 events", or the error and the last
    success), refresh interval, Treat as block-out, Refresh/Retry and
    Unsubscribe. Shading from a calendar says so on hover ("Team sync
    10:00–11:00 (busy in Work)").
Files:
    A  crates/blotter-core/src/model/subscription.rs
    A  crates/blotter-core/src/store/subscriptions.rs
    A  crates/blotter-core/src/subscriptions.rs     (management, record_fetch; AC tests)
    A  crates/blotter-core/src/ics/feed.rs           (feed → cached occurrences)
    A  src-tauri/src/subscriptions.rs                (fetcher, minute check, commands)
    A  src/views/settings/SubscriptionsSection.tsx
    M  crates/blotter-core/src/ics/import.rs         (plan_feed; busy and location kept)
    M  crates/blotter-core/src/windows.rs            (subscription windows; load takes the zone)
    M  crates/blotter-core/src/ops/reads.rs          (list_calendar_events)
    M  crates/blotter-core/src/ops/slots.rs          (zone; refusal names the calendar)
    M  crates/blotter-core/src/slots.rs, lib.rs, model/mod.rs, store/mod.rs, ics/mod.rs
    M  src-tauri/src/lib.rs
    M  src/lib/icsFormat.ts (+ test), src/lib/availability.ts (+ test), src/lib/types.ts,
       src/lib/dnd.test.ts
    M  src/views/WeekView.tsx, MonthView.tsx, diary/TimelineRail.tsx (hover text)
    M  src/views/settings/CalendarsPage.tsx
    M  .claude/COMMITS.md, .claude/roadmap.md
Test steps:
    - cargo test --workspace → 166 core (+7: an Outlook-style feed with Windows zones, a
      moved and a cancelled occurrence, free and all-day events; a Blotter export as a feed;
      URLs; add/update/remove; fetches fill the cache and failures keep it; a block-out
      calendar moves find_free_slots past its meeting, refuses a flexible task with the
      calendar named, allows a fixed one without a conflict, and stops when off or paused)
      + 41 app (+5: fetch over local HTTP, 404, over 20 MB, nothing listening, and fetch →
      cache → a refused task end to end) + 6 mcp; clippy / fmt clean
    - pnpm check → clean; Vitest 187 (+4: refresh labels, status wording, hover text)
Notes:
    - Not driven live by me: the page and the minute check in the running app. Please try
      Settings → Calendars → Subscriptions with your published Outlook URL (Outlook on the
      web → Settings → Calendar → Shared calendars → Publish a calendar → ICS link), turn
      on Treat as block-out, then drag a flexible task onto one of its meetings (expect a
      refusal) and run auto-slot. That's roadmap AC 2.
    - Subscribed events aren't drawn as events yet, only as block-out shading when that's
      on; #35a draws them read-only in Diary, Week and Month (with the calendar's colour).
    - Subscriptions aren't in the JSON export (AI providers aren't either); a database
      snapshot keeps them.
    - Next: #35a.

### #35a — Subscribed events in the calendar views            [status: committed 05cceab]
Phase: 9 (roadmap §5.8: "Subscribed items are read-only, have their own colour")
Message:
    feat(ui): subscribed events in the calendar views

    Events from subscribed calendars now show in Diary, Week and Month,
    read-only and in their calendar's colour (a dashed edge and a link
    icon set them apart from your own items; free events are fainter).

    - Week: timed events share the lanes with items, all-day ones the
      all-day row. They can't be focused, dragged or resized; arrow
      keys skip them.
    - Month: events take the chips left after a day's items, and the
      "+N more" popover lists them too.
    - Diary: each day column lists its events as quiet lines above the
      cards, and the timeline rail draws the timed ones.
    - Hover shows time, location, calendar, and whether it's free.
    - Settings → Calendars: each calendar's dot picks its colour (the
      theme's calendar colour, or one of the tag colours).

    One cached list_calendar_events query covers every view, over the
    same range as the block-out windows.
Files:
    A  src/lib/calendarEvents.ts (+ test)
    M  src/lib/days.ts                      (itemsByDay takes anything with a day span)
    M  src/views/WeekView.tsx, src/views/MonthView.tsx
    M  src/views/diary/DiaryView.tsx, DayColumn.tsx, TimelineRail.tsx
    M  src/views/settings/SubscriptionsSection.tsx (colour)
    M  .claude/COMMITS.md, .claude/roadmap.md
Test steps:
    - pnpm check → clean; Vitest 190 (+3: calendars resolved and keyed apart from items,
      times and hover text, bucketing by day)
    - Rust unchanged (166 core + 41 app + 6 mcp still pass)
Notes:
    - Not driven live by me. With a subscription added (#35), please look at a week with
      its meetings: they should sit beside your items without covering them, ignore arrow
      keys and drags, and recolour when you pick a colour in Settings → Calendars.
    - Phase 9 is complete in code. Open for Luke: AC 1 (an export into Google Calendar and
      Apple Calendar) and AC 2 live (a published Outlook URL as a block-out).
    - Next: Phase 10, #36 keybinding editor.

### #36 — Keybinding editor            [status: committed aa8c036]
Phase: 10 (roadmap §5.4)
Message:
    feat(ui): keybinding editor

    Settings → Keybindings lists every action by group (the same registry
    as the palette and the ? sheet), with its keys and where they work
    (Calendar views, Week, Inbox).

    - Search by action name, group or keys.
    - Click a binding, or Tab to it and press Enter, then press the new
      keys; a chord is its keys pressed one after another (the capture
      ends after a second's pause, or three keys). Esc cancels.
      Nothing else in the app sees keys while capturing.
    - Conflicts are caught before saving: the same keys, or a chord one
      starts with ("g" against "g w"), in contexts live at the same time
      (the calendar-view keys count as live in each of those views).
      Swap (for one exact clash: the other action gets this one's old
      keys), Take it (the other loses those keys), or Cancel.
    - Remove an action's keys, reset one, or reset all.
    - Import… / Export… a JSON file of your changes; an import leaves
      out actions this version doesn't have and keys it can't read.
    - Changes work on the next key press: the keymap, palette and ?
      sheet all read the overridden actions. Any action can get keys,
      including ones without a default (e.g. a theme).
    - The global capture shortcut (Ctrl+Shift+Space) stays as it is:
      the operating system holds it.

    Overrides are stored in the keybindings table (one binding per
    action, or none), so a snapshot restore brings them back too. The ?
    sheet links to the page, and "Edit keybindings" is in the palette.
Files:
    A  crates/blotter-core/src/store/keybindings.rs
    A  src-tauri/src/keybindings.rs                 (list/set/replace, export, pick import)
    A  src/keymap/overrides.ts (+ test)             (effective bindings, clashes, assign, import)
    A  src/keymap/keybindingStore.ts
    A  src/views/settings/KeybindingsPage.tsx
    M  crates/blotter-core/src/store/mod.rs
    M  src-tauri/src/lib.rs
    M  src/actions/actions.ts                       (NAV_VIEWS moved here; Edit keybindings)
    M  src/app/AppShell.tsx                         (overrides applied; reloaded after restore)
    M  src/app/ShortcutSheet.tsx, src/app/nav.ts, src/views/SettingsView.tsx
    M  .claude/COMMITS.md, .claude/roadmap.md
Test steps:
    - cargo test --workspace → 167 core (+1 overrides round trip) + 42 app (+1 the file
      format) + 6 mcp; clippy / fmt clean
    - pnpm check → clean; Vitest 197 (+7: defaults vs overrides vs unbound, the fixed
      capture shortcut, applied actions, context overlap, exact and prefix clashes, swap /
      take, import cleaning)
Notes:
    - Not driven live by me. Please try: rebind Quick add to q and use it straight away;
      give something n to see the Swap / Take it choice; bind g alone to see the chord
      clash; restart and check they stuck; Export…, Reset all, Import… back.
    - The ? sheet's family rows ("h j k l") describe the defaults; an overridden member of
      a family shows as its own row instead.
    - Next: #37 theme engine.

### #36a — Quick capture info on the Keybindings page            [status: committed aa98dad]
Phase: 10
Message:
    refactor(ui): quick capture info on the Keybindings page

    The Quick capture section (whether Ctrl+Shift+Space is registered,
    or the command to bind on Wayland) moves from About to Settings →
    Keybindings, as its comment planned, beside the locked capture row.
Files:
    A  src/views/settings/QuickCaptureInfo.tsx
    M  src/views/SettingsView.tsx, src/views/settings/KeybindingsPage.tsx
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 197
Notes:
    - Missed from #36. Next: #37 theme engine.

### #37 — Theme engine with live preview            [status: committed daf1ff4]
Phase: 10 (roadmap §5.5)
Message:
    feat(ui): theme engine with live preview

    Settings → Theme now makes themes as well as picking them.

    - Your themes sit beside the four presets. Customise (or Duplicate)
      a preset to make your own copy; Edit, Export… or Delete your own.
      Import… reads a .blotter-theme.json file into the editor.
    - The editor: every colour token by group (picker and hex field),
      plus Corners, Density, Text size, Font and Headings as choices
      that set their tokens together. Save keeps it, Apply saves it and
      switches the app to it, Cancel drops the draft.
    - A live preview gallery beside it shows the draft on real
      components, scoped to the gallery so the rest of the app keeps its
      theme until Apply: buttons, a field, switches, a select, key chips
      and a tooltip, Diary cards (with a conflict), a slice of the Week
      grid with a block-out, a conflict, a pencilled proposal and the
      now line, a chat bubble with a proposal card, a dialog, a reminder
      toast and the command palette. It's inert: nothing in it can be
      clicked or focused.
    - Contrast: the nine text/background pairs the app draws are checked
      against WCAG AA (4.5:1); pairs below it are listed above the
      editor, worst first, and marked on the colours involved.
    - Density is now real: buttons and selects take their height from
      --density-row (26 / 30 / 34px).

    Themes are stored in the themes table; a token must be a plain
    custom property with a plain value (no url(), @import, braces or
    semicolons), checked in the core and when reading a file. Missing
    or unknown tokens fall back to Dark's, so themes from other versions
    still draw everything. The active theme is cached whole, so a user
    theme paints right on the first frame.
Files:
    A  crates/blotter-core/src/store/themes.rs
    A  src-tauri/src/themes.rs                    (list/save/delete, export, pick import)
    A  src/theme/engine.ts (+ test)               (contrast, choices, reading tokens; luminance moved here)
    A  src/views/settings/ThemePage.tsx, ThemePreview.tsx
    M  crates/blotter-core/src/store/mod.rs, src-tauri/src/lib.rs
    M  src/theme/theme.ts                         (user themes, whole-theme cache, save/delete)
    M  src/theme/presets.ts                       (font stacks exported)
    M  src/components/ui/button.tsx, select.tsx   (density)
    M  src/views/SettingsView.tsx                 (old theme page removed)
    M  .claude/COMMITS.md, .claude/roadmap.md
Test steps:
    - cargo test --workspace → 169 core (+2: round trip, unsafe tokens refused) + 43 app (+1
      the file format) + 6 mcp; clippy / fmt clean
    - pnpm check → clean; Vitest 205 (+8: ratios, presets against AA, a bad draft's issues
      worst first, choices recognised, text scaling, reading tokens, user themes, copy names)
Notes:
    - Not driven live by me. Please try: Customise Dark, drag the accent and watch only the
      preview change; Apply; restart (it should open in your theme with no flash); set muted
      text dark to see the warnings; Export…, Delete, Import… back; Compact density.
    - Gruvbox's danger red (#fb4934, Gruvbox's own) is 4.2:1 on its background, just under
      AA, so editing Gruvbox shows that warning. I left the colour as Gruvbox has it; say if
      you'd rather it be lifted to pass.
    - Your themes aren't in the command palette's theme list yet (presets only).
    - Next: #38 remaining settings pages and first-run setup.

### #38 — General settings page            [status: committed f6b8d44]
Phase: 10 (roadmap §5.9; design S1). Split from the planned #38: first-run setup is #38a.
Message:
    feat(ui): General settings page

    Settings → General, first in the list and where Settings opens:

    - Week starts on Monday, Sunday or Saturday: Week, Month and Year
      views, the Year view's weekday initials and the date pickers.
    - Clock: 24-hour (17:30) or 12-hour (5:30 pm), for every time shown
      (cards, the grid's hour marks, toasts, the chat, backups…). Times
      you type are still read either way.
    - Timezone, shown: Blotter follows the computer's zone (the views
      draw in it, so a separate zone would put them out of step).
    - Planning hours: one window for every day, or per weekday with days
      off. The slot engine, the Diary's free-time meter and the Month's
      block-out marks all use them (the frontend's fixed 07:00–22:00 is
      gone).
    - Default task duration, default event length, buffer between items.
    - Unfinished flexible tasks: Re-slot or Return to Inbox at rollover.
    - Launch at login (moved here from Notifications) and Close to tray:
      off, closing the main window quits.

    Core: get_settings / update_settings gain planning_hours (checked:
    each day's window must end after it starts), week_start, clock,
    close_to_tray, and make default durations, slot_buffer_min and
    rollover_mode settable. Like other settings they're change-logged,
    so Ctrl+Z undoes them, and agents can read them.

    The frontend keeps a copy of week start, clock and planning hours
    (reloaded after a settings change, undo, restore or import) that the
    date helpers read; the calendar views redraw when they change.
Files:
    A  src/lib/prefs.ts (+ test)
    A  src/views/settings/GeneralPage.tsx, src/views/settings/SettingRow.tsx
    M  crates/blotter-core/src/settings.rs          (week_start, clock, close_to_tray keys)
    M  crates/blotter-core/src/ops/settings.rs      (new fields, validation, tests)
    M  crates/blotter-core/src/slots.rs             (Span/PlanningHours schemas)
    M  src-tauri/src/shell.rs                       (close to tray)
    M  src/lib/format.ts (clockTime, hourLabel), src/lib/period.ts (weekStartsOn),
       src/lib/availability.ts (planningMinutes)
    M  src/app/AppShell.tsx, ChatApp.tsx, CaptureApp.tsx, nav.ts
    M  display times: src/app/AutoSlotPanel.tsx, ItemDialog.tsx, reminders.ts,
       ReminderToasts.tsx, RolloverBanner.tsx, src/chat/proposals.ts, src/lib/backupFormat.ts,
       src/lib/icsFormat.ts, src/views/WeekView.tsx, MonthView.tsx, YearView.tsx,
       diary/TimelineRail.tsx, src/components/ui/date-picker.tsx
    M  src/views/SettingsView.tsx, src/views/settings/NotificationsPage.tsx
    M  .claude/COMMITS.md, .claude/roadmap.md
Test steps:
    - cargo test --workspace → 170 core (+1: new settings round trip, Saturday off empties
      find_free_slots, backwards hours / unknown week start / big buffer refused) + 43 app
      + 6 mcp; clippy / fmt clean
    - pnpm check → clean; Vitest 209 (+4: 12/24-hour times and hour marks, Sunday weeks,
      per-weekday planning, shared span)
Notes:
    - Not driven live by me. Please try: Sunday weeks in Week/Month/Year; 12-hour clock on
      cards, the Week grid and a reminder toast; Saturday off then auto-slot; Close to tray
      off then close the window (it should quit).
    - Timezone is shown, not chosen: a setting that differs from the computer's would make
      the views (drawn in the computer's zone) disagree with the core. The core still honours
      a timezone setting if one exists.
    - Next: #38a first-run setup.

### #38a — First-run setup            [status: committed 95b63fe]
Phase: 10 (roadmap §5.10)
Message:
    feat(ui): first-run setup

    A short welcome on a fresh install, five steps, each skippable:

    1. Theme: the four presets, applied as you pick.
    2. Your week: week start and one planning-hours window.
    3. Work hours: a Work block-out (Mon–Fri 08:30–17:00 to start with),
       or none.
    4. AI: says whether Claude Code is installed and signed in; if not,
       offers to open Settings → AI providers at the end.
    5. Start with your computer: Launch at login.

    Keyboard throughout: Tab moves, Enter goes on, Esc (or Skip setup)
    ends it. Next applies a step, Skip moves on without it, Back
    returns. It shows once: the first time Blotter opens on a database
    with no items and no block-outs, so existing databases are marked
    done without seeing it. "Run setup again" in the palette brings it
    back. The done flag (setup_done) is kept per device, outside the
    JSON export.
Files:
    A  src-tauri/src/first_run.rs                 (first_run_pending, finish_first_run; test)
    A  src/app/FirstRun.tsx
    M  crates/blotter-core/src/settings.rs        (setup_done, a device key)
    M  src-tauri/src/lib.rs
    M  src/app/AppShell.tsx, src/app/overlay.ts, src/actions/actions.ts
    M  .claude/COMMITS.md, .claude/roadmap.md
Test steps:
    - cargo test --workspace → 170 core + 44 app (+1: shown on a fresh database only, and
      once) + 6 mcp; clippy / fmt clean
    - pnpm check → clean; Vitest 209
Notes:
    - Not driven live by me. To see it: BLOTTER_DATA_DIR pointing at an empty folder, then
      pnpm tauri dev; or "Run setup again" from the palette on your own database (its
      steps really apply: Next on Work hours adds a block-out).
    - Phase 10 is complete in code. AC checks for Luke: rebinding works at once and
      persists; conflicts are caught; theme edits stay in the preview until Apply; contrast
      warnings appear (e.g. muted text darkened).
    - Next: Phase 11, #39 completion history and streaks.

### #39 — Completion history and streaks            [status: committed 4703da8]
Phase: 11 (roadmap §5.3; design: History View)
Message:
    feat(ui): completion history and streaks

    The History view (g h) replaces its placeholder, as in the design:

    - Stats: current streak, this week, this month, best streak, and a
      bar for each of the last 14 days (today in the accent).
    - Completed items grouped by the day they were done (Today,
      Yesterday, a weekday, then dates), each with its first tag, who
      completed it when it wasn't you (ai or mcp), and the time.
      Clicking a row opens the item.
    - Search, a date range (last 7, 30 or 90 days, or the year), and
      tag chips to hide tags.

    Core: a read operation, list_completions, that agents can use too.
    Who completed an item comes from the change log (the latest change
    that set it done and wasn't undone), so completions from the UI,
    the AI and MCP all show, and an undone one drops out. Days are the
    user's local days. A streak is consecutive days with a completion
    up to today (today without one yet doesn't break it); "this week"
    follows the week-start setting. Stats count everything, whatever
    the filters.

    Also: the Year view's Completed heatmap now counts every completion
    in the year, Inbox tasks included (it only saw scheduled items).
Files:
    A  crates/blotter-core/src/ops/history.rs       (list_completions, streaks; tests)
    A  src/lib/history.ts (+ test)
    A  src/views/HistoryView.tsx
    D  src/views/PendingView.tsx                    (History was the last placeholder)
    M  crates/blotter-core/src/ops/mod.rs
    M  crates/blotter-core/src/store/items.rs       (list_completed with the completer, completion_times)
    M  src/app/AppShell.tsx, src/views/YearView.tsx
    M  .claude/COMMITS.md, .claude/roadmap.md
Test steps:
    - cargo test --workspace → 172 core (+2: streak rules; completions by user, AI and MCP
      with local days, an undone one gone, tag/query filters, stats unfiltered, bad range)
      + 44 app + 6 mcp; clippy / fmt clean
    - pnpm check → clean; Vitest 212 (+3: day groups and labels, search and hidden tags,
      tags by use and streak wording)
Notes:
    - Not driven live by me. Phase 11 AC: complete something yourself, have the chat
      complete one, and complete one from Claude Code over MCP; all three should appear in
      History at the right local time, the last two badged ai and mcp.
    - Phase 11 is complete in code. Next: Phase 12 (MCP acceptance checks and README notes).

### #36b — Keybindings page to its design            [status: committed 97bed23]
Phase: 10 (fix; design: Settings Keybindings, which #36 was built without)
Message:
    fix(ui): Keybindings page to its design

    #36 was built without the Settings Keybindings design; this brings
    the page in line with it:

    - Rows are flat lists under each group heading, not boxed. The whole
      row is the target: click it, or Tab to it and press Enter.
    - Each row shows where it works (Global, Calendar views, Week,
      Inbox, System), an accent dot when changed, and a reset button
      only when there's something to reset.
    - Capturing shows a dashed accent box, "Press new keys… Esc to
      cancel", then the keys as you press them.
    - A clash shows under its row: "n is already bound to Quick add."
      with Swap and Cancel. A chord clash ("g" against "g w"), which
      can't be swapped, offers Take it instead.
    - The search box counts the actions it shows.
    - The Quick capture notice appears only when the shortcut can't
      work (Wayland, or registration failed), as a warning.
Files:
    M  src/views/settings/KeybindingsPage.tsx, src/views/settings/QuickCaptureInfo.tsx
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 212 (the override and clash rules are unchanged)
Notes:
    - Kept from #36, beyond the design: the remove-keys button on hover, Take it for chord
      clashes, and the registry's own groups (Go to, View, …) so the page matches the
      palette and ? sheet.
    - Next: #37a Theme page.

### #37a — Theme page to its design            [status: committed 7a1f99f]
Phase: 10 (fix; design: Settings Theme, which #37 was built without)
Message:
    fix(ui): Theme page to its design

    #37 was built without the Settings Theme design; the page is now the
    design's single editor screen:

    - A header bar: Theme, a "draft · name (edited)" pill (the name is
      editable there), "N below WCAG AA" when any, then Import,
      Duplicate, Export, Delete (your own themes), Discard and Apply
      (Ctrl+Enter; it reads Applied once done).
    - A 340px editor beside the live preview ("only this panel uses the
      draft until you apply"). "Start from preset" chips (a strip of
      each theme's background and accent), with your own themes below
      them, load a theme into the draft.
    - Colour rows: swatch, token name, hex field; a pair below AA puts
      "3.92:1 on surface — AA needs 4.5:1" under its text colour and
      outlines the field.
    - Shape & type: a Radius slider (0–12px, setting all three corner
      sizes), Headings Sans/Serif, Density Compact/Comfortable, plus
      Text size and Font from #37.

    Apply on an edited preset saves it as a theme of your own (named
    "Dark copy" unless renamed); on your own theme it saves over it.
    The separate theme list screen is gone.
Files:
    M  src/views/settings/ThemePage.tsx
    M  src/theme/engine.ts (+ test)                (radius slider; the design's heading and density choices)
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 213 (+1 radius slider)
Notes:
    - Kept beyond the design: your own themes as chips, Import, Delete, Text size, Font.
      Density's Spacious (34px) from #37 is gone from the choices; a theme that has it
      still works (neither choice shows selected).
    - Next: #38b first-run setup.

### #38b — First-run setup to its design            [status: committed b1efb94]
Phase: 10 (fix; design: First Run, which #38a was built without; mixed with the Claude Code route)
Message:
    fix(ui): first-run setup to its design

    #38a was built without the First Run design; the setup now follows
    it, mixed with Blotter's Claude Code route where that fits:

    - A header with the mark, "Set up Blotter", step dots and "n / 5";
      each step has a large title and a line under it (Pick a theme,
      Your week, Block out your work hours, Connect an AI, Keep
      reminders running).
    - Theme cards show a small sample of each theme.
    - Your week: Monday/Sunday/Saturday, and planning hours with a bar
      showing the window across the day.
    - Work hours: a hatched panel with Label, Days and Hours, and the
      note that fixed appointments can still go there.
    - Connect an AI: provider cards, Anthropic, OpenAI and
      OpenAI-compatible, plus Claude Code first when it's installed
      (chosen for you when it's signed in). For an API provider, paste
      a key (or an endpoint URL) and Test: it saves the provider, picks
      a model if the endpoint lists them, and reports the round trip.
      Continue saves a typed key even without a test. Keys go to the
      keychain; AI changes start in Propose mode.
    - Keep reminders running: Launch at login.
    - Back, Skip (Esc) and Continue (Enter). Esc now skips the step, as
      the design says, rather than ending the setup; "Set up later"
      ends it.
Files:
    M  src/app/FirstRun.tsx
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 213
    - cargo test --workspace → 172 core + 44 app + 6 mcp (unchanged)
Notes:
    - Not driven live by me. "Run setup again" from the palette shows it; note its steps
      really apply (Continue on Work hours adds a block-out; Test saves a provider).
    - Mixed in beyond the design: the Claude Code card, the "Set up later" link, and the
      model being picked for OpenAI / local endpoints.
    - Next: Phase 12 (MCP acceptance checks and README notes).

### #39a — Luke's accents for Dark, Light and Paper            [status: committed 1b32ffd]
Phase: pre-release polish (Luke's list, 2026-10-16)
Message:
    style(theme): Luke's accents for Dark, Light and Paper

    The built-in presets take the colours Luke tuned in his "… copy"
    themes:

    - Dark: text on accent #121317 → #ffffff, danger #f07178 → #c4323f.
    - Light: accent #5b4cd6 → #b7adff, accent text #4b3dc0 → #9086fd,
      focus ring #5b4cd6 → #a094ff.
    - Paper: accent #2b4a7d → #6e99de.

    The fallback tokens in styles.css follow Dark. The theme test now
    records which preset pairs fall under WCAG AA with these colours.
Files:
    M  src/theme/presets.ts, src/styles.css, src/theme/engine.test.ts
    M  .claude/COMMITS.md
Test steps:
    - pnpm vitest run src/theme → 15 passed
Notes:
    - Under AA (4.5:1) now: Dark text on accent 3.33, danger on background 3.43; Light text
      on accent 2.01, accent text on tint 2.60, on background 2.99; Paper text on accent
      2.72. The theme editor will show these warnings when editing the presets. Raised
      with Luke.
    - The "… copy" themes in the demo database are untouched (delete them if no longer
      needed).

### #39b — Polish from Luke's list            [status: committed 0a92ee0]
Phase: pre-release polish (Luke's list, 2026-10-16)
Message:
    style(ui): polish from Luke's list

    - Smallest window 1200×720 (was 1024×700), and the theme preview
      gallery drops to one column when its pane is narrow (a container
      query), with its control rows wrapping, so nothing overflows.
    - About: the wordmark as tall as the mark (50px beside 48px, the
      sidebar's ratio), the version on its own row below.
    - Diary: on wide windows the timeline takes the spare width after
      the last day instead of leaving it blank; the day columns still
      scroll when the window is narrow.
    - The key hint on filled buttons is a tint of the button's own fill,
      so the ↵ on a red Delete no longer shows the accent.
    - App, taskbar and tray icon: the tile is full-bleed and the mark is
      1.5× larger, from a new source, src-tauri/icons/app-icon.svg
      (public/blotter.svg stays the favicon).
Files:
    A  src-tauri/icons/app-icon.svg
    M  src-tauri/icons/32x32.png, 64x64.png, 128x128.png, 128x128@2x.png, icon.png,
       icon.ico, icon.icns
    M  src-tauri/tauri.conf.json
    M  src/views/settings/ThemePreview.tsx, src/views/SettingsView.tsx,
       src/views/diary/DiaryView.tsx, src/views/diary/TimelineRail.tsx,
       src/components/ui/kbd.tsx
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 213
    - Seen in the running app (DevTools screenshots on a copy of the demo database): the
      sidebar wordmark sits on the mark's baseline with the "B" level with the page top
      (left as Luke set it); the Diary's columns scroll with the timeline at its 236px
      minimum when the chat is open.
Notes:
    - The new icon shows in the taskbar after a rebuild (pnpm tauri dev / build).

### #39c — Settings on one page in wide windows            [status: committed 5243959]
Phase: pre-release polish (Luke's list, 2026-10-16)
Message:
    feat(ui): Settings on one page in wide windows

    In a window at least 1600px wide (maximised on most screens), every
    settings page but Theme shows on one scrolling page: the nav jumps
    to a section and highlights the one in view. Theme, being large,
    stays on its own; picking another page from it goes back to the
    long page at that section. Narrower windows show one page per
    click, as before. Links into Settings (Edit block-outs, Edit
    keybindings) scroll to their section.

    Also: the theme preview's pencilled proposal sits in its own column
    of the mini week instead of over the Review block.
Files:
    M  src/views/SettingsView.tsx, src/views/settings/ThemePreview.tsx
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 213
    - In the running app (DevTools, copy of the demo database): at 1728px wide the long page
      shows and clicking Backups scrolls there with Backups highlighted; at 1200px it's one
      page per click; at 1200px the theme preview is one 358px column with nothing
      overflowing.

### #39d — Centred content, sidebar key chip, keybinding row            [status: committed 86f2a48]
Phase: pre-release polish (Luke's list, 2026-10-16)
Message:
    style(ui): centred content, sidebar key chip, keybinding row

    - Settings content is centred in its column: one page per tab in a
      window, and on the long page in a wide window, where sections are
      also wider (up to 1120px).
    - Fixed: jumping to a section on the long page scrolled the app's
      own frame too (scrollIntoView), shifting the sidebar and title bar
      up; now only the settings column scrolls.
    - History and Inbox are centred too.
    - The sidebar's Settings shortcut (Ctrl ,) is a key chip like the
      search box's Ctrl K; chords like g d stay plain text.
    - Keybindings: a row's remove (×) button is last, at the far right,
      after the reset button.
Files:
    M  src/views/SettingsView.tsx, src/views/HistoryView.tsx, src/views/inbox/InboxView.tsx,
       src/app/Sidebar.tsx, src/views/settings/KeybindingsPage.tsx
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 213
    - In the running app (DevTools): Keybindings, History and Inbox centred; Ctrl , as a
      chip; after jumping between sections nothing outside the settings column is scrolled.

### #39e — One key chip per key in the sidebar            [status: committed 26d1f45]
Phase: pre-release polish (Luke's list, 2026-10-16)
Message:
    style(ui): one key chip per key in the sidebar

    Every sidebar shortcut is drawn as key chips, one per key, as in
    the palette: [g] [d] for the views, [Ctrl] [,] for Settings and
    [Ctrl] [K] in the search box. KeyChips gains a compact size for
    narrow places, so "Search or command" still fits on one line.
Files:
    M  src/app/Sidebar.tsx, src/components/ui/kbd.tsx
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 213
    - In the running app: every view row, Settings and the search box show separate chips;
      "Search or command" is one line, not truncated (122 of 122px).

### #39f — Smart slot            [status: committed daff565]
Phase: pre-release polish (Luke's list, 2026-10-16)
Message:
    feat(ai): smart slot

    A second auto-slot that uses the connected AI, for the context the
    deterministic one can't weigh. Auto-slot packs tasks one after
    another; people need time between things, travel to get places,
    and some tasks belong on a weekend.

    The AI is asked to: fit the kind of task to the time (personal,
    home and errands on weekends and evenings, work on working days,
    exercise mornings or early evenings), leave at least 15 minutes
    between items and more after long ones, spread tasks over days
    rather than filling one, allow travel time around anything that
    happens somewhere (a location or an errand) and group errands
    together, respect due dates and priority, and put focused work
    earlier in the day. It reads the calendar first, checks times with
    find_free_slots, places with schedule_item, and replies with each
    task, when, and why. Each task's tags, duration (or a guess),
    priority, due date, location and notes go in the request.

    It runs as a chat turn in a new "Smart slot · Fri 16 Oct" thread,
    which opens. Propose mode (the default) means nothing moves until
    the proposals are accepted; block-outs and planning hours are
    enforced by the core whatever the AI picks.

    Where: ✦ next to Auto-slot in the Diary's Inbox rail, Smart slot in
    the Inbox's bulk bar (for the selected tasks), and "Smart slot
    Inbox" in the palette.
Files:
    A  crates/blotter-core/src/smart_slot.rs       (prompt and title; test)
    A  src/app/smartSlot.ts
    M  crates/blotter-core/src/lib.rs
    M  src-tauri/src/ai/chat.rs (chat_smart_slot), src-tauri/src/lib.rs
    M  src/actions/actions.ts, src/views/diary/InboxRail.tsx, src/views/inbox/BulkBar.tsx
    M  .claude/COMMITS.md, .claude/roadmap.md     (the pre-Phase 12 backlog)
Test steps:
    - cargo test --workspace → 173 core (+1 the prompt) + 44 app + 6 mcp; clippy / fmt clean
    - pnpm check → clean; Vitest 213
Notes:
    - Not tried live yet (it needs an AI turn). On the backlog before Phase 12, with Luke's
      other items from 2026-10-16 (roadmap: "Before Phase 12").

### #39g — Blotter and Studio presets, logo and title bar colours            [status: committed 6c97806]
Phase: pre-release polish (Luke, 2026-10-16/17)
Message:
    feat(theme): Blotter and Studio presets, logo and title bar colours

    The presets are now Blotter Light, Blotter Dark (default), Studio
    Light and Studio Dark, replacing Dark, Light, Paper and Gruvbox.
    Blotter Light is Luke's warm cream theme with serif headings and
    larger text; Blotter Dark is its dark counterpart and passes every
    contrast check. The Studio pair are his orange Light and Dark. A
    theme saved under an old preset id maps to its successor.

    The logo's calendar, wordmark and wordmark shadow get their own
    theme colours (--logo-accent, --logo-text, --logo-shadow), apart
    from the accent, with a Logo panel in the live preview. The window
    title bar takes --titlebar and --titlebar-text: Windows 11 sets the
    caption, border and title colours through DWM; elsewhere the window
    gets the light/dark hint. Themes saved before these tokens take
    their own text, accent and background.

    The app icon (title bar, taskbar, tray, favicon) is redrawn in the
    new logo colours.

    The Theme page stacks the preview under the editor below 760px of
    page width, so the docked chat no longer squeezes it.
Files:
    A  src-tauri/src/titlebar.rs                    (set_titlebar; DWM on Windows; test)
    M  src-tauri/src/lib.rs
    M  src/theme/presets.ts, src/theme/engine.ts, src/theme/theme.ts
    M  src/theme/engine.test.ts, src/theme/theme.test.ts
    M  src/styles.css, src/components/BlotterMark.tsx
    M  src/views/settings/ThemePage.tsx, src/views/settings/ThemePreview.tsx
    M  public/blotter.svg, src-tauri/icons/app-icon.svg
    M  src-tauri/icons/32x32.png, 64x64.png, 128x128.png, 128x128@2x.png, icon.png, icon.ico, icon.icns
Test steps:
    - pnpm check → clean; Vitest 217
    - cargo test --workspace → clean (titlebar colorref test); clippy / fmt clean
Notes:
    - Title bar colours not yet seen live (the running app locked the exe). Windows 10 gets
      only light/dark.
    - Blotter Light and the Studio pair keep some orange pairs under AA (Luke: hues are fine).

### #39h — Calendar and tag filters, sliding panels, themed hover hints            [status: committed 876637a]
Phase: pre-release polish (Luke, 2026-10-16/17)
Message:
    feat(ui): calendar and tag filters, sliding panels, themed hover hints

    The sidebar gets the design's "Calendars & tags" section: every tag
    in use (new read op list_tags) and each subscribed calendar, with a
    checkbox in its colour that shows or hides it in the Diary, Week,
    Month and Year. An item hides only when all its tags are hidden;
    untagged items always show. Kept in local storage.

    The sidebar, the Diary's Inbox rail and the docked chat slide open
    and closed. Every title attribute shows as a themed tooltip instead
    of the browser's.

    Also from Luke's list: the Inbox count beside the sidebar's i chip
    is gone, the Diary's day columns get top padding so a focused
    card's ring isn't clipped, and the Inbox's brain-dump box grows a
    line at a time from two.
Files:
    A  src/app/filters.ts, src/app/filters.test.ts
    A  src/components/ui/slide-panel.tsx, src/components/ui/title-tips.tsx
    A  src/lib/useSlidePresence.ts
    M  crates/blotter-core/src/ops/reads.rs, crates/blotter-core/src/store/items.rs   (list_tags; test)
    M  src/app/AppShell.tsx, src/app/Sidebar.tsx, src/main.tsx
    M  src/lib/calendarEvents.ts
    M  src/views/diary/DiaryView.tsx, src/views/diary/DayColumn.tsx, src/views/diary/InboxRail.tsx
    M  src/views/WeekView.tsx, src/views/MonthView.tsx, src/views/YearView.tsx
    M  src/views/inbox/InboxView.tsx
    M  .claude/COMMITS.md, .claude/roadmap.md
Test steps:
    - cargo test --workspace → 174 core (+1 list_tags) + 45 app + 6 mcp; clippy / fmt clean
    - pnpm check → clean; Vitest 217
Notes:
    - Filters don't apply to the Inbox, History or Year's completed mode.

### #39i — Palette theme icons for the new presets            [status: committed 1d1bb76]
Phase: pre-release polish
Message:
    fix(ui): palette theme icons for the new presets
Files:
    M  src/actions/actions.ts
Test steps:
    - pnpm check → clean
Notes:
    - Found while reading the action registry for the README's keybinding table.

### Phase 12 — acceptance checks (no commit of its own; recorded with #45)
- Headless Claude Code 2.1.292 against the release build's `blotter mcp` on a scratch
  BLOTTER_DATA_DIR: list_agenda worked; Propose (default) → create_item "proposed, not
  applied", one `mcp` proposal, no item, no change; ai_mode = auto → the item was created,
  logged as `mcp`, after one before_ai_batch snapshot.
- Running-app refresh: verified live in #29a; unchanged since, not re-run (a second instance
  would hand off to Luke's open app). Claude Desktop not tried (the AC takes either client).

### #41 — Windows installers, Linux bundles and the release workflow            [status: committed 6d39450]
Phase: 13
Message:
    build: Windows installers, Linux bundles and the release workflow
Files:
    A  .github/workflows/release.yml
    A  packaging/linux/io.github.lasr0.Blotter.desktop
    A  packaging/linux/io.github.lasr0.Blotter.metainfo.xml
    M  src-tauri/tauri.conf.json       (publisher, licence, NSIS per-user, deb metainfo, AppImage, window background)
    M  .gitignore                      (Flatpak build leftovers)
Test steps:
    - pnpm tauri build → Blotter_x.y.z_x64-setup.exe (7.1 MB) and …_en-US.msi (11.4 MB)
    - msiexec /a (administrative extract, no install) → PFiles/Blotter/blotter.exe; its
      `mcp` over raw stdio: serverInfo blotter 1.0.0, 28 tools incl. list_tags
    - release.yml parses (js-yaml)
Notes:
    - Linux bundles aren't built locally (no WSL distro or Docker); the workflow builds them.
    - Not installed on a clean VM yet (AC); unsigned, so SmartScreen will warn.

### #42 — Flatpak manifest and sandbox support            [status: committed 686a4c2]
Phase: 13
Message:
    build: Flatpak manifest and sandbox support
Files:
    A  packaging/flatpak/io.github.lasr0.Blotter.yml
    A  src-tauri/src/flatpak.rs        (app id, self command, autostart entry; 2 tests)
    M  src-tauri/src/autostart.rs, src-tauri/src/ai/claude_code.rs, src-tauri/src/ai/chat.rs,
       src-tauri/src/external.rs, src-tauri/src/lib.rs
    M  src-tauri/Cargo.toml, Cargo.lock (tauri-plugin-dialog: xdg-portal instead of gtk3)
Test steps:
    - cargo test --workspace → 174 core + 47 app (+2 flatpak) + 6 mcp; clippy -D warnings / fmt clean
    - cargo check on Windows with the portal dialog backend → clean
Notes:
    - Unbuilt and untried: needs the release workflow (or a Linux box) and the smoke checklist
      (notifications, tray, keychain, autostart, --capture, ICS fetch).
    - Non-Flatpak Linux file pickers now need xdg-desktop-portal (README, AUR depends).

### #43 — AUR PKGBUILD            [status: committed cfc2e8f]
Phase: 13
Message:
    build: AUR PKGBUILD
Files:
    A  packaging/aur/PKGBUILD
    A  packaging/aur/.SRCINFO
Test steps:
    - none here (no Arch); written against the tag tarball layout
Notes:
    - sha256sums is SKIP until the tag exists: run updpkgsums, then makepkg --printsrcinfo.

### #44 — README, Hyprland notes, MCP setup and keybindings            [status: committed d84fe70]
Phase: 13
Message:
    docs: README, Hyprland notes, MCP setup and keybindings
Files:
    A  README.md
    A  LICENSE
Test steps:
    - pnpm format:check → clean; the keybinding table was read from buildActions()

### #45 — v1.0.0            [status: committed c263ecb — untagged]
Phase: 13
Message:
    chore(release): v1.0.0
Files:
    M  Cargo.toml, Cargo.lock, package.json, src-tauri/tauri.conf.json   (1.0.0)
    M  .claude/COMMITS.md, .claude/roadmap.md   (Phase 12 and 13 progress)
Test steps:
    - pnpm check → Vitest 217; cargo test --workspace → clean; pnpm tauri build → 1.0.0 installers
Notes:
    - Not tagged: Luke pushes, tags v1.0.0 (git tag -a v1.0.0 -m "Blotter 1.0.0" && git push
      origin v1.0.0) to run release.yml, then runs the clean-VM and Flatpak smoke checks
      before publishing the draft release.
    - Still open before publishing: Anthropic's terms on driving Claude Code (D29); the ICS
      checks (Phase 9, Monday); the API-key providers stay Experimental.

### #45a — No icon or title in the Windows title bar            [status: committed e957972]
Phase: 13 (Luke, before the Windows release)
Message:
    feat(app): no icon or title in the Windows title bar
Files:
    M  src-tauri/src/titlebar.rs     (SetWindowThemeAttribute: WTNCA_NODRAWCAPTION | WTNCA_NODRAWICON)
    M  .claude/COMMITS.md
Test steps:
    - cargo test -p blotter → 47; clippy -D warnings / fmt clean
    - pnpm tauri build → 1.0.0 installers
Notes:
    - The window keeps its title, so the taskbar, Alt+Tab and screen readers still say Blotter.
      Applied with the theme colours by set_titlebar, to the main window and the chat pop-out.

### #45b — Plain Windows title bar from the first frame            [status: committed d3fb700]
Phase: 13 (Luke: "the logo is still in the windows title bar")
Message:
    fix(app): plain Windows title bar from the first frame
Files:
    M  src-tauri/src/titlebar.rs     (plain_caption; SetWindowPos SWP_FRAMECHANGED after the attribute)
    M  src-tauri/src/lib.rs          (main window in setup)
    M  src-tauri/src/chat_window.rs  (chat pop-out when built)
    M  .claude/COMMITS.md
Test steps:
    - cargo test -p blotter → 47; clippy -D warnings / fmt clean
    - Live, on Luke's pnpm tauri dev after its restart: the main window's title bar captured
      untouched (PowerShell CopyFromScreen) shows only the theme colour, no icon or title
Notes:
    - Cause: #45a set the attribute when the page applied its theme, after the window had
      drawn its caption, and Windows doesn't repaint the caption for a theme attribute, so
      the icon stayed until something redrew the frame (showing it from the tray did).

### #46 — v1.1.0 plan            [status: committed 674340a]
Phase: 14a (Luke's v1.1.0 list, 2026-10-20)
Message:
    docs: v1.1.0 plan
Files:
    A  .claude/LogoRedesign1.png, .claude/LogoRedesign2.png   (Luke's logo concepts)
    M  .claude/roadmap.md     (§12 v1.1.0; §10 email invites, Codex moved into v1.1.0)
    M  .claude/COMMITS.md     (planned #46–#64)
Test steps:
    - none (docs)
Notes:
    - Logo study (both concepts traced to SVG, in each preset's logo colours):
      https://claude.ai/artifact/FJ37WR7jJ1hwoQntHT9vFE. Open questions in roadmap §12.

### #47 — Held arrows keep moving focus and items            [status: committed 4502a85]
Phase: 14a
Message:
    fix(ui): held arrows keep moving focus and items

    Holding an arrow (or h j k l, Shift+arrow, Alt+arrow) now repeats
    focus moves, item moves and resizes. The key handler dropped every
    auto-repeated keydown; actions now opt in with `repeats`, and only
    the movement ones do, so everything else still fires once per press
    and a held key never repeats mid-chord.

    Calls on the focused item run one after another, each on a fresh
    copy of the item, so held Shift+arrows build on the last move; with
    one already waiting, extra repeats are dropped instead of piling up.
Files:
    M  src/keymap/useKeymap.ts
    M  src/actions/actions.ts        (repeats on focus.*, item.move.*, item.resize.*)
    M  src/actions/itemActions.ts    (withFocusedItem queue)
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 217
Notes:
    - Each repeated move is its own change-log entry, so Ctrl+Z steps back one 15m / one day
      at a time.

### #48 — Resize handle wins over move            [status: committed 24eddd5]
Phase: 14a
Message:
    fix(ui): resize handle wins over move

    Dragging the bottom edge of a Week view item resizes it every time.
    On a focused item the focus bar was drawn over the 6px resize strip
    with no handler of its own, so a press there started a move. The bar
    now ignores the pointer, the strip is drawn last and is taller (10px,
    14px reaching below the edge when focused), and short items get a
    strip too, so 30-minute items can be stretched by mouse.
Files:
    M  src/views/WeekView.tsx
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 217
    - Live (pending, Luke's release build was running): drag the bottom edge of a focused
      and an unfocused item, and of a 30-minute item → each resizes
Notes:
    - Only the Week view has mouse resize; the Diary timeline doesn't.

### #49 — Shift+arrows in the Year view            [status: committed ab21b49]
Phase: 14a (Luke: "shift arrow left or right in year view skips over a month")
Message:
    fix(ui): Shift+arrows in the Year view

    Shift+Left/Right moves the focused day a month back or forward, and
    Shift+Up/Down a row of months (four), keeping the day of the month
    (or the month's last day when it is shorter) and crossing into the
    next or previous year. Shift+arrows did nothing in the Year view: they
    are the item-move keys elsewhere, and the Year view has no items to
    move.
Files:
    M  src/views/YearView.tsx
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 217
    - Live (pending): Year view, Shift+→ from 31 Jan → 28/29 Feb; Shift+↓ from Oct → Feb next year
Notes:
    - Read as a request (Shift+←/→ should jump a month); if Luke meant a bug where it skipped
      two months, this needs his repro: in code nothing handled Shift+arrows here.

### #50 — Brain-dump draft survives switching views            [status: committed 1ecc376]
Phase: 14a
Message:
    feat(ui): brain-dump draft survives switching views

    Text typed into the Inbox's brain-dump box, or the Diary rail's, is
    kept while you switch to another view and back, until it is added or
    cleared. The drafts live in a small store instead of each box's own
    state, which was lost when the view unmounted. Session only: they
    don't survive quitting the app.
Files:
    A  src/app/drafts.ts
    M  src/views/inbox/InboxView.tsx
    M  src/views/diary/InboxRail.tsx
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 217
    - Live (pending): type two lines in the Inbox box, g w, g i → both lines still there

### #51 — Parse preview in the brain-dump box            [status: committed 08ec23e]
Phase: 14a (Luke: "syntax displayed for events on the inbox dump as well")
Message:
    feat(ui): parse preview in the brain-dump box

    As you type in the Inbox's brain-dump box, each line shows what it
    will create, read with the quick-add grammar: title, kind (an event
    when it reads like one, e.g. "dentist tue 3pm" is a fixed event),
    when, tags, and where it lands (Inbox, Diary or Calendar). Six lines
    show, then "+N more". The hint row lists the date/time, @fixed and
    "every" syntax too, not just ~duration, #tag, !priority and due.

    The quick-add summary moves into quickParse as describeParse so both
    places describe a line the same way.
Files:
    A  src/components/ParseLines.tsx
    M  src/lib/quickParse.ts, src/lib/quickParse.test.ts   (describeParse; 2 tests)
    M  src/components/QuickAddPanel.tsx
    M  src/views/inbox/InboxView.tsx
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 219
    - Live (pending): type "dentist tue 3pm" and "laundry ~1h #home" → Fixed event … Calendar,
      Flexible task · 1h · #home … Inbox
Notes:
    - The Diary rail's one-line box doesn't get the preview (too narrow); quick add (n) has it.

### #52 — Week view add indicator clears after Enter            [status: committed 9c1078e]
Phase: 14a
Message:
    fix(ui): Week view add indicator clears after Enter

    Adding an item at the Week view's grid cursor (click or Tab, then
    Enter, then save) left the "↵ to add" cursor drawn over the new item,
    so it wasn't clear it had been made. The item dialog now gives a newly
    created item keyboard focus, in every view, and the Week view leaves
    cursor mode whenever an item has focus, so the new item shows focused
    in place of the cursor.
Files:
    M  src/app/ItemDialog.tsx     (focus the created item)
    M  src/views/WeekView.tsx     (cursor mode derived from focus)
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 219
    - Live (pending): Week view, click an empty slot, Enter, type a title, Enter → the new
      item shows with the focus ring, no cursor over it; Tab brings the cursor back

### #53 — Blotter mark on the setup screens            [status: committed 5b36d32]
Phase: 14a
Message:
    fix(ui): Blotter mark on the setup screens

    First-run setup's header showed the design's placeholder "B" tile; it
    now shows the Blotter mark in the theme's logo colours, as the sidebar
    and About do. It follows the new logo when that lands.
Files:
    M  src/app/FirstRun.tsx
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 219
    - Live (pending): palette → "Run setup again" → the header shows the mark

### #54 — Tag management in Settings            [status: committed ab38e15]
Phase: 14b (Luke: "I can't see where to see tags or edit and delete them")
Message:
    feat: tag management in Settings

    A Tags page in Settings lists every tag with its colour and how many
    items carry it. Tags can be created ahead of use, renamed (renaming
    onto an existing tag merges the two), recoloured from the theme's tag
    colours or a custom hex, set back to automatic, and deleted (taken off
    every item). Rename and delete rewrite each item and log it, so one
    Ctrl+Z puts everything back.

    Core: new ops set_tag, rename_tag and delete_tag over the existing
    tags table, which becomes an undoable "tag" entity; list_tags returns
    each tag's colour and includes tags not on any item yet. JSON export
    carries the tag records (1.0 exports still import).

    UI: a chosen colour is published as a CSS variable per tag, which
    tagColor() reads before the automatic colour, so a change shows in
    every view and window at once without refetching.
Files:
    A  crates/blotter-core/src/store/tags.rs
    A  crates/blotter-core/src/ops/tags.rs        (set_tag, rename_tag, delete_tag; 2 tests)
    M  crates/blotter-core/src/store/mod.rs, crates/blotter-core/src/ops/mod.rs
    M  crates/blotter-core/src/ops/reads.rs       (list_tags: colour, unused tags)
    M  crates/blotter-core/src/ops/undo.rs        ("tag" entity)
    M  crates/blotter-core/src/export.rs          (tags in JSON export, defaulted)
    A  src/app/TagColors.tsx
    A  src/views/settings/TagsPage.tsx
    M  src/lib/tags.ts, src/lib/tags.test.ts
    M  src/main.tsx, src/app/nav.ts, src/app/Sidebar.tsx, src/views/SettingsView.tsx
    M  .claude/COMMITS.md
Test steps:
    - cargo test --workspace → 176 core (+2 tags) + 47 app + 6 mcp; clippy -D warnings / fmt clean
    - pnpm check → clean; Vitest 222
    - Live (pending): Settings → Tags: add #garden, give it a colour, rename #shops onto
      #shopping (merges), delete a tag, Ctrl+Z each; colours change in Diary/Week at once
Notes:
    - The MCP server now lists 31 tools (set_tag, rename_tag, delete_tag are agent-callable and
      go through Propose like every mutation).
    - The sidebar's "Calendars & tags" + still opens Calendars; Tags is next to it in Settings.

### #55 — Themed context menu            [status: committed a836678]
Phase: 14b (Luke: "right click menu themable?")
Message:
    feat(ui): themed context menu

    Right-click opens Blotter's own menu, drawn in the theme, instead of
    the browser's Back / Reload / Inspect menu.

    - On an item (Diary card, Week and timeline blocks, Month chips and
      day popover, Inbox rows): Open, Mark done / not done, Move to Inbox
      (scheduled flexible tasks), Delete (with the usual confirmation).
    - On a day (Diary column, Week or timeline grid, Month cell): Add item
      on this day, or at the half hour under the pointer on a time grid,
      and Lift block-outs on this day.

    Text fields keep the browser menu for copy, paste and spelling, and
    Shift+right-click still opens it everywhere, for the dev tools.
Files:
    A  src/app/ContextMenu.tsx
    M  src/app/AppShell.tsx
    M  src/views/MonthView.tsx            (data-menu-item on chips, data-date on cells)
    M  src/views/diary/TimelineRail.tsx   (data-menu-item)
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 222
    - Live (pending): right-click a Week block, a Diary card, a Month chip, an empty Week
      slot (shows its time), a Month cell, and the brain-dump box (browser menu); each theme
Notes:
    - Main window only; the chat pop-out and quick capture keep the browser menu.

### #56 — Drag items in the Month view and all-day lane            [status: committed 5b957f5]
Phase: 14c (Luke: "make tasks, events and reminders dragable on all tabs month view, week view")
Message:
    feat(ui): drag items in the Month view and all-day lane

    Tasks, events and reminders can be dragged between days in the Month
    view, and along the Week view's all-day lane. A timed item keeps its
    time of day on the new day; an untimed one keeps its place in the day.
    The day under the pointer is outlined (Month) or tinted (all-day lane)
    while dragging, and Esc cancels as before. All-day bars also open on
    double-click and get the right-click menu.

    Drop hit-testing now looks through elements stacked over a drop zone
    without being inside it, so dropping onto an all-day bar lands on its
    day. Zones with no cards leave an item's sort order alone.
Files:
    M  src/app/drag.ts            (elementsFromPoint; no sort order without cards)
    M  src/views/MonthView.tsx    (draggable chips, day drop zones)
    M  src/views/WeekView.tsx     (all-day lane zones and draggable bars)
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 222
    - Live (pending): Month: drag a timed event and an untimed task to another day (time
      kept); Week: drag an all-day item along the lane and onto a cell with another bar
Notes:
    - Items inside the Month "+N" popover are buttons and don't drag (open the day in Week).
    - Already draggable before this: Week timed blocks, Diary cards, timeline, Inbox rail.

### #63 — New Blotter logo            [status: committed 690777d]
Phase: 14f (Luke, 2026-10-20: "Logo A … make the swap"; done ahead of 14d)
Message:
    feat(ui): new Blotter logo

    The mark is Luke's concept A (.claude/LogoRedesign1.png), traced as a
    vector on the 1024 grid: a calendar page whose right edge is a "B",
    two binder rings, a window cut through the page holding a four-point
    spark, and a diagonal sweep across its foot.

    - BlotterMark draws it from the theme: the page in --logo-accent, the
      sweep in a darker shade of it (--logo-fold overrides), the spark and
      rings in --logo-page, with --logo-outline edging them on light
      themes. The window is cut out, so the sidebar shows through.
    - public/blotter.svg (favicon) and src-tauri/icons/app-icon.svg are
      the same geometry in the Blotter Dark logo colours; the desktop
      icons are regenerated from the latter.
    - The wordmark's extrusion is twenty layers instead of four, so it
      reads as one solid block with no gaps between the copies at the
      sizes it is shown (sidebar, About, setup, theme preview).
Files:
    M  src/components/BlotterMark.tsx
    M  public/blotter.svg, src-tauri/icons/app-icon.svg
    M  src-tauri/icons/32x32.png, 64x64.png, 128x128.png, 128x128@2x.png, icon.png, icon.ico, icon.icns
    M  src/styles.css              (wordmark-shadow: 20 layers)
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 222
    - Rendered app-icon.svg through pnpm tauri icon and looked at the 256px PNG
    - Live (pending): sidebar, About, setup and the Theme preview's Logo panel in all four
      presets; the wordmark extrusion at 30px (About) has no steps
Notes:
    - --logo-cell and --logo-done no longer have anything to colour (the old grid's cells).
    - Colours are left to Luke in the theme engine; the presets still share one logo palette.

### #57 — Task context in quick add and Smart slot            [status: committed 99707a6]
Phase: 14d (Luke: "// is fine … notes is the right call")
Message:
    feat: task context in quick add and Smart slot

    Anything after "//" on a quick-add or brain-dump line is the task's
    context, stored in its notes: "groceries ~1h #home // need the car,
    shops shut 9pm". It is read last and left as written, so #words and
    dates inside it aren't parsed; a "//" inside a word (a link) doesn't
    count. The preview shows it as a Context chip and in the line's
    summary.

    Smart slot hands each task's notes to the AI as its context (up to
    300 characters, was 160) and tells it to follow the user's context
    over its general placing rules. The item dialog's notes field says it
    doubles as Smart slot context.
Files:
    M  src/lib/quickParse.ts, src/lib/quickParse.test.ts   (2 tests)
    M  src/components/QuickAddPanel.tsx, src/views/inbox/InboxView.tsx, src/app/ItemDialog.tsx
    M  crates/blotter-core/src/smart_slot.rs
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 224
    - cargo test -p blotter-core smart_slot → ok

### #58 — Holidays            [status: committed cfc4052]
Phase: 14d (Luke: holidays are a type of block-out that lift the block-outs they collide
with, expire on their own, allow everything but work, and prioritise #holiday tasks)
Message:
    feat(core): holidays

    A holiday is a date range away from the usual week, made in
    Settings → Block-outs → Holidays (or by the AI, through Propose).

    - Making one lifts every block-out that falls on its days: an
      exception per block-out and date, noted "Holiday: <name>" and
      carrying the holiday's id, shown with the block-out's other
      exceptions. Dates the user had already lifted are left alone.
      Changing the dates moves the lifts; deleting it puts the
      block-outs back. Each is one undoable batch.
    - Slotting: work-tagged tasks are kept off holiday days (the holiday
      joins their block-outs), every other task may go there, and tasks
      tagged #holiday are placed first so they get first pick. Smart
      slot reads list_holidays and is told the same rules.
    - Once a holiday's last day has passed it is removed by the app's
      minute check (prune_holidays, as the system), so it drops out of
      Settings on its own.
    - The Diary, Week and Month day headers show a holiday tag.

    Core: migration 0007 (holidays table; holiday_id on block-out
    exceptions), store and ops (list_holidays, create_holiday,
    update_holiday, delete_holiday, prune_holidays), "holiday" in undo,
    holidays in the JSON export.
Files:
    A  crates/blotter-core/migrations/0007_holidays.sql
    A  crates/blotter-core/src/model/holiday.rs
    A  crates/blotter-core/src/store/holidays.rs
    A  crates/blotter-core/src/ops/holidays.rs     (4 tests)
    M  crates/blotter-core/src/db.rs, model/mod.rs, model/blockout.rs, store/mod.rs,
       store/blockouts.rs, ops/mod.rs, ops/blockouts.rs, ops/undo.rs, ops/slots.rs,
       export.rs, smart_slot.rs, windows.rs
    M  src-tauri/src/rollover.rs                   (prune each minute check)
    A  src/views/settings/HolidaysSection.tsx, src/components/HolidayTag.tsx, src/lib/holidays.ts
    M  src/views/settings/BlockoutsPage.tsx, src/lib/types.ts
    M  src/views/WeekView.tsx, src/views/MonthView.tsx, src/views/diary/DayColumn.tsx
    M  .claude/COMMITS.md
Test steps:
    - cargo test --workspace → 180 core (+4 holidays) + 47 app + 6 mcp; clippy -D warnings / fmt
    - pnpm check → clean; Vitest 224
    - Live (pending): add a holiday over a work week → Work shows those dates lifted,
      day headers show the tag; Auto-slot leaves a #work task out of it and puts a
      #holiday task in; delete → lifts gone; Ctrl+Z → back
Notes:
    - "Work" means the #work tag. A block-out made after a holiday isn't lifted on its days
      (edit the holiday to re-lift).
    - Expiry is checked every minute while the app runs, and at startup.

### #63a — No grey edge on the logo's spark and rings            [status: committed 0e139df]
Phase: 14f (Luke: "remove the gray outlines off the star and the calendar binder")
Message:
    fix(ui): no grey edge on the logo's spark and rings

    The light presets' --logo-outline drew a grey edge round the spark and
    the binder rings. The mark no longer strokes them; they are flat
    --logo-page like the rest of the drawing.
Files:
    M  src/components/BlotterMark.tsx
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean
Notes:
    - --logo-outline is still set by the presets but nothing reads it now. The app icon
      and favicon never had the edge.

### #63b — Logo closer to its design, gradient, no taskbar tile            [status: committed 7359512]
Phase: 14f (Luke: "the middle square needs to be bigger, the star should cutout part of the B
… add back the gradient but use the logo page and logo accent … on the task bar logo lets
remove the black square background")
Message:
    fix(ui): logo closer to its design, gradient, no taskbar tile

    Closer to LogoRedesign1: the window is larger (an even border all
    round), its foot is the sweep's edge, and the spark is cut the other
    way from the window, filled inside it and a notch where its longer
    tail crosses into the B.

    The page is a gradient from --logo-accent (top left) to --logo-page
    (bottom right), with the sweep a light wash of --logo-page, so both
    stay themable. The app, taskbar and tray icon drop the dark tile: the
    mark alone, scaled to fill the square.
Files:
    M  src/components/BlotterMark.tsx
    M  public/blotter.svg (keeps its tile: favicon), src-tauri/icons/app-icon.svg (no tile)
    M  src-tauri/icons/32x32.png, 64x64.png, 128x128.png, 128x128@2x.png, icon.png, icon.ico, icon.icns
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean
    - Rendered both SVGs through pnpm tauri icon and looked at them
Notes:
    - The icon files carry the Blotter Dark colours (#fe5858 → #fef6e1); the in-app mark
      follows the theme's logo tokens.

### #59 — Task windows with replanning            [status: committed 0579005]
Phase: 14d (Luke: the window by which a task like planning a trip must be done; overdue
and back to the Inbox once it closes)
Message:
    feat(core): task windows with replanning

    A task can carry a do-window: a not-before date, with its due date as
    the window's end ("book the holiday some time between 1 and 15
    Nov"), and optionally a usual time of day to do it.

    - Slotting (auto-slot, rollover, Smart slot's guidance) never places
      it before the window opens or after it is due, and tries the usual
      time of day first before any other free time.
    - If its slot passes undone, the daily rollover re-plans it inside the
      window whatever the rollover setting; once the window has closed
      (or nothing fits) it goes back to the Inbox, overdue.
    - Item dialog: a "Do window" row (from date, until it's due, usually
      HH:MM–HH:MM). Quick add: "after 1 nov", "from 1 nov to 15 nov" or
      "between 1 nov and 15 nov" (the range's end is the due date).

    Core: migration 0008 (not_before, window_start, window_end on items),
    create_item / update_item arguments and validation, Smart slot sees
    "not before …" and "usually …".
Files:
    A  crates/blotter-core/migrations/0008_item_windows.sql
    M  crates/blotter-core/src/db.rs, time.rs, model/item.rs, store/mod.rs, store/items.rs,
       ops/items.rs, ops/slots.rs, ops/rollover.rs (2 tests), smart_slot.rs
    M  src/lib/types.ts, src/lib/itemDraft.ts, src/lib/itemDraft.test.ts (2 tests)
    M  src/lib/quickParse.ts, src/lib/quickParse.test.ts (1 test), src/components/QuickAddPanel.tsx
    M  src/app/ItemDialog.tsx
    M  .claude/COMMITS.md
Test steps:
    - cargo test --workspace → 182 core + 47 app + 6 mcp; clippy -D warnings / fmt clean
    - pnpm check → clean; Vitest 227
    - Live (pending): "book trip after 1 nov due 15 nov" → Inbox with a window; Auto-slot puts
      it on 1 Nov or later; the item dialog shows the Window row
Notes:
    - The usual time is a preference, not a rule: with no room in it, any free time is used.
    - Weekly tasks (#60) reuse the usual time of day.

### #63c — Logo keeps a shaded underside below the gradient            [status: committed 521ee95]
Phase: 14f (Luke: "keep the shaded under side of the B and keeping the rest above the under side
gradient … make the logo on the task bar match the logo for the blotter light theme")
Message:
    fix(ui): logo keeps a shaded underside below the gradient

    The sweep across the foot of the B is a solid shade again (a darker
    mix of --logo-accent; --logo-fold overrides it), and the gradient
    from --logo-accent to --logo-page covers only the page above it,
    reaching the page colour at the shade line. The app, taskbar and tray
    icon use the Blotter Light logo colours.
Files:
    M  src/components/BlotterMark.tsx
    M  public/blotter.svg, src-tauri/icons/app-icon.svg
    M  src-tauri/icons/32x32.png, 64x64.png, 128x128.png, 128x128@2x.png, icon.png, icon.ico, icon.icns
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean
    - Rendered app-icon.svg through pnpm tauri icon and looked at it
Notes:
    - No custom themes are saved in Luke's database, so "Blotter Light" is the preset:
      #fe5858 → #fef6e1, shade #b73f3f (the same logo tokens as Blotter Dark today). If the
      preset's logo tokens change, edit app-icon.svg and run pnpm tauri icon again.

### #63d — A colour for the logo's shaded underside            [status: committed 8aa6f9a]
Phase: 14f (Luke: "make a colour value for the shaded under part of B")
Message:
    feat(theme): a colour for the logo's shaded underside

    New theme token --logo-shade colours the shaded underside of the B.
    It sits in the Theme page's Logo group beside logo-accent and
    logo-page; the presets set it to #b73f3f. A theme saved before it
    existed gets a shade worked out from its own logo accent (72% of it,
    mixed with black), so it keeps the look it had.
Files:
    M  src/theme/presets.ts, src/theme/engine.ts, src/theme/engine.test.ts
    M  src/components/BlotterMark.tsx
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 227 (engine test extended)

### #60 — Weekly tasks            [status: committed b2d6ad2]
Phase: 14d (Luke: groceries, meal prep, laundry: a home day and usual time; missed → the next
suitable day and time)
Message:
    feat: weekly tasks

    A weekly task is a repeating flexible task with a home day and a
    usual time: "laundry weekly sat 9-12 ~1h" is every Saturday at 09:00,
    usually 09:00–12:00, an hour long.

    When an occurrence's day passes undone, the daily rollover moves that
    occurrence (as an override) to the next free slot before the next
    occurrence starts, trying its usual time of day first, so it still
    gets done that week. One that can't fit stays on its day; a done one,
    or one whose next occurrence has already come, is left alone. This
    applies to every repeating flexible task, whatever the rollover
    setting. Occurrences no longer fall into the ordinary rollover (which
    would have sent them to the Inbox).

    Quick add: "weekly sat" means "every sat" and "weekly" alone "every
    week"; "9-12", "6pm-8pm" or "1-3pm" sets the usual time (and the start
    time when none is given). A repeating line now starts on the first
    day its rule names ("bins every monday" typed on a Wednesday starts
    next Monday), not today.
Files:
    M  crates/blotter-core/src/ops/rollover.rs   (replan_repeats; 2 tests)
    M  crates/blotter-core/src/ops/reads.rs      (day_range pub(crate))
    M  src/lib/quickParse.ts, src/lib/quickParse.test.ts   (2 tests; the "every monday" start moved)
    M  .claude/COMMITS.md
Test steps:
    - cargo test --workspace → 184 core + 47 app + 6 mcp; clippy -D warnings / fmt clean
    - pnpm check → clean; Vitest 229
    - Live (pending): "groceries weekly sat 9-12 ~1h", leave last Saturday's undone, run
      rollover from the palette → it moves to the next free 9–12 slot before Saturday
Notes:
    - Uses the usual-time fields from #59; the item dialog's Window row edits them for a series.
    - Looks back 7 days for missed occurrences.

### #62 — Consult me slot mode            [status: committed ad1e018]
Phase: 14e (Luke: "the AI will consult you on each task in your inbox, asking important
contextual questions about each task to find the best slot period for them")
Message:
    feat(ai): Consult me slot mode

    A third way to plan the Inbox, beside Auto-slot and Smart slot. The AI
    goes through the tasks one at a time in a new chat thread: it reads
    the calendar, asks two or three numbered questions that would change
    where the task goes (how long, by when, where and how far, what time
    suits, what it depends on, days it can't be), each with a guess you
    can just say yes to, and waits. With your answer it writes what it
    learned onto the task (duration, due, not before, usual time, and a
    summary in the notes as Smart slot context), places it, says where and
    why, and asks about the next. "skip" leaves a task in the Inbox; "just
    place the rest" stops the questions. Propose mode applies as usual.

    Where: "Consult me" in the Inbox's bulk bar (for the selected tasks),
    a button in the Diary's Inbox rail, and "Consult me on the Inbox" in
    the palette.
Files:
    M  crates/blotter-core/src/smart_slot.rs     (consult_prompt, consult_title; 1 test)
    M  src-tauri/src/ai/chat.rs, src-tauri/src/lib.rs   (chat_consult_slot)
    M  src/app/smartSlot.ts, src/actions/actions.ts, src/views/inbox/BulkBar.tsx,
       src/views/diary/InboxRail.tsx
    M  .claude/COMMITS.md
Test steps:
    - cargo test --workspace → 185 core + 47 app + 6 mcp; clippy -D warnings / fmt clean
    - pnpm check → clean; Vitest 229
    - Live (pending, needs an AI turn): Consult me on two Inbox tasks with Claude Code →
      questions for the first only; answer → notes/fields updated and a proposal to place it
Notes:
    - Built on Claude Code's kept-alive session (#31a), so the conversation carries over
      between replies; API-key providers carry it through the thread's history.

### #63e — The official logo colours in every preset and the icon            [status: committed 4ac476c]
Phase: 14f (Luke: "the logo colour way in the blotter light copy is the official logo colour
way i would like to use for all the default themes and for the task bar logo")
Message:
    feat(theme): the official logo colours in every preset and the icon

    Luke's "Blotter Light copy" colourway becomes the official one: logo
    accent #fe5858, page #ffbf52 (the gradient runs coral to gold), shade
    #f05656 under the B, wordmark #ffbf52 on a #fe5858 extrusion. All four
    presets use it, and the app, taskbar and tray icon and the favicon are
    redrawn in it.
Files:
    M  src/theme/presets.ts            (--logo-page #ffbf52, --logo-shade #f05656 in all four)
    M  public/blotter.svg, src-tauri/icons/app-icon.svg
    M  src-tauri/icons/32x32.png, 64x64.png, 128x128.png, 128x128@2x.png, icon.png, icon.ico, icon.icns
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 229
    - Rendered app-icon.svg through pnpm tauri icon and looked at it
Notes:
    - Read from the demo database's "Blotter Light copy" (copied, read-only). Saved user themes
      keep their own logo colours.

### #63f — Logo spark moved down instead of stretched            [status: committed 36ad210]
Phase: 14f (Luke: "fix the star in the logo so it isn't stretched down and instead it's moved
down and cuts out part of the B")
Message:
    fix(ui): logo spark moved down instead of stretched

    The spark is a symmetric four-point star again (its lower point had
    been stretched to reach the window's foot). It now sits low in the
    window, a little left of centre where the foot is lower, so only its
    lower point crosses the foot and cuts a notch into the B.
Files:
    M  src/components/BlotterMark.tsx, public/blotter.svg, src-tauri/icons/app-icon.svg
    M  src-tauri/icons/32x32.png, 64x64.png, 128x128.png, 128x128@2x.png, icon.png, icon.ico, icon.icns
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 229
    - Rendered app-icon.svg through pnpm tauri icon and looked at it

### #63g — Logo spark centred and larger, cutting into the B            [status: committed 6ec8b01]
Phase: 14f (Luke: "the spark is moved too far down … somewhat centred, make it larger to touch
the bottom of the B and there should be a cutout on the bottom")
Message:
    fix(ui): logo spark centred and larger, cutting into the B

    The spark sits centred in the window again and is larger and fuller,
    so its top stays just inside the window and its lower point crosses
    the window's foot, cutting a clear notch into the B's underside.
Files:
    M  src/components/BlotterMark.tsx, public/blotter.svg, src-tauri/icons/app-icon.svg
    M  src-tauri/icons/32x32.png, 64x64.png, 128x128.png, 128x128@2x.png, icon.png, icon.ico, icon.icns
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 229
    - Rendered app-icon.svg through pnpm tauri icon and looked at it

### #61 — Codex CLI sessions            [status: committed 26ecac0]
Phase: 14e
Message:
    feat(ai): Codex CLI sessions

    A third AI route beside Claude Code and the API-key providers: the
    user's installed, signed-in OpenAI Codex CLI, on their ChatGPT plan,
    driven headless with Blotter's MCP server as its only tools. Chat,
    Smart slot, Consult me and the morning briefing all run through it.

    - Each message is one `codex exec --json` run in its own folder under
      the app data; later messages resume the thread's Codex thread
      (`codex exec resume <id>`), so the conversation carries on. A Codex
      thread that has gone is replaced by a fresh one, once.
    - Locked down like the Claude Code route: Codex's shell, exec, apps,
      plugins, browser, computer use, sub-agents, image tools and web
      search are switched off, it runs read-only, and the user's Codex
      config and rules aren't loaded (only its sign-in). Blotter's
      instructions go in as developer instructions; the planner snapshot
      travels with each message.
    - Changes are logged and proposed as ai:codex/<model> ("default" when
      Codex picks the model).
    - Finding it: the path set in Settings, then PATH, then npm's global
      package (on Windows npm only puts a codex.cmd shim on PATH; Blotter
      runs the bundled codex.exe directly). Flatpak goes through
      flatpak-spawn like Claude Code.
    - Settings → AI providers has a Codex card: install and sign-in status
      (codex login status), Use for chat, model (blank for Codex's
      default) and program path. The chat header and proposal cards say
      "Codex".
Files:
    A  src-tauri/src/ai/codex.rs               (driver; 3 tests + 1 ignored live test)
    M  src-tauri/src/ai/chat.rs                (Route::Codex, settings, codex_status, run_codex)
    M  src-tauri/src/ai/claude_code.rs, src-tauri/src/ai/mod.rs, src-tauri/src/lib.rs
    M  crates/blotter-core/src/settings.rs     (codex_model, codex_path; per device)
    A  src/views/settings/CodexCard.tsx
    M  src/views/settings/AiProvidersPage.tsx, src/lib/aiSettings.ts, src/chat/ChatPanel.tsx,
       src/chat/proposals.ts, src/chat/proposals.test.ts
    M  .claude/COMMITS.md
Test steps:
    - cargo test --workspace → 185 core + 50 app (+3) + 6 mcp; clippy -D warnings / fmt clean
    - pnpm check → clean; Vitest 229
    - Live, Codex CLI 0.163.0 signed in with ChatGPT, on a scratch copy of the demo data:
        · by hand: list_inbox + create_item through MCP; resume remembered the thread;
          developer_instructions honoured on a new thread
        · BLOTTER_LIVE_CODEX=<scratch> cargo test -p blotter live_codex -- --ignored → Codex
          called list_inbox through Blotter's arguments and answered (8 s)
    - Pending (Luke's app was running): Settings → AI providers → Codex → Use for chat; send a
      message, a follow-up, and run Smart slot and Consult me
Notes:
    - Codex sends each message whole, so a reply appears a message at a time rather than as
      it's typed. Each turn starts a new process (a few seconds); no prewarm.
    - First-run setup still offers Claude Code or an API key; Codex is chosen in Settings.

### #64 — v1.1.0            [status: committed — hash in the next entry, untagged]
Phase: 14f
Message:
    chore(release): v1.1.0

    Version 1.1.0 in Cargo.toml (all three crates), package.json,
    tauri.conf.json, the AUR PKGBUILD and .SRCINFO, and the AppStream
    metainfo (with a 1.1.0 release note). The README covers Smart slot and
    Consult me, task windows, weekly tasks, holidays, the Codex route and
    the // context syntax.
Files:
    M  Cargo.toml, Cargo.lock, package.json, src-tauri/tauri.conf.json   (1.1.0)
    M  packaging/aur/PKGBUILD, packaging/aur/.SRCINFO, packaging/linux/io.github.lasr0.Blotter.metainfo.xml
    M  README.md
    M  .claude/COMMITS.md, .claude/roadmap.md   (§12 progress and decisions)
Test steps:
    - pnpm check → clean; Vitest 229
    - cargo test --workspace → 185 core + 50 app + 6 mcp; clippy -D warnings clean
    - pnpm tauri build → Blotter_1.1.0_x64-setup.exe (7.2 MB), Blotter_1.1.0_x64_en-US.msi (10.0 MB)
    - target/release/blotter.exe mcp over raw stdio → serverInfo blotter 1.1.0, 36 tools
      (set_tag, rename_tag, delete_tag, the holiday ops among them)
Notes:
    - Not tagged: Luke pushes, then tags v1.1.0 (git tag -a v1.1.0 -m "Blotter 1.1.0" &&
      git push origin v1.1.0) to run release.yml for the Linux bundles and the draft release.
    - AUR: sha256sums stays SKIP until the tag exists; then updpkgsums and
      makepkg --printsrcinfo > .SRCINFO.
    - Live checks still pending for most of 14a–14e (each entry lists its own); Codex was
      checked live (#61).

### #64a — AUR checksum for v1.1.0            [status: committed — hash in the next entry]
Phase: 14f (after Luke tagged and pushed v1.1.0)
Message:
    build: AUR checksum for v1.1.0

    The sha256 of GitHub's v1.1.0 source tarball, in the PKGBUILD and
    .SRCINFO in place of SKIP. Worked out on Windows (no updpkgsums):
    downloaded the tag tarball and hashed it; it unpacks to Blotter-1.1.0
    with version 1.1.0.
Files:
    M  packaging/aur/PKGBUILD, packaging/aur/.SRCINFO
    M  .claude/COMMITS.md
Test steps:
    - Invoke-WebRequest the tag tarball → 2,359,274 bytes, SHA-256 6da073c9…094e5c;
      tar lists Blotter-1.1.0/, its Cargo.toml says 1.1.0
Notes:
    - .SRCINFO edited by hand to match (only the checksum line changes); on Arch,
      makepkg --printsrcinfo gives the same.

### #65 — v1.2.0 plan            [status: committed 74ae4f7]
Phase: 15
Message:
    docs: v1.2.0 plan

    Roadmap §13: a new item button (15a), tags whose tasks can share
    time (15b), public holiday import with holidays no longer lifting
    Sleep-type block-outs (15c), and in-app updates (15d, D31). Reverse
    block-outs were dropped: planning hours already do that.
Files:
    M  .claude/roadmap.md, .claude/COMMITS.md
Test steps:
    - none (docs only)

### #66 — New item button in the Inbox rail and top bar            [status: committed 0c828ab]
Phase: 15a
Message:
    feat(ui): new item button in the Inbox rail and top bar

    A + button opens the full item dialog, with every kind and field:
    in the Diary's Inbox rail (open and collapsed) and in the top bar of
    the Week, Month and Year views. In a calendar view the new item starts
    on the focused day; from the Diary it has no date, so a task goes to
    the Inbox. The palette's New item does the same (openNewItem).
Files:
    M  src/actions/itemActions.ts, src/actions/actions.ts
    M  src/views/diary/InboxRail.tsx, src/app/TopBar.tsx
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 229
    - Pending (Luke's installed Blotter was running): Diary rail open and collapsed → + opens
      New item with no date; Week with the cursor on Thu → + opens it dated Thu; Month likewise
Notes:
    - Year view gets the top-bar button too (it has a focused day); Inbox and History don't.

### #67 — Tags whose tasks can share time            [status: committed cabc654]
Phase: 15b
Message:
    feat(core): tags whose tasks can share time

    A tag can be marked "Simultaneous" in Settings → Tags: its tasks can
    be done during other tasks (laundry while cleaning).

    - Such a task (a flexible task carrying the tag) is slotted over other
      tasks but never over events, reminders or fixed items, and stays out
      of block-outs as before. It doesn't make its own time busy for other
      tasks, and overlaps between it and flexible tasks aren't conflicts.
    - Auto-slot, rollover and find_free_slots all follow it;
      find_free_slots takes shares_time, defaulting to whether exclude_item
      carries such a tag. Smart slot and Consult me are told the rule.
    - set_tag keeps fields it isn't given (colour, shares_time), so turning
      the switch doesn't reset a colour; a rename merge keeps sharing if
      either tag had it.

    Core: migration 0009 (tags.shares_time), store::tags::sharing,
    list_tags reports shares_time.
Files:
    A  crates/blotter-core/migrations/0009_tag_shares_time.sql
    M  crates/blotter-core/src/db.rs, crates/blotter-core/src/store/tags.rs
    M  crates/blotter-core/src/ops/tags.rs, crates/blotter-core/src/ops/reads.rs
    M  crates/blotter-core/src/ops/slots.rs, crates/blotter-core/src/smart_slot.rs
    M  src/lib/tags.ts, src/lib/tags.test.ts, src/views/settings/TagsPage.tsx
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 229
    - cargo test --workspace → 188 core (3 new) + 50 app + 6 mcp; clippy -D warnings clean
    - Pending (Luke's installed Blotter was running): Settings → Tags → turn on Simultaneous
      for a tag; a task with it auto-slots over a scheduled task, not over an event, and the
      overlap shows no conflict line
Notes:
    - Dragging was never refused for overlaps, so drag needs no change; it only shows (or
      no longer shows) the conflict.

### #68 — Holidays leave Sleep-type block-outs in place            [status: committed 57ade5b]
Phase: 15c
Message:
    fix(core): holidays leave Sleep-type block-outs in place

    A holiday lifted every block-out on its days, Sleep included. It now
    lifts only the block-outs that permit fixed items; ones that warn
    about them (Sleep) stay, so a day off still has its night. Migration
    0010 drops the Sleep lifts existing holidays made. The Holidays copy
    in Settings says so.
Files:
    A  crates/blotter-core/migrations/0010_holidays_keep_sleep.sql
    M  crates/blotter-core/src/db.rs, crates/blotter-core/src/ops/holidays.rs
    M  src/views/settings/HolidaysSection.tsx
    M  .claude/COMMITS.md
Test steps:
    - cargo test -p blotter-core → 189 (1 new); clippy -D warnings clean
    - pnpm check → clean; Vitest 229
Notes:
    - Turning permits_fixed off on a block-out later doesn't undo lifts a holiday already
      made on it; editing the holiday's dates redoes them.

### #69 — Public holiday import            [status: committed f63b0a4]
Phase: 15c
Message:
    feat: public holiday import

    Public holidays for where the user is, from Nager.Date (free, no key),
    in a new first-run step and in Settings → Block-outs → Holidays.

    - The country, and the state or province where the zone tells it
      (Australia/Melbourne → AU-VIC; Australian and Canadian zones), are
      guessed from the time zone and confirmed in two selects; the region
      list comes from the API.
    - Import brings in this year's and next year's public holidays (not
      bank or school days), from today on, as one-day holidays: work
      block-outs are lifted, #work tasks stay off them, the holiday tag shows
      in every view. Re-importing (or changing region) replaces what an
      earlier import brought in for those years; the user's own holidays
      are never touched. One undoable batch.
    - "Fetch each next year's automatically" (on by default): the minute
      check fetches next year's once the current year is the last one in,
      at most one try a day.
    - Settings shows them in their own card (place, the next one, the rest
      folded), with Change… and Remove.

    Core: public_holidays (zone → country table generated from tzdata's
    zone.tab and backward, region guess, filtering the API's answer),
    migration 0011 (holidays.source), ops get_public_holidays,
    import_public_holidays, set_public_holiday_updates and
    forget_public_holidays. App: the fetch commands and the yearly update;
    subscriptions' fetch takes an Accept header (fetch_text).
Files:
    A  crates/blotter-core/migrations/0011_holiday_source.sql
    A  crates/blotter-core/src/public_holidays/mod.rs, crates/blotter-core/src/public_holidays/zones.rs
    M  crates/blotter-core/src/lib.rs, db.rs, settings.rs, model/holiday.rs, model/mod.rs
    M  crates/blotter-core/src/ops/holidays.rs, ops/settings.rs, store/holidays.rs
    A  src-tauri/src/public_holidays.rs
    M  src-tauri/src/lib.rs, src-tauri/src/rollover.rs, src-tauri/src/subscriptions.rs
    A  src/lib/publicHolidays.ts, src/lib/publicHolidays.test.ts, src/views/settings/PublicHolidays.tsx
    M  src/app/FirstRun.tsx, src/lib/types.ts, src/views/settings/HolidaysSection.tsx
    M  .claude/COMMITS.md
Test steps:
    - cargo test --workspace → 195 core (6 new) + 50 app + 6 mcp; clippy -D warnings clean
    - cargo test -p blotter live_nager -- --ignored → AU-VIC 2027 has Melbourne Cup and Anzac
      Day; AU lists AU-VIC; 50+ countries (passed here, 2 s)
    - pnpm check → clean; Vitest 231
    - Pending (Luke's installed Blotter was running): setup's Public holidays step guesses
      Australia / VIC and imports on Continue; Settings → Holidays card shows VIC, Australia,
      the next one (Melbourne Cup), Change… to NSW swaps them, Remove clears them, Ctrl+Z
Notes:
    - zones.rs is generated (553 zones) from tzdata's current zone.tab plus chrono-tz's
      backward file by a throwaway script; regenerate the same way if a zone goes missing.
    - Region names show as codes (VIC, NSW): nothing offline names subdivisions.
    - The yearly fetch runs as the system, so Ctrl+Z skips it (D24).

### #70 — In-app updates            [status: committed 46c48ed]
Phase: 15d
Message:
    feat(app): in-app updates

    The Tauri updater (D31), signed with Blotter's own minisign key rather
    than an OS code signature.

    - A minute after start and then once a day (Settings → General → Check
      for updates, on by default), Blotter reads the latest release's
      latest.json. A newer version shows a notice: "Restart to update" for
      installs Blotter owns (NSIS, MSI, AppImage: download, install,
      relaunch), or the release notes and how to update for ones a package
      manager owns (.deb, Flatpak, AUR and other source builds). "Check now"
      and the palette's Check for updates look straight away.
    - release.yml signs the NSIS, MSI and AppImage bundles with the
      TAURI_SIGNING_PRIVATE_KEY secrets (through tauri.release.conf.json, so
      local builds need no key), and a new job writes latest.json
      (scripts/updater-manifest.mjs) into the draft release. Installed
      copies only see an update once the draft is published.
    - .gitignore keeps *.key files and a stray literal "~" folder out.

    App: tauri-plugin-updater 2.12.0 and tauri-plugin-opener 2.5.5 (for the
    release notes link), updates.rs (check_for_update, available_update,
    install_update, open_release_notes). Core: update_check_enabled setting.
Files:
    A  src-tauri/src/updates.rs, src-tauri/tauri.release.conf.json, scripts/updater-manifest.mjs
    A  src/app/updates.ts
    M  src-tauri/Cargo.toml, Cargo.lock, src-tauri/src/lib.rs, src-tauri/src/rollover.rs
    M  src-tauri/tauri.conf.json   (plugins.updater: public key, endpoint, passive Windows install)
    M  crates/blotter-core/src/settings.rs, crates/blotter-core/src/ops/settings.rs
    M  src/actions/actions.ts, src/app/AppShell.tsx, src/views/settings/GeneralPage.tsx
    M  .github/workflows/release.yml, .gitignore
    M  .claude/COMMITS.md
Test steps:
    - cargo test --workspace → 195 core + 51 app (updater_config_is_valid new) + 6 mcp;
      clippy -D warnings clean
    - pnpm check → clean; Vitest 231
    - node scripts/updater-manifest.mjs v1.2.0 LASR-0/Blotter <dummy bundles> → nsis, msi
      and appimage entries with their signatures and download URLs
    - Pending: the first tagged release with the secrets set (the workflow's signing and
      updater jobs); then an installed 1.2.0 seeing 1.2.1 and updating itself
Notes:
    - Luke's key pair lives in C:\Users\ROXBLUK\.tauri (moved out of a literal "~" folder in
      the repo, never committed). Losing the private key means installed copies can't take
      updates signed with a new one: keep a backup.
    - 1.1.0 has no updater, so 1.2.0 itself is installed by hand; 1.2.1 is the first update
      it can take.

### #70a — Plain update errors and the holiday list's empty line            [status: committed 2d94a55]
Phase: 15d
Message:
    fix(ui): plain update errors and the holiday list's empty line

    Found in the live checks of #66–#70:

    - A check that finds no latest.json (before the first release that has
      one, or offline) said "Could not fetch a valid release JSON from the
      remote"; it now says GitHub has no update information for the latest
      release, or Blotter couldn't reach it.
    - With public holidays imported, the Holidays section still said "No
      holidays coming up" under them; it now says none of your own are.
Files:
    M  src-tauri/src/updates.rs, src/views/settings/HolidaysSection.tsx
    M  .claude/COMMITS.md
Test steps:
    - clippy -D warnings clean; pnpm check → clean; Vitest 231
    - Live, on a scratch copy of the demo data (Australia/Brisbane), through the DevTools
      protocol — the pending checks of #66–#70:
        · #66 + in the Inbox rail (open and collapsed) opens New item with no date; Week's
          top-bar + opens it on Wed 21, and on Thu 22 after moving the cursor; Month likewise
        · #67 Settings → Tags shows Simultaneous per tag; a new #laundry tag switched on;
          auto-slot put Laundry 10:00 over Cleaning (10:00–12:00) and Tidy email 13:00 after
          the Vet event (12:00–13:00); no conflicts; Week shows them side by side
        · #68/#69 Holidays guessed Australia / QLD; Import → 13 holidays, Work lifted on 11,
          Sleep on none; switching to VIC replaced them (16, Melbourne Cup); Remove cleared all
          and their lifts; Ctrl+Z brought all back; setup step 4/6 guessed QLD and Continue
          imported it
        · #70 the updater plugin starts; Check now shows the (now plain) error, since no
          release has latest.json yet; the setting's switch reads on
Notes:
    - Setup's holiday step guesses from the time zone even when public holidays were already
      imported for another region (only matters for "Run setup again").

### #71 — v1.2.0            [status: committed — hash in the next entry, untagged]
Phase: 15d
Message:
    chore(release): v1.2.0

    Version 1.2.0 in Cargo.toml (all three crates), package.json,
    tauri.conf.json, the AUR PKGBUILD and .SRCINFO (checksum back to SKIP
    until the tag exists), and the AppStream metainfo with a 1.2.0 release
    note. The README covers public holidays, simultaneous tags, the setup
    step and in-app updates (from 1.2.0; 1.1.0 users install 1.2.0 by hand
    once). Roadmap §13 progress.
Files:
    M  Cargo.toml, Cargo.lock, package.json, src-tauri/tauri.conf.json   (1.2.0)
    M  packaging/aur/PKGBUILD, packaging/aur/.SRCINFO, packaging/linux/io.github.lasr0.Blotter.metainfo.xml
    M  README.md
    M  .claude/COMMITS.md, .claude/roadmap.md   (§13 progress)
Test steps:
    - pnpm check → clean; Vitest 231
    - cargo test --workspace → 195 core + 51 app + 6 mcp
    - pnpm tauri build --bundles nsis,msi (no signing key) → Blotter_1.2.0_x64-setup.exe
      (7.6 MB), Blotter_1.2.0_x64_en-US.msi (10.7 MB); the bundler stamps each exe with its
      bundle type, which the updater reads
Notes:
    - Luke: push, then git tag -a v1.2.0 -m "Blotter 1.2.0" && git push origin v1.2.0. The
      release workflow signs the NSIS, MSI and AppImage bundles and adds latest.json to the
      draft; check the Updater manifest job's log shows all five platform entries.
    - Publish the draft after the smoke checks; latest.json only resolves once it's published.
    - AUR: updpkgsums and makepkg --printsrcinfo > .SRCINFO once the tag exists.

### #72 — v1.2.1 sync plan            [status: committed — hash in the next entry]
Phase: 16
Message:
    docs: v1.2.1 sync plan

    Blotter 1.2.1 makes every desktop able to sync through Blotter Sync, a
    headless Blotter server in its own repo (LASR-0/Blotter-sync) with its
    own version, roadmap and ledger. Roadmap §14 is the app-side brief:
    compatibility with the server (sync protocol and schema versions, a
    table in both READMEs), the sync model, field groups, what syncs, the
    sync engine, joining, duties handed to the server, UI and phases
    16a-16c. §10 and D32 point to it. Planned commits #73-#85.

    #71 (v1.2.0) is 4cb1d74.
Files:
    M  .claude/roadmap.md, .claude/COMMITS.md
Test steps:
    - none (docs only)
Notes:
    - Confirmed by Luke 2026-10-22: D-S7 to D-S10, O1 (custom themes and keybindings
      sync; the selected theme stays per device), separate repos, versions.
    - Proposed in the server roadmap: D-S15 (gate on a sync schema version) and D-S16
      (the names Blotter Sync / blotter-sync).
    - Per-entry file lists are filled in as each entry moves to ready, as before.

### #72a — Sync schema gate confirmed, Blotter-Sync repo name            [status: committed — hash in the next entry]
Phase: 16
Message:
    docs: sync schema gate confirmed, Blotter-Sync repo name

    Luke confirmed gating by a sync schema version (D-S15 in the server
    roadmap) and the name blotter-sync (D-S16). The server repo is
    LASR-0/Blotter-Sync. #72 is 7cb6137.
Files:
    M  .claude/roadmap.md, .claude/COMMITS.md
Test steps:
    - none (docs only)

### #73 — Device identity and hybrid logical clock            [status: committed — hash in the next entry]
Phase: 16a
Message:
    feat(core): device identity and hybrid logical clock

    The first piece of sync groundwork (roadmap §14), with no visible
    change. A new blotter_core::sync module:

    - sync::device: every database gets a device id (a ULID) the first
      time it is opened, kept for good, plus an optional name for the
      user to set (trimmed, 1-64 characters). Two connections opening a
      new database at once (app and MCP) agree on one id.
    - sync::hlc: hybrid logical clock stamps (wall ms, counter, device
      id), ordered by all three, stored and sent as fixed-width text so
      text order is stamp order in Rust and SQL alike. tick() for a
      local write (run in the write's transaction), observe() for a
      stamp from another replica, current(). A wall clock that steps
      back is carried by the counter; a full counter moves to the next
      millisecond.
    - Migration 0012 adds the device-local sync_device and sync_clock
      tables. Snapshot restore keeps this device's identity and the later
      of the two clocks, whatever the restored file holds (another
      install's id, an older clock, or none for a pre-1.2.1 file). JSON
      import never touches them.

    #72a is 78626ec.
Files:
    A  crates/blotter-core/migrations/0012_sync_device.sql
    A  crates/blotter-core/src/sync/mod.rs, crates/blotter-core/src/sync/device.rs,
       crates/blotter-core/src/sync/hlc.rs
    M  crates/blotter-core/src/lib.rs, crates/blotter-core/src/db.rs
    M  .claude/COMMITS.md
Test steps:
    - cargo test --workspace → 212 core (17 new) + 51 app + 6 mcp; clippy -D warnings clean;
      fmt clean
    - Upgrade check on a copy of Luke's 1.2.0 database (never the original): schema 11 → 12;
      items, reminders, tags, blockouts, exceptions, holidays, changes and settings all keep
      their counts; a device id appears, ticks, and is the same after reopening
Notes:
    - Nothing writes stamps yet; capture (#75) ticks the clock from SQL triggers, in the
      same fixed-width format, so the app and the MCP process share one clock.
    - The device name is set from Settings → Sync (#81); until then it stays empty.

### #74 — Sync registry            [status: committed — hash in the next entry]
Phase: 16a
Message:
    feat(core): sync registry for tables, columns and settings

    sync::registry classifies every table, column and setting as synced,
    device-local or server-owned, and puts every column of a shared table
    in exactly one field group (roadmap §14.5): the CHECK-tied columns
    merge together (items timing, status, kind, window and series;
    reminder triggers; block-out hours and active range; holiday ranges),
    as do a block-out exception's UNIQUE (blockout_id, date) pair, a
    setting's value and timestamp, and a keybinding's binding and
    timestamp. reminders.fired_at is device-local (D-S9); subscription
    fetch status and the ics_events cache are server-owned. It also lists
    each shared table's UNIQUE constraints and cascading parents, for the
    merge.

    SYNC_SCHEMA (1) is the version Blotter Sync gates on (D-S15). Tests
    hold the registry to the migrated schema: every table classified
    once, keys equal to primary keys, every column in one group, every
    multi-column CHECK inside one group (parsed from the schema), UNIQUE
    constraints and cascades as listed, every settings.rs key in one
    class, DEVICE_KEYS never synced, and a pinned fingerprint of
    everything shared, so changing what syncs without bumping
    SYNC_SCHEMA fails.

    Settings: clock (12/24-hour) syncs; ai_mode stays per device, as it
    already did through restore and import. Unknown keys stay local.
    settings.rs gains KEY_THEME for the "theme" key. #73 is eda091d.
Files:
    A  crates/blotter-core/src/sync/registry.rs
    M  crates/blotter-core/src/sync/mod.rs, crates/blotter-core/src/settings.rs
    M  .claude/roadmap.md   (§14.5 settings: clock synced, ai_mode device-local)
    M  .claude/COMMITS.md
Test steps:
    - cargo test --workspace → 225 core (13 new) + 51 app + 6 mcp; clippy -D warnings clean;
      fmt clean
    - Mutation check: dropping items.tz from the registry fails two tests ("items.tz isn't
      classified", and the fingerprint); restored
Notes:
    - For Luke: ai_mode (Propose/Auto) is device-local and clock (12/24-hour) is synced.
      The brief had ai_mode synced and didn't list clock; say if either should change.
    - FINGERPRINTS[0] may still be replaced until 1.2.1 ships (#75 adds sync tables, all
      device-local, which don't change it).

### #75 — Capture synced writes into an outbox            [status: committed — hash in the next entry]
Phase: 16a
Message:
    feat(core): capture synced writes into an outbox

    Change capture with SQLite triggers (D-S7), so every write to synced
    data is stamped and queued, whichever code path or process makes it:
    the operation layer, direct store writes (settings, keybindings,
    themes, subscriptions, reminder snoozes), imports, and the MCP
    server's own connection.

    - sync::capture generates insert, update, re-key and delete triggers
      for every synced table from the registry and installs them when
      the database opens; a fingerprint in sync_state reinstalls them
      only when the registry or role changes. An update stamps only the
      field groups whose columns changed; a write that changes nothing
      synced isn't captured at all.
    - The triggers tick the hybrid logical clock in SQL, in the same
      format as sync::hlc, so the app, the MCP process and Rust code all
      share one clock.
    - sync_clocks (migration 0013) keeps the stamp per row and group,
      and per row under the group "" when it was created or deleted (a
      tombstone; cascaded deletes leave theirs). Its pending flag is the
      outbox: pending(), pending_count(), and acknowledge(), which keeps
      a group pending if a newer write stamped it after it was read.
    - Settings sync per key; device-local columns and tables
      (reminders.fired_at, the undo log, chat, snapshots) are never
      captured. Server-owned data (subscription fetch status, the
      ics_events cache, last_rollover_date) is captured only once Blotter
      Sync sets the role to server.
    - Writes made while applying remote changes aren't captured: the
      merge sets sync_state.applying inside its transaction (the
      connection-scoped flag of D-S7; a main-schema trigger can't read a
      temp table, and no other connection can write in between).
    - Upgrading, or restoring a pre-1.2.1 snapshot, stamps every existing
      row once. Restore keeps the role.
    - Database::open_in_memory copies one migrated template instead of
      migrating every test database (the core tests had gone from 5 s to
      16 s; now 9 s).

    #74 is 12a01e5.
Files:
    A  crates/blotter-core/migrations/0013_sync_capture.sql
    A  crates/blotter-core/src/sync/capture.rs
    M  crates/blotter-core/src/sync/mod.rs, crates/blotter-core/src/sync/registry.rs,
       crates/blotter-core/src/sync/device.rs, crates/blotter-core/src/db.rs
    M  .claude/COMMITS.md
Test steps:
    - cargo test --workspace → 243 core (18 new) + 51 app + 6 mcp; clippy -D warnings clean;
      fmt clean
    - Cost (release build): 1,000 item inserts + 1,000 updates in one transaction take
      396 ms with capture and 27 ms without (~0.2 ms a write; an operation makes a handful).
      Installing the triggers takes ~7 ms, once per upgrade.
    - Upgrade check on a copy of Luke's database: schema 11 → 13; first open 16 ms; every
      row stamped once (items 29 rows, 464 stamps = 16 each; settings 6 synced keys of 13);
      715 changes pending; the next open 5 ms
Notes:
    - The outbox is the pending flag on sync_clocks, not a separate table: a group pending
      twice is one entry with the newer stamp, and a push reads current values with them.
    - Local only mode captures too, so a first pairing can push everything it holds.

### #76 — Wire types and merging per field group            [status: committed — hash in the next entry]
Phase: 16a
Message:
    feat(core): wire types and merging remote changes per field group

    The merge every replica runs on rows from another (roadmap §14.5),
    and the types they travel in.

    - sync::wire: SYNC_PROTOCOL (1); RowState, one row's existence stamp
      and each field group's stamp and values, the one shape pushes,
      pulls and bootstraps carry; Merged and Rejection.
    - sync::state: a replica's rows as RowStates: row(), outbox() (every
      row with a pending change, whole, plus the entries to acknowledge)
      and all() (a bootstrap, tombstones included).
    - sync::merge: field groups are last-writer-wins by stamp. A row
      exists by the newer of its creation and deletion stamps, a delete
      winning a tie, and edits only stamp groups, so a concurrent edit
      never brings back a deleted row (D-S10) while making it again (an
      undo) does. A parent's tombstone takes its children with it,
      stamped at the later of the two; a UNIQUE collision keeps the row
      created first. Tombstones the merge works out itself go out with
      the next push, so every replica learns them. One transaction per
      batch with capture off and foreign keys checked at the end; a row
      that can't apply is refused on its own. Devices take server-owned
      data from the server; the server ignores it from devices; neither
      takes device-local tables or settings. The clock moves past the
      newest stamp merged.

    #75 is bbaf848.
Files:
    A  crates/blotter-core/src/sync/wire.rs, crates/blotter-core/src/sync/state.rs,
       crates/blotter-core/src/sync/merge.rs
    M  crates/blotter-core/src/sync/mod.rs, crates/blotter-core/src/sync/capture.rs
    M  .claude/COMMITS.md
Test steps:
    - cargo test --workspace → 260 core (17 new) + 51 app + 6 mcp; clippy -D warnings clean;
      fmt clean
    - Two-replica cases: a new row arrives whole and isn't sent back; edits to different
      groups both survive; the newer edit to one group wins; start and end move together
      (no row with end before start); delete beats a concurrent edit in either order; an
      undo of a delete brings the row back; a reminder added to an item deleted elsewhere
      goes too; two lifts of the same day keep the first; a newer remote edit clears a
      pending local one; merging twice changes nothing; the clock passes merged stamps; bad
      rows are refused one by one; server-owned data flows server → device only; a child
      whose parent was never known is refused; one tag made on two devices merges into one
Notes:
    - Randomised convergence across many replicas and orderings is #77.
    - A rejection is reported, not retried: Blotter Sync returns them to the device (#05
      there), which shows them as a sync problem.

### #77 — Multi-replica convergence tests            [status: committed — hash in the next entry]
Phase: 16a
Message:
    test(core): multi-replica convergence tests

    Property tests (proptest) for the merge (roadmap §14.10): three
    devices and a server hub with a change log run random interleavings
    of item edits, reschedules, completions, deletes and re-creations,
    reminders and snoozes, block-out lifts (colliding on purpose), tags
    and settings, syncing now and then with each pull shuffled and, at
    random, earlier log entries delivered again. Once everything has
    synced, every device holds exactly the server's rows, stamps and
    values, nothing is left pending, and no row breaks a constraint or
    lost its parent. 64 cases per run; failures shrink and are kept in
    proptest-regressions/ so they replay.

    The first long run found that settling in a fixed number of rounds
    could stop one round early: a tombstone a device works out in a
    merge reaches the others with its next push. The test now settles
    until nothing is pending and every device has pulled the whole log,
    and the shrunk case is a named test. #76 is b875e15.
Files:
    A  crates/blotter-core/src/sync/convergence.rs
    A  crates/blotter-core/proptest-regressions/sync/convergence.txt
    M  crates/blotter-core/src/sync/mod.rs, crates/blotter-core/Cargo.toml, Cargo.lock
       (proptest 1.12.0, dev only)
    M  .claude/COMMITS.md
Test steps:
    - cargo test --workspace → 262 core (2 new) + 51 app + 6 mcp; clippy -D warnings clean;
      fmt clean
    - PROPTEST_CASES=3000 cargo test --release -p blotter-core convergence → passes (187 s)
    - Mutation checks, each restored: an incoming group always winning, whatever its stamp,
      fails the property; so does dropping "a local delete beats incoming edits"
Notes:
    - The core suite takes ~19 s in debug now (was ~12 s); most of it is the 64 cases.

### #78 — Undo and redo scoped to this device            [status: committed — hash in the next entry]
Phase: 16a
Message:
    feat(core): undo and redo scoped to this device

    Once changes arrive from other devices, Ctrl+Z still only reverses
    what this device did (H4, amends D24), and History says where the
    rest came from.

    - Migration 0014: changes.origin (null for this device's own rows,
      else the id of the device a merged change came from), and a
      device-local sync_peers table of the other devices' names, which
      the app fills from the server (#81).
    - The merge records what it applied in the change log as "sync"
      rows, with the entity's state before and after and its origin, for
      items, reminders, block-outs and their exceptions, holidays, tags
      and settings.
    - Undo and redo, the chat's "Undo batch" and the redo check ignore
      rows with an origin; undoing one by id is refused ("undo it there").
      A merged change doesn't clear redo; if it touched the same thing,
      undo or redo refuses as it already does when an entity has changed.
    - list_completions gives each completion the device it was completed
      on, by name ("another device" until its name is known); History
      shows it in the chip where AI and MCP completions show theirs.
    #77 is d5ec285.
Files:
    A  crates/blotter-core/migrations/0014_change_origin.sql
    M  crates/blotter-core/src/db.rs, crates/blotter-core/src/model/change.rs,
       crates/blotter-core/src/store/changes.rs, crates/blotter-core/src/store/items.rs,
       crates/blotter-core/src/ops/mod.rs, crates/blotter-core/src/ops/undo.rs,
       crates/blotter-core/src/ops/history.rs
    M  crates/blotter-core/src/sync/merge.rs, crates/blotter-core/src/sync/device.rs,
       crates/blotter-core/src/sync/registry.rs, crates/blotter-core/src/sync/mod.rs
    M  src/lib/history.ts, src/views/HistoryView.tsx
    M  .claude/COMMITS.md
Test steps:
    - cargo test --workspace → 267 core (5 new) + 51 app + 6 mcp; clippy -D warnings clean;
      fmt clean
    - pnpm check → clean; Vitest 231
    - New: a merged item is logged as "sync" with the sender's id and nothing to undo, and
      undoing it by id is refused; undo reverses this device's edit, not the one merged
      after it; a remote edit to the same item blocks undo; a merged change leaves redo
      offered; a completion from the laptop reads "another device", then "Laptop" once the
      peer's name is known, and this device's own reads none
Notes:
    - Who made a merged change (user, AI, MCP) doesn't travel, only where: a remote
      completion counts as the user's, on that device.
    - The History chip with a device name is untried in the window until two instances
      sync (16b).

### #79 — Subscription and holiday upkeep a caller can drive            [status: committed — hash in the next entry]
Phase: 16a
Message:
    refactor(core): subscription and holiday upkeep a caller can drive

    Blotter Sync takes over the automatic duties once devices pair, so
    the decisions about them live in the core and each replica only
    brings its own HTTP and timer:

    - public_holidays::upkeep(planner) says which year's holidays are
      due (keep up to date on, next year not in yet), Upkeep::url() where
      to fetch them, and Upkeep::apply() imports the answer as a system
      change. year_url() builds a year's address. The app's minute check
      now uses them instead of its own copy of the logic.
    - Subscriptions already were drivable (subscriptions::due, fetch_url,
      record_fetch); rollover (Planner::rollover_if_due) and pruning (the
      prune_holidays op) already are core calls. Nothing changes there.
    #78 is 4dd7ae4.
Files:
    M  crates/blotter-core/src/public_holidays/mod.rs
    M  src-tauri/src/public_holidays.rs
    M  .claude/COMMITS.md
Test steps:
    - cargo test --workspace → 270 core (3 new) + 51 app + 6 mcp; clippy -D warnings clean;
      fmt clean
    - New: nothing is due before an import; after this year's, next year's is due with its
      URL, applying a sample answer adds New Year's Day and Melbourne Cup (not NSW's) as
      system changes, and then nothing is due; turned off, nothing is due
    - cargo test -p blotter live_nager -- --ignored → passes against date.nager.at
Notes:
    - 16a is complete in code (#73-#79). Next: Blotter Sync #03-#06 in ../Blotter-sync,
      then 16b here.

### #79a — rust-version 1.95            [status: committed — hash in the next entry]
Phase: 16a
Message:
    build: rust-version 1.95

    The workspace said 1.88, but rusqlite_migration 2.6.0 already needs
    1.95, so an older toolchain failed on a dependency rather than at the
    stated minimum. Found while setting up Blotter Sync, which builds
    blotter-core as a git dependency. #79 is 3b87cab.
Files:
    M  Cargo.toml
    M  .claude/COMMITS.md
Test steps:
    - cargo check --workspace → clean on 1.98

### #79b — The Diary moves on to today when the date changes            [status: committed — hash in the next entry]
Phase: 16 (patch, Luke's report)
Message:
    fix(ui): the Diary moves on to today when the date changes

    Left open overnight, the Diary still started on yesterday and its
    timeline still showed yesterday, until a restart. The Diary's columns
    and timeline follow the navigation anchor, which was set to midnight
    once at launch and never moved by the date changing (the columns'
    "today" marking did update, so yesterday's column just lost its
    highlight). Still so in 1.2.0.

    Now a check every minute, and whenever the window regains focus or
    becomes visible (waking from sleep), moves the anchor on to the new
    today, but only if it was on the old one: a day the user moved to
    stays put. #79a is aa31b69.
Files:
    A  src/app/useNewDay.ts, src/app/nav.test.ts
    M  src/app/nav.ts, src/app/AppShell.tsx
    M  .claude/COMMITS.md
Test steps:
    - pnpm check → clean; Vitest 236 (5 new: the anchor moves from yesterday to today,
      stays where the user moved it, doesn't move within a day, catches up after days
      asleep; the store action)
    - Pending, Luke: leave Blotter open past midnight (or put the machine to sleep across
      it) → the Diary starts on the new day and the timeline shows it

### #79c — The clock test no longer expires            [status: committed — hash in the next entry]
Phase: 16a (patch)
Message:
    test(core): the clock test no longer expires

    observing_a_later_stamp_orders_the_next_tick_after_it used fixed
    stamps on 2026-10-22, but opening a database captures the default
    settings, which ticks the clock to the real time; once that passed
    the remote stamp (08:10 UTC that day), the next tick ordered after
    the real time instead, and the test failed on every run. Its stamps
    now sit a day after the real time. #79b is bd0017b.
Files:
    M  crates/blotter-core/src/sync/hlc.rs
    M  .claude/COMMITS.md
Test steps:
    - cargo test -p blotter-core → 270 passed (failed 269/1 before); clippy, fmt clean
Notes:
    - Test-only: the clock itself was right (it should never go backwards).

### #79d — Undo per field group            [status: committed — hash in the next entry]
Phase: 16a
Message:
    feat(core): undo per field group

    Undo and redo refused whenever anything had changed the item since,
    so once a server's rollover moves tasks and other devices edit, they
    would often refuse to undo an edit nothing had touched (D33, amends
    D24). Now, when an entity isn't as the batch left it, they go ahead
    per field group, the unit sync merges by: the groups the batch
    changed must still be as it left them (or already back) and are put
    back, and every other group keeps its later value, with the row's
    updated_at from that later write. A later change to one of the
    batch's groups still refuses, as does any field outside the
    registry's groups; creating and deleting a row, and settings, stay
    all or nothing. The undo syncs as an ordinary edit. #79c is 6b33cb5.
Files:
    M  crates/blotter-core/src/ops/undo.rs, crates/blotter-core/src/sync/merge.rs
    M  .claude/roadmap.md (D33, §14 AC)
    M  .claude/COMMITS.md
Test steps:
    - cargo test --workspace → 276 core (6 new) + 51 app + 6 mcp; clippy -D warnings, fmt
      clean
    - New: undoing a move keeps a later retitle, and redo puts the move back over it; redo
      keeps an automatic move made after the undo; a system move doesn't block undoing a
      notes edit (updated_at from the move); a later edit to another column of the same
      group (timing) refuses and writes nothing; undoing a create after an edit refuses; a
      remote priority edit survives undoing a retitle and both replicas converge, while a
      remote retitle still blocks it (replaces a_remote_edit_to_the_same_item_blocks_undo)
Notes:
    - Blotter Sync's tests/two_replicas.rs expects the old refusal (a reschedule undone over
      a retitle); flip it when its blotter-core pin moves past this commit.
    - The refusal message is unchanged ("“X” has changed since").

### #79e — Blotter Sync's request and response types            [status: committed — hash in the next entry]
Phase: 16a
Message:
    feat(core): Blotter Sync's request and response types

    The HTTP API's envelopes move into blotter-core::sync::api, so the
    app's client (#80) and the server build against one copy (§14.5):
    the version headers and Versions, the endpoint paths, Hello, the
    typed ErrorCode and ErrorBody (an unknown code from a newer server
    reads as Unknown), PairRequest and Paired, push, pull and bootstrap
    bodies, the change stream's Notice, and the snapshot types. Shapes
    are exactly Blotter Sync's as of its 1.0.0-rc.1. #79d is 661cbeb.
Files:
    A  crates/blotter-core/src/sync/api.rs
    M  crates/blotter-core/src/sync/mod.rs
    M  .claude/COMMITS.md
Test steps:
    - cargo test -p blotter-core → 279 passed (3 new: hello as the server sends it, round
      trip; an error with the server's versions, an unknown code; paths and a pull page);
      clippy, fmt clean
Notes:
    - Blotter Sync switches its api.rs, changes.rs, auth.rs and admin.rs types to these
      when its blotter-core pin moves (its own ledger entry); its axum status mapping stays
      there.

### #80 — Sync engine with push, pull and address failover            [status: committed — hash in the next entry]
Phase: 16b
Message:
    feat(app): sync engine with push, pull and address failover

    Once a home server is paired, the app syncs with it in the
    background (§14.6); in Local only mode nothing runs.

    - Core: migration 0015 adds sync_server, the device-local link to
      the server (its id, the ordered addresses, the epoch and last seq
      pulled, when it last synced), kept across a snapshot restore with
      the device's identity. No row is Local only.
    - The engine picks the first address answering as this device's
      server with matching versions, and rechecks earlier ones every 5
      minutes so a laptop back home returns to the LAN; another server
      at an address is skipped. It pushes the outbox, acknowledges it
      (rejections are logged), pulls page by page and merges each, and
      records the cursor.
    - It runs after local writes (debounced 1.5 s; its own merges don't
      wake it), on every notice from the server's change stream, every
      minute as a fallback, and with backoff and jitter (2 s to 5 min)
      while unreachable. Merged rows refresh the windows through the
      changed event.
    - One status for the UI (sync_status, "sync:status"): local only,
      syncing, synced via an address, offline with why each address
      failed and a Tailscale hint, or needs attention: update Blotter or
      update Blotter Sync (from the versions), revoked, unauthorized,
      fetch the server's planner again, a different server. Plus unsent
      changes and the last sync.
    - Plain http:// only to loopback, private, link-local, unique-local
      and Tailscale addresses, judged by what a name resolves to
      (D-S6). The token is read from the OS keychain.
    #79e is 2c89f08.
Files:
    A  crates/blotter-core/migrations/0015_sync_server.sql, crates/blotter-core/src/sync/link.rs
    M  crates/blotter-core/src/db.rs, crates/blotter-core/src/sync/device.rs,
       crates/blotter-core/src/sync/mod.rs, crates/blotter-core/src/sync/registry.rs
    A  src-tauri/src/sync/mod.rs, src-tauri/src/sync/engine.rs, src-tauri/src/sync/client.rs,
       src-tauri/src/sync/address.rs, src-tauri/src/sync/token.rs
    M  src-tauri/src/lib.rs, src-tauri/Cargo.toml (url; reqwest gzip; tokio net), Cargo.lock
    M  .claude/COMMITS.md
Test steps:
    - cargo test --workspace → 281 core (2 new) + 63 app (12 new) + 6 mcp; clippy -D
      warnings, fmt clean
    - New, against an in-memory server: two devices sync both ways and the outbox empties;
      failover to Tailscale and back to the LAN only at the recheck; offline lists each
      address's failure and the Tailscale hint; another server at an address is skipped,
      and alone it's "a different server"; mismatches name the side to update; revoked,
      unauthorized and stale epoch need attention, a rate limit retries; Local only and a
      missing token do nothing. The link survives a restore. Address rule and Tailscale
      ranges; backoff; status JSON.
    - Live (ignored test live_sync_against_blotter_sync, BLOTTER_LIVE_SYNC and
      BLOTTER_LIVE_SYNC_CODE): against Blotter Sync 1.0.0-rc.1 on 127.0.0.1 → paired,
      pushed, pulled, heard the push's seq on the change stream, a bad token is
      unauthorized. Passed.
Notes:
    - Nothing sets a link yet: pairing and Settings → Sync are #81, so this is inert in
      the app until then. normalize() and the token's set/delete come with #81.
    - A stale epoch only reports "fetch again"; re-bootstrapping is #82 with joining.
    - The duties still run here too until #83.

### #81 — Pairing and Settings → Sync            [status: committed — hash in the next entry; Settings → Sync confirmed by Luke 2026-10-26]
Phase: 16b
Message:
    feat(app): pairing and Settings → Sync

    Settings gains a Sync page (§14.9), so the engine from #80 has a
    server to sync with.

    - Local only shows the mode and a Pair form: address, the code from
      `blotter-sync pair`, and this device's name. Addresses are read as
      typed (no scheme is plain http, which defaults to port 7458), and
      plain http to a public IP is refused (D-S6). Pairing checks the
      server's hello first, so a version mismatch says which side to
      update and a closed code says to run `blotter-sync pair`. The token
      goes to the OS keychain.
    - A first pairing queues the whole planner again (core:
      capture::queue_all), so a device that synced with an earlier server
      still seeds a new one. Pairing again with the same server, after a
      revoke or a lost token, keeps the link and its cursor; another
      server needs a Disconnect first.
    - Home server shows the status ("Synced via Tailscale · 2 min ago",
      offline with each address's failure, or what needs attention, with
      Pair again for a revoked or unknown token), the server's version,
      the address list (add, reorder, remove, test), this device's name
      and Disconnect.
    - Disconnect revokes this device's token at the first address that
      answers as its server, forgets the link and the token, and keeps
      the planner. Offline, it still disconnects and says to revoke the
      device on the server.
    - Editing the link makes the engine drop its route and change
      stream. The palette gains "Sync settings".
    #80 is f22b2ae.
Files:
    M  crates/blotter-core/src/sync/capture.rs
    A  src-tauri/src/sync/pairing.rs, src-tauri/src/sync/fake.rs
    M  src-tauri/src/sync/mod.rs, src-tauri/src/sync/engine.rs, src-tauri/src/sync/client.rs,
       src-tauri/src/sync/address.rs, src-tauri/src/sync/token.rs, src-tauri/src/lib.rs
    A  src/views/settings/SyncPage.tsx, src/lib/syncFormat.ts, src/lib/syncFormat.test.ts
    M  src/views/SettingsView.tsx, src/app/nav.ts, src/actions/actions.ts
    M  .claude/COMMITS.md
Test steps:
    - cargo test --workspace → 282 core (1 new) + 72 app (9 new) + 6 mcp; clippy -D
      warnings, fmt clean
    - pnpm check → typecheck, eslint, prettier clean; vitest 242 (syncFormat new)
    - New: addresses as typed and why one can't be used; pairing saves the token, names
      the device and queues an acknowledged planner; each refusal (no code open, wrong
      code, public http, unreachable, newer server, no code, no name) says what to do and
      changes nothing; pairing again keeps the link and cursor, another server waits for
      Disconnect; the address list normalises and dedupes; testing an address names who
      answered; Disconnect revokes past another server on the LAN and keeps the planner,
      and offline still disconnects. The engine's in-memory server moved to fake.rs, shared
      with these tests. syncFormat: route labels, "2 min ago", every status line, moves.
    - Live (ignored test live_pair_test_and_disconnect, BLOTTER_LIVE_SYNC and
      BLOTTER_LIVE_SYNC_CODE): against Blotter Sync 1.0.0-rc.1 on 127.0.0.1 → paired, the
      address tests ok, the used code is refused, Disconnect revoked the device (the
      server lists it as revoked) and its token is refused. Passed; the engine's live
      test still passes.
Notes:
    - Not yet checked by eye in the app window: Luke to look over Settings → Sync (both
      modes, Pair again, Disconnect's dialog) in `pnpm tauri dev` against a local server.
    - Blotter Sync has no rename endpoint, so the device name is set when pairing and shown
      read-only afterwards; renaming would be a server change (its own ledger).
    - Joining a server that already has a planner merges without §14.7's choice or its
      natural-key rules until #82 (with first run's "Connect to a home server"); a stale
      epoch still only reports.
    - Disconnect leaves unsent changes queued, and the next pairing queues everything
      anyway.

### #82 — Connect to a home server from first run, and joining            [status: committed — hash in the next entry]
Phase: 16b
Message:
    feat(app): connect to a home server from first run, and joining

    Pairing now fetches the server's planner and joins it (§14.7), and a
    planner replaced on the server is fetched again (§14.8).

    - Core: sync::join. has_planner tells a planner from nothing (live
      synced rows, not rollover bookkeeping or fetched events).
      replace_with (Use the server's planner) swaps every synced row for
      a bootstrap in one transaction and keeps device-local data.
      merge_into (Merge) queues everything here, lets the server win
      tags, settings, keybindings and themes it also has, and drops this
      device's copy of a public:* holiday the server has for the same
      dates, moving its block-out lifts to the server's copy. merge is
      split so both run in one transaction with it. Snapshots gain the
      before_sync reason.
    - Pairing: an empty server gets this device's whole planner; an
      empty device takes the server's; when both have one, nothing is
      linked until the user picks Use the server's planner or Merge
      (both snapshot this device first) or cancels, which unpairs. A
      failed fetch unpairs and saves no token.
    - Settings → Sync shows the choice while a join waits; first run
      opens with "Set up this device" or "Connect to a home server", and
      a device that took the server's planner skips the steps that make
      data (week, work hours, holidays).
    - When the server's planner was restored or imported there, the
      engine snapshots this device (unsent changes stay in it), fetches
      the new planner and pulls on from its epoch; a banner says so,
      with Open Backups.
    #81 is 7673cef.
Files:
    A  crates/blotter-core/src/sync/join.rs
    M  crates/blotter-core/src/sync/mod.rs, crates/blotter-core/src/sync/merge.rs,
       crates/blotter-core/src/snapshots.rs
    M  src-tauri/src/sync/pairing.rs, src-tauri/src/sync/engine.rs, src-tauri/src/sync/client.rs,
       src-tauri/src/sync/fake.rs, src-tauri/src/sync/mod.rs, src-tauri/src/lib.rs
    A  src/views/settings/SyncPairing.tsx, src/app/SyncBanner.tsx
    M  src/views/settings/SyncPage.tsx, src/app/FirstRun.tsx, src/app/AppShell.tsx,
       src/views/settings/BackupsPage.tsx, src/lib/syncFormat.ts, src/lib/syncFormat.test.ts
    M  .claude/COMMITS.md
Test steps:
    - cargo test --workspace → 285 core (3 new) + 77 app (5 new) + 6 mcp; clippy -D
      warnings, fmt clean
    - pnpm check → typecheck, eslint, prettier clean; vitest 243
    - New, core: what counts as a planner; Use the server's replaces tags, items and
      synced settings and keeps device-local ones with nothing queued; Merge unions items,
      the server's tag colour wins, a doubled Melbourne Cup becomes the server's with the
      lift moved to it, and the outbox brings the server level.
    - New, app: an empty device takes the server's planner; with both, nothing links
      before the choice, and each choice snapshots (before_sync) and joins as described;
      cancelling unpairs; a failed fetch unpairs without a token; a restore on the server
      is fetched again after a snapshot, unsent changes counted, and without a snapshot
      folder it only needs attention.
    - Live, against Blotter Sync 1.0.0-rc.1 on 127.0.0.1: live_pair_test_and_disconnect
      on a server with a planner (Merge) and on an empty one (seeded);
      live_refetch_after_a_restore (new; BLOTTER_LIVE_SYNC_ADMIN and
      BLOTTER_LIVE_SYNC_DATA): `blotter-sync snapshot`, a push, `restore` → fetched
      again, the restored-away and unsent items gone here; the engine's live test still
      passes.
Notes:
    - Not yet checked by eye: first run's opening choice and its connect and join
      screens, the Join card in Settings → Sync, and the banner after a restore.
    - Merge keeps block-outs both sides made (two "Work"); the card says they may need
      tidying. Natural keys and public holidays follow §14.7.
    - A join waiting on the choice lives in memory: quitting before choosing leaves the
      device paired on the server but not linked here; pair again (Settings → Sync), or
      revoke it on the server.
    - The rollover and sync banners share a spot; both showing at once overlap.
    - The duties still run here too until #83.
