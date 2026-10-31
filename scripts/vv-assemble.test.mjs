import assert from 'node:assert/strict';
import {mkdtempSync, rmSync, unlinkSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import test from 'node:test';
import {canonical, validateVvEvidence} from './sdk-vv-run.mjs';
import {assemble, parts} from './vv-assemble.mjs';

const revision = 'a'.repeat(40);
const record = (part, fields = {}) => canonical({commit: revision, part, schema: 'prismpm/vv-part/1', status: 'passed', ...fields});
const fixture = t => {
  const directory = mkdtempSync(join(tmpdir(), 'vv-assemble-'));
  t.after(() => rmSync(directory, {recursive: true, force: true}));
  for (const part of parts) writeFileSync(join(directory, `vv-part-${part}.json`), record(part));
  return directory;
};

test('four passed parts of one commit produce the unchanged 15-gate evidence', t => {
  validateVvEvidence(assemble(fixture(t), revision), revision);
});

test('assembly refuses a missing, extra, failed, foreign or altered part', t => {
  const altered = [
    ['missing part', directory => unlinkSync(join(directory, 'vv-part-browser.json'))],
    ['extra record', directory => writeFileSync(join(directory, 'vv-part-other.json'), record('other'))],
    ['failed part', directory => writeFileSync(join(directory, 'vv-part-source.json'), record('source', {status: 'failed'}))],
    ['other commit', directory => writeFileSync(join(directory, 'vv-part-conformance.json'), record('conformance', {commit: 'b'.repeat(40)}))],
    ['missing second browser part', directory => unlinkSync(join(directory, 'vv-part-browser-2.json'))],
    ['wrong part name', directory => writeFileSync(join(directory, 'vv-part-browser.json'), record('source'))],
    ['extra field', directory => writeFileSync(join(directory, 'vv-part-source.json'), record('source', {gates: [1]}))],
    ['noncanonical bytes', directory => writeFileSync(join(directory, 'vv-part-source.json'), `${record('source')}\n`)],
  ];
  for (const [name, alter] of altered) {
    const directory = fixture(t);
    alter(directory);
    assert.throws(() => assemble(directory, revision), undefined, name);
  }
  assert.throws(() => assemble(fixture(t), 'b'.repeat(40)));
  assert.throws(() => assemble(fixture(t), 'not-a-revision'));
});
