// Generic host binding. The verified generated module owns all journal
// transitions; this adapter owns cryptographic and storage authenticity.
import { BrowserEffectError, bytesCopy, digestBytes, identityPrincipal, verifyBytes } from './identity.mjs';

const empty = () => new Uint8Array();
const zero = () => new Uint8Array(32);
const context = 'prismpm/workspace-event/1';
const inputMaximum = 1235980;
const outputMaximum = 1166008;
// Local host admission budget, independent of all modeled domain limits.
const outstandingMaximum = 2;
const hashPattern = /^sha256:[0-9a-f]{64}$/;
const hex = bytes => Array.from(bytes, value => value.toString(16).padStart(2, '0')).join('');
const equal = (left, right) => left.length === right.length
  && left.every((value, index) => value === right[index]);
const storageErrors = new Set([
  'invalid-input', 'crypto-unavailable', 'identity-corrupt', 'identity-exists',
  'head-conflict', 'missing-object', 'object-corrupt', 'store-limit',
  'store-policy-mismatch', 'store-closed', 'storage-blocked', 'storage-quota',
  'storage-unavailable',
]);

export class JournalAdapterError extends Error {
  constructor(code, detail = null) {
    super(code);
    this.name = 'JournalAdapterError';
    this.code = code;
    this.detail = detail;
  }
}

const fail = code => new JournalAdapterError(code);
function hashBytes(value) {
  if (typeof value !== 'string' || !hashPattern.test(value)) throw fail('invalid-hash');
  return Uint8Array.from(value.slice(7).match(/../g), byte => parseInt(byte, 16));
}
function concatenate(...parts) {
  const output = new Uint8Array(parts.reduce((length, value) => length + value.length, 0));
  let offset = 0;
  for (const part of parts) { output.set(part, offset); offset += part.length; }
  return output;
}
function u24(value) {
  if (!Number.isSafeInteger(value) || value < 0 || value > 0xffffff) throw fail('invalid-length');
  return new Uint8Array([value >>> 16, value >>> 8 & 255, value & 255]);
}
function read24(bytes, offset) {
  if (offset + 3 > bytes.length) throw fail('invalid-generated-output');
  return bytes[offset] * 65536 + bytes[offset + 1] * 256 + bytes[offset + 2];
}
function accepted(bytes) {
  if (bytes.length === 0) throw fail('invalid-generated-output');
  if (bytes[0] !== 0) throw new JournalAdapterError('model-rejected', hex(bytes));
  return bytes.subarray(1);
}
function plan(bytes) {
  accepted(bytes);
  const headLength = read24(bytes, 1), stateLength = read24(bytes, 4);
  if (headLength > 65574 || stateLength > 1100427
      || bytes.length !== 7 + headLength + stateLength) throw fail('invalid-generated-output');
  return { head: bytes.slice(7, 7 + headLength), state: bytes.slice(7 + headLength) };
}
const appendPayload = (head, state, object, envelope) => concatenate(
  u24(head.length), u24(state.length), head, state, object, envelope);

function interpreter(module) {
  if (!(module instanceof WebAssembly.Module)) throw fail('invalid-generated-module');
  return request => {
    if (request.length > inputMaximum) throw fail('request-limit');
    let instance, response;
    try {
      // Generated allocation is monotonic: a fresh guest is mandatory.
      instance = new WebAssembly.Instance(module, {});
      const { memory, holo_alloc: allocate, holo_run: run } = instance.exports;
      if (!(memory instanceof WebAssembly.Memory) || typeof allocate !== 'function'
          || typeof run !== 'function') throw fail('invalid-generated-module');
      const pointer = allocate(request.length) >>> 0;
      if (pointer + request.length > memory.buffer.byteLength) throw fail('invalid-generated-output');
      new Uint8Array(memory.buffer, pointer, request.length).set(request);
      const packed = BigInt.asUintN(64, run(pointer, request.length));
      const offset = Number(packed >> 32n), length = Number(packed & 0xffffffffn);
      if (length > outputMaximum || offset + length > memory.buffer.byteLength
          || memory.buffer.byteLength > 41943040) throw fail('invalid-generated-output');
      response = new Uint8Array(memory.buffer, offset, length).slice();
    } catch (error) {
      if (error instanceof JournalAdapterError) throw error;
      throw fail('generated-execution-failed');
    }
    return response;
  };
}

class Journal {
  #store; #name; #call; #head = empty(); #state = empty();
  #tail = Promise.resolve(); #requiresReplay = true;
  #outstanding = 0;

  constructor(module, store, name) {
    this.#call = interpreter(module);
    this.#store = store;
    this.#name = name;
  }

  #queue(operation) {
    const pending = this.#tail.then(operation);
    const released = pending.finally(() => { this.#outstanding--; });
    this.#tail = released.catch(() => {});
    return released;
  }

  #reserve() {
    if (this.#outstanding >= outstandingMaximum) throw fail('journal-busy');
    this.#outstanding++;
  }

  snapshot() { return { head: this.#head.slice(), state: this.#state.slice() }; }

  async #storage(method, ...arguments_) {
    try { return await this.#store[method](...arguments_); }
    catch (error) {
      let code;
      try { code = error?.code; } catch { /* Host getters are not trusted. */ }
      // Preserve the known host namespace, not an arbitrary backend message,
      // payload, public cause or injected extra property.
      if (storageErrors.has(code)) throw new BrowserEffectError(code);
      throw fail('storage-outcome-unknown');
    }
  }

  async #authenticate(envelope) {
    const project = operation => accepted(this.#call(concatenate([6, operation], envelope)));
    const key = project(3), signature = project(4);
    if (await identityPrincipal(key) !== `sha256:${hex(project(6))}`) throw fail('author-mismatch');
    if (await digestBytes(project(2)) !== `sha256:${hex(project(7))}`) throw fail('event-id-mismatch');
    if (!await verifyBytes(key, context, project(1), signature)) throw fail('signature-invalid');
    return hashBytes(await digestBytes(envelope));
  }

  refresh() {
    if (arguments.length !== 0) return Promise.reject(fail('invalid-input'));
    try { this.#reserve(); }
    catch (error) { return Promise.reject(error); }
    return this.#queue(async () => {
      this.#requiresReplay = true;
      const retained = await this.#storage('readHead', this.#name);
      let head = empty(), state = empty();
      if (retained !== null) {
        const target = bytesCopy(retained.bytes, 65574);
        if (await digestBytes(target) !== retained.id) throw fail('object-corrupt');
        if (!equal(accepted(this.#call(concatenate([0], target))), target)) {
          throw fail('invalid-generated-output');
        }
        // Fixed ABI framing only; generated replay checks every selected row,
        // workspace, event identity, sequence, role and exact final prefix.
        if (target.length < 38) throw fail('invalid-generated-output');
        const count = new DataView(target.buffer, target.byteOffset, target.byteLength).getUint16(36);
        if (count < 1 || count > 1024 || target.length !== 38 + 64 * count) throw fail('invalid-generated-output');
        for (let index = 0; index < count; index++) {
          const expected = `sha256:${hex(target.subarray(70 + index * 64, 102 + index * 64))}`;
          const stored = await this.#storage('readObject', expected);
          if (stored === null) throw fail('object-missing');
          const envelope = bytesCopy(stored, 4363);
          if (await digestBytes(envelope) !== expected) throw fail('object-corrupt');
          const object = await this.#authenticate(envelope);
          const next = plan(this.#call(concatenate([3], u24(target.length), target,
            appendPayload(head, state, object, envelope))));
          head = next.head; state = next.state;
        }
        state = accepted(this.#call(concatenate([4], u24(target.length), u24(head.length),
          u24(state.length), target, head, state))).slice();
      }
      // No partial replay becomes visible, even if a later record is corrupt.
      this.#head = head; this.#state = state; this.#requiresReplay = false;
      return this.snapshot();
    });
  }

  append(value) {
    // Never accept a claimed identity, state, authentication flag or receipt.
    if (arguments.length !== 1) return Promise.reject(fail('invalid-input'));
    try { this.#reserve(); }
    catch (error) { return Promise.reject(error); }
    let envelope;
    try { envelope = bytesCopy(value, 4363); }
    catch (error) { this.#outstanding--; return Promise.reject(error); }
    return this.#queue(async () => {
      if (this.#requiresReplay) throw fail('replay-required');
      const object = await this.#authenticate(envelope);
      const candidate = plan(this.#call(concatenate([1], appendPayload(this.#head, this.#state, object, envelope))));
      const expected = this.#head.length === 0 ? null : await digestBytes(this.#head);
      const prior = expected === null ? zero() : hashBytes(expected);
      const next = await digestBytes(candidate.head), nextBytes = hashBytes(next);
      const intent = accepted(this.#call(concatenate([5], prior, nextBytes, object)));
      let nonce;
      try { nonce = crypto.getRandomValues(new Uint8Array(32)); }
      catch { throw new BrowserEffectError('crypto-unavailable'); }
      const session = concatenate([0], nonce, prior,
        nextBytes, hashBytes(await digestBytes(intent)));
      let status = 0, observed = nextBytes;
      this.#requiresReplay = true;
      try {
        const returned = await this.#storage('commit', { head: this.#name, expected, next,
          objects: [envelope, candidate.head] });
        if (returned !== next) throw fail('storage-result-mismatch');
      } catch (error) {
        const statuses = { 'head-conflict': 1, 'store-limit': 2, 'storage-quota': 3,
          'store-closed': 4, 'storage-unavailable': 5 };
        // An unexpected adapter failure may follow a commit. Do not retry or
        // promote; require fresh authenticated replay before further writes.
        this.#requiresReplay = true;
        if (!Object.hasOwn(statuses, error.code)) throw error;
        status = statuses[error.code]; observed = prior;
        if (status === 1) {
          const current = await this.#storage('readHead', this.#name);
          observed = current === null ? zero() : hashBytes(current.id);
        }
      }
      const result = this.#call(concatenate([2], session, [status], session.subarray(1), observed));
      if (result.length !== 130 || !equal(result.subarray(2), session.subarray(1))) {
        this.#requiresReplay = true;
        throw fail('invalid-generated-output');
      }
      if (status === 0 && result[0] === 0 && result[1] === 1) {
        this.#head = candidate.head; this.#state = candidate.state;
        this.#requiresReplay = false;
        return this.snapshot();
      }
      this.#requiresReplay = true;
      if (status !== 0 && result[0] === 32 + status && result[1] === 2) {
        throw new JournalAdapterError('storage-rejected', result[0]);
      }
      throw fail('invalid-generated-output');
    });
  }
}

// Trusted SDK bootstrap supplies the verified module and real storage binding.
// Product callers receive only the returned private-state journal object.
export async function openJournal(module, store, headName) {
  if (typeof headName !== 'string' || !/^[a-zA-Z0-9][a-zA-Z0-9._-]{0,127}$/.test(headName)) {
    throw fail('invalid-input');
  }
  const journal = new Journal(module, store, headName);
  await journal.refresh();
  return Object.freeze(journal);
}
