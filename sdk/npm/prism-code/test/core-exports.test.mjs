// The Core-Wasm export contract of compileGuest: a target/1 guest exports
// exactly memory, holo_alloc and holo_run; a target/2 guest (SPEC.md,
// application V2) also exports its immutable __stack_low and __stack_high
// linker globals, both or neither.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { compileGuest, PrismCodeError } from '../src/index.mjs';

const name = (text) => [text.length, ...new TextEncoder().encode(text)];
const section = (id, payload) => [id, payload.length, ...payload];
const vector = (items) => [items.length, ...items.flat()];

/**
 * A minimal import-free guest: one memory, holo_alloc and holo_run returning
 * constants, and the given immutable i32 globals exported under their names
 * (or, with `kind: 'function'`, an extra function export of that name).
 */
function guest({ globals = [], functions = [] } = {}) {
  const types = vector([
    [0x60, 1, 0x7f, 1, 0x7f], // (i32) -> i32
    [0x60, 2, 0x7f, 0x7f, 1, 0x7e], // (i32, i32) -> i64
  ]);
  const funcs = vector([[0], [1], ...functions.map(() => [0])]);
  const memory = vector([[0x00, 1]]);
  const global = vector(globals.map((_, i) => [0x7f, 0x00, 0x41, (i + 1) * 16, 0x0b]));
  const exports = vector([
    [...name('memory'), 0x02, 0],
    [...name('holo_alloc'), 0x00, 0],
    [...name('holo_run'), 0x00, 1],
    ...globals.map((g, i) => [...name(g), 0x03, i]),
    ...functions.map((f, i) => [...name(f), 0x00, 2 + i]),
  ]);
  const code = vector([
    [4, 0, 0x41, 0, 0x0b],
    [4, 0, 0x42, 0, 0x0b],
    ...functions.map(() => [4, 0, 0x41, 0, 0x0b]),
  ]);
  return Uint8Array.from([
    0x00, 0x61, 0x73, 0x6d, 1, 0, 0, 0,
    ...section(1, types),
    ...section(3, funcs),
    ...section(5, memory),
    ...(globals.length ? section(6, global) : []),
    ...section(7, exports),
    ...section(10, code),
  ]);
}

const refusedExports = (error) => error instanceof PrismCodeError && error.code === 'core.exports';

test('a target/1 guest with exactly the three Core-Wasm exports compiles', async () => {
  assert.ok(await compileGuest(guest()));
});

test('a target/2 guest that also exports __stack_low and __stack_high compiles', async () => {
  assert.ok(await compileGuest(guest({ globals: ['__stack_low', '__stack_high'] })));
});

test('one stack global without the other is refused', async () => {
  await assert.rejects(compileGuest(guest({ globals: ['__stack_low'] })), refusedExports);
  await assert.rejects(compileGuest(guest({ globals: ['__stack_high'] })), refusedExports);
});

test('a stack export that is not a global is refused', async () => {
  await assert.rejects(
    compileGuest(guest({ globals: ['__stack_low'], functions: ['__stack_high'] })),
    refusedExports,
  );
});

test('any other extra export is still refused', async () => {
  await assert.rejects(compileGuest(guest({ globals: ['__stack_low', '__stack_high', '__data_end'] })), refusedExports);
});
