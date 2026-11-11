// Validate npm's pack manifest against the reviewed distribution inventory.
// Run after npm pack --dry-run --json > pack.json, as npm-prism-code.yml does.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

export const PACKAGE_LIMIT = 1519616; // 1484 KiB: 1088 KiB plus 72 KiB for setup, open and the bundled prism skill (about 63 KiB), plus 48 KiB for the machine half of setup, the build tiers, axioms and init's next steps (about 44 KiB), plus 112 KiB for prism-code mcp, the guided journey (about 110 KiB), plus 4 KiB for closure v2 provenance verification and the Hologram studio stylesheet (about 2.7 KiB), plus 24 KiB for the hosted tiers: hosted.mjs, the routing of build --check and verify --lean, and their docs (about 24.6 KiB), plus 4 KiB for the journey's hosted proof (about 6.7 KiB, with the 0.9 KiB left before it), plus 40 KiB for build --remote github: the GitHub client, the artifact reader and the checks of what comes back, with its docs and skill section (about 37.8 KiB), plus 4 KiB for the first-run fixes: the lockfile found above app/, per-verb --help, located hosted diagnostics and their docs (about 5.0 KiB, with the 2.2 KiB left before them), plus 20 KiB for the first run without Docker: setup's next steps and --local, the journey choosing GitHub Actions, the approved repository and the pinned template files (about 16.5 KiB), plus 28 KiB for template instances: the files that make a journey app an instance of UOR-Foundation/template, the canonical LexLean layout, the repository made from the template and the live check (about 24 KiB), plus 12 KiB for specs drafted from one sentence: the input schema of prism_spec, reasons, meanings and limits, and SPEC.md in the OpenSpec format (about 14 KiB, with the 4 KiB left before it), plus 20 KiB for gap telemetry: gaps.mjs, the rule each refusal names, the round-trip check, delivery in a hosted call's _meta, and their docs (about 16.5 KiB), plus 4 KiB for the verifier refusing what the Wire codec refuses (about 2.4 KiB), plus 4 KiB for refusing an altered holo in load (#583, about 2 KiB).
export const PACKAGE_FILES = JSON.parse(readFileSync(new URL('../test/fixtures/package-files.json', import.meta.url), 'utf8'));

export function checkPackage(pack) {
  assert.equal(pack.name, 'prism-code');
  assert.deepEqual(pack.files.map(({ path }) => path).sort(), PACKAGE_FILES,
    'Review distribution additions/removals explicitly in test/fixtures/package-files.json');
  assert.equal(pack.unpackedSize, pack.files.reduce((sum, { size }) => sum + size, 0),
    'npm unpackedSize must account for every distributed file');
  assert.ok(pack.unpackedSize <= PACKAGE_LIMIT,
    `Unpacked package is ${pack.unpackedSize} bytes; cap is ${PACKAGE_LIMIT}`);
}

if (process.argv[1] && pathToFileURL(process.argv[1]).href === import.meta.url) {
  const [pack] = JSON.parse(readFileSync(process.argv[2], 'utf8'));
  checkPackage(pack);
  console.log(`${pack.files.length} required files, ${pack.unpackedSize}/${PACKAGE_LIMIT} unpacked bytes`);
}
