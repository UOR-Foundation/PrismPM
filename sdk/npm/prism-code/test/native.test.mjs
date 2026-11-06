// The fast check on the host: which prismpm `build --check` runs, and when it falls back to the SDK image.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { chmodSync, mkdirSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { lockedPrismpmVersion, nativeCandidates, parsePrismpmVersion, resolveNativePrismpm } from '../src/native.mjs';

const bin = fileURLToPath(new URL('../bin/prism-code.mjs', import.meta.url));
const IMAGE = 'ghcr.io/uor-foundation/prismpm-sdk@sha256:36da25ef924d84899453f24ec0c117a7a220e708037933eb8b499caa7723d99e';
const scratch = () => mkdtempSync(join(tmpdir(), 'prism-native-'));

/** A prismpm.lock shaped like a real one: one inventory per platform, each naming the prismpm binary. */
function lock(versions = ['0.4.0', '0.4.0']) {
  const platforms = versions.map((version, i) => ({
    platform: ['linux/amd64', 'linux/arm64'][i],
    inventory: [
      { id: 'lexlean', kind: 'binary', version: '0.4.0', digest: `sha256:${'a'.repeat(64)}` },
      ...(version ? [{ id: 'prismpm', kind: 'binary', version, digest: `sha256:${'b'.repeat(64)}` }] : []),
      { id: 'prismpm-conformance', kind: 'binary', version: '9.9.9', digest: `sha256:${'c'.repeat(64)}` },
    ],
  }));
  return JSON.stringify({ schema: 'prismpm/sdk-lock/3', sdk_image: IMAGE, platforms });
}

test('the locked prismpm version is the one every platform inventory names', () => {
  assert.equal(lockedPrismpmVersion(lock()), '0.4.0');
  assert.equal(lockedPrismpmVersion(lock(['0.4.0', '0.3.0'])), null, 'platforms that disagree pin nothing the host can match');
  assert.equal(lockedPrismpmVersion(lock([null, null])), null, 'prismpm-conformance is not prismpm');
  assert.equal(lockedPrismpmVersion(JSON.stringify({ sdk_image: IMAGE })), null);
  assert.equal(lockedPrismpmVersion('not json'), null);
});

test('a --version line names the version, and nothing else does', () => {
  assert.equal(parsePrismpmVersion('prismpm 0.4.0\n'), '0.4.0');
  assert.equal(parsePrismpmVersion('prismpm 0.4.0\r\n'), '0.4.0');
  assert.equal(parsePrismpmVersion('lexlean 0.4.0\n'), null);
  assert.equal(parsePrismpmVersion('prismpm 0.4.0\nextra\n'), null);
  assert.equal(parsePrismpmVersion(null), null);
});

test('candidates: $PRISM_PRISMPM, then the versioned cache in $PRISM_HOME, then PATH', () => {
  const env = { PRISM_HOME: '/h/.prism', PRISM_PRISMPM: '/opt/prismpm' };
  assert.deepEqual(nativeCandidates({ version: '0.4.0', env, platform: 'linux' }), [
    { path: '/opt/prismpm', source: 'PRISM_PRISMPM' },
    { path: join('/h/.prism', 'bin', 'prismpm-0.4.0', 'prismpm'), source: 'cache' },
    { path: 'prismpm', source: 'PATH' },
  ]);
  const win = nativeCandidates({ version: '0.4.0', env: { PRISM_HOME: 'C:\\p' }, platform: 'win32' });
  assert.deepEqual(win.map((c) => c.source), ['cache', 'PATH']);
  assert.ok(win[0].path.endsWith('prismpm.exe') && win[1].path === 'prismpm.exe');
});

test('the first candidate of the locked version wins; another version is passed over with a note', () => {
  const versions = { '/opt/prismpm': 'prismpm 0.3.0\n', 'prismpm': 'prismpm 0.4.0\n' };
  const probed = [];
  const probe = (path) => (probed.push(path), versions[path] ?? null);
  const env = { PRISM_HOME: '/h/.prism', PRISM_PRISMPM: '/opt/prismpm' };
  const r = resolveNativePrismpm({ lockText: lock(), probe, env, platform: 'linux' });
  assert.equal(r.path, 'prismpm');
  assert.equal(r.version, '0.4.0');
  assert.deepEqual(r.notes, ['/opt/prismpm is prismpm 0.3.0, and prismpm.lock pins 0.4.0']);
  assert.equal(probed.length, 3, 'the cache was looked at between them');
});

test('the image is used when nothing matches, the lock pins no version, or PRISM_NATIVE=0', () => {
  const any = () => 'prismpm 0.4.0\n';
  const none = () => null;
  const env = { PRISM_HOME: '/h/.prism' };
  assert.deepEqual(resolveNativePrismpm({ lockText: lock(), probe: none, env, platform: 'linux' }), { path: null, version: '0.4.0', notes: [] });
  assert.equal(resolveNativePrismpm({ lockText: JSON.stringify({ sdk_image: IMAGE }), probe: any, env }).path, null);
  assert.equal(resolveNativePrismpm({ lockText: lock(['0.4.0', '0.3.0']), probe: any, env }).path, null);
  assert.equal(resolveNativePrismpm({ lockText: lock(), probe: any, env: { ...env, PRISM_NATIVE: '0' } }).path, null);
  const missing = resolveNativePrismpm({ lockText: lock(), probe: none, env: { ...env, PRISM_PRISMPM: '/nope' }, platform: 'linux' });
  assert.deepEqual(missing.notes, ['PRISM_PRISMPM=/nope does not run'], 'an explicit choice that does not run is said, not ignored');
});

// The CLI end to end with a stand-in prismpm. A shell script is an executable only on POSIX hosts;
// the native-check workflow runs the real prismpm on Windows, macOS and Linux.
const posix = process.platform !== 'win32';

function project(lockText = lock()) {
  const dir = scratch();
  writeFileSync(join(dir, 'prismpm.lock'), lockText);
  return dir;
}

function fakePrismpm(version, record) {
  const dir = scratch();
  const path = join(dir, 'prismpm');
  writeFileSync(path, `#!/bin/sh\nif [ "$1" = "--version" ]; then echo "prismpm ${version}"; exit 0; fi\necho "$PWD|$*" > "${record}"\nexit 7\n`);
  chmodSync(path, 0o755);
  return path;
}

function prismCode(cwd, args, env) {
  const base = { ...process.env, PRISM_HOME: join(scratch(), '.prism'), PATH: process.env.PATH };
  delete base.PRISM_PRISMPM;
  delete base.PRISM_NATIVE;
  delete base.PRISM_CODE_DRY_RUN;
  return spawnSync(process.execPath, [bin, ...args], { cwd, env: { ...base, ...env }, encoding: 'utf8' });
}

test('build --check runs the matching host prismpm in the project directory and returns its exit code', { skip: !posix }, () => {
  const dir = project();
  const record = join(scratch(), 'ran.txt');
  const r = prismCode(dir, ['build', '--check', '--config', 'prismpm.toml'], { PRISM_PRISMPM: fakePrismpm('0.4.0', record) });
  assert.equal(r.status, 7, r.stderr);
  const [cwd, argv] = spawnSync('cat', [record], { encoding: 'utf8' }).stdout.trim().split('|');
  assert.equal(argv, 'check --config prismpm.toml');
  assert.equal(spawnSync('sh', ['-c', `cd "${dir}" && pwd -P`], { encoding: 'utf8' }).stdout.trim(), spawnSync('sh', ['-c', `cd "${cwd}" && pwd -P`], { encoding: 'utf8' }).stdout.trim());
});

test('the deprecated check spelling takes the same host path', { skip: !posix }, () => {
  const dir = project();
  const prismpm = fakePrismpm('0.4.0', join(scratch(), 'ran.txt'));
  const r = prismCode(dir, ['check'], { PRISM_PRISMPM: prismpm, PRISM_CODE_DRY_RUN: '1' });
  assert.equal(r.status, 0, r.stderr);
  assert.equal(r.stdout, `${prismpm} check\n`);
});

test('a host prismpm of another version is passed over for the image, with a note', { skip: !posix }, () => {
  const dir = project();
  const r = prismCode(dir, ['build', '--check'], { PRISM_PRISMPM: fakePrismpm('0.3.0', join(scratch(), 'ran.txt')), PRISM_CODE_DRY_RUN: '1', PRISM_DOCKER: 'docker' });
  assert.equal(r.status, 0, r.stderr);
  assert.match(r.stderr, /is prismpm 0\.3\.0, and prismpm\.lock pins 0\.4\.0; checking in the SDK image/);
  assert.match(r.stdout, new RegExp(`${IMAGE.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')} prismpm check\\n$`));
});

test('build, verify and conformance stay in the image even with a matching host prismpm', { skip: !posix }, () => {
  const dir = project();
  const prismpm = fakePrismpm('0.4.0', join(scratch(), 'ran.txt'));
  for (const args of [['build'], ['verify'], ['conformance']]) {
    const r = prismCode(dir, args, { PRISM_PRISMPM: prismpm, PRISM_CODE_DRY_RUN: '1', PRISM_DOCKER: 'docker' });
    assert.equal(r.status, 0, r.stderr);
    assert.ok(!r.stdout.startsWith(prismpm), `${args[0]} ran on the host: ${r.stdout}`);
    assert.match(r.stdout, new RegExp(`prismpm ${args[0]}\\n$`));
  }
});

test('PRISM_NATIVE=0 keeps the check in the image', { skip: !posix }, () => {
  const dir = project();
  const prismpm = fakePrismpm('0.4.0', join(scratch(), 'ran.txt'));
  const r = prismCode(dir, ['build', '--check'], { PRISM_PRISMPM: prismpm, PRISM_NATIVE: '0', PRISM_CODE_DRY_RUN: '1', PRISM_DOCKER: 'docker' });
  assert.equal(r.status, 0, r.stderr);
  assert.match(r.stdout, /prismpm check\n$/);
  assert.ok(!r.stdout.startsWith(prismpm));
});

test('the versioned cache in $PRISM_HOME is found without any variable', { skip: !posix }, () => {
  const dir = project();
  const home = join(scratch(), '.prism');
  const cached = join(home, 'bin', 'prismpm-0.4.0');
  mkdirSync(cached, { recursive: true });
  const fake = fakePrismpm('0.4.0', join(scratch(), 'ran.txt'));
  writeFileSync(join(cached, 'prismpm'), spawnSync('cat', [fake], { encoding: 'utf8' }).stdout);
  chmodSync(join(cached, 'prismpm'), 0o755);
  const r = spawnSync(process.execPath, [bin, 'build', '--check'], { cwd: dir, env: { ...process.env, PRISM_HOME: home, PRISM_PRISMPM: '', PRISM_CODE_DRY_RUN: '1' }, encoding: 'utf8' });
  assert.equal(r.status, 0, r.stderr);
  assert.equal(r.stdout, `${join(cached, 'prismpm')} check\n`);
});
