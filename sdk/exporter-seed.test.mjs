import assert from 'node:assert/strict';
import test from 'node:test';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {chmodSync, linkSync, mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, truncateSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {boundedConstructionRoot, buildSeed, constructionEnvironment, createConstructionStage, readSmall, runConstruction, runtimePaths, separateConstructionTrees, snapshotFile, snapshotTree, validateConstructionFilesystem} from './exporter-seed.mjs';
import {decodeExporterSeed, encodeInventory, exporterArtifactBindings} from './inventory-metadata.mjs';
import {bindSeedInventory, bindSeedManifest, stageSeedFiles, verifySeedFiles} from './exporter-seed-admission.mjs';

// Metadata-only fixture. Real construction and executable measurements are
// tested separately by exporter-seed.integration.mjs inside the pinned SDK.
function manifestFixture() {
  const directory = path => ({path, kind: 'directory', mode: 0o755});
  const file = path => ({path, kind: 'file', mode: 0o755, byte_length: 12, sha256: 'b'.repeat(64)});
  const toolchain = 'leanprover/lean4:v4.30.0';
  return {schema: 'prismpm/exporter-seed/1', compiler_revision: 'a'.repeat(40), platform: 'linux/amd64',
    archive_sha256: 'c'.repeat(64), toolchain,
    configuration: {argv: ['build', 'prod-export'], construction_root: '/tmp/prismpm-exporter-construction',
      temporary_directory: 'private-bounded-tmpfs', environment: {
        PATH: '/usr/local/elan/toolchains/leanprover--lean4---v4.30.0/bin:/usr/bin:/bin',
        LANG: 'C', LC_ALL: 'C', ELAN_HOME: '/usr/local/elan', ELAN_TOOLCHAIN: toolchain, LEAN_NUM_THREADS: '2', SOURCE_DATE_EPOCH: '0',
      }},
    source_files: [directory('Prod'), file('Prod/Export.lean')],
    toolchain_files: [directory('bin'), file('bin/lake'), file('bin/lean')],
    runtime_files: [{selected: '/lib/libc.so.6', path: '/usr/lib/libc.so.6', mode: 0o755,
      byte_length: 12, sha256: 'd'.repeat(64)}],
    files: [directory('.lake'), directory('.lake/build'), directory('.lake/build/bin'), file('.lake/build/bin/prod-export')],
  };
}

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
  chmodSync(join(root, 'extra'), 0o4755);
  assert.throws(() => snapshotTree(root), /special compiler file permissions/);
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
  symlinkSync('lib.so', join(root, 'lib-alias.so'));
  assert.equal(snapshotTree(root, {toolchainAliases: true}).find(row => row.path === 'lib-alias.so').target, 'lib.so');
  rmSync(join(root, 'lib-alias.so'));
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

test('directory enumeration is streamed and closes real iterators on every outcome', t => {
  for (const phase of ['success', 'count', 'alias']) {
    const root = fixture(t);
    for (let index = 0; index < 4; index++) writeFileSync(join(root, `entry-${index}`), 'x');
    if (phase === 'alias') symlinkSync('entry-0', join(root, 'alias'));
    const result = spawnSync(process.execPath, ['--input-type=module', '-e', `
      import assert from 'node:assert/strict';
      import fs from 'node:fs';
      import {syncBuiltinESMExports} from 'node:module';
      const [root, phase, module] = process.argv.slice(1);
      const open = fs.opendirSync;
      let opened = 0, closed = 0, reads = 0;
      // Instrument real filesystem calls; no substitute tree or acceptance.
      fs.readdirSync = () => { throw new Error('unbounded directory materialization'); };
      fs.opendirSync = (...args) => {
        assert.equal(args[1]?.bufferSize, 1);
        const stream = open(...args); opened++;
        const read = stream.readSync.bind(stream), close = stream.closeSync.bind(stream);
        stream.readSync = () => { reads++; return read(); };
        stream.closeSync = () => { closed++; return close(); };
        return stream;
      };
      syncBuiltinESMExports();
      const {snapshotTree} = await import(module);
      const capture = () => snapshotTree(root, {bounds: {files: phase === 'count' ? 1 : 8, file: 8, total: 64}});
      if (phase === 'success') assert.equal(capture().length, 4);
      else assert.throws(capture, phase === 'count' ? /entry count exceeded/ : /alias refused/);
      assert.equal(opened, 1); assert.equal(closed, opened);
      if (phase === 'count') assert.equal(reads, 2, 'stop at the first excess entry');
    `, root, phase, new URL('./exporter-seed.mjs', import.meta.url).href],
    {encoding: 'utf8', timeout: 10000, maxBuffer: 1024 * 1024});
    assert.ifError(result.error); assert.equal(result.signal, null);
    assert.equal(result.status, 0, result.stdout + result.stderr);
  }
});

test('deep and maximum-width snapshots preserve exact bytewise canonical ordering', t => {
  const root = fixture(t);
  mkdirSync(join(root, 'a')); writeFileSync(join(root, 'a', 'child'), 'child');
  writeFileSync(join(root, 'a-'), 'sibling');
  assert.deepEqual(snapshotTree(root).map(row => row.path), ['a', 'a-', 'a/child']);
  let deep = root;
  for (let depth = 0; depth < 80; depth++) { deep = join(deep, 'd'); mkdirSync(deep); }
  writeFileSync(join(deep, 'member'), 'deep');
  assert(snapshotTree(root).some(row => row.path === 'd/'.repeat(80) + 'member'));
  const wide = fixture(t);
  for (let index = 0; index < 32768; index++) writeFileSync(join(wide, `entry-${index}`), '');
  assert.equal(snapshotTree(wide).length, 32768);
  writeFileSync(join(wide, 'overflow'), '');
  assert.throws(() => snapshotTree(wide), /entry count exceeded/);
});

test('constructed source and full seed inventories keep independent entry and byte allowances', t => {
  const root = fixture(t), source = join(root, 'package'), seed = join(root, 'seed');
  mkdirSync(source);
  const member = join(source, 'source'); writeFileSync(member, ''); truncateSync(member, 16 * 1024 ** 2);
  for (let index = 1; index < 4096; index++) writeFileSync(join(source, `source-${index}`), '');
  const sources = snapshotTree(source);
  assert.equal(sources.length, 4096);
  mkdirSync(join(source, '.lake'));
  for (const name of ['first', 'second']) {
    const path = join(source, '.lake', name); writeFileSync(path, ''); truncateSync(path, 256 * 1024 ** 2);
  }
  // .lake + two full-size files + 4093 empty entries exactly meet both bounds.
  for (let index = 3; index < 4096; index++) writeFileSync(join(source, '.lake', `entry-${index}`), '');
  const files = separateConstructionTrees(source, seed, sources);
  assert.equal(files.length, 4096);
  assert.equal(files.reduce((sum, row) => sum + (row.byte_length ?? 0), 0), 512 * 1024 ** 2);
  assert.deepEqual(snapshotTree(source), sources);
  for (const defect of ['extra', 'source-oversize', 'seed-overflow']) {
    const owner = fixture(t), input = join(owner, 'package'); mkdirSync(input);
    writeFileSync(join(input, 'source'), 'source'); const expected = snapshotTree(input);
    mkdirSync(join(input, '.lake'));
    if (defect === 'extra') writeFileSync(join(input, 'extra'), 'unexpected');
    if (defect === 'source-oversize') truncateSync(join(input, 'source'), 16 * 1024 ** 2 + 1);
    if (defect === 'seed-overflow') {
      const path = join(input, '.lake', 'oversize'); writeFileSync(path, ''); truncateSync(path, 256 * 1024 ** 2 + 1);
    }
    assert.throws(() => separateConstructionTrees(input, join(owner, 'seed'), expected));
  }
});

test('deferred and previously visited directory substitution fails custody', t => {
  for (const phase of ['pending', 'visited']) {
    const root = fixture(t); mkdirSync(join(root, 'first')); mkdirSync(join(root, 'second'));
    const result = spawnSync(process.execPath, ['--input-type=module', '-e', `
      import assert from 'node:assert/strict';
      import fs from 'node:fs';
      import {syncBuiltinESMExports} from 'node:module';
      import {join} from 'node:path';
      const [root, phase, module] = process.argv.slice(1);
      const open = fs.opendirSync; let first, mutated = false, live = 0;
      fs.opendirSync = (...args) => {
        const stream = open(...args); live++;
        const close = stream.closeSync.bind(stream);
        stream.closeSync = () => {
          close(); live--;
          if (phase === 'pending' && args[0] === root) {
            // Change only the child, not its parent directory membership.
            fs.chmodSync(join(root, 'first'), 0o700); mutated = true;
          } else if (phase === 'visited' && args[0] !== root) {
            if (!first) first = args[0];
            else { fs.chmodSync(first, 0o700); mutated = true; }
          }
        };
        return stream;
      };
      syncBuiltinESMExports(); const {snapshotTree} = await import(module);
      assert.throws(() => snapshotTree(root), /compiler input changed/);
      assert(mutated); assert.equal(live, 0);
    `, root, phase, new URL('./exporter-seed.mjs', import.meta.url).href],
    {encoding: 'utf8', timeout: 10000, maxBuffer: 1024 * 1024});
    assert.ifError(result.error); assert.equal(result.signal, null);
    assert.equal(result.status, 0, result.stdout + result.stderr);
  }
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

test('configuration reads reject same-byte replacement and rewriting during the actual read', t => {
  const root = fixture(t);
  for (const mutation of ['replace', 'rewrite']) {
    const path = join(root, mutation);
    writeFileSync(path, 'unchanged configuration');
    // Isolate scheduling hooks in a child. All reads and mutations use real
    // descriptors/files; only the instant of the adversarial write is selected.
    const result = spawnSync(process.execPath, ['--input-type=module', '-e', `
      import assert from 'node:assert/strict';
      import fs from 'node:fs';
      import {syncBuiltinESMExports} from 'node:module';
      const path = process.argv[1], mutation = process.argv[2];
      const original = fs.readSync;
      let mutated = false;
      fs.readSync = (...args) => {
        const count = original(...args);
        if (!mutated && args[1].length === 257 && count > 0) {
          mutated = true;
          if (mutation === 'replace') {
            fs.writeFileSync(path + '.replacement', 'unchanged configuration');
            fs.renameSync(path + '.replacement', path);
          } else fs.writeFileSync(path, 'unchanged configuration');
        }
        return count;
      };
      syncBuiltinESMExports();
      const {readSmall} = await import(process.argv[3]);
      assert.throws(() => readSmall(path, 256), /compiler input changed/);
      assert(mutated, 'the real filesystem mutation must execute');
    `, path, mutation, new URL('./exporter-seed.mjs', import.meta.url).href],
    {encoding: 'utf8', timeout: 10000, maxBuffer: 1024 * 1024});
    assert.ifError(result.error);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, `${mutation}: ${result.stderr}`);
  }
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

test('inventory distinguishes actual exporter identity from its native seed manifest', () => {
  // Explicit metadata fixture, not an SDK image or executable acceptance.
  const revision = 'a'.repeat(40), digest = 'b'.repeat(64);
  const value = manifestFixture();
  const capture = item => Buffer.from(encodeInventory(item));
  const measured = {byte_length: 12, mode: 0o755, sha256: digest};
  const rows = exporterArtifactBindings(capture(value), revision, 'linux/amd64', measured);
  assert.deepEqual(rows[0], {id: 'lean4-prod-exporter', kind: 'binary', version: revision, digest: `sha256:${digest}`});
  assert.equal(rows[1].id, 'lean4-prod-exporter-seed'); assert.equal(rows[1].kind, 'dependency-lock');
  assert.notEqual(rows[1].digest, rows[0].digest);
  for (const mutate of [
    item => { item.platform = 'linux/arm64'; }, item => { item.compiler_revision = 'c'.repeat(40); },
    item => { item.files = []; }, item => { item.files.push({...item.files[0]}); },
    item => { item.files.at(-1).mode = 0o644; }, item => { item.files.at(-1).kind = 'symlink'; },
    item => { item.files.at(-1).byte_length = 0; }, item => { item.files.at(-1).sha256 = 'invalid'; },
  ]) {
    const altered = structuredClone(value); mutate(altered);
    assert.throws(() => exporterArtifactBindings(capture(altered), revision, 'linux/amd64', measured));
  }
  assert.throws(() => exporterArtifactBindings(Buffer.from(JSON.stringify(value)), revision, 'linux/amd64', measured));
  for (const change of [{mode: 0o644}, {byte_length: 11}, {sha256: 'c'.repeat(64)}]) {
    assert.throws(() => exporterArtifactBindings(capture(value), revision, 'linux/amd64', {...measured, ...change}));
  }
});

test('seed wire reader closes every field, path, configuration and resource bound', () => {
  const decode = value => decodeExporterSeed(Buffer.from(encodeInventory(value)));
  const value = manifestFixture(); assert.deepEqual(decode(value), value);
  const changes = [
    item => { item.extra = true; }, item => { delete item.archive_sha256; },
    item => { item.configuration.environment.EXTRA = 'untrusted'; },
    item => { delete item.configuration.environment.LEAN_NUM_THREADS; },
    item => { item.configuration.environment.LEAN_NUM_THREADS = '9999'; },
    item => { item.configuration.construction_root = '/caller/cache'; },
    item => { item.configuration.argv.push('--no-build'); },
    item => { item.archive_sha256 = 'A'.repeat(64); },
    item => { item.platform = 'linux/386'; }, item => { item.toolchain = '../toolchain'; },
    item => { item.source_files[0].extra = true; },
    item => { item.source_files[1].path = 'Prod/../Export.lean'; },
    item => { item.source_files[1].path = '/Prod/Export.lean'; },
    item => { item.source_files.shift(); },
    item => { item.source_files[1].byte_length = 16 * 1024 ** 2 + 1; },
    item => { item.files.reverse(); }, item => { item.files.push({...item.files.at(-1)}); },
    item => { item.files.at(-1).mode = 0o4755; },
    item => { item.files.at(-1).sha256 = null; },
    item => { item.files.at(-1).byte_length = 256 * 1024 ** 2 + 1; },
    item => { item.files.at(-1).byte_length = -1; },
    item => { item.files.at(-1).byte_length = 0.5; },
    item => { item.files.push({path: 'outside', kind: 'directory', mode: 0o755}); },
    item => { item.toolchain_files[1] = {path: 'bin/lake', kind: 'symlink', mode: 0o777, target: 'absent'}; },
    item => { item.toolchain_files[1].extra = false; },
    item => { item.toolchain_files[1].mode = 0o644; },
    item => { item.toolchain_files[2].byte_length = 0; },
    item => { item.runtime_files = []; },
    item => { item.runtime_files[0].extra = true; },
    item => { item.runtime_files[0].selected = 'lib/libc.so.6'; },
    item => { item.runtime_files[0].path = '/usr/../lib/libc.so.6'; },
    item => { item.runtime_files.push({...item.runtime_files[0]}); },
  ];
  for (const [index, change] of changes.entries()) {
    const altered = structuredClone(value); change(altered);
    assert.throws(() => decode(altered), `mutation ${index} must fail closed`);
  }
  const alias = structuredClone(value);
  alias.toolchain_files.push({path: 'lean-alias', kind: 'symlink', mode: 0o777, target: 'bin/lean'});
  assert.deepEqual(decode(alias), alias);
  alias.toolchain_files.push({path: 'lean-indirect', kind: 'symlink', mode: 0o777, target: 'lean-alias'});
  assert.deepEqual(decode(alias), alias);
  alias.toolchain_files.at(-2).target = 'lean-indirect'; assert.throws(() => decode(alias), /cyclic/);
  alias.toolchain_files.at(-2).target = 'bin/lean';
  alias.toolchain_files.at(-1).target = '../bin/lean'; assert.throws(() => decode(alias));
  const runtime = structuredClone(value);
  runtime.runtime_files.push({...runtime.runtime_files[0], selected: '/usr/lib/libc.so.6'});
  assert.deepEqual(decode(runtime), runtime);
  for (const change of [{sha256: 'f'.repeat(64)}, {mode: 0o644}, {byte_length: 13}]) {
    const altered = structuredClone(runtime); Object.assign(altered.runtime_files[1], change);
    assert.throws(() => decode(altered), /runtime aliases disagree/);
  }
  const duplicate = encodeInventory(value).replace('"schema":', '"schema":"discarded","schema":');
  assert.throws(() => decodeExporterSeed(Buffer.from(duplicate)), /canonical/);
  assert.throws(() => decodeExporterSeed(Buffer.from([0xff])));
  assert.throws(() => decodeExporterSeed(Buffer.alloc(8 * 1024 ** 2 + 1)), /bounded/);
});

test('seed admission requires an independent inventory binding, including coherent rehash attacks', () => {
  const encode = value => Buffer.from(encodeInventory(value));
  const hash = bytes => createHash('sha256').update(bytes).digest('hex');
  const manifest = manifestFixture(), bytes = encode(manifest);
  const inventory = {schema: 'prismpm/sdk-inventory/1', artifacts: [
    {id: 'lean4-prod-exporter', kind: 'binary', version: manifest.compiler_revision, digest: 'sha256:' + 'b'.repeat(64)},
    {id: 'lean4-prod-exporter-seed', kind: 'dependency-lock', version: '1', digest: 'sha256:' + hash(bytes)},
  ]};
  const locked = hash(encode(inventory));
  const authority = bindSeedInventory(encode(inventory), locked, manifest.compiler_revision);
  const identity = Object.fromEntries(['compiler_revision', 'archive_sha256', 'toolchain', 'platform'].map(key => [key, manifest[key]]));
  assert.deepEqual(bindSeedManifest(bytes, authority, identity), manifest);
  const tampered = structuredClone(manifest); tampered.files.at(-1).sha256 = 'e'.repeat(64);
  assert.throws(() => bindSeedManifest(encode(tampered), authority, identity), /independent SDK inventory/);
  const coherent = structuredClone(inventory);
  coherent.artifacts[0].digest = 'sha256:' + 'e'.repeat(64);
  coherent.artifacts[1].digest = 'sha256:' + hash(encode(tampered));
  assert.throws(() => bindSeedInventory(encode(coherent), locked, manifest.compiler_revision), /independent lock/);
  for (const key of Object.keys(identity)) {
    assert.throws(() => bindSeedManifest(bytes, authority, {...identity, [key]: 'changed'}));
  }
  for (const mutate of [
    item => { item.artifacts.pop(); }, item => { item.artifacts.push({...item.artifacts[0]}); },
    item => { item.artifacts[0].version = 'f'.repeat(40); },
    item => { item.artifacts[1].kind = 'binary'; },
    item => { item.artifacts[1].extra = true; },
  ]) {
    const changed = structuredClone(inventory); mutate(changed);
    assert.throws(() => bindSeedInventory(encode(changed), hash(encode(changed)), manifest.compiler_revision));
  }
  const legacy = encode({schema: 'prismpm/sdk-inventory/1', artifacts: []});
  assert.equal(bindSeedInventory(legacy, hash(legacy), manifest.compiler_revision), null);
});

test('actual seed/source snapshots refuse altered, missing, extra and aliased content', t => {
  const source = fixture(t), seed = fixture(t);
  mkdirSync(join(source, 'Prod')); writeFileSync(join(source, 'Prod/Export.lean'), 'source snapshot fixture');
  mkdirSync(join(seed, '.lake')); writeFileSync(join(seed, '.lake/trace'), 'actual fixture trace');
  const manifest = {source_files: snapshotTree(source), files: snapshotTree(seed)};
  writeFileSync(join(seed, 'manifest.json'), 'metadata is authenticated separately');
  verifySeedFiles(seed, source, manifest);
  const trace = join(seed, '.lake/trace');
  writeFileSync(trace, 'altered trace'); assert.throws(() => verifySeedFiles(seed, source, manifest), /seed closure differs/);
  rmSync(trace); assert.throws(() => verifySeedFiles(seed, source, manifest), /seed closure differs/);
  writeFileSync(trace, 'actual fixture trace');
  writeFileSync(join(seed, 'extra'), 'extra'); assert.throws(() => verifySeedFiles(seed, source, manifest), /seed closure differs/);
  rmSync(join(seed, 'extra')); rmSync(trace); symlinkSync(join(source, 'Prod/Export.lean'), trace);
  assert.throws(() => verifySeedFiles(seed, source, manifest), /alias/);
  rmSync(trace); writeFileSync(trace, 'actual fixture trace');
  writeFileSync(join(source, 'Prod/Export.lean'), 'changed source');
  assert.throws(() => verifySeedFiles(seed, source, manifest), /sources differ/);
});

test('seed staging exclusively copies bounded declared bytes and refuses changed inputs', t => {
  const seed = fixture(t), destination = fixture(t);
  mkdirSync(join(seed, '.lake/build/bin'), {recursive: true});
  const executable = join(seed, '.lake/build/bin/prod-export');
  writeFileSync(executable, 'metadata test bytes, not an executable acceptance', {mode: 0o755});
  const manifest = manifestFixture(); manifest.files = snapshotTree(seed);
  stageSeedFiles(seed, destination, manifest);
  assert.deepEqual(snapshotTree(destination), manifest.files);
  assert.throws(() => stageSeedFiles(seed, destination, manifest), /empty staging/);
  assert.equal(readFileSync(join(destination, '.lake/build/bin/prod-export'), 'utf8'),
    'metadata test bytes, not an executable acceptance');
  const oversized = fixture(t); truncateSync(executable, 8 * 1024 ** 3);
  assert.throws(() => stageSeedFiles(seed, oversized, manifest), /bounded/);
  assert.equal(snapshotTree(oversized).some(row => row.kind === 'file'), false);
  const aliased = fixture(t); rmSync(executable); symlinkSync(join(destination, '.lake/build/bin/prod-export'), executable);
  assert.throws(() => stageSeedFiles(seed, aliased, manifest), /aliased/);
  const unsafe = structuredClone(manifest); unsafe.files.at(-1).path = '../escape';
  assert.throws(() => stageSeedFiles(seed, fixture(t), unsafe), /canonical relative/);
  const publicStage = fixture(t); chmodSync(publicStage, 0o755);
  assert.throws(() => stageSeedFiles(seed, publicStage, manifest), /private/);
});

test('seed staging rejects same-byte input replacement before and during copying', t => {
  for (const phase of ['open', 'read']) {
    const seed = fixture(t), destination = fixture(t);
    mkdirSync(join(seed, '.lake/build/bin'), {recursive: true});
    const executable = join(seed, '.lake/build/bin/prod-export');
    writeFileSync(executable, 'copy custody fixture', {mode: 0o755});
    const manifest = manifestFixture(); manifest.files = snapshotTree(seed);
    const result = spawnSync(process.execPath, ['--input-type=module', '-e', `
      import assert from 'node:assert/strict';
      import fs from 'node:fs';
      import {syncBuiltinESMExports} from 'node:module';
      const [seed, destination, path, phase, encoded, module] = process.argv.slice(1);
      const open = fs.openSync, read = fs.readSync;
      let opens = 0, input, mutated = false;
      function replace() {
        fs.writeFileSync(path + '.replacement', 'copy custody fixture', {mode: 0o755});
        fs.renameSync(path + '.replacement', path); mutated = true;
      }
      fs.openSync = (...args) => {
        if (args[0] === path && ++opens === 2) {
          if (phase === 'open') replace();
          input = open(...args); return input;
        }
        return open(...args);
      };
      fs.readSync = (...args) => {
        const count = read(...args);
        if (phase === 'read' && !mutated && args[0] === input && count > 0) replace();
        return count;
      };
      syncBuiltinESMExports();
      const {stageSeedFiles} = await import(module);
      assert.throws(() => stageSeedFiles(seed, destination, JSON.parse(encoded)), /seed input identity changed/);
      assert(mutated, 'the real filesystem replacement must execute');
    `, seed, destination, executable, phase, JSON.stringify(manifest),
    new URL('./exporter-seed-admission.mjs', import.meta.url).href],
    {encoding: 'utf8', timeout: 10000, maxBuffer: 1024 * 1024});
    assert.ifError(result.error);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, `${phase}: ${result.stderr}`);
  }
});
