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
| 01 | 0 | `chore: scaffold Tauri 2 + React + Vite app with Cargo workspace` | ready |
| 02 | 0 | `chore: add rustfmt, clippy, eslint, prettier and vitest tooling` | planned |
| 03 | 0 | `ci: build and test on Windows and Linux` | planned |
| 04 | 1 | `feat(core): domain models, SQLite schema and migrations` | planned |
| 05 | 1 | `feat(core): operation registry with JSON schemas and change log` | planned |
| 06 | 2 | `feat(app): tray, close-to-tray and single-instance CLI handling` | planned |
| 07 | 2 | `feat(ui): shadcn/ui setup, theme tokens and built-in presets` | planned |
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

### #01 — Scaffold Tauri 2 + React + Vite with Cargo workspace            [status: ready]
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
