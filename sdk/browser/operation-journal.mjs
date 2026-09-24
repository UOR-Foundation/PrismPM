// SDK-private durable admission; not a public runtime or distributed receipt.
import {bytesCopy, bytesLength, randomBytes, verifyBytes} from './identity.mjs';
import {openStore} from './store.mjs';
import {openStagedEffects} from './effects.mjs';
import {decodeEffectWire as decode, encodeEffectWire as encode} from './effects-wire.mjs';
import {inspectEffectModule, inspectEffectArtifactBudget, EFFECT_ARTIFACTS_MAXIMUM} from './effects-module.mjs';
import {credentialPublicBindings, signCredential} from './credential-custody.mjs';

const FRAME = 67108864, CHUNK = 1048576, METADATA = 65536;
const CONTEXT = 'prismpm/browser-operation-journal/1';
const LIMITS = Object.freeze({maxObjectBytes: CHUNK, maxObjects: 4096, maxHeads: 2});
const same = (a, b) => a.length === b.length && a.every((value, index) => value === b[index]);
const equal = (a, b) => same(encode(a), encode(b));
const id = bytes => 'sha256:' + Array.from(bytes, byte => byte.toString(16).padStart(2, '0')).join('');
const hash = async bytes => new Uint8Array(await crypto.subtle.digest('SHA-256', bytes));
const utf8 = new TextEncoder();
const credentialSnapshot = bindings => [bindings.application, bindings.policy,
  bindings.resources.map(row => [row.resource, row.slot, row.context, row.maximum, row.publicKey, row.principal])];
function compareResource(left, right) {
  const a = utf8.encode(left), b = utf8.encode(right);
  for (let index = 0; index < Math.min(a.length, b.length); index++) if (a[index] !== b[index]) return a[index] - b[index];
  return a.length - b.length;
}
async function artifactClosure(bytes, binding, wire, partition) {
  const value = decode(bytes);
  if (!Array.isArray(value) || value.length !== 5 || value[0] !== 1
    || value.slice(1, 4).some(digest => !(digest instanceof Uint8Array) || digest.length !== 32)
    || !same(value[1], wire) || !same(value[2], partition) || !Array.isArray(value[4]) || value[4].length > 64
    || !same(bytes, encode(value))) throw fail('artifact-mismatch');
  let previous = null;
  for (const row of value[4]) {
    if (!Array.isArray(row) || row.length !== 2 || typeof row[0] !== 'string'
      || !row[0].isWellFormed() || utf8.encode(row[0]).length < 1 || utf8.encode(row[0]).length > 128
      || !(row[1] instanceof Uint8Array) || row[1].length !== 32
      || previous !== null && compareResource(previous, row[0]) >= 0) throw fail('artifact-mismatch');
    previous = row[0];
  }
  if (!same(await hash(bytes), binding[2])) throw fail('artifact-mismatch');
}

export class OperationJournalError extends Error {
  constructor(code) { super(code); this.name = 'OperationJournalError'; this.code = code; }
}
const fail = code => new OperationJournalError(code);
function exact(value, names) {
  if (typeof value !== 'object' || value === null || Object.getPrototypeOf(value) !== Object.prototype) throw fail('invalid-input');
  const descriptors = Object.getOwnPropertyDescriptors(value), keys = Reflect.ownKeys(descriptors);
  if (keys.length !== names.length || keys.some(key => typeof key !== 'string')
    || keys.sort().join(',') !== names.toSorted().join(',') || keys.some(key => !('value' in descriptors[key]))) throw fail('invalid-input');
  return Object.fromEntries(names.map(name => [name, descriptors[name].value]));
}
function captureArray(value, capture) {
  if (!Array.isArray(value) || Object.getPrototypeOf(value) !== Array.prototype) throw fail('invalid-input');
  const descriptors = Object.getOwnPropertyDescriptors(value);
  const length = descriptors.length?.value;
  if (!Number.isInteger(length) || length < 0 || length > 64
    || Reflect.ownKeys(descriptors).length !== length + 1) throw fail('invalid-input');
  return Array.from({length}, (_, index) => {
    if (!descriptors[index] || !('value' in descriptors[index])) throw fail('invalid-input');
    return capture(descriptors[index].value);
  });
}
function captureEffects(value, journalWire, partition) {
  value = exact(value, ['wire', 'wireDigest', 'manifest', 'guests', 'signers']);
  const guests = captureArray(value.guests, row => {
    row = exact(row, ['resource', 'bytes']);
    if (typeof row.resource !== 'string') throw fail('invalid-input');
    return {resource: row.resource, bytes: row.bytes};
  });
  const total = inspectEffectArtifactBudget(value.wire, guests.map(row => row.bytes));
  if (total + bytesLength(journalWire, FRAME) + bytesLength(partition, FRAME) > EFFECT_ARTIFACTS_MAXIMUM) throw fail('invalid-input');
  // The combined closure is bounded before the first artifact copy or await.
  return {
    wire: bytesCopy(value.wire, FRAME), wireDigest: bytesCopy(value.wireDigest, 32), manifest: bytesCopy(value.manifest, FRAME),
    guests: guests.map(row => ({resource: row.resource, bytes: bytesCopy(row.bytes, FRAME)})),
    signers: captureArray(value.signers, row => {
      row = exact(row, ['resource', 'custody']);
      if (typeof row.resource !== 'string') throw fail('invalid-input');
      return {resource: row.resource, custody: row.custody};
    }),
  };
}
async function guest(bytes, expected) {
  if (expected.length !== 32 || !same(await hash(bytes), expected)) throw fail('artifact-mismatch');
  try { inspectEffectModule(bytes, 16384); } catch { throw fail('invalid-generated-module'); }
  const module = await WebAssembly.compile(bytes);
  return input => {
    if (input.length > FRAME) throw fail('invalid-input');
    try {
      const instance = new WebAssembly.Instance(module, {}), {memory, holo_alloc: allocate, holo_run: run} = instance.exports;
      if (!(memory instanceof WebAssembly.Memory) || typeof allocate !== 'function' || typeof run !== 'function'
        || allocate.length !== 1 || run.length !== 2 || memory.buffer.byteLength > 1073741824) throw fail('invalid-generated-module');
      const at = allocate(input.length) >>> 0;
      if (at + input.length > memory.buffer.byteLength || memory.buffer.byteLength > 1073741824) throw fail('invalid-generated-output');
      new Uint8Array(memory.buffer, at, input.length).set(input);
      const returned = run(at, input.length);
      if (typeof returned !== 'bigint') throw fail('invalid-generated-output');
      const packed = BigInt.asUintN(64, returned), start = Number(packed >> 32n), length = Number(packed & 0xffffffffn);
      if (length > METADATA || start + length > memory.buffer.byteLength || memory.buffer.byteLength > 1073741824) throw fail('invalid-generated-output');
      return new Uint8Array(memory.buffer, start, length).slice();
    } catch (error) { if (error instanceof OperationJournalError) throw error; throw fail('generated-execution-failed'); }
  };
}
function admitted(call, input) {
  let output;
  try { output = decode(call(input)); } catch (error) { if (error instanceof OperationJournalError) throw error; throw fail('invalid-generated-output'); }
  if (!Array.isArray(output) || output.length !== 3 || output[0] !== 1) throw fail('invalid-generated-output');
  if (output[1] === 1 || output[1] === 2) throw fail('model-rejected');
  if (output[1] !== 0) throw fail('invalid-generated-output');
  return output[2];
}

class PayloadStore {
  #wire; #partition; #binding; #store; #closed = false; #busy = false;
  constructor(wire, partition, binding, store) {
    this.#wire = wire; this.#partition = partition; this.#binding = binding; this.#store = store;
  }
  #check() { if (this.#closed) throw fail('journal-closed'); }
  #plan(bytes) {
    const plan = admitted(this.#partition, bytes);
    if (!Array.isArray(plan) || plan.length < 1 || plan.length > 64) throw fail('invalid-generated-output');
    let offset = 0;
    for (const row of plan) {
      if (!Array.isArray(row) || row.length !== 2 || row[0] !== offset || !Number.isSafeInteger(row[1])
        || row[1] !== Math.min(CHUNK, bytes.length - offset) || row[1] <= 0) throw fail('invalid-generated-output');
      offset += row[1];
    }
    if (offset !== bytes.length) throw fail('invalid-generated-output');
    return plan;
  }
  async load(value) {
    if (arguments.length !== 1) throw fail('invalid-input'); this.#check();
    const captured = bytesCopy(value, METADATA);
    const payload = admitted(this.#wire, encode([1, 6, decode(captured)]));
    if (!same(captured, encode(payload))) throw fail('payload-invalid');
    const [digest, length, chunks] = payload;
    if (!Number.isSafeInteger(length) || length < 1 || length > FRAME || !Array.isArray(chunks)
      || chunks.length !== Math.ceil(length / CHUNK)) throw fail('payload-invalid');
    const bytes = new Uint8Array(length);
    let offset = 0;
    for (const reference of chunks) {
      const chunk = await this.#store.readObject(id(reference)); this.#check();
      if (chunk === null || chunk.length !== Math.min(CHUNK, length - offset)) throw fail('chunk-missing');
      bytes.set(chunk, offset); offset += chunk.length;
    }
    if (offset !== length || !same(await hash(bytes), digest)) throw fail('payload-invalid');
    this.#check(); this.#plan(bytes);
    return bytes;
  }
  async stage(value, previous) {
    if (arguments.length !== 2) throw fail('invalid-input'); this.#check();
    if (this.#busy) throw fail('journal-busy');
    const bytes = bytesCopy(value, FRAME), predecessor = bytesCopy(previous, 32);
    if (predecessor.length !== 32) throw fail('invalid-input');
    this.#busy = true;
    try {
    const plan = this.#plan(bytes), chunks = [];
    for (const [at, size] of plan) chunks.push(await hash(bytes.subarray(at, at + size)));
    const payload = [await hash(bytes), bytes.length, chunks]; this.#check();
    let expected = (await this.#store.readHead(this.#binding[8]))?.id ?? null;
    const nonce = randomBytes(32);
    for (let start = 0; start < plan.length; start += 15) {
      this.#check();
      const objects = new Map();
      for (let index = start; index < Math.min(start + 15, plan.length); index++) {
        const [at, size] = plan[index]; objects.set(id(chunks[index]), bytes.slice(at, at + size));
      }
      const marker = encode([1, nonce, predecessor, payload, start]);
      const next = id(await hash(marker)); objects.set(next, marker); this.#check();
      const acknowledged = await this.#store.commit({head: this.#binding[8], expected, next, objects: [...objects.values()]});
      this.#check();
      if (acknowledged !== next) throw fail('publication-uncertain');
      expected = next;
    }
    // The staging head is progress, not a lease. Another complete staging
    // operation cannot invalidate immutable chunks; only the journal CAS can
    // release execution. Every acknowledged chunk must actually be readable.
    for (let index = 0; index < chunks.length; index++) {
      const chunk = await this.#store.readObject(id(chunks[index])); this.#check();
      const [at, size] = plan[index];
      if (chunk === null || !same(chunk, bytes.subarray(at, at + size))) throw fail('chunk-missing');
    }
    return encode(payload);
    } finally { this.#busy = false; }
  }

  close() {
    if (arguments.length !== 0) throw fail('invalid-input');
    if (this.#closed) return; this.#closed = true; this.#store.close();
  }
}

class Journal {
  #wire; #transport; #binding; #manifest; #custody; #resource; #store; #effects;
  #state; #session = randomBytes(32); #closed = false; #busy = false; #blocked = false; #runtimeClosed = false; #receipt = null;
  constructor(wire, transport, binding, manifest, custody, resource, store, effects) {
    this.#wire = wire; this.#transport = transport; this.#binding = binding; this.#manifest = manifest;
    this.#custody = custody; this.#resource = resource; this.#store = store; this.#effects = effects;
  }
  #check() { if (this.#closed) throw fail('journal-closed'); }
  #step(input) { return admitted(this.#wire, encode(input)); }
  #record(value) {
    const checked = this.#step([1, 2, value]);
    if (!equal(checked, value) || !equal(value[0], this.#binding)) throw fail('binding-mismatch');
    return checked;
  }
  #request(record, bytes) {
    if (this.#step([1, 4, record, decode(bytes), this.#manifest]) !== true) throw fail('request-mismatch');
  }
  #result(record, request, response) {
    if (this.#step([1, 5, record, decode(request), decode(response), this.#manifest]) !== true) throw fail('result-mismatch');
  }
  async #signed(body) {
    this.#check();
    const bytes = encode(body);
    if (bytes.length > METADATA) throw fail('record-limit');
    const signature = await signCredential(this.#custody, this.#resource, bytes);
    this.#check();
    if (!await verifyBytes(this.#binding[5], CONTEXT, bytes, signature)) throw fail('signature-invalid');
    this.#check();
    const envelope = encode([1, bytes, signature]);
    if (envelope.length > METADATA) throw fail('record-limit');
    return {bytes: envelope, digest: await hash(envelope)};
  }
  async #readSigned(identifier) {
    const bytes = await this.#store.readObject(identifier); this.#check();
    if (bytes === null || bytes.length > METADATA) throw fail('record-missing');
    const envelope = decode(bytes);
    if (!Array.isArray(envelope) || envelope.length !== 3 || envelope[0] !== 1
      || !(envelope[1] instanceof Uint8Array) || !(envelope[2] instanceof Uint8Array)
      || envelope[1].length > METADATA || envelope[2].length !== 64
      || !same(bytes, encode(envelope))
      || !await verifyBytes(this.#binding[5], CONTEXT, envelope[1], envelope[2])) throw fail('signature-invalid');
    this.#check();
    const body = decode(envelope[1]);
    if (!Array.isArray(body) || body.length !== 3 || body[0] !== 1
      || ![0, 1].includes(body[1]) || !same(envelope[1], encode(body))) throw fail('record-invalid');
    if (body[1] === 0) {
      if (!equal(this.#step([1, 3, body[2]]), this.#binding)) throw fail('binding-mismatch');
    } else this.#record(body[2]);
    return {body, digest: await hash(bytes)};
  }
  async #loadPayload(payload) { return this.#transport.load(encode(payload)); }
  async #replay() {
    const current = await this.#store.readHead(this.#binding[7]); this.#check();
    if (current === null) throw fail('journal-missing');
    const chain = [], seen = new Set(); let next = current.id, genesis;
    for (let count = 0; count <= this.#binding[9]; count++) {
      if (seen.has(next)) throw fail('record-invalid'); seen.add(next);
      const signed = await this.#readSigned(next);
      if (signed.body[1] === 0) { genesis = signed; break; }
      chain.push(signed); next = id(signed.body[2][2]);
    }
    if (!genesis) throw fail('history-limit');
    let state = this.#step([1, 0, this.#binding, genesis.digest]), receipt = null;
    for (const signed of chain.reverse()) {
      const record = signed.body[2], request = await this.#loadPayload(record[10]);
      this.#request(record, request);
      if (record[9] !== 0) {
        const response = await this.#loadPayload(record[12][1]); this.#result(record, request, response);
        receipt = {operation: record[4], request, response};
      }
      state = this.#step([1, 1, state, record, signed.digest]);
    }
    if (!Array.isArray(state) || state.length !== 5 || !equal(state[0], this.#binding)
      || id(state[3]) !== current.id) throw fail('invalid-generated-output');
    this.#state = state; this.#receipt = receipt; this.#blocked = state[4][0] === 1;
  }
  async initialize() {
    const signed = await this.#signed([1, 0, this.#binding]); this.#check();
    const next = id(signed.digest);
    const acknowledged = await this.#store.commit({head: this.#binding[7], expected: null, next, objects: [signed.bytes]});
    this.#check();
    if (acknowledged !== next) throw fail('publication-uncertain');
    this.#state = this.#step([1, 0, this.#binding, signed.digest]);
  }
  async open() { await this.#replay(); }
  async #stage(bytes) { return decode(await this.#transport.stage(bytes, this.#state[3])); }
  async #publish(record) {
    this.#record(record);
    const signed = await this.#signed([1, 1, record]); this.#check();
    const state = this.#step([1, 1, this.#state, record, signed.digest]);
    const next = id(signed.digest);
    const acknowledged = await this.#store.commit({head: this.#binding[7], expected: id(this.#state[3]), next, objects: [signed.bytes]});
    this.#check();
    if (acknowledged !== next) throw fail('publication-uncertain');
    this.#state = state;
  }
  async submit(value) {
    if (arguments.length !== 1) throw fail('invalid-input');
    this.#check();
    if (this.#busy) throw fail('journal-busy');
    if (this.#blocked || this.#state[4][0] !== 0) throw fail('journal-uncertain');
    if (this.#runtimeClosed) throw fail('journal-reopen-required');
    const intent = bytesCopy(value, FRAME); this.#busy = true;
    try {
      const staged = this.#effects.prepare(intent), request = bytesCopy(staged.request, FRAME), decoded = decode(request);
      const payload = await this.#stage(request); this.#check();
      const prepared = [this.#binding, this.#state[1] + 1, this.#state[3], this.#session,
        this.#state[2], decoded[2], decoded[3], decoded[4], decoded[5][0], 0, payload, 9, [0]];
      this.#request(prepared, request);
      await this.#publish(prepared); this.#check();
      const response = bytesCopy(await staged.release(), FRAME); this.#check();
      const actual = decode(response);
      if (!Array.isArray(actual) || actual.length !== 2 || !Number.isInteger(actual[0]) || actual[0] < 0 || actual[0] > 8) throw fail('effect-uncertain');
      const terminal = [...prepared];
      terminal[1] = this.#state[1] + 1; terminal[2] = this.#state[3]; terminal[9] = actual[0] === 8 ? 2 : 1;
      terminal[11] = actual[0]; terminal[12] = [1, await this.#stage(response)]; this.#check();
      this.#result(terminal, request, response);
      await this.#publish(terminal); this.#check();
      this.#receipt = {operation: prepared[4], request, response};
      return response.slice();
    } catch (error) {
      // Even an acknowledgment failure before primitive execution cannot be
      // converted into permission to retry. Only a later authenticated replay
      // may observe an already durable terminal receipt.
      this.#blocked = true; this.#runtimeClosed = true;
      try { this.#effects.close(); } catch { /* Unknown execution remains blocked. */ }
      if (error instanceof OperationJournalError) throw error;
      throw fail('journal-uncertain');
    } finally { this.#busy = false; }
  }
  async refresh() {
    if (arguments.length !== 0) throw fail('invalid-input');
    this.#check(); if (this.#busy) throw fail('journal-busy');
    this.#busy = true;
    try { await this.#replay(); return this.status(); }
    catch (error) { this.#blocked = true; throw error instanceof OperationJournalError ? error : fail('journal-uncertain'); }
    finally { this.#busy = false; }
  }
  receipt() {
    if (arguments.length !== 0) throw fail('invalid-input'); this.#check();
    if (this.#blocked || this.#state[4][0] !== 0 || this.#receipt === null) return null;
    return Object.freeze({operation: this.#receipt.operation,
      request: this.#receipt.request.slice(), response: this.#receipt.response.slice()});
  }
  status() {
    if (arguments.length !== 0) throw fail('invalid-input');
    return Object.freeze({closed: this.#closed, busy: this.#busy, blocked: this.#blocked, runtimeClosed: this.#runtimeClosed,
      records: this.#state?.[1] ?? 0, nextOperation: this.#state?.[2] ?? 0,
      pendingOperation: this.#state?.[4]?.[0] === 1 ? this.#state[4][1][4] : null});
  }
  close() {
    if (arguments.length !== 0) throw fail('invalid-input');
    if (this.#closed) return; this.#closed = true;
    try { this.#effects.close(); } finally { this.#transport.close(); }
  }
}

export async function openOperationJournal(options) {
  let store, effects;
  try {
    if (arguments.length !== 1) throw fail('invalid-input');
    options = exact(options, ['wire', 'wireDigest', 'partition', 'partitionDigest', 'binding', 'custody', 'signingResource', 'effects', 'mode']);
    const selectedEffects = captureEffects(options.effects, options.wire, options.partition);
    const wireBytes = bytesCopy(options.wire, FRAME), wireDigest = bytesCopy(options.wireDigest, 32);
    const partitionBytes = bytesCopy(options.partition, FRAME), partitionDigest = bytesCopy(options.partitionDigest, 32);
    const bindingBytes = bytesCopy(options.binding, METADATA), custody = options.custody;
    const signingResource = options.signingResource, mode = options.mode;
    if (!['initialize', 'open'].includes(mode) || typeof signingResource !== 'string') throw fail('invalid-input');
    const wire = await guest(wireBytes, wireDigest), partition = await guest(partitionBytes, partitionDigest);
    const binding = admitted(wire, encode([1, 3, decode(bindingBytes)]));
    if (!Array.isArray(binding) || binding.length !== 10 || !same(bindingBytes, encode(binding))) throw fail('binding-mismatch');
    const publicBindings = credentialPublicBindings(custody), matches = publicBindings.resources.filter(row => row.resource === signingResource);
    if (!same(publicBindings.application, binding[0]) || !same(publicBindings.policy, binding[3]) || matches.length !== 1
      || matches[0].context !== CONTEXT || matches[0].maximum !== METADATA || !same(matches[0].publicKey, binding[5])) throw fail('credential-mismatch');
    const manifest = decode(selectedEffects.manifest);
    if (!Array.isArray(manifest) || !same(manifest[0], binding[0]) || !same(manifest[1], binding[1])
      || !same(await hash(encode([1, manifest, credentialSnapshot(publicBindings)])), binding[4])) throw fail('binding-mismatch');
    if (!Array.isArray(manifest[3]) || manifest[3].some(row => Array.isArray(row) && Array.isArray(row[1])
      && row[1][0] === 5 && Array.isArray(row[1][1]) && row[1][1][0] === binding[6])) throw fail('storage-alias');
    if (manifest[3].some(row => Array.isArray(row) && Array.isArray(row[1]) && row[1][0] === 3
      && Array.isArray(row[1][1]) && row[1][1][0] instanceof Uint8Array
      && same(row[1][1][0], binding[5]) && row[1][1][1] === CONTEXT)) throw fail('signing-alias');
    for (const signer of selectedEffects.signers) {
      const bindings = credentialPublicBindings(signer.custody);
      if (!equal(credentialSnapshot(bindings), credentialSnapshot(publicBindings))) throw fail('credential-mismatch');
    }
    const guests = [];
    for (const row of selectedEffects.guests) guests.push([row.resource, await hash(row.bytes)]);
    guests.sort((left, right) => compareResource(left[0], right[0]));
    const closure = encode([1, wireDigest, partitionDigest, await hash(selectedEffects.wire), guests]);
    await artifactClosure(closure, binding, wireDigest, partitionDigest);
    effects = await openStagedEffects(selectedEffects);
    store = await openStore(binding[6], LIMITS);
    const transport = new PayloadStore(wire, partition, binding, store);
    const journal = new Journal(wire, transport, binding, manifest, custody, signingResource, store, effects);
    if (mode === 'initialize') await journal.initialize(); else await journal.open();
    return Object.freeze({submit: journal.submit.bind(journal), refresh: journal.refresh.bind(journal),
      receipt: journal.receipt.bind(journal), status: journal.status.bind(journal), close: journal.close.bind(journal)});
  } catch (error) {
    try { effects?.close(); } finally { store?.close(); }
    if (error instanceof OperationJournalError) throw error;
    throw fail('journal-unavailable');
  }
}

// Private payload transport acceptance surface. It grants no primitive
// execution or receipt authority and is not exported by a product runtime.
export async function openOperationPayloadStore(options) {
  let store;
  try {
    if (arguments.length !== 1) throw fail('invalid-input');
    options = exact(options, ['wire', 'wireDigest', 'partition', 'partitionDigest', 'binding', 'artifacts']);
    inspectEffectArtifactBudget(options.wire, [options.partition]);
    const wireBytes = bytesCopy(options.wire, FRAME), wireDigest = bytesCopy(options.wireDigest, 32);
    const partitionBytes = bytesCopy(options.partition, FRAME), partitionDigest = bytesCopy(options.partitionDigest, 32);
    const bindingBytes = bytesCopy(options.binding, METADATA), artifacts = bytesCopy(options.artifacts, METADATA);
    const wire = await guest(wireBytes, wireDigest), partition = await guest(partitionBytes, partitionDigest);
    const binding = admitted(wire, encode([1, 3, decode(bindingBytes)]));
    if (!same(bindingBytes, encode(binding))) throw fail('binding-mismatch');
    await artifactClosure(artifacts, binding, wireDigest, partitionDigest);
    store = await openStore(binding[6], LIMITS);
    const transport = new PayloadStore(wire, partition, binding, store);
    return Object.freeze({stage: transport.stage.bind(transport), load: transport.load.bind(transport), close: transport.close.bind(transport)});
  } catch (error) {
    store?.close();
    if (error instanceof OperationJournalError) throw error;
    throw fail('journal-unavailable');
  }
}
