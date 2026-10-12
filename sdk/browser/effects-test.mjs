import test from 'node:test';
import assert from 'node:assert/strict';
import {verifyWire, verifyModelMutation, verifyInventory, prerequisite} from '../../tests/browser-effects/checks.mjs';
import {verifyJourneys, verifyHostMutants} from '../../tests/browser-effects/browser.mjs';
import {createCompilerOwner, requireCompilerOwner} from '../../tests/browser-view/compiler-owner.mjs';
import {verifyCompilerOwnerSubstitutions} from '../../tests/browser-view/compiler-owner-checks.mjs';

test('DK-20 generated bounded byte effects and real private browser execution', {timeout: 3500000}, async t => {
  const owner = createCompilerOwner('effects');
  const substitutions = verifyCompilerOwnerSubstitutions(owner);
  assert.deepEqual(substitutions, ['cloned-handle', 'cross-family-handle', 'immutable-owner',
    'driver-source', 'driver-lock', 'exporter-library', 'driver-original', 'driver-private',
    'exporter-original', 'exporter-private', 'exporter-root-mode', 'exporter-build-mode',
    'exporter-library-mode', 'driver-original-mode', 'driver-private-mode', 'exporter-original-mode',
    'exporter-private-mode', 'forged-disk-receipt', 'driver-same-byte-inode', 'exporter-same-byte-inode']);
  const build = await verifyWire(t, owner);
  assert.equal(build.compilerOwner, owner.identity);
  const browser = await prerequisite(t, 'actual browser effects and exact std/no_std native transcripts', child => verifyJourneys(child, build));
  assert.equal(browser.cases.length, 30);
  const hostMutants = await prerequisite(t, 'planted private host boundary defects fail real browser execution', child => verifyHostMutants(child, build));
  assert.equal(hostMutants.length, 10);
  await prerequisite(t, 'closed host/model diagnostic inventory', verifyInventory);
  const modelMutants = [];
  for (const kind of ['binding', 'trailing', 'unknown']) {
    modelMutants.push(await prerequisite(t, 'actual LexLean ' + kind + ' mutation fails generated native and Wasm assertions', () => {
      const result = verifyModelMutation(kind, owner);
      assert.equal(result.compilerOwner, owner.identity);
      t.diagnostic(JSON.stringify(result));
      return result;
    }));
  }
  assert.deepEqual(modelMutants.map(row => row.mutation), ['binding', 'trailing', 'unknown']);
  const retirement = owner.close();
  assert.throws(() => requireCompilerOwner(owner, 'effects'), /compiler owner closed/);
  assert.throws(() => owner.close(), /compiler owner closed/);
  t.diagnostic(JSON.stringify({compiler: owner.evidence, substitutions, retirement, browser, hostMutants, modelMutants}));
});
