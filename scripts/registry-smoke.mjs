#!/usr/bin/env node
// Transport-only preflight for the already-started, isolated SDK gate registry.
// This does not replace Prism OCI artifact checks or the OCI conformance oracle.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { isIP } from 'node:net';
import { setTimeout as delay } from 'node:timers/promises';

const digest = bytes => `sha256:${createHash('sha256').update(bytes).digest('hex')}`;
const encode = value => Buffer.from(JSON.stringify(value));
const formats = [
  ['oci', 'application/vnd.oci.image.manifest.v1+json', 'application/vnd.oci.image.config.v1+json'],
  ['docker', 'application/vnd.docker.distribution.manifest.v2+json', 'application/vnd.docker.container.image.v1+json'],
];

async function main() {
  assert.equal(process.argv.length, 3, 'usage: registry-smoke.mjs http://<registry-ip>:5000');
  const endpoint = new URL(process.argv[2]);
  assert.ok(endpoint.protocol === 'http:' && isIP(endpoint.hostname) === 4
    && endpoint.port === '5000' && endpoint.pathname === '/'
    && !endpoint.username && !endpoint.password && !endpoint.search && !endpoint.hash,
  'expected the isolated registry IPv4 endpoint on port 5000');
  const request = async (path, options = {}) => {
    const response = await fetch(new URL(path, endpoint), {
      ...options, redirect: 'error', signal: AbortSignal.timeout(2000),
    });
    return {status: response.status, headers: response.headers, body: Buffer.from(await response.arrayBuffer())};
  };
  let ready = false;
  for (let attempt = 0; attempt < 20; attempt++) {
    try { ready = (await request('/v2/')).status === 200; } catch { /* bounded startup retry */ }
    if (ready) break;
    await delay(250);
  }
  assert.ok(ready, 'registry /v2/ did not become ready');

  const repository = '/v2/prismpm-sdk-registry-smoke';
  const config = encode({architecture: 'amd64', os: 'linux', config: {}, rootfs: {type: 'layers', diff_ids: []}});
  const upload = await request(`${repository}/blobs/uploads/`, {method: 'POST'});
  assert.equal(upload.status, 202, 'config upload initialization failed');
  const location = new URL(upload.headers.get('location'), endpoint);
  assert.equal(location.origin, endpoint.origin, 'upload location escaped the isolated registry');
  location.searchParams.set('digest', digest(config));
  const uploaded = await request(location, {
    method: 'PUT', headers: {'Content-Type': 'application/octet-stream'}, body: config,
  });
  assert.equal(uploaded.status, 201, 'config blob upload failed');

  for (const [name, mediaType, configType] of formats) {
    const manifest = {schemaVersion: 2, mediaType,
      config: {mediaType: configType, size: config.length, digest: digest(config)}, layers: []};
    const bytes = encode(manifest);
    const put = (reference, body) => request(`${repository}/manifests/${reference}`, {
      method: 'PUT', headers: {'Content-Type': mediaType}, body,
    });
    const accepted = await put(name, bytes);
    assert.equal(accepted.status, 201,
      `${name} manifest rejected (${accepted.status}): ${accepted.body.toString()}`);
    assert.equal(accepted.headers.get('docker-content-digest'), digest(bytes), `${name} digest changed`);
    const retrieved = await request(`${repository}/manifests/${digest(bytes)}`, {headers: {Accept: mediaType}});
    assert.equal(retrieved.status, 200, `${name} digest retrieval failed`);
    assert.equal(retrieved.headers.get('content-type'), mediaType, `${name} media type changed`);
    assert.equal(retrieved.headers.get('docker-content-digest'), digest(bytes), `${name} retrieved digest changed`);
    assert.deepEqual(retrieved.body, bytes, `${name} manifest bytes changed`);

    const malformed = await put(`${name}-malformed`, '{not-json');
    assert.equal(malformed.status, 400, `${name} malformed manifest was not rejected`);
    assert.ok(JSON.parse(malformed.body).errors.some(error => error.code === 'MANIFEST_INVALID'));
    const missing = {...manifest, config: {...manifest.config, digest: `sha256:${'f'.repeat(64)}`}};
    const incomplete = await put(`${name}-missing-blob`, encode(missing));
    assert.equal(incomplete.status, 400, `${name} missing referenced blob was not rejected`);
    assert.ok(JSON.parse(incomplete.body).errors.some(error => error.code === 'MANIFEST_INVALID'));
    console.log(`SDK registry preflight: ${name} byte/digest preservation and invalid-input rejection passed`);
  }
}

main().catch(error => {
  console.error(`SDK registry preflight failed: ${error.message}`);
  process.exitCode = 1;
});
