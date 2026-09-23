import assert from 'node:assert/strict';
import test from 'node:test';
import {join} from 'node:path';
import {repository, run} from '../browser-view/compile.mjs';
import {verifyLiveBoundary} from './live.mjs';

test('both immutable codec corpora exactly match the pinned upstream implementation', () => {
  const output = run('cargo', ['run', '--locked', '--offline', '--manifest-path',
    'tests/holo-codec-oracle/Cargo.toml'], repository,
  {CARGO_TARGET_DIR: join(repository, 'target/holo-codec-oracle')});
  assert.match(output, /codec fixture matches upstream 2bda6a9a9476872dade705bd61ece4209607f6da/);
});

test('all generated browser codec tests execute in both std and no_std', () => {
  for (const features of [[], ['--no-default-features']]) {
    const output = run('cargo', ['test', '--locked', '--offline', '--manifest-path',
      'tests/holo-browser-codec/Cargo.toml', ...features, '--', '--nocapture'], repository,
    {CARGO_TARGET_DIR: join(repository, 'target/holo-browser-codec')});
    assert.match(output, /test result: ok\. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;/);
    assert.equal((output.match(/^test [a-z_]+ \.\.\. ok$/gm) ?? []).length, 6);
  }
});

test('pinned Live rejects the browser selector before View attachment', verifyLiveBoundary);
