// The suite against prism-code: every scenario passes, the report is deterministic, and a scenario made to fail or
// an entrypoint that misbehaves is reported as such, never as a pass.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { join } from 'node:path';
import { test } from 'node:test';
import { REPORT_SCHEMA, STATUSES, failed, parseOutcome, runSuite } from '../src/runner.mjs';
import { canonicalJson, ConformanceError } from '../src/util.mjs';
import { expandScenarios, loadScenarios, referenceArchives } from '../src/scenarios.mjs';
import { BIN, JS_ENTRYPOINT, ROOT, readJson, scratch, suiteCopy, editScenario, fakeEntrypoint } from './helpers.mjs';

const prismCodeVersion = readJson(join(ROOT, '..', 'prism-code', 'package.json')).version;
const cli = (args, options = {}) => spawnSync(process.execPath, [BIN, ...args], { encoding: 'utf8', cwd: ROOT, ...options });

test('prism-code passes every scenario over every reference archive, and the report says so', () => {
  const report = runSuite({ root: ROOT, runtime: 'prism-code', scratch: scratch() });
  assert.equal(report.schema, REPORT_SCHEMA);
  assert.deepEqual(report.runtime, { name: 'prism-code', language: 'JavaScript', version: prismCodeVersion });
  assert.deepEqual(report.archives, referenceArchives(ROOT).map((a) => a.name));
  const runs = expandScenarios(loadScenarios(ROOT), referenceArchives(ROOT));
  assert.deepEqual(
    report.scenarios.map((row) => row.id),
    runs.map((run) => run.id),
  );
  const notPassed = report.scenarios.filter((row) => row.status !== 'passed');
  assert.deepEqual(notPassed, [], JSON.stringify(notPassed, null, 2));
  assert.equal(report.summary.passed, runs.length);
  assert.ok(!failed(report));
  for (const status of STATUSES) assert.equal(typeof report.summary[status], 'number');
  assert.ok(!JSON.stringify(report).includes(ROOT.replace(/\\/g, '\\\\')), 'the report holds no host path');
});

test('two runs of the same scenarios write the same report', () => {
  const only = ['load-archive', 'check-minimum-p0'];
  const first = runSuite({ root: ROOT, runtime: 'prism-code', only, scratch: scratch() });
  const second = runSuite({ root: ROOT, runtime: 'prism-code', only, scratch: scratch() });
  assert.equal(canonicalJson(first), canonicalJson(second));
  assert.deepEqual(
    first.scenarios.map((r) => r.id),
    ['check-minimum-p0[Calculator]', 'load-archive[Calculator]'],
  );
});

test('a scenario whose expectation does not hold is failed, with every mismatch by path, and later steps do not run', () => {
  const root = suiteCopy((copy) => {
    editScenario(copy, 'load-archive', (s) => {
      s.steps[0].expect.result.physical_version = 5;
      s.steps[0].expect.result.flags = 1;
      s.steps.push({ action: 'inspect', archive: '$archive', expect: { exit: 0 } });
    });
  });
  const report = runSuite({ root, runtime: 'prism-code', entrypoint: JS_ENTRYPOINT, only: ['load-archive'], scratch: scratch() });
  const [row] = report.scenarios;
  assert.equal(row.status, 'failed');
  assert.equal(row.steps, 2);
  assert.deepEqual(
    row.failures.map((f) => [f.step, f.path]),
    [
      [0, 'result.physical_version'],
      [0, 'result.flags'],
    ],
  );
  assert.ok(failed(report));
  assert.equal(report.summary.failed, 1);
});

test('an entrypoint that exits without an outcome, prints no outcome, or prints a foreign shape is an error for the scenario', () => {
  const root = scratch();
  const cases = [
    [fakeEntrypoint(root, 'process.exit(3);'), /exited 3 without an outcome/],
    [fakeEntrypoint(root, 'process.stdout.write("");'), /printed no outcome/],
    [fakeEntrypoint(root, 'process.stdout.write("not json");'), /not one JSON object/],
    [fakeEntrypoint(root, 'process.stdout.write(JSON.stringify({ schema: "other/1" }));'), /declares schema "other\/1"/],
    [fakeEntrypoint(root, 'process.stdout.write(JSON.stringify({ schema: "prism/conformance-outcome/1", runtime: { name: "prism-code", version: "0" }, exit: 0, result: null }));'), /no error member/],
  ];
  for (const [entrypoint, expected] of cases) {
    const report = runSuite({ root: ROOT, runtime: 'prism-code', entrypoint, only: ['check-no-verdict'], scratch: scratch() });
    const [row] = report.scenarios;
    assert.equal(row.status, 'error');
    assert.match(row.failures[0].error, expected);
    assert.ok(failed(report));
  }
});

test('parseOutcome accepts exactly the contract and names what is wrong otherwise', () => {
  const good = { schema: 'prism/conformance-outcome/1', runtime: { name: 'x', version: '1' }, exit: 0, result: { a: 1 }, error: null };
  assert.deepEqual(parseOutcome(`${JSON.stringify(good)}\n`), { outcome: good });
  assert.match(parseOutcome(JSON.stringify({ ...good, extra: 1 })).problem, /unknown members extra/);
  assert.match(parseOutcome(JSON.stringify({ ...good, runtime: {} })).problem, /name its runtime/);
  assert.match(parseOutcome(JSON.stringify({ ...good, exit: -1 })).problem, /integer exit/);
  assert.match(parseOutcome(JSON.stringify({ ...good, error: 'x' })).problem, /error is null or/);
  const { result, ...noResult } = good;
  assert.match(parseOutcome(JSON.stringify(noResult)).problem, /no result/);
  assert.match(parseOutcome('[]').problem, /not an object/);
});

test('an entrypoint answering for another runtime is refused, and a runtime without an entrypoint cannot run', () => {
  const root = scratch();
  const other = fakeEntrypoint(root, 'process.stdout.write(JSON.stringify({ schema: "prism/conformance-outcome/1", runtime: { name: "prismpm", version: "0" }, exit: 0, result: null, error: null }));');
  assert.throws(
    () => runSuite({ root: ROOT, runtime: 'prism-code', entrypoint: other, only: ['check-no-verdict'], scratch: scratch() }),
    (error) => error instanceof ConformanceError && error.code === 'runtimes.name',
  );
  assert.throws(() => runSuite({ root: ROOT, runtime: 'prismpm', scratch: scratch() }), (error) => error instanceof ConformanceError && error.code === 'runtimes.entrypoint');
  assert.throws(() => runSuite({ root: ROOT, runtime: 'prism-cobol', scratch: scratch() }), (error) => error instanceof ConformanceError && error.code === 'runtimes.unknown');
  assert.throws(() => runSuite({ root: ROOT, runtime: 'prism-code', only: ['no-such-scenario'], scratch: scratch() }), (error) => error instanceof ConformanceError && error.code === 'scenario.unknown');
});

test('--entrypoint runs another runtime through its own shim, and every row of its report is a status', () => {
  const root = scratch();
  const entrypoint = fakeEntrypoint(
    root,
    'import { readFileSync } from "node:fs"; const step = JSON.parse(readFileSync(process.argv[2], "utf8")); process.stdout.write(JSON.stringify({ schema: "prism/conformance-outcome/1", runtime: { name: "prismpm", version: "0.0.0" }, exit: step.action === "check" ? 2 : 0, result: null, error: step.action === "check" ? { code: "archive.magic", message: "no" } : null }));',
  );
  const report = runSuite({ root: ROOT, runtime: 'prismpm', entrypoint, scratch: scratch() });
  assert.deepEqual(report.runtime, { name: 'prismpm', language: 'Rust', version: '0.0.0' });
  assert.ok(report.scenarios.every((row) => STATUSES.includes(row.status)));
  assert.equal(report.scenarios.find((r) => r.id === 'check-no-verdict').status, 'passed');
  assert.equal(report.scenarios.find((r) => r.id === 'load-archive[Calculator]').status, 'failed');
  assert.equal(Object.values(report.summary).reduce((a, b) => a + b, 0), report.scenarios.length);
});

test('the command line runs the suite, writes the report, lists the runs and refuses bad usage', () => {
  const directory = scratch();
  const reportPath = join(directory, 'report.json');
  const run = cli(['run', '--runtime', 'prism-code', '--scenario', 'load-archive', '--scenario', 'check-no-verdict', '--report', reportPath]);
  assert.equal(run.status, 0, run.stderr);
  assert.match(run.stdout, /^ok check-no-verdict passed\nok load-archive\[Calculator\] passed\n/);
  assert.match(run.stdout, /2 passed, 0 failed/);
  const report = readJson(reportPath);
  assert.equal(report.schema, REPORT_SCHEMA);
  assert.equal(report.scenarios.length, 2);
  const list = cli(['list']);
  assert.equal(list.status, 0);
  assert.match(list.stdout, /^attestations-absent\[Calculator\]\tattestations\tHO-31\t2026-10-23\n/);
  assert.equal(cli(['run']).status, 64);
  assert.equal(cli(['run', '--runtime']).status, 64);
  assert.equal(cli(['run', '--runtime', 'prism-code', '--nope']).status, 64);
  assert.equal(cli(['teleport']).status, 64);
  const unknown = cli(['run', '--runtime', 'prism-cobol']);
  assert.equal(unknown.status, 2);
  assert.match(unknown.stderr, /^runtimes\.unknown:/);
  assert.equal(cli(['--version']).stdout.trim(), readJson(join(ROOT, 'package.json')).version);
});
