// The suite as data: every scenario validates, cites registered ids, names a declared feature and expands over the reference archives.
import assert from 'node:assert/strict';
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';
import { ACTIONS, EACH, SCENARIO_SCHEMA, bind, expandScenarios, loadFeatures, loadScenarios, referenceArchives, validateScenario } from '../src/scenarios.mjs';
import { ConformanceError } from '../src/util.mjs';
import { ROOT, REPO, readJson, scratch, suiteCopy, editScenario } from './helpers.mjs';

const features = loadFeatures(ROOT);
const scenarios = loadScenarios(ROOT, { features });
const archives = referenceArchives(ROOT);

test('every scenario file validates, is named by its id, and the ids are unique and sorted', () => {
  const files = readdirSync(join(ROOT, 'scenarios')).filter((f) => f.endsWith('.json')).sort();
  assert.equal(scenarios.length, files.length);
  assert.deepEqual(
    scenarios.map((s) => `${s.id}.json`),
    files,
  );
  assert.ok(scenarios.length >= 16);
  for (const scenario of scenarios) {
    assert.equal(scenario.schema, SCENARIO_SCHEMA);
    for (const step of scenario.steps) assert.ok(ACTIONS.includes(step.action));
  }
});

test('every cited id is registered in CONFORMANCE.md, the one registry', () => {
  const registry = readFileSync(join(REPO, 'CONFORMANCE.md'), 'utf8');
  const registered = new Set([...registry.matchAll(/^\| `([A-Z]{2}-[0-9]{2})` \|/gm)].map((m) => m[1]));
  assert.ok(registered.size > 100);
  for (const scenario of scenarios) {
    for (const cite of scenario.cites) assert.ok(registered.has(cite), `${scenario.id} cites ${cite}, which CONFORMANCE.md does not register`);
  }
  assert.ok(scenarios.every((s) => s.cites.includes('HO-31')), 'every scenario cites HO-31, the id that owns the suite');
});

test('every feature has at least one scenario and every scenario names a declared feature', () => {
  const named = new Set(scenarios.map((s) => s.feature));
  for (const feature of Object.keys(features.features)) assert.ok(named.has(feature), `feature ${feature} has no scenario`);
  for (const scenario of scenarios) assert.ok(Object.hasOwn(features.features, scenario.feature));
  assert.deepEqual(Object.keys(features.features), ['load', 'inspect', 'vectors', 'history', 'replay', 'check', 'world', 'attestations']);
});

test('every scenario runs over each reference archive or names a fixture that exists', () => {
  for (const scenario of scenarios) {
    if (scenario.archive !== EACH) assert.ok(existsSync(join(ROOT, scenario.archive)), `${scenario.id}: ${scenario.archive}`);
    for (const step of scenario.steps) {
      if (step.archive !== '$archive') assert.ok(existsSync(join(ROOT, step.archive)), `${scenario.id}: ${step.archive}`);
      for (const history of step.histories ?? []) if (history !== '$history') assert.ok(existsSync(join(ROOT, history)), `${scenario.id}: ${history}`);
    }
  }
});

test('the reference archives are every fixtures/*.holo with its recorded history, and the Calculator is one', () => {
  assert.ok(archives.some((a) => a.name === 'Calculator'));
  for (const archive of archives) {
    assert.equal(archive.archive, `fixtures/${archive.name}.holo`);
    assert.equal(archive.history, `fixtures/${archive.name}.history.json`);
    assert.ok(existsSync(join(ROOT, archive.history)));
  }
  const without = scratch();
  mkdirSync(join(without, 'fixtures'));
  writeFileSync(join(without, 'fixtures', 'Lonely.holo'), 'HOLO');
  assert.throws(() => referenceArchives(without), (error) => error instanceof ConformanceError && error.code === 'fixtures.history');
});

test('an each scenario expands to one run per reference archive with $archive and $history bound; another expands to one', () => {
  const runs = expandScenarios(scenarios, archives);
  const each = scenarios.filter((s) => s.archive === EACH);
  const single = scenarios.filter((s) => s.archive !== EACH);
  assert.equal(runs.length, each.length * archives.length + single.length);
  assert.deepEqual(
    runs.map((r) => r.id),
    [...runs.map((r) => r.id)].sort(),
  );
  const load = runs.find((r) => r.id === 'load-archive[Calculator]');
  assert.equal(load.archive, 'Calculator');
  assert.equal(load.steps[0].archive, 'fixtures/Calculator.holo');
  const replay = runs.find((r) => r.id === 'replay-unchanged[Calculator]');
  assert.deepEqual(replay.steps[0].histories, ['fixtures/Calculator.history.json']);
  const divergence = runs.find((r) => r.id === 'replay-divergence');
  assert.equal(divergence.archive, null);
  assert.equal(divergence.steps[1].archive, 'fixtures/negative/Calculator.changed-model.holo');
  const json = JSON.stringify(runs.map((r) => r.steps));
  assert.ok(!json.includes('"$archive"') && !json.includes('"$history"'), 'no placeholder survives expansion');
});

test('bind replaces the placeholders deeply and leaves every other value alone', () => {
  const bound = bind({ a: '$archive', b: ['$history', 'x', 1, null], c: { d: '$archive$' } }, { $archive: 'A', $history: 'H' });
  assert.deepEqual(bound, { a: 'A', b: ['H', 'x', 1, null], c: { d: '$archive$' } });
});

test('a scenario that cites nothing, names an unknown feature, an unknown operator or an unknown member is refused', () => {
  const base = readJson(join(ROOT, 'scenarios', 'load-archive.json'));
  const refuse = (edit, code) => {
    const scenario = structuredClone(base);
    edit(scenario);
    assert.throws(() => validateScenario(scenario, { features, path: 'x.json' }), (error) => error instanceof ConformanceError && error.code === code, code);
  };
  refuse((s) => (s.cites = []), 'scenario.cites');
  refuse((s) => (s.cites = ['ho-20']), 'scenario.cites');
  refuse((s) => (s.feature = 'teleport'), 'scenario.feature');
  refuse((s) => (s.steps[0].expect.result = { $every: 1 }), 'scenario.pattern');
  refuse((s) => (s.extra = 1), 'scenario.member');
  refuse((s) => delete s.revision, 'scenario.member');
  refuse((s) => (s.revision = 'v4'), 'scenario.revision');
  refuse((s) => (s.id = 'Load_Archive'), 'scenario.id');
  refuse((s) => (s.steps = []), 'scenario.steps');
  refuse((s) => (s.steps[0].action = 'teleport'), 'scenario.action');
  refuse((s) => (s.steps[0].expect.exit = 'zero'), 'scenario.expect');
  refuse((s) => (s.steps[0].policy = {}), 'scenario.policy');
  refuse((s) => (s.steps[0].histories = ['x']), 'scenario.histories');
  refuse((s) => (s.steps[0].archive = '../outside.holo'), 'scenario.path');
  refuse((s) => (s.archive = 'C:/x.holo'), 'scenario.path');
  refuse((s) => {
    s.archive = 'fixtures/Calculator.holo';
  }, 'scenario.archive');
  assert.throws(() => validateScenario(base, { features, path: 'x.json', expectedId: 'other' }), /is not the file name/);
  const copy = suiteCopy((root) => editScenario(root, 'load-archive', (s) => (s.id = 'inspect-documents')));
  assert.throws(() => loadScenarios(copy), /is not the file name/);
});
