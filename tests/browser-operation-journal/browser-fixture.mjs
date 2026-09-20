// Real generated artifacts, native WebCrypto, native IndexedDB. All payloads
// here are synthetic acceptance fixtures. No positive receipt is injected.
const check = (value, message) => { if (!value) throw Error(message); };
const hex = bytes => {
  const parts = [];
  for (let at = 0; at < bytes.length; at += 4096) parts.push(Array.from(bytes.subarray(at, at + 4096), byte => byte.toString(16).padStart(2, '0')).join(''));
  return parts.join('');
};
const unhex = text => Uint8Array.from(text.match(/../g) ?? [], byte => parseInt(byte, 16));
const digest = async bytes => new Uint8Array(await crypto.subtle.digest('SHA-256', bytes));
const deferred = () => { let resolve; const promise = new Promise(done => { resolve = done; }); return {promise, resolve}; };
const observed = promise => promise.then(value => ({value}), error => ({error}));
async function bounded(promise) {
  let timer;
  try { return await Promise.race([promise, new Promise((_, reject) => { timer = setTimeout(() => reject(Error('bounded journal fixture timeout')), 20000); })]); }
  finally { clearTimeout(timer); }
}
async function rejects(promise, code) {
  const {error} = await observed(promise);
  check(error?.code === code, 'expected journal ' + code + ', got ' + error?.code);
  check(error.cause === undefined, 'diagnostics never contain private payload');
  return error;
}

function observeArtifacts(input) {
  const originalCompile = WebAssembly.compile, NativeInstance = WebAssembly.Instance;
  const modules = new WeakMap(), calls = [], large = [], maximum = {}, pending = [];
  const actual = new Map(Object.entries(input).map(([role, array]) => [hex(new Uint8Array(array)), role]));
  WebAssembly.compile = async function(bytes) {
    const captured = new Uint8Array(bytes).slice(), role = actual.get(hex(captured));
    check(role, 'only actual source-generated artifacts may compile');
    const module = await Reflect.apply(originalCompile, WebAssembly, [captured]); modules.set(module, role); return module;
  };
  WebAssembly.Instance = class {
    constructor(module, imports) {
      const role = modules.get(module); check(role, 'every guest instance has captured artifact provenance');
      const real = new NativeInstance(module, imports), exports = {...real.exports};
      exports.holo_run = (pointer, length) => {
        const request = new Uint8Array(exports.memory.buffer, pointer, length).slice();
        const result = BigInt.asUintN(64, real.exports.holo_run(pointer, length));
        const at = Number(result >> 32n), size = Number(result & 0xffffffffn);
        const response = new Uint8Array(exports.memory.buffer, at, size).slice();
        maximum[role] = Math.max(maximum[role] ?? 0, exports.memory.buffer.byteLength);
        if (role === 'Partition' && length > 2097152) {
          check(request.every((byte, at) => byte === (at % 1048576 === 0 ? at / 1048576 : 0x5a)),
            'large frame is the exact documented synthetic transport fixture with64distinctchunks');
          pending.push(digest(request).then(hash => large.push({role, length, digest: hex(hash), response: hex(response)})));
        } else calls.push({role, request: hex(request), response: hex(response)});
        return result;
      };
      return {exports};
    }
  };
  return {calls, large, maximum, flush: () => Promise.all(pending),
    restore() { WebAssembly.compile = originalCompile; WebAssembly.Instance = NativeInstance; }};
}

export async function runFixture(input) {
  const NativeBytes = Uint8Array; let copies = 0, journalModule;
  globalThis.Uint8Array = new Proxy(NativeBytes, {construct(target, args) {
    if (typeof args[0] === 'number' && args[0] > 65) copies++;
    return Reflect.construct(target, args);
  }});
  try { journalModule = await import('./operation-journal.mjs'); } finally { globalThis.Uint8Array = NativeBytes; }
  const {openOperationJournal: open, openOperationPayloadStore: openPayloads} = journalModule;
  const {openCredentialCustody, credentialPublicBindings, closeCredentialCustody} = await import('./credential-custody.mjs');
  const {encodeEffectWire: encode, decodeEffectWire: decode} = await import('./effects-wire.mjs');
  const {openStore} = await import('./store.mjs');
  const {verifyBytes} = await import('./identity.mjs');
  const {openStagedEffects} = await import('./effects.mjs');
  const observer = observeArtifacts(input), cases = [], hosts = [], custodies = [], stores = [], releases = [], frames = [];
  const originalTransaction = IDBDatabase.prototype.transaction, originalOpen = IDBFactory.prototype.open;
  const artifacts = Object.fromEntries(Object.entries(input).map(([name, bytes]) => [name, new Uint8Array(bytes)]));
  const digests = Object.fromEntries(await Promise.all(Object.entries(artifacts).map(async ([name, bytes]) => [name, await digest(bytes)])));
  const same = (a, b) => hex(encode(a)) === hex(encode(b));
  const limits = {maxObjectBytes: 1048576, maxObjects: 4096, maxHeads: 2};
  const commits = new Map(); let fault = null;
  IDBDatabase.prototype.transaction = function(...args) {
    const real = Reflect.apply(originalTransaction, this, args), database = this.name;
    const names = typeof args[0] === 'string' ? [args[0]] : [...args[0]];
    if (args[1] !== 'readwrite' || !names.includes('objects') || !names.includes('heads')) return real;
    const heads = [], objects = new Map(); let abort, lost = false;
    return new Proxy(real, {
      get(target, property) {
        if (property === 'error' && lost) return new DOMException('Lost acknowledgment', 'UnknownError');
        if (property === 'objectStore') return name => {
          const store = target.objectStore(name);
          return new Proxy(store, {get(object, key) {
            if (['put', 'add'].includes(key)) return (...values) => {
              if (name === 'heads') heads.push({head: values[1], next: values[0]});
              if (name === 'objects' && values[0] instanceof Uint8Array) objects.set(values[1], values[0].slice());
              if (fault?.database === database && fault.mode === 'quota' && !fault.used) {
                fault.used = true; throw new DOMException('Synthetic quota fault', 'QuotaExceededError');
              }
              return Reflect.apply(object[key], object, values);
            };
            const value = Reflect.get(object, key, object); return typeof value === 'function' ? value.bind(object) : value;
          }});
        };
        const value = Reflect.get(target, property, target); return typeof value === 'function' ? value.bind(target) : value;
      },
      set(target, property, value) {
        if (property === 'onabort') abort = value;
        if (property !== 'oncomplete') return Reflect.set(target, property, value, target);
        return Reflect.set(target, property, event => {
          commits.set(database, (commits.get(database) ?? 0) + 1);
          const selected = fault;
          const match = selected && selected.database === database && !selected.used && heads.some(row => {
            if (selected.phase === 'staging') return row.head === 'staging';
            if (selected.phase === 'application') return row.head === 'primary';
            if (row.head !== 'operations') return false;
            const envelope = decode(objects.get(row.next)), body = decode(envelope[1]);
            return body[1] === 1 && (selected.phase === 'prepared' ? body[2][9] === 0 : body[2][9] > 0);
          });
          if (!match) return value.call(target, event);
          selected.used = true; selected.entered.resolve();
          void selected.release.promise.then(() => {
            if (selected.mode === 'unknown') { lost = true; abort.call(target, event); }
            else value.call(target, event);
          });
        }, target);
      },
    });
  };
  const arm = (namespace, phase, mode = 'unknown') => {
    fault = {database: 'prismpm.browser.v1/' + namespace, phase, mode, used: false, entered: deferred(), release: deferred()};
    releases.push(fault.release); return fault;
  };
  const guestCalls = () => observer.calls.filter(row => row.role === 'Guest').length;
  const publicSnapshot = value => [value.application, value.policy, value.resources.map(row => [row.resource, row.slot, row.context, row.maximum, row.publicKey, row.principal])];
  const context = 'journal-effect/1', entry = 'PrismPM.Fixture.fixtureEchoBytes';
  const invoke = value => ['guest', [0, [digests.Guest, entry, context, value]]];
  const keep = async promise => { const host = await promise; hosts.push(host); return host; };
  async function setup({maximum = 1024, signMaximum = 3, signContext = context} = {}) {
    const application = crypto.getRandomValues(new Uint8Array(32)), policyRef = crypto.getRandomValues(new Uint8Array(32));
    const policy = [application, policyRef, ['shared'], [
      ['journal', 'shared', 'prismpm/browser-operation-journal/1', 65536], ['sign', 'shared', signContext, signMaximum],
    ]];
    const custody = await openCredentialCustody({wire: artifacts.Custody, wireDigest: digests.Custody, policy: encode(policy), mode: 'initialize'});
    custodies.push(custody); const publicBindings = credentialPublicBindings(custody), key = publicBindings.resources[0].publicKey;
    const namespace = 'journal-' + crypto.randomUUID(), applicationStore = 'application-' + crypto.randomUUID();
    const manifest = [application, crypto.getRandomValues(new Uint8Array(32)), [67108864, 67108864, 16384], [
      ['guest', [0, [digests.Guest, entry, context, 2097151, 2097152, 256]]], ['random', [1]], ['digest', [2]],
      ['sign', [3, [key, signContext]]], ['store', [5, [applicationStore, 1048576, 4096, 2]]],
    ]];
    const closure = encode([1, digests.Journal, digests.Partition, digests.Effects, [['guest', digests.Guest]]]);
    const binding = [application, manifest[1], await digest(closure), policyRef,
      await digest(encode([1, manifest, publicSnapshot(publicBindings)])), key, namespace, 'operations', 'staging', maximum];
    const options = (mode = 'initialize') => ({wire: artifacts.Journal.slice(), wireDigest: digests.Journal.slice(),
      partition: artifacts.Partition.slice(), partitionDigest: digests.Partition.slice(), binding: encode(binding), custody,
      signingResource: 'journal', mode, effects: {wire: artifacts.Effects.slice(), wireDigest: digests.Effects.slice(), manifest: encode(manifest),
        guests: [{resource: 'guest', bytes: artifacts.Guest.slice()}], signers: [{resource: 'sign', custody}]}});
    const readStore = async () => { const store = await openStore(namespace, limits); stores.push(store); return store; };
    return {policy, custody, binding, manifest, namespace, applicationStore, closure, options, readStore, publicBindings};
  }
  async function rawDatabase(namespace, change) {
    const database = await new Promise((resolve, reject) => {
      const request = Reflect.apply(originalOpen, indexedDB, ['prismpm.browser.v1/' + namespace, 1]);
      request.onsuccess = () => resolve(request.result); request.onerror = () => reject(request.error);
    });
    try { return await new Promise((resolve, reject) => {
      const tx = Reflect.apply(originalTransaction, database, [['objects', 'heads'], 'readwrite']);
      tx.oncomplete = resolve; tx.onabort = () => reject(tx.error); change(tx);
    }); } finally { database.close(); }
  }
  async function competing(f, kind = 'journal') {
    const frame = document.createElement('iframe'); frames.push(frame);
    const ready = deferred();
    frame.addEventListener('load', ready.resolve, {once: true});
    frame.srcdoc = '<script type="module">import {openCompetingTab} from "' + location.origin
      + '/browser-fixture.mjs"; window.openJournalTab = openCompetingTab;</script>';
    document.body.append(frame); await bounded(ready.promise);
    check(typeof frame.contentWindow.openJournalTab === 'function', 'actual second browsing context loaded');
    const remote = await frame.contentWindow.openJournalTab(JSON.stringify({input,
      policy: [...encode(f.policy)], binding: [...encode(f.binding)], manifest: [...encode(f.manifest)],
      closure: [...f.closure], kind}));
    return {async submit(bytes) { return remote.submit([...bytes]); }, async stage(bytes) { return remote.stage([...bytes]); },
      async close() {
        const observed = JSON.parse(await remote.close());
        observer.calls.push(...observed.calls); observer.large.push(...observed.large);
        for (const [role, maximum] of Object.entries(observed.maximum)) observer.maximum[role] = Math.max(observer.maximum[role] ?? 0, maximum);
        frame.remove(); return observed;
      }};
  }
  try {
    const first = await setup();
    await rejects(open(first.options('open')), 'journal-missing');
    const supplied = first.options(), opening = keep(open(supplied));
    supplied.binding.fill(0); supplied.wire.fill(0); supplied.partition.fill(0);
    supplied.effects.manifest.fill(0); supplied.effects.wire.fill(0); supplied.effects.guests[0].bytes.fill(0);
    supplied.effects.signers.length = 0;
    const host = await opening;
    check(Object.isFrozen(host) && Object.keys(host).sort().join(',') === 'close,receipt,refresh,status,submit', 'closed durable journal API');
    await rejects(open(first.options()), 'journal-unavailable');
    check(host.status().records === 0 && host.receipt() === null, 'explicit genesis has no invented operation');
    cases.push('explicit-genesis-open-capture');

    const result = decode(await host.submit(encode(invoke(Uint8Array.of(1, 2, 3)))));
    check(same(result, [0, Uint8Array.of(0x7b, 1, 2, 3)]), 'actual guest executes once after Prepared');
    check(host.status().records === 2 && host.status().nextOperation === 1, 'actual terminal increments retained history');
    const prior = host.receipt(), reopened = await keep(open(first.options('open')));
    check(same(decode(reopened.receipt().response), result) && same(reopened.receipt().request, prior.request), 'authenticated prior terminal request/result recover');
    check(reopened.status().nextOperation === 1 && !reopened.status().blocked, 'new runtime retains global operation order');
    await reopened.submit(encode(['random', [1, 7]]));
    const next = decode(reopened.receipt().request), before = decode(prior.request);
    check(next[3] === 0 && !same(next[2], before[2]) && reopened.receipt().operation === 1, 'runtime session/counter differ from durable operation identity');
    cases.push('actual-execution-terminal-reopen');

    for (const maximum of [1, 2, 65536, 1048576]) {
      const f = await setup({signMaximum: maximum}), signing = await keep(open(f.options()));
      const payload = new Uint8Array(maximum).fill(17), signed = decode(await signing.submit(encode(['sign', [3, payload]])));
      check(signed[0] === 3 && await verifyBytes(f.binding[5], context, payload, signed[1]), 'actual source-bounded signature');
      check(!await verifyBytes(f.binding[5], 'prismpm/browser-operation-journal/1', payload, signed[1]), 'signing contexts cannot alias journal authority');
      if (maximum < 1048576) {
        const records = signing.status().records, calls = guestCalls();
        await rejects(signing.submit(encode(['sign', [3, new Uint8Array(maximum + 1)]])), 'journal-uncertain');
        const recovered = await keep(open(f.options('open')));
        check(recovered.status().records === records && !recovered.status().blocked && guestCalls() === calls,
          'source-bound signing limit rejects before Prepared or effect execution');
      }
    }
    cases.push('opaque-custody-source-bounds-before-prepare');

    const alias = await setup(); alias.manifest[3].find(row => row[0] === 'store')[1][1][0] = alias.namespace;
    alias.binding[4] = await digest(encode([1, alias.manifest, publicSnapshot(alias.publicBindings)]));
    await rejects(open(alias.options()), 'storage-alias');
    check(!(await indexedDB.databases()).some(row => row.name === 'prismpm.browser.v1/' + alias.namespace), 'alias rejects before private or application store initialization');
    cases.push('journal-namespace-isolation');

    const signingAlias = await setup({signContext: 'prismpm/browser-operation-journal/1'});
    await rejects(open(signingAlias.options()), 'signing-alias');
    check(!(await indexedDB.databases()).some(row => row.name === 'prismpm.browser.v1/' + signingAlias.namespace), 'application cannot acquire journal signing authority');
    cases.push('journal-signing-domain-isolation');

    for (const [phase, expectExecuted, recoveredTerminal] of [['prepared', false, false], ['terminal', true, true]]) {
      const f = await setup(), current = await keep(open(f.options())), gate = arm(f.namespace, phase), before = guestCalls();
      const outcome = observed(current.submit(encode(invoke(Uint8Array.of(7)))));
      await bounded(gate.entered.promise);
      check(guestCalls() === before + Number(expectExecuted), 'primitive custody while actual acknowledgment is withheld');
      const retained = await keep(open(f.options('open')));
      check(retained.status().blocked !== recoveredTerminal, 'reopen derives uncertainty only from actual durable terminal');
      gate.release.resolve(); const result = await bounded(outcome); fault = null;
      check(result.error?.code === 'journal-uncertain' && current.status().blocked, 'lost acknowledgment is never called success');
      await current.refresh();
      check(current.status().blocked !== recoveredTerminal && current.status().runtimeClosed, 'refresh never reopens old effect session');
      if (recoveredTerminal) {
        check(current.receipt() !== null, 'already durable terminal is recoverable');
        await rejects(current.submit(encode(invoke(Uint8Array.of(8)))), 'journal-reopen-required');
      } else {
        check(current.receipt() === null, 'missing terminal cannot be synthesized');
        await rejects(current.submit(encode(invoke(Uint8Array.of(8)))), 'journal-uncertain');
      }
      check(guestCalls() === before + Number(expectExecuted), 'recovery never retries actual effect');
    }
    cases.push('lost-prepared-and-terminal-acknowledgments');

    for (const phase of ['staging', 'prepared']) {
      const f = await setup(), current = await keep(open(f.options())), gate = arm(f.namespace, phase, 'hold'), before = guestCalls();
      const outcome = observed(current.submit(encode(invoke(Uint8Array.of(7)))));
      await bounded(gate.entered.promise); current.close(); gate.release.resolve();
      check((await bounded(outcome)).error?.code === 'journal-closed', 'close rejects acknowledgment without release'); fault = null;
      const reopened = await keep(open(f.options('open')));
      check(reopened.status().blocked === (phase === 'prepared') && guestCalls() === before, 'close preserves exact durable staging/prepared boundary');
    }
    cases.push('close-during-staging-and-prepared');

    const uncertain = await setup(), active = await keep(open(uncertain.options())), gate = arm(uncertain.applicationStore, 'application');
    const content = Uint8Array.of(10, 20, 30), objectId = 'sha256:' + hex(await digest(content));
    const writing = observed(active.submit(encode(['store', [7, ['primary', [0], objectId, [content]]]])));
    await bounded(gate.entered.promise); gate.release.resolve();
    check((await bounded(writing)).error?.code === 'journal-uncertain', 'actual effect lost acknowledgment remains uncertain'); fault = null;
    const reader = await openStore(uncertain.applicationStore, limits); stores.push(reader);
    check(same((await reader.readHead('primary')).bytes, content), 'effect really committed before acknowledgment loss');
    const unknown = await keep(open(uncertain.options('open')));
    check(unknown.status().blocked && unknown.receipt() === null, 'similar application data is not a terminal receipt');
    await rejects(unknown.submit(encode(['store', [7, ['primary', [0], objectId, [content]]]])), 'journal-uncertain');
    check((commits.get('prismpm.browser.v1/' + uncertain.applicationStore) ?? 0) === 1, 'actual unknown operation is never retried');
    cases.push('unknown-real-application-commit-retained');

    const rejected = await setup(), rejectedHost = await keep(open(rejected.options()));
    const rejectedObject = Uint8Array.of(51), rejectedId = 'sha256:' + hex(await digest(rejectedObject));
    const rejection = await rejectedHost.submit(encode(['store', [7, ['primary', [1, 'sha256:' + 'a'.repeat(64)], rejectedId, [rejectedObject]]]]));
    check(same(decode(rejection), [8, [3]]), 'actual known head conflict is an admitted rejected completion');
    const rejectedStore = await rejected.readStore(), rejectedRecord = decode(decode((await rejectedStore.readHead('operations')).bytes)[1])[2];
    check(rejectedRecord[9] === 2 && rejectedRecord[11] === 8 && rejectedHost.status().records === 2, 'known rejection persisted a signed terminal');
    const rejectedReopen = await keep(open(rejected.options('open')));
    check(!rejectedReopen.status().blocked && same(rejectedReopen.receipt().response, rejection), 'known rejected terminal recovers without executing again');
    cases.push('actual-rejected-effect-durable-terminal');

    const missing = await setup(), damaged = await keep(open(missing.options()));
    await damaged.submit(encode(invoke(Uint8Array.of(4))));
    const stored = await missing.readStore(), head = await stored.readHead('operations');
    const record = decode(decode(head.bytes)[1])[2], chunk = 'sha256:' + hex(record[10][2][0]);
    await rawDatabase(missing.namespace, tx => { tx.objectStore('objects').delete(chunk); });
    await rejects(open(missing.options('open')), 'chunk-missing');
    cases.push('missing-chunk-fails-authenticated-replay');

    for (const maximum of [2, 3]) {
      const short = await setup({maximum}), exhausted = await keep(open(short.options()));
      await exhausted.submit(encode(invoke(Uint8Array.of(8)))); const beforeExhaustion = guestCalls();
      await rejects(exhausted.submit(encode(invoke(Uint8Array.of(9)))), 'model-rejected');
      check(guestCalls() === beforeExhaustion && (await keep(open(short.options('open')))).status().records === 2,
        'full history or one free record cannot release an operation without terminal capacity');
    }
    cases.push('finite-history-exhaustion');

    const init = await setup(), initializers = await Promise.all([observed(open(init.options())), observed(open(init.options()))]);
    check(initializers.filter(row => row.value).length === 1 && initializers.filter(row => row.error?.code === 'journal-unavailable').length === 1,
      'only one concurrent Initialize is acknowledged');
    hosts.push(initializers.find(row => row.value).value);
    const opened = await Promise.all([keep(open(init.options('open'))), keep(open(init.options('open')))]);
    check(opened.every(host => host.status().records === 0), 'concurrent Open shares exact authenticated genesis');
    cases.push('concurrent-initialize-and-open');

    const race = await setup(), winner = await keep(open(race.options())), loser = await competing(race);
    const held = arm(race.namespace, 'prepared', 'hold'), beforeRace = guestCalls();
    const winning = observed(winner.submit(encode(invoke(Uint8Array.of(81)))));
    await bounded(held.entered.promise);
    const losing = await loser.submit(encode(invoke(Uint8Array.of(82))));
    check(losing.error === 'journal-uncertain' && guestCalls() === beforeRace, 'CAS loser cannot execute before winner acknowledgment');
    held.release.resolve(); check((await bounded(winning)).value, 'winning actual Prepared completes'); fault = null;
    const other = await loser.close();
    check(other.calls.every(row => row.role !== 'Guest') && guestCalls() === beforeRace + 1, 'one actual guest across two browser realms');
    check((await keep(open(race.options('open')))).status().records === 2, 'winning terminal is the complete history');
    cases.push('actual-two-context-prepared-cas-race');

    const quota = await setup(), quotaHost = await keep(open(quota.options())), beforeQuota = guestCalls();
    arm(quota.namespace, 'staging', 'quota');
    await rejects(quotaHost.submit(encode(invoke(Uint8Array.of(1)))), 'journal-uncertain');
    check(fault.used, 'actual IDB write quota fault executed'); fault = null;
    check(guestCalls() === beforeQuota && (await keep(open(quota.options('open')))).status().records === 0,
      'aborted staging leaves no Prepared or effect');
    cases.push('quota-before-prepared-never-executes');

    const capacity = await setup(), capacityHost = await keep(open(capacity.options())), capacityStore = await capacity.readStore();
    let nextHead = null;
    for (let first = 0; first < 4095; first += 16) {
      const objects = Array.from({length: Math.min(16, 4095 - first)}, (_, offset) => {
        const bytes = new Uint8Array(4); new DataView(bytes.buffer).setUint32(0, first + offset); return bytes;
      });
      const next = 'sha256:' + hex(await digest(objects[0]));
      await capacityStore.commit({head: 'staging', expected: nextHead, next, objects}); nextHead = next;
    }
    let count;
    await rawDatabase(capacity.namespace, tx => { const request = tx.objectStore('objects').count(); request.onsuccess = () => { count = request.result; }; });
    check(count === 4096, 'actual configured store count reached with valid content-addressed objects');
    const beforeCapacity = guestCalls();
    await rejects(capacityHost.submit(encode(invoke(Uint8Array.of(1)))), 'journal-uncertain');
    check(guestCalls() === beforeCapacity && (await keep(open(capacity.options('open')))).status().records === 0,
      'object exhaustion cannot acknowledge Prepared or execute');
    cases.push('actual-object-count-exhaustion');

    const immutable = await setup(); await keep(open(immutable.options()));
    for (const [field, expected] of [[0, 'credential-mismatch'], [3, 'credential-mismatch'], [5, 'credential-mismatch'], [4, 'binding-mismatch'], [2, 'artifact-mismatch']]) {
      const option = immutable.options('open'), binding = decode(option.binding); binding[field][1] ^= 1; option.binding = encode(binding);
      await rejects(open(option), expected);
    }
    for (const field of ['wire', 'partition']) {
      const option = immutable.options('open'); option[field][option[field].length - 1] ^= 1;
      await rejects(open(option), 'artifact-mismatch');
    }
    const guestChanged = immutable.options('open'); guestChanged.effects.guests[0].bytes[guestChanged.effects.guests[0].bytes.length - 1] ^= 1;
    await rejects(open(guestChanged), 'artifact-mismatch');
    const complete = immutable.options('open'), oversized = new Uint8Array(67108864);
    complete.wire = oversized; complete.partition = oversized; complete.effects.wire = oversized;
    complete.effects.guests = [{resource: 'a', bytes: oversized}, {resource: 'b', bytes: new Uint8Array(1)}];
    const beforeCopies = copies;
    await rejects(open(complete), 'invalid-input');
    check(copies === beforeCopies, 'complete aggregate artifact budget rejects before captured native copies');
    cases.push('immutable-closure-and-precopy-budget');

    const corrupt = await setup(), corruptHost = await keep(open(corrupt.options()));
    await corruptHost.submit(encode(invoke(Uint8Array.of(44))));
    const corruptStore = await corrupt.readStore(), corruptHead = await corruptStore.readHead('operations');
    const corruptRecord = decode(decode(corruptHead.bytes)[1])[2], corruptChunk = 'sha256:' + hex(corruptRecord[10][2][0]);
    await rawDatabase(corrupt.namespace, tx => { tx.objectStore('objects').put(Uint8Array.of(4), corruptChunk); });
    await rejects(open(corrupt.options('open')), 'journal-unavailable');
    cases.push('corrupt-chunk-cannot-be-replayed');

    const forged = await setup(), signedHost = await keep(open(forged.options()));
    await signedHost.submit(encode(invoke(Uint8Array.of(90))));
    const signedStore = await forged.readStore(), signedHead = await signedStore.readHead('operations');
    const alteredEnvelope = decode(signedHead.bytes); alteredEnvelope[2][0] ^= 1;
    const alteredBytes = encode(alteredEnvelope), alteredId = 'sha256:' + hex(await digest(alteredBytes));
    await signedStore.commit({head: 'operations', expected: signedHead.id, next: alteredId, objects: [alteredBytes]});
    await rejects(open(forged.options('open')), 'signature-invalid');
    cases.push('content-addressed-forged-signature-rejected');

    const stagedFixture = await setup(), stagedEffects = await openStagedEffects(stagedFixture.options().effects); hosts.push(stagedEffects);
    const stagedBefore = guestCalls(), firstIntent = encode(invoke(Uint8Array.of(71))), secondIntent = encode(invoke(Uint8Array.of(72)));
    const firstPrepared = stagedEffects.prepare(firstIntent), secondPrepared = stagedEffects.prepare(secondIntent);
    check(decode(firstPrepared.request)[3] === 0 && decode(secondPrepared.request)[3] === 1, 'generated staged counter advances once per captured request');
    await rejects(Promise.resolve().then(() => stagedEffects.prepare(firstIntent)), 'model-rejected');
    const waiting = observed(secondPrepared.release()); await Promise.resolve();
    check(guestCalls() === stagedBefore, 'released waiter cannot bypass unacknowledged active request');
    const firstResponse = await firstPrepared.release(), secondResponse = (await waiting).value;
    check(same(decode(firstResponse), [0, Uint8Array.of(0x7b, 71)]) && same(decode(secondResponse), [0, Uint8Array.of(0x7b, 72)]), 'private release runs actual ordered requests once');
    await rejects(firstPrepared.release(), 'invalid-input');
    check(guestCalls() === stagedBefore + 2, 'release reuse never reruns a primitive');
    const closedEffects = await openStagedEffects(stagedFixture.options().effects); hosts.push(closedEffects);
    const never = [closedEffects.prepare(firstIntent), closedEffects.prepare(secondIntent)]; closedEffects.close();
    for (const item of never) await rejects(item.release(), 'host-closed');
    check(guestCalls() === stagedBefore + 2, 'closing unreleased active and waiter performs neither');
    cases.push('private-staged-counters-waiter-and-one-shot-release');

    // The same production transport used above handles an actual64MiB fixture;
    // this is not presented as an oversized admitted DK20 application request.
    const maximum = await setup();
    const transport = await openPayloads({wire: artifacts.Journal, wireDigest: digests.Journal,
      partition: artifacts.Partition, partitionDigest: digests.Partition,
      binding: encode(maximum.binding), artifacts: maximum.closure}); hosts.push(transport);
    const payload = new Uint8Array(67108864).fill(0x5a);
    for (let index = 0; index < 64; index++) payload[index * 1048576] = index;
    const descriptor = await transport.stage(payload, new Uint8Array(32));
    const loaded = await transport.load(descriptor);
    check(loaded.length === payload.length && loaded.every((byte, at) => byte === payload[at]), 'actual64MiB ordered chunk closure reopens');
    let objectCount;
    await rawDatabase(maximum.namespace, tx => {
      const count = tx.objectStore('objects').count(); count.onsuccess = () => { objectCount = count.result; };
    });
    check(objectCount === 69 && new Set(decode(descriptor)[2].map(hex)).size === 64,
      'actual64distinct1MiB objects plus five staging markers, with16objects in full batches');
    const payloadDigest = hex(await digest(loaded));
    cases.push('actual-64mib-shared-transport');

    const interleaved = await setup(), interleavedHost = await keep(open(interleaved.options()));
    const staged = await openPayloads({wire: artifacts.Journal, wireDigest: digests.Journal,
      partition: artifacts.Partition, partitionDigest: digests.Partition, binding: encode(interleaved.binding), artifacts: interleaved.closure});
    hosts.push(staged); const secondTransport = await competing(interleaved, 'transport'), firstBatch = arm(interleaved.namespace, 'staging', 'hold');
    const incomplete = observed(staged.stage(payload, new Uint8Array(32))); await bounded(firstBatch.entered.promise);
    const competingStage = await secondTransport.stage(Uint8Array.of(99)); check(competingStage.value, 'second browser actually advances staging head');
    firstBatch.release.resolve();
    check((await bounded(incomplete)).error?.code === 'head-conflict', 'stale next batch CAS cannot acknowledge complete payload'); fault = null;
    await rejects(staged.load(descriptor), 'chunk-missing');
    const secondObserved = await secondTransport.close();
    check(secondObserved.calls.every(row => row.role !== 'Guest') && interleavedHost.status().records === 0,
      'partial maximum transport confers no operation admission or execution');
    cases.push('actual-two-context-maximum-staging-race');

    await observer.flush();
    return {cases, calls: observer.calls, large: observer.large, maximum: observer.maximum,
      payload: {length: loaded.length, digest: payloadDigest, descriptor: hex(descriptor)}};
  } finally {
    for (const release of releases) release.resolve();
    for (const frame of frames) frame.remove();
    for (const host of hosts) { try { host.close(); } catch {} }
    for (const store of stores) store.close();
    for (const custody of custodies) closeCredentialCustody(custody);
    IDBDatabase.prototype.transaction = originalTransaction; observer.restore();
  }
}

// A separate real browsing context reopens native persisted custody and the
// same generated artifacts; the test receives observations, never an injected
// completion, signing key, storage adapter or execution permit.
export async function openCompetingTab(json) {
  const {input, policy, binding, manifest, closure, kind} = JSON.parse(json);
  const artifacts = Object.fromEntries(Object.entries(input).map(([role, bytes]) => [role, new Uint8Array(bytes)]));
  const digests = Object.fromEntries(await Promise.all(Object.entries(artifacts).map(async ([role, bytes]) => [role, await digest(bytes)])));
  const {openOperationJournal, openOperationPayloadStore} = await import('./operation-journal.mjs');
  const {openCredentialCustody, closeCredentialCustody} = await import('./credential-custody.mjs');
  const observer = observeArtifacts(input);
  const custody = await openCredentialCustody({wire: artifacts.Custody, wireDigest: digests.Custody, policy: new Uint8Array(policy), mode: 'open'});
  const base = {wire: artifacts.Journal, wireDigest: digests.Journal, partition: artifacts.Partition,
    partitionDigest: digests.Partition, binding: new Uint8Array(binding)};
  const host = kind === 'transport' ? await openOperationPayloadStore({...base, artifacts: new Uint8Array(closure)})
    : await openOperationJournal({...base, custody, signingResource: 'journal', mode: 'open',
      effects: {wire: artifacts.Effects, wireDigest: digests.Effects, manifest: new Uint8Array(manifest),
        guests: [{resource: 'guest', bytes: artifacts.Guest}], signers: [{resource: 'sign', custody}]}});
  return {
    async submit(bytes) { const result = await observed(host.submit(new Uint8Array(bytes))); return {value: result.value ? [...result.value] : undefined, error: result.error?.code}; },
    async stage(bytes) { const result = await observed(host.stage(new Uint8Array(bytes), new Uint8Array(32))); return {value: result.value ? [...result.value] : undefined, error: result.error?.code}; },
    async close() { host.close(); closeCredentialCustody(custody); await observer.flush(); observer.restore(); return JSON.stringify({calls: observer.calls, large: observer.large, maximum: observer.maximum}); },
  };
}
