import assert from 'node:assert/strict';
import {createHash, createPublicKey, verify} from 'node:crypto';
import test from 'node:test';
import {withBrowser} from './browser-test-server.mjs';

async function inBrowser(operation) {
  return withBrowser(async ({browser, baseURL}) => {
    const page = await browser.newPage();
    const failures = [];
    page.on('pageerror', error => failures.push(error.message));
    page.on('request', request => {
      if (!request.url().startsWith(baseURL)) failures.push(request.url());
    });
    await page.goto(baseURL);
    const result = await page.evaluate(operation);
    await page.evaluate(() => new Promise(resolve => setTimeout(resolve, 0)));
    assert.deepEqual(failures, []);
    return result;
  });
}

test('Chromium identity is nonextractable and its signature verifies independently', {timeout: 30000}, async () => {
  const result = await inBrowser(async () => {
    const {createIdentity, signBytes, verifyBytes, validateIdentity} = await import('./identity.mjs');
    const identity = await createIdentity();
    await validateIdentity(identity);
    const payload = Uint8Array.of(0, 255, 7);
    const signature = await signBytes(identity, 'test/1', payload);
    let exportFailure;
    try { await crypto.subtle.exportKey('pkcs8', identity.privateKey); }
    catch (error) { exportFailure = error.name; }
    return {publicKey: [...identity.publicKey], signature: [...signature],
      principal: identity.principal, keys: Object.keys(identity).sort(),
      extractable: identity.privateKey.extractable, exportFailure,
      valid: await verifyBytes(identity.publicKey, 'test/1', payload, signature)};
  });
  assert.deepEqual(result.keys, ['principal', 'privateKey', 'publicKey']);
  assert.equal(result.extractable, false);
  assert.equal(result.exportFailure, 'InvalidAccessError');
  assert.equal(result.valid, true);
  const publicKey = Buffer.from(result.publicKey);
  assert.equal(publicKey.length, 65);
  assert.equal(result.principal, `sha256:${createHash('sha256').update(publicKey).digest('hex')}`);
  const key = createPublicKey({format: 'jwk', key: {kty: 'EC', crv: 'P-256',
    x: publicKey.subarray(1, 33).toString('base64url'),
    y: publicKey.subarray(33).toString('base64url')}});
  const payload = Buffer.from([0, 255, 7]);
  const message = Buffer.concat([Buffer.from('prismpm/browser-signature/1\0'),
    Buffer.from([0, 6]), Buffer.from('test/1'), payload]);
  assert.equal(result.signature.length, 64);
  assert.equal(verify('sha256', message, {key, dsaEncoding: 'ieee-p1363'}, Buffer.from(result.signature)), true);
  assert.equal(verify('sha256', payload, {key, dsaEncoding: 'ieee-p1363'}, Buffer.from(result.signature)), false);
});

test('Chromium rejects changed signature contexts, authors, payloads, and signatures', {timeout: 30000}, async () => {
  const result = await inBrowser(async () => {
    const {createIdentity, signBytes, verifyBytes, validateIdentity} = await import('./identity.mjs');
    const first = await createIdentity(), second = await createIdentity();
    const payload = Uint8Array.of(7, 8, 9);
    const signature = await signBytes(first, 'test.records/1', payload);
    const results = [await verifyBytes(first.publicKey, 'test.records/1', payload, signature),
      await verifyBytes(first.publicKey, 'test.records/2', payload, signature),
      await verifyBytes(second.publicKey, 'test.records/1', payload, signature),
      await verifyBytes(first.publicKey, 'test.records/1', Uint8Array.of(7, 8, 0), signature)];
    signature[0] ^= 1;
    results.push(await verifyBytes(first.publicKey, 'test.records/1', payload, signature));
    for (const identity of [{...first, privateKey: second.privateKey}, {...first, principal: second.principal}]) {
      try { await validateIdentity(identity); results.push('accepted'); }
      catch (error) { results.push(error.code); }
    }
    return results;
  });
  assert.deepEqual(result, [true, false, false, false, false, 'identity-corrupt', 'identity-corrupt']);
});

test('Chromium captures signing, verification, and identity inputs before asynchronous work', {timeout: 30000}, async () => {
  const result = await inBrowser(async () => {
    const {createIdentity, signBytes, verifyBytes, validateIdentity} = await import('./identity.mjs');
    const first = await createIdentity(), second = await createIdentity();
    const payload = Uint8Array.of(7, 8, 9), signingIdentity = {...first};
    const signing = signBytes(signingIdentity, 'test/1', payload);
    signingIdentity.privateKey = second.privateKey;
    payload.fill(0);
    const signature = await signing;
    const valid = await verifyBytes(first.publicKey, 'test/1', Uint8Array.of(7, 8, 9), signature);
    const publicKey = new Uint8Array(first.publicKey), capturedPayload = Uint8Array.of(7, 8, 9);
    const verifying = verifyBytes(publicKey, 'test/1', capturedPayload, signature);
    publicKey.fill(0); capturedPayload.fill(0); signature.fill(0);
    const mutable = {...first, publicKey: new Uint8Array(first.publicKey)};
    const validating = validateIdentity(mutable);
    mutable.privateKey = second.privateKey; mutable.principal = second.principal; mutable.publicKey.fill(0);
    const retained = await validating;
    return {valid, capturedVerification: await verifying, retainedPrincipal: retained.principal === first.principal,
      retainedKey: retained.privateKey === first.privateKey, retainedBytes: [...retained.publicKey],
      expectedBytes: [...first.publicKey]};
  });
  assert.equal(result.valid, true);
  assert.equal(result.capturedVerification, true);
  assert.equal(result.retainedPrincipal, true);
  assert.equal(result.retainedKey, true);
  assert.deepEqual(result.retainedBytes, result.expectedBytes);
});

test('Chromium enforces malformed input, key, and exact payload bounds', {timeout: 30000}, async () => {
  const result = await inBrowser(async () => {
    const {createIdentity, signBytes, verifyBytes} = await import('./identity.mjs');
    const identity = await createIdentity(), codes = [];
    const reject = async operation => {
      try { await operation(); codes.push('accepted'); }
      catch (error) { codes.push(error.code ?? error.name); }
    };
    for (const context of ['', 'bad\0domain', '../scope', 'x'.repeat(129), 12]) {
      await reject(() => signBytes(identity, context, new Uint8Array()));
    }
    for (const bytes of [[], new Uint16Array(1), new Uint8Array(1048577)]) {
      await reject(() => signBytes(identity, 'test/1', bytes));
    }
    await reject(() => verifyBytes(new Uint8Array(65), 'test/1', new Uint8Array(), new Uint8Array(64)));
    await reject(() => verifyBytes(identity.publicKey, 'test/1', new Uint8Array(), new Uint8Array(63)));
    const detached = Uint8Array.of(1);
    structuredClone(detached.buffer, {transfer: [detached.buffer]});
    await reject(() => signBytes(identity, 'test/1', detached));
    await reject(() => signBytes({...identity, privateKey: {}}, 'test/1', new Uint8Array()));
    const extractable = await crypto.subtle.generateKey({name: 'ECDSA', namedCurve: 'P-256'}, true, ['sign', 'verify']);
    await reject(() => signBytes({...identity, privateKey: extractable.privateKey}, 'test/1', new Uint8Array()));
    const maximum = new Uint8Array(1048576);
    const signature = await signBytes(identity, 'test/1', maximum);
    return {codes, maximumValid: await verifyBytes(identity.publicKey, 'test/1', maximum, signature)};
  });
  assert.deepEqual(result.codes, [...Array(11).fill('invalid-input'), ...Array(2).fill('identity-corrupt')]);
  assert.equal(result.maximumValid, true);
});
