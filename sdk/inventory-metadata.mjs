import assert from 'node:assert/strict';

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
