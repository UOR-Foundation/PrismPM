import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { ecosystems, maxAgeSeconds, metadataUrl, sourceUrl, updateCatalog,
  validateManifest, validateMetadata, verifyBody } from './refresh-osv.mjs';

const metadata = () => ({bucket: 'osv-vulnerabilities', name: 'crates.io/all.zip',
  generation: '1789723713081168', size: '3', md5Hash: 'kAFQmDzST7DWlj99KOF/cg==',
  timeCreated: '2026-09-18T09:28:33.213Z', updated: '2026-09-18T09:28:33.213Z'});
const now = Date.parse('2026-09-19T00:00:00.000Z') / 1000;
const manifest = () => JSON.parse(readFileSync(new URL('../model/osv-databases.json', import.meta.url)));

test('metadata accepts only known bucket/object/generation and bounded fresh source facts', () => {
  assert.equal(validateMetadata(metadata(), 'crates.io', now).source_created_unix, 1789723713);
  for (const patch of [{bucket: 'attacker'}, {name: '../all.zip'}, {generation: '../file'},
    {size: '1073741825'}, {size: '0'}, {size: '3x'}, {md5Hash: 'none'},
    {timeCreated: '2026-09-20T00:00:00.000Z'}, {updated: '2026-09-20T00:00:00.000Z'},
    {timeCreated: '2026-09-10T00:00:00.000Z'}, {timeCreated: '2026-02-30T00:00:00.000Z'}]) {
    assert.throws(() => validateMetadata({...metadata(), ...patch}, 'crates.io', now));
  }
  assert.throws(() => metadataUrl('../Ubuntu'));
  assert.throws(() => sourceUrl('Ubuntu', undefined));
});

test('exact pinned response rejects generation substitution, truncation, extra bytes and corruption', async () => {
  const snapshot = validateMetadata(metadata(), 'crates.io', now);
  const response = bytes => new Response(bytes, {headers: {
    'x-goog-generation': snapshot.generation, 'content-length': '3'}});
  const written = [];
  assert.deepEqual(await verifyBody(response('abc'), snapshot, chunk => written.push(chunk), () => {}),
    {size: 3, sha256: createHash('sha256').update('abc').digest('hex')});
  assert.equal(Buffer.concat(written).toString(), 'abc');
  for (const bytes of ['ab', 'abcd', 'abd']) {
    await assert.rejects(verifyBody(response(bytes), snapshot, () => {}, () => {}));
  }
  const wrong = response('abc'); wrong.headers.set('x-goog-generation', '1789723713081169');
  await assert.rejects(verifyBody(wrong, snapshot, () => {}, () => {}));
  const wrongSize = response('abc'); wrongSize.headers.set('content-length', '4');
  await assert.rejects(verifyBody(wrongSize, snapshot, () => {}, () => {}));
  await assert.rejects(verifyBody(new Response('', {status: 404}), snapshot, () => {}, () => {}));
});

test('disk reserve failure aborts acquisition before publishing the next chunk', async () => {
  const bytes = Buffer.alloc(16 * 1024 ** 2);
  const snapshot = {generation: '1789723713081168', size: bytes.length,
    md5: createHash('md5').update(bytes).digest('base64')};
  const response = new Response(bytes, {headers: {'x-goog-generation': snapshot.generation,
    'content-length': String(bytes.length)}});
  let writes = 0;
  await assert.rejects(verifyBody(response, snapshot, () => writes++, () => {throw Error('disk reserve');}), /disk reserve/);
  assert.equal(writes, 0);
});

test('reviewed five-input manifest has exact metadata binding and oldest-source seven-day expiry', () => {
  const value = manifest();
  assert.equal(validateManifest(value), Math.min(...value.databases.map(row => row.source_created_unix)) + maxAgeSeconds);
  assert.deepEqual(value.databases.map(row => row.ecosystem), ecosystems);
  for (const mutate of [value => value.databases.pop(), value => value.databases.reverse(),
    value => value.databases.push(value.databases[0]), value => value.freshness_policy_seconds++,
    value => value.databases[0].source_created_unix++, value => value.databases[0].url += '&evil=1',
    value => value.databases[0].id += '-different', value => value.databases[0].sha256 = 'bad',
    value => value.databases[0].acquired_at = '2026-01-01T00:00:00.000Z']) {
    const changed = structuredClone(value); mutate(changed); assert.throws(() => validateManifest(changed));
  }
});

test('catalog application is idempotent and preserves licensing; shipped model includes exact manifest', () => {
  const catalog = readFileSync(new URL('../model/authorities.toml', import.meta.url), 'utf8');
  assert.equal(updateCatalog(catalog, manifest()), catalog);
  assert.throws(() => updateCatalog(catalog.replace('canonical_identifier = "gs://osv-vulnerabilities/Go/all.zip"',
    'canonical_identifier = "unreviewed"'), manifest()));
  assert.throws(() => updateCatalog(catalog.replaceAll('redistribution = "citation-only"',
    'redistribution = "redistributable"'), manifest()));
  const packageSpec = readFileSync(new URL('../crates/prismpm/Cargo.toml', import.meta.url), 'utf8');
  assert.ok(packageSpec.includes('"model/**"'));
  const policy = readFileSync(new URL('../crates/prismpm/src/supply_chain.rs', import.meta.url), 'utf8');
  assert.ok(policy.includes('include_bytes!("../model/osv-databases.json")'));
});
