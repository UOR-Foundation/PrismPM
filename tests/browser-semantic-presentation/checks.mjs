import assert from 'node:assert/strict';
import {readFileSync, writeFileSync, rmSync} from 'node:fs';
import {join} from 'node:path';
import {prepare, run, sha, repository, frozenInputs} from './compile.mjs';
import {corpus, predicates, fixture, labels, designs} from './corpus.mjs';
import {maximumCorpus} from './maximum-fixtures.mjs';
import {encodeWire} from '../../sdk/browser/presentation-wire.mjs';
import {executeWasm} from '../browser-presentation/checks.mjs';
export {executeWasm};

const tsv = rows => rows.map(row => row.id + '\t' + Buffer.from(row.request).toString('hex')
  + '\t' + Buffer.from(row.response).toString('hex') + '\n').join('');
export const closure = frozenInputs;

export function nativeReplay(build, rows, stem) {
  assert.match(stem, /^[a-z][a-z0-9-]*$/);
  const path = join(build.work, stem + '.tsv'); writeFileSync(path, tsv(rows), {flag: 'wx'});
  for (const standard of [true, false]) {
    const binary = build.compileNative(standard), output = run(binary, [path], build.runner);
    assert.deepEqual([...output.matchAll(/^PASS ([A-Za-z0-9]+)$/gm)].map(row => row[1]), rows.map(row => row.id));
    assert.match(output, new RegExp('PASS ' + rows.length + ' complete semantic vectors twice'));
  }
}

export function verifyWire(t) {
  const before = closure(), build = prepare(null, null, before);
  t.after(() => rmSync(build.work, {recursive: true, force: true}));
  assert.ok(build.verified?.attestation_id && build.cacheRetirement?.owner === 'semantic-presentation');
  // Diagnostic inventory only. Executed corpus, browser journeys and mutants
  // below establish behavior; matching source tokens never establishes it.
  const registry = JSON.parse(readFileSync(join(repository, 'model/browser-semantic-presentation-diagnostics.json')));
  assert.equal(registry.capability, 'DK-29'); assert.equal(registry.error_class, 'PresentationError');
  assert.deepEqual(registry.errors, [...new Set(registry.errors)].sort());
  const diagnosticSource = ['presentation-wire', 'presentation-dom', 'semantic-presentation-wire',
    'semantic-presentation-style'].map(name => readFileSync(join(repository, 'sdk/browser/' + name + '.mjs'), 'utf8')).join('\n');
  for (const code of registry.errors) assert.ok(diagnosticSource.includes("'" + code + "'"), code);
  for (const match of diagnosticSource.matchAll(/(?:fail|PresentationError)\('([a-z-]+)'\)/g)) assert.ok(registry.errors.includes(match[1]), match[1]);
  assert.deepEqual(registry.wire_errors, ['BadLimits', 'BadCursor', 'Truncated', 'WrongType',
    'UnsupportedHead', 'NonCanonical', 'ValueLimit', 'InvalidUtf8', 'TrailingInput', 'InvalidSemanticPresentation']);
  const samples = [
    {id: 'FixtureBaseline', request: encodeWire([1, 1, 0, 0]), response: encodeWire(fixture())},
    {id: 'LabelsBaseline', request: new Uint8Array(), response: new TextEncoder().encode(JSON.stringify(labels))},
    {id: 'DesignsBaseline', request: new Uint8Array(), response: encodeWire(designs)},
  ];
  const rows = corpus(), typed = predicates();
  nativeReplay(build, [...rows, ...typed, ...samples], 'all-vectors');
  for (const standard of [true, false]) {
    const path = build.compileNative(standard), original = readFileSync(path), changed = Buffer.from(original);
    changed[0] ^= 1;
    try {
      writeFileSync(path, changed);
      assert.throws(() => build.compileNative(standard), /immutable actual native binary/);
    } finally { writeFileSync(path, original); }
    assert.equal(build.compileNative(standard), path);
  }
  build.maximum = executeWasm(build.wasmBytes, rows);
  executeWasm(build.predicatesBytes, typed, 32);
  for (const [index, key] of ['fixtureBytes', 'labelsBytes', 'designsBytes'].entries()) executeWasm(build[key], [samples[index]], 32);
  build.maxima = [];
  function* boundedCases() {
    yield* maximumCorpus();
    yield {id: 'EnvelopeOneOver', request: new Uint8Array(67108865),
      response: Uint8Array.of(0x83, 1, 1, 6)};
  }
  for (const row of boundedCases()) {
    const input = join(build.work, row.id + '.request'), output = join(build.work, row.id + '.response');
    writeFileSync(input, row.request, {flag: 'wx'}); writeFileSync(output, row.response, {flag: 'wx'});
    for (const standard of [true, false]) {
      const binary = build.compileNative(standard);
      assert.equal(run(binary, ['--binary', input, output], build.runner), 'PASS binary complete semantic vector twice\n');
    }
    if (row.id === 'EnvelopeOneOver') {
      const instance = new WebAssembly.Instance(new WebAssembly.Module(build.wasmBytes), {});
      assert.throws(() => instance.exports.holo_alloc(row.request.length), WebAssembly.RuntimeError);
    } else {
      const maximum = executeWasm(build.wasmBytes, [row]);
      build.maxima.push({id: row.id, length: row.request.length,
        request: sha(row.request), response: sha(row.response), ...maximum});
    }
  }
  assert.deepEqual(closure(), before);
  t.diagnostic(JSON.stringify({scope: 'semantic-component-only', source: build.verified.source_id,
    attestation: build.verified.attestation_id, ir: build.generation.ir_sha256, wasm: sha(build.wasmBytes),
    corpus: rows.length, typed: typed.length, maxima: build.maxima,
    inputs: {files: Object.keys(before).length, sha256: sha(Buffer.from(JSON.stringify(before)))}}));
  return build;
}

export function replayBrowser(build, result) {
  assert.equal(result.modelChecked, true); assert.ok(result.calls.length > 0);
  const roles = {wire: 'Wire', fixture: 'Fixture', labels: 'Labels', designs: 'Designs'};
  const rows = result.calls.map((row, index) => ({id: (assert.ok(roles[row.role]), roles[row.role]) + index,
    request: Buffer.from(row.request, 'hex'), response: Buffer.from(row.response, 'hex')}));
  nativeReplay(build, rows, 'observed-browser');
  const changed = rows.map(row => ({...row, response: Buffer.from(row.response)}));
  changed[0].response[0] ^= 1;
  assert.throws(() => nativeReplay(build, changed, 'changed-browser'), /native output mismatch/);
}

export function verifyMutation(kind, baseline) {
  const build = prepare(kind, baseline.sources, baseline.inputs);
  try {
    const row = kind === 'design' ? predicates()[1] : kind === 'catalogue' ? predicates()[5]
      : corpus().find(row => row.id === ({purpose: 'PublicSecretPurpose', main: 'MissingMain', trailing: 'Trailing'}[kind]));
    assert.ok(row);
    const path = join(build.work, 'mutation.tsv'); writeFileSync(path, tsv([row]), {flag: 'wx'});
    for (const standard of [true, false]) {
      const binary = build.compileNative(standard);
      assert.throws(() => run(binary, [path], build.runner), /native output mismatch/, kind);
    }
    assert.throws(() => executeWasm(build[['design', 'catalogue'].includes(kind) ? 'predicatesBytes' : 'wasmBytes'], [row],
      ['design', 'catalogue'].includes(kind) ? 32 : 67108864), /generated Wasm output mismatch/, kind);
  } finally { rmSync(build.work, {recursive: true, force: true}); }
}
