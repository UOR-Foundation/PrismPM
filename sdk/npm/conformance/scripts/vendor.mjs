/**
 * Copies the fixtures the suite runs over from `prism-code` into `fixtures/`,
 * byte for byte, and derives the one archive no package holds.
 *
 * - Every reference archive and its recorded history, `fixtures/<Name>.holo`
 *   and `fixtures/<Name>.history.json` in sdk/npm/prism-code/fixtures. A second
 *   reference archive joins the suite by landing there and running this script.
 * - The changed model of #284, `fixtures/negative/Calculator.changed-model.holo`,
 *   from prism-code's test fixtures: it passes its own vectors and diverges on the
 *   recorded history.
 * - `fixtures/derived/Calculator.attested.holo`: the Calculator with the unsigned
 *   provenance bundle of #268 appended as its attestation section, derived with
 *   `appendAttestations`. It is a test input for the attestations scenarios, never
 *   reference evidence.
 *
 * Why copy: a runtime in another repository installs this package from npm
 * and must find the archives beside the scenarios.
 *
 * `node scripts/vendor.mjs` writes the files. `--check` fails when any of them
 * differs from what its origin would produce today, or when a reference
 * archive in prism-code has no copy here, so the copies cannot drift.
 */
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { appendAttestations } from '../../prism-code/src/attestations.mjs';

const here = (relative) => fileURLToPath(new URL(relative, import.meta.url));
const root = here('..');
const prismCode = here('../../prism-code');
const check = process.argv.includes('--check');

const bytes = (path) => new Uint8Array(readFileSync(path));
const equal = (a, b) => a.length === b.length && a.every((byte, i) => byte === b[i]);

/** The LF-free bundle line of the provenance fixture. */
function provenanceLine() {
  const line = bytes(join(prismCode, 'test/fixtures/Calculator.provenance-min.bundle.jsonl'));
  return line[line.length - 1] === 0x0a ? line.subarray(0, line.length - 1) : line;
}

/** Every file to hold: its package path and the bytes its origin produces. */
export function expected() {
  const out = new Map();
  const sources = join(prismCode, 'fixtures');
  for (const name of readdirSync(sources).sort()) {
    if (name.endsWith('.holo') || name.endsWith('.history.json')) out.set(`fixtures/${name}`, bytes(join(sources, name)));
  }
  out.set('fixtures/negative/Calculator.changed-model.holo', bytes(join(prismCode, 'test/fixtures/Calculator.changed-model.holo')));
  out.set('fixtures/derived/Calculator.attested.holo', appendAttestations(bytes(join(sources, 'Calculator.holo')), [provenanceLine()]));
  return out;
}

const files = expected();
const problems = [];
for (const [relative, want] of files) {
  const path = join(root, relative);
  if (check) {
    if (!existsSync(path)) problems.push(`${relative} is missing; run node scripts/vendor.mjs`);
    else if (!equal(bytes(path), want)) problems.push(`${relative} differs from its origin; run node scripts/vendor.mjs`);
  } else {
    mkdirSync(join(path, '..'), { recursive: true });
    writeFileSync(path, want);
    process.stdout.write(`wrote ${relative} (${want.length} bytes)\n`);
  }
}
// A reference archive removed from prism-code must leave here too, or the suite runs over a ghost.
for (const name of readdirSync(join(root, 'fixtures')).sort()) {
  if ((name.endsWith('.holo') || name.endsWith('.history.json')) && !files.has(`fixtures/${name}`)) {
    problems.push(`fixtures/${name} has no origin in sdk/npm/prism-code/fixtures; remove it`);
  }
}
if (problems.length > 0) {
  process.stderr.write(`${problems.join('\n')}\n`);
  process.exit(1);
}
if (check) process.stdout.write(`${files.size} vendored fixtures are current\n`);
