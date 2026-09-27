import assert from 'node:assert/strict';
import test from 'node:test';
import {assertCapturedRetentionSources, frozenInputs, sourceClosure} from './compile.mjs';
import {retentionCorpus} from '../browser-session-journal/retention-corpus.mjs';
import {verifyNativeInventory} from './checks.mjs';

test('retention compiler freezes complete source and transitive host/oracle inputs', () => {
  const inputs = frozenInputs(), sources = sourceClosure(); assertCapturedRetentionSources(inputs, sources);
  for (const path of ['sdk/browser/session-storage.mjs', 'sdk/browser/session-retention-wire.mjs',
    'tests/browser-session-journal/storage-browser.mjs', 'tests/browser-session-journal/retention-corpus.mjs',
    'tests/browser-session-journal/retention-mutations.mjs', 'tests/browser-view/local-module-inputs.mjs',
    'tests/browser-view/generated-wasm.mjs', 'tests/browser-session-journal/wasm-artifact-checks.mjs']) assert.ok(inputs[path], path);
  const name = 'Foundation.Browser.Application.V1.SessionJournalRetention';
  const changed = new Map(sources); changed.set(name, Buffer.concat([sources.get(name), Buffer.from('\n')]));
  assert.throws(() => assertCapturedRetentionSources(inputs, changed), /actual captured source/);
  const missing = new Map(sources); missing.delete(name);
  assert.throws(() => assertCapturedRetentionSources(inputs, missing), /complete captured source/);
});

test('retention native inventory refuses absent, repeated, partial and stale vectors', () => {
  const rows = retentionCorpus(), output = rows.map(row => 'PASS ' + row.id + '\n').join('')
    + 'PASS ' + rows.length + ' journal retention vectors twice\n';
  verifyNativeInventory(output, rows);
  for (const defect of ['', output.replace('PASS RetentionEmptySnapshot\n', ''), output + output,
    output.replace('RetentionEmptySnapshot', 'StaleFixture')]) assert.throws(() => verifyNativeInventory(defect, rows));
});
