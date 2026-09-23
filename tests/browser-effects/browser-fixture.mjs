// Acceptance only: real generated Wasm, WebCrypto and IndexedDB. No shipping
// adapter callback or synthetic successful model/guest response is supplied.
export async function runFixture(input) {
  const NativeByteArray = Uint8Array;
  let largeCopies = 0, effectModule;
  // Observe actual allocation by the captured identity byte-copy constructor;
  // all allocations still use the real native constructor and real byte slots.
  globalThis.Uint8Array = new Proxy(NativeByteArray, {construct(target, arguments_) {
    if (typeof arguments_[0] === 'number' && arguments_[0] > 65) largeCopies++;
    return Reflect.construct(target, arguments_);
  }});
  try { effectModule = await import('./effects.mjs'); }
  finally { globalThis.Uint8Array = NativeByteArray; }
  const {openEffects, EffectHostError} = effectModule;
  const {encodeEffectWire: encode, decodeEffectWire: decode} = await import('./effects-wire.mjs');
  const {createIdentity} = await import('./identity.mjs');
  const {openStore} = await import('./store.mjs');
  const check = (condition, message) => { if (!condition) throw Error(message); };
  const hex = bytes => {
    const chunks = [];
    for (let offset = 0; offset < bytes.length; offset += 4096) {
      chunks.push(Array.from(bytes.subarray(offset, offset + 4096), byte => byte.toString(16).padStart(2, '0')).join(''));
    }
    return chunks.join('');
  };
  const bytes = value => new Uint8Array(value);
  const text = value => new TextEncoder().encode(value);
  const same = (left, right) => hex(encode(left)) === hex(encode(right));
  const deferred = () => {
    let resolve;
    const promise = new Promise(done => { resolve = done; });
    return {promise, resolve};
  };
  const pause = () => new Promise(resolve => setTimeout(resolve, 0));
  const observed = promise => promise.then(value => ({value}), error => ({error}));
  async function bounded(promise, label) {
    let timer;
    try { return await Promise.race([promise, new Promise((_, reject) => {
      timer = setTimeout(() => reject(Error('timed out: ' + label)), 20000);
    })]); } finally { clearTimeout(timer); }
  }
  async function rejects(operation, code, detail) {
    const outcome = await observed(operation);
    check(outcome.error instanceof EffectHostError && outcome.error.code === code,
      'expected private effect ' + code + ', got ' + outcome.error?.code);
    if (detail !== undefined) check(outcome.error.detail === detail, 'expected effect detail ' + detail);
    check(outcome.error.cause === undefined, 'exception payload remains private');
    return outcome.error;
  }
  const wire = bytes(input.wire), guest = bytes(input.guest);
  const originalCompile = WebAssembly.compile, OriginalInstance = WebAssembly.Instance;
  const originalTransaction = IDBDatabase.prototype.transaction;
  const moduleKinds = new WeakMap(), calls = [], maximum = {}, cases = [], hosts = [], stores = [];
  const releases = [], restorations = [], commits = new Map(), instances = {Wire: 0, Guest: 0};
  const wireHex = hex(wire), guestHex = hex(guest);
  let fault = null, wireFault = null;
  const digest = async value => new Uint8Array(await crypto.subtle.digest('SHA-256', value));
  const patch = (object, key, value) => {
    const before = Object.getOwnPropertyDescriptor(object, key);
    Object.defineProperty(object, key, {configurable: true, writable: true, value});
    const restore = () => { if (before) Object.defineProperty(object, key, before); else delete object[key]; };
    restorations.push(restore); return restore;
  };
  WebAssembly.compile = async function(value) {
    const captured = new Uint8Array(value), encoded = hex(captured);
    const module = await Reflect.apply(originalCompile, WebAssembly, [captured]);
    check(encoded === wireHex || encoded === guestHex, 'only supplied actual generated artifacts are compiled');
    moduleKinds.set(module, encoded === wireHex ? 'Wire' : 'Guest');
    return module;
  };
  WebAssembly.Instance = class {
    constructor(module, imports) {
      const real = new OriginalInstance(module, imports), exports = {...real.exports};
      const kind = moduleKinds.get(module);
      check(kind === 'Wire' || kind === 'Guest', 'actual artifact-bound instance');
      instances[kind]++;
      exports.holo_run = (pointer, length) => {
        const requestBytes = new Uint8Array(exports.memory.buffer, pointer, length).slice();
        const request = hex(requestBytes);
        const output = BigInt.asUintN(64, real.exports.holo_run(pointer, length));
        const response = hex(new Uint8Array(exports.memory.buffer,
          Number(output >> 32n), Number(output & 0xffffffffn)));
        calls.push({kind, request, response});
        maximum[kind] = Math.max(maximum[kind] ?? 0, exports.memory.buffer.byteLength);
        if (kind === 'Wire' && wireFault && !wireFault.used) {
          const message = decode(requestBytes);
          const beginning = wireFault.kind === 'bootstrap-session' && message[1] === 0;
          const admitting = wireFault.kind.startsWith('admission-') && message[1] === 1 && message[2][5][0] === 1;
          const completing = (['promotion', 'trap'].includes(wireFault.kind) || wireFault.kind.startsWith('completion-')) && message[1] === 2;
          const closing = wireFault.kind.startsWith('close-') && message[1] === 3;
          if ((beginning || admitting || completing || closing)
            && hex(beginning ? message[2][0] : message[2][0][0]) === wireFault.application) {
            wireFault.used = true; wireFault.retained = message[2];
            // Preserve the genuine generated request/response above for native
            // replay. Only the host's subsequent observation is faulted.
            if (wireFault.kind === 'trap') throw new WebAssembly.RuntimeError('injected after actual generated completion');
            const at = Number(output >> 32n), size = Number(output & 0xffffffffn);
            const changed = decode(new Uint8Array(exports.memory.buffer, at, size).slice());
            check(changed[1] === 0, 'observation fault follows actual generated success');
            if (beginning) {
              check(same(changed[2], [message[2], message[3], 0, false, false, [0], [0]]),
                'actual generated begin binds the supplied fresh session');
              changed[2][1][0] ^= 1;
            } else if (admitting) {
              check(same(changed[2][5], message[2][5]) && same(changed[2][6], [1, message[3]]),
                'actual generated admission preserves its active operation and binds the waiter');
              if (wireFault.kind === 'admission-slot') changed[2][6][1][5][1][3][0] ^= 1;
              else if (wireFault.kind === 'admission-active') changed[2][5][1][5][1][3][0][0] ^= 1;
              else if (wireFault.kind === 'admission-counter') changed[2][2]--;
              else if (wireFault.kind === 'admission-closed') changed[2][3] = true;
              else if (wireFault.kind === 'admission-uncertain') changed[2][4] = true;
              else throw Error('closed admission fault inventory');
            } else if (closing) {
              check(same(changed[2], [message[2][0], message[2][1], message[2][2], true,
                message[2][4], message[2][5], message[2][6]]), 'actual generated close preserves complete pending custody');
              if (wireFault.kind === 'close-counter') changed[2][2]++;
              else if (wireFault.kind === 'close-open') changed[2][3] = false;
              else if (wireFault.kind === 'close-uncertain') changed[2][4] = !changed[2][4];
              else if (wireFault.kind === 'close-active') changed[2][5][1][5][1][3][0][0] ^= 1;
              else if (wireFault.kind === 'close-waiter') changed[2][6][1][5][1][3][0] ^= 1;
              else throw Error('closed termination fault inventory');
            } else if (wireFault.kind === 'completion-cleared-unknown') {
              check(message[3][1][0] === 9 && changed[2][4] === true
                && same(changed[2][5], message[2][5]) && same(changed[2][6], message[2][6]),
                'actual generated unknown completion retains uncertainty and both operations');
              changed[2][4] = false;
            } else {
              check(changed[2][5][0] === 1, 'actual generated completion promoted its waiter');
              if (wireFault.kind === 'promotion') changed[2][5][1][5][1][3][0] ^= 1;
              else if (wireFault.kind === 'completion-counter') changed[2][2]++;
              else if (wireFault.kind === 'completion-closed') changed[2][3] = true;
              else if (wireFault.kind === 'completion-uncertain') changed[2][4] = true;
              else throw Error('closed completion fault inventory');
            }
            const corrupted = encode(changed);
            check(corrupted.length === size, 'negative observation preserves framing length');
            new Uint8Array(exports.memory.buffer, at, size).set(corrupted);
          }
        }
        return output;
      };
      return {exports};
    }
  };
  IDBDatabase.prototype.transaction = function(...arguments_) {
    const transaction = Reflect.apply(originalTransaction, this, arguments_);
    const names = typeof arguments_[0] === 'string' ? [arguments_[0]] : [...arguments_[0]];
    if (arguments_[1] !== 'readwrite' || !names.includes('objects') || !names.includes('heads')) return transaction;
    const name = this.name;
    let abort, lost = false;
    return new Proxy(transaction, {
      get(target, key) {
        if (key === 'error' && lost) return new DOMException('Completion acknowledgment lost', 'UnknownError');
        const value = Reflect.get(target, key, target);
        return typeof value === 'function' ? value.bind(target) : value;
      },
      set(target, key, value) {
        if (key === 'onabort') abort = value;
        if (key !== 'oncomplete') return Reflect.set(target, key, value, target);
        return Reflect.set(target, key, event => {
          commits.set(name, (commits.get(name) ?? 0) + 1);
          const selected = fault;
          if (!selected || selected.database !== name || selected.used) return value.call(target, event);
          selected.used = true;
          // This callback runs only after the real IndexedDB transaction has
          // committed. Holding/dropping its acknowledgment never fakes a write.
          selected.entered.resolve();
          void selected.release.promise.then(() => {
            if (selected.mode === 'unknown') { lost = true; abort.call(target, event); }
            else value.call(target, event);
          });
        }, target);
      },
    });
  };
  const arm = (namespace, mode) => {
    const selected = {database: 'prismpm.browser.v1/' + namespace, mode,
      used: false, entered: deferred(), release: deferred()};
    releases.push(selected.release); fault = selected; return selected;
  };
  const count = kind => calls.filter(call => call.kind === kind).length;
  const completions = () => calls.filter(call => call.kind === 'Wire'
    && decode(Uint8Array.from(call.request.match(/../g), byte => parseInt(byte, 16)))[1] === 2).length;
  const context = 'effect-fixture/1', entry = 'PrismPM.Fixture.fixtureEchoBytes';
  const abc = text('abc');
  const abcDigest = 'sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad';
  let wireDigest, guestDigest;
  async function setup(namespace = 'effect-host-' + crypto.randomUUID()) {
    const identity = await createIdentity();
    const manifest = [crypto.getRandomValues(new Uint8Array(32)), crypto.getRandomValues(new Uint8Array(32)),
      [1048576, 1048576, 16384], [
        ['guest', [0, [guestDigest.slice(), entry, context, 64, 65, 256]]],
        ['random', [1]], ['digest', [2]],
        ['sign', [3, [identity.publicKey.slice(), context]]],
        ['verify', [4, [identity.publicKey.slice(), context]]],
        ['store', [5, [namespace, 1048576, 4096, 64]]],
      ]];
    const options = () => ({wire: wire.slice(), wireDigest: wireDigest.slice(), manifest: encode(manifest),
      guests: [{resource: 'guest', bytes: guest.slice()}],
      signers: [{resource: 'sign', identity: {privateKey: identity.privateKey,
        principal: identity.principal, publicKey: identity.publicKey.slice()}}]});
    const open = async value => { const host = await openEffects(value ?? options()); hosts.push(host); return host; };
    const readStore = async () => { const store = await openStore(namespace); stores.push(store); return store; };
    return {namespace, identity, manifest, options, open, readStore,
      commits: () => commits.get('prismpm.browser.v1/' + namespace) ?? 0};
  }
  const invoke = value => ['guest', [0, [guestDigest.slice(), entry, context, value]]];
  const submit = async (host, intent) => decode(await host.submit(encode(intent)));
  try {
    wireDigest = await digest(wire); guestDigest = await digest(guest);
    const f = await setup(), supplied = f.options(), opening = f.open(supplied);
    // Every mutable bootstrap byte buffer is destroyed after the call returns,
    // before any awaited digest or identity verification has finished.
    for (const value of [supplied.wire, supplied.wireDigest, supplied.manifest,
      supplied.guests[0].bytes, supplied.signers[0].identity.publicKey]) value.fill(0);
    supplied.guests.length = 0; supplied.signers.length = 0;
    const host = await opening;
    check(Object.isFrozen(host) && Object.keys(host).sort().join(',') === 'close,status,submit', 'closed private effect API');
    for (const name of ['complete', 'state', 'session', 'manifest', 'resources', 'signers', 'stores']) {
      check(host[name] === undefined, 'private effect custody: ' + name);
    }
    check(same(await submit(host, invoke(abc)), [0, Uint8Array.of(0x7b, ...abc)]), 'actual generated guest output');
    cases.push('closed-api-bootstrap-capture');

    const bootstrap = await setup();
    wireFault = {application: hex(bootstrap.manifest[0]), kind: 'bootstrap-session', used: false};
    const beforeBootstrapGuest = instances.Guest;
    await rejects(bootstrap.open(), 'invalid-generated-output');
    check(wireFault.used && instances.Guest === beforeBootstrapGuest,
      'substituted bootstrap session rejects before guest instantiation');
    check(!(await indexedDB.databases()).some(database => database.name === 'prismpm.browser.v1/' + bootstrap.namespace),
      'substituted bootstrap session rejects before storage initialization');
    wireFault = null; cases.push('bootstrap-session-observation-rejection');

    const aggregate = await setup(), aggregateOptions = aggregate.options();
    const padding = new NativeByteArray(64 * 1024 * 1024);
    const remainder = padding.length - wire.length - guest.length + 1;
    check(remainder > 0 && remainder <= padding.length, 'actual generated artifacts fit the negative aggregate fixture');
    aggregateOptions.guests.push(...['budget-a', 'budget-b', 'budget-c'].map(resource => ({resource, bytes: padding})),
      {resource: 'budget-tail', bytes: padding.subarray(0, remainder)});
    check(wire.length + aggregateOptions.guests.reduce((total, row) => total + row.bytes.length, 0) === 268435457,
      'one byte beyond the complete wire-plus-guest budget');
    const observedEffects = {digest: 0, compile: 0, storage: 0};
    const originalDigestForBudget = crypto.subtle.digest, originalCompileForBudget = WebAssembly.compile;
    const originalOpenForBudget = indexedDB.open;
    const restoreBudget = [
      patch(crypto.subtle, 'digest', function(...args) { observedEffects.digest++; return Reflect.apply(originalDigestForBudget, this, args); }),
      patch(WebAssembly, 'compile', function(...args) { observedEffects.compile++; return Reflect.apply(originalCompileForBudget, this, args); }),
      patch(indexedDB, 'open', function(...args) { observedEffects.storage++; return Reflect.apply(originalOpenForBudget, this, args); }),
    ];
    const copiesBeforeBudget = largeCopies;
    try {
      await rejects(aggregate.open(aggregateOptions), 'invalid-input');
      check(largeCopies === copiesBeforeBudget && Object.values(observedEffects).every(value => value === 0),
        'aggregate rejection precedes large copies, hashing, compilation and storage');
    } finally { for (const restore of restoreBudget.reverse()) restore(); }
    check(!(await indexedDB.databases()).some(database => database.name === 'prismpm.browser.v1/' + aggregate.namespace),
      'aggregate rejection does not initialize durable resources');
    cases.push('aggregate-artifact-budget-early-rejection');

    const random = await submit(host, ['random', [1, 32]]);
    check(random[0] === 1 && random[1] instanceof Uint8Array && random[1].length === 32, 'real bounded random result');
    check(same(await submit(host, ['digest', [2, abc]]), [2, abcDigest]), 'real SHA-256 known answer');
    const signature = await submit(host, ['sign', [3, abc]]);
    check(signature[0] === 3 && signature[1].length === 64, 'real P-256 signature width');
    check(same(await submit(host, ['verify', [4, [abc, signature[1]]]]), [4, true]), 'real signature verification');
    check(same(await submit(host, ['verify', [4, [text('abd'), signature[1]]]]), [4, false]), 'modified signed bytes reject');
    const domain = text('prismpm/browser-signature/1\0'), scope = text(context);
    const signed = new Uint8Array(domain.length + 2 + scope.length + abc.length);
    signed.set(domain); new DataView(signed.buffer).setUint16(domain.length, scope.length);
    signed.set(scope, domain.length + 2); signed.set(abc, domain.length + 2 + scope.length);
    const key = await crypto.subtle.importKey('raw', f.identity.publicKey,
      {name: 'ECDSA', namedCurve: 'P-256'}, false, ['verify']);
    check(await crypto.subtle.verify({name: 'ECDSA', hash: 'SHA-256'}, key, signature[1], signed), 'independent WebCrypto verifies captured signing context');
    cases.push('all-cryptographic-effects');

    check(same(await submit(host, ['store', [7, ['primary', [0], abcDigest, [abc]]]]), [7, abcDigest]), 'real atomic commit acknowledgment');
    check(same(await submit(host, ['store', [5, abcDigest]]), [5, [1, abc]]), 'actual committed object read');
    check(same(await submit(host, ['store', [6, 'primary']]), [6, [1, [abcDigest, abc]]]), 'actual committed head read');
    check(same(await submit(host, ['store', [5, 'sha256:' + '0'.repeat(64)]]), [5, [0]]), 'missing object remains absent');
    check(same(await submit(host, ['store', [6, 'missing']]), [6, [0]]), 'missing head remains absent');
    await rejects(host.submit(encode(['store', [7, ['primary', [0], abcDigest, [abc]]]])), 'effect-rejected', 3);
    check(f.commits() === 1 && host.status().pending === 0 && !host.status().uncertain, 'definitive conflict consumes once without writing');
    const reader = await f.readStore();
    check(same((await reader.readHead('primary')).bytes, abc), 'independent durable read after actual acknowledgment');
    const reopened = await f.open();
    check(same(await submit(reopened, ['store', [5, abcDigest]]), [5, [1, abc]]), 'fresh host reopens durable object');
    reopened.close(); cases.push('all-storage-effects-conflict-reopen');

    for (const change of [
      value => { value.wireDigest[0] ^= 1; },
      value => { value.guests[0].bytes[0] ^= 1; },
    ]) {
      const value = f.options(); change(value); await rejects(f.open(value), 'artifact-mismatch');
    }
    const wrongResource = f.options(); wrongResource.guests[0].resource = 'other';
    await rejects(f.open(wrongResource), 'artifact-mismatch');
    const smaller = structuredClone(f.manifest); smaller[3][0][1][1][5] = 1;
    await rejects(f.open({...f.options(), manifest: encode(smaller)}), 'invalid-generated-module');
    // Alter only the explicit maximum in the actual generated guest's memory
    // section. Rebind its real digest so this probes metadata admission, not a
    // hash mismatch; the altered artifact is never accepted or instantiated.
    const alteredMemory = guest.slice(); let at = 8, changedLimit = false;
    const unsigned = () => {
      let value = 0;
      for (let shift = 0; shift <= 28; shift += 7) {
        check(at < alteredMemory.length, 'generated memory framing remains bounded');
        const byte = alteredMemory[at++]; value += (byte & 127) * 2 ** shift;
        if (byte < 128) return value;
      }
      throw Error('invalid generated memory framing');
    };
    while (at < alteredMemory.length) {
      const id = alteredMemory[at++], length = unsigned(), end = at + length;
      if (id === 5) {
        check(unsigned() === 1 && unsigned() === 1, 'actual generated guest has one bounded memory');
        unsigned(); const maximumAt = at;
        check(unsigned() === 256 && at === maximumAt + 2
          && alteredMemory[maximumAt] === 0x80 && alteredMemory[maximumAt + 1] === 0x02,
        'actual fixture guest maximum is the canonical two-byte 256-page limit');
        alteredMemory[maximumAt] = 0x81; changedLimit = true; break;
      }
      at = end;
    }
    check(changedLimit, 'actual generated guest memory limit located');
    const oversized = f.options(), oversizedManifest = structuredClone(f.manifest);
    oversizedManifest[3][0][1][1][0] = await digest(alteredMemory);
    oversized.manifest = encode(oversizedManifest); oversized.guests[0].bytes = alteredMemory;
    const beforeInstances = instances.Guest;
    await rejects(f.open(oversized), 'invalid-generated-module');
    check(instances.Guest === beforeInstances, 'digest-bound oversized generated guest rejects before instantiation');
    for (const name of ['session', 'state', 'complete']) {
      await rejects(f.open({...f.options(), [name]: new Uint8Array(32)}), 'invalid-input');
    }
    for (const key of ['hidden', Symbol('hidden')]) for (const select of [value => value,
      value => value.guests[0], value => value.signers[0], value => value.signers[0].identity]) {
      const value = f.options(); Object.defineProperty(select(value), key, {value: true});
      await rejects(f.open(value), 'invalid-input');
    }
    const unused = await setup(), unusedOptions = unused.options();
    unusedOptions.guests.push({resource: 'ungranted', bytes: guest.slice()});
    await rejects(unused.open(unusedOptions), 'resource-mismatch');
    check(!(await indexedDB.databases()).some(database => database.name === 'prismpm.browser.v1/' + unused.namespace),
      'unused bootstrap binding rejected before storage initialization');
    for (const name of ['guests', 'signers']) {
      const value = f.options(); let called = false;
      value[name].map = () => { called = true; return []; };
      await rejects(f.open(value), 'invalid-input');
      check(!called, 'caller array method never runs');
    }
    const accessor = f.options(); let reads = 0;
    Object.defineProperty(accessor.guests[0], 'bytes', {get() { reads++; return guest; }});
    await rejects(f.open(accessor), 'invalid-input'); check(reads === 0, 'guest artifact getter never runs');
    const hole = f.options(); hole.guests = Array(1);
    await rejects(f.open(hole), 'invalid-input');
    cases.push('artifact-resource-memory-bootstrap-rejection');

    await rejects(host.submit(encode(['random', [1, 1]]), encode([1])), 'invalid-input');
    for (const value of [encode(['random', [1, 1], new Uint8Array(32)]),
      encode([f.manifest[0], f.manifest[1], new Uint8Array(32), 0, 'random', [1, 1]]),
      new Uint8Array(new SharedArrayBuffer(1)), new Uint8Array(), {}]) {
      await rejects(host.submit(value), 'invalid-input');
    }
    await rejects(host.submit(encode(['unknown', [1, 1]])), 'model-rejected', 10);
    for (const index of [0, 1, 2]) {
      const wrong = invoke(abc);
      if (index === 0) wrong[1][1][index][0] ^= 1; else wrong[1][1][index] += '.wrong';
      await rejects(host.submit(encode(wrong)), 'model-rejected', 11);
    }
    await rejects(host.submit(encode(invoke(new Uint8Array(65)))), 'model-rejected', 11);
    const full = new Uint8Array(64).fill(19);
    check(same(await submit(host, invoke(full)), [0, Uint8Array.of(0x7b, ...full)]), 'actual generated maximum 64-byte input and 65-byte output');
    const captured = encode(invoke(abc)), pendingCapture = host.submit(captured);
    structuredClone(captured.buffer, {transfer: [captured.buffer]});
    check(same(decode(await pendingCapture), [0, Uint8Array.of(0x7b, ...abc)]), 'synchronous detached intent capture');
    cases.push('request-bindings-closed-completion-detached-capture');

    const queue = await setup(), queuedHost = await queue.open();
    const held = text('private-held-digest'), entered = deferred(), released = deferred(); releases.push(released);
    const realDigest = crypto.subtle.digest; let selected = false, digestCalls = 0;
    const restoreDigest = patch(crypto.subtle, 'digest', async function(algorithm, value) {
      const matches = hex(new Uint8Array(value.buffer, value.byteOffset, value.byteLength)) === hex(held);
      const result = Reflect.apply(realDigest, this, [algorithm, value]);
      if (!matches) return result;
      digestCalls++; const output = await result;
      if (!selected) { selected = true; entered.resolve(); await released.promise; }
      return output;
    });
    const first = observed(queuedHost.submit(encode(['digest', [2, held]])));
    await bounded(entered.promise, 'real held digest');
    const beforeGuest = count('Guest');
    const second = observed(queuedHost.submit(encode(invoke(abc))));
    check(queuedHost.status().pending === 2 && queuedHost.status().nextOperation === 2, 'one active and one modeled waiter');
    check(count('Guest') === beforeGuest, 'waiter has not executed before promotion');
    await rejects(queuedHost.submit(encode(['random', [1, 1]])), 'model-rejected', 4);
    released.resolve();
    check((await first).error === undefined && same(decode((await second).value), [0, Uint8Array.of(0x7b, ...abc)]), 'actual waiter promotion succeeds');
    restoreDigest();
    check(digestCalls === 1 && count('Guest') === beforeGuest + 1 && queuedHost.status().pending === 0, 'each private operation executes exactly once');
    queuedHost.close(); cases.push('bounded-queue-once-only-promotion');

    for (const kind of ['admission-slot', 'admission-active', 'admission-counter', 'admission-closed', 'admission-uncertain']) {
      const corrupted = await setup(), corruptedHost = await corrupted.open(), gate = arm(corrupted.namespace, 'hold');
      const writing = observed(corruptedHost.submit(encode(['store', [7, ['primary', [0], abcDigest, [abc]]]])));
      await bounded(gate.entered.promise, 'actual durable commit before ' + kind);
      const beforeGuest = count('Guest');
      wireFault = {application: hex(corrupted.manifest[0]), kind, used: false};
      const waiting = observed(corruptedHost.submit(encode(invoke(abc))));
      check(wireFault.used && corruptedHost.status().closed && corruptedHost.status().pending === 1,
        'invalid admission is terminal before taking custody of a new waiter');
      const outcomes = await bounded(Promise.all([writing, waiting]), 'settled ' + kind + ' outcomes');
      check(outcomes[0].error?.code === 'effect-outcome-unknown' && outcomes[1].error?.code === 'invalid-generated-output',
        'invalid admission preserves unknown durable outcome and rejects the unexecuted intent');
      const closes = calls.filter(call => call.kind === 'Wire').map(call =>
        decode(Uint8Array.from(call.request.match(/../g), byte => parseInt(byte, 16))))
        .filter(message => message[1] === 3 && hex(message[2][0][0]) === wireFault.application);
      check(closes.length > 0 && closes.every(message => same(message[2], wireFault.retained)),
        'admission termination uses last good complete modeled state');
      const beforeCompletion = completions();
      gate.release.resolve(); await pause();
      check(completions() === beforeCompletion, 'late completion cannot reenter admission-closed generated session');
      check(count('Guest') === beforeGuest && corrupted.commits() === 1,
        'invalid admission never retries durable effects or executes its waiter');
      const retained = await corrupted.readStore();
      check(same((await retained.readHead('primary')).bytes, abc), 'admission observation fault preserves actual durable content');
      wireFault = null; fault = null; cases.push(kind + '-fault-custody');
    }

    const failing = text('provider-failure');
    const restoreFailure = patch(crypto.subtle, 'digest', function(algorithm, value) {
      if (hex(new Uint8Array(value.buffer, value.byteOffset, value.byteLength)) === hex(failing)) throw new DOMException('Unavailable', 'OperationError');
      return Reflect.apply(realDigest, this, [algorithm, value]);
    });
    await rejects(host.submit(encode(['digest', [2, failing]])), 'effect-rejected', 2);
    restoreFailure();
    check(!host.status().uncertain && !host.status().closed && host.status().pending === 0, 'known provider rejection stays definitive');
    check(same(await submit(host, ['digest', [2, abc]]), [2, abcDigest]), 'known rejection admits subsequent effect');
    cases.push('known-adapter-failure-consumes-once');

    for (const mode of ['unknown', 'close']) {
      const durable = await setup(), durableHost = await durable.open(), gate = arm(durable.namespace, mode);
      const beforeGuest = count('Guest');
      const writing = observed(durableHost.submit(encode(['store', [7, ['primary', [0], abcDigest, [abc]]]])));
      const waiting = observed(durableHost.submit(encode(invoke(abc))));
      await bounded(gate.entered.promise, 'actual durable commit before ' + mode);
      check(durable.commits() === 1 && durableHost.status().pending === 2 && count('Guest') === beforeGuest,
        'actual committed active record and unexecuted waiter remain in custody');
      const visible = await durable.readStore();
      check(same((await visible.readHead('primary')).bytes, abc), 'write is actually durable before acknowledgment');
      let beforeCompletion;
      if (mode === 'close') {
        durableHost.close(); durableHost.close(); beforeCompletion = completions();
        check(durableHost.status().closed && durableHost.status().pending === 2, 'close retains both pending records');
      }
      gate.release.resolve();
      const outcomes = await bounded(Promise.all([writing, waiting]), 'settled ' + mode + ' outcomes');
      check(outcomes[0].error?.code === 'effect-outcome-unknown', 'durable effect never claims rollback');
      check(outcomes[1].error?.code === (mode === 'close' ? 'host-closed' : 'effect-outcome-unknown'), 'waiter is not presented as completed');
      await pause();
      check(count('Guest') === beforeGuest && durable.commits() === 1 && durableHost.status().pending === 2,
        'unknown or closed outcome never retries or promotes waiter');
      if (mode === 'unknown') {
        check(durableHost.status().uncertain && !durableHost.status().closed, 'unknown barrier retains modeled session without inventing closure');
        const before = count('Wire');
        await rejects(durableHost.submit(encode(invoke(abc))), 'effect-outcome-unknown');
        check(count('Wire') === before, 'unknown barrier prevents further admission or execution');
      } else {
        check(completions() === beforeCompletion, 'late completion cannot reenter closed generated session');
        await rejects(durableHost.submit(encode(invoke(abc))), 'host-closed');
      }
      check(same((await visible.readHead('primary')).bytes, abc), 'retained durable content remains available after ' + mode);
      durableHost.close(); fault = null; cases.push(mode + '-actual-durable-custody');
    }
    for (const kind of ['promotion', 'trap', 'completion-counter', 'completion-closed', 'completion-uncertain', 'completion-cleared-unknown']) {
      const corrupted = await setup(), corruptedHost = await corrupted.open();
      const beforeGuest = count('Guest');
      const gate = kind === 'completion-cleared-unknown' ? arm(corrupted.namespace, 'unknown') : null;
      wireFault = {application: hex(corrupted.manifest[0]), kind, used: false};
      const writing = observed(corruptedHost.submit(encode(['store', [7, ['primary', [0], abcDigest, [abc]]]])));
      const waiting = observed(corruptedHost.submit(encode(invoke(abc))));
      if (gate) { await bounded(gate.entered.promise, 'actual committed unknown completion'); gate.release.resolve(); }
      const outcomes = await bounded(Promise.all([writing, waiting]), 'actual generated completion ' + kind + ' fault');
      check(wireFault.used && outcomes[0].error?.code === 'effect-outcome-unknown', 'corrupted completion keeps active outcome unknown');
      check(outcomes[1].error?.code === 'host-unavailable', 'corrupted completion never resolves its unexecuted waiter');
      check(corruptedHost.status().closed && corruptedHost.status().pending === 2,
        'corrupted completion retains both private pending records before terminating');
      check(count('Guest') === beforeGuest && corrupted.commits() === 1,
        'corrupted completion neither promotes nor retries an actual durable effect');
      const closes = calls.filter(call => call.kind === 'Wire').map(call =>
        decode(Uint8Array.from(call.request.match(/../g), byte => parseInt(byte, 16))))
        .filter(message => message[1] === 3 && hex(message[2][0][0]) === wireFault.application);
      check(closes.length > 0 && closes.every(message => same(message[2], wireFault.retained)),
        'fault termination uses last good complete modeled custody');
      const retained = await corrupted.readStore();
      check(same((await retained.readHead('primary')).bytes, abc), 'negative observation cannot erase actual durable bytes');
      const before = count('Wire');
      await rejects(corruptedHost.submit(encode(invoke(abc))), 'host-closed');
      check(count('Wire') === before, 'faulted host cannot accept caller retry or completion');
      wireFault = null; fault = null; cases.push(kind + '-completion-fault-custody');
    }
    for (const kind of ['close-counter', 'close-open', 'close-uncertain', 'close-active', 'close-waiter']) {
      const corrupted = await setup(), corruptedHost = await corrupted.open(), gate = arm(corrupted.namespace, 'hold');
      const beforeGuest = count('Guest');
      const writing = observed(corruptedHost.submit(encode(['store', [7, ['primary', [0], abcDigest, [abc]]]])));
      const waiting = observed(corruptedHost.submit(encode(invoke(abc))));
      await bounded(gate.entered.promise, 'actual durable commit before ' + kind);
      wireFault = {application: hex(corrupted.manifest[0]), kind, used: false};
      let closeError;
      try { corruptedHost.close(); } catch (error) { closeError = error; }
      check(wireFault.used && closeError instanceof EffectHostError && closeError.code === 'invalid-generated-output',
        'invalid close observation is rejected before replacing modeled state');
      check(corruptedHost.status().closed && corruptedHost.status().pending === 2,
        'invalid close observation retains terminal host and both pending records');
      const outcomes = await bounded(Promise.all([writing, waiting]), 'settled ' + kind + ' outcomes');
      check(outcomes[0].error?.code === 'effect-outcome-unknown' && outcomes[1].error?.code === 'host-closed',
        'invalid close never resolves an unknown write or its unexecuted waiter');
      const closes = calls.filter(call => call.kind === 'Wire').map(call =>
        decode(Uint8Array.from(call.request.match(/../g), byte => parseInt(byte, 16))))
        .filter(message => message[1] === 3 && hex(message[2][0][0]) === wireFault.application);
      check(closes.length === 2 && closes.every(message => same(message[2], wireFault.retained)),
        'close fault cleanup reuses the entire last good state');
      const beforeCompletion = completions();
      gate.release.resolve(); await pause();
      check(completions() === beforeCompletion && count('Guest') === beforeGuest && corrupted.commits() === 1,
        'close fault never retries durable effects or promotes its waiter');
      const retained = await corrupted.readStore();
      check(same((await retained.readHead('primary')).bytes, abc), 'close observation fault preserves actual durable content');
      wireFault = null; fault = null; cases.push(kind + '-fault-custody');
    }
    host.close();
    check(host.status().closed, 'idempotent close remains terminal');
    await rejects(host.submit(encode(['random', [1, 1]])), 'host-closed');

    // Execute the genuine generated fixture guest, not Workspace or Journal.
    // Their published input/output budget pairs only test compatible admission;
    // the final case reaches this fixture's actual two-MiB output boundary.
    // Keep these full transcripts last: planted host defects fail their earlier
    // owning assertions naturally, without a conditional maximum-case bypass.
    for (const [name, inputMaximum, outputMaximum] of [
      ['workspace-budget-compatibility', 1104664, 1100428],
      ['journal-budget-compatibility', 1235980, 1166008],
      ['two-mib-output-bound', 2097151, 2097152],
    ]) {
      const large = await setup();
      large.manifest[2] = [2097152, 2097152, 16384];
      large.manifest[3][0][1][1][3] = inputMaximum;
      large.manifest[3][0][1][1][4] = outputMaximum;
      const largeHost = await large.open();
      const payload = new NativeByteArray(Math.min(inputMaximum, outputMaximum - 1));
      for (let index = 0; index < payload.length; index++) payload[index] = index % 251;
      const result = await submit(largeHost, invoke(payload));
      check(result[0] === 0 && result[1] instanceof NativeByteArray
        && result[1].length === payload.length + 1 && result[1][0] === 0x7b
        && payload.every((byte, index) => result[1][index + 1] === byte),
      'actual generated guest complete bounded output: ' + name);
      if (name === 'two-mib-output-bound') {
        check(result[1].length === 2097152, 'actual generated guest reaches exact two-MiB output');
        const beforeGuest = count('Guest');
        await rejects(largeHost.submit(encode(invoke(new NativeByteArray(2097152)))), 'model-rejected', 11);
        check(count('Guest') === beforeGuest, 'guest input budget plus one rejects before execution');
      }
      largeHost.close(); cases.push('generated-guest-' + name);
    }
    return {cases, calls, maximum};
  } finally {
    for (const release of releases) release.resolve();
    for (const host of hosts) { try { host.close(); } catch { /* Preserve primary failure. */ } }
    for (const store of stores) store.close();
    for (const restore of restorations.reverse()) restore();
    IDBDatabase.prototype.transaction = originalTransaction;
    WebAssembly.compile = originalCompile; WebAssembly.Instance = OriginalInstance;
  }
}
