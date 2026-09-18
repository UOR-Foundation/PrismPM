import assert from 'node:assert/strict';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import { withBrowser } from './browser-test-server.mjs';

async function withPage(callback) {
  return withBrowser(async ({ browser, baseURL }) => {
    const page = await browser.newPage();
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    await page.goto(baseURL);
    const result = await callback(page);
    await page.evaluate(() => new Promise(resolve => setTimeout(resolve, 0)));
    assert.deepEqual(errors, []);
    return result;
  });
}

test('real browser cryptography rejects detached inputs and malformed private-key handles', { timeout: 30000 }, async () => {
  const actual = await withPage(page => page.evaluate(async () => {
    const { bytesCopy, createIdentity, digestBytes, identityPrincipal, signBytes, verifyBytes } = await import('./identity.mjs');
    const identity = await createIdentity();
    const detached = Uint8Array.of(1, 2, 3);
    structuredClone(detached.buffer, { transfer: [detached.buffer] });
    const codes = [];
    const reject = async operation => {
      try { await operation(); codes.push('unexpected-success'); }
      catch (error) { codes.push(error.code ?? error.name); }
    };
    await reject(() => bytesCopy(detached));
    await reject(() => digestBytes(detached));
    await reject(() => identityPrincipal(detached));
    await reject(() => signBytes(identity, 'test/1', detached));
    await reject(() => verifyBytes(identity.publicKey, 'test/1', detached, new Uint8Array(64)));
    await reject(() => verifyBytes(identity.publicKey, 'test/1', new Uint8Array(), detached));
    const extractable = await crypto.subtle.generateKey({ name: 'ECDSA', namedCurve: 'P-256' }, true, ['sign', 'verify']);
    const wrongCurve = await crypto.subtle.generateKey({ name: 'ECDSA', namedCurve: 'P-384' }, false, ['sign', 'verify']);
    const secret = await crypto.subtle.generateKey({ name: 'AES-GCM', length: 256 }, false, ['encrypt']);
    const fake = { type: 'private', extractable: false, algorithm: { name: 'ECDSA', namedCurve: 'P-256' }, usages: ['sign'] };
    for (const privateKey of [null, {}, fake, Object.create(CryptoKey.prototype),
      extractable.publicKey, extractable.privateKey, wrongCurve.privateKey, secret]) {
      await reject(() => signBytes({ ...identity, privateKey }, 'test/1', Uint8Array.of(1)));
    }
    return codes;
  }));
  assert.deepEqual(actual, [...Array(6).fill('invalid-input'), ...Array(8).fill('identity-corrupt')]);
});

test('durable identity saving captures the original key and public bytes before asynchronous validation', { timeout: 30000 }, async () => {
  const actual = await withPage(page => page.evaluate(async () => {
    const { openStore } = await import('./store.mjs');
    const { createIdentity, signBytes, verifyBytes } = await import('./identity.mjs');
    const original = await createIdentity(), replacement = await createIdentity();
    const input = { ...original, publicKey: new Uint8Array(original.publicKey) };
    const store = await openStore('boundary-identity-capture');
    const saving = store.saveIdentity(input);
    input.privateKey = replacement.privateKey;
    input.principal = replacement.principal;
    input.publicKey.fill(0);
    await saving;
    store.close();
    const reopened = await openStore('boundary-identity-capture');
    const retained = await reopened.loadIdentity();
    const payload = Uint8Array.of(3, 1, 4);
    const signature = await signBytes(retained, 'test/1', payload);
    const result = {
      samePrincipal: retained.principal === original.principal,
      extractable: retained.privateKey.extractable,
      valid: await verifyBytes(original.publicKey, 'test/1', payload, signature),
      replacementValid: await verifyBytes(replacement.publicKey, 'test/1', payload, signature),
    };
    reopened.close();
    return result;
  }));
  assert.deepEqual(actual, { samePrincipal: true, extractable: false, valid: true, replacementValid: false });
});

test('atomic commits capture command fields and all caller-owned buffers before hashing', { timeout: 30000 }, async () => {
  const actual = await withPage(page => page.evaluate(async () => {
    const { openStore } = await import('./store.mjs');
    const { digestBytes } = await import('./identity.mjs');
    const store = await openStore('boundary-commit-capture');
    const first = Uint8Array.of(1, 2), second = Uint8Array.of(3, 4);
    const firstId = await digestBytes(first), secondId = await digestBytes(second);
    const command = { head: 'original', expected: null, next: firstId, objects: [first, second] };
    const committing = store.commit(command);
    first.fill(9);
    structuredClone(second.buffer, { transfer: [second.buffer] });
    command.head = 'changed'; command.expected = firstId; command.next = secondId; command.objects.length = 0;
    const result = await committing;
    const head = await store.readHead('original');
    const retainedSecond = await store.readObject(secondId);
    const changed = await store.readHead('changed');
    const detached = Uint8Array.of(0);
    structuredClone(detached.buffer, { transfer: [detached.buffer] });
    let code;
    try { await store.commit({ head: 'invalid', expected: null, next: firstId, objects: [detached] }); }
    catch (error) { code = error.code ?? error.name; }
    store.close();
    return { correctId: result === firstId && head.id === firstId, first: [...head.bytes], second: [...retainedSecond], changed, code };
  }));
  assert.deepEqual(actual, { correctId: true, first: [1, 2], second: [3, 4], changed: null, code: 'invalid-input' });
});

test('transaction abort rolls back objects already written before object or head limits fail', { timeout: 30000 }, async () => {
  const actual = await withPage(page => page.evaluate(async () => {
    const { openStore } = await import('./store.mjs');
    const { digestBytes } = await import('./identity.mjs');
    const a = Uint8Array.of(1), b = Uint8Array.of(2), c = Uint8Array.of(3);
    const aid = await digestBytes(a), bid = await digestBytes(b), cid = await digestBytes(c);
    const results = [];
    for (const [name, maxObjects] of [['object', 2], ['head', 4]]) {
      const store = await openStore(`boundary-rollback-${name}`, { maxObjectBytes: 4, maxObjects, maxHeads: 1 });
      await store.commit({ head: 'original', expected: null, next: aid, objects: [a] });
      let code;
      try {
        await store.commit({ head: name === 'head' ? 'second' : 'original',
          expected: name === 'head' ? null : aid, next: cid, objects: [b, c] });
      } catch (error) { code = error.code ?? error.name; }
      results.push({ code, original: (await store.readHead('original')).id === aid,
        second: await store.readHead('second'), b: await store.readObject(bid), c: await store.readObject(cid) });
      store.close();
    }
    return results;
  }));
  assert.deepEqual(actual, Array.from({ length: 2 }, () => ({ code: 'store-limit', original: true, second: null, b: null, c: null })));
});

test('stored reads reject hash-correct objects exceeding persisted namespace limits', { timeout: 30000 }, async () => {
  const actual = await withPage(page => page.evaluate(async () => {
    const { openStore } = await import('./store.mjs');
    const codes = [];
    for (const size of [5, 1048577]) {
      const namespace = `boundary-oversize-${size}`;
      const store = await openStore(namespace, { maxObjectBytes: 4, maxObjects: 2, maxHeads: 1 });
      const bytes = new Uint8Array(size);
      const digest = new Uint8Array(await crypto.subtle.digest('SHA-256', bytes));
      const id = `sha256:${Array.from(digest, byte => byte.toString(16).padStart(2, '0')).join('')}`;
      await new Promise((resolve, reject) => {
        const request = indexedDB.open(`prismpm.browser.v1/${namespace}`);
        request.onerror = () => reject(request.error);
        request.onsuccess = () => {
          const db = request.result, tx = db.transaction(['objects', 'heads'], 'readwrite');
          tx.objectStore('objects').put(bytes, id);
          tx.objectStore('heads').put(id, 'main');
          tx.oncomplete = () => { db.close(); resolve(); };
          tx.onabort = () => { db.close(); reject(tx.error); };
        };
      });
      for (const read of [() => store.readObject(id), () => store.readHead('main')]) {
        try { await read(); codes.push('unexpected-success'); }
        catch (error) { codes.push(error.code ?? error.name); }
      }
      store.close();
    }
    return codes;
  }));
  assert.deepEqual(actual, Array(4).fill('object-corrupt'));
});

test('missing or malformed persisted metadata cannot reset namespace policy', { timeout: 30000 }, async () => {
  const actual = await withPage(page => page.evaluate(async () => {
    const { openStore } = await import('./store.mjs');
    const limits = { maxObjectBytes: 4, maxObjects: 2, maxHeads: 1 };
    const malformed = [undefined, null, 'limits', [], {}, { ...limits, extra: true },
      { ...limits, maxObjects: 0 }, { ...limits, maxObjects: 4097 }, { ...limits, maxObjectBytes: NaN }];
    const codes = [];
    for (const [index, metadata] of malformed.entries()) {
      const namespace = `boundary-metadata-${index}`;
      (await openStore(namespace, limits)).close();
      await new Promise((resolve, reject) => {
        const request = indexedDB.open(`prismpm.browser.v1/${namespace}`);
        request.onerror = () => reject(request.error);
        request.onsuccess = () => {
          const db = request.result, tx = db.transaction('metadata', 'readwrite');
          if (metadata === undefined) tx.objectStore('metadata').delete('limits');
          else tx.objectStore('metadata').put(metadata, 'limits');
          tx.oncomplete = () => { db.close(); resolve(); };
          tx.onabort = () => { db.close(); reject(tx.error); };
        };
      });
      for (const requested of [limits, { ...limits, maxObjects: 3 }]) {
        try { (await openStore(namespace, requested)).close(); codes.push('unexpected-success'); }
        catch (error) { codes.push(error.code ?? error.name); }
      }
    }
    return codes;
  }));
  assert.deepEqual(actual, Array(18).fill('object-corrupt'));
});

test('same-name IndexedDB stores with incompatible key, increment, or index schemas are rejected', { timeout: 30000 }, async () => {
  const actual = await withPage(page => page.evaluate(async () => {
    const { openStore } = await import('./store.mjs');
    const limits = { maxObjectBytes: 4, maxObjects: 2, maxHeads: 1 };
    const names = ['objects', 'heads', 'identity', 'metadata'];
    const codes = [];
    for (const target of names) {
      for (const variant of ['key-path', 'auto-increment', 'index']) {
        const namespace = `boundary-schema-${target}-${variant}`;
        await new Promise((resolve, reject) => {
          const request = indexedDB.open(`prismpm.browser.v1/${namespace}`, 1);
          request.onerror = () => reject(request.error);
          request.onupgradeneeded = () => {
            for (const name of names) {
              const options = name !== target ? {} : variant === 'key-path' ? { keyPath: 'id' }
                : variant === 'auto-increment' ? { autoIncrement: true } : {};
              const store = request.result.createObjectStore(name, options);
              if (name === target && variant === 'index') store.createIndex('unexpected', 'id');
              if (name === 'metadata') {
                if (target === name && variant === 'key-path') store.put({ ...limits, id: 'limits' });
                else store.put(limits, 'limits');
              }
            }
          };
          request.onsuccess = () => { request.result.close(); resolve(); };
        });
        try { (await openStore(namespace, limits)).close(); codes.push('unexpected-success'); }
        catch (error) { codes.push(error.code ?? error.name); }
      }
    }
    return codes;
  }));
  assert.deepEqual(actual, Array(12).fill('object-corrupt'));
});

test('identity and committed objects survive a complete browser process restart', { timeout: 30000 }, async () => {
  const profile = await mkdtemp(join(tmpdir(), 'prismpm-browser-restart-'));
  try {
    await withBrowser(async ({ baseURL, launchPersistentContext }) => {
      let context;
      try {
        context = await launchPersistentContext(profile);
        const page = await context.newPage();
        await page.goto(baseURL);
        const expected = await page.evaluate(async () => {
          const { openStore } = await import('./store.mjs');
          const { createIdentity, digestBytes } = await import('./identity.mjs');
          const store = await openStore('boundary-process-restart');
          const identity = await createIdentity();
          await store.saveIdentity(identity);
          const bytes = new TextEncoder().encode('committed before process exit');
          const id = await digestBytes(bytes);
          await store.commit({ head: 'main', expected: null, next: id, objects: [bytes] });
          store.close();
          sessionStorage.setItem('process-local', 'old');
          return { principal: identity.principal, id };
        });
        await context.close();
        await assert.rejects(context.newPage());
        context = await launchPersistentContext(profile);
        const reopened = await context.newPage();
        await reopened.goto(baseURL);
        const actual = await reopened.evaluate(async () => {
          const { openStore } = await import('./store.mjs');
          const { signBytes, verifyBytes } = await import('./identity.mjs');
          const store = await openStore('boundary-process-restart');
          const identity = await store.loadIdentity(), head = await store.readHead('main');
          const signature = await signBytes(identity, 'test/1', head.bytes);
          let exportRejected = false;
          try { await crypto.subtle.exportKey('pkcs8', identity.privateKey); } catch { exportRejected = true; }
          const result = { principal: identity.principal, id: head.id,
            text: new TextDecoder().decode(head.bytes), extractable: identity.privateKey.extractable,
            valid: await verifyBytes(identity.publicKey, 'test/1', head.bytes, signature), exportRejected,
            priorSession: sessionStorage.getItem('process-local') };
          store.close();
          return result;
        });
        assert.deepEqual(actual, { ...expected, text: 'committed before process exit',
          extractable: false, valid: true, exportRejected: true, priorSession: null });
      } finally { await context?.close(); }
    });
  } finally { await rm(profile, { recursive: true, force: true }); }
});

test('unavailable cryptography and unsupported storage versions retain typed failures', { timeout: 30000 }, async () => {
  await withBrowser(async ({ browser, baseURL }) => {
    const page = await browser.newPage();
    await page.goto(baseURL);
    const result = await page.evaluate(async () => {
      const { createIdentity, identityPrincipal } = await import('./identity.mjs');
      const { openStore } = await import('./store.mjs');
      const identity = await createIdentity();
      const codes = [];
      Object.defineProperty(globalThis, 'crypto', { value: {}, configurable: true });
      try {
        for (const action of [() => createIdentity(), () => identityPrincipal(identity.publicKey)]) {
          try { await action(); codes.push('unexpected-success'); } catch (error) { codes.push(error.code); }
        }
      } finally { delete globalThis.crypto; }
      await new Promise((resolve, reject) => {
        const request = indexedDB.open('prismpm.browser.v1/future-version', 2);
        request.onerror = () => reject(request.error);
        request.onsuccess = () => { request.result.close(); resolve(); };
      });
      try { await openStore('future-version'); codes.push('unexpected-success'); } catch (error) { codes.push(error.code); }
      return codes;
    });
    assert.deepEqual(result, ['crypto-unavailable', 'crypto-unavailable', 'storage-unavailable']);
  });
});

test('a genuinely blocked database open expires without publishing a usable store', { timeout: 15000 }, async () => {
  await withBrowser(async ({ browser, baseURL }) => {
    const page = await browser.newPage();
    await page.goto(baseURL);
    const result = await page.evaluate(async () => {
      const { openStore } = await import('./store.mjs');
      const name = 'prismpm.browser.v1/blocked-open';
      const db = await new Promise((resolve, reject) => {
        const request = indexedDB.open(name, 1);
        request.onerror = () => reject(request.error);
        request.onsuccess = () => resolve(request.result);
      });
      const deletion = indexedDB.deleteDatabase(name);
      await new Promise((resolve, reject) => { deletion.onblocked = resolve; deletion.onerror = () => reject(deletion.error); });
      let code;
      try { const store = await openStore('blocked-open'); store.close(); code = 'unexpected-success'; }
      catch (error) { code = error.code; }
      finally {
        const done = new Promise((resolve, reject) => { deletion.onsuccess = resolve; deletion.onerror = () => reject(deletion.error); });
        db.close();
        await done;
      }
      return code;
    });
    assert.equal(result, 'storage-blocked');
  });
});

test('real browser quota exhaustion rejects the commit without acknowledging or retaining its writes', { timeout: 30000 }, async () => {
  const profile = await mkdtemp(join(tmpdir(), 'prismpm-browser-quota-'));
  try {
    await withBrowser(async ({ baseURL, launchPersistentContext }) => {
      const context = await launchPersistentContext(profile);
      const page = await context.newPage();
      const errors = [];
      page.on('pageerror', error => errors.push(error.message));
      await page.goto(baseURL);
      const cdp = await context.newCDPSession(page);
      const origin = new URL(baseURL).origin;
      try {
        // Set the real browser quota before the first database operation: IDB
        // may cache previously granted disk space while a database is open.
        await cdp.send('Storage.overrideQuotaForOrigin', { origin, quotaSize: 65536 });
        const quota = await cdp.send('Storage.getUsageAndQuota', { origin });
        assert.equal(quota.overrideActive, true);
        const actual = await page.evaluate(async () => {
          const { openStore } = await import('./store.mjs');
          const { digestBytes } = await import('./identity.mjs');
          const store = await openStore('boundary-quota');
          const original = Uint8Array.of(1), originalId = await digestBytes(original);
          await store.commit({ head: 'main', expected: null, next: originalId, objects: [original] });
          const bytes = new Uint8Array(1048576);
          for (let offset = 0; offset < bytes.length; offset += 65536) crypto.getRandomValues(bytes.subarray(offset, offset + 65536));
          const id = await digestBytes(bytes);
          let code = 'unexpected-success';
          try { await store.commit({ head: 'main', expected: originalId, next: id, objects: [bytes] }); }
          catch (error) { code = error.code ?? error.name; }
          const head = await store.readHead('main'), retained = await store.readObject(id);
          store.close();
          return { code, unchangedHead: head.id === originalId, retained: retained === null };
        });
        assert.deepEqual(actual, { code: 'storage-quota', unchangedHead: true, retained: true });
        await page.evaluate(() => new Promise(resolve => setTimeout(resolve, 0)));
        assert.deepEqual(errors, []);
      } finally {
        await cdp.send('Storage.overrideQuotaForOrigin', { origin });
        await cdp.detach();
        await context.close();
      }
    });
  } finally { await rm(profile, { recursive: true, force: true }); }
});
