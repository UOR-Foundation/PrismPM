// Real browser concurrency acceptance; generated maximum cases run separately.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { withBrowser } from '../../sdk/browser/browser-test-server.mjs';
import { captureGeneratedCalls, replayNative } from './replay.mjs';

export async function verifyJournalConcurrency({wasmBytes: wasm, native, work, mutant}) {
let adapter = readFileSync(new URL('../../sdk/browser/journal.mjs', import.meta.url), 'utf8');
let storage = readFileSync(new URL('../../sdk/browser/store.mjs', import.meta.url), 'utf8');
const sha = value => createHash('sha256').update(value).digest('hex');
const originalAdapterSha256 = sha(adapter), originalStorageSha256 = sha(storage);
if (mutant === 'capture') {
  const capture = 'try { envelope = bytesCopy(value, 4363); }';
  const queued = "if (this.#requiresReplay) throw fail('replay-required');";
  assert.equal(adapter.split(capture).length, 2);
  assert.equal(adapter.split(queued).length, 2);
  adapter = adapter.replace(capture, 'try { envelope = value; }')
    .replace(queued, `envelope = bytesCopy(envelope, 4363);\n      ${queued}`);
} else if (mutant === 'admission') {
  const guard = 'const outstandingMaximum = 2;';
  assert.equal(adapter.split(guard).length, 2);
  adapter = adapter.replace(guard, 'const outstandingMaximum = 3;');
} else if (mutant === 'cas') {
  const guard = "if ((current ?? null) !== expected) throw fail('head-conflict');";
  assert.equal(storage.split(guard).length, 2);
  storage = storage.replace(guard, `if (false && (current ?? null) !== expected) throw fail('head-conflict');`);
} else assert.equal(mutant, undefined);
process.stdout.write(`${JSON.stringify({testOnly: true, mutant: mutant ?? null,
  testSha256: sha(readFileSync(new URL(import.meta.url))), wasmSha256: sha(wasm),
  originalAdapterSha256, servedAdapterSha256: sha(adapter), originalStorageSha256,
  servedStorageSha256: sha(storage), identitySha256: sha(readFileSync(new URL('../../sdk/browser/identity.mjs', import.meta.url)))})}\n`);

  const { cases, generatedCalls } = await withBrowser(async ({browser, baseURL}) => {
    const page = await browser.newPage();
    await page.addInitScript(captureGeneratedCalls);
    await page.route('**/journal.mjs', route => route.fulfill({contentType: 'text/javascript', body: adapter}));
    if (mutant === 'cas') {
      await page.route('**/store.mjs', route => route.fulfill({contentType: 'text/javascript', body: storage}));
    }
    await page.goto(baseURL);
    return page.evaluate(async wasmBytes => {
      const {openJournal} = await import('/journal.mjs');
      const {createIdentity, signBytes, digestBytes} = await import('/identity.mjs');
      const {openStore} = await import('/store.mjs');
      const module = await WebAssembly.compile(new Uint8Array(wasmBytes));
      const cases = [], stores = [];
      const check = (value, label) => { if (!value) throw Error(label); };
      const hex = value => Array.from(value, byte => byte.toString(16).padStart(2, '0')).join('');
      const unhex = value => Uint8Array.from(value.match(/../g), byte => parseInt(byte, 16));
      const bytes = value => unhex(value.slice(7));
      const same = (left, right) => hex(left) === hex(right);
      const equalState = (left, right) => same(left.head, right.head) && same(left.state, right.state);
      const text = value => new TextEncoder().encode(value);
      const join = (...parts) => {
        const result = new Uint8Array(parts.reduce((sum, part) => sum + part.length, 0));
        let offset = 0;
        for (const part of parts) { result.set(part, offset); offset += part.length; }
        return result;
      };
      const settled = promise => promise.then(value => ({ok: true, value}), error => ({ok: false, error}));
      const reject = async (promise, code, label) => {
        const result = await settled(promise);
        check(!result.ok && result.error.code === code, `${label}: expected ${code}, got ${result.error?.code}`);
      };
      const tick = () => new Promise(resolve => setTimeout(resolve, 0));
      function latch(label) {
        let release;
        const promise = new Promise(resolve => { release = resolve; });
        return {promise, release, async wait() {
          let timer;
          try {
            return await Promise.race([promise, new Promise((_, reject) => {
              timer = setTimeout(() => reject(Error(`timed out: ${label}`)), 15000);
            })]);
          } finally { clearTimeout(timer); }
        }};
      }
      function project(operation, envelope) {
        const request = join([6, operation], envelope);
        const instance = new WebAssembly.Instance(module, {});
        const pointer = instance.exports.holo_alloc(request.length);
        new Uint8Array(instance.exports.memory.buffer, pointer, request.length).set(request);
        const result = BigInt.asUintN(64, instance.exports.holo_run(pointer, request.length));
        const offset = Number(result >> 32n), length = Number(result & 0xffffffffn);
        check(length <= 1166008 && offset + length <= instance.exports.memory.buffer.byteLength, 'fixture ABI bounds');
        const output = new Uint8Array(instance.exports.memory.buffer, offset, length).slice();
        check(output[0] === 0, 'fixture envelope projection rejected');
        return output.slice(1);
      }
      async function signed(identity, previous, action, body = new Uint8Array()) {
        const event = new Uint8Array(134 + body.length);
        event[0] = 1; event[1] = action;
        event.fill(9, 2, 34); event.fill(1, 34, 66);
        event.set(bytes(identity.principal), 98);
        if (previous.head.length) {
          event.set(previous.head.slice(-64, -32), 66);
          new DataView(event.buffer).setUint16(130,
            new DataView(previous.head.buffer, previous.head.byteOffset, previous.head.byteLength).getUint16(36));
        }
        new DataView(event.buffer).setUint16(132, body.length); event.set(body, 134);
        const envelope = join([80, 87, 69, 1], identity.publicKey, new Uint8Array(64), event);
        envelope.set(bytes(await digestBytes(project(2, envelope))), 167);
        envelope.set(await signBytes(identity, 'prismpm/workspace-event/1', project(1, envelope)), 69);
        return envelope;
      }
      const owner = await createIdentity(), contributor = await createIdentity();
      async function realStore(name) { const store = await openStore(name); stores.push(store); return store; }
      const wrapped = (store, overrides) => ({
        readHead: (...args) => store.readHead(...args), readObject: (...args) => store.readObject(...args),
        commit: (...args) => store.commit(...args), ...overrides,
      });
      async function initialize(name, membership = false) {
        const store = await realStore(name), journal = await openJournal(module, store, 'workspace');
        await journal.append(await signed(owner, journal.snapshot(), 0));
        if (membership) await journal.append(await signed(owner, journal.snapshot(), 1, bytes(contributor.principal)));
        return {store, journal};
      }

      try {
        const detached = await initialize('adapter-concurrency-detached');
        const input = await signed(owner, detached.journal.snapshot(), 4, text('captured before transfer'));
        const retainedInput = input.slice(), pending = detached.journal.append(input);
        structuredClone(input.buffer, {transfer: [input.buffer]});
        check(input.byteLength === 0, 'fixture did not detach the caller buffer');
        const captured = await settled(pending);
        check(captured.ok, 'detached caller buffer was not synchronously captured');
        check(same(await detached.store.readObject(await digestBytes(retainedInput)), retainedInput),
          'detached caller buffer changed durable captured event');
        cases.push('append captures bytes before immediate ArrayBuffer detachment');

        const ordered = await initialize('adapter-concurrency-order');
        let mode = null, release = latch('order release'), entered = latch('order entry');
        const history = [];
        const orderedStore = wrapped(ordered.store, {
          readHead: async name => {
            history.push('read-head');
            if (mode === 'refresh-first') { entered.release(); await release.wait(); }
            return ordered.store.readHead(name);
          },
          commit: async request => {
            history.push('commit-enter');
            if (mode === 'append-first') { entered.release(); await release.wait(); }
            const result = await ordered.store.commit(request); history.push('commit-done'); return result;
          },
        });
        const local = await openJournal(module, orderedStore, 'workspace');
        history.length = 0; mode = 'append-first';
        const beforeAppend = local.snapshot();
        const appendEvent = await signed(owner, beforeAppend, 4, text('append before refresh'));
        const appendPromise = local.append(appendEvent), refreshPromise = local.refresh();
        await entered.wait(); await tick();
        check(history.join(',') === 'commit-enter', 'refresh ran before queued append completed');
        check(equalState(local.snapshot(), beforeAppend), 'blocked append promoted state');
        release.release();
        const appended = await appendPromise, refreshed = await refreshPromise;
        check(equalState(appended, refreshed) && equalState(local.snapshot(), appended), 'queued refresh lost committed append');
        check(history.join(',') === 'commit-enter,commit-done,read-head', 'append/refresh order differs');
        cases.push('append then refresh is serialized across a real pending transaction');

        history.length = 0; mode = 'refresh-first'; release = latch('refresh release'); entered = latch('refresh entry');
        const beforeRefresh = local.snapshot();
        const laterEvent = await signed(owner, beforeRefresh, 4, text('refresh before append'));
        const firstRefresh = local.refresh();
        await entered.wait();
        const laterAppend = local.append(laterEvent);
        await tick();
        check(history.join(',') === 'read-head', 'append overtook in-flight refresh');
        release.release();
        check(equalState(await firstRefresh, beforeRefresh), 'unchanged replay differs');
        await laterAppend;
        check(history.join(',') === 'read-head,commit-enter,commit-done', 'refresh/append order differs');
        cases.push('refresh then append waits for complete authenticated replay');

        const admission = await initialize('adapter-concurrency-admission');
        let admissionBlocked = false;
        const admissionEntered = latch('admission entry'), admissionRelease = latch('admission release');
        const bounded = await openJournal(module, wrapped(admission.store, {
          readHead: async name => {
            if (admissionBlocked) { admissionEntered.release(); await admissionRelease.wait(); }
            return admission.store.readHead(name);
          },
        }), 'workspace');
        const queuedEvent = await signed(owner, bounded.snapshot(), 4, text('bounded captured event'));
        const expectedEvent = queuedEvent.slice();
        admissionBlocked = true;
        const activeRefresh = bounded.refresh();
        await admissionEntered.wait();
        const queuedAppend = bounded.append(queuedEvent);
        queuedEvent.fill(0xff);
        let accessed = false;
        const hostile = new Proxy(new Uint8Array(), { get(target, key, receiver) {
          accessed = true; return Reflect.get(target, key, receiver);
        }});
        const overflow = settled(bounded.append(hostile));
        try {
          const outcome = await Promise.race([overflow, tick().then(() => ({timeout: true}))]);
          check(!outcome.timeout && !outcome.ok && outcome.error.code === 'journal-busy',
            'saturated append must reject journal-busy before capture');
          check(!accessed, 'busy append accessed caller bytes');
          await reject(bounded.refresh(), 'journal-busy', 'shared refresh admission budget');
        } finally {
          admissionBlocked = false; admissionRelease.release();
          await Promise.allSettled([activeRefresh, queuedAppend, overflow]);
        }
        await activeRefresh; await queuedAppend;
        check(same(await admission.store.readObject(await digestBytes(expectedEvent)), expectedEvent),
          'queued append did not preserve captured bytes');
        await bounded.refresh();
        await reject(bounded.append(null), 'invalid-input', 'capture failure releases admission');
        await reject(bounded.append(new Uint8Array()), 'model-rejected', 'model failure releases admission');
        await bounded.refresh();
        cases.push('two-operation shared admission rejects before capture and releases success/capture/model failures');

        const closedAdmission = await initialize('adapter-concurrency-admission-close');
        let closing = false;
        const closeEntered = latch('closed admission entry'), closeRelease = latch('closed admission release');
        const closingJournal = await openJournal(module, wrapped(closedAdmission.store, {
          readHead: async name => {
            if (closing) { closeEntered.release(); await closeRelease.wait(); }
            return closedAdmission.store.readHead(name);
          },
        }), 'workspace');
        const beforeClose = closingJournal.snapshot(); closing = true;
        const closingFirst = settled(closingJournal.refresh()); await closeEntered.wait();
        const closingSecond = settled(closingJournal.refresh());
        await reject(closingJournal.refresh(), 'journal-busy', 'closed storage saturated admission');
        closedAdmission.store.close(); closeRelease.release();
        for (const result of await Promise.all([closingFirst, closingSecond])) {
          check(!result.ok && result.error.code === 'store-closed', 'closed store did not settle queued replay');
        }
        check(equalState(beforeClose, closingJournal.snapshot()), 'closed replay promoted partial state');
        await reject(closingJournal.refresh(), 'store-closed', 'closed store releases all admission slots');
        cases.push('actual closed storage settles owned replay and releases admission without promotion');

        for (const winner of ['revoke', 'post']) {
          const name = `adapter-concurrency-${winner}`;
          const fixture = await initialize(name, true);
          const leftStore = await realStore(name), rightStore = await realStore(name);
          const arrivals = {revoke: latch('revoke candidate'), post: latch('post candidate')};
          const permits = {revoke: latch('revoke commit'), post: latch('post commit')};
          const requests = {}, calls = {revoke: 0, post: 0};
          const raceStore = (store, role) => wrapped(store, {commit: async request => {
            calls[role]++;
            requests[role] = {expected: request.expected, next: request.next,
              objects: request.objects.map(value => value.slice())};
            arrivals[role].release(); await permits[role].wait();
            return store.commit(request);
          }});
          const revoke = await openJournal(module, raceStore(leftStore, 'revoke'), 'workspace');
          const post = await openJournal(module, raceStore(rightStore, 'post'), 'workspace');
          const baseline = revoke.snapshot();
          check(equalState(baseline, post.snapshot()), 'independent replay disagreed');
          const revokeEvent = await signed(owner, baseline, 3, bytes(contributor.principal));
          const postEvent = await signed(contributor, baseline, 4, text('contributor raced revocation'));
          const results = {revoke: settled(revoke.append(revokeEvent)), post: settled(post.append(postEvent))};
          await Promise.all(Object.values(arrivals).map(value => value.wait()));
          check(calls.revoke === 1 && calls.post === 1, 'both legal candidates did not reach real CAS');
          check(requests.revoke.expected === requests.post.expected, 'race was not against the same prior head');
          check(equalState(revoke.snapshot(), baseline) && equalState(post.snapshot(), baseline), 'pre-CAS state promoted');
          permits[winner].release();
          const winnerResult = await results[winner];
          check(winnerResult.ok, 'selected first real CAS writer failed');
          const loser = winner === 'revoke' ? 'post' : 'revoke';
          permits[loser].release();
          const loserResult = await results[loser];
          check(!loserResult.ok && loserResult.error.code === 'storage-rejected', 'competing CAS admitted both histories');
          const loserJournal = loser === 'revoke' ? revoke : post;
          check(equalState(loserJournal.snapshot(), baseline), 'losing CAS promoted candidate state');
          for (const object of requests[loser].objects) {
            check(await fixture.store.readObject(await digestBytes(object)) === null, 'losing CAS persisted an orphan object/head');
          }
          for (const object of requests[winner].objects) {
            check(same(await fixture.store.readObject(await digestBytes(object)), object), 'winning atomic commit lost an object');
          }
          check((await fixture.store.readHead('workspace')).id === requests[winner].next, 'durable head is not the sole winner');
          await reject(loserJournal.append(loser === 'revoke' ? revokeEvent : postEvent), 'replay-required', 'CAS loser write barrier');
          await loserJournal.refresh();
          check(equalState(loserJournal.snapshot(), winnerResult.value), 'loser refresh did not adopt actual winner');
          if (winner === 'revoke') {
            await reject(loserJournal.append(await signed(contributor, loserJournal.snapshot(), 4, text('revoked retry'))),
              'model-rejected', 'revoked contributor after refresh');
          } else {
            // The post won before revocation. Fresh owner revocation is still
            // legal, and then a fresh replay must deny the contributor.
            await loserJournal.append(await signed(owner, loserJournal.snapshot(), 3, bytes(contributor.principal)));
            await post.refresh();
            await reject(post.append(await signed(contributor, post.snapshot(), 4, text('revoked after post winner'))),
              'model-rejected', 'contributor denied after eventual revocation');
          }
          cases.push(`${winner} wins competing authorized plans; loser has no orphan or promotion and replays permissions`);
        }

        const conflict = await initialize('adapter-concurrency-observation', true);
        const otherStore = await realStore('adapter-concurrency-observation');
        let failObservation = false;
        const observingStore = wrapped(otherStore, {
          readHead: async name => {
            if (failObservation) throw Object.assign(Error('planted observation failure'), {code: 'storage-unavailable'});
            return otherStore.readHead(name);
          },
          commit: async request => {
            try { return await otherStore.commit(request); }
            catch (error) { if (error.code === 'head-conflict') failObservation = true; throw error; }
          },
        });
        const stale = await openJournal(module, observingStore, 'workspace');
        const prior = stale.snapshot();
        const discarded = await signed(contributor, prior, 4, text('losing observation branch'));
        await conflict.journal.append(await signed(owner, conflict.journal.snapshot(), 3, bytes(contributor.principal)));
        await reject(stale.append(discarded), 'storage-unavailable', 'conflict observation read failure');
        check(equalState(stale.snapshot(), prior), 'failed conflict observation promoted state');
        check(await conflict.store.readObject(await digestBytes(discarded)) === null, 'observation failure wrote an orphan');
        await reject(stale.append(discarded), 'replay-required', 'observation failure blocks writes');
        await reject(stale.refresh(), 'storage-unavailable', 'failed refresh keeps write barrier');
        await reject(stale.append(discarded), 'replay-required', 'failed refresh still blocks writes');
        failObservation = false;
        await stale.refresh();
        check(equalState(stale.snapshot(), conflict.journal.snapshot()), 'restored read did not replay real history');
        await reject(stale.append(await signed(contributor, stale.snapshot(), 4, text('post after failed observation'))),
          'model-rejected', 'restored replay enforces revocation');
        cases.push('failed conflict observation preserves prior state and blocks writes until successful replay');
        return { cases, generatedCalls: globalThis.__generatedJournalCalls };
      } finally { for (const store of stores) store.close(); }
    }, Array.from(wasm));
  });
  assert.deepEqual(cases, [
    'append captures bytes before immediate ArrayBuffer detachment',
    'append then refresh is serialized across a real pending transaction',
    'refresh then append waits for complete authenticated replay',
    'two-operation shared admission rejects before capture and releases success/capture/model failures',
    'actual closed storage settles owned replay and releases admission without promotion',
    'revoke wins competing authorized plans; loser has no orphan or promotion and replays permissions',
    'post wins competing authorized plans; loser has no orphan or promotion and replays permissions',
    'failed conflict observation preserves prior state and blocks writes until successful replay',
  ]);
  process.stdout.write(`${JSON.stringify({wasmSha256: sha(wasm), cases})}\n`);
  await replayNative('concurrency', generatedCalls, wasm, {native, work});
  return {cases, calls: generatedCalls.length};
}
