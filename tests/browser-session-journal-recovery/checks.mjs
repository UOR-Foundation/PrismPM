// Complete generated component execution; not authenticated journal acceptance.
import assert from 'node:assert/strict';
import {writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {prepareRecovery, run, sha} from './compile.mjs';
import {recoveryCorpus} from './corpus.mjs';
import {corpus as sessionCorpus} from '../browser-session/corpus.mjs';
import {metadataCorpus} from '../browser-session-journal/metadata-corpus.mjs';
import {executeWasm, tsv} from './runtime.mjs';

export function verifyNativeInventory(output, rows) {
  assert.equal(output, rows.map(row => 'PASS ' + row.id + '\n').join('')
    + 'PASS ' + rows.length + ' journal recovery vectors twice\n', 'complete exact native vector inventory');
}
export function verifyRecoveryComponents() {
  const build = prepareRecovery(), recovery = recoveryCorpus();
  const vectors = {context: recovery.context, recovery: recovery.recovery,
    session: sessionCorpus(), metadata: metadataCorpus()};
  assert.equal(vectors.session.length, 895, 'complete unchanged Session corpus');
  const files = {};
  for (const [entry, rows] of Object.entries(vectors)) {
    assert.ok(rows.length > 0); assert.equal(new Set(rows.map(row => row.id)).size, rows.length);
    files[entry] = join(build.work, entry + '.tsv');
    writeFileSync(files[entry], tsv(rows), {flag: 'wx'});
  }
  for (const standard of [true, false]) {
    const runner = build.compileNative(standard);
    for (const [entry, rows] of Object.entries(vectors))
      verifyNativeInventory(run(runner, [entry, files[entry]], build.work), rows);
  }
  const observed = Object.fromEntries(Object.entries(vectors).map(([entry, rows]) => [entry, executeWasm(build.wasm[entry], rows)]));
  build.unchanged();
  const evidence = {scope: 'private-recovery-and-metadata-components', publicApplicationAccepted: false,
    source: build.verified.source_id, attestation: build.verified.attestation_id, ir: build.generation.ir_sha256,
    artifacts: Object.fromEntries(Object.entries(build.wasm).map(([entry, bytes]) => [entry, sha(bytes)])),
    cases: Object.fromEntries(Object.entries(vectors).map(([entry, rows]) => [entry, rows.length])),
    observed, inputs: build.inputs, cacheRetirement: build.cacheRetirement};
  writeFileSync(join(build.work, 'recovery-component-evidence.json'), JSON.stringify(evidence, null, 2) + '\n', {flag: 'wx'});
  return {build, evidence};
}
