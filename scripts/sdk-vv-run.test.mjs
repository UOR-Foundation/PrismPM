import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { chmodSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { canonical, executionEnvironment, gateInvocation, retainTwoRuns, validateVvEvidence } from './sdk-vv-run.mjs';
import { stop } from './ci-observe.mjs';

const revision = '1'.repeat(40);
const image = `localhost:5000/prismpm@sha256:${'2'.repeat(64)}`;
const evidence = () => ({ commit: revision, gates: Array.from({ length: 15 }, (_, i) => i + 1), schema: 'prismpm/vv-evidence/1', status: 'passed' });
const sha = bytes => createHash('sha256').update(bytes).digest('hex');

function fixture(t) {
  const work = mkdtempSync(join(tmpdir(), 'sdk-vv-run-tests.'));
  t.after(() => rmSync(work, { recursive: true, force: true }));
  const source = join(work, 'source'), output = join(work, 'evidence');
  mkdirSync(join(source, 'target'), { recursive: true });
  mkdirSync(output);
  return { work, source, output, input: join(source, 'target/vv-evidence.json') };
}

test('exact raw VV evidence requires all ordered gates and the external source revision', () => {
  const bytes = Buffer.from(canonical(evidence()));
  assert.deepEqual(validateVvEvidence(bytes, revision), evidence());
  for (const mutate of [
    value => { value.gates.pop(); },
    value => { value.gates[4] = 4; },
    value => { value.gates.reverse(); },
    value => { value.status = 'skipped'; },
    value => { value.commit = '3'.repeat(40); },
    value => { value.schema = 'prismpm/vv-evidence/2'; },
    value => { value.accepted = true; },
  ]) {
    const value = evidence(); mutate(value);
    assert.throws(() => validateVvEvidence(Buffer.from(canonical(value)), revision));
  }
  for (const bytes of [Buffer.from('{}'), Buffer.from(JSON.stringify(evidence(), null, 2)), Buffer.from(canonical(evidence()) + '\n'), Buffer.alloc(4097)]) {
    assert.throws(() => validateVvEvidence(bytes, revision));
  }
});

test('two consecutive executions retain separate unchanged raw evidence after removing stale success', async t => {
  const f = fixture(t), bytes = Buffer.from(canonical(evidence()));
  writeFileSync(f.input, bytes);
  const calls = [];
  const retained = await retainTwoRuns(f.source, f.output, revision, async index => {
    assert(!existsSync(f.input), 'previous success must be invalidated before each execution');
    calls.push(index);
    writeFileSync(f.input, bytes);
    return 0;
  });
  assert.deepEqual(calls, [1, 2]);
  assert.deepEqual(retained, [1, 2].map(run => ({ run, path: `run-${run}/vv-evidence.json`, byte_length: bytes.length, sha256: sha(bytes) })));
  for (const row of retained) assert.deepEqual(readFileSync(join(f.output, row.path)), bytes);
});

test('failed, missing, wrong-commit or aliased results cannot promote a first success', async t => {
  for (const mode of ['nonzero', 'missing', 'wrong-commit', 'symlink', 'directory']) {
    const f = fixture(t), bytes = Buffer.from(canonical(evidence()));
    let calls = 0;
    await assert.rejects(retainTwoRuns(f.source, f.output, revision, async index => {
      calls++;
      if (index === 1) { writeFileSync(f.input, bytes); return 0; }
      if (mode === 'nonzero') { writeFileSync(f.input, bytes); return 7; }
      if (mode === 'missing') return 0;
      if (mode === 'directory') mkdirSync(f.input);
      if (mode === 'wrong-commit') writeFileSync(f.input, canonical({ ...evidence(), commit: '4'.repeat(40) }));
      if (mode === 'symlink') symlinkSync(join(f.output, 'run-1/vv-evidence.json'), f.input);
      return 0;
    }));
    assert.equal(calls, 2);
    assert.deepEqual(readFileSync(join(f.output, 'run-1/vv-evidence.json')), bytes);
    assert(!existsSync(join(f.output, 'run-2/vv-evidence.json')));
  }
});

test('second execution cannot alter, delete or alias the first retained receipt', async t => {
  for (const mode of ['tamper', 'delete', 'symlink']) {
    const f = fixture(t), bytes = Buffer.from(canonical(evidence()));
    let calls = 0;
    await assert.rejects(retainTwoRuns(f.source, f.output, revision, async index => {
      calls++;
      if (index === 2) {
        const retained = join(f.output, 'run-1/vv-evidence.json');
        if (mode === 'tamper') {
          // The retaining process owns this file; mode 0444 is not immutable.
          chmodSync(retained, 0o644);
          writeFileSync(retained, canonical({ ...evidence(), commit: '4'.repeat(40) }));
        } else {
          rmSync(retained);
          if (mode === 'symlink') symlinkSync(f.input, retained);
        }
      }
      writeFileSync(f.input, bytes);
      return 0;
    }));
    assert.equal(calls, 2);
    assert(!existsSync(join(f.output, 'execution.json')));
  }
});

test('SIGHUP cancels the actual observer, stops its monitor and cannot return success', { timeout: 25000 }, async t => {
  const f = fixture(t);
  const root = dirname(dirname(fileURLToPath(import.meta.url)));
  const directory = join(f.work, 'diagnostics'), ready = join(f.work, 'ready');
  const gate = `require('node:fs').writeFileSync(${JSON.stringify(ready)}, String(process.pid)); setTimeout(() => process.exit(0), 15000);`;
  const invocation = {
    command: process.execPath,
    args: [join(root, 'scripts/ci-observe.mjs'), 'run', directory, '--', process.execPath, '-e', gate],
    cwd: root,
  };
  const runner = new URL('./sdk-vv-run.mjs', import.meta.url).href;
  const wrapper = spawn(process.execPath, ['--input-type=module', '-e', `
    import { executeGate } from ${JSON.stringify(runner)};
    try { process.exitCode = await executeGate(${JSON.stringify(invocation)}, process.env); }
    catch { process.exitCode = 1; }
  `], { stdio: 'ignore' });
  let closed = false;
  const completion = new Promise((resolve, reject) => {
    wrapper.once('error', reject);
    wrapper.once('close', (code, signal) => { closed = true; resolve({ code, signal }); });
  });
  try {
    const deadline = Date.now() + 10000;
    while (!existsSync(ready)) {
      assert(!closed && Date.now() < deadline, 'actual observed gate must start');
      await delay(20);
    }
    wrapper.kill('SIGHUP');
    const result = await Promise.race([
      completion,
      delay(10000, undefined, { ref: false }).then(() => { throw Error('cancellation did not finish'); }),
    ]);
    assert.notEqual(result.code, 0);
    assert(existsSync(join(directory, 'stop')), 'observer monitor must be stopped cooperatively');
    const monitor = JSON.parse(readFileSync(join(directory, 'monitor-result.json')));
    assert.equal(monitor.reason, 'stopped');
    const observed = JSON.parse(readFileSync(join(directory, 'gate-result.json')));
    assert.equal(observed.state, 'completed');
    assert.notEqual(observed.exitCode, 0);
    const pid = Number(readFileSync(ready, 'utf8'));
    let state = 'absent';
    try {
      const stat = readFileSync(`/proc/${pid}/stat`, 'utf8');
      state = stat.slice(stat.lastIndexOf(')') + 2).split(' ')[0];
    } catch (error) { if (error.code !== 'ENOENT') throw error; }
    assert(['absent', 'Z'].includes(state), 'owned command must no longer execute');
  } finally {
    if (!closed) wrapper.kill('SIGKILL');
    await completion;
    // This exact private monitor must also be cleaned when testing the defect.
    if (existsSync(join(directory, 'owner.json'))) await stop(directory);
  }
});

test('first-run failure stops immediately and never overwrites caller evidence', async t => {
  const f = fixture(t);
  let calls = 0;
  await assert.rejects(retainTwoRuns(f.source, f.output, revision, async () => { calls++; return 1; }));
  assert.equal(calls, 1);
  assert(!existsSync(join(f.output, 'run-2')));
  const other = fixture(t);
  mkdirSync(join(other.output, 'run-1'));
  writeFileSync(join(other.output, 'run-1/sentinel'), 'retain');
  await assert.rejects(retainTwoRuns(other.source, other.output, revision, async () => { throw Error('must not execute'); }));
  assert.equal(readFileSync(join(other.output, 'run-1/sentinel'), 'utf8'), 'retain');
});

test('exact full gate invocation has no alternate binary, source command or login shell', () => {
  assert.deepEqual(gateInvocation('/fresh/source', '/fresh/evidence/run-1'), {
    command: '/usr/local/bin/node',
    args: ['/fresh/source/scripts/ci-observe.mjs', 'run', '/fresh/evidence/run-1/diagnostics', '--', '/usr/local/bin/just', 'vv'],
    cwd: '/fresh/source',
  });
});

test('execution environment uses fresh writable caches and fixed SDK tools, not caller overrides', () => {
  const environment = executionEnvironment('/fresh', image, 'unix:///var/run/docker.sock');
  assert.equal(environment.PRISMPM_TEST_SDK_IMAGE, image);
  assert.equal(environment.CARGO_HOME, '/fresh/cargo');
  assert.equal(environment.CARGO_TARGET_DIR, '/fresh/inputs/source/target/vv-driver');
  assert.equal(environment.CARGO_NET_OFFLINE, 'true');
  assert.equal(environment.DOCKER_CONFIG, '/fresh/docker');
  assert.equal(environment.GIT_CONFIG_GLOBAL, '/dev/null');
  assert.equal(environment.PRISMPM_SDK_INVENTORY, '/opt/prismpm/share/inventory.json');
  for (const key of ['LD_PRELOAD', 'RUSTC_WRAPPER', 'CARGO_ENCODED_RUSTFLAGS', 'GH_TOKEN', 'NODE_OPTIONS', 'BASH_ENV', 'ENV', 'HOME']) assert(!Object.hasOwn(environment, key));
  for (const endpoint of ['tcp://remote:2375', 'unix://relative', 'unix:///tmp/../escape', 'unix:///']) assert.throws(() => executionEnvironment('/fresh', image, endpoint));
  for (const ref of ['sdk:latest', `${image}#other`, '', 'https://registry/image']) assert.throws(() => executionEnvironment('/fresh', ref, 'unix:///var/run/docker.sock'));
});
