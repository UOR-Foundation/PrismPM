import assert from 'node:assert/strict';
import {createHash, createPublicKey} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {browserFixture, readSemanticCounterexample} from './browser.mjs';
import {decodeEffectWire} from '../../sdk/browser/effects-wire.mjs';
import {sha} from '../browser-view/compile.mjs';
import {domain, encode} from './corpus.mjs';

const mutations = Object.freeze([
  {id: 'curve-import', from: 'keyIdentity = await identityPrincipal(publicKey);',
    to: 'keyIdentity = await digestBytes(publicKey);'},
  {id: 'identity-domain', from: 'const digest = await digestBytes(material);',
    to: 'const digest = await digestBytes(genesis);'},
  {id: 'captured-input', from: 'const genesis = copy(fields.genesis, FRAME), expected = copy(fields.expectedNamespace, 32);',
    to: 'const genesis = fields.genesis, expected = copy(fields.expectedNamespace, 32);'},
  {id: 'returned-copy', from: 'identity: copy(value.identity, 32)', to: 'identity: value.identity'},
]);

export function verifySemanticCounterexample(error, id) {
  const evidence = readSemanticCounterexample(error), genesis = Buffer.from(evidence.genesis, 'hex');
  // Fixture inputs originate outside the SDK under test. Independently check
  // their canonical bytes and derive the precise counterexample, not its label.
  const value = decodeEffectWire(genesis);
  assert.ok(Array.isArray(value) && value.length === 4 && value[0] === 1);
  assert.deepEqual(Buffer.from(value[1]), Buffer.alloc(32, 11));
  assert.deepEqual(Buffer.from(value[2]), Buffer.alloc(32, 12));
  assert.equal(value[3].length, 65); assert.equal(value[3][0], 4);
  assert.deepEqual(encode(value), genesis);
  const key = Buffer.from(value[3]);
  createPublicKey({key: {kty: 'EC', crv: 'P-256', x: key.subarray(1, 33).toString('base64url'),
    y: key.subarray(33).toString('base64url')}, format: 'jwk'});
  const hash = bytes => createHash('sha256').update(bytes).digest('hex');
  const identity = hash(Buffer.concat([domain, genesis]));
  const expected = {
    'curve-import': {journey: 'invalid-point', check: 'curve-import', expected: 'invalid-key', actual: 'accepted'},
    'identity-domain': {journey: 'declaration', check: 'identity-digest', expected: identity, actual: hash(genesis)},
    'captured-input': {journey: 'captured-inputs', check: 'captured-before-await', expected: evidence.genesis,
      actual: '00'.repeat(genesis.length)},
    'returned-copy': {journey: 'returned-copies', check: 'copied-fact-identity', expected: identity, actual: '00'.repeat(32)},
  };
  assert.ok(Object.hasOwn(expected, id), 'closed host mutation identity');
  assert.deepEqual(evidence.failure, expected[id], 'exact independently calculated host counterexample');
  return evidence;
}

export async function verifyHostMutations(build, engine) {
  const original = readFileSync(new URL('../../sdk/browser/account-genesis.mjs', import.meta.url), 'utf8');
  assert.equal(sha(Buffer.from(original)), build.inputs['sdk/browser/account-genesis.mjs']);
  // An unavailable engine or failed pristine setup cannot count as a killed defect.
  const positive = await browserFixture(build, engine);
  assert.equal(positive.journeys.length, 12);
  const counterfeit = Object.assign(new Error('actual account-genesis semantic counterexample'),
    {journey: 'invalid-point', check: 'curve-import', expected: 'invalid-key', actual: 'accepted'});
  assert.throws(() => verifySemanticCounterexample(counterfeit, 'curve-import'), /privately branded/);
  const factory = 'export async function openAccountGenesis(options) {';
  assert.equal(original.split(factory).length, 2);
  const unavailable = original.replace(factory, factory + '\nthrow Error("planted ordinary SDK failure: curve-import");');
  await assert.rejects(browserFixture(build, engine, unavailable), error => {
    assert.match(error.message, /planted ordinary SDK failure/);
    assert.throws(() => verifySemanticCounterexample(error, 'curve-import'), /privately branded/); return true;
  });
  const results = [];
  for (const mutation of mutations) {
    assert.equal(original.split(mutation.from).length, 2, 'one actual host mutation site');
    const changed = original.replace(mutation.from, mutation.to);
    assert.notEqual(changed, original);
    let evidence;
    await assert.rejects(browserFixture(build, engine, changed), error => {
      evidence = verifySemanticCounterexample(error, mutation.id);
      const wrong = mutation.id === 'curve-import' ? 'identity-domain' : 'curve-import';
      assert.throws(() => verifySemanticCounterexample(error, wrong), /exact independently calculated host counterexample/);
      return true;
    });
    results.push({id: mutation.id, failure: evidence.failure, fixtureGenesis: evidence.genesis,
      source: sha(Buffer.from(changed))});
  }
  // A no-op is never a killed mutation, even with a plausible expected check.
  await assert.rejects(async () => {
    await assert.rejects(browserFixture(build, engine, original), error => {
      verifySemanticCounterexample(error, 'curve-import'); return true;
    });
  }, error => error.code === 'ERR_ASSERTION' && error.message.includes('Missing expected rejection'));
  assert.equal(results.length, 4); return results;
}
