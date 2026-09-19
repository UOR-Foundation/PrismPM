import assert from 'node:assert/strict';
import {createPublicKey, generateKeyPairSync, sign} from 'node:crypto';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {withBrowser} from './browser-test-server.mjs';
import {oracleVectors} from './rs256-test-vectors.mjs';

const vectors = oracleVectors().map(vector => ({hash: vector.hash,
  publicKey: [...vector.publicKey], message: [...vector.message], signature: [...vector.signature]}));
const positive = vectors[1];

async function inBrowser(operation, input, source) {
  return withBrowser(async ({browser, baseURL}) => {
    const page = await browser.newPage();
    const failures = [];
    page.on('pageerror', error => failures.push(error.message));
    page.on('request', request => {
      if (!request.url().startsWith(baseURL)) failures.push(request.url());
    });
    await page.route(baseURL, route => route.fulfill({status: 200, contentType: 'text/html',
      headers: {'Cross-Origin-Opener-Policy': 'same-origin', 'Cross-Origin-Embedder-Policy': 'require-corp'},
      body: '<!doctype html><title>RS256 primitive acceptance</title>'}));
    if (source !== undefined) await page.route(baseURL + 'rs256.mjs', route =>
      route.fulfill({status: 200, contentType: 'text/javascript', body: source}));
    await page.goto(baseURL);
    try { return await page.evaluate(operation, input); }
    finally { assert.deepEqual(failures, []); }
  });
}

test('Chromium executes every fixed upstream RSA vector without external requests', {timeout: 30000}, async () => {
  const result = await inBrowser(async vectors => {
    const {verifyRs256} = await import('./rs256.mjs');
    const results = [];
    for (const vector of vectors) results.push(await verifyRs256(new Uint8Array(vector.publicKey),
      new Uint8Array(vector.message), new Uint8Array(vector.signature)));
    return results;
  }, vectors);
  assert.deepEqual(result, [false, true, false, false]);
});

test('Chromium verifies empty and maximum messages and checks the RSA modulus boundary', {timeout: 30000}, async () => {
  const pair = generateKeyPairSync('rsa', {modulusLength: 2048});
  const payloads = [new Uint8Array(), new Uint8Array(1048576).fill(217)];
  const keys = [8192, 8200].map(bits => [...createPublicKey({format: 'jwk', key: {kty: 'RSA',
    n: Buffer.alloc(bits / 8, 255).toString('base64url'), e: 'AQAB'}}).export({format: 'der', type: 'spki'})]);
  const result = await inBrowser(async input => {
    const {verifyRs256} = await import('./rs256.mjs');
    const outcomes = [];
    for (let index = 0; index < 2; index++) outcomes.push(await verifyRs256(new Uint8Array(input.key),
      new Uint8Array(index === 0 ? 0 : 1048576).fill(217), new Uint8Array(input.signatures[index])));
    outcomes.push(await verifyRs256(new Uint8Array(input.keys[0]), new Uint8Array(), new Uint8Array(1024)));
    try { await verifyRs256(new Uint8Array(input.keys[1]), new Uint8Array(), new Uint8Array(1024)); outcomes.push('accepted'); }
    catch (error) { outcomes.push(error.code); }
    return outcomes;
  }, {key: [...pair.publicKey.export({format: 'der', type: 'spki'})],
    signatures: payloads.map(message => [...sign('sha256', message, pair.privateKey)]), keys});
  assert.deepEqual(result, [true, true, false, 'invalid-input']);
});

test('Chromium rejects malformed brands and captures buffers before asynchronous import', {timeout: 30000}, async () => {
  const result = await inBrowser(async vector => {
    const {verifyRs256} = await import('./rs256.mjs');
    const key = new Uint8Array(vector.publicKey), message = new Uint8Array(vector.message), signature = new Uint8Array(vector.signature);
    const descriptor = Object.getOwnPropertyDescriptor(globalThis, 'crypto');
    let calls = 0;
    const codes = [];
    const detached = new Uint8Array(1);
    structuredClone(detached.buffer, {transfer: [detached.buffer]});
    try {
      Object.defineProperty(globalThis, 'crypto', {configurable: true, get() { calls++; throw Error('private'); }});
      for (const input of [null, [], 'bytes', new Uint16Array(2),
        new Uint8Array(new SharedArrayBuffer(8)), detached,
        new Proxy(new Uint8Array(1), {get() { calls++; throw Error('proxy'); }}), new Uint8Array(1048577)]) {
        try { await verifyRs256(key, input, signature); codes.push('accepted'); }
        catch (error) { codes.push(error.code); }
      }
      try { await verifyRs256(key, message, signature); codes.push('accepted'); }
      catch (error) { codes.push(error.code); }
    } finally { Object.defineProperty(globalThis, 'crypto', descriptor); }
    const pending = verifyRs256(key, message, signature);
    key.fill(0); message.fill(0); signature.fill(0);
    return {codes, calls, captured: await pending};
  }, positive);
  assert.deepEqual(result, {codes: [...Array(8).fill('invalid-input'), 'crypto-unavailable'], calls: 1, captured: true});
});

test('Chromium rejects signature substitutions and sanitizes unavailable verification', {timeout: 30000}, async () => {
  const result = await inBrowser(async vector => {
    const {verifyRs256} = await import('./rs256.mjs');
    const key = new Uint8Array(vector.publicKey), message = new Uint8Array(vector.message), signature = new Uint8Array(vector.signature);
    const changed = message.slice(); changed[0] ^= 1;
    const outcomes = [await verifyRs256(key, changed, signature)];
    const altered = signature.slice(); altered[0] ^= 1;
    outcomes.push(await verifyRs256(key, message, altered));
    const descriptor = Object.getOwnPropertyDescriptor(globalThis, 'crypto'), subtle = crypto.subtle;
    try {
      Object.defineProperty(globalThis, 'crypto', {configurable: true, value: {subtle: {
        importKey: subtle.importKey.bind(subtle), verify() { throw new Proxy({}, {get() { throw Error('private'); }}); },
      }}});
      try { await verifyRs256(key, message, signature); outcomes.push('accepted'); }
      catch (error) {
        if (error.message !== error.code || Object.hasOwn(error, 'cause')) throw Error('provider detail leak');
        outcomes.push(error.code);
      }
    } finally { Object.defineProperty(globalThis, 'crypto', descriptor); }
    return outcomes;
  }, positive);
  assert.deepEqual(result, [false, false, 'crypto-unavailable']);
});

test('actual browser execution rejects signature, algorithm and byte-capture defects', {timeout: 60000}, async () => {
  const original = readFileSync(new URL('./rs256.mjs', import.meta.url), 'utf8');
  async function invariant({vector, mode}) {
    const {verifyRs256} = await import('./rs256.mjs');
    const key = new Uint8Array(vector.publicKey), message = new Uint8Array(vector.message), signature = new Uint8Array(vector.signature);
    if (mode === 'signature') message[0] ^= 1;
    const pending = verifyRs256(key, message, signature);
    if (mode === 'capture') message.fill(0);
    if (await pending !== (mode !== 'signature')) throw Error('RS256 behavioral invariant rejected');
    return true;
  }
  for (const [mode, before, after] of [
    ['signature', 'return result;', 'return true;'],
    ['algorithm', "hash: 'SHA-256'", "hash: 'SHA-384'"],
    ['capture', 'bytesCopy(message, MAX_MESSAGE_BYTES)', 'message'],
  ]) {
    assert.equal(original.split(before).length, 2, 'one exact mutation point');
    assert.equal(await inBrowser(invariant, {vector: positive, mode}, original), true);
    await assert.rejects(inBrowser(invariant, {vector: positive, mode}, original.replace(before, after)),
      /RS256 behavioral invariant rejected/);
  }
});
