// Private bootstrap custody. A key proves possession, not account authority.
// All policy admission and resource selection execute the generated model.
import {bytesCopy, createIdentity, validateIdentity, signBytes} from './identity.mjs';
import {decodeEffectWire, encodeEffectWire} from './effects-wire.mjs';
import {inspectEffectModule} from './effects-module.mjs';

const FRAME = 2097152, OUTPUT = 65536, PAGES = 2048, TIMEOUT = 5000;
const FORMAT = 'prismpm/browser-credential-custody/1';
const handles = new WeakMap();
const same = (a, b) => a.length === b.length && a.every((byte, index) => byte === b[index]);
const equal = (a, b) => same(encodeEffectWire(a), encodeEffectWire(b));
const hex = bytes => Array.from(bytes, byte => byte.toString(16).padStart(2, '0')).join('');
export class CredentialCustodyError extends Error {
  constructor(code, detail) {
    super(code); this.name = 'CredentialCustodyError'; this.code = code;
    if (detail !== undefined) this.detail = detail;
  }
}
const fail = (code, detail) => new CredentialCustodyError(code, detail);

function record(value, keys) {
  if (value === null || typeof value !== 'object' || Object.getPrototypeOf(value) !== Object.prototype) return false;
  const own = Reflect.ownKeys(value), fields = Object.getOwnPropertyDescriptors(value);
  return own.length === keys.length && own.every(key => typeof key === 'string' && keys.includes(key) && 'value' in fields[key]);
}
function stateOf(handle) {
  const state = handles.get(handle);
  if (!state) throw fail('invalid-input');
  if (state.closed) throw fail('custody-closed');
  return state;
}
function invalidate(state) {
  state.closed = true; state.identities.clear();
  try { state.db.close(); } catch { /* No storage rollback is inferred. */ }
}
function interpreter(module) {
  if (WebAssembly.Module.imports(module).length !== 0) throw fail('invalid-generated-module');
  return input => {
    if (input.length > FRAME) throw fail('invalid-input');
    try {
      const {exports} = new WebAssembly.Instance(module, {});
      const {memory, holo_alloc: allocate, holo_run: run} = exports;
      if (!(memory instanceof WebAssembly.Memory) || typeof allocate !== 'function' || allocate.length !== 1
        || typeof run !== 'function' || run.length !== 2 || memory.buffer.byteLength > PAGES * 65536) throw fail('invalid-generated-module');
      const pointer = allocate(input.length) >>> 0;
      if (pointer + input.length > memory.buffer.byteLength || memory.buffer.byteLength > PAGES * 65536) throw fail('invalid-generated-output');
      new Uint8Array(memory.buffer, pointer, input.length).set(input);
      const result = run(pointer, input.length);
      if (typeof result !== 'bigint') throw fail('invalid-generated-output');
      const packed = BigInt.asUintN(64, result), at = Number(packed >> 32n), length = Number(packed & 0xffffffffn);
      if (length > OUTPUT || at + length > memory.buffer.byteLength || memory.buffer.byteLength > PAGES * 65536) throw fail('invalid-generated-output');
      return new Uint8Array(memory.buffer, at, length).slice();
    } catch (error) {
      if (error instanceof CredentialCustodyError) throw error;
      throw fail('generated-execution-failed');
    }
  };
}
function transition(call, request, tag) {
  let result;
  try { result = decodeEffectWire(call(encodeEffectWire(request))); }
  catch (error) {
    if (error instanceof CredentialCustodyError) throw error;
    throw fail('invalid-generated-output');
  }
  if (!Array.isArray(result) || result.length !== 3 || result[0] !== 1) throw fail('invalid-generated-output');
  if (result[1] === 1 && Array.isArray(result[2]) && result[2].length === 1
    && Number.isInteger(result[2][0]) && result[2][0] >= 0 && result[2][0] < 8) throw fail('model-rejected', result[2][0]);
  if (result[1] === 2 && Number.isInteger(result[2]) && result[2] >= 0 && result[2] < 9) throw fail('wire-rejected', result[2]);
  if (result[1] !== 0 || !Array.isArray(result[2]) || result[2].length !== 2 || result[2][0] !== tag) throw fail('invalid-generated-output');
  return result[2][1];
}
function schema(db, transaction) {
  if (db.version !== 1 || Array.from(db.objectStoreNames).join(',') !== 'header,keys') throw fail('custody-corrupt');
  for (const name of ['header', 'keys']) {
    const store = transaction.objectStore(name);
    if (store.keyPath !== null || store.autoIncrement !== false || store.indexNames.length !== 0) throw fail('custody-corrupt');
  }
}
function storageError(error) {
  return fail(error?.name === 'QuotaExceededError' ? 'storage-quota' : 'storage-unavailable');
}
function openDatabase(name, mode, snapshot, identities) {
  return new Promise((resolve, reject) => {
    let request, settled = false, initialized = false, problem;
    const timer = setTimeout(() => {
      try { request?.result?.close(); } catch { /* A still-pending open has no result. */ }
      finish(fail(mode === 'initialize' ? 'custody-outcome-unknown' : 'storage-unavailable'));
    }, TIMEOUT);
    function finish(error, db) {
      if (settled) { db?.close(); return; }
      settled = true; clearTimeout(timer);
      if (error) { db?.close(); reject(error); } else resolve(db);
    }
    try { request = indexedDB.open(name, 1); }
    catch (error) { finish(storageError(error)); return; }
    request.onblocked = () => finish(fail('storage-unavailable'));
    request.onerror = () => finish(problem ?? storageError(request.error));
    request.onupgradeneeded = event => {
      try {
        if (settled) throw fail('storage-unavailable');
        if (mode !== 'initialize') throw fail('custody-missing');
        if (event.oldVersion !== 0) throw fail('custody-corrupt');
        const db = request.result, transaction = request.transaction;
        db.createObjectStore('header'); db.createObjectStore('keys');
        schema(db, transaction);
        transaction.objectStore('header').add({format: FORMAT, snapshot: encodeEffectWire(snapshot)}, 'current');
        for (const binding of snapshot[1]) {
          const [application, policy, slot, publicKey, principal] = binding;
          transaction.objectStore('keys').add({application, policy, slot, publicKey, principal,
            privateKey: identities.get(slot).privateKey}, slot);
        }
        initialized = true;
      } catch (error) {
        problem = error instanceof CredentialCustodyError ? error : storageError(error);
        try { request.transaction.abort(); } catch { /* The failed upgrade cannot be acknowledged. */ }
      }
    };
    request.onsuccess = () => {
      if (mode === 'initialize' && !initialized) finish(fail('custody-conflict'), request.result);
      else finish(undefined, request.result);
    };
  });
}
function readDatabase(db, call, policy) {
  return new Promise((resolve, reject) => {
    let transaction, settled = false, header, keys, rows, problem, pending = 3, writing = false;
    const timer = setTimeout(() => {
      try { transaction?.abort(); } catch { /* No successful read is inferred. */ }
      finish(fail(writing ? 'custody-outcome-unknown' : 'storage-unavailable'));
    }, TIMEOUT);
    function finish(error) {
      if (settled) return;
      settled = true; clearTimeout(timer);
      if (error) reject(error); else resolve({header, keys, rows});
    }
    try {
      transaction = db.transaction(['header', 'keys'], 'readwrite', {durability: 'strict'});
      if (transaction.durability !== 'strict') throw fail('storage-unavailable');
      schema(db, transaction);
      transaction.onabort = () => finish(problem ?? fail('custody-outcome-unknown'));
      transaction.onerror = () => { problem ??= fail('custody-outcome-unknown'); };
      transaction.oncomplete = () => finish();
      const headers = transaction.objectStore('header'), credentials = transaction.objectStore('keys');
      const headerCount = headers.count(), keyCount = credentials.count();
      function abort() {
        try { transaction.abort(); }
        catch { finish(problem ?? fail('custody-outcome-unknown')); }
      }
      function corrupt() { problem = fail('custody-corrupt'); abort(); }
      function flush() {
        if (--pending !== 0) return;
        try {
          if (!record(header, ['format', 'snapshot']) || header.format !== FORMAT) throw fail('custody-corrupt');
          const encoded = bytesCopy(header.snapshot, OUTPUT), snapshot = decodeEffectWire(encoded);
          if (!same(encodeEffectWire(snapshot), encoded)) throw fail('custody-corrupt');
          const accepted = transition(call, [1, 2, policy, [1, snapshot]], 1);
          if (!equal(accepted, snapshot)) throw fail('invalid-generated-output');
          if (rows.length !== snapshot[1].length || keys.length !== snapshot[1].length) throw fail('custody-corrupt');
          for (let index = 0; index < rows.length; index++) {
            const row = rows[index], binding = snapshot[1][index];
            if (!record(row, ['application', 'policy', 'slot', 'publicKey', 'principal', 'privateKey'])
              || keys[index] !== binding[2]
              || !equal([row.application, row.policy, row.slot, row.publicKey, row.principal], binding)) throw fail('custody-corrupt');
          }
          // Upgrade transactions cannot request strict durability. Both first
          // creation and reopening therefore acknowledge this strict barrier:
          // the exact immutable records read in this transaction are rewritten,
          // without choosing/replacing keys or creating missing records.
          writing = true;
          headers.put(header, 'current');
          for (let index = 0; index < rows.length; index++) credentials.put(rows[index], keys[index]);
        } catch (error) {
          problem = error instanceof CredentialCustodyError ? error : fail(writing ? 'custody-outcome-unknown' : 'custody-corrupt');
          abort();
        }
      }
      headerCount.onsuccess = () => {
        if (headerCount.result !== 1) { corrupt(); return; }
        const request = headers.get('current'); request.onsuccess = () => { header = request.result; flush(); };
      };
      keyCount.onsuccess = () => {
        if (!Number.isSafeInteger(keyCount.result) || keyCount.result < 1 || keyCount.result > 64) { corrupt(); return; }
        const names = credentials.getAllKeys(), values = credentials.getAll();
        names.onsuccess = () => { keys = names.result; flush(); };
        values.onsuccess = () => { rows = values.result; flush(); };
      };
    } catch (error) {
      try { transaction?.abort(); } catch { /* Read failure cannot grant a handle. */ }
      finish(error instanceof CredentialCustodyError ? error : storageError(error));
    }
  });
}

export async function openCredentialCustody(options) {
  if (arguments.length !== 1 || !record(options, ['wire', 'wireDigest', 'policy', 'mode'])) throw fail('invalid-input');
  let wire, expected, encoded, policy;
  const mode = options.mode;
  if (mode !== 'initialize' && mode !== 'open') throw fail('invalid-input');
  try {
    wire = bytesCopy(options.wire, 67108864); expected = bytesCopy(options.wireDigest, 32);
    encoded = bytesCopy(options.policy, OUTPUT); policy = decodeEffectWire(encoded);
    if (expected.length !== 32 || !same(encodeEffectWire(policy), encoded)) throw fail('invalid-input');
    inspectEffectModule(wire, PAGES);
  } catch { throw fail('invalid-input'); }
  let module;
  try {
    if (!same(new Uint8Array(await crypto.subtle.digest('SHA-256', wire)), expected)) throw fail('artifact-mismatch');
    module = await WebAssembly.compile(wire);
  } catch (error) {
    if (error instanceof CredentialCustodyError) throw error;
    throw fail('invalid-generated-module');
  }
  const call = interpreter(module), admitted = transition(call, [1, 0, policy], 0);
  if (!equal(admitted, policy)) throw fail('invalid-generated-output');
  const identities = new Map(), candidates = [];
  if (mode === 'initialize') {
    for (const slot of policy[2]) {
      const identity = await validateIdentity(await createIdentity());
      identities.set(slot, identity);
      candidates.push([policy[0], policy[1], slot, identity.publicKey, identity.principal]);
    }
  }
  let snapshot;
  if (mode === 'initialize') {
    snapshot = transition(call, [1, 1, policy, [0], candidates], 1);
    if (!equal(snapshot, [policy, candidates])) throw fail('invalid-generated-output');
  }
  const db = await openDatabase('prismpm.browser.custody.v1/' + hex(policy[0]), mode, snapshot, identities);
  const state = {db, call, snapshot: undefined, identities: new Map(), closed: false};
  db.onversionchange = () => invalidate(state);
  db.onclose = () => invalidate(state);
  try {
    const stored = await readDatabase(db, call, policy);
    if (!record(stored.header, ['format', 'snapshot']) || stored.header.format !== FORMAT) throw fail('custody-corrupt');
    const saved = bytesCopy(stored.header.snapshot, OUTPUT);
    snapshot = decodeEffectWire(saved);
    if (!same(encodeEffectWire(snapshot), saved)) throw fail('custody-corrupt');
    const accepted = transition(call, [1, 2, policy, [1, snapshot]], 1);
    if (!equal(accepted, snapshot)) throw fail('invalid-generated-output');
    if (!Array.isArray(stored.rows) || !Array.isArray(stored.keys)
      || stored.rows.length !== snapshot[1].length || stored.keys.length !== snapshot[1].length) throw fail('custody-corrupt');
    for (let index = 0; index < snapshot[1].length; index++) {
      const binding = snapshot[1][index], row = stored.rows[index];
      if (!record(row, ['application', 'policy', 'slot', 'publicKey', 'principal', 'privateKey'])
        || stored.keys[index] !== binding[2]
        || !equal([row.application, row.policy, row.slot, row.publicKey, row.principal], binding)) throw fail('custody-corrupt');
      const identity = await validateIdentity({privateKey: row.privateKey, publicKey: row.publicKey, principal: row.principal});
      if (state.closed) throw fail('custody-closed');
      state.identities.set(binding[2], identity);
    }
    if (state.closed) throw fail('custody-closed');
    state.snapshot = snapshot;
    const handle = Object.freeze(Object.create(null)); handles.set(handle, state); return handle;
  } catch (error) {
    invalidate(state);
    if (error instanceof CredentialCustodyError) throw error;
    throw fail('custody-corrupt');
  } finally { identities.clear(); }
}

export function credentialPublicBindings(handle) {
  if (arguments.length !== 1) throw fail('invalid-input');
  const state = stateOf(handle), [policy, bindings] = state.snapshot;
  const rows = new Map(bindings.map(row => [row[2], row]));
  return Object.freeze({application: policy[0].slice(), policy: policy[1].slice(),
    resources: Object.freeze(policy[3].map(([resource, slot, context, maximum]) => {
      const row = rows.get(slot);
      return Object.freeze({resource, slot, context, maximum, publicKey: row[3].slice(), principal: row[4]});
    }))});
}

function authorizeCredential(handle, resource, value) {
  const state = stateOf(handle);
  if (typeof resource !== 'string' || resource.length > 128 || !resource.isWellFormed()
    || new TextEncoder().encode(resource).length > 128) throw fail('invalid-input');
  let payload;
  try { payload = bytesCopy(value); } catch { throw fail('invalid-input'); }
  const [policy, bindings] = state.snapshot;
  const authorized = transition(state.call, [1, 3, state.snapshot, policy[0], policy[1], resource, payload], 2);
  const requested = policy[3].find(row => row[0] === resource);
  const binding = requested && bindings.find(row => row[2] === requested[1]);
  if (!requested || !binding || !equal(authorized, [...requested, binding[3], binding[4]])) throw fail('invalid-generated-output');
  return {state, payload, authorized};
}

// Private staged-effect preflight: no signature, key, permit or retained
// authorization escapes. The later signer reruns this exact generated check.
export function checkCredentialSigning(handle, resource, value) {
  if (arguments.length !== 3) throw fail('invalid-input');
  authorizeCredential(handle, resource, value);
}

export async function signCredential(handle, resource, value) {
  if (arguments.length !== 3) throw fail('invalid-input');
  const {state, payload, authorized} = authorizeCredential(handle, resource, value);
  const identity = state.identities.get(authorized[1]);
  const signature = await signBytes(identity, authorized[2], payload);
  if (state.closed) throw fail('custody-closed');
  return signature;
}

export function closeCredentialCustody(handle) {
  if (arguments.length !== 1) throw fail('invalid-input');
  const state = handles.get(handle);
  if (!state) throw fail('invalid-input');
  invalidate(state);
}
