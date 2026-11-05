// Selector/custody recording controls only, never an installed SDK receipt.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {dirname} from 'node:path';
import test from 'node:test';
import {encodeInventory} from '../sdk/inventory-metadata.mjs';
import {selectSemanticSourceRuntime} from './semantic-source.mjs';

const marker = '/etc/profile.d/prismpm-sdk.sh', inventoryPath = '/opt/prismpm/share/inventory.json';
const helper = '/opt/prismpm/conformance/bin/prismpm-semantic-source';
const fixedRoot = '/opt/prismpm/share/conformance-root';
const operations = ['read', 'write', 'to-notation', 'runtime-audit'];
function fixture() {
  const rows = new Map(), hashes = new Map([[helper, 'a'.repeat(64)]]);
  let sequence = 0;
  function stat(file = false, mode = file ? 0o444n : 0o755n) {
    return {dev: 1n, ino: BigInt(++sequence), uid: 0n, gid: 0n, mode, size: 1n, nlink: 1n,
      mtimeNs: 0n, ctimeNs: 0n, isFile: () => file, isDirectory: () => !file};
  }
  for (const file of [helper, inventoryPath]) {
    rows.set(file, stat(true, file === helper ? 0o555n : 0o444n));
    for (let path = dirname(file); ; path = dirname(path)) {
      if (!rows.has(path)) rows.set(path, stat());
      if (path === '/') break;
    }
  }
  const inventory = {schema: 'prismpm/sdk-inventory/1', commands: [], artifacts: [
    {id: 'conformance-semantic-source', kind: 'binary', version: '0.4.0', digest: 'sha256:' + 'a'.repeat(64)},
  ]};
  let text = encodeInventory(inventory);
  const calls = [];
  const files = {stat(path) {calls.push(['stat', path]); return rows.has(path) ? {...rows.get(path)} : undefined;},
    realpath: path => path,
    read(path, maximum) {assert.equal(path, inventoryPath); assert.equal(maximum, 8 * 1024 * 1024); return text;},
    snapshot(path, maximum) {assert.equal(path, helper); assert.equal(maximum, 256 * 1024 * 1024); return {sha256: hashes.get(path)};}};
  return {rows, hashes, inventory, files, calls, marker: () => rows.set(marker, stat(true)),
    update: () => {text = encodeInventory(inventory);}, replaceText: value => {text = value;}};
}

test('source bootstrap retains explicit tools and source Cargo only outside every SDK context', () => {
  const absent = {stat: () => undefined};
  for (const operation of operations) {
    const source = selectSemanticSourceRuntime(operation, '/workspace/source', undefined, absent);
    assert.equal(source.program, 'cargo'); assert.equal(source.args.at(-1), operation);
    assert(source.args.includes('--offline') && source.args.includes('--locked'));
    const explicit = selectSemanticSourceRuntime(operation, '/workspace/source', '/source-built/helper', absent);
    assert.equal(explicit.program, '/source-built/helper'); assert.deepEqual(explicit.args, [operation]);
  }
  assert.throws(() => selectSemanticSourceRuntime('unregistered', '/workspace/source', undefined, absent));
});

test('fixed inventory or global SDK marker selects the same private path for all four operations', () => {
  for (const withMarker of [false, true]) for (const root of [fixedRoot, '/workspace/run/inputs/source']) {
    const f = fixture(); if (withMarker) f.marker();
    for (const operation of operations) {
      const selected = selectSemanticSourceRuntime(operation, root, undefined, f.files);
      assert.equal(selected.program, helper); assert.deepEqual(selected.args, [operation]); selected.verify();
    }
  }
  for (const root of [fixedRoot, fixedRoot + '/captured']) {
    assert.throws(() => selectSemanticSourceRuntime('read', root, undefined,
      {stat: () => undefined, realpath: path => path}), /custody/);
  }
  const captured = fixture();
  captured.rows.delete(inventoryPath);
  captured.rows.set(fixedRoot, {...captured.rows.get(dirname(fixedRoot))});
  assert.throws(() => selectSemanticSourceRuntime('read', '/workspace/run/inputs/source', undefined, captured.files), /custody/);
  assert.throws(() => selectSemanticSourceRuntime('read', '/workspace/run/inputs/source', '/external/helper', captured.files), /external/);
});

test('installed admission refuses overrides and missing, duplicated, changed or malformed inventory', () => {
  const f = fixture();
  assert.throws(() => selectSemanticSourceRuntime('read', fixedRoot, '/external/helper', f.files), /external/);
  for (const mutate of [
    f => f.rows.delete(helper), f => {f.marker(); f.rows.delete(inventoryPath);},
    f => {f.inventory.artifacts = []; f.update();},
    f => {f.inventory.artifacts.push(f.inventory.artifacts[0]); f.update();},
    f => {f.inventory.artifacts[0].digest = 'sha256:' + 'b'.repeat(64); f.update();},
    f => {f.inventory.artifacts[0].kind = 'schema'; f.update();},
    f => {f.inventory.artifacts[0].version = '0.3.0'; f.update();},
    f => f.replaceText('{not JSON'), f => f.replaceText(JSON.stringify(f.inventory)),
    f => {f.inventory.extra = true; f.update();},
  ]) {
    const changed = fixture(); mutate(changed);
    assert.throws(() => selectSemanticSourceRuntime('read', fixedRoot, undefined, changed.files));
  }
  const denied = {stat() {throw Object.assign(Error('stat denied'), {code: 'EACCES'});}};
  assert.throws(() => selectSemanticSourceRuntime('read', '/workspace/source', '/external/helper', denied), /stat denied/);
});

test('installed custody refuses aliases, writable or unowned ancestors and malformed helper files', () => {
  for (const mutate of [
    f => {f.files.realpath = path => path === helper ? '/aliased/helper' : path;},
    f => {f.rows.get(helper).uid = 1000n;}, f => {f.rows.get(helper).mode = 0o575n;},
    f => {f.rows.get(dirname(helper)).mode = 0o775n;}, f => {f.rows.get(dirname(helper)).uid = 1000n;},
    f => {f.rows.get(helper).nlink = 2n;}, f => {f.rows.get(helper).mode = 0o4555n;},
    f => {f.rows.get(helper).mode = 0o444n;}, f => {f.rows.get(inventoryPath).mode = 0o464n;},
    f => {f.rows.get(helper).size = 256n * 1024n * 1024n + 1n;},
  ]) {
    const f = fixture(); mutate(f);
    assert.throws(() => selectSemanticSourceRuntime('read', fixedRoot, undefined, f.files));
  }
});

test('post-dispatch custody rejects same-byte replacement and inventory, ancestor or executable drift', () => {
  for (const mutate of [
    f => {f.rows.get(helper).ino++;}, f => {f.rows.get(inventoryPath).ino++;},
    f => {f.rows.get(dirname(helper)).ino++;}, f => {f.rows.get(helper).mtimeNs++;},
    f => {f.hashes.set(helper, 'b'.repeat(64));}, f => f.replaceText(encodeInventory({...f.inventory, changed: true})),
    f => f.rows.delete(helper),
  ]) {
    const f = fixture(), selected = selectSemanticSourceRuntime('runtime-audit', fixedRoot, undefined, f.files);
    mutate(f); assert.throws(() => selected.verify());
  }
  for (const [operation, path] of [['read', inventoryPath], ['snapshot', helper]]) {
    const f = fixture(), selected = selectSemanticSourceRuntime('read', fixedRoot, undefined, f.files);
    const original = f.files[operation];
    f.files[operation] = (...args) => {
      const result = original(...args);
      f.rows.get(path).ino++; // Same bytes/digest, replaced during the fresh read.
      return result;
    };
    assert.throws(() => selected.verify(), /replaced/);
  }
});

test('normal SDK construction binds the private asset and smoke without publishing another command', () => {
  const recipe = readFileSync(new URL('../sdk/Dockerfile', import.meta.url), 'utf8');
  assert(recipe.includes('install -D -m 0555 target/release/prismpm-semantic-source ' + helper));
  assert(!recipe.includes('target/release/prismpm-semantic-source /usr/local/bin/'));
  const smoke = recipe.indexOf('scripts/semantic-source-runtime-check.mjs');
  assert(smoke > recipe.indexOf('node /opt/prismpm/generate-inventory.mjs /opt/prismpm/share/inventory.json'));
  const register = readFileSync(new URL('../sdk/generate-inventory.mjs', import.meta.url), 'utf8');
  assert(register.includes("['conformance-semantic-source', 'binary', '0.4.0', '" + helper + "']"));
});
