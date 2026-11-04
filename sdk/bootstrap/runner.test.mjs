import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync, spawn } from 'node:child_process';
import {
  chmodSync,
  cpSync,
  existsSync,
  lstatSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  readlinkSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import {
  checkedFile,
  cleanEnvironment,
  historicalInvocation,
  platformMode,
  qemuArguments,
  RUNTIME,
  validateLock,
  verifyRuntime,
} from './runner.mjs';
import {
  installRuntime,
  publishRuntime,
  TRANSIENT_CURL_STATUS,
  withTransientRetry,
} from './install.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const lock = JSON.parse(
  readFileSync(join(root, 'sdk/bootstrap/runtime.lock.json')),
);
const sha = (bytes) => createHash('sha256').update(bytes).digest('hex');

test('bootstrap acquisition disables ambient curl configuration before any download', () => {
  const work = mkdtempSync(join(tmpdir(), 'bootstrap-curl-config.'));
  try {
    writeFileSync(join(work, '.curlrc'), 'unregistered-prismpm-test-option = true\n');
    const module = new URL('./install.mjs', import.meta.url).href;
    execFileSync(process.execPath, ['--input-type=module', '-e', `
      import assert from 'node:assert/strict';
      import cp from 'node:child_process';
      import { syncBuiltinESMExports } from 'node:module';
      import { installRuntime } from ${JSON.stringify(module)};
      const original = cp.execFileSync;
      assert.match(cp.spawnSync('curl', ['--version'], { encoding: 'utf8' }).stderr,
        /unregistered-prismpm-test-option/);
      let calls = 0;
      cp.execFileSync = (command, args, options) => {
        if (command !== 'curl') return original(command, args, options);
        calls++;
        assert.equal(args[0], '--disable', 'curl must ignore user configuration');
        assert.equal(options.timeout, 310000);
        assert.deepEqual(options.stdio, ['ignore', 'pipe', 'inherit']);
        assert.equal(options.maxBuffer, 1024);
        const result = cp.spawnSync(command, [args[0], '--version'], { encoding: 'utf8' });
        assert.equal(result.status, 0);
        assert(!result.stderr.includes('unregistered-prismpm-test-option'));
        throw Error('verified acquisition boundary');
      };
      syncBuiltinESMExports();
      assert.throws(() => installRuntime(${JSON.stringify(join(work, 'runtime'))}),
        /verified acquisition boundary/);
      assert.equal(calls, 1);
    `], { env: { ...process.env, CURL_HOME: work }, timeout: 15000, stdio: 'pipe' });
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
});

test('supported Linux process architectures select direct or explicit emulated historical invocation', () => {
  assert.equal(platformMode('linux', 'x64', 'x86_64'), 'direct-amd64');
  assert.equal(
    platformMode('linux', 'arm64', 'aarch64'),
    'explicit-qemu-amd64',
  );
  for (const args of [
    ['darwin', 'arm64', 'arm64'],
    ['linux', 'x64', 'aarch64'],
    ['linux', 'arm64', 'x86_64'],
    ['linux', 'riscv64', 'riscv64'],
  ]) {
    assert.throws(() => platformMode(...args), /compatible Linux/);
  }
});

test('historical runtime lock rejects extra fields, mutable locations and altered identities', () => {
  validateLock(lock);
  for (const mutate of [
    (value) => {
      value.extra = true;
    },
    (value) => {
      value.schema = 'unrecognized';
    },
    (value) => {
      value.binary_sha256 = '0'.repeat(64);
    },
    (value) => {
      value.qemu.arm64.url = 'https://example.invalid/qemu';
    },
    (value) => {
      value.qemu.arm64.sha256 = 'latest';
    },
    (value) => {
      value.libraries[0].path = '../escape';
    },
    (value) => {
      value.libraries[0].member = '.';
    },
    (value) => {
      value.libraries[0].member = '..';
    },
    (value) => {
      value.libraries.push(value.libraries[0]);
    },
  ]) {
    const changed = structuredClone(lock);
    mutate(changed);
    assert.throws(() => validateLock(changed));
  }
});

test('binary authentication rejects substitutions, symlinks, directories and unbounded files', () => {
  const work = mkdtempSync(join(tmpdir(), 'bootstrap-runner-files.'));
  try {
    const file = join(work, 'binary');
    writeFileSync(file, 'pinned');
    chmodSync(file, 0o755);
    checkedFile(file, sha('pinned'), true);
    assert.throws(() => checkedFile(file, sha('changed'), true), /digest/);
    chmodSync(file, 0o644);
    assert.throws(() => checkedFile(file, sha('pinned'), true), /executable/);
    symlinkSync(file, join(work, 'alias'));
    assert.throws(
      () => checkedFile(join(work, 'alias'), sha('pinned')),
      /regular/,
    );
    mkdirSync(join(work, 'directory'));
    assert.throws(
      () => checkedFile(join(work, 'directory'), sha('')),
      /regular/,
    );
    assert.throws(() => checkedFile(file, sha('pinned'), false, 5), /bounded/);
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
});

test('bounded file reads reject growth and same-size mutation during the actual read', () => {
  const module = new URL('./runner.mjs', import.meta.url).href;
  for (const mutation of ['growth', 'mode']) {
    execFileSync(
      process.execPath,
      [
        '--input-type=module',
        '-e',
        `
      import fs from 'node:fs'; import { syncBuiltinESMExports } from 'node:module';
      import { tmpdir } from 'node:os'; import { join } from 'node:path'; import assert from 'node:assert/strict';
      import { checkedFile } from ${JSON.stringify(module)};
      const work = fs.mkdtempSync(join(tmpdir(), 'bootstrap-reader-race.')), file = join(work, 'input');
      fs.writeFileSync(file, 'pinned', { mode: 0o644 });
      const original = fs.readSync; let mutated = false, read = 0;
      fs.readSync = (...args) => { const count = original(...args); read += count;
        if (!mutated) { mutated = true; ${mutation === 'growth' ? 'fs.appendFileSync(file, "x".repeat(100000));' : 'fs.chmodSync(file, 0o600);'} } return count; };
      syncBuiltinESMExports();
      try { assert.throws(() => checkedFile(file, ${JSON.stringify(sha('pinned'))}), /during read/); assert(read <= 7); }
      finally { fs.readSync = original; syncBuiltinESMExports(); fs.rmSync(work, { recursive: true, force: true }); }
    `,
      ],
      { timeout: 10000 },
    );
  }
});

test('installer refuses existing destinations and cleans only its failed partial stage', () => {
  const work = mkdtempSync(join(tmpdir(), 'bootstrap-install-failure.'));
  try {
    const existing = join(work, 'existing'),
      missing = join(work, 'missing');
    mkdirSync(existing);
    writeFileSync(join(existing, 'sentinel'), 'retain');
    assert.throws(() => installRuntime(existing, work), /already exists/);
    assert.equal(readFileSync(join(existing, 'sentinel'), 'utf8'), 'retain');
    const empty = join(work, 'empty'),
      dangling = join(work, 'dangling');
    mkdirSync(empty);
    const emptyIdentity = lstatSync(empty).ino;
    assert.throws(() => installRuntime(empty, work), /already exists/);
    assert.equal(lstatSync(empty).ino, emptyIdentity);
    symlinkSync('absent-target', dangling);
    assert.throws(() => installRuntime(dangling, work), /already exists/);
    assert.equal(readlinkSync(dangling), 'absent-target');
    rmSync(empty, { recursive: true });
    rmSync(dangling);
    assert.throws(() => installRuntime(missing, work), /ENOENT/);
    assert(!existsSync(missing));
    assert.deepEqual(readdirSync(work), ['existing']);
    const packageName = new URL(lock.packages.libc.url).pathname
      .split('/')
      .at(-1);
    writeFileSync(join(work, packageName), 'bad package');
    assert.throws(() => installRuntime(missing, work), /digest/);
    assert(!existsSync(missing));
    assert(
      !readdirSync(work).some((name) => name.startsWith('.bootstrap-stage.')),
    );
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
});

test('runtime publication never clobbers an existing or late-created destination', () => {
  const module = new URL('./install.mjs', import.meta.url).href;
  const work = mkdtempSync(join(tmpdir(), 'bootstrap-publish.'));
  try {
    const stage = join(work, 'stage'),
      destination = join(work, 'published');
    mkdirSync(stage);
    writeFileSync(join(stage, 'sentinel'), 'verified runtime');
    const identity = lstatSync(stage).ino;
    publishRuntime(stage, destination);
    assert(!existsSync(stage));
    assert.equal(lstatSync(destination).ino, identity);
    assert.equal(
      readFileSync(join(destination, 'sentinel'), 'utf8'),
      'verified runtime',
    );
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
  for (const collision of ['empty-directory', 'dangling-link']) {
    execFileSync(
      process.execPath,
      [
        '--input-type=module',
        '-e',
        `
      import fs from 'node:fs'; import cp from 'node:child_process';
      import { syncBuiltinESMExports } from 'node:module';
      import { tmpdir } from 'node:os'; import { join } from 'node:path'; import assert from 'node:assert/strict';
      import { publishRuntime } from ${JSON.stringify(module)};
      const work = fs.mkdtempSync(join(tmpdir(), 'bootstrap-publish-race.'));
      const stage = join(work, 'stage'), destination = join(work, 'destination');
      fs.mkdirSync(stage); fs.writeFileSync(join(stage, 'sentinel'), 'retain');
      const original = cp.execFileSync; let identity;
      cp.execFileSync = (command, args, options) => {
        assert.equal(command, '/usr/bin/mv');
        ${collision === 'empty-directory' ? 'fs.mkdirSync(destination);' : 'fs.symlinkSync("absent-target", destination);'}
        identity = fs.lstatSync(destination).ino;
        return original(command, args, options);
      };
      syncBuiltinESMExports();
      try {
        assert.throws(() => publishRuntime(stage, destination), /destination already exists/);
        assert.equal(fs.lstatSync(destination).ino, identity);
        assert.equal(fs.readFileSync(join(stage, 'sentinel'), 'utf8'), 'retain');
        ${collision === 'empty-directory' ? 'assert.deepEqual(fs.readdirSync(destination), []);' : "assert.equal(fs.readlinkSync(destination), 'absent-target');"}
      } finally { cp.execFileSync = original; syncBuiltinESMExports(); fs.rmSync(work, { recursive: true, force: true }); }
    `,
      ],
      { timeout: 10000 },
    );
  }
});

test('loader and emulator overrides are removed without dropping project/tool configuration', () => {
  const env = cleanEnvironment({
    PATH: '/usr/bin',
    CARGO_HOME: '/cargo',
    ELAN_HOME: '/elan',
    LANG: 'C',
    LD_PRELOAD: 'bad',
    LD_LIBRARY_PATH: 'bad',
    QEMU_LD_PREFIX: 'bad',
    QEMU_SET_ENV: 'bad',
    GLIBC_TUNABLES: 'bad',
    GCONV_PATH: 'bad',
    LOCPATH: 'bad',
  });
  assert.deepEqual(env, {
    PATH: '/usr/bin',
    CARGO_HOME: '/cargo',
    ELAN_HOME: '/elan',
    LANG: 'C',
  });
  const args = qemuArguments('/tmp/prior with spaces', [
    '--project',
    '/tmp/project with spaces',
    'check',
    '--json',
  ]);
  assert.deepEqual(args, [
    '-L',
    '/opt/prismpm/bootstrap-0.2.0/sysroot',
    '-E',
    'LD_LIBRARY_PATH=/opt/prismpm/bootstrap-0.2.0/sysroot/lib/x86_64-linux-gnu',
    '/tmp/prior with spaces',
    '--project',
    '/tmp/project with spaces',
    'check',
    '--json',
  ]);
});

test('a current or arbitrary executable cannot substitute for the accepted historical binary', () => {
  assert.throws(() => historicalInvocation('/usr/bin/true', []), /digest/);
});

test('installed immutable runtime has complete closure and detects deletion, extras, aliases and modified code', () => {
  const arch = process.arch === 'arm64' ? 'arm64' : 'amd64';
  verifyRuntime(RUNTIME, arch);
  const work = mkdtempSync(join(tmpdir(), 'bootstrap-runtime-tamper.'));
  try {
    const copy = join(work, 'runtime');
    cpSync(RUNTIME, copy, { recursive: true });
    verifyRuntime(copy, arch);
    writeFileSync(join(copy, 'extra'), 'extra');
    assert.throws(() => verifyRuntime(copy, arch), /closure/);
    rmSync(join(copy, 'extra'));
    const libc = join(copy, 'sysroot/lib/x86_64-linux-gnu/libc.so.6'),
      original = readFileSync(libc);
    chmodSync(libc, 0o755);
    writeFileSync(libc, 'tampered');
    assert.throws(() => verifyRuntime(copy, arch), /digest/);
    rmSync(libc);
    assert.throws(() => verifyRuntime(copy, arch), /closure/);
    symlinkSync(join(RUNTIME, 'sysroot/lib/x86_64-linux-gnu/libc.so.6'), libc);
    assert.throws(() => verifyRuntime(copy, arch), /links/);
    rmSync(libc);
    writeFileSync(libc, original, { mode: 0o755 });
    assert.throws(
      () => verifyRuntime(copy, arch === 'arm64' ? 'amd64' : 'arm64'),
      /digest/,
    );
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
});

test('inherited execution preserves stdout, stderr, nonzero exit and child signal', async () => {
  const module = new URL('./runner.mjs', import.meta.url).href;
  const probe = async (code) => {
    const child = spawn(process.execPath, [
      '--input-type=module',
      '-e',
      `import { runInherited } from ${JSON.stringify(module)}; await runInherited(process.execPath, ['-e', ${JSON.stringify(code)}], process.env);`,
    ]);
    let stdout = '',
      stderr = '';
    child.stdout.on('data', (value) => {
      stdout += value;
    });
    child.stderr.on('data', (value) => {
      stderr += value;
    });
    const result = await new Promise((resolve, reject) => {
      child.once('error', reject);
      child.once('close', (status, signal) => resolve({ status, signal }));
    });
    return { ...result, stdout, stderr };
  };
  assert.deepEqual(
    await probe(
      'process.stdout.write("out"); process.stderr.write("err"); process.exit(23);',
    ),
    { status: 23, signal: null, stdout: 'out', stderr: 'err' },
  );
  assert.deepEqual(await probe('process.kill(process.pid, "SIGTERM");'), {
    status: null,
    signal: 'SIGTERM',
    stdout: '',
    stderr: '',
  });
});

test(
  'runner forwards termination to its own child before terminating',
  { timeout: 10000 },
  async () => {
    const module = new URL('./runner.mjs', import.meta.url).href;
    const child = spawn(process.execPath, [
      '--input-type=module',
      '-e',
      `import { runInherited } from ${JSON.stringify(module)}; await runInherited(process.execPath, ['-e', 'process.on("SIGTERM", () => { process.stdout.write("terminated"); process.exit(19); }); process.stdout.write("ready"); setInterval(() => {}, 1000);'], process.env);`,
    ]);
    let stdout = '';
    child.stdout.on('data', (value) => {
      stdout += value;
      if (stdout === 'ready') child.kill('SIGTERM');
    });
    const result = await new Promise((resolve, reject) => {
      child.once('error', reject);
      child.once('close', (status, signal) => resolve({ status, signal }));
    });
    assert.deepEqual(result, { status: 19, signal: null });
    assert.equal(stdout, 'readyterminated');
  },
);

test('both historical callers use the authenticated shared runner; current SDK stays direct', () => {
  const shell = readFileSync(join(root, 'scripts/bootstrap-verify.sh'), 'utf8');
  const evidence = readFileSync(
    join(root, 'scripts/bootstrap-evidence.test.mjs'),
    'utf8',
  );
  assert(shell.includes('sdk/bootstrap/runner.mjs'));
  assert(!/if ! "\$prior"|^"\$prior"/m.test(shell));
  assert(evidence.includes('../sdk/bootstrap/runner.mjs'));
  assert(!evidence.includes('run(prior,'));
  assert(
    shell.includes('cargo run --locked --offline --quiet --package prismpm'),
  );
  assert(
    readFileSync(join(root, 'xtask/src/main.rs'), 'utf8').includes(
      'sdk/bootstrap/runner.test.mjs',
    ),
  );
});

test('downloads retry only transient curl statuses, with bounded doubling backoff', () => {
  const failing = (statuses) => {
    let calls = 0;
    return {
      download: () => {
        const status = statuses[calls++];
        if (status === undefined) return 'downloaded';
        throw Object.assign(new Error(`curl ${status}`), { status });
      },
      calls: () => calls,
    };
  };
  const waits = [];
  const sleep = (milliseconds) => waits.push(milliseconds);
  const recovered = failing([56, 7]);
  assert.equal(withTransientRetry(recovered.download, { delay: 10, sleep }), 'downloaded');
  assert.equal(recovered.calls(), 3);
  assert.deepEqual(waits, [10, 20]);
  const exhausted = failing([56, 56, 56, 56, 56]);
  assert.throws(
    () => withTransientRetry(exhausted.download, { attempts: 3, delay: 1, sleep }),
    /curl 56/,
  );
  assert.equal(exhausted.calls(), 3);
  for (const status of [22, 23, 60, null]) {
    assert(!TRANSIENT_CURL_STATUS.has(status));
    let calls = 0;
    const permanent = () => {
      calls++;
      throw Object.assign(new Error('final'), status === null ? {} : { status });
    };
    assert.throws(() => withTransientRetry(permanent, { sleep }), /final/);
    assert.equal(calls, 1, `status ${status} must not be retried`);
  }
  assert.throws(() => withTransientRetry(() => 'x', { attempts: 0 }), /positive integer/);
});

test('HTTP acquisition retries explicit transient responses and discards failed bytes', async (t) => {
  const module = new URL('./install.mjs', import.meta.url).href;
  const success = { status: 0, http: '200', body: 'untrusted archive' };
  const failure = (http, status = 22) => ({ status, http, body: 'partial archive' });
  const cases = [
    ...[408, 429, 500, 502, 503, 504].map((http) => ({
      name: `HTTP ${http} recovers to digest verification`,
      steps: [failure(String(http)), success], calls: 2, message: /digest/,
    })),
    {
      name: 'mixed partial transfer and HTTP failures recover',
      steps: [failure('200', 18), failure('503'), success], calls: 3, message: /digest/,
    },
    {
      name: 'transient HTTP failures exhaust four attempts',
      steps: Array(5).fill(failure('503')), calls: 4, status: 22, httpStatus: 503,
    },
    {
      name: 'transient transport failures exhaust four attempts',
      steps: Array(5).fill(failure('000', 56)), calls: 4, status: 56, httpStatus: 0,
    },
    ...['400', '401', '403', '404', '407', '410', '501', '200', '000', '', '503\n', '504\n', '504\r\n', 'x504', '503\nnoise'].map((http) => ({
      name: `permanent or unknown HTTP ${JSON.stringify(http)} fails immediately`,
      steps: [failure(http), success], calls: 1, status: 22,
    })),
    {
      name: 'TLS verification failure cannot be retried by an HTTP status',
      steps: [failure('503', 60), success], calls: 1, status: 60,
    },
    {
      name: 'status output larger than the bound fails without retry',
      steps: [failure('5'.repeat(2048)), success], calls: 1, message: /ENOBUFS|maxBuffer/,
    },
    {
      name: 'successful corrupt archive fails integrity without retry',
      steps: [success, success], calls: 1, message: /digest/,
    },
    {
      name: 'failed partial bytes cannot survive a later empty success',
      steps: [failure('200', 18), { status: 0, http: '200' }],
      calls: 2, message: /ENOENT/,
    },
  ];
  for (const fixture of cases) await t.test(fixture.name, () => {
    const work = mkdtempSync(join(tmpdir(), 'bootstrap-http-retry.'));
    try {
      const log = join(work, 'calls.jsonl'), destination = join(work, 'runtime');
      // Exercise real execFileSync exit/stdout handling without remote requests.
      writeFileSync(join(work, 'curl'), `#!${process.execPath}
        const fs = require('node:fs');
        const args = process.argv.slice(2), log = process.env.BOOTSTRAP_TEST_LOG;
        const calls = fs.existsSync(log) ? fs.readFileSync(log, 'utf8').trim().split('\\n').length : 0;
        const step = JSON.parse(process.env.BOOTSTRAP_TEST_STEPS)[calls];
        const archive = args[args.indexOf('--output') + 1];
        fs.appendFileSync(log, JSON.stringify({ args, stale: fs.existsSync(archive) }) + '\\n');
        if (!step) throw Error('unexpected download attempt');
        if (step.body !== undefined) fs.writeFileSync(archive, step.body);
        process.stdout.write(step.http);
        process.exit(step.status);
      `, { mode: 0o755 });
      const result = JSON.parse(execFileSync(process.execPath, ['--input-type=module', '-e', `
        import assert from 'node:assert/strict';
        import fs from 'node:fs';
        import { installRuntime } from ${JSON.stringify(module)};
        const waits = [];
        Atomics.wait = (_array, _index, _value, milliseconds) => {
          const lines = fs.readFileSync(process.env.BOOTSTRAP_TEST_LOG, 'utf8').trim().split('\\n');
          const { args } = JSON.parse(lines.at(-1));
          assert(!fs.existsSync(args[args.indexOf('--output') + 1]), 'failed bytes must be removed before backoff');
          waits.push(milliseconds);
        };
        let failure;
        try { installRuntime(${JSON.stringify(destination)}); }
        catch (error) { failure = { status: error.status, httpStatus: error.httpStatus, message: error.message }; }
        assert(failure, 'fixture must fail acquisition or integrity');
        process.stdout.write(JSON.stringify({ ...failure, waits }));
      `], {
        env: {
          ...process.env,
          PATH: `${work}:${process.env.PATH}`,
          BOOTSTRAP_TEST_LOG: log,
          BOOTSTRAP_TEST_STEPS: JSON.stringify(fixture.steps),
        },
        timeout: 10000, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'],
      }));
      const calls = readFileSync(log, 'utf8').trim().split('\n').map((line) => JSON.parse(line));
      assert.equal(calls.length, fixture.calls);
      assert.deepEqual(result.waits, [2000, 4000, 8000].slice(0, fixture.calls - 1));
      if (fixture.message) assert.match(result.message, fixture.message);
      if (fixture.status !== undefined) assert.equal(result.status, fixture.status);
      if (fixture.httpStatus !== undefined) assert.equal(result.httpStatus, fixture.httpStatus);
      for (const call of calls) {
        assert.equal(call.stale, false, 'each attempt must start without prior bytes');
        assert.deepEqual(call.args, [
          '--disable', '--proto', '=https', '--proto-redir', '=https', '--tlsv1.2',
          '--fail', '--location', '--silent', '--show-error', '--max-time', '300',
          '--max-filesize', '67108864', '--write-out', '%{http_code}',
          lock.packages.libc.url, '--output', call.args.at(-1),
        ]);
        assert(!existsSync(call.args.at(-1)), 'failed installation must clean archive');
      }
      assert(!existsSync(destination));
      assert(!readdirSync(work).some((name) => name.startsWith('.bootstrap-stage.')));
    } finally {
      rmSync(work, { recursive: true, force: true });
    }
  });
});

test('HTTP retry uses the exact captured status string and a shared attempt budget', () => {
  for (const code of ['408', '429', '500', '502', '503', '504']) {
    let calls = 0; const waits = [];
    assert.equal(withTransientRetry(() => {
      if (++calls < 3) throw Object.assign(new Error('HTTP failure'), {status: 22, stdout: Buffer.from(code)});
      return 'downloaded';
    }, {delay: 10, sleep: ms => waits.push(ms)}), 'downloaded');
    assert.equal(calls, 3); assert.deepEqual(waits, [10, 20]);
  }
  for (const code of ['400', '401', '403', '404', '407', '410', '501', '200', '000', '', '503\n', '504\n', '504\r\n', 'x504']) {
    let calls = 0;
    assert.throws(() => withTransientRetry(() => {
      calls++; throw Object.assign(new Error('permanent'), {status: 22, stdout: Buffer.from(code)});
    }, {sleep: () => assert.fail('permanent failure must not wait')}), /permanent/);
    assert.equal(calls, 1);
  }
});
