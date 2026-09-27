// Test infrastructure: freeze a fresh generator's complete declared output.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {lstatSync, readdirSync, readFileSync, realpathSync} from 'node:fs';
import {dirname, join, resolve} from 'node:path';

const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const layouts = Object.freeze({
  native: {schema: 'lean4-prod/cargo-package-manifest/1',
    files: ['Cargo.lock', 'Cargo.toml', 'LICENSE-APACHE', 'LICENSE-MIT', 'README.md', 'src/lib.rs']},
  wasm: {schema: 'lean4-prod/core-wasm-target/1',
    files: ['.cargo/config.toml', 'Cargo.lock', 'Cargo.toml', 'src/lib.rs']},
});
const manifestPath = 'generation-manifest.json';

function fileBytes(path) {
  const stat = lstatSync(path);
  assert.equal(realpathSync(path), path, 'unaliased generated package file');
  assert.ok(stat.isFile() && stat.nlink === 1, 'regular singly linked generated package file');
  return readFileSync(path);
}

function inventory(directory, kind, inputIrSha256) {
  assert.equal(resolve(directory), directory, 'absolute normalized generated package');
  assert.ok(Object.hasOwn(layouts, kind), 'closed generated package kind');
  assert.match(inputIrSha256, /^[a-f0-9]{64}$/, 'exact expected input IR');
  const layout = layouts[kind], before = fileBytes(join(directory, manifestPath));
  const manifest = JSON.parse(before.toString('utf8'));
  assert.equal(manifest.schema, layout.schema, 'exact generated package schema');
  assert.equal(manifest.input_ir_sha256, inputIrSha256, 'generated package input IR');
  assert.ok(Array.isArray(manifest.files));
  assert.deepEqual(manifest.files.map(row => row.path), layout.files, 'closed generated file declarations');
  const declared = new Map(manifest.files.map(row => {
    assert.deepEqual(Object.keys(row).sort(), ['path', 'sha256']);
    assert.match(row.sha256, /^[a-f0-9]{64}$/); return [row.path, row.sha256];
  }));
  const expectedFiles = [...layout.files, manifestPath].sort(), expectedDirectories = new Set(['']);
  for (const path of expectedFiles) {
    let parent = dirname(path);
    while (parent !== '.') {expectedDirectories.add(parent); parent = dirname(parent);}
  }
  const directories = [], files = {};
  function visit(relative) {
    const path = relative ? join(directory, relative) : directory;
    assert.equal(realpathSync(path), path, 'unaliased generated package directory');
    assert.ok(lstatSync(path).isDirectory(), 'actual generated package directory');
    directories.push(relative);
    for (const name of readdirSync(path).sort()) {
      const child = relative ? relative + '/' + name : name, absolute = join(directory, child);
      const stat = lstatSync(absolute);
      if (stat.isDirectory()) visit(child);
      else files[child] = digest(fileBytes(absolute));
    }
  }
  visit('');
  assert.deepEqual(Object.keys(files).sort(), expectedFiles, 'complete generated package file inventory');
  assert.deepEqual(directories.sort(), [...expectedDirectories].sort(), 'complete generated package directory inventory');
  for (const [path, sha256] of declared) assert.equal(files[path], sha256, 'generated package manifest digest ' + path);
  assert.deepEqual(fileBytes(join(directory, manifestPath)), before, 'stable generation manifest during capture');
  assert.equal(files[manifestPath], digest(before), 'captured generation manifest');
  return Object.freeze(Object.fromEntries(Object.entries(files).sort(([left], [right]) => left.localeCompare(right))));
}

export function captureGeneratedPackage(directory, {kind, inputIrSha256}) {
  const files = inventory(directory, kind, inputIrSha256);
  return Object.freeze({directory, kind, inputIrSha256, files,
    verify() {
      assert.deepEqual(inventory(directory, kind, inputIrSha256), files,
        'immutable generated package captured immediately after code generation');
    }});
}
