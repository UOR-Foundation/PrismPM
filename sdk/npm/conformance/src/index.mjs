/**
 * @prism-code/conformance: the executable suite every Prism runtime runs.
 *
 * The suite is data (`scenarios/`, `features.json`, `runtimes.json`, `xfail/`)
 * and a runner that drives a runtime through its entrypoint shim, one process
 * per step. Design: docs/proposals/287-conformance.md.
 */
export { ConformanceError, canonicalJson } from './util.mjs';
export { OPERATORS, matches, mismatches, validatePattern } from './match.mjs';
export {
  ACTIONS,
  EACH,
  FEATURES_SCHEMA,
  PACKAGE_ROOT,
  SCENARIO_SCHEMA,
  bind,
  expandScenarios,
  fixturePath,
  loadFeatures,
  loadScenarios,
  referenceArchives,
  validateScenario,
} from './scenarios.mjs';
export { RUNTIMES_SCHEMA, XFAIL_SCHEMA, checkXfail, loadRuntimes, loadXfail, resolveEntrypoint } from './runtimes.mjs';
export {
  FAILING,
  OUTCOME_SCHEMA,
  REPORT_SCHEMA,
  STATUSES,
  STEP_SCHEMA,
  STEP_TIMEOUT,
  applyXfail,
  compare,
  failed,
  parseOutcome,
  runScenario,
  runStep,
  runSuite,
  stepFile,
  summarize,
  validateReport,
} from './runner.mjs';
export { CELLS, HEADER, cellOf, current, renderMatrix } from './matrix.mjs';
export { checkGate, gate, loadRevisions } from './gate.mjs';
