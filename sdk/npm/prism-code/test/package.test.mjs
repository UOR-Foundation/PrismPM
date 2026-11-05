// The source inventory catches budget regressions in npm test; CI additionally
// passes npm's real pack manifest through the same check to catch packlist drift.
import assert from 'node:assert/strict';
import { readFileSync, statSync } from 'node:fs';
import { test } from 'node:test';
import { checkPackage, PACKAGE_FILES, PACKAGE_LIMIT } from '../scripts/check-package.mjs';

const files = PACKAGE_FILES.map((path) => ({ path, size: statSync(new URL(`../${path}`, import.meta.url)).size }));
const pack = { name: 'prism-code', files, unpackedSize: files.reduce((sum, file) => sum + file.size, 0) };

test('all required distributed files fit the unchanged 1088 KiB package budget', () => {
  checkPackage(pack);
});

test('one byte over budget is refused, and the exact boundary is accepted', () => {
  const atSize = (unpackedSize) => ({
    ...pack, unpackedSize,
    files: files.map((file, i) => i === 0 ? { ...file, size: file.size + unpackedSize - pack.unpackedSize } : file),
  });
  checkPackage(atSize(PACKAGE_LIMIT));
  assert.throws(() => checkPackage(atSize(PACKAGE_LIMIT + 1)), /cap is 1114112/);
});

test('a smaller package cannot pass by omitting a required file or changing its inventory', () => {
  const without = files.filter(({ path }) => path !== 'src/vendor/verify.mjs');
  assert.throws(() => checkPackage({ ...pack, files: without, unpackedSize: without.reduce((sum, f) => sum + f.size, 0) }), /distribution additions\/removals/);
  assert.throws(() => checkPackage({ ...pack, files: [...files, files[0]] }), /distribution additions\/removals/);
  assert.throws(() => checkPackage({ ...pack, files: [...files, { path: 'unexpected.txt', size: 0 }] }), /distribution additions\/removals/);
  assert.throws(() => checkPackage({ ...pack, unpackedSize: pack.unpackedSize - 1 }), /account for every distributed file/);
});

test('README guide links resolve to repository files and headings', () => {
  const root = new URL('../../../../', import.meta.url);
  const readme = readFileSync(new URL('../README.md', import.meta.url), 'utf8');
  const guide = readFileSync(new URL('docs/npm/prism-code.md', root), 'utf8');
  const slugs = new Set([...guide.matchAll(/^#+ (.+)$/gm)].map(([, title]) => title.toLowerCase().replace(/[^\w\s-]/g, '').replace(/ /g, '-')));
  const links = [...readme.matchAll(/https:\/\/github\.com\/UOR-Foundation\/PrismPM\/blob\/main\/docs\/npm\/prism-code\.md(?:#([\w-]+))?/g)];
  assert.ok(links.length > 0, 'the complete guide is discoverable from the npm README');
  for (const [, anchor] of links) if (anchor) assert.ok(slugs.has(anchor), anchor);
  for (const [, path] of guide.matchAll(/\]\((\.\.\/proposals\/[^)#]+\.md)\)/g)) {
    assert.ok(statSync(new URL(path, new URL('docs/npm/prism-code.md', root))).isFile(), path);
  }
});
