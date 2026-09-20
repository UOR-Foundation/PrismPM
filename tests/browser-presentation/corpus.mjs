import assert from 'node:assert/strict';
import {encodeWire, decodePresentation, decodeIntent} from '../../sdk/browser/presentation-wire.mjs';
import {combinedShape, maximumShape, maximumCases} from './maximum-fixtures.mjs';

export const failure = code => Uint8Array.of(0x83, 1, 1, code);
export const view = (nodes = [], options = {}) => [1, options.revision ?? 1,
  options.phase ?? 0, options.status ?? 0, options.live ?? 0, options.focus ?? 0, nodes];
// Fixture convenience only: all encoded field records include an explicit epoch.
export const node = (content, parent = 0) => [parent,
  [5, 6, 7].includes(content[0]) && content.length === 6 ? [...content, 0] : content];
export function basic() {
  return view([node([0, 0]), node([3, 1, 1], 1), node([4, 'Plain <script>text</script>\ufeff'], 1),
    node([2, 2], 1), node([5, 3, true, true, 32, 'initial'], 4),
    node([6, 4, true, false, 64, 'line\r\nbreak'], 4),
    node([7, 5, true, true, 1, [[1, 6], [4294967295, 7]]], 4),
    node([8, 8, 1, true, true, [5, 6, 7]], 4),
    node([9, 9, [10, 11], [['a', 'b'], ['c', 'd']]], 1),
    node([1, 12], 1)], {status: 14, live: 1, focus: 5});
}

export function corpus() {
  const rows = [], add = (id, request, response = request) => rows.push({id, request, response});
  const accepted = (id, value) => { const bytes = encodeWire(value); (value.length === 7 ? decodePresentation : decodeIntent)(bytes); add(id, bytes); };
  const semantic = (id, value) => { const bytes = encodeWire(value); assert.throws(() => (value.length === 7 ? decodePresentation : decodeIntent)(bytes)); add(id, bytes, failure(9)); };
  accepted('Empty', view()); accepted('EveryNode', basic());
  for (let phase = 0; phase < 4; phase++) for (let live = 0; live < 3; live++) accepted('Phase' + phase + 'Live' + live, view([node([4, 'status'])], {phase, live}));
  for (const value of ['', '\ufeff', '\u0000', 'é', '𐀀', '\r\n']) accepted('Text' + rows.length, view([node([4, value])]));
  for (const number of [0, 23, 24, 255, 256, 65535, 65536, 4294967295]) accepted('Revision' + number, view([], {revision: number}));
  accepted('IntentText', [1, 4294967295, 4294967295, [[1, '\ufeffexact'], [256, '']]]);
  accepted('IntentChoices', [1, 1, 1, [[1, 0], [2, 4294967295]]]);
  accepted('IntentEmpty', [1, 0, 1, []]);
  const epochs = basic();
  for (const row of epochs[6]) if ([5, 6, 7].includes(row[1][0])) row[1][6] = 4294967295;
  accepted('DraftEpochMaximum', epochs);
  const secret = view([node([2, 0]), node([10, 0, true, true, 67108864, 4294967295], 1),
    node([8, 0, 1, true, true, [2]], 1)]);
  accepted('SecretInputMaximum', secret);
  const optional = structuredClone(secret); optional[6][1][1][3] = false;
  accepted('SecretInputOptional', optional);
  for (const [id, mutate] of [
    ['SecretZeroMaximum', x => { x[6][1][1][4] = 0; }],
    ['SecretOverMaximum', x => { x[6][1][1][4] = 67108865; }],
    ['SecretOutsideForm', x => { x[6][1][0] = 0; }],
    ['SecretDisabledBinding', x => { x[6][1][1][2] = false; }],
    ['SecretEnabledPending', x => { x[2] = 1; }],
  ]) { const value = structuredClone(secret); mutate(value); semantic(id, value); }
  for (let phase = 1; phase <= 3; phase++) {
    const value = structuredClone(secret); value[2] = phase;
    value[6][1][1][2] = false; value[6][2][1][3] = false;
    accepted('SecretDisabledPhase' + phase, value);
  }
  const suppliedDefault = structuredClone(secret); suppliedDefault[6][1][1].splice(5, 0, 'forbidden');
  add('SecretSuppliedDefault', encodeWire(suppliedDefault), failure(3));
  for (const [id, update] of [
    ['BadPhase', x => { x[2] = 4; }], ['BadStatus', x => { x[3] = 257; }],
    ['BadLive', x => { x[4] = 3; }], ['MissingFocus', x => { x[5] = 256; }],
    ['ClosedFocus', x => { x[2] = 3; x[5] = 1; }],
    ['ForwardParent', x => { x[6][0][0] = 2; }], ['SelfParent', x => { x[6][0][0] = 1; }],
    ['LeafParent', x => { x[6][3][0] = 2; }], ['NestedForm', x => { x[6].push(node([2, 0], 4)); }],
    ['BadHeading', x => { x[6][1][1][1] = 7; }], ['BadLabel', x => { x[6][0][1][1] = 256; }],
    ['FieldOutsideForm', x => { x[6][4][0] = 1; }], ['EnabledPending', x => { x[2] = 1; }],
    ['DefaultOversize', x => { x[6][4][1][4] = 1; }], ['ZeroFieldMaximum', x => { x[6][4][1][4] = 0; }],
    ['MissingSelected', x => { x[6][6][1][4] = 42; }], ['DuplicateOption', x => { x[6][6][1][5][1][0] = 1; }],
    ['DuplicateAction', x => { x[6].push(node([8, 0, 1, true, false, []], 4)); }],
    ['DuplicateDefault', x => { x[6].push(node([8, 0, 2, true, true, []], 4)); }],
    ['UnorderedBinding', x => { x[6][7][1][5] = [6, 5]; }], ['MissingBinding', x => { x[6][7][1][5] = [255]; }],
    ['WrongBindingKind', x => { x[6][7][1][5] = [3]; }], ['DisabledBinding', x => { x[6][4][1][2] = false; }],
    ['RaggedTable', x => { x[6][8][1][3][0].pop(); }], ['NoColumns', x => { x[6][8][1][2] = []; x[6][8][1][3] = []; }],
  ]) { const value = basic(); update(value); semantic(id, value); }
  semantic('IntentZeroAction', [1, 0, 0, []]);
  semantic('IntentDuplicateField', [1, 0, 1, [[1, 'a'], [1, 'b']]]);
  semantic('IntentZeroField', [1, 0, 1, [[0, 'a']]]);
  add('SecretMissingFields', encodeWire(view([node([10, 0])])), failure(3));
  const missingEpoch = basic(); missingEpoch[6][4][1].pop();
  add('MissingDraftEpoch', encodeWire(missingEpoch), failure(3));
  const wrongEpoch = basic(); wrongEpoch[6][4][1][6] = true;
  add('WrongDraftEpochType', encodeWire(wrongEpoch), failure(3));
  const epochZero = encodeWire(view([node([2, 0]), node([5, 0, true, false, 1, ''], 1)]));
  const epochOver = new Uint8Array(epochZero.length + 8);
  epochOver.set(epochZero.subarray(0, epochZero.length - 1));
  epochOver.set([0x1b, 0, 0, 0, 1, 0, 0, 0, 0], epochZero.length - 1);
  add('DraftEpochUint64', epochOver, failure(4));
  add('UnsupportedNode', encodeWire(view([node([11, 0])])), failure(3));
  add('Version', encodeWire([2, 0, 0, 0, 0, 0, []]), failure(3));
  add('Trailing', Uint8Array.from([...encodeWire(view()), 0]), failure(8));
  add('Noncanonical', Uint8Array.of(0x98, 7, 1, 0, 0, 0, 0, 0, 0x80), failure(5));
  add('InvalidUtf8', Uint8Array.of(0x87, 1, 0, 0, 0, 0, 0, 0x81, 0x82, 0, 0x82, 4, 0x61, 0xff), failure(7));
  add('WrongTop', Uint8Array.of(0), failure(3));
  for (const [name, prefix] of [['Map', 0xa0], ['Tag', 0xc0], ['Indefinite', 0x9f], ['Negative', 0x20], ['Float', 0xfa]]) add(name, Uint8Array.of(prefix), failure(prefix === 0x9f ? 4 : 3));
  const minimal = encodeWire(view());
  for (let at = 0; at < minimal.length; at++) add('Truncated' + at, minimal.slice(0, at), failure(2));
  return rows;
}

export function boundaries() {
  const rows = [], add = (id, value, error) => { const request = encodeWire(value); rows.push({id, request, response: error === undefined ? request : failure(error)}); };
  add('NodesMaximum', view(Array.from({length: 256}, () => node([4, '']))));
  add('NodesOver', view(Array.from({length: 257}, () => node([4, '']))), 6);
  add('DepthMaximum', view(Array.from({length: 16}, (_, i) => node([0, 0], i))));
  add('DepthOver', view(Array.from({length: 17}, (_, i) => node([0, 0], i))), 9);
  add('ActionsMaximum', view([node([2, 0]), ...Array.from({length: 64}, (_, i) => node([8, 0, i + 1, true, false, []], 1))]));
  add('ActionsOver', view([node([2, 0]), ...Array.from({length: 65}, (_, i) => node([8, 0, i + 1, true, false, []], 1))]), 9);
  const fields = [node([2, 0]), ...Array.from({length: 16}, () => node([5, 0, true, false, 1, ''], 1))];
  add('BindingsMaximum', view([...fields, node([8, 0, 1, true, false, fields.slice(1).map((_, i) => i + 2)], 1)]));
  add('BindingsOver', view([...fields, node([8, 0, 1, true, false, Array.from({length: 17}, (_, i) => i + 2)], 1)]), 6);
  const secretFields = [node([2, 0]), ...Array.from({length: 16}, () => node([10, 0, true, false, 67108864, 4294967295], 1))];
  add('SecretBindingsMaximum', view([...secretFields, node([8, 0, 1, true, true, Array.from({length: 16}, (_, i) => i + 2)], 1)]));
  add('ChoicesMaximum', view([node([2, 0]), node([7, 0, true, false, 256, Array.from({length: 256}, (_, i) => [i + 1, 255])], 1)]));
  add('ChoicesOver', view([node([2, 0]), node([7, 0, true, false, 0, Array.from({length: 256}, (_, i) => [i + 1, 0])], 1), node([7, 0, true, false, 0, [[1, 0]]], 1)]), 9);
  add('CellsMaximum', view([node([9, 0, Array(16).fill(0), Array.from({length: 256}, () => Array(16).fill(''))])]));
  add('RowsMaximum', view([node([9, 0, [0], Array.from({length: 4096}, () => [''])])]));
  for (const count of [15, 16, 17, 255, 256, 257, 4095]) {
    add('RowsBoundary' + count, view([node([9, 0, [0], Array.from({length: count}, () => ['row'])])]));
  }
  add('CellsOver', view([node([9, 0, Array(16).fill(0), Array.from({length: 257}, () => Array(16).fill(''))])]), 9);
  add('ColumnsOver', view([node([9, 0, Array(17).fill(0), []])]), 6);
  add('IntentFieldsMaximum', [1, 1, 1, Array.from({length: 16}, (_, i) => [i + 1, ''])]);
  add('IntentFieldsOver', [1, 1, 1, Array.from({length: 17}, (_, i) => [i + 1, ''])], 6);
  for (const rowMaximum of [false, true]) {
    add(rowMaximum ? 'CombinedRowsMaximum' : 'CombinedMaximum', combinedShape(rowMaximum));
  }
  // Every per-list limit is respected; only aggregate structural fuel rejects.
  // Encode each modest table independently, not with a disabled host budget.
  const table = encodeWire(node([9, 0, Array(16).fill(0), Array.from({length: 256}, () => Array(16).fill(''))]));
  const bomb = new Uint8Array(8 + 8 * table.length);
  bomb.set([0x87, 1, 1, 0, 0, 0, 0, 0x88]);
  for (let i = 0; i < 8; i++) bomb.set(table, 8 + i * table.length);
  rows.push({id: 'AggregateFuelOver', request: bomb, response: failure(6)});
  return rows;
}

export function maximumCorpus() {
  // A four-byte UTF-8 string-length head at this size has five bytes total.
  const fixed = encodeWire(view([node([4, ''])])).length + 4;
  const request = encodeWire(view([node([4, 'x'.repeat(67108864 - fixed)])]));
  assert.equal(request.length, 67108864);
  const over = new Uint8Array(67108865); over.set(request);
  return [{id: 'FrameMaximum', request, response: request}, {id: 'FrameOver', request: over, response: failure(6)}];
}

export function* combinedMaximumCorpus() {
  for (const id of maximumCases.slice(1)) {
    const shape = maximumShape(id), fixed = encodeWire(shape.frame).length + 4;
    shape.setText('x'.repeat(67108864 - fixed));
    const request = encodeWire(shape.frame);
    assert.equal(request.length, 67108864, id);
    decodePresentation(request);
    yield {id, request, response: request};
  }
}

// Independent expected synthetic fixture. Never an application secret or a
// runtime fallback: the owning gate compares this to the generated source.
export function secretMaximumResponse() {
  return encodeWire([1, 2, 0, 13, 1, 0, [
    [0, [0, 0]], [1, [1, 8]], [1, [3, 2, 5]],
    [1, [4, '\uFEFFdynamic <script>globalThis.presentationInjected=2</script> 😀']],
    [1, [2, 4]], [5, [10, 6, true, true, 67108864, 1]],
    [5, [6, 7, true, false, 128, '\uFEFFfirst\r\nsecond', 0]],
    [5, [7, 2, true, true, 10, [[10, 9], [20, 10]], 0]],
    [5, [8, 3, 101, true, false, []]], [5, [8, 11, 202, true, true, [6, 7, 8]]],
    [1, [9, 1, [13], [['<svg onload="globalThis.presentationInjected=3">'], ['\uFEFFtable 😀']]]],
  ]]);
}

export function* secretMaximumCorpus() {
  for (const length of [67108864, 67108865]) yield {id: 'SecretRaw' + length,
    mode: 'maxfield', request: new Uint8Array(length).fill(120), response: Uint8Array.of(length === 67108864 ? 245 : 244)};
  const fields = text => [1, 1, 202, [[6, text], [7, ''], [8, 10]]];
  const textLength = 67108864 - encodeWire(fields('')).length - 4;
  const request = encodeWire(fields('x'.repeat(textLength)));
  assert.equal(request.length, 67108864); decodeIntent(request);
  yield {id: 'SecretFramedWireMaximum', mode: 'wasm', request, response: request};
  yield {id: 'SecretFramedRouteMaximum', mode: 'maxroute', request, response: Uint8Array.of(245)};
  yield {id: 'SecretFramedSinkMaximum', mode: 'maxsink', request, response: secretMaximumResponse()};
}

export function* progressMaximumCorpus() {
  const shape = maximumShape('FrameNodesMaximumLast'); shape.frame[2] = 1;
  const textLength = 67108864 - encodeWire(shape.frame).length - 4;
  shape.setText('x'.repeat(textLength));
  let request;
  for (const revision of [2, 3]) {
    shape.frame[1] = revision; request = encodeWire(shape.frame);
    assert.equal(request.length, 67108864); decodePresentation(request);
    yield {id: 'ProgressFrame' + revision, mode: 'wasm', request, response: request};
    yield {id: 'ProgressPair' + revision, mode: 'maxprogress', request, response: Uint8Array.of(revision === 3 ? 245 : 244)};
  }
  const over = new Uint8Array(67108865); over.set(request);
  // Native checks the source guard; the unchanged Wasm ABI rejects allocation.
  yield {id: 'ProgressFrameOver', mode: 'maxprogress', allocationReject: true,
    request: over, response: Uint8Array.of(244)};
}
