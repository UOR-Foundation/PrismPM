import assert from 'node:assert/strict';
import {test} from 'node:test';
import {decodePresentation, decodeIntent, encodeWire, encodeIntent, validateIntent} from '../../sdk/browser/presentation-wire.mjs';
import {corpus, boundaries, basic} from './corpus.mjs';

test('independent strict host parser accepts complete positive corpus and rejects all negative mutations', () => {
  for (const row of [...corpus(), ...boundaries()]) {
    const decode = row.request[0] === 0x84 ? decodeIntent : decodePresentation;
    if (row.request === row.response) assert.deepEqual(encodeWire(decode(row.request)), row.request, row.id);
    else assert.throws(() => decode(row.request), undefined, row.id);
  }
});
test('exact modeled intent bindings, revision, required values, choice and byte limits', () => {
  const frame = basic(), good = [1, 1, 1, [[5, 'a'], [6, 'b'], [7, 1]]];
  assert.equal(validateIntent(frame, good), true);
  for (const mutate of [x => {x[1]++;}, x => {x[2]++;}, x => {x[3].pop();},
    x => {x[3][0][1] = '';}, x => {x[3][0][1] = 'x'.repeat(33);},
    x => {x[3][2][1] = 0;}, x => {x[3][2][1] = 2;}, x => {x[3][0][1] = 1;}]) {
    const bad = structuredClone(good); mutate(bad); assert.throws(() => validateIntent(frame, bad));
  }
  const bytes = encodeIntent(good); assert.deepEqual(encodeIntent(good, bytes.length), bytes);
  assert.throws(() => encodeIntent(good, bytes.length - 1));
});
test('unpaired UTF-16, native byte brands, detached and shared buffers reject', () => {
  assert.throws(() => encodeIntent([1, 0, 1, [[1, '\ud800']]]));
  assert.throws(() => decodePresentation({byteLength: 8, buffer: new ArrayBuffer(8)}));
  assert.throws(() => decodePresentation(new Uint8Array(new SharedArrayBuffer(8))));
  const detached = new Uint8Array(8); structuredClone(detached, {transfer: [detached.buffer]});
  assert.throws(() => decodePresentation(detached));
});
test('declared per-View cap accepts exact bytes and rejects one-over without a fallback', () => {
  const bytes = encodeWire(basic());
  assert.deepEqual(decodePresentation(bytes, bytes.length), basic());
  assert.throws(() => decodePresentation(bytes, bytes.length - 1));
  assert.throws(() => decodePresentation(bytes, 0));
  assert.throws(() => decodePresentation(bytes, 67108865));
  const bomb = boundaries().find(row => row.id === 'AggregateFuelOver');
  assert.ok(bomb.request.length < 67108864);
  assert.throws(() => decodePresentation(bomb.request), error => error.code === 'limit');
});
