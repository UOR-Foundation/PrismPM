import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import { createRequire } from 'node:module';
import { fixture as metadataFixture } from './metadata-test-fixture.mjs';
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
  const uncompressed = structuredClone(original);
  uncompressed.layers[0].mediaType = 'application/vnd.docker.image.rootfs.diff.tar';
  const expectedUncompressed = structuredClone(expected);
  expectedUncompressed.layers[0].mediaType = 'application/vnd.oci.image.layer.v1.tar';
  assert.deepEqual(JSON.parse(ociFixtureManifest(encode(uncompressed))), expectedUncompressed);
  assert.deepEqual(ociFixtureManifest(encode(expectedUncompressed)), encode(expectedUncompressed));
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

test('published migration schema admits complete evidence and rejects empty or wrong-major locks', async () => {
  const require = createRequire('/opt/prismpm/oracles/package.json');
  assert.equal(require('ajv/package.json').version, '8.20.0');
  const Ajv = require('ajv/dist/2020').default;
  const schema = JSON.parse(await readFile(new URL('../schemas/sdk-lock-migration.schema.json', import.meta.url)));
  const validate = new Ajv({strict: false, allErrors: true}).compile(schema);
  const directory = await mkdtemp(join(tmpdir(), 'prismpm-migration-schema-'));
  try {
    const {reference, inventories} = await fixture(directory);
    const target = await createPlatformLock(directory, reference, inventories.get('amd64'), standards, 'x64');
    const legacy = {schema: 'prismpm/sdk-lock/1', sdk_version: '0.3.0', sdk_image: reference,
      standards_lock: sha(standards), inventory: [{id: 'sdk-manifest', kind: 'image', version: '0.3.0', digest: reference.split('@')[1]}]};
    const proposal = {schema: 'prismpm/sdk-lock-migration/1', compatibility_review: 'required',
      generated_output_diff: 'required', security_review: 'required',
      patch: [{op: 'test', path: '', value: legacy}, {op: 'replace', path: '', value: target}]};
    assert.equal(validate(proposal), true, JSON.stringify(validate.errors));
    for (const mutation of ['empty-source', 'empty-target', 'wrong-source', 'wrong-target', 'extra-source', 'extra-target', 'missing-test', 'partial-path']) {
      const invalid = structuredClone(proposal);
      if (mutation === 'empty-source') invalid.patch[0].value = {};
      if (mutation === 'empty-target') invalid.patch[1].value = {};
      if (mutation === 'wrong-source') invalid.patch[0].value = target;
      if (mutation === 'wrong-target') invalid.patch[1].value = legacy;
      if (mutation === 'extra-source') invalid.patch[0].value.extra = true;
      if (mutation === 'extra-target') invalid.patch[1].value.extra = true;
      if (mutation === 'missing-test') invalid.patch.shift();
      if (mutation === 'partial-path') invalid.patch[1].path = '/sdk_image';
      assert.equal(validate(invalid), false, mutation);
    }
  } finally { await rm(directory, {recursive: true, force: true}); }
});

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

test('all platform-lock entry points reject coherently rehashed ambiguous OCI indexes', async () => {
  const directory = await mkdtemp(join(tmpdir(), 'prismpm-strict-index-'));
  try {
    const {index, inventories} = await fixture(directory);
    const pretty = Buffer.from(JSON.stringify(JSON.parse(index), null, 2) + '\n');
    assert.equal(parseSdkIndex(pretty, `example.invalid/test-sdk@${sha(pretty)}`).length, 2,
      'OCI index bytes are preserved, not required to use Prism canonical serialization');
    for (const spelling of ['2.0', '2e0']) {
      const bytes = Buffer.from(index.toString().replace('"schemaVersion":2', '"schemaVersion":' + spelling)
        .replace('"size":100', '"size":1e2'));
      assert.equal(parseSdkIndex(bytes, `example.invalid/test-sdk@${sha(bytes)}`).length, 2);
    }
    const nested = depth => Buffer.from('{"extension":' + '['.repeat(depth) + '0' + ']'.repeat(depth) + ',' + index.toString().slice(1));
    const allowedDepth = nested(63);
    assert.equal(parseSdkIndex(allowedDepth, `example.invalid/test-sdk@${sha(allowedDepth)}`).length, 2);
    const changes = [
      value => {value.SchemaVersion = 2;},
      value => {value.Manifeſts = [];},
      value => {value.manifests[0].MediaType = value.manifests[0].mediaType;},
      value => {value.manifests[0].urls = [];},
      value => {value.manifests[0].data = null;},
      value => {value.manifests[0].size = 1024 * 1024 + 1;},
      value => {value.manifests[0].platform.OS = 'linux';},
      value => {value.manifests[0].platform.variant = null;},
      value => {value.manifests[0].platform['os.version'] = null;},
      value => {value.manifests[1].platform['os.features'] = [];},
    ].map(mutate => {const value = JSON.parse(index); mutate(value); return encode(value);});
    changes.unshift(Buffer.from('{"schemaVersion":2,' + index.toString().slice(1)));
    changes.unshift(nested(64));
    for (const number of ['0', '-0', '-1', '0.5', '100.1', '1048577', '9007199254740993', '1e400'])
      changes.push(Buffer.from(index.toString().replace('"size":100', '"size":' + number)));
    changes.unshift(Buffer.from(index.toString().replace('"architecture":"amd64"',
      '"architecture":"amd64","architec\\u0074ure":"amd64"')));
    for (const bytes of changes) {
      const reference = `example.invalid/test-sdk@${sha(bytes)}`;
      assert.equal(parseSdkIndex(index, `example.invalid/test-sdk@${sha(index)}`).length, 2);
      assert.throws(() => parseSdkIndex(bytes, reference), bytes.toString());
      await writeFile(`${directory}/index.json`, bytes);
      await assert.rejects(createPlatformLock(directory, reference, inventories.get('amd64'), standards, 'x64'));
    }
  } finally { await rm(directory, {recursive:true, force:true}); }
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

test('public capture uses only the seven bounded OCI reads', async t => {
  const f = metadataFixture(t);
  const proposed = await capturePlatformLock(f.reference, sha(f.standards), f.transport);
  assert.equal(proposed.schema, 'prismpm/sdk-lock/2');
  assert.equal(proposed.sdk_index, f.index.toString());
  assert.equal(f.calls.length, 7);
  for (const row of proposed.platforms) assert.equal(row.inventory_document, f.inventories.get(row.platform.split('/')[1]).toString());
});

test('public capture refuses legacy and corrupted metadata without a full-image fallback', async t => {
  for (const mutation of ['legacy','standards','platform','diffid','identities']) {
    const f = metadataFixture(t, mutation);
    await assert.rejects(capturePlatformLock(f.reference, sha(f.standards), f.transport));
    assert(f.calls.length <= 7);
  }
});
