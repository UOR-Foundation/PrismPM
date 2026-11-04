import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';

const module = await WebAssembly.compile(await readFile(process.argv[2]));
assert.deepEqual(WebAssembly.Module.imports(module), []);
const fresh = () => new WebAssembly.Instance(module, {}).exports;
const read = (e, pointer, length) => new Uint8Array(e.memory.buffer, pointer, length);
const reserve = (e, size, alignment) => {
  const pointer = e.test_alloc(size, alignment) >>> 0;
  assert.equal(pointer % alignment, 0);
  return pointer;
};

// Fragmentation: adjacent holes combine and a free tail can grow again.
{
  const e = fresh();
  const a = reserve(e, 32768, 8), b = reserve(e, 32768, 8), c = reserve(e, 32768, 8);
  read(e, c, 32768).fill(99);
  e.test_free(b, 32768, 8); e.test_free(a, 32768, 8);
  const combined = reserve(e, 65536, 8);
  assert.equal(combined, a);
  assert.ok(read(e, c, 32768).every(x => x === 99));
  e.test_free(combined, 65536, 8); e.test_free(c, 32768, 8);
  assert.equal(reserve(e, 131072, 8), a);
}

// Deterministic allocation churn with live-value checks, odd sizes, and
// over-alignment. Cumulative allocations exceed the entire guest by >100x.
{
  const e = fresh(), live = new Map(); let seed = 0x36a72819, total = 0;
  const random = () => (seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0);
  for (let step = 0; step < 40000; step++) {
    const slot = random() % 32, old = live.get(slot);
    if (old) {
      assert.ok(read(e, old.pointer, old.size).every(x => x === old.value));
      e.test_free(old.pointer, old.size, old.alignment); live.delete(slot);
    }
    const size = 1 + random() % 2048, alignment = 2 ** (random() % 13);
    const pointer = reserve(e, size, alignment), value = (step % 251) + 1;
    for (const other of live.values()) {
      assert.ok(pointer + size <= other.pointer || other.pointer + other.size <= pointer);
      assert.ok(read(e, other.pointer, other.size).every(x => x === other.value));
    }
    read(e, pointer, size).fill(value); live.set(slot, {pointer, size, alignment, value});
    total += size;
  }
  assert.ok(total > 32 * 1024 * 1024);
  for (const old of live.values()) e.test_free(old.pointer, old.size, old.alignment);
  reserve(e, 65536, 65536);
  assert.ok(e.memory.buffer.byteLength <= 8 * 65536);
}

// ABI buffers retain their bytes while temporary Rust allocations are reused.
{
  const e = fresh(), saved = [];
  for (let round = 0; round < 50; round++) {
    const p = e.holo_alloc(1024), value = round + 1;
    read(e, p, 1024).fill(value);
    const result = BigInt.asUintN(64, e.holo_run(p, 1024));
    const output = Number(result >> 32n), length = Number(result & 0xffffffffn);
    assert.equal(length, 1024); saved.push([p, value], [output, value]);
    for (let i = 0; i < 100; i++) {
      const q = reserve(e, 2048, 256); read(e, q, 2048).fill(255); e.test_free(q, 2048, 256);
    }
    for (const [pointer, byte] of saved) assert.ok(read(e, pointer, 1024).every(x => x === byte));
  }
  // Caller allocations are deliberately cumulative; exceeding the real
  // declared memory bound still traps, with no import or automatic reset.
  assert.throws(() => { for (let i = 0; i < 100; i++) e.holo_alloc(65536); }, WebAssembly.RuntimeError);
  assert.equal(e.memory.buffer.byteLength, 8 * 65536);
}
console.log('Core-Wasm allocator: churn, alignment, coalescing, ABI lifetime, and bound exhaustion passed');
