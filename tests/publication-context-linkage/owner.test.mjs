import test from 'node:test';

test('OC-10 binds actual source declaration and complete captured publication context',
  {timeout: 3500000}, async t => {
    // Refuse before the expensive protocol component until the complete real
    // capture owner exists. Its absence cannot be replaced by conditional vectors.
    const {verifyCapturedPublicationLinkage} = await import('./capture-owner.mjs');
    const {verifyGeneratedLinkage} = await import('./checks.mjs');
    await verifyGeneratedLinkage(t);
    await verifyCapturedPublicationLinkage(t);
  });
