// Independent finite protocol expectations; not an application implementation.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {encodeEffectWire as encode} from '../../sdk/browser/effects-wire.mjs';
const bytes = (size, value) => new Uint8Array(size).fill(value);
const digest = byte => 'sha256:' + byte.repeat(64);
const optional = value => value === undefined ? [0] : [1, value];
const app = bytes(32, 1), reference = bytes(32, 2), session = bytes(32, 3), artifact = bytes(32, 4);
const publicKey = bytes(65, 5); publicKey[0] = 4;
const grant = [publicKey, 'effect-fixture/1'];
export const manifest = [app, reference, [67108864, 67108864, 16384], [
  ['guest', [0, [artifact, 'Fixture.echoBytes', 'fixture/1', 64, 64, 16]]],
  ['random', [1]], ['digest', [2]], ['sign', [3, grant]], ['verify', [4, grant]],
  ['store', [5, ['effects-fixture', 1048576, 4096, 64]]],
]];
const state = (changes = {}) => {
  const value = {manifest, session, next: 0, closed: false, uncertain: false, active: [0], waiter: [0], ...changes};
  return [value.manifest, value.session, value.next, value.closed, value.uncertain, value.active, value.waiter];
};
const request = (resource, effect, changes = {}) => {
  const value = {app, reference, session, operation: 0, resource, effect, ...changes};
  return [value.app, value.reference, value.session, value.operation, value.resource, value.effect];
};
const pending = (request, changes = {}) => state({next: request[3] + 1, active: optional(request), ...changes});
const success = value => [1, 0, value], rejected = error => [1, 1, [error]], malformed = error => [1, 2, error];
const payload = bytes(3, 7), signature = bytes(64, 8);
const operations = [
  ['Guest', 'guest', [0, [artifact, 'Fixture.echoBytes', 'fixture/1', payload]], [0, payload]],
  ['Random', 'random', [1, 3], [1, payload]],
  ['Digest', 'digest', [2, payload], [2, digest('a')]],
  ['Sign', 'sign', [3, payload], [3, signature]],
  ['Verify', 'verify', [4, [payload, signature]], [4, true]],
  ['Object', 'store', [5, digest('a')], [5, optional(payload)]],
  ['Head', 'store', [6, 'head'], [6, optional([digest('a'), payload])]],
  ['Commit', 'store', [7, ['head', [0], digest('a'), [payload]]], [7, digest('a')]],
];
const allowedFailures = [[10, 11, 12], [0, 2], [0, 2], [0, 1, 2], [0, 2],
  [0, 2, 5, 7, 8, 9], [0, 2, 5, 7, 8, 9], [0, 2, 3, 4, 5, 6, 7, 8]];

export function corpus() {
  const cases = [];
  const add = (id, input, output, area) => cases.push({id, request: encode(input), response: encode(output), area});
  const raw = (id, input, output, area) => cases.push({id, request: Uint8Array.from(input), response: encode(output), area});
  add('Begin', [1, 0, manifest, session], success(state()), 'begin');
  const altered = structuredClone(manifest); altered[3] = [];
  add('BadManifest', [1, 0, altered, session], rejected(0), 'protocol-error');
  add('BadSession', [1, 0, manifest, bytes(31, 3)], rejected(1), 'protocol-error');
  for (const [index, [name, resource, effect, result]] of operations.entries()) {
    const selected = request(resource, effect);
    add(name + 'Admit', [1, 1, state(), selected], success(pending(selected)), 'admission');
    add(name + 'Complete', [1, 2, pending(selected), [selected, result]], success(state({next: 1})), 'completion');
    add(name + 'Unknown', [1, 2, pending(selected), [selected, [9]]], success(pending(selected, {uncertain: true})), 'unknown');
    for (const [other, , , output] of operations) if (other !== name) {
      add(name + 'Rejects' + other, [1, 2, pending(selected), [selected, output]], rejected(14), 'result-binding');
    }
    for (let failure = 0; failure < 13; failure++) {
      add(name + 'Failure' + failure, [1, 2, pending(selected), [selected, [8, [failure]]]],
        allowedFailures[index].includes(failure) ? success(state({next: 1})) : rejected(14), 'failure-binding');
    }
  }
  const selected = request('random', [1, 3]), waiter = request('digest', [2, payload], {operation: 1});
  const two = pending(selected, {next: 2, waiter: optional(waiter)});
  add('Waiter', [1, 1, pending(selected), waiter], success(two), 'queue');
  add('Busy', [1, 1, two, request('random', [1, 3], {operation: 2})], rejected(4), 'protocol-error');
  add('Promote', [1, 2, two, [selected, [1, payload]]], success(pending(waiter)), 'queue');
  add('WaiterCannotComplete', [1, 2, two, [waiter, [2, digest('a')]]], rejected(13), 'queue');
  add('UnknownRetainsBoth', [1, 2, two, [selected, [9]]], success(pending(selected, {next: 2, waiter: optional(waiter), uncertain: true})), 'unknown');
  add('CloseRetainsBoth', [1, 3, two], success(pending(selected, {next: 2, waiter: optional(waiter), closed: true})), 'close');
  add('Closed', [1, 1, state({closed: true}), selected], rejected(2), 'protocol-error');
  add('UnknownBarrier', [1, 1, pending(selected, {uncertain: true}), waiter], rejected(3), 'protocol-error');
  add('Exhausted', [1, 1, state({next: 0xffffffff}), selected], rejected(5), 'protocol-error');
  for (const [name, changes, code] of [
    ['Application', {app: bytes(32, 9)}, 6], ['Manifest', {reference: bytes(32, 9)}, 7],
    ['Session', {session: bytes(32, 9)}, 8], ['Operation', {operation: 1}, 9],
    ['Resource', {resource: 'unknown'}, 10], ['Effect', {effect: [2, payload]}, 11],
  ]) add('Wrong' + name, [1, 1, state(), request('random', [1, 3], changes)], rejected(code), 'protocol-error');
  add('NoActive', [1, 2, state(), [selected, [1, payload]]], rejected(12), 'protocol-error');
  add('CompletionSubstitution', [1, 2, pending(selected), [request('random', [1, 4]), [1, bytes(4, 7)]]], rejected(13), 'protocol-error');
  add('ResultWidth', [1, 2, pending(selected), [selected, [1, bytes(4, 7)]]], rejected(14), 'protocol-error');
  add('MissingObject', [1, 2, pending(request('store', [5, digest('a')])), [request('store', [5, digest('a')]), [5, [0]]]], success(state({next: 1})), 'optional');
  add('MissingHead', [1, 2, pending(request('store', [6, 'head'])), [request('store', [6, 'head']), [6, [0]]]], success(state({next: 1})), 'optional');
  add('VerifyFalse', [1, 2, pending(request('verify', [4, [payload, signature]])), [request('verify', [4, [payload, signature]]), [4, false]]], success(state({next: 1})), 'verification');
  add('Version', [2, 0, manifest, session], malformed(3), 'wire');
  add('OperationTag', [1, 4, state()], malformed(3), 'wire');
  const encoded = encode([1, 0, manifest, session]);
  raw('Trailing', [...encoded, 0], malformed(8), 'wire');
  raw('Empty', [], malformed(2), 'wire');
  raw('ShortArray', [0x84, 1], malformed(2), 'wire');
  raw('NonminimalVersion', [0x84, 0x18, 1, ...encoded.slice(2)], malformed(5), 'wire');
  raw('UnsupportedArray', [0x9f, ...encoded.slice(1), 0xff], malformed(4), 'wire');
  raw('TooManyArrayItems', [0x98, 65], malformed(6), 'wire');
  assert.equal(new Set(cases.map(row => row.id)).size, cases.length);
  assert.deepEqual([...new Set(cases.filter(row => row.area === 'protocol-error').map(row => decodeError(row.response)))].sort((a, b) => a - b), Array.from({length: 15}, (_, i) => i));
  return cases;
}

export function maximumCorpus() {
  const maximum = 64 * 1024 * 1024;
  const wide = structuredClone(manifest);
  const guestResource = 'g'.repeat(128), storeResource = 's'.repeat(128);
  const entry = 'Fixture.' + 'a'.repeat(504), protocol = 'p'.repeat(128), head = 'h'.repeat(128);
  wide[2] = [0x7fffffff, 0x7fffffff, 65536];
  wide[3] = [
    [guestResource, [0, [artifact, entry, protocol, 2097152, 2097152, 65536]]],
    [storeResource, [5, ['n'.repeat(128), 1048576, 4096, 64]]],
  ];
  while (wide[3].length < 64) {
    const grant = structuredClone(wide[3][0]);
    grant[0] = ('grant-' + wide[3].length).padEnd(128, '-');
    wide[3].push(grant);
  }
  const payload = bytes(2097152, 0x5a), objects = Array.from({length: 16}, (_, index) => bytes(1048576, index));
  const first = request(storeResource, [7, [head, [1, digest('c')], digest('a'), objects]], {operation: 0xfffffffd});
  const second = request(storeResource, [7, [head, [1, digest('d')], digest('b'), objects]], {operation: 0xfffffffe});
  const two = pending(first, {manifest: wide, next: 0xffffffff, waiter: optional(second)});
  const cases = [];
  const add = (id, input, output) => cases.push({id, request: encode(input), response: encode(output), area: 'maximum'});
  add('MaximumCommitWaiter', [1, 1, pending(first, {manifest: wide}), second], success(two));
  add('MaximumCommitCompletion', [1, 2, two, [first, [7, digest('a')]]], success(pending(second, {manifest: wide})));
  const uncertain = structuredClone(two); uncertain[4] = true;
  add('MaximumCommitUnknown', [1, 2, two, [first, [9]]], success(uncertain));
  const guest = request(guestResource, [0, [artifact, entry, protocol, payload]]);
  add('MaximumGuestCompletion', [1, 2, pending(guest, {manifest: wide}), [guest, [0, payload]]], success(state({manifest: wide, next: 1})));
  const exact = new Uint8Array(maximum), beginning = encode([1, 0, manifest, session]); exact.set(beginning);
  cases.push({id: 'MaximumRejectedFrame', request: exact, response: encode(malformed(8)), area: 'maximum'});
  assert.ok(cases.every(row => row.request.length <= maximum && row.response.length <= maximum));
  return cases;
}
export function boundaryCorpus() {
  const cases = [];
  const add = (id, input, output) => cases.push({id, request: encode(input), response: encode(output), area: 'bounds'});
  const raw = (id, input, output) => cases.push({id, request: Uint8Array.from(input), response: encode(output), area: 'bounds'});
  const wide = structuredClone(manifest); wide[3][0][1][1][3] = 2097152; wide[3][0][1][1][4] = 2097152; wide[3][0][1][1][5] = 64;
  for (const slot of [3, 4]) {
    const oversized = structuredClone(wide); oversized[3][0][1][1][slot]++;
    add('GuestBudgetPlusOne' + slot, [1, 0, oversized, session], malformed(6));
  }
  for (const [name, size, resource, operation, result] of [
    ['Random', 65536, 'random', n => [1, n], n => [1, bytes(n, 0x5a)]],
    ['Digest', 1048576, 'digest', n => [2, bytes(n, 0x5a)], () => [2, digest('a')]],
    ['Sign', 1048576, 'sign', n => [3, bytes(n, 0x5a)], () => [3, signature]],
    ['Guest', 2097152, 'guest', n => [0, [artifact, 'Fixture.echoBytes', 'fixture/1', bytes(n, 0x5a)]], n => [0, bytes(n, 0x5a)]],
  ]) {
    const current = request(resource, operation(size));
    add(name + 'ExactByteBound', [1, 2, pending(current, {manifest: wide}), [current, result(size)]], success(state({manifest: wide, next: 1})));
    const beyond = request(resource, operation(size + 1));
    add(name + 'ByteBoundPlusOne', [1, 1, state({manifest: wide}), beyond], name === 'Random' ? rejected(11) : malformed(6));
  }
  const fullGuest = request('guest', [0, [artifact, 'Fixture.echoBytes', 'fixture/1', bytes(2097152, 0x5a)]]);
  add('GuestResultByteBoundPlusOne', [1, 2, pending(fullGuest, {manifest: wide}), [fullGuest, [0, bytes(2097153, 0x5a)]]], malformed(6));
  // These are the existing owning compiler-fixture declarations, not smaller
  // substitute bounds. This verifies wire compatibility, not composition of
  // the Workspace/Journal applications or their distinct semantic oracles.
  for (const [name, path, expected] of [
    ['Workspace', '../browser-workspace/src/main.rs', [1104664, 1100428, 512]],
    ['Journal', '../browser-journal/driver/src/main.rs', [1235980, 1166008, 640]],
  ]) {
    const source = readFileSync(new URL(path, import.meta.url), 'utf8');
    const declared = ['input_allocation_cap', 'output_allocation_cap', 'maximum_pages'].map(field => {
      const matched = new RegExp(field + ':\\s*([0-9_]+),').exec(source); assert.ok(matched);
      return Number(matched[1].replaceAll('_', ''));
    });
    assert.deepEqual(declared, expected, 'existing complete ' + name + ' guest declarations');
    const compatible = structuredClone(manifest); compatible[3][0][1][1].splice(3, 3, ...declared);
    add(name + 'ExistingGuestBudget', [1, 0, compatible, session], success(state({manifest: compatible})));
    const invocation = request('guest', [0, [artifact, 'Fixture.echoBytes', 'fixture/1', bytes(declared[0], 0x5a)]]);
    add(name + 'ExistingGuestClosure', [1, 2, pending(invocation, {manifest: compatible}), [invocation, [0, bytes(declared[1], 0x5a)]]], success(state({manifest: compatible, next: 1})));
  }
  const limits = structuredClone(manifest); limits[2] = [0xffffffff, 0xffffffff, 65536];
  add('UInt32SumDoesNotWrap', [1, 0, limits, session], rejected(0));
  const huge = structuredClone(manifest); huge[3][0][1][1][1] = 'A'.repeat(512);
  const encodedHuge = encode([1, 0, huge, session]);
  const at = Buffer.from(encodedHuge).indexOf(Buffer.from(encode('A'.repeat(512)))); assert.ok(at > 0);
  raw('TextByteBoundPlusOne', [...encodedHuge.slice(0, at), 0x79, 2, 1, ...new Uint8Array(513).fill(65), ...encodedHuge.slice(at + 515)], malformed(6));
  add('WrongScalarType', [true, 0, manifest, session], malformed(3));
  raw('UInt64HeadRejected', [0x84, 0x1b, 0, 0, 0, 0, 0, 0, 0, 1], malformed(4));
  const prefix = [0x84, 1, 0, 0x84, ...encode(app), ...encode(reference), ...encode(manifest[2])];
  raw('GrantListBoundEarly', [...prefix, 0x98, 65], malformed(6));
  raw('InvalidResourceUtf8', [...prefix, 0x81, 0x82, 0x61, 0xff], malformed(7));
  raw('InvalidResourceScalar', [...prefix, 0x81, 0x82, 0xf6], malformed(3));
  for (const [name, objects, error] of [
    ['Duplicate', [payload, payload], rejected(11)], ['Empty', [], rejected(11)],
    ['CountPlusOne', Array.from({length: 17}, (_, i) => bytes(1, i)), malformed(6)],
  ]) add('CommitObjects' + name, [1, 1, state(), request('store', [7, ['head', [0], digest('a'), objects]])], error);
  const committed = request('store', [7, ['head', [0], digest('a'), [payload]]]);
  add('CommitWrongResultDigest', [1, 2, pending(committed), [committed, [7, digest('b')]]], rejected(14));
  assert.equal(new Set(cases.map(row => row.id)).size, cases.length);
  return cases;
}
function decodeError(bytes) {
  assert.equal(bytes.length, 5); assert.deepEqual([...bytes.slice(0, 4)], [0x83, 1, 1, 0x81]);
  return bytes[4];
}
