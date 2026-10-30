/**
 * The suite as data: `features.json`, `scenarios/*.json` and the reference
 * archives in `fixtures/`. Loading validates every file, so a scenario that
 * cites nothing, names an unknown feature or uses an operator the matcher
 * lacks is refused before any runtime runs.
 *
 * A scenario with `archive: "each"` expands to one run per reference archive,
 * `fixtures/<Name>.holo`, with `$archive` bound to it and `$history` to its
 * recorded history `fixtures/<Name>.history.json`. Its expanded id is
 * `<id>[<Name>]`.
 */
import { existsSync, readdirSync, statSync } from 'node:fs';
import { basename, join, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { validatePattern } from './match.mjs';
import { fail, isObject, readJson } from './util.mjs';

export const SCENARIO_SCHEMA = 'prism/conformance-scenario/1';
export const FEATURES_SCHEMA = 'prism/conformance-features/1';
/** The actions a step may ask of an entrypoint, in the order the contract lists them. */
export const ACTIONS = Object.freeze(['inspect', 'vectors', 'history', 'replay', 'check']);
const STEP_MEMBERS = Object.freeze(['action', 'archive', 'histories', 'policy', 'expect']);
const SCENARIO_MEMBERS = Object.freeze(['schema', 'id', 'feature', 'cites', 'revision', 'archive', 'statement', 'steps']);
const EXPECT_MEMBERS = Object.freeze(['exit', 'result', 'error']);

export const ID = /^[a-z][a-z0-9]*(-[a-z0-9]+)*$/;
export const CITE = /^[A-Z]{2}-[0-9]{2}$/;
export const REVISION = /^[0-9]{4}-[0-9]{2}-[0-9]{2}$/;
export const EACH = 'each';

/** The package root: the directory holding `scenarios/`, `features.json` and `fixtures/`. */
export const PACKAGE_ROOT = fileURLToPath(new URL('..', import.meta.url));

export function loadFeatures(root = PACKAGE_ROOT) {
  const path = join(root, 'features.json');
  const file = readJson(path);
  if (!isObject(file) || file.schema !== FEATURES_SCHEMA) fail('features.schema', `${path} must declare schema ${FEATURES_SCHEMA}`);
  if (!isObject(file.features) || Object.keys(file.features).length === 0) fail('features.empty', `${path} declares no features`);
  for (const [name, feature] of Object.entries(file.features)) {
    if (!ID.test(name)) fail('features.name', `${path}: feature ${name} is not lowercase words joined by hyphens`);
    if (!isObject(feature) || typeof feature.title !== 'string' || !feature.title) fail('features.title', `${path}: feature ${name} has no title`);
    if (!Array.isArray(feature.definedBy) || feature.definedBy.length === 0 || !feature.definedBy.every((d) => typeof d === 'string' && d)) {
      fail('features.defined-by', `${path}: feature ${name} must say what defines it, an id or a document`);
    }
    const unknown = Object.keys(feature).filter((key) => !['title', 'definedBy'].includes(key));
    if (unknown.length > 0) fail('features.member', `${path}: feature ${name} has unknown members ${unknown.join(', ')}`);
  }
  return file;
}

/** A relative path inside the package that names no parent and no absolute root. */
function closedRelative(value, what, path) {
  if (typeof value !== 'string' || !value || value.startsWith('/') || /^[A-Za-z]:/.test(value) || value.includes('\\')) {
    fail('scenario.path', `${path}: ${what} must be a relative path with forward slashes, found ${JSON.stringify(value)}`);
  }
  if (value.split('/').some((part) => part === '' || part === '.' || part === '..')) fail('scenario.path', `${path}: ${what} may not name . or ..`);
  return value;
}

function validateStep(step, index, scenario, path) {
  const at = `${path}: steps[${index}]`;
  if (!isObject(step)) fail('scenario.step', `${at} is not an object`);
  const unknown = Object.keys(step).filter((key) => !STEP_MEMBERS.includes(key));
  if (unknown.length > 0) fail('scenario.step', `${at} has unknown members ${unknown.join(', ')}; a step takes ${STEP_MEMBERS.join(', ')}`);
  if (!ACTIONS.includes(step.action)) fail('scenario.action', `${at}: action ${JSON.stringify(step.action)} is not one of ${ACTIONS.join(', ')}`);
  const each = scenario.archive === EACH;
  if (step.archive === undefined) fail('scenario.archive', `${at} names no archive`);
  if (step.archive === '$archive') {
    if (!each) fail('scenario.archive', `${at} binds $archive, but the scenario does not run over each archive`);
  } else closedRelative(step.archive, 'archive', at);
  if (step.histories !== undefined) {
    if (step.action !== 'replay') fail('scenario.histories', `${at}: only replay takes histories`);
    if (!Array.isArray(step.histories) || step.histories.length === 0) fail('scenario.histories', `${at}: histories must be a non-empty array`);
    for (const history of step.histories) {
      if (history === '$history') {
        if (!each) fail('scenario.histories', `${at} binds $history, but the scenario does not run over each archive`);
      } else closedRelative(history, 'history', at);
    }
  } else if (step.action === 'replay') fail('scenario.histories', `${at}: replay takes histories`);
  if (step.policy !== undefined) {
    if (step.action !== 'check') fail('scenario.policy', `${at}: only check takes a policy`);
    if (!isObject(step.policy)) fail('scenario.policy', `${at}: policy must be an object`);
  }
  if (!isObject(step.expect)) fail('scenario.expect', `${at} has no expect object`);
  const unknownExpect = Object.keys(step.expect).filter((key) => !EXPECT_MEMBERS.includes(key));
  if (unknownExpect.length > 0) fail('scenario.expect', `${at}: expect has unknown members ${unknownExpect.join(', ')}; it takes ${EXPECT_MEMBERS.join(', ')}`);
  if (!Number.isInteger(step.expect.exit) || step.expect.exit < 0 || step.expect.exit > 255) fail('scenario.expect', `${at}: expect.exit must be an exit code`);
  try {
    if (step.expect.result !== undefined) validatePattern(step.expect.result, 'result');
    if (step.expect.error !== undefined) validatePattern(step.expect.error, 'error');
  } catch (cause) {
    fail('scenario.pattern', `${at}: ${cause.message}`);
  }
}

/** Validate one scenario object as read from `path`, whose base name must be its id. */
export function validateScenario(scenario, { features, path = '<scenario>', expectedId = null } = {}) {
  if (!isObject(scenario)) fail('scenario.object', `${path} is not an object`);
  if (scenario.schema !== SCENARIO_SCHEMA) fail('scenario.schema', `${path} must declare schema ${SCENARIO_SCHEMA}`);
  const unknown = Object.keys(scenario).filter((key) => !SCENARIO_MEMBERS.includes(key));
  if (unknown.length > 0) fail('scenario.member', `${path} has unknown members ${unknown.join(', ')}; a scenario takes ${SCENARIO_MEMBERS.join(', ')}`);
  for (const member of SCENARIO_MEMBERS) if (scenario[member] === undefined) fail('scenario.member', `${path} lacks ${member}`);
  if (typeof scenario.id !== 'string' || !ID.test(scenario.id)) fail('scenario.id', `${path}: id must be lowercase words joined by hyphens`);
  if (expectedId !== null && scenario.id !== expectedId) fail('scenario.id', `${path}: id ${scenario.id} is not the file name ${expectedId}`);
  if (features && !Object.hasOwn(features.features, scenario.feature)) {
    fail('scenario.feature', `${path}: feature ${JSON.stringify(scenario.feature)} is not in features.json (${Object.keys(features.features).join(', ')})`);
  }
  if (!Array.isArray(scenario.cites) || scenario.cites.length === 0) fail('scenario.cites', `${path}: cites must name at least one registered id`);
  for (const cite of scenario.cites) if (typeof cite !== 'string' || !CITE.test(cite)) fail('scenario.cites', `${path}: ${JSON.stringify(cite)} is not a conformance id such as HO-20`);
  if (new Set(scenario.cites).size !== scenario.cites.length) fail('scenario.cites', `${path}: cites repeats an id`);
  if (typeof scenario.revision !== 'string' || !REVISION.test(scenario.revision)) fail('scenario.revision', `${path}: revision must be a date, the holo format revision`);
  if (scenario.archive !== EACH) closedRelative(scenario.archive, 'archive', path);
  if (typeof scenario.statement !== 'string' || !scenario.statement.trim()) fail('scenario.statement', `${path}: statement must be one sentence`);
  if (!Array.isArray(scenario.steps) || scenario.steps.length === 0) fail('scenario.steps', `${path}: steps must hold at least one step`);
  scenario.steps.forEach((step, index) => validateStep(step, index, scenario, path));
  return scenario;
}

/** Every scenario under `scenarios/`, validated, sorted by id. */
export function loadScenarios(root = PACKAGE_ROOT, { features = loadFeatures(root) } = {}) {
  const directory = join(root, 'scenarios');
  if (!existsSync(directory)) fail('scenarios.missing', `${directory} does not exist`);
  const files = readdirSync(directory).filter((name) => name.endsWith('.json')).sort();
  if (files.length === 0) fail('scenarios.empty', `${directory} holds no scenario`);
  const scenarios = files.map((name) => {
    const path = join(directory, name);
    return validateScenario(readJson(path), { features, path, expectedId: basename(name, '.json') });
  });
  const ids = new Set();
  for (const scenario of scenarios) {
    if (ids.has(scenario.id)) fail('scenario.id', `scenario ${scenario.id} is declared twice`);
    ids.add(scenario.id);
  }
  return scenarios;
}

/**
 * The reference archives: every `fixtures/<Name>.holo` with its recorded
 * history `fixtures/<Name>.history.json`. An archive without its history is
 * refused, because every scenario over each archive may replay it.
 */
export function referenceArchives(root = PACKAGE_ROOT) {
  const directory = join(root, 'fixtures');
  if (!existsSync(directory)) fail('fixtures.missing', `${directory} does not exist`);
  const names = readdirSync(directory)
    .filter((name) => name.endsWith('.holo') && statSync(join(directory, name)).isFile())
    .sort();
  if (names.length === 0) fail('fixtures.empty', `${directory} holds no reference archive`);
  return names.map((file) => {
    const name = basename(file, '.holo');
    const history = join(directory, `${name}.history.json`);
    if (!existsSync(history)) fail('fixtures.history', `reference archive ${file} has no recorded history ${name}.history.json beside it`);
    return { name, archive: `fixtures/${file}`, history: `fixtures/${name}.history.json` };
  });
}

/** Replace `$archive` and `$history` in a step, deeply. */
export function bind(value, bindings) {
  if (typeof value === 'string') return Object.hasOwn(bindings, value) ? bindings[value] : value;
  if (Array.isArray(value)) return value.map((item) => bind(item, bindings));
  if (isObject(value)) return Object.fromEntries(Object.entries(value).map(([key, member]) => [key, bind(member, bindings)]));
  return value;
}

/**
 * One run per scenario, or per reference archive for an `each` scenario:
 * { id, scenario, archive (the name, or null), steps (bound) }, sorted by id.
 */
export function expandScenarios(scenarios, archives) {
  const runs = [];
  for (const scenario of scenarios) {
    if (scenario.archive === EACH) {
      for (const reference of archives) {
        const bindings = { $archive: reference.archive, $history: reference.history };
        runs.push({
          id: `${scenario.id}[${reference.name}]`,
          scenario,
          archive: reference.name,
          steps: scenario.steps.map((step) => bind(step, bindings)),
        });
      }
    } else {
      runs.push({ id: scenario.id, scenario, archive: null, steps: scenario.steps });
    }
  }
  return runs.sort((a, b) => (a.id < b.id ? -1 : a.id > b.id ? 1 : 0));
}

/** Resolve a package-relative fixture path to an absolute one, refusing a path outside the root. */
export function fixturePath(root, relative) {
  const absolute = resolve(root, relative);
  const base = resolve(root);
  if (absolute !== base && !absolute.startsWith(base + sep)) fail('scenario.path', `${relative} leaves the package`);
  return absolute;
}
