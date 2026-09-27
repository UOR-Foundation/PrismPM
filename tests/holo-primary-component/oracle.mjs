// Pinned, source-built independent oracle; no adopted target/executable cache.
import assert from 'node:assert/strict';
import {chmodSync, existsSync, lstatSync, mkdirSync, mkdtempSync, readdirSync, readFileSync,
  realpathSync, statfsSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname, join, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {run, sha} from '../browser-view/compile.mjs';

const repository = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const sourcePaths = [
  'vendor/hologram-live.tar', 'model/dependencies.toml', 'scripts/fetch-oracle-cargo.sh',
  'tests/hologram-oracle/Cargo.toml', 'tests/hologram-oracle/Cargo.lock',
  'tests/holo-primary-component/oracle.rs', 'tests/holo-primary-component/oracle.mjs',
];
const read = path => {
  assert.equal(realpathSync(path), path, 'unaliased oracle input');
  const stat = lstatSync(path);
  assert.ok(stat.isFile() && stat.nlink === 1, 'singly linked regular oracle input');
  return readFileSync(path);
};
function tree(root) {
  const files = {};
  function visit(relative) {
    const directory = join(root, relative);
    assert.equal(realpathSync(directory), directory, 'unaliased oracle source directory');
    assert.ok(lstatSync(directory).isDirectory());
    for (const name of readdirSync(directory).sort()) {
      const child = relative ? relative + '/' + name : name;
      const path = join(root, child), stat = lstatSync(path);
      if (stat.isDirectory()) visit(child);
      else files[child] = sha(read(path));
    }
  }
  visit(''); return files;
}
function binaryIdentity(path, expectedLinks = null) {
  assert.equal(realpathSync(path), path, 'unaliased oracle executable');
  const before = lstatSync(path, {bigint: true});
  assert.ok(before.isFile() && before.uid === BigInt(process.getuid()) && before.size > 0n
    && before.size <= 512n * 1024n ** 2n, 'bounded owned oracle executable');
  if (expectedLinks !== null) assert.equal(before.nlink, expectedLinks);
  const bytes = readFileSync(path), after = lstatSync(path, {bigint: true});
  for (const key of ['dev', 'ino', 'nlink', 'size', 'mode', 'uid', 'mtimeNs', 'ctimeNs'])
    assert.equal(before[key], after[key], 'stable executable ' + key);
  return {bytes, identity: {device: before.dev.toString(), inode: before.ino.toString(),
    links: before.nlink.toString(), bytes: bytes.length, sha256: sha(bytes)}};
}
export function prepareOracle() {
  const captured = new Map(sourcePaths.map(path => [path, read(join(repository, path))]));
  const text = path => captured.get(path).toString('utf8');
  const dependency = text('model/dependencies.toml').split('[[dependency]]').slice(1)
    .filter(section => /^id = "hologram-live"$/m.test(section));
  assert.equal(dependency.length, 1);
  assert.match(dependency[0], /^role = "validation-oracle"$/m);
  const revision = /^revision = "([0-9a-f]{40})"$/m.exec(dependency[0])?.[1];
  assert.equal(revision, 'd8208266d8abdc2445b7bbc0cef412a566adfaf1');
  const artifacts = dependency[0].split('[[dependency.artifact]]').slice(1)
    .filter(section => /^path = "vendor\/hologram-live.tar"$/m.test(section));
  assert.equal(artifacts.length, 1);
  assert.equal(sha(captured.get('vendor/hologram-live.tar')), /^sha256 = "([0-9a-f]{64})"$/m.exec(artifacts[0])?.[1]);
  for (const name of ['Cargo.toml', 'Cargo.lock']) {
    const path = 'tests/hologram-oracle/' + name;
    const pins = text('scripts/fetch-oracle-cargo.sh').split('\n')
      .map(line => /^([0-9a-f]{64})  (\S+)$/.exec(line)).filter(row => row?.[2] === path);
    assert.equal(pins.length, 1, 'independently acquired exact graph pin');
    assert.equal(sha(captured.get(path)), pins[0][1]);
  }
  const disk = statfsSync(tmpdir(), {bigint: true});
  assert.ok(disk.bavail * disk.bsize >= 12n * 1024n ** 3n, '12 GiB private compiler reserve');
  const work = mkdtempSync(join(tmpdir(), 'prismpm-primary-live-'));
  const put = (relative, bytes) => {
    const path = join(work, relative); mkdirSync(dirname(path), {recursive: true});
    writeFileSync(path, bytes, {flag: 'wx'}); return path;
  };
  const archive = put('hologram-live.tar', captured.get('vendor/hologram-live.tar'));
  const upstream = join(work, 'hologram-live'); mkdirSync(upstream);
  run('tar', ['-xf', archive, '-C', upstream], work);
  const harness = join(work, 'hologram-oracle');
  for (const name of ['Cargo.toml', 'Cargo.lock']) put('hologram-oracle/' + name, captured.get('tests/hologram-oracle/' + name));
  put('hologram-oracle/src/main.rs', captured.get('tests/holo-primary-component/oracle.rs'));
  const upstreamFiles = tree(upstream), harnessFiles = tree(harness);
  function unchanged() {
    for (const [path, bytes] of captured) assert.deepEqual(read(join(repository, path)), bytes, 'frozen oracle source ' + path);
    assert.deepEqual(tree(upstream), upstreamFiles, 'complete pinned upstream source unchanged');
    assert.deepEqual(tree(harness), harnessFiles, 'complete executing oracle source unchanged');
    assert.deepEqual(read(archive), captured.get('vendor/hologram-live.tar'));
  }
  const target = join(work, 'cargo-target');
  try {
    run('cargo', ['build', '--locked', '--offline', '--jobs', '2', '--config', 'profile.dev.debug=0',
      '--config', 'build.incremental=false', '--manifest-path', join(harness, 'Cargo.toml')], work,
    {CARGO_TARGET_DIR: target});
  } finally {unchanged();}
  const originalPath = join(target, 'debug/prismpm-hologram-oracle');
  const original = binaryIdentity(originalPath);
  const executable = put('primary-live-oracle', original.bytes); chmodSync(executable, 0o500);
  const private_ = binaryIdentity(executable, 1n);
  assert.equal(private_.identity.sha256, original.identity.sha256);
  let retired = false;
  function verify() {
    assert.equal(retired, false, 'completed independent oracle is closed');
    unchanged();
    assert.deepEqual(binaryIdentity(originalPath, BigInt(original.identity.links)).identity, original.identity);
    assert.deepEqual(binaryIdentity(executable, 1n).identity, private_.identity);
  }
  return Object.freeze({work, verify,
    evidence: {revision, inputs: Object.fromEntries([...captured].map(([path, bytes]) => [path, sha(bytes)])),
      upstreamFiles, harnessFiles, original: original.identity, private: private_.identity},
    call(args) {
      verify();
      try {return JSON.parse(run(executable, args, work));}
      finally {verify();}
    },
    retire() {
      verify();
      assert.equal(realpathSync(target), target);
      assert.equal(lstatSync(target).uid, process.getuid());
      assert.equal(readFileSync(join(target, 'CACHEDIR.TAG'), 'utf8').split('\n')[0],
        'Signature: 8a477f597d28d172789f06886806bc55');
      const receipt = {scope:'completed-private-upstream-cargo-cache-only',
        original:original.identity, preservedExecutable:private_.identity,
        upstreamFiles, harnessFiles};
      run('cargo', ['clean', '--manifest-path', join(harness, 'Cargo.toml'), '--target-dir', target], work);
      assert.equal(existsSync(originalPath), false);
      unchanged(); assert.deepEqual(binaryIdentity(executable, 1n).identity, private_.identity);
      retired = true;
      writeFileSync(join(work, 'oracle-cache-retirement.json'), JSON.stringify(receipt) + '\n', {flag:'wx',mode:0o444});
      return receipt;
    },
  });
}
