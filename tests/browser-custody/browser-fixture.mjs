// Acceptance only: all successful transitions execute the actual generated
// artifact; all stored keys, signatures and transactions are browser-native.
export async function runFixture(input) {
  const {openCredentialCustody: open, credentialPublicBindings: bindings,
    signCredential: sign, checkCredentialSigning: checkSigning, closeCredentialCustody: close} = await import('./credential-custody.mjs');
  const {encodeEffectWire: encode, decodeEffectWire: decode} = await import('./effects-wire.mjs');
  const {verifyBytes, createIdentity} = await import('./identity.mjs');
  const check = (condition, message) => { if (!condition) throw Error(message); };
  const hex = bytes => {
    const chunks = [];
    for (let at = 0; at < bytes.length; at += 4096) chunks.push(Array.from(bytes.subarray(at, at + 4096), x => x.toString(16).padStart(2, '0')).join(''));
    return chunks.join('');
  };
  const same = (a, b) => hex(encode(a)) === hex(encode(b));
  const digest = async value => new Uint8Array(await crypto.subtle.digest('SHA-256', value));
  const wire = new Uint8Array(input.wire), wireDigest = await digest(wire);
  const cases = [], calls = [], handles = [], restores = [];
  let maximum = 0, seed = 1, observation = null;
  const patch = (object, field, descriptor) => {
    const before = Object.getOwnPropertyDescriptor(object, field);
    Object.defineProperty(object, field, {configurable: true, ...descriptor});
    const restore = () => { if (before) Object.defineProperty(object, field, before); else delete object[field]; };
    restores.push(restore); return restore;
  };
  const replace = (object, field, value) => patch(object, field, {writable: true, value});
  const NativeInstance = WebAssembly.Instance;
  replace(WebAssembly, 'Instance', class {
    constructor(module, imports) {
      const real = new NativeInstance(module, imports), exports = {...real.exports};
      exports.holo_run = (pointer, length) => {
        const request = new Uint8Array(exports.memory.buffer, pointer, length).slice();
        const result = BigInt.asUintN(64, real.exports.holo_run(pointer, length));
        const at = Number(result >> 32n), size = Number(result & 0xffffffffn);
        const response = new Uint8Array(exports.memory.buffer, at, size).slice();
        calls.push({request: hex(request), response: hex(response)});
        maximum = Math.max(maximum, exports.memory.buffer.byteLength);
        if (observation && decode(request)[1] === observation.operation) {
          const current = observation; observation = null;
          const changed = decode(response); check(changed[1] === 0, 'fault follows actual generated success');
          current.change(changed[2][1]); const encoded = encode(changed);
          check(encoded.length === size, 'same-length generated-output observation fault');
          new Uint8Array(exports.memory.buffer, at, size).set(encoded);
        }
        return result;
      };
      this.exports = exports;
    }
  });
  const nativeOpen = IDBFactory.prototype.open, nativeTransaction = IDBDatabase.prototype.transaction;
  async function rejects(promise, code, detail) {
    let error;
    try { await promise; } catch (caught) { error = caught; }
    check(error?.code === code, 'expected custody ' + code + ', got ' + error?.code);
    if (detail !== undefined) check(error.detail === detail, 'expected model rejection ' + detail);
    return error;
  }
  const policy = () => [new Uint8Array(32).fill(seed++), new Uint8Array(32).fill(90), ['alpha', 'beta'], [
    ['journal', 'alpha', 'prismpm/browser-operation-journal/1', 65536],
    ['small', 'alpha', 'custody/small/1', 2], ['wide', 'beta', 'custody/wide/1', 1048576],
  ]];
  const name = p => 'prismpm.browser.custody.v1/' + hex(p[0]);
  const options = (p, mode) => ({wire, wireDigest, policy: encode(p), mode});
  async function create(p = policy()) {
    const handle = await open(options(p, 'initialize')); handles.push(handle); return {p, handle};
  }
  async function database(p, version) {
    return new Promise((resolve, reject) => {
      const request = Reflect.apply(nativeOpen, indexedDB, version === undefined ? [name(p)] : [name(p), version]);
      request.onerror = () => reject(request.error); request.onsuccess = () => resolve(request.result);
    });
  }
  async function alter(p, change) {
    const db = await database(p);
    try {
      await new Promise((resolve, reject) => {
        const tx = Reflect.apply(nativeTransaction, db, [['header', 'keys'], 'readwrite', {durability: 'strict'}]);
        tx.oncomplete = resolve; tx.onabort = () => reject(tx.error ?? Error('fixture transaction aborted'));
        change(tx);
      });
    } finally { db.close(); }
  }
  function row(tx, slot, update) {
    const store = tx.objectStore('keys'), request = store.get(slot);
    request.onsuccess = () => { const value = request.result; update(value); store.put(value, slot); };
  }
  async function readKeys(p) {
    const db = await database(p);
    try { return await new Promise((resolve, reject) => {
      const tx = Reflect.apply(nativeTransaction, db, [['keys'], 'readonly']), request = tx.objectStore('keys').getAll();
      tx.oncomplete = () => resolve(request.result); tx.onabort = () => reject(tx.error);
    }); } finally { db.close(); }
  }
  try {
    const p = policy(), invalid = options(p, 'initialize');
    Object.defineProperty(invalid, Symbol('extra'), {value: true});
    await rejects(open(invalid), 'invalid-input');
    const hidden = options(p, 'initialize'); Object.defineProperty(hidden, 'extra', {value: 1});
    await rejects(open(hidden), 'invalid-input');
    const mismatch = options(p, 'initialize'); mismatch.wireDigest = wireDigest.slice(); mismatch.wireDigest[0] ^= 1;
    await rejects(open(mismatch), 'artifact-mismatch');
    const captured = options(p, 'initialize'), pending = open(captured);
    captured.policy.fill(255); captured.wireDigest = new Uint8Array(32);
    const first = await pending; handles.push(first);
    check(Object.isFrozen(first) && Reflect.ownKeys(first).length === 0, 'opaque branded handle has no keys');
    cases.push('closed-bootstrap-and-synchronous-policy-capture');

    const publicRows = bindings(first);
    check(publicRows.resources.length === 3 && Object.isFrozen(publicRows) && Object.isFrozen(publicRows.resources), 'closed public binding shape');
    check(hex(publicRows.resources[0].publicKey) === hex(publicRows.resources[1].publicKey), 'one slot supports multiple exact resources');
    check(hex(publicRows.resources[0].publicKey) !== hex(publicRows.resources[2].publicKey), 'different slots use different real keys');
    publicRows.application.fill(0); publicRows.resources[0].publicKey.fill(0);
    check(bindings(first).application[0] === p[0][0] && bindings(first).resources[0].publicKey[0] === 4, 'public copies cannot mutate custody');
    for (const key of await readKeys(p)) {
      check(key.privateKey instanceof CryptoKey && key.privateKey.extractable === false, 'actual retained private key is nonextractable');
      let exported = false; try { await crypto.subtle.exportKey('pkcs8', key.privateKey); exported = true; } catch { /* Required native rejection. */ }
      check(!exported, 'private key export must reject');
    }
    cases.push('real-nonextractable-multiple-shared-slot-keys');

    const message = new Uint8Array([7, 8]), signing = sign(first, 'small', message); message.fill(9);
    const signature = await signing, small = bindings(first).resources[1];
    check(signature.length === 64 && await verifyBytes(small.publicKey, small.context, new Uint8Array([7, 8]), signature), 'exact captured bytes are signed');
    check(!await verifyBytes(small.publicKey, bindings(first).resources[0].context, new Uint8Array([7, 8]), signature), 'shared-slot contexts cannot cross');
    await rejects(sign(first, 'small', new Uint8Array(3)), 'model-rejected', 7);
    await rejects(sign(first, 'missing', new Uint8Array()), 'model-rejected', 6);
    await rejects(sign(first, 'x'.repeat(129), new Uint8Array()), 'invalid-input');
    await rejects(sign(first, '\ud800', new Uint8Array()), 'invalid-input');
    await rejects(sign(first, 'é'.repeat(65), new Uint8Array()), 'invalid-input');
    check(checkSigning(first, 'small', new Uint8Array(2)) === undefined, 'preflight exports no permit or key');
    let preflight;
    try { checkSigning(first, 'small', new Uint8Array(3)); } catch (error) { preflight = error; }
    check(preflight?.code === 'model-rejected' && preflight.detail === 7, 'generated preflight preserves smaller resource maximum');
    const journal = bindings(first).resources[0], metadata = new Uint8Array(65536);
    check(await verifyBytes(journal.publicKey, journal.context, metadata, await sign(first, 'journal', metadata)), 'actual exact journal metadata maximum');
    await rejects(sign(first, 'journal', new Uint8Array(65537)), 'model-rejected', 7);
    cases.push('generated-sign-admission-contexts-and-capture');

    const before = bindings(first).resources.map(x => hex(x.publicKey)); close(first);
    let generated = 0; const generateKey = crypto.subtle.generateKey;
    const restoreGenerate = replace(crypto.subtle, 'generateKey', function(...args) { generated++; return Reflect.apply(generateKey, this, args); });
    const reopened = await open(options(p, 'open')); handles.push(reopened); restoreGenerate();
    check(generated === 0, 'Open never creates keys');
    check(JSON.stringify(bindings(reopened).resources.map(x => hex(x.publicKey))) === JSON.stringify(before), 'Open retains exactly the same keys');
    cases.push('strict-reopen-never-generates-or-replaces');

    const racePolicy = policy();
    const races = await Promise.allSettled([open(options(racePolicy, 'initialize')), open(options(racePolicy, 'initialize'))]);
    check(races.filter(x => x.status === 'fulfilled').length === 1, 'one atomic initial creator wins');
    const winner = races.find(x => x.status === 'fulfilled').value; handles.push(winner);
    check(races.find(x => x.status === 'rejected').reason.code === 'custody-conflict', 'losing creator is a conflict');
    check((await readKeys(racePolicy)).length === 2, 'atomic complete winner inventory');
    cases.push('atomic-first-creation-race');

    const absent = policy(); await rejects(open(options(absent, 'open')), 'custody-missing');
    check(!(await indexedDB.databases()).some(x => x.name === name(absent)), 'Open does not leave an empty database');
    const altered = structuredClone(p); altered[3][0][2] = 'different/1';
    await rejects(open(options(altered, 'open')), 'model-rejected', 4);
    cases.push('missing-custody-and-immutable-policy');

    for (const [label, change, code] of [
      ['missing-key', tx => tx.objectStore('keys').delete('beta'), 'custody-corrupt'],
      ['unknown-key', tx => row(tx, 'alpha', value => { tx.objectStore('keys').put(value, 'z'); }), 'custody-corrupt'],
      ['missing-header', tx => tx.objectStore('header').delete('current'), 'custody-corrupt'],
      ['unknown-header', tx => tx.objectStore('header').put({}, 'extra'), 'custody-corrupt'],
      ['application', tx => row(tx, 'alpha', value => { value.application[0] ^= 1; }), 'custody-corrupt'],
      ['slot', tx => row(tx, 'alpha', value => { value.slot = 'other'; }), 'custody-corrupt'],
      ['public-key', tx => row(tx, 'alpha', value => { value.publicKey[2] ^= 1; }), 'custody-corrupt'],
      ['principal', tx => row(tx, 'alpha', value => { value.principal = 'sha256:' + '0'.repeat(64); }), 'custody-corrupt'],
      ['unknown-field', tx => row(tx, 'alpha', value => { value.extra = 1; }), 'custody-corrupt'],
      ['missing-secret', tx => row(tx, 'alpha', value => { delete value.privateKey; }), 'custody-corrupt'],
      ['header-bytes', tx => tx.objectStore('header').put({format: 'prismpm/browser-credential-custody/1', snapshot: new Uint8Array([0])}, 'current'), 'wire-rejected'],
    ]) {
      const item = await create(); close(item.handle); await alter(item.p, change);
      await rejects(open(options(item.p, 'open')), code); cases.push('corrupt-' + label);
    }
    const foreign = await createIdentity();
    const wrong = await create(); close(wrong.handle);
    await alter(wrong.p, tx => row(tx, 'alpha', value => { value.privateKey = foreign.privateKey; }));
    await rejects(open(options(wrong.p, 'open')), 'custody-corrupt'); cases.push('wrong-actual-key-possession');

    const extractable = await crypto.subtle.generateKey({name: 'ECDSA', namedCurve: 'P-256'}, true, ['sign', 'verify']);
    const extract = await create(); close(extract.handle);
    await alter(extract.p, tx => row(tx, 'alpha', value => { value.privateKey = extractable.privateKey; }));
    await rejects(open(options(extract.p, 'open')), 'custody-corrupt'); cases.push('extractable-key-rejected');

    for (const kind of ['index', 'extra-store', 'keypath']) {
      const q = policy();
      await new Promise((resolve, reject) => {
        const request = Reflect.apply(nativeOpen, indexedDB, [name(q), 1]);
        request.onerror = () => reject(request.error);
        request.onupgradeneeded = () => {
          const db = request.result; db.createObjectStore('header');
          const keys = db.createObjectStore('keys', kind === 'keypath' ? {keyPath: 'slot'} : {});
          if (kind === 'index') keys.createIndex('unexpected', 'slot');
          if (kind === 'extra-store') db.createObjectStore('unknown');
        };
        request.onsuccess = () => { request.result.close(); resolve(); };
      });
      await rejects(open(options(q, 'open')), 'custody-corrupt'); cases.push('schema-' + kind + '-rejected');
    }

    const originalAdd = IDBObjectStore.prototype.add;
    for (const [label, name_, expected] of [['abort', 'AbortError', 'storage-unavailable'], ['quota', 'QuotaExceededError', 'storage-quota']]) {
      const q = policy(); let fired = false;
      const restore = replace(IDBObjectStore.prototype, 'add', function(...args) {
        if (!fired && this.name === 'keys') { fired = true; throw new DOMException('injected storage fault', name_); }
        return Reflect.apply(originalAdd, this, args);
      });
      await rejects(open(options(q, 'initialize')), expected); restore(); check(fired, 'actual upgrade fault reached');
      check(!(await indexedDB.databases()).some(x => x.name === name(q)), 'aborted atomic upgrade leaves no partial custody');
      const retry = await open(options(q, 'initialize')); handles.push(retry);
      cases.push('atomic-upgrade-' + label);
    }

    const strict = await create(); close(strict.handle);
    const restoreDurability = replace(IDBDatabase.prototype, 'transaction', function(stores, mode, options_) {
      const result = Reflect.apply(nativeTransaction, this, [stores, mode, {...options_, durability: 'default'}]); return result;
    });
    await rejects(open(options(strict.p, 'open')), 'storage-unavailable'); restoreDurability();
    cases.push('unsupported-strict-durability-rejected');

    const barrier = await create(); const retained = bindings(barrier.handle).resources.map(x => hex(x.publicKey)); close(barrier.handle);
    const originalPut = IDBObjectStore.prototype.put; let aborted = false;
    const restorePut = replace(IDBObjectStore.prototype, 'put', function(...args) {
      const result = Reflect.apply(originalPut, this, args);
      if (!aborted && this.name === 'keys') { aborted = true; this.transaction.abort(); }
      return result;
    });
    await rejects(open(options(barrier.p, 'open')), 'custody-outcome-unknown'); restorePut();
    const afterBarrier = await open(options(barrier.p, 'open')); handles.push(afterBarrier);
    check(JSON.stringify(bindings(afterBarrier).resources.map(x => hex(x.publicKey))) === JSON.stringify(retained), 'barrier abort cannot replace keys');
    cases.push('strict-barrier-abort-and-explicit-reopen');

    for (const kind of ['upgrade', 'barrier']) {
      const q = policy(); let fired = false, actualCommitted = false;
      const prototype = kind === 'upgrade' ? IDBRequest.prototype : IDBTransaction.prototype;
      const event = kind === 'upgrade' ? 'onsuccess' : 'oncomplete';
      const descriptor = Object.getOwnPropertyDescriptor(prototype, event);
      const restore = patch(prototype, event, {get: descriptor.get, set(callback) {
        if (!fired && (kind === 'upgrade' ? this instanceof IDBOpenDBRequest : this.durability === 'strict')) {
          fired = true;
          Reflect.apply(descriptor.set, this, [() => { actualCommitted = true; }]);
        } else Reflect.apply(descriptor.set, this, [callback]);
      }});
      await rejects(open(options(q, 'initialize')), 'custody-outcome-unknown'); restore();
      check(fired && actualCommitted, 'acknowledgment fault follows real committed ' + kind);
      const rows = await readKeys(q), reopened = await open(options(q, 'open')); handles.push(reopened);
      check(hex(rows[0].publicKey) === hex(bindings(reopened).resources[0].publicKey), 'lost ack reopens actual winner without regeneration');
      cases.push('lost-' + kind + '-acknowledgment');
    }

    const closing = await create(), saved = bindings(closing.handle).resources[1], originalSign = crypto.subtle.sign;
    let release, began; const started = new Promise(resolve => { began = resolve; });
    const waiting = new Promise(resolve => { release = resolve; });
    const restoreSign = replace(crypto.subtle, 'sign', async function(...args) {
      const result = await Reflect.apply(originalSign, this, args); began(); await waiting; return result;
    });
    const late = sign(closing.handle, 'small', new Uint8Array([1]));
    await started; close(closing.handle); release();
    await rejects(late, 'custody-closed'); restoreSign();
    let inspected = false;
    await rejects(sign(closing.handle, 'small', new Proxy({}, {get() { inspected = true; throw Error('untrusted getter'); }})), 'custody-closed');
    let closedCheck;
    try { checkSigning(closing.handle, 'small', new Proxy({}, {get() { inspected = true; throw Error('untrusted getter'); }})); }
    catch (error) { closedCheck = error; }
    check(closedCheck?.code === 'custody-closed', 'closed preflight cannot authorize future signing');
    check(!inspected && saved.publicKey[0] === 4, 'close precedes payload inspection and late sign cannot reopen');
    cases.push('closed-and-late-sign-custody');

    const version = await create(); const upgraded = await database(version.p, 2); upgraded.close();
    await rejects(sign(version.handle, 'small', new Uint8Array()), 'custody-closed');
    await rejects(open(options(version.p, 'open')), 'storage-unavailable');
    cases.push('schema-versionchange-invalidates-handle');

    for (const [label, operation, change] of [
      ['policy', 0, value => { value[0][0] ^= 1; }],
      ['initialize', 1, value => { value[0][0][0] ^= 1; }],
      ['open', 2, value => { value[0][0][0] ^= 1; }],
      ['sign', 3, value => { value[2] = 'custody/other/1'; }],
    ]) {
      const q = label === 'sign' || label === 'open' ? await create() : {p: policy()};
      observation = {operation, change};
      if (label === 'sign') await rejects(sign(q.handle, 'small', new Uint8Array()), 'invalid-generated-output');
      else await rejects(open(options(q.p, label === 'open' ? 'open' : 'initialize')), 'invalid-generated-output');
      check(observation === null, 'actual generated output fault executed'); cases.push('generated-' + label + '-binding-observation');
    }

    const maximumPolicy = policy(); maximumPolicy[2] = []; maximumPolicy[3] = [];
    for (let index = 0; index < 64; index++) {
      const slot = 's'.repeat(127) + '-.0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz'[index];
      maximumPolicy[2].push(slot); maximumPolicy[3].push([slot, slot, 'c'.repeat(128), 1048576]);
    }
    const maximumCustody = await create(maximumPolicy), maximumRows = bindings(maximumCustody.handle);
    check(maximumRows.resources.length === 64, 'all64 actual nonextractable keys initialized');
    const maximumPayload = new Uint8Array(1048576), last = maximumRows.resources[63];
    const signatureMaximum = await sign(maximumCustody.handle, last.resource, maximumPayload);
    check(await verifyBytes(last.publicKey, last.context, maximumPayload, signatureMaximum), 'actual full1MiB signature verifies');
    await rejects(sign(maximumCustody.handle, last.resource, new Uint8Array(1048577)), 'invalid-input');
    close(maximumCustody.handle);
    const final = await open(options(maximumPolicy, 'open')); handles.push(final);
    check(hex(bindings(final).resources[63].publicKey) === hex(last.publicKey), 'complete max inventory reopens unchanged');
    cases.push('maximum64slots128names1mib-real-sign-and-reopen');
    return {cases, calls, maximum};
  } finally {
    for (const handle of handles) { try { close(handle); } catch { /* Test cleanup does not alter authority. */ } }
    for (const restore of restores.reverse()) restore();
  }
}
