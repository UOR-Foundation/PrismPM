import assert from 'node:assert/strict';
import test from 'node:test';
import {verifyWire, verifyMutation, replayBrowser, closure, completeEvidence} from '../../tests/browser-semantic-presentation/checks.mjs';
import {journey, verifyMutants, verifyMaximum} from '../../tests/browser-semantic-presentation/browser.mjs';
import {prerequisite} from '../../tests/browser-view/prerequisites.mjs';

test('DK-38 private generated semantic presentation and actual DOM oracle', {timeout: 3500000}, async t => {
  const before = closure(), build = verifyWire(t);
  await prerequisite(t, 'actual generated source, labels and design catalogue drive rendered DOM oracle journeys', async child => {
    const result = await journey(child, build); replayBrowser(build, result);
    assert.equal(result.cases.length, 7);
  });
  await prerequisite(t, 'exact 64 MiB combined semantic envelopes execute and render in actual Chromium', child => verifyMaximum(child, build));
  await prerequisite(t, 'actual semantic adapter defects fail generated browser journeys', child => verifyMutants(child, build));
  for (const kind of ['purpose', 'main', 'trailing', 'design', 'catalogue']) {
    await prerequisite(t, 'actual source ' + kind + ' mutant fails std/no_std/Wasm', () => verifyMutation(kind, build));
  }
  assert.deepEqual(closure(), before, 'all owning implementation and oracle inputs remained frozen');
  t.diagnostic('Retained complete private semantic component evidence ' + completeEvidence(build));
});
