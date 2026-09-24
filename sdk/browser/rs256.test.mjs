import assert from 'node:assert/strict';
import {constants, createPublicKey, generateKeyPairSync, sign, verify, webcrypto} from 'node:crypto';
import {runInNewContext} from 'node:vm';
import test from 'node:test';
import {verifyRs256, MAX_MESSAGE_BYTES, MAX_PUBLIC_KEY_BYTES, MAX_SIGNATURE_BYTES} from './rs256.mjs';
import {oracleVectors} from './rs256-test-vectors.mjs';

globalThis.crypto ??= webcrypto;
const vectors = oracleVectors();
const positive = vectors[1];
const keyPair = generateKeyPairSync('rsa', {modulusLength: 2048});
const spki = new Uint8Array(keyPair.publicKey.export({format: 'der', type: 'spki'}));

test('all pinned WPT PKCS1 vectors execute with only the fixed SHA-256 algorithm admitted', async () => {
  for (const vector of vectors) {
    assert.equal(await verifyRs256(vector.publicKey, vector.message, vector.signature), vector.hash === 'SHA-256');
  }
});

test('changed key, message, signature and padding cannot preserve a valid signature', async () => {
  const message = positive.message.slice(), signature = positive.signature.slice();
  message[0] ^= 1;
  assert.equal(await verifyRs256(positive.publicKey, message, positive.signature), false);
  signature[0] ^= 1;
  assert.equal(await verifyRs256(positive.publicKey, positive.message, signature), false);
  assert.equal(await verifyRs256(spki, positive.message, positive.signature), false);
  const pss = sign('sha256', positive.message, {key: keyPair.privateKey, padding: constants.RSA_PKCS1_PSS_PADDING, saltLength: 32});
  assert.equal(await verifyRs256(spki, positive.message, pss), false);
});

test('empty and maximum-size exact messages verify against independent native signing', async () => {
  assert.deepEqual([MAX_MESSAGE_BYTES, MAX_PUBLIC_KEY_BYTES, MAX_SIGNATURE_BYTES], [1048576, 2048, 1024]);
  for (const message of [new Uint8Array(), new Uint8Array(MAX_MESSAGE_BYTES).fill(217)]) {
    const signature = sign('sha256', message, keyPair.privateKey);
    assert.equal(verify('sha256', message, keyPair.publicKey, signature), true);
    assert.equal(await verifyRs256(spki, message, signature), true);
  }
});

test('weak, non-RSA, malformed and over-limit keys cannot reach signature acceptance', async () => {
  const weak = generateKeyPairSync('rsa', {modulusLength: 1024});
  const ec = generateKeyPairSync('ec', {namedCurve: 'P-256'});
  for (const key of [new Uint8Array(), Uint8Array.of(1, 2, 3), new Uint8Array(2049),
    new Uint8Array(weak.publicKey.export({format: 'der', type: 'spki'})),
    new Uint8Array(ec.publicKey.export({format: 'der', type: 'spki'}))]) {
    await assert.rejects(verifyRs256(key, positive.message, positive.signature), {code: 'invalid-input'});
  }
  for (const bits of [2047, 8192, 8193]) {
    const modulus = Buffer.alloc(Math.ceil(bits / 8), 255);
    modulus[0] = 255 >>> ((8 - bits % 8) % 8);
    const key = createPublicKey({format: 'jwk', key: {kty: 'RSA', n: modulus.toString('base64url'), e: 'AQAB'}});
    const bytes = new Uint8Array(key.export({format: 'der', type: 'spki'}));
    if (bits === 8192) assert.equal(await verifyRs256(bytes, positive.message, new Uint8Array(1024)), false);
    else await assert.rejects(verifyRs256(bytes, positive.message, new Uint8Array(1024)), {code: 'invalid-input'});
  }
});

test('invalid byte brands and resource bounds fail before provider access or coercion', async () => {
  const original = Object.getOwnPropertyDescriptor(globalThis, 'crypto');
  let accessed = 0;
  try {
    Object.defineProperty(globalThis, 'crypto', {configurable: true, get() { accessed++; throw Error('provider'); }});
    const bad = [null, [], 'bytes', new Uint16Array(2), new DataView(new ArrayBuffer(1)),
      new Uint8Array(new SharedArrayBuffer(256)),
      new Proxy(new Uint8Array(256), {get() { accessed++; throw Error('proxy'); }})];
    for (const value of bad) {
      for (const args of [[value, positive.message, positive.signature],
        [positive.publicKey, value, positive.signature], [positive.publicKey, positive.message, value]]) {
        await assert.rejects(verifyRs256(...args), {code: 'invalid-input'});
      }
    }
    for (const args of [[new Uint8Array(2049), positive.message, positive.signature],
      [positive.publicKey, new Uint8Array(1048577), positive.signature],
      [positive.publicKey, positive.message, new Uint8Array(255)],
      [positive.publicKey, positive.message, new Uint8Array(1025)]]) {
      await assert.rejects(verifyRs256(...args), {code: 'invalid-input'});
    }
    assert.equal(accessed, 0);
  } finally { Object.defineProperty(globalThis, 'crypto', original); }
  await assert.rejects(verifyRs256(positive.publicKey, positive.message, new Uint8Array(257)), {code: 'invalid-input'});
});

test('foreign typed arrays work while shadowed getters and detached buffers are never trusted', async () => {
  const foreign = runInNewContext('Uint8Array');
  assert.equal(await verifyRs256(new foreign(positive.publicKey), new foreign(positive.message), new foreign(positive.signature)), true);
  const view = bytes => {
    const padded = new Uint8Array(bytes.length + 17).fill(173);
    padded.set(bytes, 5);
    return padded.subarray(5, 5 + bytes.length);
  };
  assert.equal(await verifyRs256(view(positive.publicKey), view(positive.message), view(positive.signature)), true);
  const message = positive.message.slice();
  for (const property of ['length', 'byteLength', 'buffer', Symbol.iterator]) {
    Object.defineProperty(message, property, {get() { throw Error('shadowed getter'); }});
  }
  assert.equal(await verifyRs256(positive.publicKey, message, positive.signature), true);
  const detached = positive.message.slice();
  structuredClone(detached.buffer, {transfer: [detached.buffer]});
  await assert.rejects(verifyRs256(positive.publicKey, detached, positive.signature), {code: 'invalid-input'});
});

test('all caller bytes are snapshotted before asynchronous key import', async () => {
  const key = positive.publicKey.slice(), message = positive.message.slice(), signature = positive.signature.slice();
  const pending = verifyRs256(key, message, signature);
  key.fill(0); message.fill(0); signature.fill(0);
  assert.equal(await pending, true);
});

test('provider methods are captured before asynchronous import and retain their receiver', async () => {
  const descriptor = Object.getOwnPropertyDescriptor(globalThis, 'crypto');
  const subtle = webcrypto.subtle;
  let imported = 0, checked = 0;
  const provider = {
    async importKey(...args) {
      assert.equal(this, provider); imported++;
      provider.verify = () => { throw Error('replacement method'); };
      return subtle.importKey(...args);
    },
    verify(...args) { assert.equal(this, provider); checked++; return subtle.verify(...args); },
  };
  try {
    Object.defineProperty(globalThis, 'crypto', {configurable: true, value: {subtle: provider}});
    assert.equal(await verifyRs256(positive.publicKey, positive.message, positive.signature), true);
    assert.deepEqual([imported, checked], [1, 1]);
  } finally { Object.defineProperty(globalThis, 'crypto', descriptor); }
});

test('provider failures are closed typed errors and never expose thrown details', async () => {
  const descriptor = Object.getOwnPropertyDescriptor(globalThis, 'crypto');
  const subtle = webcrypto.subtle;
  const hostile = new Proxy({}, {get() { throw Error('private thrown getter'); }});
  const cases = [[undefined, 'crypto-unavailable'], [{}, 'crypto-unavailable'],
    [{subtle: {get importKey() { throw hostile; }}}, 'crypto-unavailable'],
    [{subtle: {importKey() { throw hostile; }, verify() { throw Error('must not verify'); }}}, 'invalid-input'],
    ...[undefined, null, 1, 'true', {}].map(result => [{subtle: {
      importKey: subtle.importKey.bind(subtle), verify: async () => result}}, 'crypto-unavailable']),
    [{subtle: {importKey: subtle.importKey.bind(subtle), verify() { throw hostile; }}}, 'crypto-unavailable']];
  try {
    for (const [provider, code] of cases) {
      Object.defineProperty(globalThis, 'crypto', {configurable: true, value: provider});
      await assert.rejects(verifyRs256(positive.publicKey, positive.message, positive.signature), error => {
        assert.equal(error.code, code); assert.equal(error.message, code);
        assert.deepEqual(Object.keys(error).sort(), ['code', 'name']);
        assert.equal(Object.hasOwn(error, 'cause'), false); return true;
      });
    }
  } finally { Object.defineProperty(globalThis, 'crypto', descriptor); }
});
