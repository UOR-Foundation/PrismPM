import assert from 'node:assert/strict';
import { createPublicKey, verify, webcrypto } from 'node:crypto';
import test from 'node:test';
import { createIdentity, identityPrincipal, signBytes, verifyBytes, validateIdentity } from './identity.mjs';

globalThis.crypto ??= webcrypto;

test('nonextractable identity proves possession without assigning organization roles', async () => {
  const identity = await createIdentity();
  assert.match(identity.principal, /^sha256:[0-9a-f]{64}$/);
  assert.equal(identity.publicKey.length, 65);
  assert.equal(identity.privateKey.extractable, false);
  assert.equal(await identityPrincipal(identity.publicKey), identity.principal);
  const payload = new TextEncoder().encode('a portable signed record');
  const signature = await signBytes(identity, 'test.records/1', payload);
  assert.equal(signature.length, 64);
  assert.equal(await verifyBytes(identity.publicKey, 'test.records/1', payload, signature), true);
  await validateIdentity(identity);
  await assert.rejects(crypto.subtle.exportKey('pkcs8', identity.privateKey));
  assert.deepEqual(Object.keys(identity).sort(), ['principal', 'privateKey', 'publicKey']);
});

test('signatures cannot cross contexts, authors, or bytes', async () => {
  const first = await createIdentity();
  const second = await createIdentity();
  const bytes = Uint8Array.of(0, 255, 42);
  const signature = await signBytes(first, 'test.records/1', bytes);
  assert.equal(await verifyBytes(first.publicKey, 'test.records/2', bytes, signature), false);
  assert.equal(await verifyBytes(second.publicKey, 'test.records/1', bytes, signature), false);
  assert.equal(await verifyBytes(first.publicKey, 'test.records/1', Uint8Array.of(0, 255, 43), signature), false);
  signature[0] ^= 1;
  assert.equal(await verifyBytes(first.publicKey, 'test.records/1', bytes, signature), false);
  await assert.rejects(validateIdentity({ ...first, privateKey: second.privateKey }), { code: 'identity-corrupt' });
  await assert.rejects(validateIdentity({ ...first, principal: second.principal }), { code: 'identity-corrupt' });
});

test('cryptographic boundary rejects malformed and excessive input before dispatch', async () => {
  const identity = await createIdentity();
  for (const context of ['', 'bad\0domain', 'x'.repeat(129), '../scope', 12]) {
    await assert.rejects(signBytes(identity, context, new Uint8Array()), { code: 'invalid-input' });
  }
  for (const bytes of [[], new Uint8Array(1048577), new Uint16Array(1)]) {
    await assert.rejects(signBytes(identity, 'test/1', bytes), { code: 'invalid-input' });
  }
  await assert.rejects(verifyBytes(new Uint8Array(65), 'test/1', new Uint8Array(), new Uint8Array(64)), { code: 'invalid-input' });
  await assert.rejects(verifyBytes(identity.publicKey, 'test/1', new Uint8Array(), new Uint8Array(63)), { code: 'invalid-input' });
  const bytes = new Uint8Array(1048576);
  const signature = await signBytes(identity, 'test/1', bytes);
  assert.equal(await verifyBytes(identity.publicKey, 'test/1', bytes, signature), true);
});

test('signed input is captured before asynchronous key operations', async () => {
  const identity = await createIdentity();
  const bytes = Uint8Array.of(7, 8, 9);
  const operation = signBytes(identity, 'test/1', bytes);
  bytes[0] = 0;
  const signature = await operation;
  assert.equal(await verifyBytes(identity.publicKey, 'test/1', Uint8Array.of(7, 8, 9), signature), true);
});

test('identity validation captures its private key before any asynchronous work', async () => {
  const original = await createIdentity();
  const replacement = await createIdentity();
  let reads = 0;
  const identity = { publicKey: original.publicKey, principal: original.principal,
    get privateKey() { return ++reads === 1 ? original.privateKey : replacement.privateKey; } };
  const validated = await validateIdentity(identity);
  assert.equal(reads, 1);
  assert.equal(validated.privateKey, original.privateKey);
});

test('independent native verifier checks the exact domain-separated wire format', async () => {
  const identity = await createIdentity();
  const payload = Uint8Array.of(0, 255, 7);
  const signature = await signBytes(identity, 'test/1', payload);
  const key = createPublicKey({ format: 'jwk', key: {
    kty: 'EC', crv: 'P-256',
    x: Buffer.from(identity.publicKey.slice(1, 33)).toString('base64url'),
    y: Buffer.from(identity.publicKey.slice(33)).toString('base64url'),
  } });
  const message = Buffer.concat([Buffer.from('prismpm/browser-signature/1\0'),
    Buffer.from([0, 6]), Buffer.from('test/1'), Buffer.from(payload)]);
  assert.equal(verify('sha256', message, { key, dsaEncoding: 'ieee-p1363' }, signature), true);
  assert.equal(verify('sha256', payload, { key, dsaEncoding: 'ieee-p1363' }, signature), false);
});
