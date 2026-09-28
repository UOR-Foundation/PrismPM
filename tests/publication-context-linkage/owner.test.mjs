import test from 'node:test';

test('OC-10 binds actual source declaration and complete captured publication context',
  {timeout: 3500000}, async t => {
    const {verifyCompletePublicationLinkage} = await import('./checks.mjs');
    await verifyCompletePublicationLinkage(t);
  });
