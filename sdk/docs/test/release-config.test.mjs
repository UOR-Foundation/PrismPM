import { test } from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { HEADERS } from '../lib/changelog.mjs';
import { ROOT } from '../lib/spec.mjs';
import { releasedPackages } from '../../../scripts/release-please-notes.mjs';

const read = (path) => readFileSync(join(ROOT, path), 'utf8').split('\r\n').join('\n');
const config = JSON.parse(read('release-please-config.json'));
const manifest = JSON.parse(read('.release-please-manifest.json'));
const workflow = read('.github/workflows/release-please.yml');

/** One top-level job of the workflow, as text. */
const job = (name) => new RegExp(`^  ${name}:\\n((?:(?:    .*)?\\n)+)`, 'm').exec(workflow)[1];
const uncommented = (text) => text.replace(/^\s*#.*$/gm, '');

test('below 1.0 release-please bumps the minor for breaking changes and the patch for the rest', () => {
  assert.equal(config['bump-minor-pre-major'], true);
  assert.equal(config['bump-patch-for-minor-pre-major'], true);
});

test('every changelog section release-please writes is one of the four headers', () => {
  const sections = new Set(config['changelog-sections'].map((s) => s.section));
  assert.ok(sections.size > 0);
  for (const section of sections) assert.ok(HEADERS.includes(section), `${section} is not a STABILITY.md header`);
  assert.ok(config['changelog-sections'].every((s) => !s.hidden), 'a hidden type would drop changes from the log');
});

test('the manifest and the config name the same packages, and each exists', () => {
  assert.deepEqual(Object.keys(manifest).sort(), Object.keys(config.packages).sort());
  for (const [path, settings] of Object.entries(config.packages)) {
    assert.ok(existsSync(join(ROOT, path)), `${path} does not exist`);
    if (settings['release-type'] === 'node') assert.ok(existsSync(join(ROOT, path, 'package.json')), `${path} has no package.json`);
    for (const file of settings['extra-files'] ?? []) assert.ok(existsSync(join(ROOT, file.path)), `${file.path} does not exist`);
  }
});

test('the root tag stays v<version>, which release.yml reads, and npm tags carry the component', () => {
  assert.equal(config['include-v-in-tag'], true);
  assert.equal(config.packages['.']['include-component-in-tag'], false);
  assert.ok(read('.github/workflows/release.yml').includes('tags:'));
  assert.ok(read('.github/workflows/npm-prism-code.yml').includes('prism-code-v*'));
  assert.equal(config.packages['sdk/npm/prism-code'].component, 'prism-code');
});

test('the workflow runs by hand, and on main only when the manifest changes', () => {
  const trigger = /^on:\n((?:  .*\n)+)/m.exec(workflow)[1];
  assert.match(trigger, /^  workflow_dispatch:\n/m);
  assert.match(trigger, /^  push:\n    branches: \[main\]\n    paths:\n      - \.release-please-manifest\.json\n/m);
  assert.doesNotMatch(trigger, /pull_request|schedule/);
});

test('no job can publish, and every action is pinned', () => {
  assert.doesNotMatch(uncommented(workflow), /npm publish|cargo publish|git tag|git push --tags|docker push|cosign/);
  for (const [, ref] of workflow.matchAll(/uses:\s*\S+?@(\S+)/g)) assert.match(ref, /^[0-9a-f]{40}$/);
  assert.match(workflow, /googleapis\/release-please-action@[0-9a-f]{40}/);
});

test('the tag job uses GITHUB_TOKEN alone, so its tag cannot start the npm publish workflow', () => {
  const tag = uncommented(job('tag'));
  assert.doesNotMatch(tag, /secrets\./);
  assert.match(tag, /token: \$\{\{ github\.token \}\}/);
  assert.match(tag, /skip-github-pull-request: true/);
  assert.doesNotMatch(tag, /skip-github-release/);
});

test('the open-pr job never tags, and leaves each pull request passing the changelog check', () => {
  const openPr = uncommented(job('open-pr'));
  assert.match(openPr, /skip-github-release: true/);
  assert.match(openPr, /changelog\.mjs --normalize/);
  assert.match(openPr, /cargo update --workspace/);
  assert.match(openPr, /node sdk\/docs\/changelog\.mjs\n/);
});

test('the dry run can only read', () => {
  const dryRun = uncommented(job('dry-run'));
  assert.match(dryRun, /contents: read/);
  assert.doesNotMatch(dryRun, /: write|secrets\.|release-please-action/);
  assert.match(dryRun, /npm ci/);
});

test('the dry run pins release-please at the version the pinned action bundles', () => {
  const lock = JSON.parse(read('scripts/release-please-dry-run/package-lock.json'));
  assert.equal(lock.packages['node_modules/release-please'].version, '17.1.3');
  assert.match(workflow, /release-please-action@16a9c90856f42705d54a6fda1823352bdc62cf38 # v4\.4\.0/);
});

test('a GitHub Release that release-please created keeps its body when release.yml adds the assets', () => {
  const step = /softprops\/action-gh-release@[\s\S]*?files: \|/.exec(read('.github/workflows/release.yml'))[0];
  assert.match(step, /append_body: true/);
  assert.match(step, /body_path: RELEASE_NOTES\.md/);
});

test('npm can be published from a tag that started no workflow, and a prerelease never takes latest', () => {
  const npm = read('.github/workflows/npm-prism-code.yml');
  assert.match(npm, /^  workflow_dispatch:$/m);
  assert.match(npm, /if: startsWith\(github\.ref, 'refs\/tags\/prism-code-v'\)\n    runs-on/);
  assert.match(npm, /--tag next/);
});

test('the action outputs name a root release and a package release differently', () => {
  assert.deepEqual(releasedPackages({
    paths_released: '[".","sdk/npm/prism-code"]',
    tag_name: 'v0.4.1', version: '0.4.1',
    'sdk/npm/prism-code--tag_name': 'prism-code-v0.3.1', 'sdk/npm/prism-code--version': '0.3.1',
  }), [
    { path: '.', tag: 'v0.4.1', version: '0.4.1', changelog: 'CHANGELOG.md' },
    { path: 'sdk/npm/prism-code', tag: 'prism-code-v0.3.1', version: '0.3.1', changelog: 'sdk/npm/prism-code/CHANGELOG.md' },
  ]);
  assert.deepEqual(releasedPackages({ paths_released: '[]' }), []);
  assert.deepEqual(releasedPackages({}), []);
  assert.throws(() => releasedPackages({ paths_released: '["."]' }), /without a tag/);
});
