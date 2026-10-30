// The merge gate on the holo format: every revision @prism-code/spec declares is named by a scenario that loads an
// archive of it, every scenario names a declared revision, and every reference archive is loaded.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';
import { checkGate, gate, loadRevisions } from '../src/gate.mjs';
import { loadFeatures, loadScenarios, referenceArchives } from '../src/scenarios.mjs';
import { ConformanceError } from '../src/util.mjs';
import { BIN, ROOT, SPEC_CONSTANTS, scratch } from './helpers.mjs';

const scenarios = loadScenarios(ROOT, { features: loadFeatures(ROOT) });
const archives = referenceArchives(ROOT);
const cli = (args) => spawnSync(process.execPath, [BIN, ...args], { encoding: 'utf8', cwd: ROOT });

test('the committed suite passes the gate against the revisions @prism-code/spec declares', async () => {
  const { revisions, problems } = await gate({ root: ROOT });
  assert.deepEqual(problems, []);
  assert.deepEqual(revisions, await loadRevisions(SPEC_CONSTANTS));
  assert.ok(revisions.includes('2026-10-23'));
  assert.ok(scenarios.every((s) => revisions.includes(s.revision)));
});

test('a revision named by no scenario fails the gate: a format change cannot merge without a scenario', () => {
  const problems = checkGate({ revisions: ['2026-10-23', '2099-01-01'], scenarios, archives });
  assert.deepEqual(problems, ['format revision 2099-01-01 is named by no scenario; a revision cannot merge without one that loads an archive of it']);
});

test('a scenario naming a revision the spec does not declare fails the gate', () => {
  const foreign = scenarios.map((s) => (s.id === 'load-archive' ? { ...s, revision: '2099-01-01' } : s));
  const problems = checkGate({ revisions: ['2026-10-23'], scenarios: foreign, archives });
  assert.deepEqual(problems, ['scenario load-archive names revision 2099-01-01, which @prism-code/spec does not declare (2026-10-23)']);
});

test('a revision whose scenarios have no inspect step pinning it is a label, not evidence, and fails the gate', () => {
  const unpinned = scenarios.map((s) => ({
    ...s,
    steps: s.steps.map((step) => (step.action === 'inspect' && step.expect.result?.revision === '2026-10-23' ? { ...step, expect: { ...step.expect, result: { ...step.expect.result, revision: { $type: 'string' } } } } : step)),
  }));
  const problems = checkGate({ revisions: ['2026-10-23'], scenarios: unpinned, archives });
  assert.equal(problems.length, 1);
  assert.match(problems[0], /^format revision 2026-10-23 is named by .*, but none has an inspect step whose expected result\.revision is 2026-10-23$/);
});

test('a reference archive loaded by no scenario, or a suite with no scenario over each archive, fails the gate', () => {
  const ghost = [...archives, { name: 'Ghost', archive: 'fixtures/Ghost.holo', history: 'fixtures/Ghost.history.json' }];
  assert.deepEqual(checkGate({ revisions: ['2026-10-23'], scenarios, archives: ghost }), []);
  const single = scenarios.filter((s) => s.archive !== 'each');
  const problems = checkGate({ revisions: ['2026-10-23'], scenarios: single, archives });
  assert.ok(problems.includes('reference archive fixtures/Calculator.holo is loaded by no scenario over each archive'));
  assert.ok(problems.includes('no scenario runs over each reference archive'));
});

test('loadRevisions reads the keys of FORMAT_REVISIONS, sorted, and refuses a module without them', async () => {
  const directory = scratch();
  const spec = join(directory, 'constants.mjs');
  writeFileSync(spec, "export const FORMAT_REVISIONS = Object.freeze({ '2099-01-01': {}, '2026-10-23': {} });\n");
  assert.deepEqual(await loadRevisions(spec), ['2026-10-23', '2099-01-01']);
  const empty = join(directory, 'empty.mjs');
  writeFileSync(empty, 'export const FORMAT_REVISIONS = {};\n');
  await assert.rejects(loadRevisions(empty), (error) => error instanceof ConformanceError && error.code === 'gate.spec');
});

test('the command line gate exits 0 on the committed suite and 1 naming the revision a spec adds without a scenario', () => {
  const ok = cli(['gate']);
  assert.equal(ok.status, 0, ok.stderr);
  assert.match(ok.stdout, /^gate: every format revision \(2026-10-23\) is named by a scenario/);
  const directory = scratch();
  const spec = join(directory, 'constants.mjs');
  writeFileSync(spec, "export const FORMAT_REVISIONS = Object.freeze({ '2026-10-23': {}, '2099-01-01': {} });\n");
  const added = cli(['gate', '--spec', spec]);
  assert.equal(added.status, 1);
  assert.match(added.stderr, /^gate: format revision 2099-01-01 is named by no scenario/);
  assert.equal(cli(['gate', 'extra']).status, 64);
});
