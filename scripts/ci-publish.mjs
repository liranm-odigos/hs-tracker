// CI: cut a GitHub Release from the artifacts the workflow just built.
//
// This fork ships the Windows installer only. latest.json is written when
// that installer was signed.
//
//   node scripts/ci-publish.mjs              # reads release/
//   node scripts/ci-publish.mjs --dry        # say what would happen

import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const dry = process.argv.includes('--dry');
const dir = join(root, 'release');

const run = (file, argv, opts = {}) =>
  execFileSync(file, argv, { cwd: root, encoding: 'utf8', ...opts });

function die(why) {
  console.error(`\n  ${why}\n`);
  process.exit(1);
}

const version = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8')).version;
const tag = process.env.GITHUB_REF_NAME || `v${version}`;
if (tag.replace(/^v/, '') !== version) {
  die(`tag ${tag} does not match package.json ${version}`);
}

if (!existsSync(dir)) die('nothing in release/. The workflow copies the artifacts there first.');
const files = readdirSync(dir).filter((n) => n.includes(version));
if (!files.length) die(`release/ holds nothing for ${version}`);

const WANTED = ['.exe'];
const missing = WANTED.filter((ext) => !files.some((n) => n.endsWith(ext)));
if (missing.length) die(`release/ is missing the Windows installer for ${version}`);

const changelog = readFileSync(join(root, 'CHANGELOG.md'), 'utf8').split(/\r?\n/);
const first = changelog.findIndex((l) => l.startsWith('## '));
if (first < 0) die('CHANGELOG.md has no section to cut notes from');
let last = changelog.findIndex((l, i) => i > first && l.startsWith('## '));
if (last < 0) last = changelog.length;
const notes = changelog.slice(first, last).join('\n').trim();
if (!notes.includes(version)) {
  die(`CHANGELOG.md opens with "${changelog[first].trim()}", which is not ${version}`);
}

const TARGETS = [
  ['windows-x86_64', (n) => n.endsWith('-setup.exe')],
];
const slug = process.env.GITHUB_REPOSITORY || run('gh', ['repo', 'view', '--json', 'nameWithOwner', '-q', '.nameWithOwner']).trim();
const platforms = {};
const unsigned = [];
for (const [target, wanted] of TARGETS) {
  const name = files.find((n) => wanted(n));
  if (!name) continue;
  const sig = join(dir, name + '.sig');
  if (!existsSync(sig)) {
    unsigned.push(name);
    continue;
  }
  platforms[target] = {
    signature: readFileSync(sig, 'utf8').trim(),
    url: `https://github.com/${slug}/releases/download/${tag}/${name.replace(/ /g, '.')}`,
  };
}
const manifest = { version, notes, pub_date: new Date().toISOString(), platforms };
const manifestPath = join(dir, 'latest.json');

console.log(`\n  ${tag}\n`);
for (const name of files.sort()) console.log(`    ${name}`);
console.log(`\n    updates  ${Object.keys(platforms).join(', ') || 'nothing was signed'}`);
if (unsigned.length) console.log(`    unsigned ${unsigned.join(', ')}`);
console.log(`\n    notes    ${changelog[first].trim()}\n`);

if (dry) {
  console.log('  --dry, so nothing was done.\n');
  process.exit(0);
}

const notesPath = join(root, 'RELEASE_NOTES.md');
writeFileSync(notesPath, notes + '\n');
if (Object.keys(platforms).length) {
  writeFileSync(manifestPath, JSON.stringify(manifest, null, 2) + '\n');
}

const paths = files.filter((n) => !n.endsWith('.sig')).map((n) => join(dir, n));
if (Object.keys(platforms).length) paths.push(manifestPath);

const exists = (() => {
  try {
    run('gh', ['release', 'view', tag], { stdio: 'ignore' });
    return true;
  } catch {
    return false;
  }
})();

if (exists) {
  console.log('  the release is already there; replacing assets from this run\n');
  run('gh', ['release', 'upload', tag, ...paths, '--clobber'], { stdio: 'inherit' });
} else {
  run(
    'gh',
    ['release', 'create', tag, ...paths, '--title', version, '--notes-file', notesPath],
    { stdio: 'inherit' },
  );
}

console.log(`\n  ${run('gh', ['release', 'view', tag, '--json', 'url', '-q', '.url']).trim()}\n`);
