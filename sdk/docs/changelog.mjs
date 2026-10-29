#!/usr/bin/env node
/**
 * The changelog check: the release entries of CHANGELOG.md against STABILITY.md
 * section 2 (the four headers, and the pre-1.0 bump rule). Reports every
 * problem as `file:line: message`, then exits 1.
 *
 *   node sdk/docs/changelog.mjs            check CHANGELOG.md at the repository root
 *   node sdk/docs/changelog.mjs <path>     check this file
 */
import { readFileSync } from 'node:fs';
import { join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { LEGACY_THROUGH, releaseEntryProblems } from './lib/changelog.mjs';
import { ROOT } from './lib/spec.mjs';

function main(argv) {
  const file = argv.length ? resolve(process.cwd(), argv[0]) : join(ROOT, 'CHANGELOG.md');
  const rel = relative(ROOT, file);
  const name = rel.startsWith('..') ? file : rel.split(sep).join('/');
  const problems = releaseEntryProblems(readFileSync(file, 'utf8'));
  for (const problem of problems) console.error(`${name}:${problem.line}: ${problem.message}`);
  if (problems.length) {
    console.error(`${problems.length} problem(s) in ${name}`);
    return 1;
  }
  console.log(`changelog ok: ${name} (entries after ${LEGACY_THROUGH} checked)`);
  return 0;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  process.exit(main(process.argv.slice(2)));
}
