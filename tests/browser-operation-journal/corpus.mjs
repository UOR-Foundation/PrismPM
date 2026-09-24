// Independent finite expectations; these bytes are fixtures, not user data.
import assert from 'node:assert/strict';
import {encodeEffectWire as encode} from '../../sdk/browser/effects-wire.mjs';

const bytes = (length, value) => new Uint8Array(length).fill(value);
export const reference = value => bytes(32, value);
export const binding = [reference(1), reference(2), reference(3), reference(4), reference(5),
  Uint8Array.of(4, ...bytes(64, 6)), 'journal-fixture', 'operations', 'staging', 1024];
export const manifest = [binding[0], binding[1], [67108864, 67108864, 16384], [['random', [1]]]];
const payload = [reference(10), 8, [reference(11)]];
export const request = [binding[0], binding[1], reference(7), 0, 'random', [1, 3]];
const response = [1, bytes(3, 9)];
export const prepared = [binding, 1, reference(8), reference(9), 0, request[2], 0, 'random', 1, 0, payload, 9, [0]];
export const beginning = [binding, 0, 0, reference(8), [0]];
export const pending = [binding, 1, 1, reference(12), [1, prepared]];
export const terminal = [...prepared];
terminal[1] = 2; terminal[2] = reference(12); terminal[9] = 1; terminal[11] = 1; terminal[12] = [1, payload];
const success = value => [1, 0, value], error = code => [1, 1, code], malformed = code => [1, 2, code];
const changed = (value, index, item) => { const next = structuredClone(value); next[index] = item; return next; };

export function corpus() {
  const cases = [];
  const add = (id, input, output, area) => cases.push({id, request: encode(input), response: encode(output), area});
  add('Binding', [1, 3, binding], success(binding), 'binding');
  add('Begin', [1, 0, binding, reference(8)], success(beginning), 'begin');
  add('Prepared', [1, 1, beginning, prepared, reference(12)], success(pending), 'admission');
  add('Terminal', [1, 1, pending, terminal, reference(13)], success([binding, 2, 1, reference(13), [0]]), 'completion');
  const rejected = changed(changed(terminal, 9, 2), 11, 8);
  add('RejectedTerminal', [1, 1, pending, rejected, reference(13)], success([binding, 2, 1, reference(13), [0]]), 'completion');
  add('RecordRoundTrip', [1, 2, prepared], success(prepared), 'wire');
  add('PayloadRoundTrip', [1, 6, payload], success(payload), 'wire');
  add('RequestBinding', [1, 4, prepared, request, manifest], success(true), 'request');
  add('ResultBinding', [1, 5, terminal, request, response, manifest], success(true), 'result');
  add('RejectedResultBinding', [1, 5, rejected, request, [8, [2]], manifest], success(true), 'result');
  for (const index of [0, 1, 2, 3, 4]) {
    const other = changed(binding, index, reference(77));
    add('ChangedBinding' + index, [1, 1, beginning, changed(prepared, 0, other), reference(12)], error(3), 'binding');
  }
  for (const [index, value] of [[5, Uint8Array.of(4, ...bytes(64, 77))], [6, 'elsewhere'], [7, 'other-head'], [8, 'other-staging'], [9, 1022]]) {
    add('ChangedBinding' + index, [1, 1, beginning, changed(prepared, 0, changed(binding, index, value)), reference(12)], error(3), 'binding');
  }
  for (const [name, value] of [['RecordsLow', changed(binding, 9, 1)], ['RecordsHigh', changed(binding, 9, 1025)],
    ['HeadAlias', changed(binding, 8, binding[7])], ['BadNamespace', changed(binding, 6, '../not-a-name')],
    ['BadKey', changed(binding, 5, bytes(64, 6))], ['BadApplication', changed(binding, 0, bytes(31, 1))]]) {
    add(name, [1, 0, value, reference(8)], error(0), 'limits');
  }
  add('BadState', [1, 1, changed(beginning, 1, 1025), prepared, reference(12)], error(2), 'state');
  add('BadRecord', [1, 1, beginning, changed(prepared, 8, 8), reference(12)], error(1), 'record');
  add('RevisionMismatch', [1, 1, beginning, changed(prepared, 1, 2), reference(12)], error(5), 'order');
  add('PredecessorMismatch', [1, 1, beginning, changed(prepared, 2, reference(77)), reference(12)], error(6), 'order');
  add('OperationMismatch', [1, 1, beginning, changed(prepared, 4, 1), reference(12)], error(8), 'order');
  const next = changed(changed(changed(prepared, 1, 2), 2, reference(12)), 4, 1);
  add('PendingBlocksExecution', [1, 1, pending, next, reference(13)], error(9), 'uncertainty');
  add('MissingPending', [1, 1, beginning, changed(changed(terminal, 1, 1), 2, reference(8)), reference(13)], error(10), 'uncertainty');
  for (const [index, value] of [[3, reference(88)], [4, 1], [5, reference(88)], [6, 1], [7, 'other'],
    [8, 2], [10, [reference(88), 8, [reference(11)]]]]) {
    const altered = changed(terminal, index, value);
    if (index === 8) altered[11] = 2;
    add('TerminalMismatch' + index, [1, 1, pending, altered, reference(13)], error(11), 'receipt');
  }
  const full = [binding, 1024, 512, reference(13), [0]];
  // Valid record limit precedes a prospective, out-of-range revision.
  add('HistoryExhausted', [1, 1, full, prepared, reference(12)], error(4), 'limit');
  const oneLeft = [binding, 1023, 511, reference(13), [0]], lastPrepared = changed(changed(changed(prepared, 1, 1024), 2, reference(13)), 4, 511);
  add('ReserveTerminalSlot', [1, 1, oneLeft, lastPrepared, reference(12)], error(4), 'limit');
  // Hostile typed-state probes, not a claim of four billion replayed records.
  add('CounterOverflow', [1, 1, [binding, 1, 0xffffffff, reference(8), [0]], changed(prepared, 1, 2), reference(12)], error(7), 'counter');
  add('CounterBeyondUint32', [1, 1, beginning, changed(prepared, 4, 0xffffffff), reference(12)], error(1), 'counter');
  for (const [index, value] of [[0, reference(77)], [1, reference(77)], [2, reference(77)], [3, 1], [4, 'other'], [5, [2, bytes(3, 1)]]]) {
    add('RequestMismatch' + index, [1, 4, prepared, changed(request, index, value), manifest], success(false), 'request');
  }
  add('RequestGrantMismatch', [1, 4, prepared, request, changed(manifest, 3, [['random', [2]]])], success(false), 'request');
  add('RequestManifestMismatch', [1, 4, prepared, request, changed(manifest, 0, reference(77))], success(false), 'request');
  for (const [name, value] of [['Width', [1, bytes(4, 9)]], ['Kind', [2, 'sha256:' + 'a'.repeat(64)]], ['Unknown', [9]], ['WrongFailure', [8, [3]]]]) {
    add('Result' + name, [1, 5, terminal, request, value, manifest], success(false), 'result');
  }
  for (const [name, value] of [['Empty', [reference(1), 0, []]], ['Missing', [reference(1), 8, []]],
    ['Extra', [reference(1), 8, [reference(2), reference(3)]]], ['Oversized', [reference(1), 67108865, Array.from({length: 64}, () => reference(2))]],
    ['ShortDigest', [bytes(31, 1), 8, [reference(2)]]]]) add('Payload' + name, [1, 6, value], malformed(3), 'payload');
  const canonical = encode([1, 3, binding]);
  for (const [name, raw, code] of [['Empty', [], 2], ['Trailing', [...canonical, 0], 8], ['Version', [0x83, 2, ...canonical.slice(2)], 3],
    ['Nonminimal', [0x83, 0x18, 1, ...canonical.slice(2)], 5], ['Indefinite', [0x9f, ...canonical.slice(1), 0xff], 4],
    ['OversizedArray', [0x98, 65], 6], ['UnknownOperation', [0x82, 1, 7], 3]]) {
    cases.push({id: 'Wire' + name, request: Uint8Array.from(raw), response: encode(malformed(code)), area: 'wire'});
  }
  assert.equal(new Set(cases.map(row => row.id)).size, cases.length);
  return cases;
}

export function historyCorpus() {
  const result = []; let state = structuredClone(beginning);
  for (let operation = 0; operation < 512; operation++) {
    const digest = reference(0); new DataView(digest.buffer).setUint32(0, operation + 1000);
    const record = structuredClone(prepared);
    record[1] = operation * 2 + 1; record[2] = state[3]; record[4] = operation; record[6] = operation;
    const pending = [binding, record[1], operation + 1, digest, [1, record]];
    result.push({id: 'HistoryPrepared' + operation, request: encode([1, 1, state, record, digest]), response: encode(success(pending))});
    const terminal = structuredClone(record), finished = reference(0); new DataView(finished.buffer).setUint32(0, operation + 2000);
    terminal[1]++; terminal[2] = digest; terminal[9] = 1; terminal[11] = 1; terminal[12] = [1, payload];
    state = [binding, terminal[1], operation + 1, finished, [0]];
    result.push({id: 'HistoryTerminal' + operation, request: encode([1, 1, pending, terminal, finished]), response: encode(success(state))});
  }
  assert.equal(state[1], 1024); assert.equal(state[2], 512); return result;
}

export function partitionCorpus() {
  const admitted = [1, 1048575, 1048576, 1048577, 67108864].map(length => {
    const request = bytes(length, 0x5a), plan = [];
    for (let offset = 0; offset < length; offset += 1048576) plan.push([offset, Math.min(1048576, length - offset)]);
    return {id: 'Partition' + length, request, response: encode(success(plan))};
  });
  return [{id: 'PartitionEmpty', request: new Uint8Array(), response: encode(malformed(6))}, ...admitted,
    {id: 'PartitionOversized', request: bytes(67108865, 0x5a), response: encode(malformed(6)), nativeOnly: true}];
}

export function maximumCorpus() {
  const largestBinding = structuredClone(binding);
  for (const index of [6, 7, 8]) largestBinding[index] = String.fromCharCode(97 + index).repeat(128);
  const largestPayload = [reference(1), 67108864, Array.from({length: 64}, () => reference(2))];
  const record = structuredClone(prepared); record[0] = largestBinding;
  record[7] = 'r'.repeat(128); record[10] = largestPayload;
  const state = [largestBinding, 1, 1, reference(12), [1, record]];
  const done = structuredClone(record); done[1] = 2; done[2] = reference(12);
  done[9] = 1; done[11] = done[8]; done[12] = [1, largestPayload];
  const objectId = 'sha256:' + 'a'.repeat(64), storeName = 's'.repeat(128);
  const fullManifest = [largestBinding[0], largestBinding[1], [67108864, 67108864, 16384],
    Array.from({length: 64}, (_, index) => [index === 63 ? record[7] : ('resource' + index).padEnd(128, 'x'),
      [5, [storeName, 1048576, 4096, 64]]])];
  const objects = Array.from({length: 16}, (_, index) => bytes(1048576, index));
  const fullRequest = [largestBinding[0], largestBinding[1], record[5], record[6], record[7],
    [7, ['h'.repeat(128), [1, objectId], objectId, objects]]];
  const storeRecord = structuredClone(record); storeRecord[8] = 7;
  const storeDone = structuredClone(done); storeDone[8] = 7; storeDone[11] = 7;
  return [
    {id: 'MaximumRecord', request: encode([1, 2, done]), response: encode(success(done))},
    {id: 'MaximumStateAndTerminal', request: encode([1, 1, state, done, reference(13)]),
      response: encode(success([largestBinding, 2, 1, reference(13), [0]]))},
    {id: 'MaximumActualRequest', request: encode([1, 4, storeRecord, fullRequest, fullManifest]), response: encode(success(true))},
    {id: 'MaximumActualResult', request: encode([1, 5, storeDone, fullRequest, [7, objectId], fullManifest]), response: encode(success(true))},
  ];
}
