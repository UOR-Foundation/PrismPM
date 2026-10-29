import { test } from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { HEADERS } from '../lib/changelog.mjs';
import { ROOT } from '../lib/spec.mjs';

const read = (path) => readFileSync(join(ROOT, path), 'utf8');
const config = JSON.parse(read('release-please-config.json'));
const manifest = JSON.parse(read('.release-please-manifest.json'));

test('release-please bumps the minor for breaking changes and the patch for the rest, before 1.0', () => {
  assert.equal(config['bump-minor-pre-major'], true);
  assert.equal(config['bump-patch-for-minor-pre-major'], false);
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

test('the release-please workflow is manual and cannot tag or publish', () => {
  const workflow = read('.github/workflows/release-please.yml');
  const trigger = /^on:\n((?:  .*\n)+)/m.exec(workflow)[1];
  assert.equal(trigger.trim(), 'workflow_dispatch:');
  assert.match(workflow, /skip-github-release: true/);
  assert.doesNotMatch(workflow.replace(/^\s*#.*$/gm, ''), /npm publish|cargo publish|git tag|gh release/);
  assert.match(workflow, /googleapis\/release-please-action@[0-9a-f]{40}/);
});
