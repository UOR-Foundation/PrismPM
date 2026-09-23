import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import { copyFileSync, existsSync, readFileSync, symlinkSync } from 'node:fs';
import { capturePlatformLock, createPlatformLock, parseSdkIndex, validateInventory } from './platform-lock.mjs';
import { ociFixtureManifest } from './oci-test-fixture.mjs';

const sha = bytes => `sha256:${createHash('sha256').update(bytes).digest('hex')}`;
const canonical = value => Array.isArray(value) ? value.map(canonical) : value && typeof value === 'object'
  ? Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])])) : value;
const encode = value => Buffer.from(JSON.stringify(canonical(value)));
const standards = Buffer.from('synthetic test standards, not a published lock');

test('test-image conversion changes only the supported descriptor labels and retains all blob identities', () => {
  const original = {
    schemaVersion: 2, mediaType: 'application/vnd.docker.distribution.manifest.v2+json',
    config: {mediaType: 'application/vnd.docker.container.image.v1+json', digest: sha('actual config'), size: 13},
    layers: [{mediaType: 'application/vnd.docker.image.rootfs.diff.tar.gzip', digest: sha('actual layer'), size: 12,
      annotations: {'test.fixture/retained': 'layer metadata'}}],
    annotations: {'test.fixture/retained': 'manifest metadata'},
  };
  const expected = structuredClone(original);
  expected.mediaType = 'application/vnd.oci.image.manifest.v1+json';
  expected.config.mediaType = 'application/vnd.oci.image.config.v1+json';
  expected.layers[0].mediaType = 'application/vnd.oci.image.layer.v1.tar+gzip';
  const bytes = ociFixtureManifest(encode(original));
  assert.deepEqual(JSON.parse(bytes), expected);
  const exactOci = Buffer.from(`${JSON.stringify(expected, null, 2)}\n`);
  assert.deepEqual(ociFixtureManifest(exactOci), exactOci);
  for (const mutation of ['schema', 'manifest-list', 'config', 'layer', 'zstd', 'empty-layers', 'digest', 'size']) {
    const changed = structuredClone(original);
    if (mutation === 'schema') changed.schemaVersion = 1;
    if (mutation === 'manifest-list') changed.mediaType = 'application/vnd.docker.distribution.manifest.list.v2+json';
    if (mutation === 'config') changed.config.mediaType = 'application/octet-stream';
    if (mutation === 'layer') changed.layers[0].mediaType = 'application/octet-stream';
    if (mutation === 'zstd') changed.layers[0].mediaType = 'application/vnd.oci.image.layer.v1.tar+zstd';
    if (mutation === 'empty-layers') changed.layers = [];
    if (mutation === 'digest') changed.config.digest = 'sha256:invalid';
    if (mutation === 'size') changed.layers[0].size = -1;
    assert.throws(() => ociFixtureManifest(encode(changed)), mutation);
  }
  assert.throws(() => ociFixtureManifest(Buffer.alloc(1024 * 1024 + 1)), /byte bound/);
});

test('SDK capture continues to reject Docker manifest lists and Docker child descriptors', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'prismpm-platform-lock-'));
  try {
    const {index} = await fixture(directory);
    for (const mutation of ['index', 'child']) {
      const changed = JSON.parse(index);
      if (mutation === 'index') changed.mediaType = 'application/vnd.docker.distribution.manifest.list.v2+json';
      else changed.manifests[0].mediaType = 'application/vnd.docker.distribution.manifest.v2+json';
      const bytes = encode(changed);
      assert.throws(() => parseSdkIndex(bytes, `example.invalid/test-sdk@${sha(bytes)}`));
    }
  } finally { await rm(directory, {recursive: true, force: true}); }
});

async function fixture(directory) {
  const inventories = new Map();
  const manifests = [];
  for (const architecture of ['amd64', 'arm64']) {
    const artifacts = ['adapter', 'base-image', 'binary', 'crate', 'dependency-lock',
      'oracle', 'schema', 'test-corpus', 'trust-root', 'workflow'].map(kind => ({
      id: `test-${kind}`, kind, version: 'test-fixture', digest: sha(`${architecture}/${kind}`),
    }));
    const bytes = encode({schema: 'prismpm/sdk-inventory/1', artifacts,
      commands: ['cargo', 'devcontainer', 'docker', 'just', 'prismpm'].map(command => ({
        command, executable: `/usr/local/bin/${command}`, sha256: sha(`${architecture}/${command}`).slice(7),
      }))});
    inventories.set(architecture, bytes);
    const digest = sha(`synthetic manifest ${architecture}`);
    manifests.push({digest, size: 100, mediaType: 'application/vnd.oci.image.manifest.v1+json',
      platform: {os: 'linux', architecture}});
    await mkdir(`${directory}/${architecture}`);
    await writeFile(`${directory}/${architecture}/inventory.json`, bytes);
    await writeFile(`${directory}/${architecture}/standards.lock`, standards);
    await writeFile(`${directory}/${architecture}/image.json`, encode({Os: 'linux', Architecture: architecture,
      RepoDigests: [`example.invalid/test-sdk@${digest}`]}));
  }
  const index = encode({schemaVersion: 2, mediaType: 'application/vnd.oci.image.index.v1+json', manifests});
  await writeFile(`${directory}/index.json`, index);
  return {reference: `example.invalid/test-sdk@${sha(index)}`, inventories, index};
}

test('one exact index yields the same complete platform lock from either native architecture', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'prismpm-platform-lock-'));
  try {
    const {reference, inventories, index} = await fixture(directory);
    const amd64 = await createPlatformLock(directory, reference, inventories.get('amd64'), standards, 'x64');
    const arm64 = await createPlatformLock(directory, reference, inventories.get('arm64'), standards, 'arm64');
    assert.deepEqual(amd64, arm64);
    assert.equal(amd64.schema, 'prismpm/sdk-lock/2');
    assert.equal(amd64.sdk_index, index.toString());
    assert.notEqual(amd64.platforms[0].inventory_digest, amd64.platforms[1].inventory_digest);
    for (const row of amd64.platforms) {
      const actual = inventories.get(row.platform.split('/')[1]);
      assert.equal(row.inventory_document, actual.toString('utf8'));
      assert.equal(row.inventory_digest, sha(Buffer.from(row.inventory_document)));
      assert.deepEqual(JSON.parse(row.inventory_document).artifacts, row.inventory);
    }
    assert.deepEqual(amd64.platforms.map(row => row.platform), ['linux/amd64', 'linux/arm64']);
    assert.throws(() => parseSdkIndex(Buffer.concat([index, Buffer.from('\n')]), reference));
    const changed = JSON.parse(index); changed.manifests.pop();
    const changedBytes = encode(changed);
    assert.throws(() => parseSdkIndex(changedBytes, `example.invalid/test-sdk@${sha(changedBytes)}`));
    const oversizedIndex = Buffer.concat([index, Buffer.alloc(1024 * 1024, ' ')]);
    assert.throws(() => parseSdkIndex(oversizedIndex, `example.invalid/test-sdk@${sha(oversizedIndex)}`), /byte bound/);
  } finally { await rm(directory, {recursive: true, force: true}); }
});

test('inventory documents have closed canonical metadata and preserve exact final newlines', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'prismpm-inventory-document-'));
  try {
    const {inventories} = await fixture(directory);
    const bytes = inventories.get('amd64');
    assert.deepEqual(validateInventory(Buffer.concat([bytes, Buffer.from('\n')])), JSON.parse(bytes));
    assert.throws(() => validateInventory(Buffer.concat([bytes, Buffer.alloc(8 * 1024 * 1024, ' ')])), /byte bound/);
    for (const mutation of ['extra', 'command-extra', 'command-digest', 'command-relative', 'command-duplicate', 'duplicate-json-key']) {
      const document = JSON.parse(bytes);
      if (mutation === 'extra') document.unexpected = true;
      if (mutation === 'command-extra') document.commands[0].unexpected = true;
      if (mutation === 'command-digest') document.commands[0].sha256 = 'bad digest';
      if (mutation === 'command-relative') document.commands[0].executable = 'cargo';
      if (mutation === 'command-duplicate') document.commands.push(document.commands[0]);
      const changed = mutation === 'duplicate-json-key' ? Buffer.from(bytes.toString().replace('{', '{"schema":"duplicated",')) : encode(document);
      assert.throws(() => validateInventory(changed), mutation);
    }
  } finally { await rm(directory, {recursive: true, force: true}); }
});

test('wrong architecture, swapped inventories, missing files, changed standards and child substitution fail closed', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'prismpm-platform-lock-'));
  try {
    const {reference, inventories} = await fixture(directory);
    const create = () => createPlatformLock(directory, reference, inventories.get('amd64'), standards, 'x64');
    const inspectPath = `${directory}/amd64/image.json`;
    const inspected = await readFile(inspectPath);
    const changed = JSON.parse(inspected); changed.Architecture = 'arm64';
    await writeFile(inspectPath, encode(changed)); await assert.rejects(create);
    changed.Architecture = 'amd64'; changed.RepoDigests = [`example.invalid/test-sdk@${sha('another child')}`];
    await writeFile(inspectPath, encode(changed)); await assert.rejects(create);
    await writeFile(inspectPath, inspected);
    await writeFile(`${directory}/amd64/inventory.json`, inventories.get('arm64')); await assert.rejects(create);
    await writeFile(`${directory}/amd64/inventory.json`, inventories.get('amd64'));
    await writeFile(`${directory}/arm64/standards.lock`, 'changed'); await assert.rejects(create);
    await writeFile(`${directory}/arm64/standards.lock`, standards);
    await rm(`${directory}/arm64/inventory.json`); await assert.rejects(create);
  } finally { await rm(directory, {recursive: true, force: true}); }
});

// Synthetic Docker-port fixtures exercise capture commands and cleanup; they
// are deliberately not represented as real released SDK inventory evidence.
function captureRunner(directory, reference, index, mutation = '') {
  const calls = [], destinations = [];
  const children = parseSdkIndex(index, reference);
  const containers = new Map(children.map((child, i) => [(i ? 'b' : 'a').repeat(64), child.architecture]));
  const run = args => {
    calls.push(args);
    if (args[0] === 'buildx') {
      assert.deepEqual(args, ['buildx', 'imagetools', 'inspect', '--raw', reference]);
      return mutation === 'index' ? Buffer.concat([index, Buffer.from('\n')]) : index;
    }
    if (args[0] === 'pull') {
      assert.ok(children.some(child => child.reference === args[3] && args[2] === `linux/${child.architecture}`));
      if (mutation === 'pull') throw new Error('synthetic pull failure');
      return Buffer.alloc(0);
    }
    if (args[0] === 'image') {
      const child = children.find(child => child.reference === args[4]);
      assert.ok(child);
      const inspected = JSON.parse(readFileSync(`${directory}/${child.architecture}/image.json`));
      if (mutation === 'architecture') inspected.Architecture = 'riscv64';
      if (mutation === 'digest') inspected.RepoDigests = [`example.invalid/test-sdk@${sha('wrong')}`];
      return encode(inspected);
    }
    if (args[0] === 'create') {
      const child = children.find(child => child.reference === args[5]);
      assert.deepEqual(args.slice(0, 5), ['create', '--network', 'none', '--platform', `linux/${child.architecture}`]);
      return Buffer.from([...containers].find(([, architecture]) => architecture === child.architecture)[0]);
    }
    if (args[0] === 'cp') {
      const [container, source] = args[1].split(':');
      assert.ok(containers.has(container));
      assert.ok(['/opt/prismpm/share/inventory.json', '/opt/prismpm/share/standards.lock'].includes(source));
      if (mutation === 'copy') throw new Error('synthetic copy failure');
      destinations.push(args[2]);
      const input = `${directory}/${containers.get(container)}/${source.split('/').at(-1)}`;
      if (mutation === 'symlink') symlinkSync(input, args[2]);
      else copyFileSync(input, args[2]);
      return Buffer.alloc(0);
    }
    assert.deepEqual(args.slice(0, 2), ['rm', '--volumes']);
    assert.ok(containers.has(args[2]));
    return Buffer.alloc(0);
  };
  return {run, calls, destinations};
}

test('update captures both exact images without running foreign code and cleans its temporary inputs', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'prismpm-platform-update-test-'));
  try {
    const {reference, inventories, index} = await fixture(directory);
    for (const asynchronous of [false, true]) {
      const capture = captureRunner(directory, reference, index);
      const run = asynchronous ? async args => { await new Promise(resolve => setImmediate(resolve)); return capture.run(args); } : capture.run;
      const proposed = await capturePlatformLock(reference, sha(standards), run);
      assert.deepEqual(proposed, await createPlatformLock(directory, reference, inventories.get('amd64'), standards, 'x64'));
      assert.equal(capture.calls.length, 13);
      assert.equal(capture.calls.filter(args => args[0] === 'rm').length, 2);
      assert.ok(capture.calls.every(args => args[0] !== 'run' && args[0] !== 'start' && args[0] !== 'exec'));
      assert.ok(capture.destinations.every(file => !existsSync(file)));
    }
  } finally { await rm(directory, {recursive: true, force: true}); }
});

test('update rejects index/digest/architecture/copy/standards/symlink failures and removes only its own containers', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'prismpm-platform-update-test-'));
  try {
    const {reference, index} = await fixture(directory);
    for (const asynchronous of [false, true]) for (const mutation of ['index', 'pull', 'architecture', 'digest', 'copy', 'symlink', 'standards']) {
      const capture = captureRunner(directory, reference, index, mutation);
      const run = asynchronous ? async args => { await new Promise(resolve => setImmediate(resolve)); return capture.run(args); } : capture.run;
      await assert.rejects(capturePlatformLock(reference, mutation === 'standards' ? sha('wrong standards') : sha(standards), run));
      const creates = capture.calls.filter(args => args[0] === 'create').length;
      assert.equal(capture.calls.filter(args => args[0] === 'rm').length, creates, mutation);
      assert.ok(capture.destinations.every(file => !existsSync(file)), mutation);
    }
  } finally { await rm(directory, {recursive: true, force: true}); }
});
