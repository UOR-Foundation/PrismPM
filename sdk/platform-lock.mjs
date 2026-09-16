// SDK-owned bootstrap transport validation. Inventory facts come from the
// actual digest-selected images; this does not grant release acceptance.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { lstat, mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

const sha = bytes => `sha256:${createHash('sha256').update(bytes).digest('hex')}`;
const digest = /^sha256:[0-9a-f]{64}$/;
const image = /^[a-z0-9.-]+(?::[0-9]{1,5})?\/[a-z0-9./_-]+@sha256:[0-9a-f]{64}$/;
const architectures = ['amd64', 'arm64'];
const kinds = ['adapter', 'base-image', 'binary', 'crate', 'dependency-lock', 'image',
  'oracle', 'schema', 'test-corpus', 'trust-root', 'workflow'];
const canonical = value => Array.isArray(value) ? value.map(canonical)
  : value && typeof value === 'object'
  ? Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])])) : value;

async function boundedFile(file, maximum = 8 * 1024 * 1024) {
  const metadata = await lstat(file);
  assert.ok(metadata.isFile() && !metadata.isSymbolicLink(), 'SDK evidence must be a regular file');
  assert.ok(metadata.size <= maximum, 'SDK evidence exceeds its byte bound');
  return readFile(file);
}

export function parseSdkIndex(bytes, reference) {
  assert.ok(bytes.length <= 1024 * 1024, 'SDK index exceeds its byte bound');
  assert.match(reference, image);
  const port = reference.split('/', 1)[0].split(':', 2)[1];
  assert.ok(port === undefined || (Number(port) >= 1 && Number(port) <= 65535));
  assert.equal(sha(bytes), reference.split('@')[1], 'SDK index bytes disagree with pinned digest');
  const index = JSON.parse(bytes);
  assert.equal(index.schemaVersion, 2);
  assert.equal(index.mediaType, 'application/vnd.oci.image.index.v1+json');
  assert.equal(index.manifests.length, 2, 'SDK index must have exactly the two supported native platforms');
  const children = architectures.map(architecture => {
    const matching = index.manifests.filter(row => row.platform?.os === 'linux'
      && row.platform.architecture === architecture);
    assert.equal(matching.length, 1, `SDK index must contain exactly one linux/${architecture}`);
    const descriptor = matching[0];
    assert.equal(descriptor.mediaType, 'application/vnd.oci.image.manifest.v1+json');
    assert.ok(Number.isSafeInteger(descriptor.size) && descriptor.size > 0);
    assert.match(descriptor.digest, digest);
    assert.ok(descriptor.platform.variant === undefined
      || (architecture === 'arm64' && descriptor.platform.variant === 'v8'));
    return {architecture, reference: `${reference.split('@')[0]}@${descriptor.digest}`, descriptor};
  });
  assert.notEqual(children[0].descriptor.digest, children[1].descriptor.digest);
  return children;
}

export function validateInventory(bytes) {
  assert.ok(bytes.length <= 8 * 1024 * 1024, 'SDK inventory exceeds its byte bound');
  const inventory = JSON.parse(bytes);
  const encoded = JSON.stringify(canonical(inventory));
  assert.ok(bytes.toString() === encoded || bytes.toString() === `${encoded}\n`, 'SDK inventory is not canonical');
  assert.deepEqual(Object.keys(inventory).sort(), ['artifacts', 'commands', 'schema']);
  assert.equal(inventory.schema, 'prismpm/sdk-inventory/1');
  assert.ok(Array.isArray(inventory.commands) && inventory.commands.length > 0);
  assert.ok(Array.isArray(inventory.artifacts) && inventory.artifacts.length > 0);
  let previousCommand = '';
  for (const row of inventory.commands) {
    assert.deepEqual(Object.keys(row).sort(), ['command', 'executable', 'sha256']);
    assert.equal(typeof row.command, 'string');
    assert.ok(row.command.length > 0);
    assert.ok(Buffer.from(previousCommand).compare(Buffer.from(row.command)) < 0, 'commands must be unique and sorted');
    assert.equal(typeof row.executable, 'string');
    assert.ok(row.executable.startsWith('/'));
    assert.match(row.sha256, /^[0-9a-f]{64}$/);
    previousCommand = row.command;
  }
  let previous = '';
  for (const row of inventory.artifacts) {
    assert.deepEqual(Object.keys(row).sort(), ['digest', 'id', 'kind', 'version']);
    assert.match(row.id, /^[^\n]{1,128}$/);
    assert.match(row.version, /^[^\n]{1,128}$/);
    assert.match(row.digest, digest);
    assert.ok(kinds.includes(row.kind));
    assert.ok(Buffer.from(previous).compare(Buffer.from(row.id)) < 0, 'inventory IDs must be unique and sorted');
    assert.notEqual(row.id, 'sdk-manifest');
    previous = row.id;
  }
  // The SDK index is the enclosing image artifact; other classes must be real
  // image-local inventory entries, not rows synthesized by the consumer.
  for (const kind of kinds.filter(kind => kind !== 'image')) {
    assert.ok(inventory.artifacts.some(row => row.kind === kind), `SDK lacks ${kind} artifacts`);
  }
  for (const command of ['cargo', 'devcontainer', 'docker', 'just', 'prismpm']) {
    assert.ok(inventory.commands.some(row => row.command === command), `SDK lacks ${command}`);
  }
  return inventory;
}

// Shared target-image validation. Unlike bootstrap, an update deliberately
// selects a different SDK and must not compare it to the currently running one.
export async function validatePlatformBundle(directory, reference, standardsBytes) {
  const indexBytes = await boundedFile(`${directory}/index.json`, 1024 * 1024);
  const children = parseSdkIndex(indexBytes, reference);
  const platforms = [];
  let expectedIdentities;
  for (const child of children) {
    const path = `${directory}/${child.architecture}`;
    const inspected = JSON.parse(await boundedFile(`${path}/image.json`));
    assert.equal(inspected.Os, 'linux');
    assert.equal(inspected.Architecture, child.architecture);
    assert.ok(inspected.RepoDigests.includes(child.reference), 'copied inventory image is not the exact indexed child');
    const bytes = await boundedFile(`${path}/inventory.json`);
    const inventory = validateInventory(bytes);
    assert.deepEqual(await boundedFile(`${path}/standards.lock`, 16 * 1024 * 1024), standardsBytes, 'SDK platform standards locks differ');
    const identities = inventory.artifacts.map(row => [row.id, row.kind, row.version]);
    if (expectedIdentities) assert.deepEqual(identities, expectedIdentities, 'SDK platform artifact identities differ');
    expectedIdentities = identities;
    platforms.push({platform: `linux/${child.architecture}`, manifest_digest: child.descriptor.digest,
      inventory_digest: sha(bytes), inventory_document: bytes.toString('utf8'), inventory: inventory.artifacts});
  }
  return {schema: 'prismpm/sdk-lock/2', sdk_image: reference, sdk_index: indexBytes.toString('utf8'),
    sdk_version: '0.3.0', standards_lock: sha(standardsBytes), platforms};
}

export async function createPlatformLock(directory, reference, nativeInventoryBytes, standardsBytes, architecture) {
  const nativeArchitecture = {x64: 'amd64', arm64: 'arm64'}[architecture];
  assert.ok(nativeArchitecture, 'unsupported bootstrap host architecture');
  const lock = await validatePlatformBundle(directory, reference, standardsBytes);
  assert.deepEqual(await boundedFile(`${directory}/${nativeArchitecture}/inventory.json`), nativeInventoryBytes,
    'bootstrap runs a different native SDK inventory');
  return lock;
}

// The injected runner is solely a test seam; the CLI always uses the verified
// Docker executable supplied by PrismPM. Never run either target image.
export async function capturePlatformLock(reference, standardsDigest, run) {
  assert.match(reference, image);
  assert.match(standardsDigest, digest);
  const directory = await mkdtemp(join(tmpdir(), 'prismpm-sdk-update-'));
  try {
    const index = run(['buildx', 'imagetools', 'inspect', '--raw', reference]);
    const children = parseSdkIndex(index, reference);
    await writeFile(`${directory}/index.json`, index);
    for (const child of children) {
      const childDirectory = `${directory}/${child.architecture}`;
      await mkdir(childDirectory);
      run(['pull', '--platform', `linux/${child.architecture}`, child.reference]);
      const inspected = run(['image', 'inspect', '--format', '{{json .}}', child.reference]);
      await writeFile(`${childDirectory}/image.json`, inspected);
      const container = run(['create', '--network', 'none', '--platform', `linux/${child.architecture}`, child.reference]).toString().trim();
      assert.match(container, /^[0-9a-f]{64}$/, 'Docker did not return one exact created container ID');
      try {
        run(['cp', `${container}:/opt/prismpm/share/inventory.json`, `${childDirectory}/inventory.json`]);
        run(['cp', `${container}:/opt/prismpm/share/standards.lock`, `${childDirectory}/standards.lock`]);
      } finally {
        run(['rm', '--volumes', container]);
      }
    }
    const standards = await boundedFile(`${directory}/amd64/standards.lock`, 16 * 1024 * 1024);
    assert.equal(sha(standards), standardsDigest, 'target SDK standards digest differs from requested update');
    return await validatePlatformBundle(directory, reference, standards);
  } finally {
    await rm(directory, {recursive: true, force: true});
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [command, reference, standardsDigest, docker] = process.argv.slice(2);
  if (command === 'index') {
    assert.equal(process.argv.length, 4);
    for (const child of parseSdkIndex(await readFile('/dev/stdin'), reference)) {
      process.stdout.write(`${child.architecture} ${child.reference}\n`);
    }
  } else {
    assert.equal(command, 'capture');
    assert.equal(process.argv.length, 6);
    assert.ok(docker.startsWith('/'), 'Docker must be an SDK-resolved absolute executable');
    const lock = await capturePlatformLock(reference, standardsDigest, args => execFileSync(docker, args,
      {timeout: 120_000, maxBuffer: 8 * 1024 * 1024, stdio: ['ignore', 'pipe', 'pipe']}));
    // SDK lock files are exact canonical JSON; unlike inventory files, they
    // do not permit a trailing newline outside the canonical value.
    process.stdout.write(JSON.stringify(canonical(lock)));
  }
}
