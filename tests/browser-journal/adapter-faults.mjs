import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { withBrowser } from '../../sdk/browser/browser-test-server.mjs';
import { captureGeneratedCalls, replayNative } from './replay.mjs';

export async function verifyJournalFaults({wasmBytes, native, work, mutant, transcriptMutant}) {
let source = readFileSync(new URL('../../sdk/browser/journal.mjs', import.meta.url), 'utf8');
if (mutant === 'signature') {
  const original = 'if (!await verifyBytes(key, context, project(1), signature))';
  assert.equal(source.split(original).length, 2);
  source = source.replace(original, 'if (false && !await verifyBytes(key, context, project(1), signature))');
} else {
  assert.equal(mutant, undefined);
}

  const { cases, generatedCalls } = await withBrowser(async ({ browser, baseURL }) => {
    const page = await browser.newPage();
    await page.addInitScript(captureGeneratedCalls);
    await page.route('**/journal.mjs', route => route.fulfill({ contentType: 'text/javascript', body: source }));
    await page.goto(baseURL);
    return page.evaluate(async value => {
      const { openJournal } = await import('/journal.mjs');
      const { createIdentity, signBytes, digestBytes } = await import('/identity.mjs');
      const { openStore } = await import('/store.mjs');
      const module = await WebAssembly.compile(new Uint8Array(value));
      const cases = [];
      const check = (condition, message) => { if (!condition) throw Error(message); };
      const hex = bytes => Array.from(bytes, byte => byte.toString(16).padStart(2, '0')).join('');
      const unhex = text => Uint8Array.from(text.match(/../g), byte => parseInt(byte, 16));
      const bytes = id => unhex(id.slice(7));
      const same = (left, right) => hex(left) === hex(right);
      const join = (...parts) => {
        const output = new Uint8Array(parts.reduce((sum, part) => sum + part.length, 0));
        let offset = 0;
        for (const part of parts) { output.set(part, offset); offset += part.length; }
        return output;
      };
      function project(operation, envelope) {
        const input = join([6, operation], envelope);
        const instance = new WebAssembly.Instance(module);
        const pointer = instance.exports.holo_alloc(input.length);
        new Uint8Array(instance.exports.memory.buffer, pointer, input.length).set(input);
        const result = BigInt.asUintN(64, instance.exports.holo_run(pointer, input.length));
        const output = new Uint8Array(instance.exports.memory.buffer, Number(result >> 32n), Number(result & 0xffffffffn));
        check(output[0] === 0, 'fixture projection rejected');
        return output.slice(1);
      }
      const owner = await createIdentity(), contributor = await createIdentity(), reader = await createIdentity();
      const empty = () => ({ head: new Uint8Array(), state: new Uint8Array() });
      async function signed(identity, previous, action, body = new Uint8Array(), context = 'prismpm/workspace-event/1') {
        const event = new Uint8Array(134 + body.length);
        event[0] = 1; event[1] = action;
        event.fill(9, 2, 34); event.fill(1, 34, 66);
        event.set(bytes(identity.principal), 98);
        if (previous.head.length) {
          event.set(previous.head.slice(-64, -32), 66);
          new DataView(event.buffer).setUint16(130, new DataView(previous.head.buffer).getUint16(36));
        }
        new DataView(event.buffer).setUint16(132, body.length);
        event.set(body, 134);
        const envelope = join([80, 87, 69, 1], identity.publicKey, new Uint8Array(64), event);
        envelope.set(bytes(await digestBytes(project(2, envelope))), 167);
        envelope.set(await signBytes(identity, context, project(1, envelope)), 69);
        return envelope;
      }
      async function reject(name, operation, code) {
        let caught;
        try { await operation(); } catch (error) { caught = error; }
        check(caught && (!code || caught.code === code), `${name}: expected ${code}, received ${caught?.code}`);
        cases.push(name);
        return caught;
      }
      let store = await openStore('adapter-actual');
      await store.saveIdentity(owner);
      let journal = await openJournal(module, store, 'workspace');
      const genesis = await signed(owner, empty(), 0);
      const original = genesis.slice();
      const pending = journal.append(genesis);
      genesis.fill(0);
      await pending;
      check(await store.readObject(await digestBytes(original)) !== null, 'captured envelope committed');
      const snapshot = journal.snapshot(); snapshot.head.fill(0); snapshot.state.fill(0);
      check(journal.snapshot().head[0] === 80, 'snapshot alias escaped');
      cases.push('captured input and defensive snapshot with real atomic genesis');
      await reject('caller authentication and receipt are not accepted', () => journal.append(original, { verified: true }), 'invalid-input');
      const post = text => new TextEncoder().encode(text);
      async function append(identity, action, body) {
        return journal.append(await signed(identity, journal.snapshot(), action, body));
      }
      await append(owner, 1, bytes(contributor.principal));
      await append(owner, 2, bytes(reader.principal));
      await append(contributor, 4, post('retained contributor message'));
      await reject('reader cannot write', () => append(reader, 4, post('forbidden')), 'model-rejected');
      await append(owner, 3, bytes(contributor.principal));
      await reject('revoked contributor cannot write', () => append(contributor, 4, post('forbidden')), 'model-rejected');
      const retained = journal.snapshot();
      store.close(); store = await openStore('adapter-actual');
      journal = await openJournal(module, store, 'workspace');
      check(same(retained.head, journal.snapshot().head) && same(retained.state, journal.snapshot().state), 'replay differs');
      check((await store.loadIdentity()).principal === owner.principal, 'persisted identity differs');
      cases.push('persisted identity and complete authenticated replay');
      const valid = await signed(owner, journal.snapshot(), 4, post('valid'));
      const corrupt = valid.slice(); corrupt[69] ^= 1;
      await reject('signature mutation cannot append', () => journal.append(corrupt), 'signature-invalid');
      const corruptId = valid.slice(); corruptId[167] ^= 1;
      await reject('unsigned event ID substitution rejected', () => journal.append(corruptId), 'event-id-mismatch');
      await reject('wrong signing context rejected', async () => journal.append(
        await signed(owner, journal.snapshot(), 4, post('wrong'), 'prismpm/workspace-event/2')), 'signature-invalid');
      await reject('caller snapshot cannot seed state', () => journal.refresh(retained), 'invalid-input');

      // Two independently replayed adapters, one real IndexedDB head.
      const secondStore = await openStore('adapter-actual');
      const second = await openJournal(module, secondStore, 'workspace');
      const other = await signed(owner, second.snapshot(), 4, post('other branch'));
      await journal.append(valid);
      const stale = second.snapshot();
      await reject('cross-adapter CAS conflict rejects candidate', () => second.append(other), 'storage-rejected');
      check(same(second.snapshot().state, stale.state), 'conflict promoted state');
      check(await store.readObject(await digestBytes(other)) === null, 'conflict stored orphan event');
      await reject('conflict requires authenticated refresh', () => second.append(other), 'replay-required');
      await second.refresh();
      check(same(journal.snapshot().state, second.snapshot().state), 'conflict recovery replay differs');
      cases.push('real CAS conflict recovery preserves exactly committed history');
      secondStore.close();

      // Inject failures at the trusted storage boundary, retaining real crypto,
      // generated transitions and actual IndexedDB transactions underneath.
      const wrapper = overrides => ({
        readHead: (...args) => store.readHead(...args),
        readObject: (...args) => store.readObject(...args),
        commit: (...args) => store.commit(...args), ...overrides,
      });
      for (const method of ['readHead', 'readObject']) {
        let failing = false;
        const guarded = await openJournal(module, wrapper({ [method]: async (...args) => {
          if (failing) throw Object.assign(Error('private storage payload must not escape'), {
            code: 'unregistered-backend-failure', payload: 'private payload',
          });
          return store[method](...args);
        } }), 'workspace');
        const before = guarded.snapshot(); failing = true;
        const error = await reject(`unknown ${method} error is normalized`, () => guarded.refresh(), 'storage-outcome-unknown');
        check(error.name === 'JournalAdapterError' && error.message === 'storage-outcome-unknown'
          && error.detail === null && !Object.hasOwn(error, 'cause') && !Object.hasOwn(error, 'payload'),
          'unknown storage exception leaked its original payload');
        check(same(before.head, guarded.snapshot().head) && same(before.state, guarded.snapshot().state),
          'unknown read exposed partial replay');
        await reject(`unknown ${method} failure blocks writes`, () => guarded.append(valid), 'replay-required');
        failing = false; await guarded.refresh();
        check(same(before.state, guarded.snapshot().state), 'unknown read recovery differs');
        cases.push(`unknown ${method} error recovers only by complete replay`);
      }
      let corruptReads = false;
      const lastObject = 'sha256:' + hex(journal.snapshot().head.slice(-32));
      const replayAdapter = await openJournal(module, wrapper({ readObject: async id => {
        const object = await store.readObject(id);
        if (corruptReads && id === lastObject && object !== null) object[69] ^= 1;
        return object;
      } }), 'workspace');
      const beforeReplay = replayAdapter.snapshot(); corruptReads = true;
      await reject('corrupt replay cannot replace prior state', () => replayAdapter.refresh(), 'object-corrupt');
      check(same(beforeReplay.state, replayAdapter.snapshot().state), 'partial replay visible');
      await reject('failed replay blocks subsequent writes', () => replayAdapter.append(valid), 'replay-required');

      let missingReads = false;
      const missingAdapter = await openJournal(module, wrapper({ readObject: async id =>
        missingReads && id === lastObject ? null : store.readObject(id) }), 'workspace');
      const beforeMissing = missingAdapter.snapshot(); missingReads = true;
      await reject('missing final replay object cannot expose a partial prefix', () => missingAdapter.refresh(), 'object-missing');
      check(same(beforeMissing.state, missingAdapter.snapshot().state), 'missing final object promoted prefix');

      // Keep the real module, crypto and storage; fault only the generated
      // completion boundary after its actual IndexedDB commit has completed.
      for (const fault of ['trap', 'short', 'binding', 'phase']) {
        const isolated = await openJournal(module, store, 'workspace');
        const before = isolated.snapshot();
        const event = await signed(owner, before, 4, post(`completion ${fault}`));
        const OriginalInstance = WebAssembly.Instance;
        WebAssembly.Instance = class {
          constructor(...args) {
            const actual = new OriginalInstance(...args);
            const exports = { ...actual.exports };
            exports.holo_run = (pointer, length) => {
              const operation = new Uint8Array(exports.memory.buffer, pointer, length)[0];
              const result = actual.exports.holo_run(pointer, length);
              if (operation !== 2) return result;
              if (fault === 'trap') throw new WebAssembly.RuntimeError('planted completion trap');
              const packed = BigInt.asUintN(64, result), offset = Number(packed >> 32n);
              if (fault === 'short') return (packed & ~0xffffffffn) | 1n;
              const output = new Uint8Array(exports.memory.buffer, offset, Number(packed & 0xffffffffn));
              if (fault === 'binding') output[2] ^= 1;
              if (fault === 'phase') output[1] = 2;
              return result;
            };
            return { exports };
          }
        };
        try {
          await reject(`completion ${fault} cannot promote state`, () => isolated.append(event),
            fault === 'trap' ? 'generated-execution-failed' : 'invalid-generated-output');
        } finally { WebAssembly.Instance = OriginalInstance; }
        check(same(before.state, isolated.snapshot().state), `${fault} promoted state`);
        await reject(`completion ${fault} requires replay`, () => isolated.append(event), 'replay-required');
        await isolated.refresh();
        check(!same(before.state, isolated.snapshot().state), `${fault} committed event was not recovered`);
      }
      cases.push('actual commits with rejected completion recover without fabricated receipts');

      const uncertain = await openJournal(module, wrapper({ commit: async request => {
        await store.commit(request);
        throw Object.assign(Error('simulated lost acknowledgment'), { code: 'lost-acknowledgment' });
      } }), 'workspace');
      const old = uncertain.snapshot();
      const lost = await signed(owner, old, 4, post('committed without receipt'));
      const lostError = await reject('lost acknowledgment does not promote or retry', () => uncertain.append(lost), 'storage-outcome-unknown');
      check(lostError.name === 'JournalAdapterError' && lostError.message === 'storage-outcome-unknown'
        && lostError.detail === null && !Object.hasOwn(lostError, 'cause'), 'lost acknowledgment leaked its cause');
      check(same(uncertain.snapshot().state, old.state), 'uncertain commit promoted state');
      await reject('uncertain outcome blocks subsequent writes', () => uncertain.append(lost), 'replay-required');
      await uncertain.refresh();
      check(!same(uncertain.snapshot().state, old.state), 'committed history not recovered');
      cases.push('failure-after-commit recovers only through complete authenticated replay');
      store.close();
      const limited = await openStore('adapter-limited', { maxObjectBytes:1048576, maxObjects:1, maxHeads:1 });
      const bounded = await openJournal(module, limited, 'workspace');
      await reject('real quota bound aborts the whole transaction', () => bounded.append(original), 'storage-rejected');
      check(await limited.readHead('workspace') === null && await limited.readObject(await digestBytes(original)) === null,
        'failed transaction persisted partial data');
      check(bounded.snapshot().state.length === 0, 'failed transaction promoted state');
      limited.close();
      return { cases, generatedCalls: globalThis.__generatedJournalCalls };
    }, Array.from(wasmBytes));
  });
  assert.deepEqual(cases, [
    'captured input and defensive snapshot with real atomic genesis',
    'caller authentication and receipt are not accepted', 'reader cannot write', 'revoked contributor cannot write',
    'persisted identity and complete authenticated replay', 'signature mutation cannot append',
    'unsigned event ID substitution rejected', 'wrong signing context rejected', 'caller snapshot cannot seed state',
    'cross-adapter CAS conflict rejects candidate', 'conflict requires authenticated refresh',
    'real CAS conflict recovery preserves exactly committed history',
    'unknown readHead error is normalized', 'unknown readHead failure blocks writes',
    'unknown readHead error recovers only by complete replay',
    'unknown readObject error is normalized', 'unknown readObject failure blocks writes',
    'unknown readObject error recovers only by complete replay',
    'corrupt replay cannot replace prior state',
    'failed replay blocks subsequent writes', 'missing final replay object cannot expose a partial prefix',
    'completion trap cannot promote state', 'completion trap requires replay',
    'completion short cannot promote state', 'completion short requires replay',
    'completion binding cannot promote state', 'completion binding requires replay',
    'completion phase cannot promote state', 'completion phase requires replay',
    'actual commits with rejected completion recover without fabricated receipts',
    'lost acknowledgment does not promote or retry',
    'uncertain outcome blocks subsequent writes', 'failure-after-commit recovers only through complete authenticated replay',
    'real quota bound aborts the whole transaction',
  ]);
  await replayNative('faults', generatedCalls, wasmBytes, {native, work, mutant: transcriptMutant});
  return {cases, calls: generatedCalls.length};
}
