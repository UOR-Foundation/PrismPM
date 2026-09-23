import test from 'node:test';
import assert from 'node:assert/strict';
import {decodeEffectWire, encodeEffectWire} from './effects-wire.mjs';

const bytes = text => Uint8Array.from(Buffer.from(text, 'hex'));
const fixtures = [
  ['00', 0], ['1818', 24], ['190100', 256], ['1affffffff', 0xffffffff],
  ['40', bytes('')], ['430001ff', bytes('0001ff')], ['60', ''],
  ['62c3bc', 'ü'], ['64f0908591', '𐅑'], ['63efbbbf', '\ufeff'],
  ['6365cc81', 'e\u0301'], ['f4', false], ['f5', true],
  ['80', []], ['840100818001', [1, 0, [[]], 1]],
];

test('effect host framing preserves independent deterministic CBOR literals', () => {
  for (const [hex, value] of fixtures) {
    assert.deepEqual(decodeEffectWire(bytes(hex)), value, hex);
    assert.deepEqual(encodeEffectWire(value), bytes(hex), hex);
  }
});

test('effect host framing rejects noncanonical, unsupported, truncated and trailing data', () => {
  for (const hex of ['', '1800', '190018', '1a00000100', '1b0000000000000001',
    '20', 'f6', 'f7', 'f818', 'a0', 'c0', 'f93e00', '9f01ff', '8180ff',
    '181800', '5affffffff', '7affffffff', '9affffffff', '9841',
    '61ff', '62c080', '63eda080', '64f4908080']) {
    assert.throws(() => decodeEffectWire(bytes(hex)), undefined, hex);
  }
  for (const [hex] of fixtures) {
    for (let length = 0; length < hex.length / 2; length++) {
      assert.throws(() => decodeEffectWire(bytes(hex.slice(0, length * 2))), undefined, hex);
    }
  }
});

test('effect host framing rejects type, string, nesting, list and node amplification', () => {
  for (const value of [-1, 0x100000000, NaN, Infinity, 0.5, null, undefined, {},
    '\ud800', 'a'.repeat(513), Array(65).fill(0)]) assert.throws(() => encodeEffectWire(value));
  assert.deepEqual(decodeEffectWire(encodeEffectWire('a'.repeat(512))), 'a'.repeat(512));
  let depth = 0;
  for (let i = 0; i < 18; i++) depth = [depth];
  assert.throws(() => encodeEffectWire(depth));
  assert.throws(() => decodeEffectWire(bytes('81'.repeat(18) + '00')));
  const wide = Array.from({length: 64}, () => Array(64).fill(0));
  assert.throws(() => encodeEffectWire(wide));
  assert.throws(() => decodeEffectWire(bytes('9840' + ('9840' + '00'.repeat(64)).repeat(64))));
  assert.throws(() => decodeEffectWire(new Uint8Array(new SharedArrayBuffer(1))));
});
