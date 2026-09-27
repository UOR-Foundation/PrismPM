import assert from 'node:assert/strict';
import {writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {prepareProjection, run, sha, draft} from './compile.mjs';
import {projectionCorpus, projectExpected, observationCorpus, observedExpected} from './projection-corpus.mjs';
import {decodeEffectWire as decode} from '../../sdk/browser/effects-wire.mjs';
import {corpus} from '../browser-session/corpus.mjs';
import {maximumVectors, effectResultMaxima} from '../browser-session/maxima.mjs';
import {executeWasm, tsv} from './runtime.mjs';

export function verifyNativeProjectionInventory(output, rows) {
  const expected = rows.map(row => 'PASS ' + row.id + '\n').join('')
    + 'PASS ' + rows.length + ' journal projection vectors twice\n';
  assert.equal(output, expected, 'complete exact native vector inventory');
}

export function verifyProjection() {
  const build = prepareProjection(), projection = projectionCorpus(), session = corpus(), observation = observationCorpus();
  assert.equal(projection.length, 904); assert.equal(session.length, 895);
  const vectors = {predecessor: projection, session, observation};
  const paths = {};
  for (const [entry, rows] of Object.entries(vectors)) {
    paths[entry] = join(build.work, entry + '.tsv'); writeFileSync(paths[entry], tsv(rows), {flag: 'wx'});
  }
  for (const standard of [true, false]) {
    const runner = build.compileNative(standard);
    for (const [entry, rows] of Object.entries(vectors)) {
      const output = run(runner, [entry, paths[entry]], build.work);
      verifyNativeProjectionInventory(output, rows);
    }
  }
  const observed = Object.fromEntries(Object.entries(vectors).map(([entry, rows]) => [entry, executeWasm(build.wasm[entry], rows)]));
  build.unchanged();
  const evidence = {scope: 'private-predecessor-projection-component', publicApplicationAccepted: false,
    source: build.verified.source_id, attestation: build.verified.attestation_id, ir: build.generation.ir_sha256,
    artifacts: Object.fromEntries(Object.entries(build.wasm).map(([entry, bytes]) => [entry, sha(bytes)])),
    cases: {predecessor: projection.length, session: session.length, observation: observation.length}, observed,
    inputs: build.inputs, cacheRetirement: build.cacheRetirement};
  writeFileSync(join(build.work, 'projection-evidence.json'), JSON.stringify(evidence, null, 2) + '\n', {flag: 'wx'});
  return {build, evidence};
}

export function verifyProjectionMaxima(build) {
  const programs = [true, false].map(standard => build.compileNative(standard));
  const wasm = join(build.work, 'projection-maxima.wasm');
  writeFileSync(wasm, build.wasm.predecessor, {flag: 'wx'});
  const input = join(build.work, 'projection-maximum-input.bin'), expected = join(build.work, 'projection-maximum-output.bin');
  const results = [];
  // Generator chaining avoids retaining every complete 64-MiB vector at once.
  for (const factory of [maximumVectors, effectResultMaxima]) for (const baseline of factory()) {
    const row = projectExpected(baseline);
    writeFileSync(input, row.request); writeFileSync(expected, row.response);
    for (const program of programs) assert.equal(run(program, ['predecessor', input, expected], build.work),
      'PASS binary journal projection twice\n');
    const observed = JSON.parse(run(process.execPath, [join(draft, 'maximum-runner.mjs'), wasm, input, expected], build.work));
    assert.equal(observed.request, sha(row.request)); assert.equal(observed.response, sha(row.response));
    assert.ok(observed.maximumBytes <= 1073741824); assert.equal(observed.declaredPages, 16384);
    const result = {id: row.id, requestBytes: row.request.length, responseBytes: row.response.length, ...observed};
    console.log(JSON.stringify(result)); results.push(result);
  }
  assert.equal(results.length, 27, '18 session and 9 effect-result domain maxima');
  build.unchanged();
  writeFileSync(join(build.work, 'projection-maxima-evidence.json'), JSON.stringify(results, null, 2) + '\n', {flag: 'wx'});
  return results;
}

export function verifyObservationMaxima(build) {
  const programs = [true, false].map(standard => build.compileNative(standard));
  const wasm = join(build.work, 'observation-maxima.wasm');
  writeFileSync(wasm, build.wasm.observation, {flag: 'wx'});
  const input = join(build.work, 'observation-maximum-input.bin'), expected = join(build.work, 'observation-maximum-output.bin');
  const results = [];
  for (const factory of [maximumVectors, effectResultMaxima]) for (const baseline of factory()) {
    const reply = decode(baseline.response);
    if (reply[1] !== 0 || !Array.isArray(reply[2])) continue;
    const row = {id: baseline.id, request: baseline.response, response: observedExpected(baseline.response)};
    writeFileSync(input, row.request); writeFileSync(expected, row.response);
    for (const program of programs) assert.equal(run(program, ['observation', input, expected], build.work),
      'PASS binary journal projection twice\n');
    const observed = JSON.parse(run(process.execPath, [join(draft, 'maximum-runner.mjs'), wasm, input, expected], build.work));
    assert.equal(observed.request, sha(row.request)); assert.equal(observed.response, sha(row.response));
    const result = {id: row.id, requestBytes: row.request.length, responseBytes: row.response.length, ...observed};
    console.log(JSON.stringify(result)); results.push(result);
  }
  assert.equal(results.length, 21, 'all accepted session and effect-result successors, including exact64MiB input');
  build.unchanged();
  writeFileSync(join(build.work, 'observation-maxima-evidence.json'), JSON.stringify(results, null, 2) + '\n', {flag: 'wx'});
  return results;
}
