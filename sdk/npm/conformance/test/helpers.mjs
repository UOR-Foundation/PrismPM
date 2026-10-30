// Shared by the tests: paths, a scratch directory, and a copy of the suite's data that a test may change.
import { cpSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

export const here = (relative) => fileURLToPath(new URL(relative, import.meta.url));
export const ROOT = here('..');
export const REPO = here('../../../..');
export const BIN = here('../bin/prism-conformance.mjs');
export const SHIM = here('../shims/prism-code.mjs');
export const PRISM_CODE = here('../../prism-code/bin/prism-code.mjs');
export const SPEC_CONSTANTS = here('../../spec/constants.mjs');
/** The registered JavaScript entrypoint, as an absolute command, for a suite copied elsewhere. */
export const JS_ENTRYPOINT = [process.execPath, SHIM];

export const scratch = (prefix = 'prism-conformance-test-') => mkdtempSync(join(tmpdir(), prefix));

export const readJson = (path) => JSON.parse(readFileSync(path, 'utf8'));
export const writeJson = (path, value) => writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);

/**
 * A copy of the suite's data in a scratch directory: features, runtimes,
 * xfail, scenarios and fixtures. `edit(root)` changes it before use.
 */
export function suiteCopy(edit = () => {}) {
  const root = scratch('prism-conformance-suite-');
  for (const entry of ['features.json', 'runtimes.json']) cpSync(join(ROOT, entry), join(root, entry));
  for (const directory of ['xfail', 'scenarios', 'fixtures']) cpSync(join(ROOT, directory), join(root, directory), { recursive: true });
  edit(root);
  return root;
}

/** Edit one scenario file of a copy in place. */
export function editScenario(root, id, edit) {
  const path = join(root, 'scenarios', `${id}.json`);
  const scenario = readJson(path);
  edit(scenario);
  writeJson(path, scenario);
}

let fakes = 0;
/** Write a fake entrypoint, an ES module whose body runs for every step, as a file of its own in `root`. */
export function fakeEntrypoint(root, body) {
  fakes += 1;
  const path = join(root, `fake-shim-${fakes}.mjs`);
  writeFileSync(path, body);
  return [process.execPath, path];
}
