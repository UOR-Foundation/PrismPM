#!/usr/bin/env node
/**
 * The changelog check: the release entries of the changelogs against
 * STABILITY.md section 2 (the four headers, and the pre-1.0 bump rule). Reports
 * every problem as `file:line: message`, then exits 1.
 *
 *   node sdk/docs/changelog.mjs            check CHANGELOG.md at the repository root and
 *                                          every sdk/npm/<package>/CHANGELOG.md that exists
 *   node sdk/docs/changelog.mjs <path>...  check these files
 *
 * The root changelog has entries up to 0.4.0 that predate the rule (see
 * lib/changelog.mjs). The package changelogs under sdk/npm/ are written by
 * release-please and have none.
 */
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { LEGACY_THROUGH, releaseEntryProblems } from './lib/changelog.mjs';
import { ROOT } from './lib/spec.mjs';

const NPM = join(ROOT, 'sdk', 'npm');

export function defaultTargets() {
  const packages = existsSync(NPM) ? readdirSync(NPM).sort().map((name) => join(NPM, name, 'CHANGELOG.md')) : [];
  return [join(ROOT, 'CHANGELOG.md'), ...packages.filter(existsSync)];
}

function check(file) {
  const rel = relative(ROOT, file);
  const name = rel.startsWith('..') ? file : rel.split(sep).join('/');
  const legacyThrough = name.startsWith('sdk/npm/') ? '0.0.0' : LEGACY_THROUGH;
  return releaseEntryProblems(readFileSync(file, 'utf8'), legacyThrough).map((p) => ({ name, ...p }));
}

function main(argv) {
  const files = argv.length ? argv.map((a) => resolve(process.cwd(), a)) : defaultTargets();
  const problems = files.flatMap(check);
  for (const problem of problems) console.error(`${problem.name}:${problem.line}: ${problem.message}`);
  if (problems.length) {
    console.error(`${problems.length} problem(s) in ${files.length} file(s)`);
    return 1;
  }
  console.log(`changelog ok: ${files.length} file(s) (root entries after ${LEGACY_THROUGH} checked)`);
  return 0;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  process.exit(main(process.argv.slice(2)));
}
