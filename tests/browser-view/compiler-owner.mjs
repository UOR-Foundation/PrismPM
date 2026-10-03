// One freshly compiled tool closure per serial verification owner. This never
// adopts a prior Cargo target, on-disk receipt, caller executable or model output.
import assert from 'node:assert/strict';
import {closeSync, constants, fstatSync, lstatSync, mkdirSync, mkdtempSync, openSync, readFileSync, readdirSync,
  readSync, realpathSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname, join} from 'node:path';
import {createPrivateDriverTarget, ensureProdExport, repository, run, sha} from './compile.mjs';
import {captureCompilerArtifact} from './compiler-artifact.mjs';

const families = Object.freeze({
  view: Object.freeze({directory: 'browser-view', executable: 'browser-workspace-view-driver'}),
  effects: Object.freeze({directory: 'browser-effects', executable: 'browser-effects-driver'}),
  presentation: Object.freeze({directory: 'browser-presentation', executable: 'browser-presentation-driver'}),
  'operation-journal': Object.freeze({directory: 'browser-operation-journal', executable: 'browser-operation-journal-driver'}),
  budget: Object.freeze({directory: 'browser-budget', executable: 'browser-budget-driver'}),
  custody: Object.freeze({directory: 'browser-custody', executable: 'browser-custody-driver'}),
});
const owners = new WeakMap();
function family(name) {
  assert.ok(typeof name === 'string' && Object.hasOwn(families, name), 'registered compiler family required');
  return families[name];
}
function requiredInputs(name) {
  const selected = family(name), manifest = 'tests/' + selected.directory + '/driver/Cargo.toml';
  const paths = new Set([manifest, manifest.replace('Cargo.toml', 'Cargo.lock'), manifest.replace('Cargo.toml', 'src/main.rs'),
    'lean-toolchain', 'rust-toolchain.toml', 'model/dependencies.toml', 'model/authorities.toml',
    'vendor/lean4-prod/lean.tar', 'tests/browser-view/compile.mjs', 'tests/browser-view/driver-cache.mjs',
    'tests/browser-view/compiler-owner.mjs', 'tests/browser-view/compiler-artifact.mjs']);
  for (const tree of ['vendor/lexlean', 'vendor/lean4-prod/rust']) {
    const manifestPath = tree + '/MANIFEST.sha256'; paths.add(manifestPath);
    for (const line of file(join(repository, manifestPath)).toString().trimEnd().split('\n')) {
      const row = /^([a-f0-9]{64})  ([A-Za-z0-9_./-]+)$/.exec(line); assert.ok(row, 'registered compiler manifest row');
      assert(!row[2].startsWith('/') && !row[2].split('/').some(part => !part || part === '.' || part === '..'));
      const path = tree + '/' + row[2]; assert(!paths.has(path), 'duplicate compiler input'); paths.add(path);
    }
  }
  return [...paths].sort();
}
export function captureCompilerInputs(name) {
  return inputMap(Object.fromEntries(requiredInputs(name).map(path => [path, sha(file(join(repository, path)))])));
}
function inputMap(value) {
  assert.ok(value && Object.getPrototypeOf(value) === Object.prototype, 'compiler input map required');
  const fields = Object.getOwnPropertyDescriptors(value), keys = Reflect.ownKeys(fields);
  assert.ok(keys.every(key => typeof key === 'string' && 'value' in fields[key]), 'compiler input map must contain data');
  return Object.freeze(Object.fromEntries(keys.sort().map(path => {
    assert.ok(/^[A-Za-z0-9_./-]+$/.test(path)
      && !path.split('/').some(part => part === '' || part === '.' || part === '..'), 'compiler input path refused');
    const digest = fields[path].value;
    assert.ok(typeof digest === 'string' && /^[a-f0-9]{64}$/.test(digest), 'compiler input digest refused');
    return [path, digest];
  })));
}
function file(path, expected) {
  assert.equal(realpathSync(path), path, 'unaliased compiler source required');
  const fd = openSync(path, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
  try {
    const before = fstatSync(fd, {bigint: true});
    assert(before.isFile() && before.nlink === 1n && before.size <= 268435456n, 'bounded single-link compiler source required');
    const bytes = Buffer.alloc(Number(before.size));
    for (let offset = 0; offset < bytes.length;) {
      const count = readSync(fd, bytes, offset, bytes.length - offset, null); assert(count > 0, 'compiler source shortened'); offset += count;
    }
    assert.equal(readSync(fd, Buffer.alloc(1), 0, 1, null), 0, 'compiler source grew');
    for (const after of [fstatSync(fd, {bigint: true}), lstatSync(path, {bigint: true})])
      for (const key of ['dev', 'ino', 'uid', 'mode', 'size', 'nlink', 'mtimeNs', 'ctimeNs']) assert.equal(after[key], before[key], 'stable compiler source');
    assert.equal(realpathSync(path), path, 'compiler source ancestry changed');
    if (expected !== undefined) assert.equal(sha(bytes), expected, 'captured compiler source ' + path);
    return bytes;
  } finally {closeSync(fd);}
}
function exporterTree(root) {
  const entries = {};
  function visit(path, relative) {
    assert.equal(realpathSync(path), path, 'unaliased exporter runtime closure');
    const stat = lstatSync(path, {bigint: true});
    assert.equal(stat.uid, BigInt(process.getuid()), 'owned exporter runtime closure');
    assert.equal(stat.mode & 0o022n, 0n, 'exporter runtime cannot be group/other writable');
    if (stat.isDirectory()) {
      entries[relative] = {kind: 'directory', inode: stat.ino.toString()};
      for (const name of readdirSync(path).sort()) visit(join(path, name), relative + '/' + name);
    } else {
      assert.ok(stat.isFile() && stat.size <= 268435456n, 'bounded regular exporter runtime input');
      entries[relative] = {kind: 'file', inode: stat.ino.toString(), links: stat.nlink.toString(),
        size: Number(stat.size), sha256: sha(file(path))};
    }
  }
  visit(root, '.');
  return Object.freeze(Object.fromEntries(Object.entries(entries).map(([path, value]) => [path, Object.freeze(value)])));
}

export function createCompilerOwner(name, selectedInputs = captureCompilerInputs(name)) {
  const selected = family(name), inputs = inputMap(selectedInputs);
  assert.deepEqual(Object.keys(inputs), requiredInputs(name), 'complete closed compiler input closure required');
  const manifest = 'tests/' + selected.directory + '/driver/Cargo.toml';
  const driverFiles = [manifest, manifest.replace('Cargo.toml', 'Cargo.lock'), manifest.replace('Cargo.toml', 'src/main.rs')];
  for (const path of [...driverFiles, 'lean-toolchain', 'rust-toolchain.toml', 'model/dependencies.toml',
    'model/authorities.toml', 'vendor/lean4-prod/lean.tar', 'vendor/lean4-prod/rust/MANIFEST.sha256',
      'vendor/lexlean/MANIFEST.sha256', 'tests/browser-view/compile.mjs', 'tests/browser-view/compiler-owner.mjs',
    'tests/browser-view/compiler-artifact.mjs']) assert.ok(Object.hasOwn(inputs, path), 'compiler input closure missing ' + path);
  const captured = new Map(Object.entries(inputs).map(([path, digest]) => [path, file(join(repository, path), digest)]));
  for (const tree of ['vendor/lexlean', 'vendor/lean4-prod/rust']) {
    for (const line of captured.get(tree + '/MANIFEST.sha256').toString().trimEnd().split('\n')) {
      const match = /^([a-f0-9]{64})  ([A-Za-z0-9_./-]+)$/.exec(line); assert.ok(match);
      assert.equal(inputs[tree + '/' + match[2]], match[1], 'complete registered compiler source inventory');
    }
  }
  const work = mkdtempSync(join(tmpdir(), 'prismpm-compiler-owner-')), staged = new Map();
  function stage(path, bytes) {
    const target = join(work, path); mkdirSync(dirname(target), {recursive: true, mode: 0o700});
    writeFileSync(target, bytes, {flag: 'wx', mode: 0o600}); staged.set(path, sha(bytes));
  }
  for (const [path, bytes] of captured) if (path.startsWith('vendor/lexlean/')
    || path.startsWith('vendor/lean4-prod/rust/') || driverFiles.includes(path)
    || ['lean-toolchain', 'rust-toolchain.toml'].includes(path)) stage(path, bytes);
  const began = performance.now();
  let constructed = false;
  try {
    const exporterStarted = performance.now(), exporter = ensureProdExport(repository, work);
    const exporterBuildMs = performance.now() - exporterStarted;
    const runtime = exporterTree(exporter.dir);
    const exportArtifact = captureCompilerArtifact(work, exporter.bin, 'exporter');
    const target = createPrivateDriverTarget(work), driverStarted = performance.now();
    run('cargo', ['build', '--locked', '--offline', '--jobs', '1', '--config', 'profile.dev.debug=0',
      '--config', 'build.incremental=false', '--manifest-path', join(work, manifest)], work, {CARGO_TARGET_DIR: target});
    const driverBuildMs = performance.now() - driverStarted;
    const driver = captureCompilerArtifact(work, join(target, 'debug', selected.executable), 'driver');
    const evidence = Object.freeze({scope: 'fresh-owner-scoped-compiler-tools', family: name, work, inputs,
      exporter: exportArtifact.evidence, driver: driver.evidence, exporterRuntime: runtime,
      exporterBuildMs, driverBuildMs, preparationMs: performance.now() - began});
    let closed = false, busy = false;
    function verify() {
      assert.equal(closed, false, 'compiler owner closed');
      assert.equal(realpathSync(work), work, 'unaliased private compiler owner');
      assert.equal(lstatSync(work).mode & 0o077, 0, 'private compiler owner directory required');
      for (const [path, digest] of Object.entries(inputs)) file(join(repository, path), digest);
      for (const [path, digest] of staged) file(join(work, path), digest);
      assert.deepEqual(exporterTree(exporter.dir), runtime, 'immutable complete exporter runtime closure');
      exportArtifact.verify(); driver.verify();
    }
    function execute(artifact, args, cwd, environment) {
      assert.equal(busy, false, 'serial compiler owner required'); verify(); busy = true;
      try {return artifact.run(args, cwd, environment);} finally {busy = false; verify();}
    }
    const identity = sha(Buffer.from(JSON.stringify(evidence)));
    const owner = Object.freeze({evidence, identity, verify,
      runDriver(args, cwd) {return execute(driver, args, cwd, {});},
      runExporter(args, leanPath) {return execute(exportArtifact, args, exporter.dir, {LEAN_PATH: leanPath});},
      close() {
        assert.equal(busy, false, 'compiler owner still executing'); verify(); closed = true;
        // Retire only this completed owner's reconstructible tool caches.
        // Private executables, input sources and immutable evidence remain.
        writeFileSync(join(work, 'compiler-owner-retirement-intent.json'),
          JSON.stringify({status: 'requested', compiler: identity}) + '\n', {flag: 'wx'});
        run('cargo', ['clean', '--manifest-path', join(work, manifest), '--target-dir', target], work);
        run('lake', ['clean'], exporter.dir);
        assert.equal(lstatSync(join(target, 'debug', selected.executable), {throwIfNoEntry: false}), undefined);
        assert.equal(lstatSync(join(exporter.dir, '.lake/build'), {throwIfNoEntry: false}), undefined);
        for (const artifact of [driver, exportArtifact]) {
          const stat = lstatSync(artifact.path);
          assert.equal(stat.ino.toString(), artifact.evidence.private.inode);
          assert.equal(stat.nlink, 1);
          assert.equal(sha(readFileSync(artifact.path)), artifact.evidence.private.sha256);
        }
        const receipt = Object.freeze({scope: 'completed-owner-compiler-caches', status: 'retired', compiler: identity});
        writeFileSync(join(work, 'compiler-owner-retirement.json'), JSON.stringify(receipt) + '\n', {flag: 'wx'});
        return receipt;
      }});
    owners.set(owner, {inputs, family: name}); verify(); constructed = true;
    return owner;
  } finally {
    if (!constructed) process.stderr.write('Retained failed compiler owner ' + work + '\n');
  }
}

export function requireCompilerOwner(owner, name) {
  const state = owners.get(owner);
  assert.ok(state, 'actual fresh compiler owner required');
  assert.equal(name, state.family, 'same registered compiler family required');
  owner.verify(); return owner;
}
