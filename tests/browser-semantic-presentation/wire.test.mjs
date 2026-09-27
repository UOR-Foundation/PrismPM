// Component-only evidence. Generated source/native/Wasm parity is mandatory
// in the owning gate; these independently constructed inputs cannot replace it.
import assert from 'node:assert/strict';
import test from 'node:test';
import {readFileSync} from 'node:fs';
import {join} from 'node:path';
import {frozenInputs, assertFrozenInputs, assertBaselineSources, prepare, modules, repository, sha} from './compile.mjs';
import {sourceRoots} from '../../scripts/browser-api-sdk-check.mjs';
import {PresentationError, encodeWire} from '../../sdk/browser/presentation-wire.mjs';
import {captureDesignCatalogue, decodeSemanticPresentation, encodeSemanticPresentation,
  semanticCatalogueFits, semanticMainNode, semanticProgressFits} from '../../sdk/browser/semantic-presentation-wire.mjs';
import {corpus, fixture, designs, light} from './corpus.mjs';
import {semanticStyle} from '../../sdk/browser/semantic-presentation-style.mjs';
import {verifyNativeInventory} from './checks.mjs';

test('whole-owner closure binds every source, compiler input and exact mutation baseline', () => {
  const inputs = frozenInputs(), sources = new Map();
  for (const path of Object.keys(inputs)) assert.ok(sourceRoots.some(root => path === root || path.startsWith(root + '/')),
    'installed SDK closure covers every owning input: ' + path);
  for (const module of modules) {
    const path = (module === 'Fixture' ? 'tests/browser-semantic-presentation' : 'stdlib')
      + '/src/' + module.replaceAll('.', '/') + '.lex.tex';
    const bytes = readFileSync(join(repository, path));
    assert.equal(inputs[path], sha(bytes), path); sources.set(module, bytes);
  }
  for (const path of ['model/authorities.toml', 'model/dependencies.toml', 'lean-toolchain',
    'rust-toolchain.toml', 'LICENSE-MIT', 'LICENSE-APACHE', 'vendor/lean4-prod/lean.tar',
    'tests/fixtures/library/native-library/project/lexlean.toml', 'sdk/browser/presentation-dom.mjs',
    'sdk/oracles/package-lock.json', 'tests/browser-view/compile.mjs', 'tests/browser-view/driver-cache.mjs',
    'tests/browser-view/generated-package.mjs', 'tests/browser-view/generated-package.test.mjs']) {
    assert.equal(inputs[path], sha(readFileSync(join(repository, path))), path);
  }
  for (const tree of ['vendor/lexlean', 'vendor/lean4-prod/rust']) {
    const rows = readFileSync(join(repository, tree, 'MANIFEST.sha256'), 'utf8').trimEnd().split('\n');
    for (const row of rows) {
      const match = /^([0-9a-f]{64})  (.+)$/.exec(row); assert.ok(match);
      assert.equal(inputs[tree + '/' + match[2]], match[1]);
    }
    const path = tree + '/' + /^([0-9a-f]{64})  (.+)$/.exec(rows[0])[2];
    assert.throws(() => prepare(null, null, {...inputs, [path]: '0'.repeat(64)}), /complete frozen semantic presentation owner inputs/);
    const missing = {...inputs}; delete missing[path];
    assert.throws(() => assertFrozenInputs(missing), /complete frozen semantic presentation owner inputs/);
  }
  assertBaselineSources(sources, new Map(sources));
  assert.throws(() => assertBaselineSources(sources, null), /positive source closure/);
  for (const module of modules) {
    const absent = new Map(sources); absent.delete(module);
    assert.throws(() => assertBaselineSources(absent, sources), /module inventory/);
    const changed = new Map(sources); changed.set(module, Buffer.concat([sources.get(module), Buffer.from('\n')]));
    assert.throws(() => assertBaselineSources(changed, sources), /immutable positive source/);
  }
});

test('every independent semantic wire corpus row is accepted or rejected', () => {
  const rows = [{id: 'first-row'}, {id: 'second-row'}], output = 'PASS first-row\nPASS second-row\nPASS 2 complete semantic vectors twice\n';
  verifyNativeInventory(output, rows);
  for (const changed of ['', output.replace('PASS first-row\n', ''), output + 'PASS extra\n',
    output.replace('PASS second-row', 'PASS first-row'), 'PASS 2 complete semantic vectors twice\n'])
    assert.throws(() => verifyNativeInventory(changed, rows));
  const seen = new Set();
  for (const {id, request, response} of corpus()) {
    assert.ok(!seen.has(id)); seen.add(id);
    if (Buffer.from(request).equals(Buffer.from(response))) {
      const decoded = decodeSemanticPresentation(request);
      assert.deepEqual(encodeSemanticPresentation(decoded), request, id);
      assert.ok(Object.isFrozen(decoded) && Object.isFrozen(decoded[1][6]), id);
    } else assert.throws(() => decodeSemanticPresentation(request), PresentationError, id);
  }
  assert.equal(seen.size, 150);
});

test('catalogue authority is independent of frame-local reference bounds', () => {
  assert.equal(semanticCatalogueFits(fixture(), 10, 1), true);
  assert.equal(semanticCatalogueFits(fixture(), 8, 1), false);
  assert.equal(semanticCatalogueFits(fixture(), 9, 1), false, 'base heading caption is catalogue-bound');
  assert.equal(semanticCatalogueFits(fixture(), 10, 0), false);
  const frame = fixture(); frame[2] = 1;
  assert.equal(semanticCatalogueFits(frame, 10, 1), false);
  assert.equal(semanticMainNode(fixture()), 1);
});

test('immutable design capture rejects malformed and out-of-range tokens', () => {
  const original = structuredClone(designs), captured = captureDesignCatalogue(original);
  original[0][0][0] = 0;
  assert.equal(captured[0][0][0], 0xffffff); assert.ok(Object.isFrozen(captured[0][0]));
  const bounds = [[0, 16777215], [0, 16777215], [0, 16777215], [0, 16777215],
    [0, 16777215], [0, 16777215], [0, 16777215], [0, 2], [1000, 4000],
    [1000, 3000], [0, 4000], [0, 4000], [16, 120], [8, 40], [20, 100], [24, 96]];
  for (let index = 0; index < bounds.length; index++) for (const value of [bounds[index][0] - 1, bounds[index][1] + 1, 0.5, NaN, '0']) {
    const invalid = structuredClone(designs); invalid[0][0][index] = value;
    assert.throws(() => captureDesignCatalogue(invalid), PresentationError, `token ${index}`);
  }
  for (const value of [[], Array(17).fill(designs[0]), [light], [[light]], [[light, light, light]], [[light.slice(1), light]]]) {
    assert.throws(() => captureDesignCatalogue(value), PresentationError);
  }
  let calls = 0; const tokens = [...light];
  Object.defineProperty(tokens, '0', {get() { calls++; return 0; }});
  assert.throws(() => captureDesignCatalogue([[tokens, light]]), PresentationError); assert.equal(calls, 0);
  const hole = [...light]; delete hole[3];
  assert.throws(() => captureDesignCatalogue([[hole, light]]), PresentationError);
  const injected = [...light]; injected.css = 'url(https://untrusted.invalid)';
  assert.throws(() => captureDesignCatalogue([[injected, light]]), PresentationError);
  for (const scope of ['', 'arbitrary', 'prismpm-semantic-1"]{background:red}']) {
    assert.throws(() => semanticStyle(scope, designs[0], 0), PresentationError);
  }
  assert.throws(() => semanticStyle('prismpm-semantic-1', [injected, light], 0), PresentationError);
  assert.throws(() => semanticStyle('prismpm-semantic-1', designs[0], 3), PresentationError);
});

test('semantic frame keeps exact aggregate bound and strict progress revision', () => {
  const bytes = encodeWire(fixture());
  assert.deepEqual(decodeSemanticPresentation(bytes, bytes.length), fixture());
  assert.throws(() => decodeSemanticPresentation(bytes, bytes.length - 1), PresentationError);
  assert.equal(semanticProgressFits(fixture(), fixture()), false);
  const pending = fixture(2); pending[1][2] = 1;
  pending[1][6][3][1][2] = false; pending[1][6][4][1][2] = false; pending[1][6][5][1][3] = false;
  assert.equal(semanticProgressFits(fixture(), pending), true);
});
