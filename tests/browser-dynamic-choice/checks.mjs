import assert from 'node:assert/strict';
import {readFileSync, writeFileSync, rmSync} from 'node:fs';
import {join} from 'node:path';
import {prepare, frozenInputs, assertFrozenInputs, sha, repository} from './compile.mjs';
import {ownerCorpus} from './corpus.mjs';
import {maximumFrame, maximumIds} from './maximum-fixtures.mjs';
import {mutationNames} from './mutations.mjs';
import {journey, journeyNames, verifyMutants, maximumJourney} from './browser.mjs';
import {corpus as legacyCorpus, boundaries as legacyBoundaries} from '../browser-presentation/corpus.mjs';
import {executeWasm} from '../browser-presentation/checks.mjs';
import {encodeWire} from '../../sdk/browser/presentation-wire.mjs';

const roles = ['wire', 'semantic', 'fixture', 'labels', 'designs', 'intent', 'catalogue', 'size'];
const tsv = rows => rows.map(row => row.id + '\t' + row.role + '\t'
  + Buffer.from(row.request).toString('hex') + '\t' + Buffer.from(row.response).toString('hex') + '\n').join('');

export function nativeReplay(build, rows, stem) {
  assert.match(stem, /^[a-z][a-z0-9-]*$/); assert.ok(rows.length);
  assert.equal(new Set(rows.map(row => row.id)).size, rows.length);
  for (const row of rows) { assert.match(row.id, /^[A-Za-z0-9]+$/); assert.ok(roles.includes(row.role)); }
  const path = join(build.work, stem + '.tsv'); writeFileSync(path, tsv(rows), {flag: 'wx'});
  for (const standard of [true, false]) {
    const output = build.runNative(standard, [path]);
    assert.deepEqual([...output.matchAll(/^PASS ([A-Za-z0-9]+)$/gm)].map(row => row[1]), rows.map(row => row.id));
    assert.ok(output.endsWith('PASS ' + rows.length + ' complete dynamic choice vectors twice\n'));
  }
}

function replayCalls(build, calls, stem) {
  assert.ok(calls.length);
  const rows = calls.map((row, index) => ({id: 'Observed' + index, role: row.role,
    request: Buffer.from(row.request, 'hex'), response: Buffer.from(row.response, 'hex')}));
  nativeReplay(build, rows, stem);
  // Both actual native observers must reject altered browser observations.
  const changed = rows.map(row => ({...row, response: Buffer.from(row.response)}));
  assert.ok(changed[0].response.length); changed[0].response[0] ^= 1;
  const path = join(build.work, stem + '-changed.tsv'); writeFileSync(path, tsv(changed), {flag: 'wx'});
  for (const standard of [true, false]) assert.throws(() => build.runNative(standard, [path]), /native output mismatch/);
}

function binaryReplay(build, role, input, output) {
  for (const standard of [true, false]) assert.equal(build.runNative(standard,
    ['--binary', role, input, output]), 'PASS binary dynamic choice vector twice\n');
}

function verifyMaximum(build) {
  const results = [];
  for (const id of maximumIds) {
    const row = maximumFrame(id), input = join(build.work, id + '.request');
    writeFileSync(input, row.bytes, {flag: 'wx'});
    binaryReplay(build, row.role, input, input);
    const observed = build.withWasm(row.role, bytes => executeWasm(bytes,
      [{id, request: row.bytes, response: row.bytes}]));
    if (row.role === 'wire') {
      const output = join(build.work, id + '.size'); writeFileSync(output, encodeWire(row.bytes.length), {flag: 'wx'});
      binaryReplay(build, 'size', input, output);
      build.withWasm('size', bytes => executeWasm(bytes,
        [{id: id + 'Size', request: row.bytes, response: encodeWire(row.bytes.length)}]));
    }
    results.push({id, role: row.role, request: sha(row.bytes), response: sha(row.bytes),
      length: row.bytes.length, payload: row.payload, ...observed});
  }
  const input = join(build.work, 'one-over.request'), output = join(build.work, 'one-over.response');
  writeFileSync(input, new Uint8Array(67108865), {flag: 'wx'});
  writeFileSync(output, Uint8Array.of(0x83, 1, 1, 6), {flag: 'wx'});
  for (const role of ['wire', 'semantic']) {
    binaryReplay(build, role, input, output);
    build.withWasm(role, bytes => {
      const over = new WebAssembly.Instance(new WebAssembly.Module(bytes), {});
      assert.throws(() => over.exports.holo_alloc(67108865), WebAssembly.RuntimeError);
    });
  }
  return results;
}

function verifySourceMutation(kind, baseline) {
  const id = ({duplicate: 'DuplicateIdentifier', 'name-limit': 'OverNameBytes', selection: 'IntentUnknownId',
    aggregate: 'MixedOptionsOver', 'semantic-field': 'SemanticDynamicFieldHelpers', size: 'SizeSourceOrderDuplicateNames'})[kind];
  const row = ownerCorpus().find(row => row.id === id); assert.ok(row);
  // An unrelated build failure is a failed owner, never a killed mutation.
  const build = prepare(kind, baseline.sources, baseline.inputs);
  try {
    const path = join(build.work, 'intended-mutation.tsv'); writeFileSync(path, tsv([row]), {flag: 'wx'});
    for (const standard of [true, false]) assert.throws(() => build.runNative(standard, [path]),
      new RegExp(id + ' native output mismatch'), 'intended native mutation assertion ' + kind);
    build.withWasm(row.role, bytes => assert.throws(() => executeWasm(bytes, [row]),
      new RegExp(id + ' generated Wasm output mismatch'), 'intended Wasm mutation assertion ' + kind));
    build.unchanged();
    return {kind, id, source: build.verified.source_id, attestation: build.verified.attestation_id,
      ir: build.generation.ir_sha256, native: build.nativeEvidence(), wasm: build.wasmEvidence()};
  } finally { rmSync(build.work, {recursive: true, force: true}); }
}

export async function verifyDynamicChoice(t) {
  const inputs = frozenInputs(), build = prepare(null, null, inputs);
  let success = false;
  try {
    assert.ok(build.verified.attestation_id && build.cacheRetirement.owner === 'dynamic-choice');
    // No new diagnostic codes or weakened legacy code ownership.
    const registry = JSON.parse(readFileSync(join(repository, 'model/browser-semantic-presentation-diagnostics.json')));
    assert.equal(registry.capability, 'DK-29'); assert.equal(registry.error_class, 'PresentationError');
    const sources = ['presentation-wire', 'presentation-dom', 'semantic-presentation-wire', 'semantic-presentation-style']
      .map(name => readFileSync(join(repository, 'sdk/browser/' + name + '.mjs'), 'utf8')).join('\n');
    for (const match of sources.matchAll(/(?:fail|PresentationError)\('([a-z-]+)'\)/g)) assert.ok(registry.errors.includes(match[1]));
    const rows = [...ownerCorpus(), ...[...legacyCorpus(), ...legacyBoundaries()]
      .map(row => ({...row, id: 'Legacy' + row.id, role: 'wire'}))];
    nativeReplay(build, rows, 'all-vectors');
    const memory = {};
    for (const role of roles) memory[role] = build.withWasm(role, bytes => executeWasm(bytes,
      rows.filter(row => row.role === role), ['fixture', 'labels', 'designs'].includes(role) ? 32 : 67108864));
    const maxima = verifyMaximum(build);
    const browser = await journey(build); assert.deepEqual(browser.cases, journeyNames); assert.equal(browser.calls.length, 53);
    replayCalls(build, browser.calls, 'observed-browser');
    const rendererMutants = await verifyMutants(build); assert.equal(rendererMutants.calls.length, 33);
    replayCalls(build, rendererMutants.calls, 'mutated-browser');
    const maximumBrowser = await maximumJourney(build);
    for (const result of maximumBrowser.results) {
      const expected = maxima.find(row => row.id === result.id); assert.ok(expected);
      assert.equal(result.payload, expected.payload); assert.equal(result.observations.length, 2);
      for (const [repeat, observed] of result.observations.entries()) {
        assert.deepEqual({...observed, memory: undefined}, {repeat, role: expected.role, request: expected.request,
          response: expected.response, length: expected.length, memory: undefined});
        assert.ok(observed.memory <= 1073741824);
      }
    }
    replayCalls(build, maximumBrowser.calls, 'maximum-browser-catalogues');
    // Observed maximum hashes bind to the exact files already executed twice by
    // both native modes; they are not constructed browser response transcripts.
    const sourceMutants = [];
    for (const kind of mutationNames) {
      t.diagnostic('Compiling intended source mutation ' + kind);
      sourceMutants.push(verifySourceMutation(kind, build));
    }
    build.unchanged(); assertFrozenInputs(inputs);
    t.diagnostic(JSON.stringify({scope: 'dynamic-choice-component-only', source: build.verified.source_id,
      attestation: build.verified.attestation_id, ir: build.generation.ir_sha256,
      corpus: rows.length, native: build.nativeEvidence(), wasm: build.wasmEvidence(),
      packages: build.generatedPackages, memory, maxima, browser: {cases: browser.cases, calls: browser.calls.length,
        audits: browser.audits, maximum: maximumBrowser.results, rendererMutants: rendererMutants.names}, sourceMutants,
      frozenInputs: {files: Object.keys(inputs).length, sha256: sha(Buffer.from(JSON.stringify(inputs)))},
      obligations: ['unchanged DK-23, DK-26 and DK-29 full owners', 'installed SDK/source archive regeneration',
        'application authorization, meaningful option names and human usability/accessibility assessment']}));
    success = true;
  } finally {
    if (success) rmSync(build.work, {recursive: true, force: true});
    else process.stderr.write('Retained failed dynamic choice owner ' + build.work + '\n');
  }
}
