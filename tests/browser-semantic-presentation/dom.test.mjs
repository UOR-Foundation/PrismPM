// Component-only; mandatory owning tests also supply generated artifacts.
import assert from 'node:assert/strict';
import test from 'node:test';
import {journey, verifyMutants} from './browser.mjs';

test('actual semantic DOM component and imported accessibility oracle', async t => {
  const result = await journey(t);
  assert.equal(result.modelChecked, false);
  assert.equal(result.cases.length, 7);
  assert.deepEqual(result.calls, []);
});

test('actual semantic DOM mutants fail complete component journeys', async t => {
  await verifyMutants(t);
});
