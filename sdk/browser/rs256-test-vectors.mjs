// Pinned external oracle input. Never included in an application release.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {runInNewContext} from 'node:vm';

export function oracleVectors() {
  const root = new URL('./oracles/wpt-rs256/', import.meta.url);
  const source = JSON.parse(readFileSync(new URL('source.json', root), 'utf8'));
  assert.equal(source.repository, 'https://github.com/web-platform-tests/wpt');
  assert.equal(source.revision, '986e75d7897742148c16253c08c50c8ba0e7b0f7');
  assert.deepEqual(source.files.map(row => row.file), ['rsa_pkcs_vectors.js', 'rsa_key_fixtures.js', 'LICENSE.md']);
  const files = new Map();
  for (const row of source.files) {
    const bytes = readFileSync(new URL(row.file, root));
    assert.equal(createHash('sha256').update(bytes).digest('hex'), row.sha256);
    files.set(row.file, bytes.toString('utf8'));
  }
  const vectors = runInNewContext(files.get('rsa_key_fixtures.js') + '\n'
    + files.get('rsa_pkcs_vectors.js') + '\ngetTestVectors()', {}, {timeout: 1000});
  assert.deepEqual(Array.from(vectors, vector => vector.hash), ['SHA-1', 'SHA-256', 'SHA-384', 'SHA-512']);
  return Array.from(vectors, vector => {
    assert.equal(vector.algorithm.name, 'RSASSA-PKCS1-v1_5');
    assert.equal(vector.publicKeyFormat, 'spki');
    return {hash: vector.hash, publicKey: new Uint8Array(vector.publicKeyBuffer),
      message: new Uint8Array(vector.plaintext), signature: new Uint8Array(vector.signature)};
  });
}
