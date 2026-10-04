// Internal SDK acquisition helper. The caller must supply an inventory digest
// from the independently validated consumer lock, never from this filesystem.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {closeSync, constants, fchmodSync, fstatSync, futimesSync, lstatSync, mkdirSync,
  openSync, readlinkSync, readSync, realpathSync, writeSync} from 'node:fs';
import {dirname, join, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {readSmall, snapshotFile, snapshotTree} from './exporter-seed.mjs';
import {decodeExporterSeed, encodeInventory} from './inventory-metadata.mjs';

const installedInventory = '/opt/prismpm/share/inventory.json';
const installedSeed = '/opt/prismpm/share/exporter-seed';
const sha = value => createHash('sha256').update(value).digest('hex');
const seedBounds = {files: 4097, file: 256 * 1024 ** 2, total: 520 * 1024 ** 2};

export function bindSeedInventory(bytes, expectedDigest, revision) {
  assert.match(expectedDigest, /^[0-9a-f]{64}$/);
  assert(bytes.length <= 8 * 1024 ** 2 && sha(bytes) === expectedDigest, 'SDK inventory differs from the independent lock');
  const text = new TextDecoder('utf-8', {fatal: true}).decode(bytes), inventory = JSON.parse(text);
  assert.equal(encodeInventory(inventory), text, 'canonical SDK inventory required');
  assert.equal(inventory.schema, 'prismpm/sdk-inventory/1');
  assert(Array.isArray(inventory.artifacts));
  const select = id => {
    const rows = inventory.artifacts.filter(row => row.id === id);
    assert(rows.length <= 1, 'duplicate SDK exporter artifact'); return rows[0];
  };
  const manifest = select('lean4-prod-exporter-seed'), executable = select('lean4-prod-exporter');
  if (!manifest && !executable) return null;
  assert(manifest && executable, 'incomplete SDK exporter advertisement');
  for (const [row, kind, version] of [[manifest, 'dependency-lock', '1'], [executable, 'binary', revision]]) {
    assert.deepEqual(Object.keys(row).sort(), ['digest', 'id', 'kind', 'version']);
    assert.equal(row.kind, kind); assert.equal(row.version, version); assert.match(row.digest, /^sha256:[0-9a-f]{64}$/);
  }
  return {manifest_sha256: manifest.digest.slice(7), executable_sha256: executable.digest.slice(7)};
}

export function bindSeedManifest(bytes, authority, identity) {
  assert.equal(sha(bytes), authority.manifest_sha256, 'exporter manifest differs from the independent SDK inventory');
  const manifest = decodeExporterSeed(bytes);
  assert.equal(manifest.compiler_revision, identity.compiler_revision);
  assert.equal(manifest.archive_sha256, identity.archive_sha256);
  assert.equal(manifest.toolchain, identity.toolchain);
  assert.equal(manifest.platform, identity.platform);
  assert.equal(manifest.files.find(row => row.path === '.lake/build/bin/prod-export').sha256, authority.executable_sha256);
  return manifest;
}

// Ownership supplements the external digest binding; it is not provenance.
export function immutablePath(path) {
  assert(typeof path === 'string' && path.startsWith('/') && path.length <= 4096);
  let current = '/', links = 0, steps = 0;
  const parts = path.split('/').filter(Boolean);
  const root = lstatSync('/'); assert(root.uid === 0 && (root.mode & 0o022) === 0);
  while (parts.length) {
    assert(++steps <= 16384, 'runtime path resolution exceeded bound');
    const part = parts.shift();
    if (part === '.') continue;
    if (part === '..') { current = dirname(current); continue; }
    const candidate = join(current, part), entry = lstatSync(candidate);
    assert(entry.uid === 0 && (entry.isSymbolicLink() || (entry.mode & 0o022) === 0),
      'SDK compiler custody must be root-owned and not group/world writable');
    if (entry.isSymbolicLink()) {
      assert(++links <= 40, 'runtime alias cycle or depth exceeded');
      const target = readlinkSync(candidate); assert(target.length <= 4096);
      if (target.startsWith('/')) current = '/';
      parts.unshift(...target.split('/').filter(Boolean));
    } else current = candidate;
  }
  assert.equal(realpathSync(path), current, 'runtime path changed while resolving');
  return current;
}

function immutableTree(root, rows) {
  immutablePath(root);
  for (const row of rows) {
    const entry = lstatSync(join(root, row.path));
    assert(entry.uid === 0 && (entry.isSymbolicLink() || (entry.mode & 0o022) === 0), 'mutable SDK compiler entry refused');
  }
}

export function verifySeedFiles(seed, source, manifest) {
  assert.deepEqual(snapshotTree(source), manifest.source_files, 'fresh compiled-in exporter sources differ');
  const files = snapshotTree(seed, {bounds: seedBounds}).filter(row => row.path !== 'manifest.json');
  assert.deepEqual(files, manifest.files, 'actual SDK seed closure differs');
  return files;
}

// Copy only declared bytes with exclusive creation. A concurrent growth cannot
// turn an already-validated sparse input into an unbounded persistent write.
export function stageSeedFiles(seed, staging, manifest) {
  decodeExporterSeed(Buffer.from(encodeInventory(manifest)));
  assert.equal(realpathSync(staging), staging);
  const directory = lstatSync(staging);
  assert(directory.isDirectory() && directory.uid === process.getuid() && (directory.mode & 0o7777) === 0o700,
    'staging must be private and owned by the current process user');
  assert.deepEqual(snapshotTree(staging), [], 'exclusive empty staging required');
  let total = 0;
  for (const row of manifest.files) {
    const destination = join(staging, row.path);
    if (row.kind === 'directory') { mkdirSync(destination, {mode: row.mode}); continue; }
    assert.equal(row.kind, 'file');
    total += row.byte_length; assert(total <= 512 * 1024 ** 2, 'seed copy aggregate exceeded');
    const source = join(seed, row.path);
    const expected = {byte_length: row.byte_length, mode: row.mode, sha256: row.sha256};
    const custody = lstatSync(source, {bigint: true});
    const sameInput = after => {
      for (const key of ['dev', 'ino', 'mode', 'size', 'nlink', 'mtimeNs', 'ctimeNs'])
        assert.equal(after[key], custody[key], `seed input identity changed: ${key}`);
    };
    assert.deepEqual(snapshotFile(source, row.byte_length), expected, 'seed file changed before copying');
    const input = openSync(source, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
    try {
      const before = fstatSync(input, {bigint: true});
      sameInput(before);
      assert(before.isFile() && before.nlink === 1n && before.size === BigInt(row.byte_length));
      const output = openSync(destination, constants.O_WRONLY | constants.O_CREAT | constants.O_EXCL | constants.O_NOFOLLOW, row.mode);
      try {
        const buffer = Buffer.alloc(64 * 1024), hash = createHash('sha256');
        let remaining = row.byte_length;
        while (remaining) {
          const count = readSync(input, buffer, 0, Math.min(remaining, buffer.length), null);
          assert(count > 0, 'seed file shortened while copying'); hash.update(buffer.subarray(0, count));
          for (let written = 0; written < count;) {
            const size = writeSync(output, buffer, written, count - written, null);
            assert(size > 0, 'seed copy made no progress'); written += size;
          }
          remaining -= count;
        }
        assert.equal(readSync(input, buffer, 0, 1, null), 0, 'seed file grew while copying');
        assert.equal(hash.digest('hex'), row.sha256, 'copied seed hash differs');
        sameInput(fstatSync(input, {bigint: true}));
        sameInput(lstatSync(source, {bigint: true}));
        assert.equal(realpathSync(source), source, 'seed input ancestor changed');
        fchmodSync(output, row.mode);
        futimesSync(output, Number(before.atimeNs) / 1e9, Number(before.mtimeNs) / 1e9);
      } finally { closeSync(output); }
    } finally { closeSync(input); }
    assert.deepEqual(snapshotFile(source, row.byte_length), expected, 'seed file changed during copying');
    assert.deepEqual(snapshotFile(destination, row.byte_length), expected, 'staged seed file changed');
  }
  assert.deepEqual(snapshotTree(staging, {bounds: seedBounds}), manifest.files, 'staged exporter bytes differ');
}

export function stageInstalledSeed(source, staging, inventoryDigest, identity) {
  immutablePath(installedInventory);
  assert.equal(lstatSync(installedInventory).mode & 0o222, 0, 'SDK inventory must be read-only');
  const inventory = Buffer.from(readSmall(installedInventory, 8 * 1024 ** 2));
  const authority = bindSeedInventory(inventory, inventoryDigest, identity.compiler_revision);
  if (authority === null) {
    assert.equal(lstatSync(installedSeed, {throwIfNoEntry: false}), undefined, 'unadvertised installed seed refused');
    return {schema: 'prismpm/exporter-acquisition/1', mode: 'cold'};
  }
  immutablePath(installedSeed);
  const manifestBytes = Buffer.from(readSmall(join(installedSeed, 'manifest.json'), 8 * 1024 ** 2));
  const manifest = bindSeedManifest(manifestBytes, authority, identity);
  const verify = () => {
    const files = verifySeedFiles(installedSeed, source, manifest);
    immutableTree(installedSeed, [...files, {path: 'manifest.json'}]);
    const toolchain = '/usr/local/elan/toolchains/' + identity.toolchain.replace('/', '--').replace(':', '---');
    assert.deepEqual(snapshotTree(toolchain, {toolchainAliases: true}), manifest.toolchain_files, 'installed toolchain differs');
    immutableTree(toolchain, manifest.toolchain_files);
    for (const row of manifest.runtime_files) {
      assert.equal(realpathSync(row.selected), row.path, 'runtime selection changed');
      immutablePath(row.selected); immutablePath(row.path);
      assert.deepEqual(snapshotFile(row.path), {byte_length: row.byte_length, mode: row.mode, sha256: row.sha256},
        'installed runtime differs');
    }
    assert.equal(readSmall(installedInventory, 8 * 1024 ** 2), inventory.toString('utf8'), 'SDK inventory changed during acquisition');
    assert.equal(readSmall(join(installedSeed, 'manifest.json'), 8 * 1024 ** 2), manifestBytes.toString('utf8'), 'seed manifest changed');
  };
  verify();
  stageSeedFiles(installedSeed, staging, manifest);
  verify();
  return {schema: 'prismpm/exporter-acquisition/1', mode: 'sdk-seed', inventory_sha256: inventoryDigest,
    manifest_sha256: authority.manifest_sha256, executable_sha256: authority.executable_sha256,
    compiler_revision: identity.compiler_revision, archive_sha256: identity.archive_sha256,
    platform: identity.platform, toolchain: identity.toolchain};
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert.equal(process.argv.length, 8, 'internal seed admission requires source, staging and independently bound SDK identity');
  const [source, staging, inventory, revision, archive, toolchain] = process.argv.slice(2);
  const platform = `linux/${{x64: 'amd64', arm64: 'arm64'}[process.arch]}`;
  assert.equal(process.platform, 'linux');
  process.stdout.write(encodeInventory(stageInstalledSeed(resolve(source), resolve(staging), inventory,
    {compiler_revision: revision, archive_sha256: archive, toolchain, platform})));
}
