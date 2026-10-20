import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {chmodSync,chownSync,lstatSync,mkdtempSync,mkdirSync,realpathSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname,isAbsolute,join} from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';

const initializer = fileURLToPath(new URL('../sdk/devcontainer-init.sh', import.meta.url));

function launch(limits, args) {
  const env = {...process.env};
  delete env.PRISMPM_EPHEMERAL_HOME;
  return spawnSync('/bin/sh', ['-c', `${limits}\nexec /bin/sh "$@"`, '--', initializer, ...args], {
    env, encoding: 'utf8', timeout: 10_000,
  });
}

test('SDK launcher supplies a bounded 64 MiB stack to the actual requested process', () => {
  const child = launch('ulimit -S -s 8192', ['/bin/sh', '-c', 'ulimit -S -s; ulimit -H -s']);
  assert.equal(child.error, undefined);
  assert.equal(child.signal, null);
  assert.equal(child.status, 0, child.stderr);
  const [soft, hard] = child.stdout.trim().split('\n');
  assert.equal(soft, '65536');
  const baseline = spawnSync('/bin/sh', ['-c', 'ulimit -H -s'], {encoding: 'utf8'});
  assert.equal(baseline.status, 0);
  assert.equal(hard, baseline.stdout.trim(), 'the SDK must preserve the host hard limit');
});

test('SDK launcher refuses an insufficient hard stack limit before executing the command', () => {
  const child = launch('ulimit -S -s 8192\nulimit -H -s 8192', ['/bin/sh', '-c', 'printf command-executed']);
  assert.equal(child.error, undefined);
  assert.equal(child.signal, null);
  assert.notEqual(child.status, 0);
  assert.equal(child.stdout, '');
  assert.match(child.stderr, /ulimit|stack/i);
});

test('SDK launcher preserves requested argv and nonzero exit status', () => {
  const child = launch('ulimit -S -s 8192', ['/bin/sh', '-c', 'printf "%s\\n" "$1"; exit 23', '--', 'literal argument']);
  assert.equal(child.error, undefined);
  assert.equal(child.signal, null);
  assert.equal(child.status, 23);
  assert.equal(child.stdout, 'literal argument\n');
});

// These observations execute in the real requested child after the actual
// initializer. The tiny marker skips only the unrelated Cargo cache copy.
const xdgKeys = ['XDG_CACHE_HOME', 'XDG_CONFIG_HOME', 'XDG_RUNTIME_DIR'];
const observeXdg = String.raw`
const fs = require('node:fs');
const path = require('node:path');
const keys = ['XDG_CACHE_HOME', 'XDG_CONFIG_HOME', 'XDG_RUNTIME_DIR'];
const fixture = process.argv[1];
function metadata(value) {
  const stat = fs.lstatSync(value);
  return {path: value, directory: stat.isDirectory(), symlink: stat.isSymbolicLink(),
    uid: stat.uid, gid: stat.gid, mode: stat.mode & 0o7777,
    realpath: fs.realpathSync(value)};
}
const rows = keys.map(key => {
  const value = process.env[key] ?? null;
  if (value === null || value === '') return {key, value};
  const observed = metadata(value);
  const probe = path.join(value, 'launcher-observation-' + process.pid);
  fs.writeFileSync(probe, 'owned-xdg\n', {flag: 'wx', mode: 0o600});
  const written = fs.readFileSync(probe, 'utf8');
  fs.unlinkSync(probe);
  return {key, value, ...observed, parent: metadata(path.dirname(value)), written};
});
const explicit = keys.map((key, index) => {
  const value = path.join(fixture, 'explicit-' + index);
  return {key, ...metadata(value), marker: fs.readFileSync(path.join(value, 'caller-marker'), 'utf8')};
});
fs.writeFileSync(1, JSON.stringify({rows, explicit, temporaryEntries: fs.readdirSync(process.env.TMPDIR).sort(),
  uid: process.getuid(), gid: process.getgid(), HOME: process.env.HOME,
  TMPDIR: process.env.TMPDIR, umask: process.umask(),
  seeded: fs.lstatSync(path.join(process.env.CARGO_HOME, '.seeded')).isFile()}) + '\n');
`;
const retireXdg = String.raw`
const assert = require('node:assert/strict');
const fs = require('node:fs');
const [root, expectedUid, expectedGid, expectedDev, expectedIno] = process.argv.slice(1);
const stat = fs.lstatSync(root);
assert(stat.isDirectory() && !stat.isSymbolicLink());
assert.equal(fs.realpathSync(root), root);
assert.equal(stat.uid, Number(expectedUid)); assert.equal(stat.gid, Number(expectedGid));
assert.equal(stat.dev, Number(expectedDev)); assert.equal(stat.ino, Number(expectedIno));
assert.equal(process.getuid(), Number(expectedUid)); assert.equal(process.getgid(), Number(expectedGid));
const temporary = require('node:path').join(root, 'tmp');
const directory = fs.lstatSync(temporary);
assert(directory.isDirectory() && !directory.isSymbolicLink());
assert.equal(directory.uid, Number(expectedUid)); assert.equal(directory.gid, Number(expectedGid));
fs.chmodSync(temporary, 0o1777);
fs.rmSync(root, {recursive: true});
assert.throws(() => fs.lstatSync(root), error => error.code === 'ENOENT');
`;

function xdgFixture() {
  const initialUid = process.getuid(), initialGid = process.getgid();
  const uid = initialUid === 0 ? 1000 : initialUid;
  const gid = initialUid === 0 ? 1000 : initialGid;
  const root = realpathSync(mkdtempSync(join(tmpdir(), 'sdk-launcher-xdg-test-')));
  const identity = lstatSync(root);
  const temporary = join(root, 'tmp'), home = join(root, 'caller-home');
  const cargo = join(temporary, 'prismpm-cargo-' + initialUid);
  for (const path of [temporary, home, cargo]) mkdirSync(path, {mode: 0o700});
  // Mirror /tmp search/write semantics for the initial-root entrypoint without
  // adding DAC capabilities; only the XDG defaults and explicit dirs are private.
  chmodSync(root, 0o711); chmodSync(temporary, 0o1777); chmodSync(cargo, 0o755);
  const seeded = join(cargo, '.seeded');
  writeFileSync(seeded, 'tiny-environment-regression-fixture\n', {flag: 'wx', mode: 0o600});
  const explicit = Object.fromEntries(xdgKeys.map((key, index) => {
    const path = join(root, 'explicit-' + index);
    mkdirSync(path, {mode: 0o700});
    writeFileSync(join(path, 'caller-marker'), 'unchanged-' + key, {flag: 'wx', mode: 0o600});
    return [key, path];
  }));
  // Transfer children before ancestors. Afterward a capability-restricted root
  // parent performs no reads inside: observations and cleanup run as the child.
  if (initialUid === 0) {
    for (const path of [seeded, ...Object.values(explicit).map(path => join(path, 'caller-marker')),
      cargo, ...Object.values(explicit), home, temporary, root]) chownSync(path, uid, gid);
  }
  const env = {...process.env, HOME: home, TMPDIR: temporary, UID: String(initialUid),
    PRISMPM_EPHEMERAL_HOME: '1'};
  delete env.NODE_TEST_CONTEXT;
  for (const key of xdgKeys) delete env[key];
  return {root, temporary, home, cargo, explicit, env, uid, gid, initialUid,
    dev: identity.dev, ino: identity.ino, initialEntries: ['prismpm-cargo-' + initialUid],
    expectedHome: initialUid === 0 || initialUid === 1000 ? '/home/vscode' : home};
}

function observeInitializer(fixture, environment = fixture.env) {
  const child = spawnSync('/bin/sh', ['-c', 'umask 0027\nexec /bin/sh "$@"', '--',
    initializer, process.execPath, '-e', observeXdg, fixture.root], {
    env: environment, encoding: 'utf8', timeout: 10_000,
  });
  assert.equal(child.error, undefined);
  assert.equal(child.signal, null);
  assert.equal(child.status, 0, child.stderr);
  const observed = JSON.parse(child.stdout);
  assert.equal(observed.uid, fixture.uid); assert.equal(observed.gid, fixture.gid);
  assert.equal(observed.HOME, fixture.expectedHome);
  assert.equal(observed.TMPDIR, fixture.temporary);
  assert.equal(observed.umask, 0o027, 'launcher must not change requested command umask');
  assert.equal(observed.seeded, true, 'tiny fixture bypasses only unrelated Cargo cache copying');
  return observed;
}

function fixtureOwnerNode(fixture, script, values) {
  const args = ['-e', script, ...values];
  const command = fixture.initialUid === 0 ? 'setpriv' : process.execPath;
  const argv = fixture.initialUid === 0
    ? ['--reuid=1000', '--regid=1000', '--init-groups', process.execPath, ...args] : args;
  const child = spawnSync(command, argv, {env: process.env, encoding: 'utf8', timeout: 10_000});
  assert.equal(child.error, undefined); assert.equal(child.signal, null);
  assert.equal(child.status, 0, child.stderr);
}

function retireXdgFixture(fixture) {
  fixtureOwnerNode(fixture, retireXdg, [fixture.root, String(fixture.uid), String(fixture.gid),
    String(fixture.dev), String(fixture.ino)]);
}

function assertPrivateDefault(row, fixture) {
  assert.equal(typeof row.value, 'string', row.key + ' private default');
  assert(isAbsolute(row.value));
  assert.equal(row.directory, true); assert.equal(row.symlink, false);
  assert.equal(row.realpath, row.value);
  assert.equal(row.uid, fixture.uid); assert.equal(row.gid, fixture.gid);
  assert.equal(row.mode, 0o700); assert.equal(row.written, 'owned-xdg\n');
  const parent = row.parent;
  assert.equal(dirname(parent.path), fixture.temporary);
  assert.equal(parent.directory, true); assert.equal(parent.symlink, false);
  assert.equal(parent.realpath, parent.path);
  assert.equal(parent.uid, fixture.uid); assert.equal(parent.gid, fixture.gid);
  assert.equal(parent.mode, 0o700);
  return parent.path;
}

function assertExplicitUnchanged(observed, fixture) {
  assert.deepEqual(observed.explicit.map(row => row.path), xdgKeys.map(key => fixture.explicit[key]));
  for (const row of observed.explicit) {
    assert.equal(row.marker, 'unchanged-' + row.key);
    assert.equal(row.directory, true); assert.equal(row.symlink, false);
    assert.equal(row.realpath, row.path);
    assert.equal(row.uid, fixture.uid); assert.equal(row.gid, fixture.gid);
    assert.equal(row.mode, 0o700);
  }
}

// The exact complete file also runs under real initial-root/1000/1001 profiles.
// No initializer, id, setpriv, or requested-child observation is mocked.
test('SDK ephemeral launcher creates writable private XDG defaults for the actual child identity', () => {
  const fixture = xdgFixture();
  try {
    const observed = observeInitializer(fixture);
    assert.equal(observed.rows.length, 3);
    const parents = observed.rows.map(row => assertPrivateDefault(row, fixture));
    assert.equal(new Set(observed.rows.map(row => row.value)).size, 3);
    assert.equal(new Set(parents).size, 1, 'one allocation for all default directories');
    assert.equal(observed.temporaryEntries.filter(name => !fixture.initialEntries.includes(name)).length, 1);
    assertExplicitUnchanged(observed, fixture);
  } finally { retireXdgFixture(fixture); }
});

test('SDK ephemeral launcher preserves explicit XDG values and defaults only missing or empty keys', () => {
  const fixture = xdgFixture();
  try {
    const all = observeInitializer(fixture, {...fixture.env, ...fixture.explicit});
    assert.deepEqual(all.rows.map(row => row.value), xdgKeys.map(key => fixture.explicit[key]));
    assert.deepEqual(all.temporaryEntries, fixture.initialEntries, 'no unused allocation with all keys explicit');
    assertExplicitUnchanged(all, fixture);
    const mixed = observeInitializer(fixture, {...fixture.env, XDG_CACHE_HOME: fixture.explicit.XDG_CACHE_HOME,
      XDG_CONFIG_HOME: ''});
    assert.equal(mixed.rows[0].value, fixture.explicit.XDG_CACHE_HOME);
    const parents = mixed.rows.slice(1).map(row => assertPrivateDefault(row, fixture));
    assert.equal(parents[0], parents[1]);
    assertExplicitUnchanged(mixed, fixture);
  } finally { retireXdgFixture(fixture); }
});

test('SDK XDG defaults are ephemeral opt-in and fresh on each actual launcher invocation', () => {
  const fixture = xdgFixture();
  try {
    const baseline = {...fixture.env, CARGO_HOME: fixture.cargo};
    delete baseline.PRISMPM_EPHEMERAL_HOME;
    const unselected = observeInitializer(fixture, baseline);
    assert.deepEqual(unselected.rows.map(row => row.value), [null, null, null]);
    assert.deepEqual(unselected.temporaryEntries, fixture.initialEntries);
    const first = observeInitializer(fixture), second = observeInitializer(fixture);
    const firstParents = first.rows.map(row => assertPrivateDefault(row, fixture));
    const secondParents = second.rows.map(row => assertPrivateDefault(row, fixture));
    assert.equal(new Set(firstParents).size, 1); assert.equal(new Set(secondParents).size, 1);
    assert.notEqual(firstParents[0], secondParents[0]);
    assert.equal(second.temporaryEntries.filter(name => !fixture.initialEntries.includes(name)).length, 2);
    assertExplicitUnchanged(first, fixture); assertExplicitUnchanged(second, fixture);
    // Exec leaves disposal with this test-owned temporary fixture's real owner.
  } finally { retireXdgFixture(fixture); }
});

test('SDK ephemeral launcher refuses failed XDG directory setup before the requested command', () => {
  const fixture = xdgFixture();
  try {
    fixtureOwnerNode(fixture, String.raw`
const assert = require('node:assert/strict');
const fs = require('node:fs');
const [temporary, uid, gid] = process.argv.slice(1);
const stat = fs.lstatSync(temporary);
assert(stat.isDirectory() && !stat.isSymbolicLink());
assert.equal(stat.uid, Number(uid)); assert.equal(stat.gid, Number(gid));
assert.equal(process.getuid(), Number(uid)); assert.equal(process.getgid(), Number(gid));
fs.chmodSync(temporary, 0o555);
assert.equal(fs.lstatSync(temporary).mode & 0o7777, 0o555);
`, [fixture.temporary, String(fixture.uid), String(fixture.gid)]);
    // The identical read-only TMPDIR still admits the actual cache/stack/
    // identity/Node path when all XDG settings are explicit. Missing defaults
    // are the only input changed for the following requested-command sentinel.
    const preflight = observeInitializer(fixture, {...fixture.env, ...fixture.explicit});
    assertExplicitUnchanged(preflight, fixture);
    const sentinel = spawnSync('/bin/sh', [initializer, '/bin/sh', '-c',
      'printf requested-command-started'], {
      env: fixture.env, encoding: 'utf8', timeout: 10_000,
    });
    assert.equal(sentinel.error, undefined); assert.equal(sentinel.signal, null);
    assert.notEqual(sentinel.status, 0, 'missing-default directory setup must refuse the command');
    assert.equal(sentinel.stdout, '', 'requested command must not start');
    assert.match(sentinel.stderr, /mktemp/);
    assert.match(sentinel.stderr, /Permission denied|cannot create/i);
  } finally { retireXdgFixture(fixture); }
});
