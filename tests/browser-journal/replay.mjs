// Test-only instrumentation: native replay receives actual generated Wasm bytes.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { run } from './compile.mjs';
const sha = bytes => createHash('sha256').update(bytes).digest('hex');

export function captureGeneratedCalls() {
  const Instance = WebAssembly.Instance;
  const calls = [];
  const hex = bytes => Array.from(bytes, byte => byte.toString(16).padStart(2, '0')).join('');
  globalThis.__generatedJournalCalls = calls;
  WebAssembly.Instance = class {
    constructor(...args) {
      const instance = new Instance(...args);
      const exports = { ...instance.exports };
      exports.holo_run = (pointer, length) => {
        const request = hex(new Uint8Array(exports.memory.buffer, pointer, length));
        const result = instance.exports.holo_run(pointer, length);
        const packed = BigInt.asUintN(64, result);
        const offset = Number(packed >> 32n), size = Number(packed & 0xffffffffn);
        const response = hex(new Uint8Array(exports.memory.buffer, offset, size));
        // Snapshot before outer test wrappers inject completion faults.
        calls.push({ request, response });
        return result;
      };
      return { exports };
    }
  };
}

export async function replayNative(label, calls, wasm, {native: runner, work, mutant}) {
  assert.match(label, /^[a-z]+$/);
  assert.ok(runner, 'exact generated-native journal runner is required');
  assert.ok(Array.isArray(calls) && calls.length > 0, 'actual Wasm transcript is absent');
  const operations = new Set();
  for (const call of calls) {
    assert.deepEqual(Object.keys(call).sort(), ['request', 'response']);
    assert.match(call.request, /^(?:[0-9a-f]{2})+$/);
    assert.match(call.response, /^(?:[0-9a-f]{2})+$/);
    operations.add(Number.parseInt(call.request.slice(0, 2), 16));
  }
  assert.deepEqual([...operations].sort(), [0, 1, 2, 3, 4, 5, 6]);
  const rows = calls.map((call, index) => ({ id: `browser${index}`, ...call }));
  if (mutant === 'response') {
    rows[0].response = (Number.parseInt(rows[0].response.slice(0, 2), 16) ^ 1)
      .toString(16).padStart(2, '0') + rows[0].response.slice(2);
  } else assert.equal(mutant, undefined);
  const corpus = rows.map(row => `${row.id}\t${row.request}\t${row.response}\n`).join('');
  const path = join(work, `${label}${mutant ? '-mutant' : ''}.tsv`);
  await writeFile(path, corpus, {mode: 0o600, flag: 'wx'});
  const stdout = run(runner, [path], work);
  const lines = stdout.trimEnd().split('\n');
  assert.equal(lines.pop(), `PASS ${rows.length} complete generated journal vectors twice`);
  assert.equal(lines.length, rows.length);
  lines.forEach((line, index) => assert.match(line, new RegExp(`^PASS browser${index} [0-9]+ms$`)));
  process.stdout.write(`${JSON.stringify({
    testOnly: true, label, generatedCalls: calls.length, nativeRepetitions: 2,
    wasmSha256: sha(wasm), nativeSha256: sha(await readFile(runner)), transcriptSha256: sha(corpus),
  })}\n`);
}
