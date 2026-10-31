#!/usr/bin/env node
// Writes V&V acceptance evidence only from all three passed part records of
// one commit. Reasons for the split are in SPEC.md, next to the 15 gates.
import assert from 'node:assert/strict';
import {readdirSync, readFileSync, writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {pathToFileURL} from 'node:url';
import {canonical, validateVvEvidence} from './sdk-vv-run.mjs';

export const parts = Object.freeze(['browser', 'conformance', 'source']);

export function assemble(directory, revision) {
  assert.match(revision, /^[a-f0-9]{40}$/);
  assert.deepEqual(readdirSync(directory).sort(), parts.map(part => `vv-part-${part}.json`), 'exactly the three part records required');
  for (const part of parts) {
    const bytes = readFileSync(join(directory, `vv-part-${part}.json`));
    assert(bytes.length <= 4096, 'bounded part record');
    const value = JSON.parse(bytes.toString('utf8'));
    assert.equal(bytes.toString('utf8'), canonical(value), 'original canonical part bytes required');
    assert.deepEqual(value, {commit: revision, part, schema: 'prismpm/vv-part/1', status: 'passed'});
  }
  const evidence = Buffer.from(canonical({
    commit: revision,
    gates: Array.from({length: 15}, (_, index) => index + 1),
    schema: 'prismpm/vv-evidence/1',
    status: 'passed',
  }));
  validateVvEvidence(evidence, revision);
  return evidence;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [directory, output, revision, ...extra] = process.argv.slice(2);
  assert(directory && output && revision && extra.length === 0, 'usage: vv-assemble.mjs PARTS_DIRECTORY OUTPUT REVISION');
  writeFileSync(output, assemble(directory, revision), {flag: 'wx', mode: 0o644});
  console.log(`All 3 V&V parts and 15 gates PASSED for commit ${revision}.`);
}
