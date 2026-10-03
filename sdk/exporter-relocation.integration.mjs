// Real cold/relocated compiler measurements, not SDK or application acceptance.
// No Lake traces or process output are rewritten. Production consumers remain
// cold until authenticated acquisition and complete equivalence are qualified.
import assert from 'node:assert/strict';
import {cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join, resolve} from 'node:path';
import {performance} from 'node:perf_hooks';
import {isDeepStrictEqual} from 'node:util';
import {buildSeed, constructionEnvironment, readSmall, runConstruction, snapshotTree} from './exporter-seed.mjs';
import {decodeExporterSeed} from './inventory-metadata.mjs';

assert.equal(process.argv.length, 4, 'usage: exporter-relocation.integration.mjs SOURCE OUTPUT_PARENT');
const source = resolve(process.argv[2]);
const output = mkdtempSync(join(resolve(process.argv[3]), 'exporter-relocation-check-'));
const temporary = mkdtempSync(join(tmpdir(), 'exporter-relocation-'));
try {
  const seed = join(output, 'seed');
  const construction = buildSeed(source, seed);
  const manifest = decodeExporterSeed(Buffer.from(readSmall(join(seed, 'manifest.json'), 8 * 1024 ** 2)));
  const environment = constructionEnvironment(manifest.configuration.environment, temporary);
  const lake = environment.PATH.split(':')[0] + '/lake';
  const immutableBefore = snapshotTree(seed);
  let expected;
  const observations = [];
  for (const location of ['first-root', 'independent-second-root']) {
    for (const acquisition of ['cold', 'relocated']) {
      const root = join(output, location, acquisition); mkdirSync(root, {recursive: true});
      const extraction = runConstruction('/usr/bin/tar',
        ['--extract', '--file', join(source, 'vendor/lean4-prod/lean.tar'), '--directory', root], source, environment);
      assert.deepEqual(snapshotTree(root), manifest.source_files);
      if (acquisition === 'relocated') {
        cpSync(join(seed, '.lake'), join(root, '.lake'), {recursive: true, errorOnExist: true,
          force: false, preserveTimestamps: true});
        assert.deepEqual(snapshotTree(root).filter(row => row.path === '.lake' || row.path.startsWith('.lake/')), manifest.files);
      }
      const start = performance.now();
      const build = runConstruction(lake, ['build', 'prod-export'], root, environment);
      const buildMilliseconds = performance.now() - start;
      const builtFiles = snapshotTree(join(root, '.lake'));
      const previous = new Map(manifest.files.filter(row => row.path !== '.lake').map(row => [row.path, row]));
      const current = new Map(builtFiles.map(row => {
        const path = `.lake/${row.path}`; return [path, {...row, path}];
      }));
      const changedBuildFiles = [...new Set([...previous.keys(), ...current.keys()])].sort()
        .filter(path => !isDeepStrictEqual(previous.get(path), current.get(path)))
        .map(path => ({path, change: !previous.has(path) ? 'added' : !current.has(path) ? 'removed' : 'changed'}));
      // This upstream fixture is LexLean-generated. No handwritten Lean driver
      // or replacement source is introduced by the relocation measurement.
      const module = runConstruction(lake, ['build', 'Conformance.LexLean11'], root, environment);
      const kernel = runConstruction(lake, ['env', 'leanchecker', 'Conformance.LexLean11'], root, environment);
      const exports = [];
      for (const replay of ['export-a', 'export-b']) {
        const destination = join(root, replay);
        const record = runConstruction(lake, ['exe', 'prod-export', '--module', 'Conformance.LexLean11',
          '--root', 'SemanticFixture.Main.allConsecutive', '--ir-module', 'exporter_relocation', '--out', destination], root, environment);
        const artifacts = Object.fromEntries(['kernel.ir', 'roots.json', 'coverage.json'].map(name =>
          [name, readFileSync(join(destination, name), 'utf8')]));
        if (expected === undefined) expected = artifacts;
        else assert.deepEqual(artifacts, expected, 'cold and relocated named exports must agree at both roots and both replays');
        exports.push(record);
      }
      observations.push({root, acquisition, invocation_milliseconds: buildMilliseconds, changed_build_files: changedBuildFiles,
        extraction, build, module, kernel, exports});
      assert.deepEqual(snapshotTree(seed), immutableBefore, 'relocation must not mutate its seed');
      rmSync(root, {recursive: true});
    }
  }
  process.stdout.write(JSON.stringify({scope: 'compiler-relocation-measurement-only',
    manifest_sha256: construction.manifest_sha256, construction, observations}) + '\n');
} finally {
  rmSync(output, {recursive: true});
  rmSync(temporary, {recursive: true});
}
