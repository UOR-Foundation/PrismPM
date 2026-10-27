// Cheap shape tests only; actual generated owner/browser execution is separate.
import assert from 'node:assert/strict';
import test from 'node:test';
import {createSessionOperationCapture} from '../../sdk/browser/session-operation-capture.mjs';
import {operationCorpus} from './corpus.mjs';
import {frozenInputs, sourceClosure, assertCapturedOperationSources} from './compile.mjs';

const roles = ['predecessor', 'session', 'observation', 'partition', 'descriptor'];
const options = () => Object.fromEntries(roles.map(role => [role, {bytes: new Uint8Array(8), sha256: new Uint8Array(32)}]));
test('private operation artifact options are closed data and never invoke accessors', async () => {
  for (const value of [null, [], Object.create(null), {...options(), state: {}}, {...options(), authority: true},
    {...options(), success: true}, {...options(), predecessor: {bytes: new Uint8Array(), sha256: new Uint8Array(32), accepted: true}}])
    await assert.rejects(createSessionOperationCapture(value), /invalid-input/);
  let calls = 0;
  for (const depth of ['outer', 'artifact']) {
    const value = options(), target = depth === 'outer' ? value : value.predecessor, key = depth === 'outer' ? 'predecessor' : 'bytes';
    Object.defineProperty(target, key, {get() {calls++; throw Error('accessor executed');}});
    await assert.rejects(createSessionOperationCapture(value), /invalid-input/);
  }
  assert.equal(calls, 0);
  await assert.rejects(createSessionOperationCapture(options(), true), /invalid-input/);
  await assert.rejects(createSessionOperationCapture(options()), /artifact-mismatch/);
});
test('independent operation corpus distinguishes source rejection from complete derived frames', () => {
  const rows = operationCorpus();
  assert.ok(rows.some(row => row.expected.error));
  assert.ok(rows.some(row => row.expected.frames?.before === null));
  assert.ok(rows.some(row => row.expected.frames?.before !== null && row.expected.frames?.before));
  for (const row of rows) if (!row.expected.error) {
    assert.deepEqual(Object.keys(row.expected.frames), ['operation', 'before', 'after', 'beforeObservation', 'afterObservation']);
    for (const [name, bytes] of Object.entries(row.expected.frames))
      assert.equal(row.expected.descriptions[name] === null, bytes === null);
  }
});
test('private owner captures complete source and transitive local harness closure', () => {
  const inputs = frozenInputs(), sources = sourceClosure(); assertCapturedOperationSources(inputs, sources);
  for (const path of ['sdk/browser/session-operation-capture.mjs', 'sdk/browser/session-storage.mjs',
    'sdk/browser/session-payloads.mjs', 'tests/browser-view/kernel-provenance.mjs',
    'tests/browser-view/compiler-owner.mjs', 'tests/browser-view/compiler-artifact.mjs']) assert.ok(inputs[path]);
  sources.delete('Foundation.Bytes');
  assert.throws(() => assertCapturedOperationSources(inputs, sources), /source module inventory/);
});
