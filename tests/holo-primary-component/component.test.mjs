import assert from 'node:assert/strict';
import test from 'node:test';
import {readFileSync} from 'node:fs';
import {join} from 'node:path';
import {repository, sourceClosure, frozenInputs, requirePreparedComponent, run, verifyFrozenInputs} from './compile.mjs';
import {mutations, mutateSource, mutationProbes} from './mutations.mjs';
import {inputClosureAdversaries} from './adversaries.mjs';
import {profileLinkCases} from './profile-links.mjs';

test('component uses exact source modules and cannot adopt a caller receipt', () => {
  const sources = sourceClosure();
  assert.equal(sources.size, 12);
  assert.ok(sources.has('Foundation.Holo.V1.PrimaryWire'));
  assert.ok(sources.has('Foundation.Browser.Application.V1.SessionWire'));
  assert.throws(() => requirePreparedComponent({verified:true,wasm:new Uint8Array()}), /actual fresh owning compiler/);
});

test('seven actual source mutants change only the selected PrimaryWire module', () => {
  assert.equal(mutations.length, 7);
  for (const mutation of mutations) {
    const before = sourceClosure(), changed = sourceClosure();
    assert.equal(mutateSource(changed, mutation.id), mutation);
    assert.deepEqual([...changed.keys()], [...before.keys()]);
    let modified = 0;
    for (const [name, bytes] of changed) {
      if (!bytes.equals(before.get(name))) {assert.equal(name, 'Foundation.Holo.V1.PrimaryWire'); modified++;}
    }
    assert.equal(modified, 1);
  }
});

test('complete mutation-probe inventory refuses missing and duplicate probes before compilation', () => {
  const rows = mutations.map(mutation => ({id: mutation.probe}));
  assert.deepEqual(mutationProbes(rows), rows);
  for (const row of rows) {
    assert.throws(() => mutationProbes(rows.filter(value => value !== row)), /exact codec probe/);
    assert.throws(() => mutationProbes([...rows, row]), /exact codec probe/);
  }
  assert.equal(mutations.find(row => row.id === 'sorted-blobs').probe, 'BlobOrder102');
});

test('captured graph rejects five actual transitive defects without writing the source tree', () => {
  const inputs = frozenInputs();
  const observed = inputClosureAdversaries(inputs);
  assert.equal(observed.cases.length, 5);
  assert.deepEqual(observed.inputs, inputs);
  verifyFrozenInputs(inputs);
});

test('private substitution entry refuses direct invocation against repository or SDK inputs', () => {
  const inputs = frozenInputs();
  assert.throws(() => run(process.execPath,
    [join(repository, 'tests/holo-primary-component/adversaries.mjs'), '--private-input-checks'], repository),
  /private input-copy parent/);
  verifyFrozenInputs(inputs);
});

test('owning input inventory includes external oracle, generated artifact guards and whole dynamic entry closure', () => {
  const inputs = frozenInputs();
  for (const path of ['tests/holo-primary-component/owner.test.mjs', 'tests/holo-primary-component/checks.mjs',
    'tests/holo-primary-component/oracle.rs', 'vendor/hologram-live.tar', 'tests/hologram-oracle/Cargo.lock',
    'tests/holo-primary-component/profile-links.mjs',
    'tests/browser-view/generated-package.mjs', 'tests/browser-view/generated-wasm.mjs',
    'tests/browser-session-journal/runtime.mjs', 'tests/browser-effects/corpus.mjs'])
    assert.match(inputs[path], /^[a-f0-9]{64}$/, path);
  const source = readFileSync(join(repository, 'tests/holo-primary-component/oracle.rs'), 'utf8');
  assert.match(source, /catalog\.import\("source-session\.holo"\.into\(\), archive\.clone\(\)\)/);
});

test('independent profile-link negative inventory retains all reference and source seams', () => {
  assert.equal(profileLinkCases.length,24);
  assert.equal(new Set(profileLinkCases.map(row=>row.id)).size,24);
  for(const expected of ['model_content_kappa','guest_content_kappa','capabilities_content_kappa',
    'application_kappa','source equality','source facts','module input closure','IR equality',
    'package closure equality','metadata shape','provenance shape','authority revisions','guest SHA-256'])
    assert.ok(profileLinkCases.some(row=>row.expected===expected),expected);
});
