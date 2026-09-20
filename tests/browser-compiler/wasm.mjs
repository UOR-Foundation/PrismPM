// Independent generated-artifact ABI oracle, not browser application semantics.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {join} from 'node:path';
import {inspectEffectModule} from '../../sdk/browser/effects-module.mjs';

const root = process.argv[2];
assert.equal(process.argv.length, 3);
const binding = JSON.parse(readFileSync(join(root, 'compiler/binding.json')));
const application = JSON.parse(readFileSync(join(root, 'source/model.json'))).application;
assert.equal(application.acceptance_vectors.length, 1);
assert.equal(Object.keys(binding.targets).length, 6);
let cases = 0, maxima = 0, overruns = 0, limits = 0, outputs = 0;
for (const [id, target] of Object.entries(binding.targets)) {
  const bytes = readFileSync(join(root, 'guests', id, 'core.wasm'));
  const memory = inspectEffectModule(bytes, target.memory_pages);
  assert.equal(memory.maximumPages, target.memory_pages);
  assert.throws(() => inspectEffectModule(bytes, target.memory_pages - 1));
  const module = await WebAssembly.compile(bytes);
  assert.deepEqual(WebAssembly.Module.imports(module), []);
  if (binding.roles.primary === id) for (const vector of application.acceptance_vectors) {
    const {exports: {memory, holo_alloc, holo_run}} = new WebAssembly.Instance(module, {});
    const at = holo_alloc(vector.request.length) >>> 0;
    new Uint8Array(memory.buffer, at, vector.request.length).set(vector.request);
    const result = BigInt.asUintN(64, holo_run(at, vector.request.length));
    const pointer = Number(result >> 32n), length = Number(result & 0xffffffffn);
    assert.deepEqual(Array.from(new Uint8Array(memory.buffer, pointer, length)), vector.response);
    cases++;
  }
  const leaf = target.root.split('.').at(-1);
  const over = leaf === 'overOutput';
  const tag = {dispatch: 0xd1, present: 0xe2, replay: 0xf3}[leaf];
  if (!over) assert.notEqual(tag, undefined);
  for (const length of (over ? [0, 1] : [0, 1, 255, target.input_maximum])) {
    const {exports: {memory, holo_alloc, holo_run}} = new WebAssembly.Instance(module, {});
    const at = holo_alloc(length) >>> 0;
    new Uint8Array(memory.buffer, at, length).fill(0x5a);
    if (over) {
      assert.equal(target.output_maximum, 1);
      assert.throws(() => holo_run(at, length), WebAssembly.RuntimeError);
      outputs++;
      continue;
    }
    const result = BigInt.asUintN(64, holo_run(at, length));
    const pointer = Number(result >> 32n), size = Number(result & 0xffffffffn);
    assert.equal(size, length + 1);
    const output = new Uint8Array(memory.buffer, pointer, size);
    assert.ok(output.subarray(0, -1).every(byte => byte === 0x5a));
    assert.equal(output.at(-1), tag);
    assert.ok(memory.buffer.byteLength <= target.memory_pages * 65536);
    if (length === target.input_maximum) { assert.equal(size, target.output_maximum); maxima++; }
    cases++;
  }
  const {exports: {memory: bounded, holo_alloc}} = new WebAssembly.Instance(module, {});
  assert.throws(() => holo_alloc(target.input_maximum + 1), WebAssembly.RuntimeError); overruns++;
  assert.throws(() => bounded.grow(target.memory_pages + 1), RangeError); limits++;
}
assert.equal(cases, 21); assert.equal(maxima, 5); assert.equal(overruns, 6); assert.equal(limits, 6); assert.equal(outputs, 2);
console.log('PASS 21 Wasm source vectors, five exact maxima, six input overruns, six memory limits, two output overruns');
