// Acceptance-only trusted adapter composition, not a shipped application.
// Authorization and every state/replay/CAS-completion decision are generated.
export async function browserJournalFixture({wasmBytes, genesisTemplate}) {
  const cryptoHost = await import('/identity.mjs');
  const storageHost = await import('/store.mjs');
  const module = await WebAssembly.compile(new Uint8Array(wasmBytes));
  const empty = new Uint8Array(), zero = new Uint8Array(32);
  const hex = bytes => Array.from(bytes, byte => byte.toString(16).padStart(2, '0')).join('');
  const unhex = value => new Uint8Array(value.match(/../g)?.map(byte => parseInt(byte, 16)) ?? []);
  const equal = (a, b) => hex(a) === hex(b);
  const concat = (...parts) => {
    const out = new Uint8Array(parts.reduce((n, part) => n + part.length, 0));
    let offset = 0; for (const part of parts) { out.set(part, offset); offset += part.length; }
    return out;
  };
  const u24 = value => new Uint8Array([value >>> 16, value >>> 8 & 255, value & 255]);
  const read24 = (bytes, offset) => bytes[offset] * 65536 + bytes[offset + 1] * 256 + bytes[offset + 2];
  const check = (condition, message) => { if (!condition) throw Error(message); };
  const calls = [], cases = [];
  function generated(request) {
    const instance = new WebAssembly.Instance(module, {});
    const pointer = instance.exports.holo_alloc(request.length);
    new Uint8Array(instance.exports.memory.buffer, pointer, request.length).set(request);
    const result = BigInt.asUintN(64, instance.exports.holo_run(pointer, request.length));
    const offset = Number(result >> 32n), length = Number(result & 0xffffffffn);
    check(length <= 1166008 && offset + length <= instance.exports.memory.buffer.byteLength, 'generated output bounds');
    const bytes = new Uint8Array(instance.exports.memory.buffer, offset, length).slice();
    calls.push([hex(request), hex(bytes)]);
    return bytes;
  }
  function projection(op, envelope) {
    const bytes = generated(concat([6, op], envelope));
    check(bytes[0] === 0, 'generated envelope rejection');
    return bytes.slice(1);
  }
  const digest = async bytes => cryptoHost.digestBytes(bytes);
  const digestBytes = async bytes => unhex((await digest(bytes)).slice(7));
  const context = 'prismpm/workspace-event/1';
  async function authenticate(value) {
    const envelope = new Uint8Array(value);
    const key = projection(3, envelope), signature = projection(4, envelope);
    check(await cryptoHost.identityPrincipal(key) === 'sha256:' + hex(projection(6, envelope)), 'key-author mismatch');
    check(await digest(projection(2, envelope)) === 'sha256:' + hex(projection(7, envelope)), 'event identity mismatch');
    check(await cryptoHost.verifyBytes(key, context, projection(1, envelope), signature), 'signature failure');
    return {envelope, objectId: await digestBytes(envelope)};
  }
  function unpack(result) {
    check(result[0] === 0, 'modeled rejection ' + hex(result));
    const headLength = read24(result, 1), stateLength = read24(result, 4);
    check(7 + headLength + stateLength === result.length, 'closed generated plan');
    const head = result.slice(7, 7 + headLength);
    check(equal(generated(concat([0], head)), concat([0], head)), 'generated candidate-head closure');
    return {head, state: result.slice(7 + headLength)};
  }
  const appendRequest = (head, state, id, envelope) => concat([1], u24(head.length), u24(state.length), head, state, id, envelope);
  async function replay(target, load) {
    check(generated(concat([0], target))[0] === 0, 'target head syntax');
    let head = empty, state = empty;
    const count = new DataView(target.buffer, target.byteOffset, target.byteLength).getUint16(36);
    for (let index = 0; index < count; index++) {
      const id = 'sha256:' + hex(target.slice(70 + 64 * index, 102 + 64 * index));
      const bytes = await load(id); check(bytes !== null, 'missing replay object');
      const authenticated = await authenticate(bytes);
      const next = unpack(generated(concat([3], u24(target.length), target,
        appendRequest(head, state, authenticated.objectId, authenticated.envelope).slice(1))));
      head = next.head; state = next.state;
    }
    const finished = generated(concat([4], u24(target.length), u24(head.length), u24(state.length), target, head, state));
    check(finished[0] === 0, 'complete replay required');
    return {head, state: finished.slice(1)};
  }
  class FixtureAdapter {
    #store; #head = empty; #state = empty; #pending = new WeakMap();
    constructor(store) { this.#store = store; }
    snapshot() { return {head: this.#head.slice(), state: this.#state.slice()}; }
    async reopen() {
      const retained = await this.#store.readHead('workspace');
      const restored = retained === null ? {head: empty, state: empty}
        : await replay(retained.bytes, id => this.#store.readObject(id));
      this.#head = restored.head; this.#state = restored.state;
      return this.snapshot();
    }
    async prepare(value) {
      const authenticated = await authenticate(value);
      const candidate = unpack(generated(appendRequest(this.#head, this.#state, authenticated.objectId, authenticated.envelope)));
      const expected = this.#head.length ? await digest(this.#head) : null;
      const next = await digest(candidate.head);
      const intent = generated(concat([5], expected === null ? zero : unhex(expected.slice(7)),
        unhex(next.slice(7)), authenticated.objectId));
      check(intent[0] === 0, 'generated commit intent');
      const session = concat([0], crypto.getRandomValues(new Uint8Array(32)), expected === null ? zero : unhex(expected.slice(7)),
        unhex(next.slice(7)), await digestBytes(intent.slice(1)));
      const token = Object.freeze({});
      this.#pending.set(token, {candidate, authenticated, expected, next, session});
      return token;
    }
    async commit(token) {
      check(arguments.length === 1 && token && typeof token === 'object' && this.#pending.has(token), 'unowned completion');
      const pending = this.#pending.get(token); this.#pending.delete(token);
      let status = 0, observed;
      try {
        const returned = await this.#store.commit({head: 'workspace', expected: pending.expected, next: pending.next,
          objects: [pending.authenticated.envelope, pending.candidate.head]});
        check(returned === pending.next, 'storage result binding'); observed = unhex(returned.slice(7));
      } catch (error) {
        const known = {'head-conflict': 1, 'store-limit': 2, 'storage-quota': 3, 'store-closed': 4, 'storage-unavailable': 5};
        check(Object.hasOwn(known, error.code), 'unexpected storage failure'); status = known[error.code];
        if (status === 1) { const current = await this.#store.readHead('workspace'); observed = current === null ? zero : unhex(current.id.slice(7)); }
        else observed = pending.expected === null ? zero : unhex(pending.expected.slice(7));
      }
      const receipt = concat([status], pending.session.slice(1), observed);
      const result = generated(concat([2], pending.session, receipt));
      check(result.length === 130, 'modeled terminal completion');
      // Only the generated terminal result can authorize candidate promotion.
      if (result[0] === 0) { this.#head = pending.candidate.head; this.#state = pending.candidate.state; }
      return result.slice();
    }
  }
  const owner = await cryptoHost.createIdentity(), contributor = await cryptoHost.createIdentity(), reader = await cryptoHost.createIdentity();
  const template = unhex(genesisTemplate);
  async function signed(identity, prior, action, body = empty, signingContext = context) {
    const event = new Uint8Array(134 + body.length); event.set(template.slice(133, 267));
    event[1] = action; event.set(unhex(identity.principal.slice(7)), 98);
    if (prior.head.length) {
      const count = new DataView(prior.head.buffer, prior.head.byteOffset, prior.head.byteLength).getUint16(36);
      event.set(prior.head.slice(prior.head.length - 64, prior.head.length - 32), 66);
      new DataView(event.buffer).setUint16(130, count);
    }
    new DataView(event.buffer).setUint16(132, body.length); event.set(body, 134);
    const envelope = concat([0x50,0x57,0x45,1], identity.publicKey, new Uint8Array(64), event);
    envelope.set(await digestBytes(projection(2, envelope)), 167);
    envelope.set(await cryptoHost.signBytes(identity, signingContext, projection(1, envelope)), 69);
    return envelope;
  }
  async function rejected(name, operation) { let failed = false; try { await operation(); } catch { failed = true; } check(failed, name); cases.push(name); }
  let store = await storageHost.openStore('journal-real'); await store.saveIdentity(owner);
  let adapter = new FixtureAdapter(store);
  const genesis = await signed(owner, adapter.snapshot(), 0);
  const zeroWorkspace = genesis.slice(); zeroWorkspace.fill(0,135,167);
  zeroWorkspace.set(await digestBytes(projection(2,zeroWorkspace)),167);
  zeroWorkspace.set(await cryptoHost.signBytes(owner,context,projection(1,zeroWorkspace)),69);
  await rejected('zero workspace rejected before candidate or commit',()=>adapter.prepare(zeroWorkspace));
  check(adapter.snapshot().state.length===0&&(await store.readHead('workspace'))===null,'zero workspace cannot create state or head');
  const token = await adapter.prepare(genesis);
  await rejected('unowned completion token', () => adapter.commit({}));
  await rejected('caller supplied success receipt', () => adapter.commit(token, {status: 0}));
  check((await adapter.commit(token))[0] === 0, 'durable genesis');
  await rejected('replayed completion token', () => adapter.commit(token));
  cases.push('real P-256 genesis and atomic object/head commit');
  // ECDSA's second valid s representative changes the immutable envelope
  // object, not the generated unsigned-event identity. Verify the real crypto
  // result before asking the generated reducer to reject the semantic replay.
  const alternateSignature = genesis.slice();
  const order=BigInt('0xffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551');
  const originalS=BigInt('0x'+hex(alternateSignature.slice(101,133)));
  alternateSignature.set(unhex((order-originalS).toString(16).padStart(64,'0')),101);
  check(await cryptoHost.verifyBytes(owner.publicKey,context,projection(1,alternateSignature),projection(4,alternateSignature)),
    'alternate object signature is genuinely valid');
  check(await digest(alternateSignature)!==await digest(genesis),'distinct content object');
  check(equal(projection(7,alternateSignature),projection(7,genesis)),'unchanged unsigned event identity');
  check(equal(generated(appendRequest(adapter.snapshot().head,adapter.snapshot().state,await digestBytes(alternateSignature),alternateSignature)),new Uint8Array([16,5])),
    'generated replay rejection distinguishes event identity from content identity');
  await rejected('distinct signed object cannot replay the same event ID',()=>adapter.prepare(alternateSignature));
  async function accept(identity, action, body = empty) {
    const envelope = await signed(identity, adapter.snapshot(), action, body);
    check((await adapter.commit(await adapter.prepare(envelope)))[0] === 0, 'accepted durable event');
    return envelope;
  }
  await accept(owner, 1, unhex(contributor.principal.slice(7)));
  await accept(owner, 2, unhex(reader.principal.slice(7)));
  await accept(contributor, 4, new TextEncoder().encode('retained contributor message'));
  await rejected('reader cannot post', async () => adapter.prepare(await signed(reader, adapter.snapshot(), 4, new TextEncoder().encode('forbidden'))));
  await accept(owner, 3, unhex(contributor.principal.slice(7)));
  await rejected('revoked contributor cannot post', async () => adapter.prepare(await signed(contributor, adapter.snapshot(), 4, new TextEncoder().encode('forbidden'))));
  const retained = adapter.snapshot(); store.close(); store = await storageHost.openStore('journal-real');
  check((await store.loadIdentity()).principal === owner.principal, 'persisted nonextractable key possession');
  adapter = new FixtureAdapter(store); const restored = await adapter.reopen();
  check(equal(restored.head, retained.head) && equal(restored.state, retained.state), 'fresh verified event replay');
  cases.push('reopened IndexedDB authenticated replay and identity');
  const first = await signed(owner, adapter.snapshot(), 4, new TextEncoder().encode('first branch'));
  const second = await signed(owner, adapter.snapshot(), 4, new TextEncoder().encode('second branch'));
  const firstToken = await adapter.prepare(first), secondToken = await adapter.prepare(second);
  check((await adapter.commit(firstToken))[0] === 0, 'first local CAS'); const committed = adapter.snapshot();
  check((await adapter.commit(secondToken))[0] === 33, 'stale CAS is explicit conflict');
  check(equal(adapter.snapshot().state, committed.state), 'conflict cannot promote state');
  check(await store.readObject(await digest(second)) === null, 'conflict writes no orphan event');
  cases.push('concurrent plans preserve committed state on real CAS conflict');
  const changed = bytes => { const next = bytes.slice(); next[69] ^= 1; return next; };
  await rejected('tampered replay envelope', () => replay(committed.head, async id => changed(await store.readObject(id))));
  await rejected('missing replay envelope', () => replay(committed.head, async () => null));
  const reordered = committed.head.slice(), firstRow = reordered.slice(38, 102);
  reordered.set(reordered.slice(102, 166), 38); reordered.set(firstRow, 102);
  await rejected('authenticated events in wrong replay order', () => replay(reordered, id => store.readObject(id)));
  await rejected('valid signed stale branch cannot prepare again', () => adapter.prepare(second));
  await rejected('wrong signing context', async () => adapter.prepare(await signed(owner, adapter.snapshot(), 4, new TextEncoder().encode('wrong'), 'prismpm/workspace-event/2')));
  const corruptId = first.slice(); corruptId[167] ^= 1;
  await rejected('valid signature with forged event ID', () => adapter.prepare(corruptId));
  const forgedAuthor = await signed(owner, adapter.snapshot(), 4, new TextEncoder().encode('forged'));
  forgedAuthor.set(unhex(reader.principal.slice(7)), 231);
  forgedAuthor.set(await digestBytes(projection(2, forgedAuthor)), 167);
  forgedAuthor.set(await cryptoHost.signBytes(owner, context, projection(1, forgedAuthor)), 69);
  await rejected('valid signature cannot claim another principal', () => adapter.prepare(forgedAuthor));
  const closedToken = await adapter.prepare(await signed(owner, adapter.snapshot(), 4, new TextEncoder().encode('closed')));
  store.close(); check((await adapter.commit(closedToken))[0] === 36, 'real closed-store failure is explicit');
  check(equal(adapter.snapshot().state, committed.state), 'closed store cannot promote state');
  cases.push('real storage failure never promotes state');
  const limited = await storageHost.openStore('journal-limit', {maxObjectBytes:1048576,maxObjects:1,maxHeads:1});
  const limitedAdapter = new FixtureAdapter(limited);
  check((await limitedAdapter.commit(await limitedAdapter.prepare(genesis)))[0] === 34, 'real transactional store limit');
  check((await limited.readHead('workspace')) === null && (await limited.readObject(await digest(genesis))) === null,
    'aborted store-limit transaction has no partial object or head');
  check(limitedAdapter.snapshot().state.length === 0, 'store limit cannot promote state'); limited.close();
  cases.push('real object limit aborts the whole transaction');
  return {cases, calls, count: new DataView(committed.head.buffer).getUint16(36)};
}
