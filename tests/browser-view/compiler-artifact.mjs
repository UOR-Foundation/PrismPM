// Test infrastructure: executable custody is not compiler-source provenance.
// Only an owning fresh, pinned compilation may supply an actual compiler here.
import assert from 'node:assert/strict';
import {closeSync, constants, fstatSync, lstatSync, mkdirSync, openSync, readSync,
  realpathSync, writeFileSync} from 'node:fs';
import {dirname, join, relative, resolve, sep} from 'node:path';
import {run, sha} from './compile.mjs';

const maximum = 268435456, owners = new WeakSet();
function directory(path, privateOwner = false) {
  assert.equal(resolve(path), path, 'normalized absolute compiler directory');
  assert.equal(realpathSync(path), path, 'unaliased compiler directory');
  const stat = lstatSync(path);
  assert.ok(stat.isDirectory() && stat.uid === process.getuid(), 'owned compiler directory');
  assert.equal(stat.mode & (privateOwner ? 0o077 : 0o022), 0,
    privateOwner ? 'private compiler owner directory required' : 'compiler directory cannot be group/other writable');
}

function read(path, links = null) {
  directory(dirname(path));
  assert.equal(realpathSync(path), path, 'unaliased compiler executable');
  const fd = openSync(path, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
  try {
    const before = fstatSync(fd, {bigint: true});
    assert.ok(before.isFile() && before.uid === BigInt(process.getuid())
      && before.size >= 4n && before.size <= BigInt(maximum), 'bounded owned regular compiler required');
    assert.ok((before.mode & 0o100n) !== 0n && (before.mode & 0o022n) === 0n,
      'owned executable without group/other write access required');
    if (links !== null) assert.equal(before.nlink, links, 'compiler link count');
    const bytes = Buffer.alloc(Number(before.size));
    for (let offset = 0; offset < bytes.length;) {
      const count = readSync(fd, bytes, offset, bytes.length - offset, null);
      assert.ok(count > 0, 'compiler shortened during capture'); offset += count;
    }
    assert.equal(readSync(fd, Buffer.alloc(1), 0, 1, null), 0, 'compiler grew during capture');
    for (const after of [fstatSync(fd, {bigint: true}), lstatSync(path, {bigint: true})]) {
      assert.ok(after.isFile(), 'compiler replaced during capture');
      for (const key of ['dev', 'ino', 'uid', 'nlink', 'size', 'mode', 'mtimeNs', 'ctimeNs'])
        assert.equal(after[key], before[key], 'stable compiler capture ' + key);
    }
    assert.deepEqual(bytes.subarray(0, 4), Buffer.from([0x7f, 0x45, 0x4c, 0x46]),
      'native Linux compiler artifact required');
    return {bytes, identity: Object.freeze({device: before.dev.toString(), inode: before.ino.toString(),
      links: before.nlink.toString(), size: bytes.length, sha256: sha(bytes)})};
  } finally {closeSync(fd);}
}

export function captureCompilerArtifact(work, original, name) {
  directory(work, true);
  assert.equal(resolve(original), original, 'absolute original compiler path required');
  const child = relative(work, original);
  assert.ok(child && child !== '..' && !child.startsWith('..' + sep) && !child.startsWith(sep),
    'original compiler belongs to the private owner');
  function ancestors() {
    directory(work, true);
    let path = work;
    for (const part of child.split(sep).slice(0, -1)) {path = join(path, part); directory(path);}
  }
  ancestors();
  assert.match(name, /^[a-z][a-z0-9-]{0,95}$/, 'closed compiler artifact name');
  const captured = read(original);
  // The tool runner confines LEAN_PATH to the generated prod-export entrypoint.
  // Preserve that basename in the separately owned private execution directory;
  // do not widen the environment guard to arbitrary captured executables.
  let path = join(work, name + '-execution');
  if (name === 'exporter') {
    assert.ok(original.endsWith('/prod-export'), 'exact generated exporter basename required');
    mkdirSync(path, {mode: 0o700}); path = join(path, 'prod-export');
  }
  assert.notEqual(original, path, 'separate compiler execution artifact required');
  writeFileSync(path, captured.bytes, {flag: 'wx', mode: 0o500});
  const private_ = read(path, 1n);
  assert.equal(private_.identity.sha256, captured.identity.sha256);
  const evidence = Object.freeze({original: Object.freeze({path: child, ...captured.identity}),
    private: Object.freeze({path: relative(work, path), ...private_.identity})});
  function verify() {
    ancestors();
    assert.deepEqual(read(original, BigInt(captured.identity.links)).identity, captured.identity,
      'immutable original compiler');
    assert.deepEqual(read(path, 1n).identity, private_.identity, 'immutable private compiler');
  }
  const owner = Object.freeze({path, evidence, verify,
    run(arguments_, cwd, environment = {}) {
      verify();
      try {return run(path, arguments_, cwd, environment);} finally {verify();}
    }});
  owners.add(owner); verify(); return owner;
}

export function requireCompilerArtifact(owner) {
  assert.ok(owners.has(owner), 'actual captured compiler artifact required');
  owner.verify(); return owner;
}
