import test from 'node:test';
import {verifyRecoveryComponents} from './checks.mjs';

test('fresh linked source/kernel/native/Wasm recovery and metadata components', {timeout: 3500000}, t => {
  const {build, evidence} = verifyRecoveryComponents();
  t.diagnostic('retained private recovery component work ' + build.work);
  t.diagnostic(JSON.stringify({scope: evidence.scope, cases: evidence.cases}));
});
