// Explicit maintenance only: validation never queries an unpinned OSV endpoint.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { createReadStream } from 'node:fs';
import { chmod, link, mkdir, open, readFile, statfs, unlink, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

export const ecosystems = ['crates.io', 'Debian', 'Go', 'npm', 'Ubuntu'];
export const maxAgeSeconds = 7 * 24 * 60 * 60;
const maximumBytes = 1024 ** 3;
const minimumFree = 12 * 1024 ** 3;
const bucket = 'osv-vulnerabilities';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const prefix = ecosystem => ({'crates.io': 'CRATES', Debian: 'DEBIAN', Go: 'GO', npm: 'NPM', Ubuntu: 'UBUNTU'})[ecosystem];

export function metadataUrl(ecosystem, generation) {
  assert.ok(ecosystems.includes(ecosystem), 'unsupported ecosystem');
  if (generation !== undefined) assert.match(generation, /^[1-9][0-9]{15}$/);
  return `https://storage.googleapis.com/storage/v1/b/${bucket}/o/${encodeURIComponent(`${ecosystem}/all.zip`)}${generation ? `?generation=${generation}` : ''}`;
}

export function sourceUrl(ecosystem, generation) {
  metadataUrl(ecosystem, generation);
  assert.ok(generation, 'generation is required');
  return `https://${bucket}.storage.googleapis.com/${ecosystem}/all.zip?generation=${generation}`;
}

function timestamp(value) {
  assert.match(value, /^20[0-9]{2}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]{3}Z$/);
  const parsed = Date.parse(value);
  assert.ok(Number.isFinite(parsed) && new Date(parsed).toISOString() === value, 'invalid timestamp');
  return Math.floor(parsed / 1000);
}

export function validateMetadata(value, ecosystem, now) {
  assert.equal(value.bucket, bucket);
  assert.equal(value.name, `${ecosystem}/all.zip`);
  sourceUrl(ecosystem, value.generation);
  assert.match(value.size, /^[1-9][0-9]*$/);
  const size = Number(value.size);
  assert.ok(Number.isSafeInteger(size) && size <= maximumBytes, 'OSV object exceeds acquisition limit');
  assert.match(value.md5Hash, /^[A-Za-z0-9+/]{22}==$/);
  const created = timestamp(value.timeCreated);
  assert.ok(created <= now && timestamp(value.updated) >= created && timestamp(value.updated) <= now,
    'invalid or future source time');
  assert.ok(now < created + maxAgeSeconds, 'source is already stale');
  return {ecosystem, generation: value.generation, size, md5: value.md5Hash,
    source_created: value.timeCreated, source_updated: value.updated, source_created_unix: created};
}

export function validateManifest(manifest) {
  assert.equal(manifest.schema, 'prismpm/osv-inputs/1');
  assert.equal(manifest.freshness_policy_seconds, maxAgeSeconds);
  assert.deepEqual(manifest.databases.map(row => row.ecosystem), ecosystems);
  for (const row of manifest.databases) {
    const acquired = timestamp(row.acquired_at);
    const checked = validateMetadata({bucket, name: `${row.ecosystem}/all.zip`, generation: row.generation,
      size: String(row.size), md5Hash: row.md5, timeCreated: row.source_created,
      updated: row.source_updated}, row.ecosystem, acquired);
    assert.equal(row.source_created_unix, checked.source_created_unix);
    assert.equal(row.id, `OSV-${prefix(row.ecosystem)}-DB-G${row.generation}`);
    assert.equal(row.url, sourceUrl(row.ecosystem, row.generation));
    assert.equal(row.metadata_url, metadataUrl(row.ecosystem, row.generation));
    assert.match(row.sha256, /^[0-9a-f]{64}$/);
  }
  return Math.min(...manifest.databases.map(row => row.source_created_unix)) + maxAgeSeconds;
}

export function updateCatalog(text, manifest) {
  validateManifest(manifest);
  for (const row of manifest.databases) {
    const canonical = `canonical_identifier = "gs://${bucket}/${row.ecosystem}/all.zip"`;
    const blocks = text.split('[[authority]]');
    const matches = blocks.map((block, index) => block.includes(canonical) ? index : -1).filter(index => index >= 0);
    assert.equal(matches.length, 1, 'each ecosystem must have one authority');
    let block = blocks[matches[0]];
    assert.ok(block.includes('redistribution = "citation-only"'));
    const fields = {id: row.id, edition: `gcs-generation-${row.generation}`, immutable_url: row.url,
      revision: row.generation, acquired_sha256: row.sha256, retrieval_date: row.acquired_at.slice(0, 10)};
    for (const [key, value] of Object.entries(fields)) {
      const pattern = new RegExp(`^${key} = "[^"\\n]*"$`, 'gm');
      assert.equal([...block.matchAll(pattern)].length, 1, `missing/duplicate ${key}`);
      block = block.replace(pattern, `${key} = "${value}"`);
    }
    blocks[matches[0]] = block;
    text = blocks.join('[[authority]]');
  }
  return text;
}

async function space(directory, reserved = 0) {
  const status = await statfs(directory);
  assert.ok(status.bavail * status.bsize >= minimumFree + reserved, 'maintain 12 GiB free disk reserve');
}

async function metadata(ecosystem, generation) {
  const response = await fetch(metadataUrl(ecosystem, generation),
    {redirect: 'error', signal: AbortSignal.timeout(30_000)});
  assert.equal(response.status, 200, 'official GCS metadata request failed');
  let text = '';
  for await (const chunk of response.body) {
    text += Buffer.from(chunk).toString('utf8');
    assert.ok(text.length <= 65536, 'metadata exceeds 64 KiB');
  }
  return JSON.parse(text);
}

async function digestFile(path) {
  const sha = createHash('sha256');
  let size = 0;
  for await (const chunk of createReadStream(path)) {
    size += chunk.length;
    assert.ok(size <= maximumBytes, 'cached object exceeds size limit');
    sha.update(chunk);
  }
  return {size, sha256: sha.digest('hex')};
}

export async function verifyBody(response, snapshot, write, checkSpace) {
  assert.equal(response.status, 200, 'pinned OSV acquisition failed');
  assert.equal(response.headers.get('x-goog-generation'), snapshot.generation);
  assert.equal(Number(response.headers.get('content-length')), snapshot.size);
  const sha = createHash('sha256'), md5 = createHash('md5');
  let size = 0, checkedSpace = 0;
  for await (const chunk of response.body) {
    size += chunk.length;
    assert.ok(size <= snapshot.size, 'OSV response exceeded metadata size');
    if (size - checkedSpace >= 16 * 1024 ** 2) { await checkSpace(); checkedSpace = size; }
    sha.update(chunk); md5.update(chunk);
    await write(chunk);
  }
  assert.equal(size, snapshot.size, 'OSV response truncated');
  assert.equal(md5.digest('base64'), snapshot.md5, 'GCS object checksum differs');
  return {size, sha256: sha.digest('hex')};
}

async function acquire() {
  const cache = join(root, '.prism/cache/authorities/sha256');
  await mkdir(cache, {recursive: true});
  const databases = [];
  for (const ecosystem of ecosystems) {
    const discovered = await metadata(ecosystem);
    const snapshot = validateMetadata(discovered, ecosystem, Math.floor(Date.now() / 1000));
    const pinned = validateMetadata(await metadata(ecosystem, snapshot.generation), ecosystem, Math.floor(Date.now() / 1000));
    assert.deepEqual(pinned, snapshot, 'metadata changed while pinning generation');
    await space(cache, snapshot.size);
    const temporary = join(cache, `.osv-${process.pid}-${prefix(ecosystem)}.partial`);
    const file = await open(temporary, 'wx', 0o600);
    try {
      const response = await fetch(sourceUrl(ecosystem, snapshot.generation),
        {redirect: 'error', signal: AbortSignal.timeout(600_000)});
      const {size, sha256} = await verifyBody(response, snapshot,
        chunk => file.writeFile(chunk), () => space(cache));
      await file.sync();
      await chmod(temporary, 0o444);
      const destination = join(cache, sha256);
      try { await link(temporary, destination); }
      catch (error) {
        if (error.code !== 'EEXIST') throw error;
        assert.deepEqual(await digestFile(destination), {size, sha256});
      }
      databases.push({...snapshot, id: `OSV-${prefix(ecosystem)}-DB-G${snapshot.generation}`, sha256,
        url: sourceUrl(ecosystem, snapshot.generation), metadata_url: metadataUrl(ecosystem, snapshot.generation),
        acquired_at: new Date().toISOString()});
      console.error(`acquired ${ecosystem}: ${snapshot.generation} ${sha256} (${size} bytes)`);
    } finally {
      await file.close();
      await unlink(temporary);
    }
  }
  const manifest = {schema: 'prismpm/osv-inputs/1', freshness_policy_seconds: maxAgeSeconds, databases};
  validateManifest(manifest);
  const proposal = join(root, '.prism/cache/osv-refresh.json');
  await writeFile(proposal, `${JSON.stringify(manifest, null, 2)}\n`, {flag: 'wx'});
  console.log(proposal);
}

async function apply() {
  const proposal = await readFile(join(root, '.prism/cache/osv-refresh.json'), 'utf8');
  const manifest = JSON.parse(proposal);
  const expiry = validateManifest(manifest);
  assert.ok(Math.floor(Date.now() / 1000) < expiry, 'proposal has expired');
  for (const row of manifest.databases) {
    assert.deepEqual(await digestFile(join(root, '.prism/cache/authorities/sha256', row.sha256)),
      {size: row.size, sha256: row.sha256});
  }
  const catalogPath = join(root, 'model/authorities.toml');
  const catalog = updateCatalog(await readFile(catalogPath, 'utf8'), manifest);
  await writeFile(join(root, 'model/osv-databases.json'), proposal);
  await writeFile(catalogPath, catalog);
  console.log(`Reviewed proposal applied; regenerate standards.lock with this source's prismpm authority resolve. Expires ${new Date(expiry * 1000).toISOString()}`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert.equal(process.argv.length, 3, 'usage: node scripts/refresh-osv.mjs acquire|apply');
  if (process.argv[2] === 'acquire') await acquire();
  else if (process.argv[2] === 'apply') await apply();
  else throw new Error('usage: node scripts/refresh-osv.mjs acquire|apply');
}
