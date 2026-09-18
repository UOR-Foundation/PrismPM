import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { appendFileSync, copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

const root = fileURLToPath(new URL('../', import.meta.url));
const helper = 'scripts/fetch-oracle-cargo.sh';
const pinned = [
  'tests/hologram-oracle/Cargo.toml',
  'tests/hologram-oracle/Cargo.lock',
  'tests/holo-codec-oracle/Cargo.toml',
  'tests/holo-codec-oracle/Cargo.lock',
  'tests/browser-workspace/Cargo.toml',
  'tests/browser-workspace/Cargo.lock',
  'tests/browser-envelope/driver/Cargo.toml',
  'tests/browser-envelope/driver/Cargo.lock',
  'tests/browser-journal/driver/Cargo.toml',
  'tests/browser-journal/driver/Cargo.lock',
  'tests/browser-command/driver/Cargo.toml',
  'tests/browser-command/driver/Cargo.lock',
  'tests/browser-query/driver/Cargo.toml',
  'tests/browser-query/driver/Cargo.lock',
  'crates/prismpm/vendor/hologram-live.tar',
];
const embedded = [
  'crates/prismpm/src/embedded/hologram-oracle.Cargo.toml',
  'crates/prismpm/src/embedded/hologram-oracle.Cargo.lock',
  'crates/prismpm/src/embedded/hologram-oracle.main.rs',
];
const check = (directory, args = ['--check']) => spawnSync('bash', [helper, ...args], {
  cwd: directory, encoding: 'utf8', timeout: 10_000,
});

test('oracle acquisition inputs match reviewed pins and embedded verifier inputs', () => {
  const result = check(root);
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stdout + result.stderr);
  assert.equal(check(root, ['--unknown']).status, 64);
  const script = readFileSync(join(root, helper), 'utf8');
  const catalog = readFileSync(join(root, 'model/dependencies.toml'), 'utf8');
  const live = catalog.split('[[dependency]]').find(row => /^id = "hologram-live"$/m.test(row));
  const upstream = catalog.split('[[dependency]]').find(row => /^id = "uor-hologram"$/m.test(row));
  assert.ok(live && upstream);
  const digest = /^sha256 = "([a-f0-9]{64})"$/m.exec(live)?.[1];
  const revision = /^revision = "([a-f0-9]{40})"$/m.exec(upstream)?.[1];
  assert.ok(digest && revision);
  assert.ok(script.includes(`${digest}  crates/prismpm/vendor/hologram-live.tar`));
  for (const oracle of ['hologram-oracle', 'holo-codec-oracle']) {
    assert.ok(readFileSync(join(root, `tests/${oracle}/Cargo.toml`), 'utf8').includes(`rev = "${revision}"`));
    assert.ok(readFileSync(join(root, `tests/${oracle}/Cargo.lock`), 'utf8').includes(`?rev=${revision}#${revision}`));
  }
});

test('changed pinned inputs and embedded divergence fail before Cargo acquisition', () => {
  const scratch = mkdtempSync(join(tmpdir(), 'prismpm-oracle-fetch-'));
  try {
    const files = [helper, ...pinned, ...embedded, 'tests/hologram-oracle/src/main.rs'];
    for (const path of files) {
      mkdirSync(dirname(join(scratch, path)), { recursive: true });
      copyFileSync(join(root, path), join(scratch, path));
    }
    assert.equal(check(scratch).status, 0);
    for (const path of [...pinned, ...embedded]) {
      appendFileSync(join(scratch, path), '\nchanged\n');
      const result = check(scratch);
      assert.ifError(result.error);
      assert.notEqual(result.status, 0, `accepted modified ${path}`);
      copyFileSync(join(root, path), join(scratch, path));
    }
    assert.equal(check(scratch).status, 0);
  } finally {
    rmSync(scratch, { recursive: true });
  }
});

test('source and SDK acquisition share the helper before the SDK cache is frozen', () => {
  const fetch = readFileSync(join(root, 'scripts/fetch.sh'), 'utf8');
  const dockerfile = readFileSync(join(root, 'sdk/Dockerfile'), 'utf8');
  const invocation = `bash ${helper}`;
  assert.equal(fetch.split(invocation).length, 2);
  assert.equal(dockerfile.split(invocation).length, 2);
  const acquisition = dockerfile.indexOf(invocation);
  const snapshot = dockerfile.indexOf('cp -a /usr/local/cargo/registry /opt/prismpm/cargo-home/');
  assert.ok(acquisition >= 0 && snapshot > acquisition);
});
