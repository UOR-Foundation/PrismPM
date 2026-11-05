// Validate npm's pack manifest against the reviewed distribution inventory.
// Run after npm pack --dry-run --json > pack.json, as npm-prism-code.yml does.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

export const PACKAGE_LIMIT = 1236992; // 1208 KiB: 1088 KiB plus 72 KiB for setup, open and the bundled prism skill (about 63 KiB), plus 48 KiB for the machine half of setup, the build tiers, axioms and init's next steps (about 44 KiB).
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
