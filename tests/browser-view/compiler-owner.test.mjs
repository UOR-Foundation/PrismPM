import assert from 'node:assert/strict';
import test from 'node:test';
import {captureCompilerInputs, createCompilerOwner, requireCompilerOwner} from './compiler-owner.mjs';

test('compiler owner refuses forged handles and unregistered tool families', () => {
  for (const value of [null, {}, {runDriver() {}, runExporter() {}, verify() {}}])
    assert.throws(() => requireCompilerOwner(value, {}), /actual fresh compiler owner/);
  for (const name of ['', '../escape', 'caller-driver', null, {toString() {throw Error('coercion');}}])
    assert.throws(() => createCompilerOwner(name, {}), /registered compiler family/);
});

test('compiler owner refuses malformed or incomplete input closures before building', () => {
  let called = false;
  const accessor = {get malicious() {called = true; return '0'.repeat(64);}};
  for (const inputs of [null, [], accessor, {'../escape': '0'.repeat(64)},
    {'vendor/lexlean/MANIFEST.sha256': 'invalid'}, {'a': null}, {'a': 42}, {}])
    assert.throws(() => createCompilerOwner('view', inputs),
      /compiler input (map|path|digest|closure)/);
  assert.equal(called, false, 'input accessors cannot run before immutable capture');
  for (const name of ['view', 'effects']) {
    const inputs = captureCompilerInputs(name);
    for (const missing of ['tests/browser-view/compile.mjs', 'tests/browser-view/compiler-artifact.mjs', 'vendor/lean4-prod/lean.tar']) {
      const changed = {...inputs}; delete changed[missing];
      assert.throws(() => createCompilerOwner(name, changed), /complete closed compiler input closure/);
    }
    assert.throws(() => createCompilerOwner(name, {...inputs, unexpected: '0'.repeat(64)}), /complete closed compiler input closure/);
  }
});
