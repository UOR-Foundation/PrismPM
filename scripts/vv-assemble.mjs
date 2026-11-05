#!/usr/bin/env node
// Writes V&V acceptance evidence only from every passed part record of
// one commit. Reasons for the split are in SPEC.md, next to the 15 gates.
import assert from 'node:assert/strict';
import {readdirSync, writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {pathToFileURL} from 'node:url';
import {canonical, validateVvEvidence} from './sdk-vv-run.mjs';
import {readOriginal} from './sdk-bootstrap-retention.mjs';

// The one list of V&V parts. Every other script and workflow matrix derives
// from it; the main source workflow and Rust dispatch are checked against it.
// This does not change the installed SDK's two-run acceptance contract.
export const parts = Object.freeze(['conformance', 'dk-0-1', 'dk-2', 'dk-3', 'dk-4', 'gates', 'ho-oc', 'source', 'st-sy',
  'standalone-journal-recovery', 'standalone-payload', 'standalone-projection',
  'standalone-recovery-frames', 'standalone-reservation', 'standalone-retention']);

// The release dispatcher/assembler contract, applied only to source scheduling.
// The installed SDK executor retains both complete, consecutive monolithic runs.
export function validateVvPart(bytes, revision, part) {
  assert.match(revision, /^[a-f0-9]{40}$/);
  assert(parts.includes(part), 'closed V&V part');
  assert(Buffer.isBuffer(bytes) && bytes.length <= 4096);
  const value = JSON.parse(bytes.toString('utf8'));
  assert.equal(bytes.toString('utf8'), canonical(value), 'original canonical V&V part bytes required');
  assert.deepEqual(value, {commit: revision, part, schema: 'prismpm/vv-part/1', status: 'passed'},
    'complete exact-commit V&V part required');
  return value;
}

export function assembleVvEvidence(records, revision) {
  assert(records instanceof Map);
  assert.deepEqual([...records.keys()].sort(), [...parts].sort(), 'exactly the part records required');
  for (const part of parts) validateVvPart(records.get(part), revision, part);
  const evidence = Buffer.from(canonical({commit: revision,
    gates: Array.from({length: 15}, (_, index) => index + 1),
    schema: 'prismpm/vv-evidence/1', status: 'passed'}));
  validateVvEvidence(evidence, revision);
  return evidence;
}

export function assemble(directory, revision) {
  assert.match(revision, /^[a-f0-9]{40}$/);
  assert.deepEqual(readdirSync(directory).sort(), parts.map(part => `vv-part-${part}.json`), 'exactly the part records required');
  return assembleVvEvidence(new Map(parts.map(part => {
    const bytes = readOriginal(join(directory, `vv-part-${part}.json`), 4096);
    return [part, bytes];
  })), revision);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  if (process.argv.length === 3 && process.argv[2] === '--list') {
    console.log(parts.join('\n'));
    process.exit(0);
  }
  const [directory, output, revision, ...extra] = process.argv.slice(2);
  assert(directory && output && revision && extra.length === 0, 'usage: vv-assemble.mjs PARTS_DIRECTORY OUTPUT REVISION | --list');
  writeFileSync(output, assemble(directory, revision), {flag: 'wx', mode: 0o644});
  console.log(`All ${parts.length} V&V parts and 15 gates PASSED for commit ${revision}.`);
}
