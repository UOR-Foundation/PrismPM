import assert from 'node:assert/strict';
import {mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, statfsSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname, join} from 'node:path';
import {repository, run, sha} from '../browser-view/compile.mjs';

const testNames = [
  'pinned_executor_refuses_browser_selector_before_session_start',
  'pinned_provider_refuses_browser_selector_before_preparation_or_attachment',
];

export function verifyLiveBoundary() {
  const paths = [
    'vendor/hologram-live.tar',
    'model/dependencies.toml',
    'crates/prismpm/src/verification.rs',
    'scripts/fetch-oracle-cargo.sh',
    'tests/hologram-oracle/Cargo.toml',
    'tests/hologram-oracle/Cargo.lock',
    'tests/hologram-oracle/tests/browser_surface.rs',
    'tests/data/holo-browser-codec-v1.json',
  ];
  const source = new Map(paths.map(path => [path, readFileSync(join(repository, path))]));
  const text = path => source.get(path).toString('utf8');
  const dependency = text('model/dependencies.toml').split('[[dependency]]').slice(1)
    .filter(section => /^id = "hologram-live"$/m.test(section));
  assert.equal(dependency.length, 1, 'one pinned Live validation oracle');
  assert.match(dependency[0], /^role = "validation-oracle"$/m);
  assert.match(dependency[0], /^revision = "[0-9a-f]{40}"$/m);
  const artifacts = dependency[0].split('[[dependency.artifact]]').slice(1)
    .filter(section => /^path = "vendor\/hologram-live.tar"$/m.test(section));
  assert.equal(artifacts.length, 1, 'one exact Live source archive');
  const digest = /^sha256 = "([0-9a-f]{64})"$/m.exec(artifacts[0])?.[1];
  assert.ok(digest, 'declared immutable Live source digest');
  const runtimePins = [...text('crates/prismpm/src/verification.rs').matchAll(
    /const HOLOGRAM_ORACLE_SOURCE_SHA256: &str =\s*"([0-9a-f]{64})";/g)];
  assert.equal(runtimePins.length, 1, 'one runtime verification source pin');
  assert.equal(digest, runtimePins[0][1], 'model and executing oracle use identical source');
  assert.equal(sha(source.get('vendor/hologram-live.tar')), digest, 'unchanged pinned Live archive');
  for (const file of ['Cargo.toml', 'Cargo.lock']) {
    const path = 'tests/hologram-oracle/' + file;
    const pins = text('scripts/fetch-oracle-cargo.sh').split('\n')
      .map(line => /^([0-9a-f]{64})  (\S+)$/.exec(line)).filter(row => row?.[2] === path);
    assert.equal(pins.length, 1, 'one acquired oracle dependency graph pin: ' + file);
    assert.equal(sha(source.get(path)), pins[0][1], 'unchanged oracle dependency graph: ' + file);
  }

  const target = join(repository, 'target/holo-browser-live-boundary');
  mkdirSync(target, {recursive: true});
  assert.equal(realpathSync(target), target, 'compiler target must not escape through a symlink');
  const targetDisk = statfsSync(target, {bigint: true});
  assert.ok(targetDisk.bavail * targetDisk.bsize >= 12n * 1024n ** 3n,
    '12 GiB source-build reserve on the compiler target filesystem');
  // The pinned tar is 4,485,120 bytes and expands to 4,160,643 regular-file
  // bytes across 343 files/112 directories. Keep extraction separate from
  // compilation: the SDK intentionally gives /tmp a 256 MiB tmpfs.
  const scratchDisk = statfsSync(tmpdir(), {bigint: true});
  assert.ok(scratchDisk.bavail * scratchDisk.bsize >= 32n * 1024n ** 2n,
    '32 MiB temporary archive/extraction headroom');
  const work = mkdtempSync(join(tmpdir(), 'prismpm-browser-live-'));
  let completed = false;
  try {
    const put = (target, bytes) => {
      mkdirSync(dirname(target), {recursive: true});
      writeFileSync(target, bytes, {flag: 'wx'});
    };
    const archive = join(work, 'hologram-live.tar');
    put(archive, source.get('vendor/hologram-live.tar'));
    const upstream = join(work, 'hologram-live');
    mkdirSync(upstream);
    run('tar', ['-xf', archive, '-C', upstream], work);
    const harness = join(work, 'hologram-oracle');
    for (const file of ['Cargo.toml', 'Cargo.lock', 'tests/browser_surface.rs']) {
      put(join(harness, file), source.get('tests/hologram-oracle/' + file));
    }
    put(join(work, 'data/holo-browser-codec-v1.json'),
      source.get('tests/data/holo-browser-codec-v1.json'));
    const output = run('cargo', [
      'test', '--locked', '--offline', '--jobs', '1',
      '--config', 'profile.dev.debug=0', '--config', 'profile.test.debug=0',
      '--config', 'build.incremental=false', '--manifest-path', join(harness, 'Cargo.toml'),
      '--test', 'browser_surface', '--', '--nocapture', '--test-threads=1',
    ], work, {CARGO_TARGET_DIR: target});
    assert.match(output,
      /test result: ok\. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;/);
    assert.deepEqual([...output.matchAll(/^test ([a-z_]+) \.\.\. ok$/gm)]
      .map(row => row[1]).sort(), testNames);
    for (const [path, bytes] of source) {
      assert.deepEqual(readFileSync(join(repository, path)), bytes, 'oracle source remained frozen: ' + path);
    }
    assert.deepEqual(readFileSync(join(harness, 'Cargo.lock')),
      source.get('tests/hologram-oracle/Cargo.lock'), 'locked graph remained unchanged');
    completed = true;
    return output;
  } finally {
    if (completed) rmSync(work, {recursive: true});
    else process.stderr.write('Retained failed Live boundary fixture: ' + work + '\n');
  }
}
