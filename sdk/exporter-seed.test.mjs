import assert from 'node:assert/strict';
import test from 'node:test';
import {chmodSync, linkSync, mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, truncateSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {boundedConstructionRoot, buildSeed, constructionEnvironment, createConstructionStage, readSmall, runConstruction, runtimePaths, snapshotFile, snapshotTree, validateConstructionFilesystem} from './exporter-seed.mjs';

function fixture(t, parent = tmpdir()) {
  const root = mkdtempSync(join(parent, 'prismpm-seed-snapshot-'));
  t.after(() => rmSync(root, {recursive: true}));
  return root;
}

test('runtime parser distinguishes explicit static linkage from missing dependencies', () => {
  assert.deepEqual(runtimePaths('\tstatically linked\n'), []);
  assert.deepEqual(runtimePaths('\tlinux-vdso.so.1 (0x00001)\n\tlibc.so.6 => /lib/libc.so.6 (0x00002)\n\t/lib64/ld-linux.so.2 (0x00003)\n'),
    ['/lib/libc.so.6', '/lib64/ld-linux.so.2']);
  for (const text of ['', '\tlibc.so.6 => not found\n', '\tnot a dynamic executable\n', '\tstatically linked\nextra']) {
    assert.throws(() => runtimePaths(text));
  }
});

test('snapshot binds every file, directory, byte and mode without accepting a seed', t => {
  const root = fixture(t); mkdirSync(join(root, '.lake'));
  writeFileSync(join(root, '.lake', 'trace'), 'actual fixture bytes', {mode: 0o644});
  const first = snapshotTree(root);
  assert.deepEqual(first.map(row => [row.path, row.kind]), [['.lake', 'directory'], ['.lake/trace', 'file']]);
  writeFileSync(join(root, '.lake', 'trace'), 'other fixture bytes');
  assert.notDeepEqual(snapshotTree(root), first);
  const second = snapshotTree(root); chmodSync(join(root, '.lake', 'trace'), 0o600);
  assert.notDeepEqual(snapshotTree(root), second);
  writeFileSync(join(root, 'extra'), 'x'); assert.equal(snapshotTree(root).length, 3);
});

test('seed snapshots reject file/directory aliases and hard links', t => {
  const root = fixture(t), other = fixture(t);
  writeFileSync(join(other, 'source'), 'content');
  symlinkSync(join(other, 'source'), join(root, 'alias'));
  assert.throws(() => snapshotTree(root), /alias refused/);
  rmSync(join(root, 'alias')); symlinkSync(other, join(root, 'alias'));
  assert.throws(() => snapshotTree(root), /alias refused/);
  rmSync(join(root, 'alias')); linkSync(join(other, 'source'), join(root, 'linked'));
  assert.throws(() => snapshotTree(root), /singly-linked/);
});

test('toolchain aliases are explicit, relative, file-only, and confined', t => {
  const root = fixture(t);
  writeFileSync(join(root, 'lib.so.1'), 'fixture');
  symlinkSync('lib.so.1', join(root, 'lib.so'));
  assert.equal(snapshotTree(root, {toolchainAliases: true}).find(row => row.path === 'lib.so').target, 'lib.so.1');
  rmSync(join(root, 'lib.so')); symlinkSync('../outside', join(root, 'lib.so'));
  assert.throws(() => snapshotTree(root, {toolchainAliases: true}), /unconfined/);
  rmSync(join(root, 'lib.so')); mkdirSync(join(root, 'directory')); symlinkSync('directory', join(root, 'lib.so'));
  assert.throws(() => snapshotTree(root, {toolchainAliases: true}), /escapes closure/);
});

test('bounded snapshots reject oversized sparse files and aggregate/count excess', t => {
  const root = fixture(t), file = join(root, 'source');
  writeFileSync(file, 'x'); truncateSync(file, 8 * 1024 ** 3);
  assert.throws(() => snapshotFile(file), /bounded/);
  truncateSync(file, 8);
  assert.throws(() => snapshotTree(root, {bounds: {files: 1, file: 8, total: 7}}), /bounded/);
  writeFileSync(join(root, 'extra'), 'x');
  assert.throws(() => snapshotTree(root, {bounds: {files: 1, file: 8, total: 16}}), /count exceeded/);
});

test('configuration reads are bounded, regular, singly linked, and strict UTF-8', t => {
  const root = fixture(t), path = join(root, 'configuration');
  writeFileSync(path, 'valid'); assert.equal(readSmall(path, 256), 'valid');
  truncateSync(path, 8 * 1024 ** 3);
  assert.throws(() => readSmall(path, 256), /bounded/);
  truncateSync(path, 0); writeFileSync(path, Buffer.from([0xff]));
  assert.throws(() => readSmall(path, 256));
  symlinkSync(path, join(root, 'alias'));
  assert.throws(() => readSmall(join(root, 'alias'), 256), /aliased/);
  linkSync(path, join(root, 'linked'));
  assert.throws(() => readSmall(path, 256), /singly-linked/);
});

test('construction requires a bounded memory filesystem and refuses existing output', t => {
  const root = fixture(t, '/dev/shm');
  assert.equal(boundedConstructionRoot(root), root);
  assert.throws(() => boundedConstructionRoot('/'), /bounded tmpfs/);
  const output = join(root, 'existing'); mkdirSync(output);
  assert.throws(() => buildSeed(root, output), /overwrite/);
  for (const [blocks, bsize] of [[0n, 4096n], [4096n, 0n], [1048576n, 4096n]]) {
    assert.throws(() => validateConstructionFilesystem({type: 0x01021994n, blocks, bsize}), /positive capacity/);
  }
});

test('real construction children use the owned bounded temporary directory', t => {
  const root = fixture(t, '/dev/shm');
  const environment = constructionEnvironment({PATH: '/usr/bin:/bin'}, root);
  const result = runConstruction('/usr/bin/printenv', ['TMPDIR'], root, environment);
  assert.equal(result.stdout, root + '\n');
  assert.equal(result.environment.TMPDIR, root);
  assert.equal(result.exit_code, 0);
  assert.throws(() => constructionEnvironment({}, '/'), /bounded tmpfs/);
});

test('fixed construction path refuses existing state without adopting or deleting it', t => {
  const root = fixture(t, '/dev/shm');
  const stage = createConstructionStage(root);
  writeFileSync(join(stage.path, 'sentinel'), 'retain');
  assert.throws(() => createConstructionStage(root), /EEXIST/);
  assert.equal(readFileSync(join(stage.path, 'sentinel'), 'utf8'), 'retain');
  const other = fixture(t, '/dev/shm');
  symlinkSync(stage.path, join(other, 'prismpm-exporter-construction'));
  assert.throws(() => createConstructionStage(other), /EEXIST/);
  assert.equal(readFileSync(join(stage.path, 'sentinel'), 'utf8'), 'retain');
});
