import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {mutateStorageHost, storageHostMutations} from './storage-verification.mjs';

test('all storage host defects alter exactly one real guard and name a unique browser counterexample', () => {
  const source = readFileSync(new URL('../../sdk/browser/session-storage.mjs', import.meta.url), 'utf8');
  const fixture = readFileSync(new URL('./storage-browser.mjs', import.meta.url), 'utf8');
  assert.equal(storageHostMutations.length, 6);
  assert.equal(new Set(storageHostMutations.map(row => row.id)).size, 6);
  for (const mutation of storageHostMutations) {
    const changed = mutateStorageHost(source, mutation);
    assert.notEqual(changed, source);
    assert.equal(fixture.split("t.test('" + mutation.case).length, 2);
    assert.throws(() => mutateStorageHost(changed, mutation));
  }
});
