/**
 * Run the suite against one runtime through its entrypoint.
 *
 * One process per step: the suite writes the step as a file, runs
 * `<entrypoint> <step-file>`, and reads one `prism/conformance-outcome/1`
 * object from stdout. The entrypoint's own exit code is 0 when it produced an
 * outcome, whatever the runtime's exit was; anything else is a suite error
 * for the scenario, never a pass.
 */
import { spawnSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { mismatches } from './match.mjs';
import { expandScenarios, fixturePath, loadFeatures, loadScenarios, referenceArchives, PACKAGE_ROOT } from './scenarios.mjs';
import { checkXfail, loadRuntimes, loadXfail, resolveEntrypoint } from './runtimes.mjs';
import { fail, isObject } from './util.mjs';

export const STEP_SCHEMA = 'prism/conformance-step/1';
export const OUTCOME_SCHEMA = 'prism/conformance-outcome/1';
export const REPORT_SCHEMA = 'prism/conformance-report/1';
/** Every status a run can end in. `failed`, `unexpected-pass` and `error` fail the suite. */
export const STATUSES = Object.freeze(['passed', 'failed', 'xfail', 'unexpected-pass', 'error', 'not-applicable']);
export const FAILING = Object.freeze(['failed', 'unexpected-pass', 'error']);
/** Milliseconds one step may take. A guest dispatch is milliseconds; a cold runtime start is seconds. */
export const STEP_TIMEOUT = 120_000;
const MAX_STDERR = 4000;

/** The outcome the entrypoint printed, or the reason it is not one. */
export function parseOutcome(stdout) {
  const text = String(stdout ?? '').trim();
  if (!text) return { problem: 'the entrypoint printed no outcome' };
  let outcome;
  try {
    outcome = JSON.parse(text);
  } catch (cause) {
    return { problem: `the entrypoint's stdout is not one JSON object: ${cause.message}` };
  }
  if (!isObject(outcome)) return { problem: 'the outcome is not an object' };
  if (outcome.schema !== OUTCOME_SCHEMA) return { problem: `the outcome declares schema ${JSON.stringify(outcome.schema)}, expected ${OUTCOME_SCHEMA}` };
  const unknown = Object.keys(outcome).filter((key) => !['schema', 'runtime', 'exit', 'result', 'error'].includes(key));
  if (unknown.length > 0) return { problem: `the outcome has unknown members ${unknown.join(', ')}` };
  if (!isObject(outcome.runtime) || typeof outcome.runtime.name !== 'string' || typeof outcome.runtime.version !== 'string') {
    return { problem: 'the outcome must name its runtime: { name, version }' };
  }
  if (!Number.isInteger(outcome.exit) || outcome.exit < 0) return { problem: 'the outcome has no integer exit' };
  if (!Object.hasOwn(outcome, 'result')) return { problem: 'the outcome has no result (null when the runtime printed none)' };
  if (outcome.error === undefined) return { problem: 'the outcome has no error member (null when there was none)' };
  if (outcome.error !== null && (!isObject(outcome.error) || typeof outcome.error.message !== 'string')) {
    return { problem: 'the outcome error is null or { code?, message }' };
  }
  return { outcome };
}

/** The step file an entrypoint reads: the action and its inputs with absolute paths, and a scratch directory. */
export function stepFile(root, run, index, scratch) {
  const step = run.steps[index];
  const file = {
    schema: STEP_SCHEMA,
    scenario: run.id,
    step: index,
    action: step.action,
    archive: fixturePath(root, step.archive),
    scratch,
  };
  if (step.histories) file.histories = step.histories.map((history) => fixturePath(root, history));
  if (step.policy) file.policy = step.policy;
  return file;
}

function clip(text) {
  const value = String(text ?? '');
  return value.length > MAX_STDERR ? `${value.slice(0, MAX_STDERR)}\n[clipped]` : value;
}

/**
 * Run one step. Returns { outcome } or { error, stderr } when the entrypoint
 * did not produce an outcome.
 */
export function runStep(entrypoint, file, path, { timeout = STEP_TIMEOUT } = {}) {
  writeFileSync(path, `${JSON.stringify(file, null, 2)}\n`);
  const [command, ...args] = entrypoint;
  const child = spawnSync(command, [...args, path], {
    cwd: file.scratch,
    encoding: 'utf8',
    timeout,
    maxBuffer: 64 * 1024 * 1024,
    windowsHide: true,
  });
  if (child.error) return { error: `the entrypoint could not run: ${child.error.code ?? child.error.message}`, stderr: '' };
  if (child.status !== 0) {
    return { error: `the entrypoint exited ${child.status ?? `by signal ${child.signal}`} without an outcome`, stderr: clip(child.stderr) };
  }
  const parsed = parseOutcome(child.stdout);
  if (parsed.problem) return { error: parsed.problem, stderr: clip(child.stderr) };
  return { outcome: parsed.outcome };
}

/** Compare one outcome with a step's expectation. Empty when it matches. */
export function compare(expect, outcome) {
  const out = [];
  if (outcome.exit !== expect.exit) out.push({ path: 'exit', expected: String(expect.exit), actual: String(outcome.exit) });
  if (expect.result !== undefined) out.push(...mismatches(expect.result, outcome.result, 'result'));
  if (expect.error !== undefined) out.push(...mismatches(expect.error, outcome.error, 'error'));
  return out;
}

function safeName(id) {
  return id.replace(/[^A-Za-z0-9_.-]+/g, '_');
}

/**
 * Run every step of one expanded scenario. Steps run in order and stop at
 * the first that does not match. Returns the run's row of the report, with
 * `status` passed, failed or error, before expected failures are applied.
 */
export function runScenario(root, entrypoint, run, { scratch, timeout } = {}) {
  const directory = join(scratch, safeName(run.id));
  mkdirSync(directory, { recursive: true });
  const row = { id: run.id, scenario: run.scenario.id, feature: run.scenario.feature, cites: [...run.scenario.cites], archive: run.archive, status: 'passed', steps: run.steps.length };
  let runtime = null;
  for (let index = 0; index < run.steps.length; index += 1) {
    const file = stepFile(root, run, index, directory);
    const result = runStep(entrypoint, file, join(directory, `step-${index}.json`), { timeout });
    if (result.error) {
      row.status = 'error';
      row.failures = [{ step: index, error: result.error, stderr: result.stderr }];
      return { row, runtime };
    }
    runtime = result.outcome.runtime;
    const differences = compare(run.steps[index].expect, result.outcome);
    if (differences.length > 0) {
      row.status = 'failed';
      row.failures = differences.map((difference) => ({ step: index, ...difference }));
      return { row, runtime };
    }
  }
  return { row, runtime };
}

/** Apply a runtime's declarations to a run's status. */
export function applyXfail(row, xfail) {
  if (Object.hasOwn(xfail.notApplicable, row.feature)) {
    return { ...row, status: 'not-applicable', reason: xfail.notApplicable[row.feature] };
  }
  const reason = xfail.xfail[row.id] ?? xfail.xfail[row.scenario];
  if (reason === undefined) return row;
  if (row.status === 'passed') return { ...row, status: 'unexpected-pass', reason };
  return { ...row, status: 'xfail', reason };
}

export function summarize(rows) {
  const summary = Object.fromEntries(STATUSES.map((status) => [status, 0]));
  for (const row of rows) summary[row.status] += 1;
  return summary;
}

/**
 * Run the suite against one runtime and return its report. `entrypoint`
 * overrides the registered one; `only` limits the runs to those ids (base or
 * expanded). A runtime with no entrypoint cannot run.
 */
export function runSuite({ root = PACKAGE_ROOT, runtime, entrypoint = null, only = null, scratch, timeout, log = () => {} }) {
  const runtimes = loadRuntimes(root);
  const registered = runtimes.runtimes[runtime];
  if (!registered) fail('runtimes.unknown', `runtime ${runtime} is not in runtimes.json (${Object.keys(runtimes.runtimes).join(', ')})`);
  const command = resolveEntrypoint(root, registered.entrypoint, entrypoint);
  if (!command) fail('runtimes.entrypoint', `runtime ${runtime} has no entrypoint registered; pass --entrypoint`);
  const features = loadFeatures(root);
  const scenarios = loadScenarios(root, { features });
  const archives = referenceArchives(root);
  const runs = expandScenarios(scenarios, archives);
  const xfail = checkXfail(loadXfail(root, runtime, runtimes), { runs, features });
  const selected = only ? runs.filter((run) => only.includes(run.id) || only.includes(run.scenario.id)) : runs;
  if (only && selected.length === 0) fail('scenario.unknown', `no scenario is named ${only.join(', ')}`);
  mkdirSync(scratch, { recursive: true });
  const rows = [];
  let version = null;
  for (const run of selected) {
    let row;
    if (Object.hasOwn(xfail.notApplicable, run.scenario.feature)) {
      row = applyXfail({ id: run.id, scenario: run.scenario.id, feature: run.scenario.feature, cites: [...run.scenario.cites], archive: run.archive, status: 'passed', steps: run.steps.length }, xfail);
    } else {
      const executed = runScenario(root, command, run, { scratch, timeout });
      if (executed.runtime) {
        if (executed.runtime.name !== runtime) fail('runtimes.name', `the entrypoint answers for runtime ${executed.runtime.name}, the suite ran ${runtime}`);
        version = executed.runtime.version;
      }
      row = applyXfail(executed.row, xfail);
    }
    rows.push(row);
    log(row);
  }
  return {
    schema: REPORT_SCHEMA,
    runtime: { name: runtime, language: registered.language, version },
    archives: archives.map((archive) => archive.name),
    scenarios: rows,
    summary: summarize(rows),
  };
}

/** True when a report fails the suite. */
export const failed = (report) => FAILING.some((status) => report.summary[status] > 0);

/** Validate a report read from a file, so the matrix never renders a foreign shape. */
export function validateReport(report, path = '<report>') {
  if (!isObject(report) || report.schema !== REPORT_SCHEMA) fail('report.schema', `${path} must declare schema ${REPORT_SCHEMA}`);
  if (!isObject(report.runtime) || typeof report.runtime.name !== 'string' || typeof report.runtime.language !== 'string') fail('report.runtime', `${path} names no runtime`);
  if (report.runtime.version !== null && typeof report.runtime.version !== 'string') fail('report.runtime', `${path}: runtime.version is a string or null`);
  if (!Array.isArray(report.archives)) fail('report.archives', `${path} lists no archives`);
  if (!Array.isArray(report.scenarios)) fail('report.scenarios', `${path} lists no scenarios`);
  for (const row of report.scenarios) {
    if (!isObject(row) || typeof row.id !== 'string' || typeof row.scenario !== 'string' || typeof row.feature !== 'string' || !Array.isArray(row.cites)) {
      fail('report.row', `${path}: a scenario row lacks id, scenario, feature or cites`);
    }
    if (!STATUSES.includes(row.status)) fail('report.status', `${path}: ${row.id} has status ${JSON.stringify(row.status)}`);
  }
  if (!isObject(report.summary)) fail('report.summary', `${path} has no summary`);
  return report;
}
