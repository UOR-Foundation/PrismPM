import assert from 'node:assert/strict';

export function encodeInventory(value) {
  const canonical = item => Array.isArray(item) ? item.map(canonical)
    : item && typeof item === 'object'
    ? Object.fromEntries(Object.keys(item).sort((a, b) => Buffer.from(a).compare(Buffer.from(b)))
      .map(key => [key, canonical(item[key])])) : item;
  return `${JSON.stringify(canonical(value))}\n`;
}

// This bootstrap reader intentionally accepts only the register's closed
// string-valued TOML shape, not arbitrary TOML. A representation change fails
// closed and must update this reader alongside the authoritative Rust model.
export function compilerRevision(source) {
  const root = {};
  const dependencies = [];
  let table = root;
  for (const line of source.split('\n').map(line => line.trim())) {
    if (!line || line.startsWith('#')) continue;
    if (line === '[[dependency]]') {
      table = {};
      dependencies.push(table);
    } else if (line === '[[dependency.artifact]]') {
      assert.ok(dependencies.length, 'artifact has no dependency');
      table = {};
    } else {
      const match = /^([a-z][a-z0-9_]*) = ("(?:[^"\\]|\\["\\bfnrt]|\\u[0-9a-fA-F]{4})*")$/.exec(line);
      assert.ok(match, 'unsupported dependency-register syntax');
      assert.ok(!Object.hasOwn(table, match[1]), 'duplicate dependency-register field');
      table[match[1]] = JSON.parse(match[2]);
    }
  }
  assert.deepEqual(root, {spec: 'prismpm/dependencies/1'});
  assert.equal(new Set(dependencies.map(row => row.id)).size, dependencies.length);
  const matches = dependencies.filter(row => row.id === 'lean4-prod');
  assert.equal(matches.length, 1, 'exact lean4-prod authority is required');
  const dependency = matches[0];
  assert.deepEqual(Object.keys(dependency).sort(), ['id', 'lean_version', 'revision', 'source']);
  assert.equal(dependency.source, 'vendored');
  assert.match(dependency.lean_version, /^[0-9]+\.[0-9]+\.[0-9]+$/);
  assert.match(dependency.revision, /^[0-9a-f]{40}$/);
  return dependency.revision;
}

export function validateAuthorityMetadata(artifacts, revision) {
  const compiler = artifacts.filter(row => row.id === 'lean4-prod');
  const corpus = artifacts.filter(row => row.id === 'conformance-corpus');
  assert.equal(compiler.length, 1);
  assert.equal(compiler[0].version, revision, 'SDK compiler inventory names a stale authority revision');
  assert.equal(corpus.length, 1);
  assert.equal(corpus[0].version, 'prismpm/ids/1', 'SDK corpus metadata must identify its stable schema, not stale counts');
}

// These rows measure sealed input data, not a materialized checkout's mutable
// Git metadata and not proof that any acceptance command ran.
export function validateImageInputMetadata(artifacts, policy, manifest, manifestDigest, policyDigest) {
  assert.equal(manifest.schema, 'prismpm/sdk-vv-input-closure/1');
  assert.equal(manifest.scope, 'sdk-vv-inputs-only');
  assert.deepEqual(manifest.policy, policy);
  assert(Array.isArray(manifest.artifacts));
  assert.deepEqual(manifest.artifacts.map(row => row.path), ['advisory.pack', 'bootstrap.tar.gz', 'source.pack']);
  for (const row of manifest.artifacts) {
    assert.deepEqual(Object.keys(row).sort(), ['byte_length', 'path', 'sha256']);
    assert.match(row.sha256, /^[0-9a-f]{64}$/);
    assert(Number.isSafeInteger(row.byte_length) && row.byte_length > 0
      && row.byte_length <= (row.path === 'bootstrap.tar.gz' ? 64 : 256) * 1024 * 1024,
    'bounded sealed payload length required');
  }
  const payloads = new Map(manifest.artifacts.map(row => [row.path, row.sha256]));
  assert.equal(payloads.size, 3);
  assert.equal(payloads.get('bootstrap.tar.gz'), policy.bootstrap_sha256);
  const rows = [
    ['sdk-vv-source', policy.source_revision, `sha256:${payloads.get('source.pack')}`],
    ['sdk-vv-advisory', policy.advisory_revision, `sha256:${payloads.get('advisory.pack')}`],
    ['sdk-vv-bootstrap', '0.2.0', `sha256:${policy.bootstrap_sha256}`],
    ['sdk-vv-manifest', '1', manifestDigest],
    ['sdk-vv-policy', '1', policyDigest],
  ];
  for (const [id, version, digest] of rows) {
    const matches = artifacts.filter(row => row.id === id);
    assert.equal(matches.length, 1, `exact measured SDK input required: ${id}`);
    assert.deepEqual(matches[0], { id, kind: 'test-corpus', version, digest });
    assert.match(digest, /^sha256:[0-9a-f]{64}$/);
  }
  for (const [id, path] of [
    ['sdk-vv-verifier', 'scripts/sdk-vv-inputs.mjs'],
    ['sdk-image-input-verifier', 'scripts/sdk-image-inputs.mjs'],
    ['sdk-image-input-authorities', 'sdk/vv-inputs.lock.json'],
  ]) {
    const selected = manifest.source.files.filter(row => row.path === path);
    const rows = artifacts.filter(row => row.id === id);
    assert.equal(selected.length, 1); assert.equal(rows.length, 1);
    assert.deepEqual(rows[0], { id, kind: 'test-corpus', version: '1', digest: `sha256:${selected[0].sha256}` });
  }
}
