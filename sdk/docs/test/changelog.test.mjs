import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseChangelog, releaseEntryProblems } from '../lib/changelog.mjs';
import { ROOT } from '../lib/spec.mjs';

const command = fileURLToPath(new URL('../changelog.mjs', import.meta.url));

const LEGACY = '## [0.4.0] - 2026-10-27\n\n### Added\n- Old.\n\n### Breaking Changes\n- Old break.\n\n### Compatibility\n- Kept.\n\n';
const messages = (text) => releaseEntryProblems(text).map((p) => p.message);

test('the repository CHANGELOG.md meets STABILITY.md section 2', () => {
  const text = readFileSync(join(ROOT, 'CHANGELOG.md'), 'utf8');
  assert.deepEqual(releaseEntryProblems(text), []);
  assert.ok(parseChangelog(text).some((e) => e.version === '0.4.0'), 'the 0.4.0 entry is still there');
});

test('entries up to 0.4.0 keep the headers they were published with', () => {
  assert.deepEqual(releaseEntryProblems(`# Changelog\n\n${LEGACY}`), []);
});

test('a new entry with the four headers in order passes', () => {
  const text = '## [0.5.0] - 2026-11-20\n\n### Features Added\n- New.\n\n### Breaking Changes\n- A break.\n\n### Bugs Fixed\n- Fix.\n\n### Other Changes\n- Misc.\n\n' + LEGACY;
  assert.deepEqual(releaseEntryProblems(text), []);
});

test('a new entry may use only the headers it needs, and Breaking Changes is always there', () => {
  const ok = '## [0.4.1] - 2026-11-02\n\n### Breaking Changes\nNone.\n\n### Bugs Fixed\n- Fix.\n\n' + LEGACY;
  assert.deepEqual(releaseEntryProblems(ok), []);
  const missing = '## [0.4.1] - 2026-11-02\n\n### Bugs Fixed\n- Fix.\n\n' + LEGACY;
  assert.match(messages(missing)[0], /0\.4\.1 has no ### Breaking Changes/);
});

test('Keep a Changelog headers, repeats and a wrong order are named', () => {
  const text = '## [0.5.0] - 2026-11-20\n\n### Added\n- New.\n\n### Bugs Fixed\n- Fix.\n\n### Breaking Changes\n- A break.\n\n### Bugs Fixed\n- Again.\n\n' + LEGACY;
  const found = messages(text);
  assert.ok(found.some((m) => /### Added is not one of/.test(m)));
  assert.ok(found.some((m) => /### Breaking Changes is out of order/.test(m)));
  assert.ok(found.some((m) => /### Bugs Fixed appears twice/.test(m)));
});

test('an Unreleased entry follows the headers but needs no Breaking Changes yet', () => {
  assert.deepEqual(releaseEntryProblems('## [Unreleased]\n\n### Features Added\n- Soon.\n\n' + LEGACY), []);
  assert.match(messages('## [Unreleased]\n\n### Fixed\n- Soon.\n\n' + LEGACY)[0], /### Fixed is not one of/);
});

test('before 1.0 breaking changes need a minor bump and everything else a patch bump', () => {
  const patchBreaks = '## [0.4.1] - 2026-11-02\n\n### Breaking Changes\n- A break.\n\n' + LEGACY;
  assert.match(messages(patchBreaks)[0], /lists breaking changes, so before 1\.0 it must be a minor bump over 0\.4\.0/);
  const minorQuiet = '## [0.5.0] - 2026-11-20\n\n### Breaking Changes\nNone.\n\n' + LEGACY;
  assert.match(messages(minorQuiet)[0], /breaks nothing, so before 1\.0 it must be a patch bump over 0\.4\.0/);
  const skipped = '## [0.6.0] - 2026-11-20\n\n### Breaking Changes\n- A break.\n\n' + LEGACY;
  assert.match(messages(skipped)[0], /must be a minor bump over 0\.4\.0/);
});

test('release-please entries are read: link heading, dated in parentheses, BREAKING CHANGES named', () => {
  const written = '## [0.5.0](https://github.com/UOR-Foundation/PrismPM/compare/v0.4.0...v0.5.0) (2026-11-20)\n\n### ⚠ BREAKING CHANGES\n\n* a break\n\n### Features Added\n\n* new\n\n' + LEGACY;
  assert.deepEqual(parseChangelog(written).map((e) => [e.version, e.date]), [['0.5.0', '2026-11-20'], ['0.4.0', '2026-10-27']]);
  const found = messages(written);
  assert.ok(found.some((m) => /### ⚠ BREAKING CHANGES is not one of .*rename it to ### Breaking Changes and move it after ### Features Added/.test(m)));
  const fixed = '## [0.5.0](https://github.com/UOR-Foundation/PrismPM/compare/v0.4.0...v0.5.0) (2026-11-20)\n\n### Features Added\n\n* new\n\n### Breaking Changes\n\n* a break\n\n' + LEGACY;
  assert.deepEqual(releaseEntryProblems(fixed), []);
});

test('a package changelog has no legacy entries', () => {
  const text = '# Changelog\n\n## [0.3.0](https://example.test) (2026-11-20)\n\n### Added\n\n* old header\n\n';
  assert.deepEqual(releaseEntryProblems(text), []);
  assert.match(releaseEntryProblems(text, '0.0.0').map((p) => p.message)[0], /### Added is not one of/);
});

test('a released entry needs a date', () => {
  assert.match(messages('## [0.4.1]\n\n### Breaking Changes\nNone.\n\n' + LEGACY)[0], /0\.4\.1 has no date/);
});

test('the command exits 1 naming file and line', () => {
  const dir = mkdtempSync(join(tmpdir(), 'prism-changelog-'));
  try {
    const file = join(dir, 'CHANGELOG.md');
    writeFileSync(file, '# Changelog\n\n## [0.5.0] - 2026-11-20\n\n### Added\n- New.\n\n### Breaking Changes\n- A break.\n\n' + LEGACY);
    const result = spawnSync(process.execPath, [command, file], { encoding: 'utf8' });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /CHANGELOG\.md:5: 0\.5\.0: ### Added is not one of/);
    writeFileSync(file, '# Changelog\n\n' + LEGACY);
    assert.equal(spawnSync(process.execPath, [command, file], { encoding: 'utf8' }).status, 0);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
