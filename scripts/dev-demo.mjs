// `pnpm dev:demo [--fresh]`: runs `pnpm tauri dev` against a demo database,
// leaving your real data untouched.
//
// The demo lives in <OS temp>/blotter-demo (override with BLOTTER_DEMO_DIR).
// It is seeded with crates/blotter-core/examples/seed.rs when it doesn't
// exist yet, when it was seeded in an earlier week (the seed is relative to
// the current week), or when --fresh is passed. Otherwise it's kept, so
// changes made while testing survive restarts.

import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const dir = process.env.BLOTTER_DEMO_DIR || join(tmpdir(), 'blotter-demo');
const db = join(dir, 'blotter.db');
const marker = join(dir, 'seeded-week');
const fresh = process.argv.includes('--fresh');

/** Monday of the current local week, yyyy-MM-dd (the seed's week too). */
function thisMonday() {
  const d = new Date();
  d.setDate(d.getDate() - ((d.getDay() + 6) % 7));
  const pad = (n) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

// pnpm is a .cmd shim on Windows, so it needs a shell; cargo doesn't (and a shell
// would split a demo path containing spaces).
function run(cmd, args, { env = {}, shell = false } = {}) {
  const r = spawnSync(cmd, args, {
    stdio: 'inherit',
    shell: shell && process.platform === 'win32',
    env: { ...process.env, ...env },
  });
  if (r.status !== 0) process.exit(r.status ?? 1);
}

const week = thisMonday();
const seededWeek = existsSync(marker) ? readFileSync(marker, 'utf8').trim() : null;
const reason = fresh
  ? '--fresh'
  : !existsSync(db)
    ? 'no demo database yet'
    : seededWeek !== week
      ? `seeded for the week of ${seededWeek ?? 'unknown'}`
      : null;

if (reason) {
  console.log(`dev:demo: seeding a new demo database (${reason})`);
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
  run('cargo', ['run', '-q', '-p', 'blotter-core', '--example', 'seed', '--', db]);
  writeFileSync(marker, week);
} else {
  console.log("dev:demo: reusing this week's demo database (pass --fresh to reset it)");
}

console.log(`dev:demo: BLOTTER_DATA_DIR=${dir}`);
console.log('dev:demo: close any running Blotter first, or this launch is handed to it');
run('pnpm', ['tauri', 'dev'], { env: { BLOTTER_DATA_DIR: dir }, shell: true });
