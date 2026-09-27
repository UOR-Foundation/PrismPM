import assert from 'node:assert/strict';
import test from 'node:test';
import {createHash} from 'node:crypto';
import {linkSync, mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, unlinkSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname, join} from 'node:path';
import {captureGeneratedPackage} from './generated-package.mjs';

const sha = bytes => createHash('sha256').update(bytes).digest('hex'), ir = sha('synthetic fixture IR');
function fixture(t, kind = 'native') {
  const directory = mkdtempSync(join(tmpdir(), 'prismpm-generated-package-test-'));
  t.after(() => rmSync(directory, {recursive: true, force: true}));
  const paths = kind === 'native'
    ? ['Cargo.lock', 'Cargo.toml', 'LICENSE-APACHE', 'LICENSE-MIT', 'README.md', 'src/lib.rs']
    : ['.cargo/config.toml', 'Cargo.lock', 'Cargo.toml', 'src/lib.rs'];
  const files = paths.map(path => {
    const bytes = Buffer.from('synthetic fixture: ' + path + '\n');
    mkdirSync(dirname(join(directory, path)), {recursive: true}); writeFileSync(join(directory, path), bytes, {flag: 'wx'});
    return {path, sha256: sha(bytes)};
  });
  const manifest = {schema: kind === 'native' ? 'lean4-prod/cargo-package-manifest/1' : 'lean4-prod/core-wasm-target/1',
    input_ir_sha256: ir, files};
  const writeManifest = () => writeFileSync(join(directory, 'generation-manifest.json'), JSON.stringify(manifest));
  writeManifest();
  return {directory, manifest, writeManifest, capture: () => captureGeneratedPackage(directory, {kind, inputIrSha256: ir})};
}

test('fresh native and Wasm output have exact declared files and directories', t => {
  for (const kind of ['native', 'wasm']) {
    const value = fixture(t, kind), captured = value.capture(); captured.verify();
    assert.ok(Object.isFrozen(captured) && Object.isFrozen(captured.files));
    assert.equal(Object.keys(captured.files).length, kind === 'native' ? 7 : 5);
  }
});
test('changed source bytes and a rewritten matching manifest cannot replace captured output', t => {
  const value = fixture(t), captured = value.capture(), path = join(value.directory, 'src/lib.rs');
  writeFileSync(path, 'substituted generated source');
  assert.throws(() => captured.verify(), /manifest digest/);
  value.manifest.files.find(row => row.path === 'src/lib.rs').sha256 = sha(readFileSync(path)); value.writeManifest();
  assert.throws(() => captured.verify(), /immutable generated package/);
});
test('manifest rejects wrong IR, schema, digest and missing/extra/repeated declarations', t => {
  for (const change of [m => {m.input_ir_sha256 = sha('other');}, m => {m.schema += 'x';},
    m => {m.files[0].sha256 = 'not-a-digest';}, m => {m.files.pop();},
    m => {m.files.push(m.files[0]);}, m => {m.files[0].path = '../outside';}]) {
    const value = fixture(t); change(value.manifest); value.writeManifest(); assert.throws(value.capture);
  }
});
test('missing files, extra files and empty extra directories are rejected', t => {
  for (const change of [root => unlinkSync(join(root, 'src/lib.rs')),
    root => writeFileSync(join(root, 'extra'), 'unexpected'), root => mkdirSync(join(root, 'extra'))]) {
    const value = fixture(t), captured = value.capture(); change(value.directory);
    assert.throws(value.capture); assert.throws(() => captured.verify());
  }
});
test('file/directory aliases and hard-linked files are rejected', t => {
  for (const mode of ['symlink-file', 'symlink-directory', 'hardlink-file']) {
    const value = fixture(t), captured = value.capture(), extra = mkdtempSync(join(tmpdir(), 'prismpm-generated-alias-test-'));
    t.after(() => rmSync(extra, {recursive: true, force: true}));
    const original = join(value.directory, 'src/lib.rs'), bytes = readFileSync(original);
    if (mode === 'hardlink-file') linkSync(original, join(extra, 'alias'));
    else if (mode === 'symlink-file') {
      const outside = join(extra, 'lib.rs'); writeFileSync(outside, bytes); unlinkSync(original); symlinkSync(outside, original);
    } else {
      rmSync(join(value.directory, 'src'), {recursive: true}); writeFileSync(join(extra, 'lib.rs'), bytes);
      symlinkSync(extra, join(value.directory, 'src'));
    }
    assert.throws(value.capture); assert.throws(() => captured.verify());
  }
});
test('a manifest-only change is rejected even if declared source bytes stay unchanged', t => {
  const value = fixture(t), captured = value.capture(); value.manifest.unexpected = true; value.writeManifest();
  assert.throws(() => captured.verify(), /immutable generated package/);
});
