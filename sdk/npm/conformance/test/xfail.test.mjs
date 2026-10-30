// Expected failures, declared per runtime: a declared failure that fails is xfail, one that passes fails the run,
// a declaration naming nothing is refused, and a feature declared not applicable does not run.
import assert from 'node:assert/strict';
import { join } from 'node:path';
import { test } from 'node:test';
import { applyXfail, failed, runSuite, summarize } from '../src/runner.mjs';
import { checkXfail, loadRuntimes, loadXfail } from '../src/runtimes.mjs';
import { expandScenarios, loadFeatures, loadScenarios, referenceArchives } from '../src/scenarios.mjs';
import { ConformanceError } from '../src/util.mjs';
import { JS_ENTRYPOINT, ROOT, readJson, scratch, suiteCopy, editScenario, writeJson } from './helpers.mjs';

const features = loadFeatures(ROOT);
const runs = expandScenarios(loadScenarios(ROOT, { features }), referenceArchives(ROOT));
const row = (id, scenario, feature, status) => ({ id, scenario, feature, cites: ['HO-31'], archive: null, status, steps: 1 });

test('every registered runtime has an expected-failures file, and prism-code declares nothing', () => {
  const runtimes = loadRuntimes(ROOT);
  for (const name of Object.keys(runtimes.runtimes)) {
    const xfail = checkXfail(loadXfail(ROOT, name, runtimes), { runs, features });
    assert.equal(xfail.runtime, name);
  }
  const js = loadXfail(ROOT, 'prism-code', runtimes);
  assert.deepEqual(js.xfail, {});
  assert.deepEqual(js.notApplicable, {});
});

test('a declared failure that fails is xfail, by its base id or its expanded id, and does not fail the suite', () => {
  const declared = { runtime: 'x', xfail: { 'load-archive': 'not yet', 'check-no-verdict': 'later' }, notApplicable: {} };
  assert.deepEqual(applyXfail(row('load-archive[Calculator]', 'load-archive', 'load', 'failed'), declared), { ...row('load-archive[Calculator]', 'load-archive', 'load', 'xfail'), reason: 'not yet' });
  assert.deepEqual(applyXfail(row('load-archive[Other]', 'load-archive', 'load', 'error'), declared), { ...row('load-archive[Other]', 'load-archive', 'load', 'xfail'), reason: 'not yet' });
  assert.deepEqual(applyXfail(row('check-no-verdict', 'check-no-verdict', 'check', 'failed'), declared), { ...row('check-no-verdict', 'check-no-verdict', 'check', 'xfail'), reason: 'later' });
  assert.deepEqual(applyXfail(row('vectors-pass[Calculator]', 'vectors-pass', 'vectors', 'failed'), declared), row('vectors-pass[Calculator]', 'vectors-pass', 'vectors', 'failed'));
  const summary = summarize([applyXfail(row('load-archive[Calculator]', 'load-archive', 'load', 'failed'), declared)]);
  assert.equal(summary.xfail, 1);
  assert.ok(!failed({ summary }));
});

test('a declared failure that passes is unexpected-pass and fails the suite: the declaration is stale', () => {
  const declared = { runtime: 'x', xfail: { 'load-archive[Calculator]': 'not yet' }, notApplicable: {} };
  const applied = applyXfail(row('load-archive[Calculator]', 'load-archive', 'load', 'passed'), declared);
  assert.equal(applied.status, 'unexpected-pass');
  assert.equal(applied.reason, 'not yet');
  assert.equal(applyXfail(row('load-archive[Other]', 'load-archive', 'load', 'passed'), declared).status, 'passed');
  assert.ok(failed({ summary: summarize([applied]) }));
});

test('a feature declared not applicable makes every run of it not-applicable, whatever it would have done', () => {
  const declared = { runtime: 'x', xfail: {}, notApplicable: { vectors: 'no engine' } };
  for (const status of ['passed', 'failed', 'error']) {
    const applied = applyXfail(row('vectors-pass[Calculator]', 'vectors-pass', 'vectors', status), declared);
    assert.equal(applied.status, 'not-applicable');
    assert.equal(applied.reason, 'no engine');
  }
  assert.ok(!failed({ summary: summarize([applyXfail(row('vectors-pass[Calculator]', 'vectors-pass', 'vectors', 'failed'), declared)]) }));
});

test('a declaration naming no scenario or no feature is refused', () => {
  assert.throws(() => checkXfail({ runtime: 'x', xfail: { 'no-such-scenario': 'why' }, notApplicable: {} }, { runs, features }), (error) => error instanceof ConformanceError && error.code === 'xfail.unknown');
  assert.throws(() => checkXfail({ runtime: 'x', xfail: { 'load-archive[Nobody]': 'why' }, notApplicable: {} }, { runs, features }), (error) => error instanceof ConformanceError && error.code === 'xfail.unknown');
  assert.throws(() => checkXfail({ runtime: 'x', xfail: {}, notApplicable: { teleport: 'why' } }, { runs, features }), (error) => error instanceof ConformanceError && error.code === 'xfail.unknown');
  checkXfail({ runtime: 'x', xfail: { 'load-archive': 'a', 'load-archive[Calculator]': 'b', 'check-no-verdict': 'c' }, notApplicable: { vectors: 'd' } }, { runs, features });
});

test('an expected-failures file with the wrong runtime, an unknown member, no reason or a malformed id is refused', () => {
  const refuse = (edit, code) => {
    const root = suiteCopy((copy) => {
      const path = join(copy, 'xfail', 'prism-code.json');
      const file = readJson(path);
      edit(file);
      writeJson(path, file);
    });
    assert.throws(() => loadXfail(root, 'prism-code'), (error) => error instanceof ConformanceError && error.code === code, code);
  };
  refuse((f) => (f.runtime = 'prismpm'), 'xfail.runtime');
  refuse((f) => (f.extra = {}), 'xfail.member');
  refuse((f) => (f.xfail = { 'load-archive': '' }), 'xfail.reason');
  refuse((f) => (f.xfail = { 'Load Archive': 'x' }), 'xfail.id');
  refuse((f) => (f.notApplicable = { 'Not A Feature': 'x' }), 'xfail.feature');
  refuse((f) => (f.schema = 'other'), 'xfail.schema');
  refuse((f) => delete f.notApplicable, 'xfail.member');
});

test('end to end: a stale declaration fails the run, a true one does not, and a not-applicable feature is not executed', () => {
  const stale = suiteCopy((copy) => writeJson(join(copy, 'xfail', 'prism-code.json'), { schema: 'prism/conformance-xfail/1', runtime: 'prism-code', xfail: { 'load-archive': 'pretend' }, notApplicable: {} }));
  const staleReport = runSuite({ root: stale, runtime: 'prism-code', entrypoint: JS_ENTRYPOINT, only: ['load-archive'], scratch: scratch() });
  assert.equal(staleReport.scenarios[0].status, 'unexpected-pass');
  assert.ok(failed(staleReport));

  const honest = suiteCopy((copy) => {
    writeJson(join(copy, 'xfail', 'prism-code.json'), { schema: 'prism/conformance-xfail/1', runtime: 'prism-code', xfail: { 'load-archive': 'the version is wrong on purpose' }, notApplicable: { vectors: 'not here' } });
    editScenario(copy, 'load-archive', (s) => (s.steps[0].expect.result.physical_version = 5));
  });
  const honestReport = runSuite({ root: honest, runtime: 'prism-code', entrypoint: JS_ENTRYPOINT, only: ['load-archive', 'vectors-pass'], scratch: scratch() });
  const byId = Object.fromEntries(honestReport.scenarios.map((r) => [r.id, r]));
  assert.equal(byId['load-archive[Calculator]'].status, 'xfail');
  assert.equal(byId['load-archive[Calculator]'].reason, 'the version is wrong on purpose');
  assert.equal(byId['vectors-pass[Calculator]'].status, 'not-applicable');
  assert.equal(byId['vectors-pass[Calculator]'].reason, 'not here');
  assert.equal(byId['vectors-pass[Calculator]'].failures, undefined);
  assert.ok(!failed(honestReport));
  assert.deepEqual(honestReport.summary, { passed: 0, failed: 0, xfail: 1, 'unexpected-pass': 0, error: 0, 'not-applicable': 1 });
});
