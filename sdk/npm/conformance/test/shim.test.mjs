// The prism-code shim: one outcome per step in the contract's shape, the runtime's refusal with its code, and exit 2
// for a step it cannot understand.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { existsSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';
import { OUTCOME_SCHEMA, STEP_SCHEMA, parseOutcome } from '../src/runner.mjs';
import { resolveEntrypoint, loadRuntimes } from '../src/runtimes.mjs';
import { PRISM_CODE, ROOT, SHIM, readJson, scratch } from './helpers.mjs';

const version = readJson(join(ROOT, '..', 'prism-code', 'package.json')).version;
const CALCULATOR = join(ROOT, 'fixtures', 'Calculator.holo');
const HISTORY = join(ROOT, 'fixtures', 'Calculator.history.json');
const NOT_A_HOLO = join(ROOT, 'fixtures', 'negative', 'not-a-history.json');

function shim(step, { env = {} } = {}) {
  const directory = scratch('prism-conformance-shim-');
  const path = join(directory, 'step.json');
  const file = step === null ? null : { schema: STEP_SCHEMA, scenario: 'test', step: 0, scratch: directory, ...step };
  if (file) writeFileSync(path, typeof step === 'string' ? step : JSON.stringify(file));
  const child = spawnSync(process.execPath, step === null ? [SHIM] : [SHIM, path], { encoding: 'utf8', env: { ...process.env, ...env } });
  return { ...child, directory, outcome: child.status === 0 ? parseOutcome(child.stdout).outcome ?? null : null };
}

function expectOutcome(child, exit) {
  assert.equal(child.status, 0, child.stderr);
  assert.ok(child.outcome, child.stdout);
  assert.equal(child.outcome.schema, OUTCOME_SCHEMA);
  assert.deepEqual(child.outcome.runtime, { name: 'prism-code', version });
  assert.equal(child.outcome.exit, exit);
  return child.outcome;
}

test('inspect, vectors, history and replay each answer with the runtime JSON and no error', () => {
  const inspect = expectOutcome(shim({ action: 'inspect', archive: CALCULATOR }), 0);
  assert.equal(inspect.result.schema, 'prismpm/holo-inspection/1');
  assert.equal(inspect.error, null);
  const vectors = expectOutcome(shim({ action: 'vectors', archive: CALCULATOR }), 0);
  assert.deepEqual(vectors.result, { passed: 22, total: 22, failed: 0 });
  assert.equal(vectors.error, null);
  const history = expectOutcome(shim({ action: 'history', archive: CALCULATOR }), 0);
  assert.equal(history.result.schema, 'prism/history/1');
  assert.equal(history.result.events.length, 22);
  const replay = expectOutcome(shim({ action: 'replay', archive: CALCULATOR, histories: [HISTORY] }), 0);
  assert.equal(replay.result.schema, 'prismpm/replay-result/1');
  assert.equal(replay.result.divergenceCount, 0);
  assert.equal(replay.error, null);
});

test('check without a policy judges by the default one; with a policy the shim writes it to scratch and passes it', () => {
  const byDefault = expectOutcome(shim({ action: 'check', archive: CALCULATOR }), 1);
  assert.equal(byDefault.result.schema, 'prism/verdict/1');
  assert.equal(byDefault.result.policy.name, 'default');
  assert.deepEqual(byDefault.error, { code: 'policy.failed', message: 'the policy failed' });
  const child = shim({ action: 'check', archive: CALCULATOR, policy: { minimum: 'P0' } });
  const withPolicy = expectOutcome(child, 0);
  assert.equal(withPolicy.result.passed, true);
  assert.equal(withPolicy.error, null);
  assert.ok(existsSync(join(child.directory, 'policy.json')));
  assert.deepEqual(readJson(join(child.directory, 'policy.json')), { minimum: 'P0' });
});

test('a refusal is reported as the error with the runtime stable code and no result', () => {
  const check = expectOutcome(shim({ action: 'check', archive: NOT_A_HOLO }), 2);
  assert.equal(check.result, null);
  assert.equal(check.error.code, 'archive.magic');
  assert.ok(check.error.message.length > 0);
  const replay = expectOutcome(shim({ action: 'replay', archive: CALCULATOR, histories: [NOT_A_HOLO] }), 2);
  assert.equal(replay.result, null);
  assert.equal(replay.error.code, 'replay.history');
  const inspect = expectOutcome(shim({ action: 'inspect', archive: NOT_A_HOLO }), 2);
  assert.equal(inspect.result, null);
  assert.equal(inspect.error.code, 'archive.magic');
  const missing = expectOutcome(shim({ action: 'inspect', archive: join(ROOT, 'fixtures', 'Missing.holo') }), 2);
  assert.equal(missing.result, null);
  assert.ok(missing.error.message.length > 0);
});

test('a divergent replay answers exit 1 with the result and the divergence as the error', () => {
  const changed = join(ROOT, 'fixtures', 'negative', 'Calculator.changed-model.holo');
  const replay = expectOutcome(shim({ action: 'replay', archive: changed, histories: [HISTORY] }), 1);
  assert.equal(replay.result.divergenceCount, 7);
  assert.equal(replay.error.code, 'replay.divergence');
});

test('an unknown action, a missing step file, a non-JSON step or a wrong schema each exit 2 with no outcome', () => {
  const unknown = shim({ action: 'teleport', archive: CALCULATOR });
  assert.equal(unknown.status, 2);
  assert.match(unknown.stderr, /unknown action "teleport"/);
  assert.equal(unknown.stdout, '');
  const none = shim(null);
  assert.equal(none.status, 2);
  assert.match(none.stderr, /takes one step file/);
  const notJson = shim('not json');
  assert.equal(notJson.status, 2);
  assert.match(notJson.stderr, /not a JSON step file/);
  const directory = scratch();
  const path = join(directory, 'step.json');
  writeFileSync(path, JSON.stringify({ schema: 'other/1', action: 'inspect', archive: CALCULATOR }));
  const wrong = spawnSync(process.execPath, [SHIM, path], { encoding: 'utf8' });
  assert.equal(wrong.status, 2);
  assert.match(wrong.stderr, /must declare schema prism\/conformance-step\/1/);
  const noHistories = shim({ action: 'replay', archive: CALCULATOR });
  assert.equal(noHistories.status, 2);
  assert.match(noHistories.stderr, /replay takes histories/);
});

test('PRISM_CODE names the runtime to drive: a missing path exits 2, the sibling bin answers', () => {
  const missing = shim({ action: 'inspect', archive: CALCULATOR }, { env: { PRISM_CODE: join(ROOT, 'no-such-prism-code.mjs') } });
  assert.equal(missing.status, 2);
  assert.match(missing.stderr, /PRISM_CODE names .* which does not exist/);
  const named = expectOutcome(shim({ action: 'inspect', archive: CALCULATOR }, { env: { PRISM_CODE } }), 0);
  assert.equal(named.result.name, 'Calculator');
});

test('the registered JavaScript entrypoint resolves to this Node and the shim in the package', () => {
  const { runtimes } = loadRuntimes(ROOT);
  assert.deepEqual(runtimes['prism-code'].entrypoint, ['node', 'shims/prism-code.mjs']);
  assert.deepEqual(resolveEntrypoint(ROOT, runtimes['prism-code'].entrypoint), [process.execPath, join(ROOT, 'shims/prism-code.mjs')]);
  assert.deepEqual(resolveEntrypoint(ROOT, runtimes['prism-code'].entrypoint, ['cargo', 'run', '-q', '--']), ['cargo', 'run', '-q', '--']);
  assert.equal(resolveEntrypoint(ROOT, null), null);
  assert.equal(runtimes.prismpm.entrypoint, null);
  assert.equal(runtimes['prism-python'].entrypoint, null);
});
