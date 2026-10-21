// Writes latest.json for the Tauri updater from a release's signed
// bundles: node scripts/updater-manifest.mjs <tag> <repo> <dir> > latest.json
// <dir> holds the installers and their .sig files (searched recursively).
import fs from 'node:fs';
import path from 'node:path';

const [tag, repo, dir] = process.argv.slice(2);
if (!tag || !repo || !dir) {
  console.error('usage: updater-manifest.mjs <tag> <repo> <dir>');
  process.exit(2);
}

const files = [];
const walk = (d) => {
  for (const e of fs.readdirSync(d, { withFileTypes: true })) {
    const p = path.join(d, e.name);
    if (e.isDirectory()) walk(p);
    else files.push(p);
  }
};
walk(dir);

/** The bundle ending in `suffix` and its signature. */
const entry = (suffix) => {
  const file = files.find((f) => f.endsWith(suffix));
  if (!file || !fs.existsSync(`${file}.sig`)) {
    console.error(`no signed ${suffix} in ${dir}`);
    process.exit(1);
  }
  const name = path.basename(file);
  return {
    signature: fs.readFileSync(`${file}.sig`, 'utf8').trim(),
    url: `https://github.com/${repo}/releases/download/${tag}/${encodeURIComponent(name)}`,
  };
};

const nsis = entry('-setup.exe');
const msi = entry('.msi');
const appimage = entry('.AppImage');
const manifest = {
  version: tag.replace(/^v/, ''),
  notes: `See https://github.com/${repo}/releases/tag/${tag}`,
  pub_date: new Date().toISOString(),
  // The updater tries {os}-{arch}-{installer} first, then {os}-{arch}.
  platforms: {
    'windows-x86_64-nsis': nsis,
    'windows-x86_64-msi': msi,
    'windows-x86_64': nsis,
    'linux-x86_64-appimage': appimage,
    'linux-x86_64': appimage,
  },
};
process.stdout.write(`${JSON.stringify(manifest, null, 2)}\n`);
