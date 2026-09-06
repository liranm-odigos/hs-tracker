// CI: allow `tauri build` to produce installers when this fork has no
// updater signing key.
//
// Upstream signs every bundle. The private half of that key never left the
// maintainer's machine, so a fork's runners cannot. Without a key, Tauri
// refuses to build at all if createUpdaterArtifacts is on. Turning that off
// here is the difference between an unsigned installer and no installer.
//
// When TAURI_SIGNING_PRIVATE_KEY is set (a repo secret), this does nothing
// and the build signs as usual. Empty password is a password; unset is a
// prompt nobody is there to answer.

import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const conf = join(root, 'src-tauri', 'tauri.conf.json');

if (process.env.TAURI_SIGNING_PRIVATE_KEY) {
  if (process.env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD == null) {
    process.env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD = '';
  }
  console.log('signing key present; updater artifacts will be produced');
  process.exit(0);
}

const before = readFileSync(conf, 'utf8');
const after = before.replace(/"createUpdaterArtifacts"\s*:\s*true/, '"createUpdaterArtifacts": false');
if (after === before) {
  console.log('no signing key; createUpdaterArtifacts was already off');
} else {
  writeFileSync(conf, after);
  console.log('no signing key; building unsigned installers (auto-update will not apply them)');
}
