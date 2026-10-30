// COMPLIANCE.md: the committed file is the matrix rendered from the suite's report, cells follow the statuses, and the
// stale check fails on a changed file.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';
import { CELLS, HEADER, cellOf, current, renderMatrix } from '../src/matrix.mjs';
import { runSuite } from '../src/runner.mjs';
import { expandScenarios, loadFeatures, loadScenarios, referenceArchives } from '../src/scenarios.mjs';
import { ConformanceError, lf } from '../src/util.mjs';
import { BIN, REPO, ROOT, readJson, scratch, writeJson } from './helpers.mjs';

const COMPLIANCE = join(REPO, 'COMPLIANCE.md');
const report = runSuite({ root: ROOT, runtime: 'prism-code', scratch: scratch() });
const rendered = renderMatrix({ root: ROOT, reports: { 'prism-code': report } });
const cli = (args) => spawnSync(process.execPath, [BIN, ...args], { encoding: 'utf8', cwd: ROOT });

const synthetic = (rows) => ({
  schema: 'prism/conformance-report/1',
  runtime: { name: 'prismpm', language: 'Rust', version: '0.4.0' },
  archives: ['Calculator'],
  scenarios: rows.map(([id, feature, status, reason]) => ({ id, scenario: id.replace(/\[.*\]$/, ''), feature, cites: ['HO-31'], archive: null, status, steps: 1, ...(reason ? { reason } : {}) })),
  summary: { passed: 0, failed: 0, xfail: 0, 'unexpected-pass': 0, error: 0, 'not-applicable': 0 },
});

test('the committed COMPLIANCE.md is the matrix rendered from the suite against prism-code', () => {
  const committed = readFileSync(COMPLIANCE, 'utf8');
  assert.ok(current(rendered, committed), 'COMPLIANCE.md is stale: run `npm run matrix` in sdk/npm/conformance and commit it');
  assert.ok(lf(committed).startsWith(`${HEADER}\n\n# COMPLIANCE\n`));
});

test('the rendering names every feature, every expanded run and every runtime, and holds no path or clock', () => {
  const features = loadFeatures(ROOT);
  const runs = expandScenarios(loadScenarios(ROOT, { features }), referenceArchives(ROOT));
  for (const feature of Object.keys(features.features)) assert.ok(rendered.includes(`| \`${feature}\` | + | - | - |`), feature);
  for (const run of runs) assert.ok(rendered.includes(`| \`${run.id}\` | ${run.scenario.cites.map((c) => `\`${c}\``).join(', ')} | passed | no report | no report |`), run.id);
  assert.ok(rendered.includes('| Feature | JavaScript (`prism-code`) | Rust (`prismpm`) | Python (`prism-python`) |'));
  assert.ok(rendered.includes('| `prismpm` | Rust |  | no report: no entrypoint registered |'));
  assert.ok(rendered.includes('Reference archives: `Calculator`.'));
  assert.ok(rendered.endsWith('None declared.\n'));
  assert.ok(!rendered.includes(ROOT) && !rendered.includes('/tmp') && !/20[0-9]{2}-[0-9]{2}-[0-9]{2}T/.test(rendered));
  assert.equal(renderMatrix({ root: ROOT, reports: { 'prism-code': report } }), rendered, 'deterministic');
});

test('a cell is + only when every scenario of the feature passed, - otherwise or without a report, and N/A when declared', () => {
  const r = synthetic([
    ['a[X]', 'load', 'passed'],
    ['a[Y]', 'load', 'passed'],
    ['b', 'inspect', 'passed'],
    ['c', 'inspect', 'xfail', 'later'],
    ['d', 'vectors', 'failed'],
    ['e', 'history', 'error'],
    ['f', 'replay', 'unexpected-pass', 'stale'],
    ['g', 'check', 'not-applicable', 'no'],
    ['h', 'check', 'not-applicable', 'no'],
    ['i', 'world', 'not-applicable', 'no'],
    ['j', 'world', 'passed'],
  ]);
  assert.equal(cellOf('load', r), CELLS.PASS);
  assert.equal(cellOf('inspect', r), CELLS.FAIL);
  assert.equal(cellOf('vectors', r), CELLS.FAIL);
  assert.equal(cellOf('history', r), CELLS.FAIL);
  assert.equal(cellOf('replay', r), CELLS.FAIL);
  assert.equal(cellOf('check', r), CELLS.NOT_APPLICABLE);
  assert.equal(cellOf('world', r), CELLS.FAIL);
  assert.equal(cellOf('attestations', r), CELLS.FAIL);
  assert.equal(cellOf('load', null), CELLS.FAIL);
});

test('expected failures and not-applicable features are listed with their reasons', () => {
  const r = synthetic([
    ['c', 'inspect', 'xfail', 'inspect lands with the wire change'],
    ['f', 'replay', 'unexpected-pass', 'stale'],
    ['g', 'check', 'not-applicable', 'no verdict in this runtime'],
    ['h', 'check', 'not-applicable', 'no verdict in this runtime'],
  ]);
  const text = renderMatrix({ root: ROOT, reports: { prismpm: r } });
  assert.ok(text.includes('| `prismpm` | `c` | xfail | inspect lands with the wire change |'));
  assert.ok(text.includes('| `prismpm` | `f` | unexpected-pass | stale |'));
  assert.ok(text.includes('| `prismpm` | `check` | no verdict in this runtime |'));
  assert.equal(text.split('no verdict in this runtime').length, 2, 'one row per feature');
  assert.ok(text.includes('| `check` | - | N/A | - |'));
  assert.ok(text.includes('| `prism-code` | JavaScript |  | no report: no entrypoint registered |'));
});

test('a report for an unregistered runtime or of a foreign shape is refused', () => {
  assert.throws(() => renderMatrix({ root: ROOT, reports: { 'prism-cobol': report } }), (error) => error instanceof ConformanceError && error.code === 'matrix.runtime');
  assert.throws(() => renderMatrix({ root: ROOT, reports: { prismpm: { ...report, schema: 'other' } } }), (error) => error instanceof ConformanceError && error.code === 'report.schema');
  assert.throws(() => renderMatrix({ root: ROOT, reports: { prismpm: { ...report, scenarios: [{ id: 'x', scenario: 'x', feature: 'load', cites: [], status: 'maybe' }] } } }), (error) => error instanceof ConformanceError && error.code === 'report.status');
});

test('the command line writes the matrix, passes --check on a current file and fails it on a stale one', () => {
  const directory = scratch();
  const reportPath = join(directory, 'prism-code.json');
  writeJson(reportPath, report);
  const out = join(directory, 'COMPLIANCE.md');
  const written = cli(['matrix', reportPath, '--out', out]);
  assert.equal(written.status, 0, written.stderr);
  assert.equal(lf(readFileSync(out, 'utf8')), lf(rendered));
  const ok = cli(['matrix', reportPath, '--check', out]);
  assert.equal(ok.status, 0, ok.stderr);
  assert.match(ok.stdout, /is current/);
  writeFileSync(out, readFileSync(out, 'utf8').replace('| `load` | + |', '| `load` | - |'));
  const stale = cli(['matrix', reportPath, '--check', out]);
  assert.equal(stale.status, 1);
  assert.match(stale.stderr, /is stale/);
  const crlf = join(directory, 'crlf.md');
  writeFileSync(crlf, lf(rendered).replace(/\n/g, '\r\n'));
  assert.equal(cli(['matrix', reportPath, '--check', crlf]).status, 0, 'line endings do not make a file stale');
  assert.equal(cli(['matrix']).status, 64);
  assert.equal(cli(['matrix', reportPath]).status, 64);
  assert.equal(cli(['matrix', reportPath, '--out', out, '--check', out]).status, 64);
  assert.equal(cli(['matrix', reportPath, reportPath, '--out', out]).status, 64, 'two reports for one runtime');
  assert.equal(cli(['matrix', reportPath, '--check', join(directory, 'missing.md')]).status, 1);
  assert.equal(readJson(join(ROOT, 'package.json')).scripts.matrix.includes('--out ../../../COMPLIANCE.md'), true);
});
