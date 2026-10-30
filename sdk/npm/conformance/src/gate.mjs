/**
 * The merge gate on the holo format. A holo does not carry its revision date;
 * `FORMAT_REVISIONS` in `@prism-code/spec` is the one declaration of every
 * revision. The gate fails unless every revision is named by a scenario that
 * loads an archive of it, every scenario names a revision that exists, and
 * every reference archive is loaded by a scenario, so a revision cannot be
 * declared without a scenario and a scenario cannot claim a revision nothing
 * reads.
 */
import { existsSync } from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { expandScenarios, loadFeatures, loadScenarios, referenceArchives, EACH, PACKAGE_ROOT } from './scenarios.mjs';
import { fail, isObject } from './util.mjs';

/** Where the spec's constants are read from: the repository sibling, else the installed package. */
export async function loadRevisions(specPath = null) {
  let module;
  if (specPath) {
    module = await import(pathToFileURL(specPath).href);
  } else {
    const sibling = join(PACKAGE_ROOT, '..', 'spec', 'constants.mjs');
    if (existsSync(sibling)) module = await import(pathToFileURL(sibling).href);
    else {
      try {
        module = await import('@prism-code/spec');
      } catch (cause) {
        fail('gate.spec', `@prism-code/spec is not installed and ${sibling} does not exist; pass --spec <constants.mjs>: ${cause.message}`);
      }
    }
  }
  if (!isObject(module.FORMAT_REVISIONS) || Object.keys(module.FORMAT_REVISIONS).length === 0) fail('gate.spec', 'the spec declares no FORMAT_REVISIONS');
  return Object.keys(module.FORMAT_REVISIONS).sort();
}

/** True when a step is an `inspect` whose expectation pins the archive's revision to `revision`. */
function pinsRevision(step, revision) {
  return step.action === 'inspect' && step.expect.exit === 0 && isObject(step.expect.result) && step.expect.result.revision === revision;
}

/**
 * The gate's findings: an empty array passes. `revisions` are the spec's
 * keys; `scenarios` and `archives` default to the package's.
 */
export function checkGate({ revisions, scenarios, archives }) {
  const problems = [];
  const named = new Map(revisions.map((revision) => [revision, []]));
  for (const scenario of scenarios) {
    if (!named.has(scenario.revision)) {
      problems.push(`scenario ${scenario.id} names revision ${scenario.revision}, which @prism-code/spec does not declare (${revisions.join(', ')})`);
      continue;
    }
    named.get(scenario.revision).push(scenario);
  }
  for (const [revision, owners] of named) {
    if (owners.length === 0) {
      problems.push(`format revision ${revision} is named by no scenario; a revision cannot merge without one that loads an archive of it`);
      continue;
    }
    const evidence = owners.filter((scenario) => scenario.steps.some((step) => pinsRevision(step, revision)));
    if (evidence.length === 0) {
      problems.push(`format revision ${revision} is named by ${owners.map((s) => s.id).join(', ')}, but none has an inspect step whose expected result.revision is ${revision}`);
    }
  }
  const loaded = new Set();
  for (const run of expandScenarios(scenarios, archives)) {
    if (run.archive !== null) loaded.add(run.archive);
  }
  for (const archive of archives) {
    if (!loaded.has(archive.name)) problems.push(`reference archive ${archive.archive} is loaded by no scenario over each archive`);
  }
  if (!scenarios.some((scenario) => scenario.archive === EACH)) problems.push('no scenario runs over each reference archive');
  return problems;
}

/** Run the gate on the package. Returns the problems. */
export async function gate({ root = PACKAGE_ROOT, specPath = null } = {}) {
  const revisions = await loadRevisions(specPath);
  const scenarios = loadScenarios(root, { features: loadFeatures(root) });
  const archives = referenceArchives(root);
  return { revisions, problems: checkGate({ revisions, scenarios, archives }) };
}
