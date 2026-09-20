import assert from 'node:assert/strict';
import {existsSync, mkdirSync, mkdtempSync, readFileSync, renameSync, rmSync, symlinkSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname, join} from 'node:path';
import test from 'node:test';
import {retireCompletedCompilerCaches} from '../tests/browser-view/driver-cache.mjs';
import {run, sha} from '../tests/browser-view/compile.mjs';

function fixture(t, prefix = 'prismpm-publication-') {
  const work = mkdtempSync(join(tmpdir(), prefix));
  t.after(() => rmSync(work, {recursive:true, force:true}));
  const manifest = join(work, 'tests/publication-admission/driver/Cargo.toml');
  mkdirSync(join(dirname(manifest), 'src'), {recursive:true});
  writeFileSync(manifest, '[package]\nname="publication-admission-driver"\nversion="0.1.0"\nedition="2021"\npublish=false\n[workspace]\n');
  writeFileSync(join(dirname(manifest), 'src/main.rs'), 'fn main() {}\n');
  writeFileSync(join(dirname(manifest), 'Cargo.lock'), 'version = 4\n[[package]]\nname="publication-admission-driver"\nversion="0.1.0"\n');
  const target = join(work, 'driver-target');
  run('cargo', ['build','--locked','--offline','--jobs','1','--manifest-path',manifest], work, {CARGO_TARGET_DIR:target});
  const exporter = join(work, 'exporter');
  mkdirSync(join(exporter, '.lake/build/bin'), {recursive:true});
  writeFileSync(join(exporter, 'lakefile.toml'), 'name="private_cache_probe"\nversion="0.1.0"\n');
  // This is cache fixture data, never executed or passed off as a compiler.
  writeFileSync(join(exporter, '.lake/build/bin/prod-export'), 'reconstructible test cache\n');
  const preserved = new Map(['source.lex.tex','proof.json','kernel.ir','guest.wasm'].map(name => [name, Buffer.from(name)]));
  for (const [name, bytes] of preserved) writeFileSync(join(work,name), bytes);
  return {work, target, manifest, exporter, preserved};
}

test('actual Cargo and Lake retirement preserves all non-cache evidence and exact original tool identities', t => {
  const f = fixture(t);
  const binary = readFileSync(join(f.target,'debug/publication-admission-driver'));
  const result = retireCompletedCompilerCaches(f.work, 'publication');
  assert.equal(result.scope, 'completed-private-tool-caches-only');
  assert.deepEqual(result.records[1], {path:'driver-target/debug/publication-admission-driver', byte_length:binary.length, sha256:sha(binary)});
  assert.deepEqual(JSON.parse(readFileSync(join(f.work,'compiler-cache-retirement.json'))), result);
  for (const [name, bytes] of f.preserved) assert.deepEqual(readFileSync(join(f.work,name)),bytes);
  assert(existsSync(f.manifest) && existsSync(join(f.exporter,'lakefile.toml')));
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
