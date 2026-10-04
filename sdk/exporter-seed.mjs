// SDK construction only. A manifest is input data, not an acceptance receipt.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {closeSync, constants, cpSync, fstatSync, lstatSync, mkdirSync, mkdtempSync,
  openSync, readdirSync, readlinkSync, readSync, realpathSync,
  renameSync, rmSync, statfsSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname, join, resolve, sep} from 'node:path';
import {fileURLToPath} from 'node:url';
import {compilerRevision, encodeInventory} from './inventory-metadata.mjs';

const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const order = (a, b) => Buffer.from(a).compare(Buffer.from(b));
const limit = Object.freeze({files: 32768, file: 1024 ** 3, total: 4 * 1024 ** 3});
const seedLimit = Object.freeze({files: 4096, file: 256 * 1024 ** 2, total: 512 * 1024 ** 2});
const identity = ['dev', 'ino', 'mode', 'size', 'nlink', 'mtimeNs', 'ctimeNs'];
function unchanged(before, after) {
  for (const key of identity) assert.equal(after[key], before[key], `compiler input changed: ${key}`);
}

// Stream hashes; never allocate an input-sized buffer. The same routine is
// used before and after construction, so a changed toolchain cannot be sealed.
export function snapshotFile(path, maximum = limit.file) {
  assert.equal(realpathSync(path), path, 'compiler file is aliased');
  const before = lstatSync(path, {bigint: true});
  assert(before.isFile() && before.nlink === 1n && before.size <= BigInt(maximum),
    'bounded singly-linked compiler file required');
  assert.equal(before.mode & 0o7000n, 0n, 'special compiler file permissions refused');
  const fd = openSync(path, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
  try {
    unchanged(before, fstatSync(fd, {bigint: true}));
    const hash = createHash('sha256'), buffer = Buffer.alloc(64 * 1024);
    let length = 0;
    for (;;) {
      const count = readSync(fd, buffer, 0, buffer.length, null);
      if (!count) break;
      length += count;
      assert(length <= maximum && BigInt(length) <= before.size, 'compiler file grew');
      hash.update(buffer.subarray(0, count));
    }
    assert.equal(BigInt(length), before.size, 'compiler file shortened');
    unchanged(before, fstatSync(fd, {bigint: true}));
    unchanged(before, lstatSync(path, {bigint: true}));
    assert.equal(realpathSync(path), path, 'compiler file ancestor changed');
    return {byte_length: length, mode: Number(before.mode & 0o777n), sha256: hash.digest('hex')};
  } finally { closeSync(fd); }
}

export function readSmall(path, maximum) {
  assert.equal(realpathSync(path), path, 'compiler file is aliased');
  const before = lstatSync(path, {bigint: true});
  assert(before.isFile() && before.nlink === 1n && before.size <= BigInt(maximum),
    'bounded singly-linked compiler file required');
  assert.equal(before.mode & 0o7000n, 0n, 'special compiler file permissions refused');
  const fd = openSync(path, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
  try {
    // Bind the bytes to one descriptor and its original inode. Comparing only
    // hashes from separately reopened files accepts same-byte replacement.
    unchanged(before, fstatSync(fd, {bigint: true}));
    const bytes = Buffer.alloc(maximum + 1);
    let size = 0;
    while (size <= maximum) {
      const count = readSync(fd, bytes, size, bytes.length - size, null);
      if (!count) break;
      size += count;
    }
    assert(size <= maximum && BigInt(size) === before.size,
      'compiler configuration changed or exceeded bound');
    unchanged(before, fstatSync(fd, {bigint: true}));
    unchanged(before, lstatSync(path, {bigint: true}));
    assert.equal(realpathSync(path), path, 'compiler file ancestor changed');
    return new TextDecoder('utf-8', {fatal: true}).decode(bytes.subarray(0, size));
  } finally { closeSync(fd); }
}

export function boundedConstructionRoot(path) {
  assert.equal(realpathSync(path), path, 'construction root is aliased');
  const filesystem = statfsSync(path, {bigint: true});
  validateConstructionFilesystem(filesystem);
  return path;
}

export function validateConstructionFilesystem(filesystem) {
  assert.equal(filesystem.type, 0x01021994n, 'exporter construction requires a bounded tmpfs');
  assert(filesystem.blocks > 0n && filesystem.bsize > 0n
    && filesystem.blocks * filesystem.bsize <= 1024n ** 3n, 'exporter construction tmpfs must have a positive capacity at most 1 GiB');
}

// Symlinks are forbidden in seeds and sources. The upstream toolchain itself
// has declared relative library aliases; record those links and their confined
// targets rather than silently dereferencing or omitting them.
export function snapshotTree(root, {toolchainAliases = false, bounds = limit} = {}) {
  assert.equal(realpathSync(root), root, 'compiler tree root is aliased');
  const rows = [];
  let total = 0;
  function visit(directory, prefix) {
    const before = lstatSync(directory, {bigint: true});
    assert(before.isDirectory() && !before.isSymbolicLink(), 'regular compiler directory required');
    assert.equal(before.mode & 0o7000n, 0n, 'special compiler directory permissions refused');
    assert.equal(realpathSync(directory), directory, 'compiler ancestor is aliased');
    for (const name of readdirSync(directory).sort(order)) {
      assert(/^[A-Za-z0-9_.+-]+$/.test(name), 'noncanonical compiler entry');
      assert(rows.length < bounds.files, 'compiler entry count exceeded');
      const path = join(directory, name), relative = prefix ? `${prefix}/${name}` : name;
      const stat = lstatSync(path, {bigint: true});
      if (stat.isDirectory()) {
        rows.push({path: relative, kind: 'directory', mode: Number(stat.mode & 0o777n)});
        visit(path, relative);
      } else if (stat.isSymbolicLink()) {
        assert(toolchainAliases, 'compiler seed/source alias refused');
        const target = readlinkSync(path);
        assert(!target.startsWith('/') && !target.split('/').includes('..'), 'unconfined toolchain alias');
        const resolved = realpathSync(path);
        assert(resolved.startsWith(root + sep) && lstatSync(resolved).isFile(), 'toolchain alias escapes closure');
        unchanged(stat, lstatSync(path, {bigint: true}));
        assert.equal(readlinkSync(path), target);
        rows.push({path: relative, kind: 'symlink', target, mode: Number(stat.mode & 0o777n)});
      } else {
        const row = snapshotFile(path, Math.min(bounds.file, bounds.total - total));
        total += row.byte_length;
        rows.push({path: relative, kind: 'file', ...row});
      }
    }
    unchanged(before, lstatSync(directory, {bigint: true}));
  }
  visit(root, '');
  return rows.sort((a, b) => order(a.path, b.path));
}

export function constructionEnvironment(environment, directory) {
  boundedConstructionRoot(directory);
  return {...environment, TMPDIR: directory};
}

export function createConstructionStage(root) {
  boundedConstructionRoot(root);
  const path = join(root, 'prismpm-exporter-construction');
  mkdirSync(path, {mode: 0o700});
  return {path, identity: lstatSync(path)};
}

export function runConstruction(program, argv, cwd, environment) {
  const result = spawnSync('/usr/bin/timeout', ['--signal=TERM', '--kill-after=5s', '360s', program, ...argv],
    {cwd, env: environment, encoding: 'utf8', timeout: 370000, maxBuffer: 16 * 1024 * 1024});
  assert.ifError(result.error);
  assert.equal(result.signal, null, 'exporter construction interrupted');
  assert.equal(result.status, 0, `exporter construction failed: ${result.stderr}`);
  return {argv: [program, ...argv], environment, executable_sha256: snapshotFile(realpathSync(program)).sha256,
    exit_code: result.status, stdout: result.stdout, stderr: result.stderr};
}

export function runtimePaths(stdout) {
  if (stdout.trim() === 'statically linked') return [];
  const paths = [];
  for (const line of stdout.split('\n').filter(line => line.trim())) {
    if (/^\s*linux-vdso\.so\.1 \(0x[0-9a-f]+\)$/.test(line)) continue;
    const path = /(?:=>\s+|^\s*)(\/[^\s]+)\s+\(0x[0-9a-f]+\)$/.exec(line)?.[1];
    assert(path, 'unresolved or unrecognized compiler runtime dependency');
    paths.push(resolve(path));
  }
  assert(paths.length, 'empty runtime dependency observation');
  return paths;
}

function runtimeClosure(programs, toolchain, cwd, environment) {
  const paths = new Map();
  for (const program of programs) {
    const fd = openSync(program, constants.O_RDONLY | constants.O_NOFOLLOW);
    const magic = Buffer.alloc(4);
    try { readSync(fd, magic, 0, 4, 0); } finally { closeSync(fd); }
    if (!magic.equals(Buffer.from([0x7f, 0x45, 0x4c, 0x46]))) continue;
    const observation = runConstruction('/usr/bin/ldd', [program], cwd, environment);
    for (const selected of runtimePaths(observation.stdout)) {
      const canonical = realpathSync(selected);
      if (canonical.startsWith(toolchain + sep)) continue;
      paths.set(selected, {selected, path: canonical, ...snapshotFile(canonical)});
    }
  }
  return [...paths.values()].sort((a, b) => order(a.selected, b.selected));
}

export function buildSeed(source, destination) {
  assert.equal(process.platform, 'linux');
  const architecture = {x64: 'amd64', arm64: 'arm64'}[process.arch];
  assert(architecture, 'native supported SDK platform required');
  assert.equal(realpathSync(source), source);
  assert.equal(lstatSync(destination, {throwIfNoEntry: false}), undefined, 'cannot overwrite exporter seed');
  assert.equal(realpathSync(dirname(destination)), dirname(destination));
  const constructionRoot = boundedConstructionRoot(realpathSync(tmpdir()));
  const dependencyPath = join(source, 'model/dependencies.toml');
  const dependency = readSmall(dependencyPath, 1024 * 1024);
  const revision = compilerRevision(dependency);
  const archive = join(source, 'vendor/lean4-prod/lean.tar');
  const archiveRow = snapshotFile(archive, 16 * 1024 * 1024);
  const records = dependency.split('[[dependency.artifact]]').slice(1).map(section => section.split('[[dependency]]')[0]);
  const matches = records.filter(section => /^path = "vendor\/lean4-prod\/lean.tar"$/m.test(section));
  assert.equal(matches.length, 1, 'one registered exporter archive required');
  assert.equal(/^sha256 = "([0-9a-f]{64})"$/m.exec(matches[0])?.[1], archiveRow.sha256);
  const toolchainName = readSmall(join(source, 'lean-toolchain'), 256).trim();
  assert.match(toolchainName, /^leanprover\/lean4:v[0-9]+\.[0-9]+\.[0-9]+$/);
  const toolchain = '/usr/local/elan/toolchains/' + toolchainName.replace('/', '--').replace(':', '---');
  const toolsBefore = snapshotTree(toolchain, {toolchainAliases: true});
  const environment = {PATH: `${toolchain}/bin:/usr/bin:/bin`, LANG: 'C', LC_ALL: 'C',
    ELAN_HOME: '/usr/local/elan', ELAN_TOOLCHAIN: toolchainName, LEAN_NUM_THREADS: '2', SOURCE_DATE_EPOCH: '0'};
  // Lake records absolute compiler paths in its traces. A fixed private path
  // inside each isolated SDK build makes those genuine traces reproducible.
  // Exclusive creation rejects collisions; existing state is never reused.
  const {path: staging, identity: owned} = createConstructionStage(constructionRoot);
  try {
    const childEnvironment = constructionEnvironment(environment, staging);
    const packageRoot = join(staging, 'package'); mkdirSync(packageRoot);
    const extraction = runConstruction('/usr/bin/tar', ['--extract', '--file', archive, '--directory', packageRoot], source, childEnvironment);
    assert.deepEqual(snapshotFile(archive, 16 * 1024 * 1024), archiveRow);
    assert.equal(lstatSync(join(packageRoot, '.lake'), {throwIfNoEntry: false}), undefined);
    const sources = snapshotTree(packageRoot);
    const build = runConstruction(join(toolchain, 'bin/lake'), ['build', 'prod-export'], packageRoot, childEnvironment);
    const sourcesAfter = snapshotTree(packageRoot, {bounds: seedLimit}).filter(row => row.path !== '.lake' && !row.path.startsWith('.lake/'));
    assert.deepEqual(sourcesAfter, sources, 'exporter construction changed source/configuration');
    const seedRoot = join(staging, 'seed'); mkdirSync(seedRoot);
    renameSync(join(packageRoot, '.lake'), join(seedRoot, '.lake'));
    const files = snapshotTree(seedRoot, {bounds: seedLimit});
    assert(files.some(row => row.path === '.lake/build/bin/prod-export' && row.kind === 'file' && (row.mode & 0o111)),
      'actual native exporter missing');
    assert.deepEqual(snapshotTree(toolchain, {toolchainAliases: true}), toolsBefore, 'toolchain changed during construction');
    const programs = toolsBefore.filter(row => row.kind === 'file' && row.path.startsWith('bin/') && (row.mode & 0o111))
      .map(row => join(toolchain, row.path));
    const runtime = runtimeClosure([...programs, join(seedRoot, '.lake/build/bin/prod-export')], toolchain, source, childEnvironment);
    const manifest = {schema: 'prismpm/exporter-seed/1', platform: `linux/${architecture}`,
      compiler_revision: revision, archive_sha256: archiveRow.sha256, toolchain: toolchainName,
      configuration: {argv: ['build', 'prod-export'], environment, construction_root: staging,
        temporary_directory: 'private-bounded-tmpfs'},
      source_files: sources, toolchain_files: toolsBefore, runtime_files: runtime, files};
    const encoded = encodeInventory(manifest);
    assert(Buffer.byteLength(encoded) <= 8 * 1024 * 1024, 'exporter manifest exceeded bound');
    // Snapshot/size checks precede the only persistent copy. An interrupted
    // copy never publishes the destination; raw process output stays separate
    // from the deterministic manifest and is not rewritten to invent a build.
    const publication = mkdtempSync(join(dirname(destination), '.exporter-publication-'));
    const publicationIdentity = lstatSync(publication);
    try {
      const payload = join(publication, 'seed');
      cpSync(seedRoot, payload, {recursive: true, errorOnExist: true, force: false, preserveTimestamps: true});
      assert.deepEqual(snapshotTree(payload, {bounds: seedLimit}), files, 'exporter seed copy changed');
      writeFileSync(join(payload, 'manifest.json'), encoded, {flag: 'wx', mode: 0o444});
      assert.equal(lstatSync(destination, {throwIfNoEntry: false}), undefined);
      renameSync(payload, destination);
    } finally {
      const current = lstatSync(publication);
      assert(current.isDirectory() && current.dev === publicationIdentity.dev && current.ino === publicationIdentity.ino);
      rmSync(publication, {recursive: true});
    }
    return {manifest_sha256: sha(encoded), files: files.length, construction: {extraction, build}};
  } finally {
    const current = lstatSync(staging);
    assert(current.isDirectory() && current.dev === owned.dev && current.ino === owned.ino, 'owned exporter staging replaced');
    rmSync(staging, {recursive: true});
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert.equal(process.argv.length, 4, 'usage: exporter-seed.mjs SOURCE DESTINATION');
  process.stdout.write(encodeInventory(buildSeed(resolve(process.argv[2]), resolve(process.argv[3]))));
}
