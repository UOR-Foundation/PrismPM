import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {pathToFileURL} from 'node:url';
import test from 'node:test';

const helper = new URL('../tests/browser-view/prerequisites.mjs', import.meta.url);
function suite(failure, mutant = false) {
  const work = mkdtempSync(join(tmpdir(), 'prismpm-prerequisite-'));
  try {
    const module = join(work, 'prerequisites.mjs');
    const original = readFileSync(helper, 'utf8');
    const source = mutant ? original.replace("  assert.ok(completed, name + ': prerequisite did not complete');", '') : original;
    if (mutant) assert.notEqual(source, original, 'guard omission mutant is applied');
    writeFileSync(module, source, {flag:'wx'});
    const entry = join(work, 'owner.test.mjs');
    writeFileSync(entry, `import assert from 'node:assert/strict';
import test from 'node:test';
import {prerequisite} from ${JSON.stringify(pathToFileURL(module).href)};
test('owning View prerequisite gate', async t => {
  const seen = [];
  for (const name of ['View native', 'View no_std', 'View Wasm', 'Command', 'Query']) {
    const result = await prerequisite(t, name, async () => {
      await Promise.resolve();
      if (name === ${JSON.stringify(failure)}) throw new Error('original compiler failure');
      seen.push(name); return {accepted:name};
    });
    if (name !== ${JSON.stringify(failure)}) assert.deepEqual(result, {accepted:name});
  }
  assert.equal(seen.length, ${failure ? 4 : 5});
  console.log('DEPENDENT_BROWSER_PHASE_EXECUTED');
});
`, {flag:'wx'});
    const env = {...process.env};
    delete env.NODE_TEST_CONTEXT;
    return spawnSync(process.execPath, ['--test', '--test-reporter=tap', '--test-timeout=10000', entry],
      {encoding:'utf8', env, timeout:15000, maxBuffer:1024*1024});
  } finally { rmSync(work, {recursive:true, force:true}); }
}

test('complete successful prerequisites run every check and then the dependent phase', () => {
  const result = suite(null);
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stdout + result.stderr);
  for (const name of ['View native', 'View no_std', 'View Wasm', 'Command', 'Query']) {
    assert.ok(result.stdout.includes('# Subtest: ' + name));
  }
  assert.match(result.stdout, /# tests 6\n/);
  assert.match(result.stdout, /# pass 6\n/);
  assert.match(result.stdout, /# skipped 0\n/);
  assert.match(result.stdout, /DEPENDENT_BROWSER_PHASE_EXECUTED/);
});

test('actual failed Node subtests preserve their cause and refuse the dependent phase', () => {
  for (const name of ['View native', 'View no_std', 'View Wasm', 'Command', 'Query']) {
    const result = suite(name);
    assert.ifError(result.error);
    assert.equal(result.status, 1, result.stdout + result.stderr);
    assert.ok(result.stdout.includes(name + ': prerequisite did not complete'));
    assert.match(result.stdout, /original compiler failure/);
    assert.doesNotMatch(result.stdout, /DEPENDENT_BROWSER_PHASE_EXECUTED|Cannot read properties of undefined/);
  }
});

test('omitting the prerequisite guard is detected even though Node still exits nonzero', () => {
  const result = suite('Query', true);
  assert.ifError(result.error);
  assert.equal(result.status, 1);
  assert.match(result.stdout, /DEPENDENT_BROWSER_PHASE_EXECUTED/);
  assert.doesNotMatch(result.stdout, /prerequisite did not complete/);
});
