import assert from 'node:assert/strict';
import {chmodSync, copyFileSync, cpSync, existsSync, linkSync, mkdirSync, mkdtempSync, readFileSync, renameSync, rmSync, symlinkSync, writeFileSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import {tmpdir} from 'node:os';
import {dirname, join} from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import {retireCompletedCompilerCaches} from '../tests/browser-view/driver-cache.mjs';
import {ensureProdExport, repository, run, sha} from '../tests/browser-view/compile.mjs';
import * as compiler from '../tests/browser-view/compile.mjs';

// Reuse the actual pinned exporter configuration; do not substitute a Lake
// manifest format which none of these compiler fixtures executes.
const archive = fileURLToPath(new URL('../vendor/lean4-prod/lean.tar', import.meta.url));
const exporterManifest = Buffer.from(run('tar', ['-xOf', archive, 'lakefile.lean'], dirname(archive)));

function exporterFixture(t) {
  const repo = mkdtempSync(join(tmpdir(), 'prismpm-exporter-adversary-'));
  t.after(() => rmSync(repo, {recursive:true, force:true}));
  for (const path of ['model/dependencies.toml', 'lean-toolchain', 'rust-toolchain.toml', 'vendor/lean4-prod/lean.tar']) {
    mkdirSync(dirname(join(repo, path)), {recursive:true});
    copyFileSync(join(repository, path), join(repo, path));
  }
  for (const path of ['vendor/lexlean', 'vendor/lean4-prod/rust']) {
    cpSync(join(repository, path), join(repo, path), {recursive:true});
  }
  const cache = join(repo, 'target/lean4-prod-export');
  const executable = join(cache, '.lake/build/bin/prod-export');
  const marker = join(repo, 'planted-executable-ran');
  mkdirSync(dirname(executable), {recursive:true});
  writeFileSync(executable, '#!/bin/sh\nprintf planted > "' + marker + '"\nexit 0\n');
  chmodSync(executable, 0o700);
  mkdirSync(join(cache, 'Prod'));
  writeFileSync(join(cache, 'Prod/Emit.lean'), 'untrusted cached source\n');
  return {repo, cache, executable, marker};
}

test('a planted executable and source cache cannot replace a freshly compiled pinned exporter', t => {
  const f = exporterFixture(t), bytes = readFileSync(f.executable);
  const result = ensureProdExport(f.repo);
  t.after(() => { if (result.dir !== f.cache) rmSync(result.dir, {recursive:true, force:true}); });
  const probe = spawnSync(result.bin, ['--module'], {cwd:result.dir, encoding:'utf8', timeout:30000});
  assert.ifError(probe.error);
  assert(!existsSync(f.marker), 'never execute an unproved cached binary');
  assert.notEqual(result.dir, f.cache);
  assert.notEqual(probe.status, 0, 'actual exporter rejects invalid arguments');
  assert.match(probe.stderr, /prod-export: unknown or incomplete named-export argument `--module`/);
  assert.deepEqual(readFileSync(f.executable), bytes, 'unowned old cache is not overwritten');
  assert.equal(readFileSync(join(f.cache, 'Prod/Emit.lean'), 'utf8'), 'untrusted cached source\n');
});

test('a linked shared cache is neither executed nor overwritten', t => {
  const f = exporterFixture(t), retained = f.cache + '-retained';
  renameSync(f.cache, retained); symlinkSync(retained, f.cache);
  const result = ensureProdExport(f.repo);
  t.after(() => { if (result.dir !== f.cache) rmSync(result.dir, {recursive:true, force:true}); });
  const probe = spawnSync(result.bin, ['--module'], {cwd:result.dir, encoding:'utf8', timeout:30000});
  assert.ifError(probe.error);
  assert(!existsSync(f.marker), 'never execute through an unproved cached link');
  assert.notEqual(result.dir, f.cache);
  assert.notEqual(probe.status, 0);
  assert.match(probe.stderr, /prod-export: unknown or incomplete named-export argument `--module`/);
  assert.equal(readFileSync(join(retained, 'Prod/Emit.lean'), 'utf8'), 'untrusted cached source\n');
});

test('changed source pins reject even when a cached executable exists', t => {
  const f = exporterFixture(t);
  writeFileSync(join(f.repo, 'vendor/lean4-prod/lean.tar'), 'changed pinned source');
  assert.throws(() => ensureProdExport(f.repo), /lean.tar/);
  assert(!existsSync(f.marker));
});

test('aliased pinned source rejects even when its bytes and an existing executable appear valid', t => {
  const f = exporterFixture(t), source = join(f.repo, 'vendor/lean4-prod/lean.tar');
  renameSync(source, source + '-retained'); symlinkSync(source + '-retained', source);
  assert.throws(() => ensureProdExport(f.repo), /aliased/);
  assert(!existsSync(f.marker));
});

test('multiply linked pinned source is not an immutable private input', t => {
  const f = exporterFixture(t), source = join(f.repo, 'vendor/lean4-prod/lean.tar');
  linkSync(source, source + '-linked');
  assert.throws(() => ensureProdExport(f.repo), /singly owned regular compiler source/);
  assert(!existsSync(f.marker));
});

test('a linked private driver parent rejects before creating a target', t => {
  const parent = mkdtempSync(join(tmpdir(), 'prismpm-driver-parent-'));
  t.after(() => rmSync(parent, {recursive:true, force:true}));
  const work = join(parent, 'actual'), alias = join(parent, 'alias');
  mkdirSync(work); symlinkSync(work, alias);
  assert.throws(() => compiler.createPrivateDriverTarget(alias), /aliased driver parent/);
  assert(!existsSync(join(work, 'driver-target')));
});

test('a private exporter destination cannot be adopted or overwritten', t => {
  const f = exporterFixture(t), work = join(f.repo, 'private-work');
  mkdirSync(join(work, 'exporter'), {recursive:true});
  writeFileSync(join(work, 'exporter/evidence'), 'preserve');
  assert.throws(() => ensureProdExport(f.repo, work), /exist|EEXIST/);
  assert.equal(readFileSync(join(work, 'exporter/evidence'), 'utf8'), 'preserve');
});

function fixture(t, prefix = 'prismpm-publication-', owner = 'publication') {
  const work = mkdtempSync(join(tmpdir(), prefix));
  t.after(() => rmSync(work, {recursive:true, force:true}));
  const directory = {publication:'publication-admission', effects:'browser-effects', custody:'browser-custody',
    'operation-journal':'browser-operation-journal', presentation:'browser-presentation',
    view:'browser-view', journal:'browser-journal', query:'browser-query', command:'browser-command'}[owner];
  assert.ok(directory);
  const executable = (['view','journal','query','command'].includes(owner) ? 'browser-workspace-' + owner : directory) + '-driver';
  const manifest = join(work, 'tests', directory, 'driver/Cargo.toml');
  mkdirSync(join(dirname(manifest), 'src'), {recursive:true});
  writeFileSync(manifest, '[package]\nname="' + executable + '"\nversion="0.1.0"\nedition="2021"\npublish=false\n[workspace]\n');
  writeFileSync(join(dirname(manifest), 'src/main.rs'), 'fn main() {}\n');
  writeFileSync(join(dirname(manifest), 'Cargo.lock'), 'version = 4\n[[package]]\nname="' + executable + '"\nversion="0.1.0"\n');
  const target = join(work, 'driver-target');
  run('cargo', ['build','--locked','--offline','--jobs','1','--manifest-path',manifest], work, {CARGO_TARGET_DIR:target});
  const exporter = join(work, 'exporter');
  mkdirSync(join(exporter, '.lake/build/bin'), {recursive:true});
  writeFileSync(join(exporter, 'lakefile.lean'), exporterManifest);
  // This is cache fixture data, never executed or passed off as a compiler.
  writeFileSync(join(exporter, '.lake/build/bin/prod-export'), 'reconstructible test cache\n');
  const preserved = new Map(['source.lex.tex','proof.json','kernel.ir','guest.wasm'].map(name => [name, Buffer.from(name)]));
  for (const [name, bytes] of preserved) writeFileSync(join(work,name), bytes);
  return {work, target, manifest, exporter, preserved, executable};
}

test('Cargo fingerprints cannot authorize a planted driver executable or reuse its target', t => {
  assert.equal(typeof compiler.createPrivateDriverTarget, 'function');
  const f = fixture(t), driver = join(f.target, 'debug', f.executable);
  const poison = '#!/bin/sh\nprintf planted-driver\n';
  writeFileSync(driver, poison);
  // Real Cargo considers this source unchanged; it does not authenticate the
  // replaced output. The owner must create a new target before compiling.
  run('cargo', ['build','--locked','--offline','--jobs','1','--manifest-path',f.manifest], f.work, {CARGO_TARGET_DIR:f.target});
  assert.equal(run(driver, [], f.work), 'planted-driver');
  assert.throws(() => compiler.createPrivateDriverTarget(f.work), /already exists/);
  assert.equal(readFileSync(driver, 'utf8'), poison, 'rejection preserves unowned cache evidence');
  const fresh = mkdtempSync(join(tmpdir(), 'prismpm-driver-fresh-'));
  t.after(() => rmSync(fresh, {recursive:true, force:true}));
  const target = compiler.createPrivateDriverTarget(fresh);
  assert.equal(target, join(fresh, 'driver-target'));
  run('cargo', ['build','--locked','--offline','--jobs','1','--manifest-path',f.manifest], f.work, {CARGO_TARGET_DIR:target});
  assert.equal(readFileSync(join(target, 'CACHEDIR.TAG'), 'utf8').split('\n')[0],
    'Signature: 8a477f597d28d172789f06886806bc55', 'Cargo must initialize its own private cache');
  assert.equal(run(join(target, 'debug', f.executable), [], f.work), '');
  assert.throws(() => compiler.createPrivateDriverTarget(fresh), /already exists/);
});

test('completed effects and custody tool caches retire under their exact owning paths', t => {
  for (const owner of ['effects', 'custody']) {
    const f = fixture(t, 'prismpm-' + owner + '-', owner);
    const path = join(f.target, 'debug', f.executable), bytes = readFileSync(path);
    const receipt = retireCompletedCompilerCaches(f.work, owner);
    assert.equal(receipt.owner, owner);
    assert.deepEqual(receipt.records[1], {path:'driver-target/debug/' + f.executable,
      byte_length:bytes.length, sha256:sha(bytes)});
    assert(!existsSync(path) && !existsSync(join(f.exporter, '.lake/build')));
    assert(existsSync(f.manifest));
    for (const [name, preserved] of f.preserved) assert.deepEqual(readFileSync(join(f.work, name)), preserved);
    assert.throws(() => retireCompletedCompilerCaches(f.work, owner));
  }
});

test('all remaining retained browser fixtures retire only their exact tool caches', t => {
  for (const owner of ['operation-journal','presentation','view','journal','query','command']) {
    const f = fixture(t, 'prismpm-' + owner + '-', owner);
    const source = join(repository, 'tests/browser-' + owner + '/driver/Cargo.toml');
    const original = readFileSync(source);
    const binary = readFileSync(join(f.target, 'debug', f.executable));
    const result = retireCompletedCompilerCaches(f.work, owner);
    assert.equal(result.owner, owner);
    const external = ['view','journal','query','command'].includes(owner);
    assert.deepEqual(result.records[0], {path:external ? 'repository/tests/browser-' + owner + '/driver/Cargo.toml'
      : 'tests/browser-' + owner + '/driver/Cargo.toml',
    byte_length:external ? original.length : readFileSync(f.manifest).length,
    sha256:sha(external ? original : readFileSync(f.manifest))});
    assert.deepEqual(result.records[1], {path:'driver-target/debug/' + f.executable,
      byte_length:binary.length, sha256:sha(binary)});
    assert.deepEqual(readFileSync(source), original, 'source manifest must never be modified by retirement');
    assert(!existsSync(join(f.target, 'debug', f.executable)) && !existsSync(join(f.exporter, '.lake/build')));
    for (const [name, bytes] of f.preserved) assert.deepEqual(readFileSync(join(f.work, name)), bytes);
    assert.throws(() => retireCompletedCompilerCaches(f.work, owner), /overwrite/);
  }
});

test('actual Cargo and Lake retirement preserves all non-cache evidence and exact original tool identities', t => {
  const f = fixture(t);
  const binary = readFileSync(join(f.target,'debug/publication-admission-driver'));
  const result = retireCompletedCompilerCaches(f.work, 'publication');
  assert.equal(result.scope, 'completed-private-tool-caches-only');
  assert.deepEqual(result.records[1], {path:'driver-target/debug/publication-admission-driver', byte_length:binary.length, sha256:sha(binary)});
  assert.deepEqual(JSON.parse(readFileSync(join(f.work,'compiler-cache-retirement.json'))), result);
  for (const [name, bytes] of f.preserved) assert.deepEqual(readFileSync(join(f.work,name)),bytes);
  assert(existsSync(f.manifest));
  assert.deepEqual(readFileSync(join(f.exporter,'lakefile.lean')), exporterManifest);
  assert.deepEqual(result.records[2], {path:'exporter/lakefile.lean', byte_length:exporterManifest.length, sha256:sha(exporterManifest)});
  assert(!existsSync(join(f.target,'debug/publication-admission-driver')));
  assert(!existsSync(join(f.exporter,'.lake/build')));
  assert.throws(() => retireCompletedCompilerCaches(f.work, 'publication'));
});

test('invalid owner, prefix, tag, aliases and existing evidence reject before any deletion', t => {
  const f = fixture(t), driver = join(f.target,'debug/publication-admission-driver');
  assert.throws(() => retireCompletedCompilerCaches(f.work, 'unknown'), /unregistered/);
  assert.throws(() => retireCompletedCompilerCaches(f.work, 'budget'), /owned work prefix/);
  const tag = join(f.target,'CACHEDIR.TAG'), original = readFileSync(tag);
  writeFileSync(tag, 'not a Cargo cache\n');
  assert.throws(() => retireCompletedCompilerCaches(f.work,'publication'), /cache tag/);
  writeFileSync(tag, original);
  renameSync(f.target, f.target + '-retained'); symlinkSync(f.target + '-retained', f.target);
  assert.throws(() => retireCompletedCompilerCaches(f.work,'publication'), /aliased/);
  rmSync(f.target); renameSync(f.target + '-retained',f.target);
  renameSync(f.manifest,f.manifest + '-retained'); symlinkSync(f.manifest + '-retained',f.manifest);
  assert.throws(() => retireCompletedCompilerCaches(f.work,'publication'));
  rmSync(f.manifest); renameSync(f.manifest + '-retained',f.manifest);
  const exporterBinary = join(f.exporter,'.lake/build/bin/prod-export');
  renameSync(exporterBinary,exporterBinary + '-retained');
  assert.throws(() => retireCompletedCompilerCaches(f.work,'publication'));
  assert(existsSync(driver), 'late missing input must not remove the earlier Cargo cache');
  renameSync(exporterBinary + '-retained',exporterBinary);
  writeFileSync(join(f.work,'compiler-cache-retirement.json'),'original evidence');
  assert.throws(() => retireCompletedCompilerCaches(f.work,'publication'), /overwrite/);
  assert.equal(readFileSync(join(f.work,'compiler-cache-retirement.json'),'utf8'),'original evidence');
  assert(existsSync(driver) && existsSync(join(f.exporter,'.lake/build/bin/prod-export')));
  for (const [name,bytes] of f.preserved) assert.deepEqual(readFileSync(join(f.work,name)),bytes);
});

test('removing the owned-prefix guard is detected by actual cleanup behavior', async t => {
  const f = fixture(t, 'unowned-cache-');
  assert.throws(() => retireCompletedCompilerCaches(f.work,'publication'), /owned work prefix/);
  const path = new URL('../tests/browser-view/driver-cache.mjs', import.meta.url);
  const source = readFileSync(path,'utf8');
  const guard = "  assert.match(basename(work), new RegExp('^prismpm-' + owner + '-[A-Za-z0-9]+$'), 'owned work prefix required');";
  assert.equal(source.split(guard).length,2);
  const mutation = source.replace(guard,'').replace("'./compile.mjs'", JSON.stringify(new URL('../tests/browser-view/compile.mjs',import.meta.url).href));
  const changed = await import('data:text/javascript;base64,' + Buffer.from(mutation).toString('base64'));
  assert.throws(() => assert.throws(() => changed.retireCompletedCompilerCaches(f.work,'publication'), /owned work prefix/), /Missing expected exception/);
  assert(!existsSync(join(f.target,'debug/publication-admission-driver')));
  for (const [name,bytes] of f.preserved) assert.deepEqual(readFileSync(join(f.work,name)),bytes);
});
