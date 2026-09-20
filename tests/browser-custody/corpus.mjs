// Independent positional protocol expectations, never cryptographic witnesses.
import assert from 'node:assert/strict';
// Test-only CBOR authoring includes deliberately over-limit lists and payloads.
// It is independent of the host framer and contains no custody transition code.
const encode = value => {
  function head(major, size) {
    if (size < 24) return Buffer.from([major * 32 + size]);
    if (size < 256) return Buffer.from([major * 32 + 24, size]);
    const result = Buffer.alloc(size < 65536 ? 3 : 5); result[0] = major * 32 + (size < 65536 ? 25 : 26);
    if (size < 65536) result.writeUInt16BE(size, 1); else result.writeUInt32BE(size, 1);
    return result;
  }
  function item(current) {
    if (Number.isSafeInteger(current) && current >= 0 && current <= 0xffffffff) return head(0, current);
    if (typeof current === 'string') { const bytes = Buffer.from(current); return Buffer.concat([head(3, bytes.length), bytes]); }
    if (current instanceof Uint8Array) return Buffer.concat([head(2, current.length), current]);
    assert.ok(Array.isArray(current)); return Buffer.concat([head(4, current.length), ...current.map(item)]);
  }
  return new Uint8Array(item(value));
};
export const bytes = (length, value = 1) => new Uint8Array(length).fill(value);
export const policy = [bytes(32), bytes(32, 2), ['alpha', 'beta'], [
  ['journal', 'alpha', 'prismpm/browser-operation-journal/1', 65536],
  ['small', 'alpha', 'custody/small/1', 2], ['wide', 'beta', 'custody/wide/1', 1048576],
]];
export const bindings = policy[2].map((slot, index) => {
  const key = bytes(65, index + 3); key[0] = 4;
  return [policy[0], policy[1], slot, key, 'sha256:' + String(index + 1).repeat(64)];
});
const snapshot = [policy, bindings];
const success = (tag, value) => [1, 0, [tag, value]];
const rejected = error => [1, 1, [error]], malformed = error => [1, 2, error];
const authorization = index => [...policy[3][index], bindings[index === 2 ? 1 : 0][3], bindings[index === 2 ? 1 : 0][4]];
const clone = value => structuredClone(value);
const vector = (id, request, response) => ({id, request: encode(request), response: encode(response)});

export function corpus() {
  const rows = [], add = (id, input, output) => rows.push(vector(id, input, output));
  const raw = (id, request, output) => rows.push({id, request: Uint8Array.from(request), response: encode(output)});
  add('Policy', [1, 0, policy], success(0, policy));
  const alphabet = '-.0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz';
  for (const [label, names] of [['Alphabet0', [...alphabet.slice(0, 64)].map(x => 'A' + x)],
    ['Alphabet1', [...alphabet.slice(63)].map(x => 'A' + x)], ['PrefixOrder', ['A', 'AA', 'AAA']]]) {
    const p = [policy[0], policy[1], names, names.map(name => [name, name, 'context/1', 1])];
    add(label, [1, 0, p], success(0, p));
  }
  add('Initialize', [1, 1, policy, [0], bindings], success(1, snapshot));
  add('Open', [1, 2, policy, [1, snapshot]], success(1, snapshot));
  add('AlreadyInitialized', [1, 1, policy, [1, snapshot], bindings], rejected(2));
  add('Missing', [1, 2, policy, [0]], rejected(3));
  for (const [index, resource] of policy[3].entries()) {
    add('Sign' + index, [1, 3, snapshot, policy[0], policy[1], resource[0], bytes(1)], success(2, authorization(index)));
  }
  add('EmptySign', [1, 3, snapshot, policy[0], policy[1], 'small', bytes(0)], success(2, authorization(1)));
  add('SignExactSmall', [1, 3, snapshot, policy[0], policy[1], 'small', bytes(2)], success(2, authorization(1)));
  add('SignOverSmall', [1, 3, snapshot, policy[0], policy[1], 'small', bytes(3)], rejected(7));
  add('SignExactJournal', [1, 3, snapshot, policy[0], policy[1], 'journal', bytes(65536)], success(2, authorization(0)));
  add('SignOverJournal', [1, 3, snapshot, policy[0], policy[1], 'journal', bytes(65537)], rejected(7));
  add('UnknownResource', [1, 3, snapshot, policy[0], policy[1], 'missing', bytes(0)], rejected(6));
  for (const [label, change] of [
    ['Application', p => { p[0][0] ^= 1; }], ['Reference', p => { p[1][0] ^= 1; }],
    ['Context', p => { p[3][0][2] = 'changed/1'; }], ['Maximum', p => { p[3][0][3] = 65535; }],
    ['Resource', p => { p[3][0][0] = 'journal2'; }], ['Slot', p => { p[3][0][1] = 'beta'; }],
  ]) {
    const altered = clone(policy); change(altered);
    add('PolicyMismatch' + label, [1, 2, altered, [1, snapshot]], rejected(4));
  }
  add('SignApplicationMismatch', [1, 3, snapshot, bytes(32, 9), policy[1], 'small', bytes(0)], rejected(4));
  add('SignPolicyMismatch', [1, 3, snapshot, policy[0], bytes(32, 9), 'small', bytes(0)], rejected(4));
  for (const [label, change] of [
    ['AppLength', p => { p[0] = bytes(31); }], ['RefLength', p => { p[1] = bytes(31); }],
    ['NoSlots', p => { p[2] = []; }], ['NoResources', p => { p[3] = []; }],
    ['SlotsOrder', p => { p[2].reverse(); }], ['SlotsDuplicate', p => { p[2][1] = p[2][0]; }],
    ['ResourcesOrder', p => { p[3].reverse(); }], ['ResourcesDuplicate', p => { p[3][1][0] = p[3][0][0]; }],
    ['UnusedSlot', p => { p[2].push('unused'); }], ['UnknownSlot', p => { p[3][0][1] = 'missing'; }],
    ['ZeroMaximum', p => { p[3][0][3] = 0; }], ['OverMaximum', p => { p[3][0][3] = 1048577; }],
    ['EmptyContext', p => { p[3][0][2] = ''; }], ['DotContext', p => { p[3][0][2] = '.bad'; }],
    ['DoubleDot', p => { p[3][0][2] = 'bad..context'; }], ['UnicodeContext', p => { p[3][0][2] = 'café'; }],
    ['ResourceSlash', p => { p[3][0][0] = 'bad/name'; }], ['SlotSlash', p => { p[2][0] = 'bad/name'; }],
  ]) {
    const altered = clone(policy); change(altered);
    add('BadPolicy' + label, [1, 0, altered], rejected(0));
  }
  for (const [label, change] of [
    ['Missing', b => { b.pop(); }], ['Extra', b => { b.push(clone(b[0])); }],
    ['Order', b => { b.reverse(); }], ['Application', b => { b[0][0] = bytes(32, 9); }],
    ['Policy', b => { b[0][1] = bytes(32, 9); }], ['Slot', b => { b[0][2] = 'changed'; }],
    ['KeyLength', b => { b[0][3] = bytes(64); }], ['KeyPrefix', b => { b[0][3][0] = 3; }],
    ['Principal', b => { b[0][4] = 'sha256:' + 'g'.repeat(64); }],
    ['DuplicateKey', b => { b[1][3] = clone(b[0][3]); }], ['DuplicatePrincipal', b => { b[1][4] = b[0][4]; }],
  ]) {
    const altered = clone(bindings); change(altered);
    add('BadBindings' + label, [1, 1, policy, [0], altered], rejected(5));
    add('BadSnapshot' + label, [1, 2, policy, [1, [policy, altered]]], rejected(1));
    add('BadExisting' + label, [1, 1, policy, [1, [policy, altered]], bindings], rejected(1));
  }
  const valid = encode([1, 0, policy]);
  raw('Trailing', [...valid, 0], malformed(8));
  raw('EmptyWire', [], malformed(2));
  raw('Indefinite', [0x9f, 1, 0, 0xff], malformed(4));
  raw('WrongRoot', [0], malformed(3));
  raw('NonminimalRoot', [0x98, 3, ...valid.slice(1)], malformed(5));
  add('WrongVersion', [2, 0, policy], malformed(3));
  add('UnknownOperation', [1, 4, policy], malformed(3));
  add('WrongArity', [1, 0, policy, 0], malformed(3));
  add('WrongOption', [1, 2, policy, [2]], malformed(3));
  const truncated = [1, 2, 3, 4, valid.length - 1];
  for (const length of truncated) raw('Truncated' + length, valid.slice(0, length), malformed(2));
  assert.equal(new Set(rows.map(row => row.id)).size, rows.length);
  return rows;
}

export function maximumCorpus() {
  const p = [bytes(32), bytes(32, 2), [], []], b = [];
  for (let index = 0; index < 64; index++) {
    const name = 's'.repeat(127) + '-.0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz'[index];
    p[2].push(name); p[3].push([name, name, 'c'.repeat(128), 1048576]);
    const key = bytes(65, index + 1); key[0] = 4;
    b.push([p[0], p[1], name, key, 'sha256:' + index.toString(16).padStart(64, '0')]);
  }
  const snapshot = [p, b], rows = [
    vector('MaximumPolicy', [1, 0, p], success(0, p)),
    vector('MaximumInitialize', [1, 1, p, [0], b], success(1, snapshot)),
    vector('MaximumOpen', [1, 2, p, [1, snapshot]], success(1, snapshot)),
    vector('MaximumSign', [1, 3, snapshot, p[0], p[1], p[3][63][0], bytes(1048576)], success(2, [...p[3][63], b[63][3], b[63][4]])),
    vector('PayloadPlusOne', [1, 3, snapshot, p[0], p[1], p[3][63][0], bytes(1048577)], malformed(6)),
  ];
  for (const [id, change] of [
    ['TextPlusOne', x => { x[3][0][2] += 'c'; }],
    ['SlotsPlusOne', x => { x[2].push('z'); }],
    ['ResourcesPlusOne', x => { x[3].push(['z', x[2][0], 'z', 1]); }],
  ]) {
    const modified = clone(p); change(modified);
    rows.push(vector(id, [1, 0, modified], malformed(6)));
  }
  const full = bytes(2097152, 0); full.set(encode([1, 0, policy]));
  rows.push({id: 'FrameMaximum', request: full, response: encode(malformed(8))});
  rows.push({id: 'FramePlusOne', request: bytes(2097153), response: encode(malformed(6)), nativeOnly: true});
  return rows;
}
