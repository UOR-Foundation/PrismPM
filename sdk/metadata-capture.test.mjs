import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdtempSync, mkdirSync, writeFileSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import test from 'node:test';
import {captureMetadataLock} from './metadata-capture.mjs';
import {label, limits, paths, profile, sha} from './metadata-layer.mjs';

const canonical = value => Array.isArray(value) ? value.map(canonical) : value && typeof value === 'object'
  ? Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])])) : value;
const encode = value => Buffer.from(JSON.stringify(canonical(value)) + '\n');
const media = 'application/vnd.oci.image.';

// All fixture inventories are synthetic. No mocked transport response counts
// as actual SDK image materialization or acceptance evidence.
function fixture(t, mutation = '') {
  const work = mkdtempSync(join(tmpdir(), 'prismpm-metadata-capture-'));
  t.after(() => rmSync(work, {recursive:true, force:true}));
  const blobs = new Map(), manifests = [], inventories = new Map(), lowerDigests = [];
  const put = (bytes, mediaType) => {
    const digest = sha(bytes); blobs.set(digest, bytes); return {digest, size:bytes.length, mediaType};
  };
  const standards = Buffer.from('synthetic test standards\n');
  for (const architecture of ['amd64', 'arm64']) {
    const root = join(work, architecture); mkdirSync(root);
    for (const dir of ['opt', 'opt/prismpm', 'opt/prismpm/share']) mkdirSync(join(root, dir), {mode:0o755});
    const artifacts = ['adapter','base-image','binary','crate','dependency-lock','oracle','schema','test-corpus','trust-root','workflow']
      .map(kind => ({id:'test-' + kind, kind, version:'test', digest:sha(architecture + kind)}));
    if (mutation === 'identities' && architecture === 'arm64') artifacts[0].version = 'different';
    const inventory = encode({schema:'prismpm/sdk-inventory/1', artifacts,
      commands:['cargo','devcontainer','docker','just','prismpm'].map(command =>
        ({command, executable:'/usr/local/bin/' + command, sha256:sha(architecture + command).slice(7)}))});
    inventories.set(architecture, inventory);
    writeFileSync(join(root, paths.inventory), inventory, {mode:0o444});
    writeFileSync(join(root, paths.standards), mutation === 'standards' && architecture === 'arm64' ? 'different' : standards, {mode:0o444});
    const result = spawnSync('/usr/bin/tar', ['--format=ustar','--owner=0','--group=0','--mtime=@0','--no-recursion','-cf','-',
      'opt/','opt/prismpm/','opt/prismpm/share/',paths.inventory,paths.standards], {cwd:root, timeout:10000, maxBuffer:1024 * 1024});
    assert.ifError(result.error); assert.equal(result.status, 0);
    const lower = put(Buffer.from('never fetched lower layer ' + architecture), media + 'layer.v1.tar');
    lowerDigests.push(lower.digest);
    const terminal = put(result.stdout, media + 'layer.v1.tar');
    const config = {architecture, os:'linux', config:{Labels:{[label]:profile}}, rootfs:{type:'layers', diff_ids:[lower.digest, terminal.digest]}};
    if (mutation === 'legacy') delete config.config.Labels[label];
    if (mutation === 'platform' && architecture === 'arm64') config.architecture = 'amd64';
    if (mutation === 'diffid') config.rootfs.diff_ids[1] = sha('other');
    const manifest = put(encode({schemaVersion:2, mediaType:media + 'manifest.v1+json',
      config:put(encode(config), media + 'config.v1+json'), layers:[lower, terminal]}), media + 'manifest.v1+json');
    manifests.push({...manifest, platform:{os:'linux', architecture}});
  }
  const index = encode({schemaVersion:2, mediaType:media + 'index.v1+json', manifests});
  const reference = 'example.invalid/test-sdk@' + put(index, media + 'index.v1+json').digest;
  const calls = [];
  const transport = async request => {
    calls.push(request);
    assert(['manifest','blob'].includes(request.kind));
    assert(request.timeout_ms > 0 && request.timeout_ms <= 45000);
    assert(request.maximum > 0 && request.maximum <= limits.compressed);
    const bytes = blobs.get(request.reference.split('@')[1]); assert(bytes, 'only exact selected graph blobs');
    assert(!lowerDigests.includes(request.reference.split('@')[1]), 'ordinary filesystem layers must never be fetched');
    return bytes;
  };
  return {reference, standards, inventories, index, calls, transport};
}

test('bounded capture returns the ordinary complete lock from exact raw graph bytes without image pulls', async t => {
  const f = fixture(t);
  const lock = await captureMetadataLock(f.reference, sha(f.standards), f.transport);
  assert.equal(lock.schema, 'prismpm/sdk-lock/2'); assert.equal(lock.sdk_index, f.index.toString());
  assert.equal(lock.sdk_image, f.reference); assert.equal(lock.standards_lock, sha(f.standards));
  assert.equal(f.calls.length, 7); assert.deepEqual(f.calls.map(call => call.kind), ['manifest','manifest','blob','blob','manifest','blob','blob']);
  assert.deepEqual(lock.platforms.map(row => row.platform), ['linux/amd64','linux/arm64']);
  for (const row of lock.platforms) {
    const inventory = f.inventories.get(row.platform.split('/')[1]);
    assert.equal(row.inventory_document, inventory.toString()); assert.equal(row.inventory_digest, sha(inventory));
    assert.deepEqual(row.inventory, JSON.parse(inventory).artifacts);
  }
});

test('coherent image graphs cannot bypass standards, cross-platform identities, profile or DiffID checks', async t => {
  for (const mutation of ['identities','standards','legacy','platform','diffid']) {
    const f = fixture(t, mutation);
    await assert.rejects(captureMetadataLock(f.reference, sha(f.standards), f.transport), undefined, mutation);
    assert(f.calls.length <= 7, 'rejected capture must not retry or fall back to image pull');
  }
  const f = fixture(t);
  await assert.rejects(captureMetadataLock(f.reference, sha('wrong request'), f.transport), /standards digest/);
  assert.equal(f.calls.length, 4, 'first platform mismatch prevents further acquisition');
});

test('transport overflow and exact-byte mismatch refuse capture without subsequent requests', async t => {
  for (const fault of ['overflow','string','changed','failure']) {
    const f = fixture(t); let calls = 0;
    await assert.rejects(captureMetadataLock(f.reference, sha(f.standards), async request => {
      calls++;
      if (fault === 'overflow') return Buffer.alloc(request.maximum + 1);
      if (fault === 'string') return '{}';
      if (fault === 'failure') throw Error('synthetic acquisition failure');
      return Buffer.concat([await f.transport(request), Buffer.from(' ')]);
    }));
    assert.equal(calls, 1);
  }
});

test('coherently rehashed index mutations refuse duplicate platforms, extra children, aliases and blob indirection', async t => {
  for (const mutate of [
    value => {value.manifests[1].platform.architecture = 'amd64';},
    value => {value.manifests.push(structuredClone(value.manifests[0]));},
    value => {value.manifests[0].MediaType = 'text/plain';},
    value => {value.manifests[0].platform.OS = 'windows';},
    value => {value.manifests[0].urls = ['https://example.invalid/other'];},
    value => {value.manifests[0].data = '';},
    value => {value.Manifeſts = [];},
  ]) {
    const f = fixture(t), index = JSON.parse(f.index); mutate(index); const bytes = encode(index);
    const reference = f.reference.split('@')[0] + '@' + sha(bytes);
    await assert.rejects(captureMetadataLock(reference, sha(f.standards), async selected =>
      selected.reference === reference ? bytes : f.transport(selected)));
  }
  const f = fixture(t), bytes = Buffer.from('{"manifests":[],' + f.index.toString().slice(1));
  await assert.rejects(captureMetadataLock(f.reference.split('@')[0] + '@' + sha(bytes), sha(f.standards), async () => bytes), /duplicate/);
});

test('invalid repository paths and ports are refused before any acquisition', async () => {
  for (const repository of ['example.invalid/a/../b','example.invalid/a//b','example.invalid:0/a','example.invalid:65536/a']) {
    let fetched = false;
    await assert.rejects(captureMetadataLock(repository + '@' + sha('test'), sha('standards'), async () => {fetched = true;}));
    assert.equal(fetched, false);
  }
});
