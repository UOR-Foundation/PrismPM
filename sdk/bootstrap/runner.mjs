import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { spawn } from 'node:child_process';
import {
  constants,
  closeSync,
  fstatSync,
  lstatSync,
  openSync,
  readdirSync,
  readSync,
} from 'node:fs';
import { machine } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

export const RUNTIME = '/opt/prismpm/bootstrap-0.2.0';
const digest = (bytes) => createHash('sha256').update(bytes).digest('hex');
const keys = (value, expected) =>
  assert.deepEqual(
    Object.keys(value).sort(),
    expected.slice().sort(),
    'closed bootstrap runtime fields',
  );
const hex = (value) => assert.match(value, /^[a-f0-9]{64}$/);
const relative = (value) => {
  assert.match(value, /^(?:[a-zA-Z0-9_-]+\/)*[a-zA-Z0-9_.-]+$/);
  assert(
    value.split('/').every((part) => part !== '.' && part !== '..'),
    'dot path component',
  );
};

export function validateLock(value) {
  keys(value, [
    'schema',
    'archive_sha256',
    'binary_sha256',
    'qemu',
    'packages',
    'libraries',
    'licenses',
  ]);
  assert.equal(value.schema, 'bootstrap-runtime-lock/1');
  assert.equal(
    value.archive_sha256,
    'f3dd999f5618db154fa06222a06f9de95d86e1dbf683954426ea91c974cbe24c',
  );
  assert.equal(
    value.binary_sha256,
    'fed990f31a1cdc5f819f5a99e79bc441c7db11f7c8ea45c748d9cec1097ee8e6',
  );
  keys(value.qemu, ['amd64', 'arm64']);
  keys(value.packages, ['libc', 'libgcc', 'gcc-license']);
  for (const [name, row] of [
    ...Object.entries(value.packages),
    ...Object.entries(value.qemu),
  ]) {
    keys(
      row,
      name === 'amd64' || name === 'arm64'
        ? ['url', 'sha256', 'binary_sha256']
        : ['url', 'sha256'],
    );
    assert.match(
      row.url,
      /^https:\/\/snapshot\.debian\.org\/archive\/debian\/20260901T000000Z\/pool\/main\/(?:q\/qemu|g\/(?:glibc|gcc-12))\/[a-zA-Z0-9_+.~-]+\.deb$/,
    );
    hex(row.sha256);
    if (row.binary_sha256) hex(row.binary_sha256);
  }
  const expected = [
    'sysroot/lib/x86_64-linux-gnu/libc.so.6',
    'sysroot/lib/x86_64-linux-gnu/libgcc_s.so.1',
    'sysroot/lib64/ld-linux-x86-64.so.2',
  ];
  assert.deepEqual(
    value.libraries.map((row) => row.path),
    expected,
  );
  assert.deepEqual(
    value.licenses.map((row) => row.path),
    ['licenses/gcc', 'licenses/glibc', 'licenses/qemu'],
  );
  for (const row of [...value.libraries, ...value.licenses]) {
    keys(row, ['package', 'member', 'path', 'sha256']);
    relative(row.member);
    relative(row.path);
    hex(row.sha256);
    assert(['qemu', ...Object.keys(value.packages)].includes(row.package));
  }
  return value;
}

const lockBytes = readBoundedFile(
  new URL('./runtime.lock.json', import.meta.url),
  false,
  16384,
);
export const LOCK = validateLock(JSON.parse(lockBytes));
assert.equal(
  `${JSON.stringify(LOCK, null, 2)}\n`,
  lockBytes.toString(),
  'canonical bootstrap lock',
);

export function checkedFile(
  path,
  expected,
  executable = false,
  limit = 64 * 1024 * 1024,
) {
  const bytes = readBoundedFile(path, executable, limit);
  assert.equal(
    digest(bytes),
    expected,
    `bootstrap input digest mismatch: ${path}`,
  );
}

function readBoundedFile(path, executable, limit) {
  assert(
    lstatSync(path).isFile(),
    `bootstrap input must be a regular file: ${path}`,
  );
  const fd = openSync(path, constants.O_RDONLY | constants.O_NOFOLLOW);
  try {
    const stat = fstatSync(fd, { bigint: true });
    assert(
      stat.isFile() && stat.size <= BigInt(limit),
      `bootstrap input must be a bounded regular file: ${path}`,
    );
    assert(
      !executable || (stat.mode & 0o111n) !== 0n,
      `bootstrap input must be executable: ${path}`,
    );
    const bytes = Buffer.alloc(Number(stat.size));
    for (let offset = 0; offset < bytes.length; ) {
      const count = readSync(fd, bytes, offset, bytes.length - offset, null);
      assert(count > 0, 'bootstrap input shortened during read');
      offset += count;
    }
    assert.equal(
      readSync(fd, Buffer.alloc(1), 0, 1, null),
      0,
      'bootstrap input grew during read',
    );
    const after = fstatSync(fd, { bigint: true });
    for (const key of [
      'dev',
      'ino',
      'size',
      'mode',
      'nlink',
      'mtimeNs',
      'ctimeNs',
    ])
      assert.equal(
        after[key],
        stat[key],
        'bootstrap input changed during read',
      );
    return bytes;
  } finally {
    closeSync(fd);
  }
}

export function platformMode(platform, arch, hostMachine) {
  if (platform === 'linux' && arch === 'x64' && hostMachine === 'x86_64')
    return 'direct-amd64';
  if (platform === 'linux' && arch === 'arm64' && hostMachine === 'aarch64')
    return 'explicit-qemu-amd64';
  throw new Error(
    'historical bootstrap requires compatible Linux x64 or arm64',
  );
}

export function cleanEnvironment(environment) {
  return Object.fromEntries(
    Object.entries(environment).filter(
      ([key]) =>
        !key.startsWith('LD_') &&
        !key.startsWith('QEMU_') &&
        !['GLIBC_TUNABLES', 'GCONV_PATH', 'LOCPATH'].includes(key),
    ),
  );
}

export function verifyRuntime(root, arch) {
  assert(arch === 'amd64' || arch === 'arm64');
  const expected = [
    'qemu-x86_64-static',
    'runtime.lock.json',
    ...LOCK.libraries.map((row) => row.path),
    ...LOCK.licenses.map((row) => row.path),
  ].sort();
  const files = [];
  const directories = new Set(
    expected.flatMap((path) =>
      path
        .split('/')
        .slice(0, -1)
        .map((_, index) =>
          path
            .split('/')
            .slice(0, index + 1)
            .join('/'),
        ),
    ),
  );
  const walk = (directory, prefix = '') => {
    assert(
      lstatSync(directory).isDirectory(),
      'bootstrap runtime directory cannot be a symlink',
    );
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const relative = `${prefix}${entry.name}`;
      assert(
        expected.includes(relative) || directories.has(relative),
        'complete bootstrap runtime file closure',
      );
      if (entry.isDirectory()) {
        assert(directories.has(relative), 'unexpected bootstrap directory');
        walk(join(directory, entry.name), `${relative}/`);
      } else {
        assert(
          entry.isFile(),
          'bootstrap runtime rejects links and special files',
        );
        files.push(relative);
      }
    }
  };
  walk(root);
  assert.deepEqual(
    files.sort(),
    expected,
    'complete bootstrap runtime file closure',
  );
  checkedFile(join(root, 'runtime.lock.json'), digest(lockBytes));
  checkedFile(
    join(root, 'qemu-x86_64-static'),
    LOCK.qemu[arch].binary_sha256,
    true,
  );
  for (const row of [...LOCK.libraries, ...LOCK.licenses])
    checkedFile(
      join(root, row.path),
      row.sha256,
      row.path.startsWith('sysroot/'),
    );
}

export function qemuArguments(binary, args) {
  return [
    '-L',
    `${RUNTIME}/sysroot`,
    '-E',
    `LD_LIBRARY_PATH=${RUNTIME}/sysroot/lib/x86_64-linux-gnu`,
    binary,
    ...args,
  ];
}

export function historicalInvocation(binary, args) {
  assert(
    Array.isArray(args) &&
      args.every((arg) => typeof arg === 'string' && !arg.includes('\0')),
  );
  binary = resolve(binary);
  checkedFile(binary, LOCK.binary_sha256, true);
  const mode = platformMode(process.platform, process.arch, machine());
  const environment = cleanEnvironment(process.env);
  if (mode === 'direct-amd64')
    return { command: binary, args, environment, mode };
  verifyRuntime(RUNTIME, 'arm64');
  return {
    command: `${RUNTIME}/qemu-x86_64-static`,
    args: qemuArguments(binary, args),
    environment,
    mode,
  };
}

// Inherit the original streams; forward only this runner's signals to its child.
export async function runInherited(command, args, env) {
  const child = spawn(command, args, { env, stdio: 'inherit' });
  const signals = ['SIGHUP', 'SIGINT', 'SIGTERM'];
  const listeners = signals.map((signal) => [signal, () => child.kill(signal)]);
  for (const [signal, listener] of listeners) process.on(signal, listener);
  const result = await new Promise((resolve, reject) => {
    child.once('error', reject);
    child.once('exit', (code, signal) => resolve({ code, signal }));
  }).finally(() => {
    for (const [signal, listener] of listeners)
      process.removeListener(signal, listener);
  });
  if (result.signal) process.kill(process.pid, result.signal);
  else process.exitCode = result.code;
}

if (
  process.argv[1] &&
  resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  const args = process.argv.slice(2),
    describe = args[0] === '--describe';
  if (describe) args.shift();
  assert(
    args.length >= 1,
    'usage: runner.mjs [--describe] HISTORICAL_BINARY [ARGUMENT...]',
  );
  const invocation = historicalInvocation(args.shift(), args);
  if (describe) {
    process.stdout.write(
      `${JSON.stringify({ scope: 'historical-bootstrap-only', version: '0.2.0', mode: invocation.mode, process_architecture: process.arch, binary_sha256: LOCK.binary_sha256, runtime_lock_sha256: digest(lockBytes) })}\n`,
    );
  } else
    await runInherited(
      invocation.command,
      invocation.args,
      invocation.environment,
    );
}
