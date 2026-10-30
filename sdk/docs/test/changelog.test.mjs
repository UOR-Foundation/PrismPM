import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { entryFor, normalizeNewestEntry, parseChangelog, releaseEntryProblems as problemsWith, LEGACY_THROUGH } from '../lib/changelog.mjs';
import { ROOT } from '../lib/spec.mjs';

const command = fileURLToPath(new URL('../changelog.mjs', import.meta.url));

const LEGACY = '## [0.4.0] - 2026-10-27\n\n### Added\n- Old.\n\n### Breaking Changes\n- Old break.\n\n### Compatibility\n- Kept.\n\n';
// These fixtures hold the legacy cutoff at 0.4.0, so each entry below is judged against one before it.
const releaseEntryProblems = (text, legacyThrough = '0.4.0') => problemsWith(text, legacyThrough);
const messages = (text) => releaseEntryProblems(text).map((p) => p.message);

test('the repository CHANGELOG.md meets STABILITY.md section 2', () => {
  const text = readFileSync(join(ROOT, 'CHANGELOG.md'), 'utf8');
  assert.deepEqual(problemsWith(text), []);
  assert.equal(LEGACY_THROUGH, '0.1.0');
  assert.ok(parseChangelog(text).some((e) => e.version === '0.4.0'), 'the 0.4.0 entry is still there');
});

test('entries up to the cutoff keep the headers they were published with', () => {
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
    writeFileSync(file, '# Changelog\n\n' + LEGACY.replace('0.4.0', LEGACY_THROUGH));
    assert.equal(spawnSync(process.execPath, [command, file], { encoding: 'utf8' }).status, 0);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

const HEADING = '## [0.5.0](https://github.com/UOR-Foundation/PrismPM/compare/v0.4.0...v0.5.0) (2026-11-20)';

test('normalization moves the breaking section release-please writes first, and changes no bullet', () => {
  const written = `# Changelog\n\n${HEADING}\n\n\n### ⚠ BREAKING CHANGES\n\n* **holo:** a break\n\n### Features Added\n\n* **holo:** a break ([abc1234](https://example.test))\n\n### Bugs Fixed\n\n* a fix\n\n${LEGACY}`;
  const normalized = normalizeNewestEntry(written);
  assert.deepEqual(releaseEntryProblems(normalized), []);
  const [entry] = parseChangelog(normalized);
  assert.deepEqual(entry.sections.map((s) => s.name), ['Features Added', 'Breaking Changes', 'Bugs Fixed']);
  for (const bullet of ['* **holo:** a break\n', '* **holo:** a break ([abc1234](https://example.test))\n', '* a fix\n']) assert.ok(normalized.includes(bullet), bullet);
  assert.ok(normalized.endsWith(LEGACY), 'older entries are untouched');
});

test('normalization says None. when release-please wrote no breaking section', () => {
  const written = `## [0.4.1](https://example.test) (2026-11-02)\n\n### Bugs Fixed\n\n* a fix\n\n${LEGACY}`;
  const normalized = normalizeNewestEntry(written);
  assert.deepEqual(releaseEntryProblems(normalized), []);
  assert.deepEqual(parseChangelog(normalized)[0].sections.map((s) => [s.name, s.empty]), [['Breaking Changes', true], ['Bugs Fixed', false]]);
  assert.match(normalized, /### Breaking Changes\n\nNone\.\n\n### Bugs Fixed/);
});

test('normalization is idempotent, keeps line endings and leaves a correct entry byte for byte', () => {
  const written = `${HEADING}\n\n### ⚠ BREAKING CHANGES\n\n* a break\n\n${LEGACY}`;
  const once = normalizeNewestEntry(written);
  assert.equal(normalizeNewestEntry(once), once);
  const crlf = written.replaceAll('\n', '\r\n');
  assert.equal(normalizeNewestEntry(crlf), once.replaceAll('\n', '\r\n'));
  const repository = readFileSync(join(ROOT, 'CHANGELOG.md'), 'utf8');
  assert.equal(normalizeNewestEntry(repository), repository);
  assert.equal(normalizeNewestEntry('# Changelog\n'), '# Changelog\n');
});

test('normalization leaves an unknown header for the check to name', () => {
  const normalized = normalizeNewestEntry(`## [0.4.1] - 2026-11-02\n\n### Added\n- Old.\n\n${LEGACY}`);
  assert.ok(messages(normalized).some((m) => /### Added is not one of/.test(m)));
});

test('the command normalizes a file in place', () => {
  const dir = mkdtempSync(join(tmpdir(), 'prism-changelog-'));
  try {
    const file = join(dir, 'CHANGELOG.md');
    writeFileSync(file, `# Changelog\n\n${HEADING.replaceAll('0.5.0', '0.1.1').replaceAll('0.4.0', '0.1.0')}\n\n### Bugs Fixed\n\n* a fix\n\n${LEGACY.replace('0.4.0', LEGACY_THROUGH)}`);
    assert.equal(spawnSync(process.execPath, [command, file], { encoding: 'utf8' }).status, 1);
    assert.equal(spawnSync(process.execPath, [command, '--normalize', file], { encoding: 'utf8' }).status, 0);
    const result = spawnSync(process.execPath, [command, file], { encoding: 'utf8' });
    assert.equal(result.status, 0, result.stderr);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('the release body is one version entry with its heading', () => {
  const text = `# Changelog\n\n## Policy\n\n### Compatibility\n- Kept.\n\n${HEADING}\n\n### Breaking Changes\n\n* a break\n\n${LEGACY}`;
  assert.equal(entryFor(text, '0.5.0'), `${HEADING}\n\n### Breaking Changes\n\n* a break\n`);
  assert.match(entryFor(text, '0.4.0'), /^## \[0\.4\.0\] - 2026-10-27\n/);
  assert.equal(entryFor(text, '9.9.9'), null);
  assert.equal(entryFor(text.replaceAll('\n', '\r\n'), '0.5.0'), `${HEADING}\n\n### Breaking Changes\n\n* a break\n`);
});
