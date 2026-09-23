import test from 'node:test';
import assert from 'node:assert/strict';
import {inspectEffectModule, inspectEffectArtifactBudget, EFFECT_ARTIFACTS_MAXIMUM} from './effects-module.mjs';
import {runInNewContext} from 'node:vm';
import {EFFECT_FRAME_MAXIMUM} from './effects-wire.mjs';

// These are rejection-only binary framing fragments, never executable guest
// fixtures. The owning acceptance suite supplies actual generated artifacts.
const header = [0, 97, 115, 109, 1, 0, 0, 0];
function unsigned(value) {
  const result = [];
  do {
    const byte = value % 128;
    value = Math.floor(value / 128);
    result.push(byte + (value ? 128 : 0));
  } while (value);
  return result;
}
const section = (id, payload) => [id, ...unsigned(payload.length), ...payload];
const memory = (initial, maximum) => section(5, [1, 1, ...unsigned(initial), ...unsigned(maximum)]);
const fragment = (...sections) => Uint8Array.from([...header, ...sections.flat()]);
const rejects = (bytes, pages = 16) => assert.throws(() => inspectEffectModule(bytes, pages), /invalid-effect-module/);

test('effect artifact byte budget admits the exact aggregate and rejects one byte more without compiler acceptance', () => {
  // These owned buffers test byte admission only, not valid Wasm or bootstrap
  // acceptance. Repeated references each count toward the future copied bytes.
  const large = new Uint8Array(EFFECT_FRAME_MAXIMUM);
  assert.equal(EFFECT_ARTIFACTS_MAXIMUM, 268435456);
  assert.equal(inspectEffectArtifactBudget(large, [large, large, large]), EFFECT_ARTIFACTS_MAXIMUM);
  assert.throws(() => inspectEffectArtifactBudget(large, [large, large, large, new Uint8Array(1)]), /invalid-effect-module/);
  const small = new Uint8Array(EFFECT_ARTIFACTS_MAXIMUM / 64);
  assert.equal(inspectEffectArtifactBudget(new Uint8Array(), Array(64).fill(small)), EFFECT_ARTIFACTS_MAXIMUM);
  assert.throws(() => inspectEffectArtifactBudget(new Uint8Array(), Array(65).fill(small)), /invalid-effect-module/);
  assert.throws(() => inspectEffectArtifactBudget(new Uint8Array(EFFECT_FRAME_MAXIMUM + 1), []), /invalid-effect-module/);
  assert.throws(() => inspectEffectArtifactBudget(new Uint8Array(), [new Uint8Array(EFFECT_FRAME_MAXIMUM + 1)]), /invalid-effect-module/);
  for (const field of ['byteLength', 'length', 'buffer']) Object.defineProperty(large, field, {get() { throw Error('caller getter'); }});
  assert.equal(inspectEffectArtifactBudget(large, [large, large, large]), EFFECT_ARTIFACTS_MAXIMUM);
  assert.throws(() => inspectEffectArtifactBudget(large, [large, large, large, new Uint8Array(1)]), /invalid-effect-module/);
});

test('effect artifact budget validates native byte brands and every captured list entry before copying', () => {
  const empty = new Uint8Array();
  const detached = new Uint8Array(1); structuredClone(detached.buffer, {transfer: [detached.buffer]});
  const detachedEmpty = new Uint8Array(); structuredClone(detachedEmpty.buffer, {transfer: [detachedEmpty.buffer]});
  const buffer = new ArrayBuffer(4, {maxByteLength: 8}), view = new Uint8Array(buffer, 2, 2);
  assert.equal(inspectEffectArtifactBudget(view, [view]), 4);
  buffer.resize(1);
  for (const invalid of [null, {}, [], new Uint16Array(1), detached, detachedEmpty, view,
    new Uint8Array(new SharedArrayBuffer(1)), new Proxy(new Uint8Array(1), {})]) {
    assert.throws(() => inspectEffectArtifactBudget(invalid, []), /invalid-effect-module/);
    assert.throws(() => inspectEffectArtifactBudget(empty, [empty, invalid]), /invalid-effect-module/);
  }
  const otherRealm = runInNewContext('new Uint8Array([1,2,3])');
  assert.equal(inspectEffectArtifactBudget(otherRealm, [otherRealm]), 6);
  let touched = 0;
  const accessor = [empty]; Object.defineProperty(accessor, '0', {get() { touched++; throw Error('entry accessor'); }});
  const extra = [empty]; Object.defineProperty(extra, 'hidden', {value: true});
  const symbol = [empty]; symbol[Symbol('hidden')] = true;
  for (const invalid of [null, {}, Array(1), accessor, extra, symbol]) {
    assert.throws(() => inspectEffectArtifactBudget(empty, invalid), /invalid-effect-module/);
  }
  assert.equal(touched, 0);
  assert.throws(() => inspectEffectArtifactBudget(empty), /invalid-effect-module/);
  assert.throws(() => inspectEffectArtifactBudget(empty, [], 'extra'), /invalid-effect-module/);
});

test('effect artifact preflight rejects invalid limits and non-owned byte inputs', () => {
  for (const pages of [undefined, null, 0, -1, 0.5, NaN, Infinity, 65537, '16']) rejects(fragment(), pages);
  for (const bytes of [null, {}, [], new Uint32Array(2), new Uint8Array(new SharedArrayBuffer(8))]) rejects(bytes);
  rejects(new Uint8Array(EFFECT_FRAME_MAXIMUM + 1));
  assert.throws(() => inspectEffectModule(fragment()), /invalid-effect-module/);
  assert.throws(() => inspectEffectModule(fragment(), 16, 'extra'), /invalid-effect-module/);
});

test('effect artifact preflight rejects truncated, noncanonical and unbounded framing', () => {
  for (let length = 0; length <= header.length; length++) rejects(Uint8Array.from(header.slice(0, length)));
  rejects(Uint8Array.from([0, 97, 115, 109, 2, 0, 0, 0]));
  for (const tail of [
    [5], [5, 128], [5, 128, 0], [5, 129, 0, 0],
    [5, 128, 128, 128, 128, 128], [5, 255, 255, 255, 255, 16],
    [5, 255, 255, 255, 255, 15], [5, 4, 1, 1, 1],
    [13, 0], [255, 0],
  ]) rejects(fragment(tail));
  rejects(fragment(...Array.from({length: 65}, () => section(0, [0])), memory(1, 16)));
  rejects(fragment(memory(1, 16), section(1, [0])));
  rejects(fragment(memory(1, 16), memory(1, 16)));
  rejects(fragment(memory(1, 16), section(10, [0]), section(12, [0])));
  rejects(fragment(memory(1, 16), section(12, [0]), section(12, [0])));
});

test('effect artifact preflight rejects imports, starts and ambiguous memory limits before execution', () => {
  rejects(fragment(section(2, [1]), memory(1, 16)));
  rejects(fragment(section(2, [0, 0]), memory(1, 16)));
  rejects(fragment(section(2, [128, 0]), memory(1, 16)));
  rejects(fragment(section(2, []), memory(1, 16)));
  rejects(fragment(memory(1, 16), section(8, [0])));
  for (const payload of [
    [], [0], [2, 1, 1, 16, 1, 1, 16],
    [1, 0, 1], // No explicit maximum.
    [1, 2, 1], [1, 3, 1, 16], // Shared memory flags.
    [1, 4, 1], [1, 5, 1, 16], [1, 7, 1, 16], // Memory64/combined flags.
    [1, 1, 1, 16, 0], [1, 1, 1],
    [129, 0, 1, 1, 16], [1, 129, 0, 1, 16],
    [1, 1, 129, 0, 16], [1, 1, 1, 144, 0],
  ]) rejects(fragment(section(5, payload)));
  rejects(fragment(memory(17, 17)));
  rejects(fragment(memory(1, 17)));
  rejects(fragment(memory(16, 1)));
  rejects(fragment(memory(1, 65537)), 65536);
});
