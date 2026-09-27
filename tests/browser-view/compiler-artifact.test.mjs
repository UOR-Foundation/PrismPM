import assert from 'node:assert/strict';
import test from 'node:test';
import {chmodSync, copyFileSync, existsSync, linkSync, mkdirSync, mkdtempSync, readFileSync,
  renameSync, rmSync, symlinkSync, truncateSync, unlinkSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {captureCompilerArtifact, requireCompilerArtifact} from './compiler-artifact.mjs';

// This tests executable custody, not compiler provenance. Full component owners
// must construct their compiler from exact pinned source before capturing it.
function fixture(t, executable = '/bin/true') {
  const work = mkdtempSync(join(tmpdir(), 'prismpm-compiler-artifact-test-'));
  t.after(() => rmSync(work, {recursive: true, force: true}));
  const output = join(work, 'target'); mkdirSync(output);
  const original = join(output, 'tool'); copyFileSync(executable, original);
  chmodSync(original, 0o700);
  linkSync(original, join(output, 'dependency-output'));
  return {work, original};
}

test('fresh compiler capture retains original and separate singly linked private executable', t => {
  const {work, original} = fixture(t);
  const owner = captureCompilerArtifact(work, original, 'driver');
  assert.equal(requireCompilerArtifact(owner), owner);
  assert.equal(owner.evidence.original.links, '2');
  assert.equal(owner.evidence.private.links, '1');
  assert.equal(owner.run([], work), '');
  assert.equal(owner.run([], work), '');
  assert.throws(() => requireCompilerArtifact({...owner}), /actual captured compiler/);
  assert.throws(() => {owner.evidence.private.sha256 = '0'.repeat(64);}, TypeError);
  assert.throws(() => {owner.run = () => '';}, TypeError);
  assert.throws(() => captureCompilerArtifact(work, original, 'driver'), /EEXIST/);
});

test('private exporter keeps its guarded entrypoint without allowing driver environment overrides', t => {
  const {work, original} = fixture(t, '/bin/sh');
  const exporter = join(work, 'target', 'prod-export'); renameSync(original, exporter);
  const driver = captureCompilerArtifact(work, exporter, 'driver');
  assert.throws(() => driver.run(['-c', 'printf %s "$LEAN_PATH"'], work, {LEAN_PATH: work}),
    /LEAN_PATH belongs only to the exact generated exporter/);
  const owner = captureCompilerArtifact(work, exporter, 'exporter');
  assert.equal(owner.evidence.private.path, 'exporter-execution/prod-export');
  assert.equal(owner.run(['-c', 'printf %s "$LEAN_PATH"'], work, {LEAN_PATH: work}), work);
  owner.verify();
});

test('changed original/private executable refuses before execution and cannot be adopted', t => {
  const {work, original} = fixture(t, '/bin/sh');
  const owner = captureCompilerArtifact(work, original, 'driver'), sentinel = join(work, 'executed');
  for (const [kind, path] of [['original', original], ['private', owner.path]]) {
    const bytes = readFileSync(path), changed = Buffer.concat([bytes, Buffer.from([0])]);
    chmodSync(path, 0o700); writeFileSync(path, changed);
    assert.throws(() => owner.run(['-c', ': > "$1"', 'artifact-custody-probe', sentinel], work),
      new RegExp('immutable ' + kind + ' compiler'));
    assert.equal(existsSync(sentinel), false, 'substituted executable refused before planted sentinel');
    writeFileSync(path, bytes); owner.verify();
  }
});

test('same bytes in a new inode or an added hard link are not the captured compiler', t => {
  const {work, original} = fixture(t);
  const owner = captureCompilerArtifact(work, original, 'driver');
  for (const path of [original, owner.path]) {
    const alias = join(work, 'alias'); linkSync(path, alias);
    assert.throws(() => owner.verify(), /link count/);
    unlinkSync(alias); owner.verify();
  }
  // Keep the old inode alive so a filesystem cannot reuse it for this probe.
  const old = join(work, 'old-private'); renameSync(owner.path, old);
  copyFileSync(old, owner.path); chmodSync(owner.path, 0o700);
  assert.throws(() => owner.run([], work), /immutable private compiler/);
});

test('successful and failed actual executions both check compiler identity afterward', t => {
  const {work, original} = fixture(t, '/bin/sh');
  const owner = captureCompilerArtifact(work, original, 'driver'), bytes = readFileSync(original);
  for (const status of ['0', '1']) {
    assert.throws(() => owner.run(['-c', 'printf x >> "$1"; exit "$2"',
      'artifact-custody-probe', original, status], work), /immutable original compiler/);
    writeFileSync(original, bytes); owner.verify();
  }
});

test('compiler custody refuses writable parents and loss of private-owner isolation', t => {
  const {work, original} = fixture(t), output = join(work, 'target');
  chmodSync(output, 0o757);
  assert.throws(() => captureCompilerArtifact(work, original, 'driver'), /group\/other writable/);
  chmodSync(output, 0o755);
  const owner = captureCompilerArtifact(work, original, 'driver');
  for (const [path, changed, restored] of [[output, 0o757, 0o755], [work, 0o755, 0o700]]) {
    chmodSync(path, changed);
    assert.throws(() => owner.run([], work), /group\/other writable|private compiler owner/);
    chmodSync(path, restored); owner.verify();
  }
});

test('capture refuses aliases, unowned paths, nonexecutables and out-of-bound files', t => {
  const {work, original} = fixture(t), other = fixture(t);
  assert.throws(() => captureCompilerArtifact(work, other.original, 'driver'), /belongs to/);
  for (const name of ['', '../escape', 'UPPER', 'a'.repeat(97)])
    assert.throws(() => captureCompilerArtifact(work, original, name), /artifact name/);
  const alias = join(work, 'alias'); symlinkSync(original, alias);
  assert.throws(() => captureCompilerArtifact(work, alias, 'driver'), /unaliased/);
  chmodSync(original, 0o600);
  assert.throws(() => captureCompilerArtifact(work, original, 'driver'), /executable/);
  chmodSync(original, 0o700);
  const bytes = readFileSync(original), wrong = Buffer.from(bytes); wrong[0] = 0;
  writeFileSync(original, wrong);
  assert.throws(() => captureCompilerArtifact(work, original, 'driver'), /native Linux/);
  writeFileSync(original, bytes);
  for (const size of [0, 3, 268435457]) {
    truncateSync(original, size);
    assert.throws(() => captureCompilerArtifact(work, original, 'driver'), /bounded owned/);
  }
});
