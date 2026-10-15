import assert from 'node:assert/strict';
import { execFileSync, spawn } from 'node:child_process';
import { chmodSync, copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { setTimeout as delay } from 'node:timers/promises';
import test from 'node:test';
import { boundedLog, compilerProcess, dockerUsage, keyNumbers, limits, monitor, pressure, redactor } from './ci-observe.mjs';
import {sessionFixture} from './vv-sdk-session-fixture.mjs';

const script = fileURLToPath(new URL('./ci-observe.mjs', import.meta.url));
const repository = dirname(dirname(script));
const require = createRequire('/opt/prismpm/oracles/package.json');
const {load} = require('js-yaml');
const workflow = name => load(readFileSync(join(repository, '.github/workflows', name), 'utf8'));
const readJson = path => JSON.parse(readFileSync(path, 'utf8'));
function temporary(t) {
  const directory = mkdtempSync(join(tmpdir(), 'prismpm-ci-observe-'));
  t.after(() => rmSync(directory, {recursive: true, force: true}));
  return directory;
}
function owner(directory) {
  mkdirSync(directory, {recursive: true});
  writeFileSync(join(directory, 'owner.json'), JSON.stringify({owner: 'ci-observe', version: 1}));
}
function fixture(t) {
  const root = temporary(t), bin = join(root, 'bin');
  mkdirSync(bin);
  const docker = join(bin, 'docker');
  writeFileSync(docker, `#!${process.execPath}\nconst assert=require('node:assert/strict');assert.deepEqual(process.argv.slice(2),['stats','--no-stream','--format','{{.ID}}\\t{{.CPUPerc}}\\t{{.MemUsage}}\\t{{.MemPerc}}\\t{{.PIDs}}\\t{{.BlockIO}}\\t{{.NetIO}}']);\nprocess.stdout.write('abcdef012345\\t2.3%\\t4MiB / 8GiB\\t0.1%\\t3\\t0B / 1kB\\t1MB / 2MiB\\n');\n`);
  chmodSync(docker, 0o700);
  return {root, env: {...process.env, PATH: `${bin}:${process.env.PATH}`}};
}
const invoke = (args, options = {}) => execFileSync(process.execPath, [script, ...args],
  {encoding: 'utf8', timeout: 15_000, stdio: 'pipe', ...options});

test('numeric allowlists reject names, command lines, environment and malformed resource values', () => {
  const secret = 'DO_NOT_RETAIN_this_credential';
  assert.deepEqual(keyNumbers(`MemTotal: 12 kB\nMemAvailable: ${secret}\nTOKEN: ${secret}\n`, ['MemTotal', 'MemAvailable']),
    {MemTotal: 12, MemAvailable: null});
  const resource = pressure(`some avg10=1.23 avg60=0.00 avg300=${secret} total=40 extra=${secret}\ncommand ${secret}`);
  assert.deepEqual(resource.some, {avg10: 1.23, avg60: 0, avg300: null, total: 40});
  assert.ok(!JSON.stringify(resource).includes(secret));
  const valid = 'abcdef012345\t2.3%\t4MiB / 8GiB\t0.1%\t3\t0B / 1kB\t1MB / 2MiB';
  const rows = dockerUsage(`${secret}\t1%\t1B / 2B\t0%\t1\t0B / 0B\t0B / 0B\n${valid}\n${valid.replace('2.3%', secret)}`);
  assert.equal(rows.length, 2);
  assert.deepEqual(rows[0], {id: 'abcdef012345', cpuPercent: 2.3, memoryBytes: 4 * 1024 ** 2,
    memoryLimitBytes: 8 * 1024 ** 3, memoryPercent: 0.1, pids: 3, blockReadBytes: 0,
    blockWriteBytes: 1000, networkReadBytes: 1e6, networkWriteBytes: 2 * 1024 ** 2});
  assert.equal(rows[1].cpuPercent, null);
  assert.ok(!JSON.stringify(rows).includes(secret));
  assert.equal(dockerUsage(Array(300).fill(valid).join('\n')).length, limits.containers);
  const fields = Array(22).fill('0');
  fields[0] = 'S'; fields[1] = '11'; fields[11] = '23'; fields[12] = '5'; fields[21] = '400';
  const stat = (name = 'lean', data = fields) => `12 (${name}) ${data.join(' ')}\n`;
  assert.deepEqual(compilerProcess(stat()), {tool: 'lean', pid: 12, parentPid: 11, state: 'S', userTicks: 23, systemTicks: 5, residentPages: 400});
  for (const name of ['cargo', 'rustc', 'lake', 'xtask', 'timeout']) assert.equal(compilerProcess(stat(name)).tool, name);
  for (const name of [secret, 'lean --password', 'lean) extra']) assert.equal(compilerProcess(stat(name)), null);
  for (const index of [0, 1, 11, 12, 21]) {
    const bad = [...fields]; bad[index] = secret; assert.equal(compilerProcess(stat('lean', bad)), null);
  }
  assert.equal(compilerProcess(stat().replace('12 (', '0 (')), null);
  assert.equal(compilerProcess(stat('lean', fields.slice(0, 21))), null);
  assert.equal(compilerProcess('x'.repeat(4097)), null);
});

test('credential redaction spans every chunk boundary and never echoes environment names', () => {
  const text = 'before-credential-value-after-credential-longer-after-✓';
  const expected = 'before-[REDACTED]-after-[REDACTED]-after-✓';
  const input = Buffer.from(text);
  for (let split = 0; split <= input.length; split++) {
    const output = [], redact = redactor({GITHUB_TOKEN: 'credential-value', PRIVATE_KEY: 'credential-longer', OTHER: 'before'}, bytes => output.push(bytes));
    redact(input.subarray(0, split)); redact(input.subarray(split)); redact(Buffer.alloc(0), true);
    assert.equal(Buffer.concat(output).toString(), expected);
  }
  const output = [], redact = redactor({SECRET: 'abc', TOKEN: 'abcdef'}, bytes => output.push(bytes));
  for (const byte of Buffer.from('abcdefabc')) redact(Buffer.from([byte]));
  redact(Buffer.alloc(0), true);
  assert.equal(Buffer.concat(output).toString(), '[REDACTED][REDACTED]');
  for (const name of ['API_KEY', 'AWS_ACCESS_KEY_ID', 'SIGNING_KEY', 'DATABASE_URL', 'AUTHORIZATION', 'PASSWORD']) {
    const bytes = [], redactKey = redactor({[name]: 'sensitive-value'}, value => bytes.push(value));
    redactKey(Buffer.from('sensitive-value'), true);
    assert.equal(Buffer.concat(bytes).toString(), '[REDACTED]', name);
  }
});

test('retained logs have a fixed prefix and two bounded tail files with explicit truncation', t => {
  const root = temporary(t), log = boundedLog(root, {headBytes: 16, tailBytes: 8});
  const value = Buffer.from(Array.from({length: 200}, (_, n) => String.fromCharCode(65 + n % 26)).join(''));
  for (let at = 0; at < value.length; at += 7) log.write(value.subarray(at, at + 7));
  const result = log.close();
  assert.deepEqual(readFileSync(join(root, 'gate.log')), value.subarray(0, 16));
  assert.equal(result.observedBytes, 200); assert.equal(result.truncated, true);
  const files = readdirSync(root).filter(name => name.endsWith('.log'));
  assert.equal(files.length, 3);
  assert.ok(files.reduce((total, name) => total + statSync(join(root, name)).size, 0) <= 32);
  const last = readFileSync(join(root, `gate.tail-${result.generation % 2}.log`));
  assert.deepEqual(last, value.subarray(value.length - result.tailBytes));
  assert.equal(readJson(join(root, 'log-retention.json')).truncated, true);
  assert.throws(() => boundedLog(root, {headBytes: 0}));
});

test('ordinary gate logs remain complete and are marked untruncated', t => {
  const root = temporary(t), log = boundedLog(root, {headBytes: 16, tailBytes: 8});
  log.write(Buffer.from('all gate output'));
  assert.equal(log.close().truncated, false);
  assert.equal(readFileSync(join(root, 'gate.log'), 'utf8'), 'all gate output');
  assert.ok(!existsSync(join(root, 'gate.tail-0.log')));
});

test('monitor stops at sample, byte and duration bounds and records collection errors without error text', async t => {
  for (const [name, options, expected] of [
    ['samples', {intervalMs: 1, samples: 3, collect: async () => ({used: 7})}, 'sample-limit'],
    ['bytes', {intervalMs: 1, telemetryBytes: 12, collect: async () => ({used: 7})}, 'byte-limit'],
    ['duration', {intervalMs: 10, durationMs: 5, collect: async () => ({used: 7})}, 'duration-limit'],
    ['error', {collect: async () => { throw new Error('DO_NOT_RETAIN_secret'); }}, 'collection-error'],
  ]) {
    const directory = join(temporary(t), name); owner(directory);
    await monitor(directory, options);
    const result = readJson(join(directory, 'monitor-result.json'));
    assert.equal(result.reason, expected);
    assert.ok(result.samples <= 3);
    assert.equal(result.bytes, statSync(join(directory, 'resources.jsonl')).size);
    assert.ok(!JSON.stringify(result).includes('DO_NOT_RETAIN'));
  }
  const directory = temporary(t); owner(directory);
  await assert.rejects(() => monitor(directory, {samples: limits.samples + 1}));
});

test('actual host monitor starts and stops cooperatively without stopping a second observer', t => {
  const {root, env} = fixture(t), first = join(root, 'first'), second = join(root, 'second');
  try {
    invoke(['start', first], {env}); invoke(['start', second], {env});
    invoke(['stop', first], {env});
    assert.equal(readJson(join(first, 'monitor-result.json')).reason, 'stopped');
    assert.ok(!existsSync(join(second, 'stop')));
    assert.ok(!existsSync(join(second, 'monitor-result.json')));
  } finally {
    invoke(['stop', first], {env}); invoke(['stop', second], {env});
  }
  assert.equal(readJson(join(second, 'monitor-result.json')).reason, 'stopped');
});

test('a rejected second invocation cannot stop or overwrite an existing observer', t => {
  const {root, env} = fixture(t), directory = join(root, 'owned');
  invoke(['start', directory], {env});
  try {
    assert.throws(() => invoke(['run', directory, '--', process.execPath, '-e', 'process.exit(0)'], {env}), error => error.status === 125);
    assert.ok(!existsSync(join(directory, 'stop')));
    assert.ok(!existsSync(join(directory, 'monitor-result.json')));
    assert.ok(!existsSync(join(directory, 'gate-result.json')));
  } finally { invoke(['stop', directory], {env}); }
});

test('actual wrapper preserves zero, nonzero, signal and missing-command exits and always stops its observer', t => {
  for (const [label, code, command] of [
    ['pass', 0, [process.execPath, '-e', 'process.stdout.write("complete-output\\n")']],
    ['fail', 37, [process.execPath, '-e', 'process.stderr.write("failure-output\\n");process.exit(37)']],
    ['signal', 143, [process.execPath, '-e', 'process.kill(process.pid,"SIGTERM")']],
    ['missing', 127, ['missing-command-DO_NOT_ECHO', 'DO_NOT_ECHO_argument']],
  ]) {
    const {root, env} = fixture(t), directory = join(root, label);
    let output;
    if (code === 0) output = invoke(['run', directory, '--', ...command], {env});
    else assert.throws(() => invoke(['run', directory, '--', ...command], {env}), error => {
      assert.equal(error.status, code); output = error.stdout; return true;
    });
    assert.ok(!output.includes('DO_NOT_ECHO'));
    assert.equal(readJson(join(directory, 'gate-result.json')).exitCode, code);
    assert.equal(readJson(join(directory, 'monitor-result.json')).reason, 'stopped');
    assert.equal(readFileSync(join(directory, 'gate.log'), 'utf8'), output);
  }
});

test('actual wrapper redacts inherited credentials from retained stdout and stderr', t => {
  const {root, env} = fixture(t), directory = join(root, 'gate');
  const secret = 'test-credential-do-not-retain';
  const output = invoke(['run', directory, '--', process.execPath, '-e',
    'process.stdout.write(process.env.GITHUB_TOKEN.slice(0,7));setTimeout(()=>{process.stdout.write(process.env.GITHUB_TOKEN.slice(7));process.stderr.write(process.env.GITHUB_TOKEN)},30)'],
  {env: {...env, GITHUB_TOKEN: secret}});
  assert.equal(output, '[REDACTED][REDACTED]');
  for (const file of readdirSync(directory)) {
    const content = readFileSync(join(directory, file), 'utf8');
    assert.ok(!content.includes(secret));
    assert.ok(!content.includes('GITHUB_TOKEN'));
  }
});

test('interleaved stderr cannot interrupt redaction of a split stdout credential (and vice versa)', t => {
  const {root, env} = fixture(t), directory = join(root, 'gate');
  const secret = 'synthetic-api-key-do-not-retain';
  const output = invoke(['run', directory, '--', process.execPath, '-e',
    'process.stdout.write(process.env.API_KEY.slice(0,10));setTimeout(()=>{process.stderr.write("stderr-progress\\n"+process.env.API_KEY.slice(0,10));setTimeout(()=>{process.stdout.write(process.env.API_KEY.slice(10)+"stdout-progress\\n");setTimeout(()=>process.stderr.write(process.env.API_KEY.slice(10)),20)},20)},20)'],
  {env: {...env, API_KEY: secret}});
  assert.equal((output.match(/\[REDACTED\]/g) ?? []).length, 2);
  assert.ok(output.includes('stdout-progress')); assert.ok(output.includes('stderr-progress'));
  assert.ok(!output.includes(secret.slice(0, 10))); assert.ok(!output.includes(secret.slice(10)));
  assert.equal(readFileSync(join(directory, 'gate.log'), 'utf8'), output);
});

test('numeric/redacted artifact files remain readable by a different uploader UID even with umask 077', t => {
  const {root, env} = fixture(t), directory = join(root, 'new-parent/gate');
  execFileSync('bash', ['-c', 'umask 077; exec "$@"', 'fixture', process.execPath, script, 'run', directory,
    '--', process.execPath, '-e', 'process.stdout.write("public-diagnostic\\n")'], {env, stdio: 'pipe', timeout: 15_000});
  assert.equal(statSync(join(root, 'new-parent')).mode & 0o777, 0o755);
  assert.equal(statSync(directory).mode & 0o777, 0o755);
  for (const file of readdirSync(directory)) assert.equal(statSync(join(directory, file)).mode & 0o777, 0o644, file);
});

test('cancellation signals the owned gate process group and stops only its observer', async t => {
  const {root, env} = fixture(t), directory = join(root, 'gate');
  const child = spawn(process.execPath, [script, 'run', directory, '--', process.execPath, '-e',
    'process.stdout.write("ready\\n");setInterval(()=>{},1000)'], {env, stdio: ['ignore', 'pipe', 'pipe']});
  const result = new Promise((resolveResult, reject) => {
    child.once('error', reject);
    child.once('close', (code, signal) => resolveResult({code, signal}));
  });
  await new Promise(resolveReady => child.stdout.once('data', resolveReady));
  child.kill('SIGTERM');
  assert.deepEqual(await result, {code: 143, signal: null});
  assert.equal(readJson(join(directory, 'monitor-result.json')).reason, 'stopped');
});

async function waitForFile(path) {
  const deadline = Date.now() + 5000;
  while (!existsSync(path)) {
    assert.ok(Date.now() < deadline, 'owned fixture did not become ready');
    await delay(20);
  }
}

test('normal command completion kills pipe-holding descendants and preserves buffered output and exact exit', async t => {
  for (const expected of [0, 29]) {
    const {root, env} = fixture(t), directory = join(root, 'gate'), pidFile = join(root, 'descendant.pid');
    const descendant = `require('node:fs').writeFileSync(${JSON.stringify(pidFile)},String(process.pid));setInterval(()=>{},1000)`;
    const payload = 'buffered-' + 'x'.repeat(65536) + '\n';
    const command = `const fs=require('node:fs');require('node:child_process').spawn(process.execPath,['-e',${JSON.stringify(descendant)}],{stdio:'inherit'});const ready=setInterval(()=>{if(fs.existsSync(${JSON.stringify(pidFile)})){clearInterval(ready);process.stdout.write(${JSON.stringify(payload)},()=>process.stderr.write('buffered-stderr\\n',()=>process.exit(${expected}))) }},10)`;
    const child = spawn(process.execPath, [script, 'run', directory, '--', process.execPath, '-e', command],
      {env, stdio: ['ignore', 'pipe', 'pipe']});
    const output = []; child.stdout.on('data', chunk => output.push(chunk)); child.stderr.resume();
    const result = new Promise((resolveResult, reject) => {
      child.once('error', reject); child.once('close', (code, signal) => resolveResult({code, signal}));
    });
    let timer;
    const completed = await Promise.race([result, new Promise(resolveTimeout => { timer = setTimeout(() => resolveTimeout(null), 3000); })]);
    clearTimeout(timer);
    if (!completed) { child.kill('SIGTERM'); await result; }
    assert.ok(completed, 'normal completion waited indefinitely for a surviving descendant');
    assert.deepEqual(completed, {code: expected, signal: null});
    const retained = readFileSync(join(directory, 'gate.log'), 'utf8');
    assert.ok(retained.includes(payload)); assert.ok(retained.includes('buffered-stderr\n'));
    assert.equal(retained, Buffer.concat(output).toString());
    assert.equal(readJson(join(directory, 'gate-result.json')).exitCode, expected);
    assert.equal(readJson(join(directory, 'monitor-result.json')).reason, 'stopped');
    const pid = Number(readFileSync(pidFile, 'utf8'));
    const state = existsSync(`/proc/${pid}/stat`) ? readFileSync(`/proc/${pid}/stat`, 'utf8').split(' ')[2] : 'gone';
    assert.ok(state === 'gone' || state === 'Z', state);
  }
});

test('a completed command and its descendant are already stopped before a late cancellation', async t => {
  const {root, env} = fixture(t), directory = join(root, 'gate'), pidFile = join(root, 'descendant.pid');
  const command = `const c=require('node:child_process').spawn(process.execPath,['-e','setInterval(()=>{},1000)'],{stdio:'inherit'});require('node:fs').writeFileSync(${JSON.stringify(pidFile)},String(c.pid));process.exit(0)`;
  const child = spawn(process.execPath, [script, 'run', directory, '--', process.execPath, '-e', command],
    {env, stdio: ['ignore', 'pipe', 'pipe']});
  const result = new Promise((resolveResult, reject) => {
    child.once('error', reject); child.once('close', (code, signal) => resolveResult({code, signal}));
  });
  await waitForFile(pidFile);
  const descendant = Number(readFileSync(pidFile, 'utf8'));
  assert.deepEqual(await result, {code: 0, signal: null});
  assert.equal(child.kill('SIGTERM'), false);
  assert.equal(readJson(join(directory, 'monitor-result.json')).reason, 'stopped');
  // Container PID 1 may briefly retain a zombie, but it must not be executing.
  const state = existsSync(`/proc/${descendant}/stat`) ? readFileSync(`/proc/${descendant}/stat`, 'utf8').split(' ')[2] : 'gone';
  assert.ok(state === 'gone' || state === 'Z', state);
});

test('a closed console pipe fails safely, stops the owned gate and cleans up its observer', async t => {
  const {root, env} = fixture(t), directory = join(root, 'gate'), pidFile = join(root, 'gate.pid');
  const command = `require('node:fs').writeFileSync(${JSON.stringify(pidFile)},String(process.pid));setInterval(()=>process.stdout.write('x'.repeat(65536)),5)`;
  const child = spawn(process.execPath, [script, 'run', directory, '--', process.execPath, '-e', command],
    {env, stdio: ['ignore', 'pipe', 'pipe']});
  const result = new Promise((resolveResult, reject) => {
    child.once('error', reject); child.once('close', (code, signal) => resolveResult({code, signal}));
  });
  await waitForFile(pidFile); child.stdout.destroy(); child.stderr.resume();
  const status = await result;
  assert.notEqual(status.code, 0); assert.equal(status.signal, null);
  const pid = Number(readFileSync(pidFile, 'utf8'));
  assert.throws(() => process.kill(pid, 0), error => error.code === 'ESRCH');
  assert.equal(readJson(join(directory, 'monitor-result.json')).reason, 'stopped');
  assert.equal(readJson(join(directory, 'gate-result.json')).diagnosticsError, true);
});

test('cancellation escalates only the owned gate when it ignores SIGTERM', async t => {
  const {root, env} = fixture(t), directory = join(root, 'gate'), pidFile = join(root, 'gate.pid');
  const command = `process.on('SIGTERM',()=>{});require('node:fs').writeFileSync(${JSON.stringify(pidFile)},String(process.pid));setInterval(()=>{},1000)`;
  const child = spawn(process.execPath, [script, 'run', directory, '--', process.execPath, '-e', command],
    {env, stdio: ['ignore', 'pipe', 'pipe']});
  const result = new Promise((resolveResult, reject) => {
    child.once('error', reject); child.once('close', (code, signal) => resolveResult({code, signal}));
  });
  await waitForFile(pidFile); child.kill('SIGTERM');
  assert.deepEqual(await result, {code: 137, signal: null});
  const pid = Number(readFileSync(pidFile, 'utf8'));
  const state = existsSync(`/proc/${pid}/stat`) ? readFileSync(`/proc/${pid}/stat`, 'utf8').split(' ')[2] : 'gone';
  assert.ok(state === 'gone' || state === 'Z', state);
  assert.equal(readJson(join(directory, 'monitor-result.json')).reason, 'stopped');
});

test('EPIPE cleanup kills a SIGTERM-ignoring descendant instead of mistaking closed local pipes for group exit', async t => {
  const {root, env} = fixture(t), directory = join(root, 'gate'), pidFile = join(root, 'descendant.pid');
  const descendant = `process.on('SIGTERM',()=>{});require('node:fs').writeFileSync(${JSON.stringify(pidFile)},String(process.pid));setInterval(()=>process.stdout.write('x'.repeat(65536)),5)`;
  const command = `require('node:child_process').spawn(process.execPath,['-e',${JSON.stringify(descendant)}],{stdio:'inherit'});setInterval(()=>{},1000)`;
  const child = spawn(process.execPath, [script, 'run', directory, '--', process.execPath, '-e', command],
    {env, stdio: ['ignore', 'pipe', 'pipe']});
  const result = new Promise((resolveResult, reject) => {
    child.once('error', reject); child.once('close', (code, signal) => resolveResult({code, signal}));
  });
  await waitForFile(pidFile); child.stdout.destroy(); child.stderr.resume();
  const status = await result;
  assert.notEqual(status.code, 0); assert.equal(status.signal, null);
  const pid = Number(readFileSync(pidFile, 'utf8'));
  const state = existsSync(`/proc/${pid}/stat`) ? readFileSync(`/proc/${pid}/stat`, 'utf8').split(' ')[2] : 'gone';
  assert.ok(state === 'gone' || state === 'Z', state);
  assert.equal(readJson(join(directory, 'monitor-result.json')).reason, 'stopped');
});

test('cancellation kills a resistant descendant even when it has no inherited output pipes', async t => {
  const {root, env} = fixture(t), directory = join(root, 'gate'), pidFile = join(root, 'descendant.pid');
  const descendant = `process.on('SIGTERM',()=>{});require('node:fs').writeFileSync(${JSON.stringify(pidFile)},String(process.pid));setInterval(()=>{},1000)`;
  const command = `require('node:child_process').spawn(process.execPath,['-e',${JSON.stringify(descendant)}],{stdio:'ignore'});setInterval(()=>{},1000)`;
  const child = spawn(process.execPath, [script, 'run', directory, '--', process.execPath, '-e', command],
    {env, stdio: ['ignore', 'pipe', 'pipe']});
  const result = new Promise((resolveResult, reject) => {
    child.once('error', reject); child.once('close', (code, signal) => resolveResult({code, signal}));
  });
  await waitForFile(pidFile); child.kill('SIGTERM');
  assert.deepEqual(await result, {code: 143, signal: null});
  const pid = Number(readFileSync(pidFile, 'utf8'));
  const state = existsSync(`/proc/${pid}/stat`) ? readFileSync(`/proc/${pid}/stat`, 'utf8').split(' ')[2] : 'gone';
  assert.ok(state === 'gone' || state === 'Z', state);
  assert.equal(readJson(join(directory, 'monitor-result.json')).reason, 'stopped');
});

test('actual normative twice-VV workflow propagates first/second failures and rejects removal of fail-fast', t => {
  const body = workflow('vv.yml').jobs.vv.steps.find(step => step.with?.runCmd).with.runCmd;
  const check = candidate => {
    for (const [first, second] of [[0, 0], [19, 0], [0, 23], [19, 23]]) {
      const {root, env: baseEnv} = fixture(t);
      const env = sessionFixture(root, baseEnv, {readDuringPreparation: true, image: `ghcr.io/uor-foundation/prismpm-sdk-candidate@sha256:${'b'.repeat(64)}`});
      copyFileSync(script, join(root, 'scripts/ci-observe.mjs'));
      const just = join(root, 'bin/just');
      writeFileSync(just, `#!${process.execPath}\nconst fs=require('node:fs');const assert=require('node:assert/strict');assert.deepEqual(process.argv.slice(2),['vv']);const path='count';const count=fs.existsSync(path)?Number(fs.readFileSync(path))+1:1;fs.writeFileSync(path,String(count));process.stdout.write('call:'+count+'\\n');process.exit(count===1?${first}:${second});\n`);
      chmodSync(just, 0o700);
      const expected = first || second;
      if (!expected) assert.equal(execFileSync('bash', ['-c', candidate], {cwd: root, env, encoding: 'utf8', timeout: 20_000}), 'call:1\ncall:2\n');
      else assert.throws(() => execFileSync('bash', ['-c', candidate], {cwd: root, env, encoding: 'utf8', timeout: 20_000}), error => {
        assert.equal(error.status, expected); return true;
      });
      assert.equal(readFileSync(join(root, 'count'), 'utf8'), first ? '1' : '2');
      for (const name of first ? ['vv-first'] : ['vv-first', 'vv-second']) {
        const directory = join(root, 'target/ci-diagnostics', name);
        assert.equal(readJson(join(directory, 'monitor-result.json')).reason, 'stopped');
        assert.equal(readJson(join(directory, 'gate-result.json')).exitCode, name === 'vv-first' ? first : second);
      }
    }
  };
  check(body);
  assert.throws(() => check(body.replace('--with-sdk bash -euo pipefail', '--with-sdk bash')));
});

test('all observed workflows retain their full gates, always stop/upload, and keep pinned actions', () => {
  for (const [name, job, runs] of [['bootstrap.yml', 'native-gate', 1], ['vv.yml', 'vv', 2]]) {
    const steps = workflow(name).jobs[job].steps;
    const start = steps.findIndex(step => step.run?.includes('ci-observe.mjs start'));
    const gate = steps.findIndex(step => step.with?.runCmd);
    const stop = steps.findIndex(step => step.run?.includes('ci-observe.mjs stop'));
    const upload = steps.findIndex(step => step.uses?.startsWith('actions/upload-artifact@'));
    assert.ok(start < gate && gate < stop && stop < upload);
    assert.equal(steps[stop].if, 'always()'); assert.equal(steps[upload].if, 'always()');
    assert.equal(steps[upload].with.path, 'target/ci-diagnostics/');
    assert.equal(steps[upload].with['retention-days'], 7);
    assert.equal((steps[gate].with.runCmd.match(/ -- just vv/g) ?? []).length, runs);
    assert.equal(steps[gate]['continue-on-error'], undefined);
    for (const step of steps.filter(value => value.uses)) assert.match(step.uses, /@[0-9a-f]{40}$/);
  }
  const native = workflow('native-golden.yml').jobs['native-source-review'].steps;
  const start = native.findIndex(step => step.run === 'node scripts/ci-observe.mjs start "$RUNNER_TEMP/native-observer"');
  const gate = native.findIndex(step => step.name === 'Generate review-only native records');
  const stop = native.findIndex(step => step.run === 'node scripts/ci-observe.mjs stop "$RUNNER_TEMP/native-observer"');
  const upload = native.findIndex(step => step.with?.path === '${{ runner.temp }}/native-observer/');
  assert.ok(start >= 0 && start < gate && gate < stop && stop < upload);
  assert.equal(native[stop].if, 'always()'); assert.equal(native[upload].if, 'always()');
  assert.equal(native[upload].with['retention-days'], 7);
  assert.equal(native[gate]['continue-on-error'], undefined);
  for (const step of native.filter(value => value.uses)) assert.match(step.uses, /@[0-9a-f]{40}$/);
  assert.match(readFileSync(join(repository, 'xtask/src/main.rs'), 'utf8'), /"scripts\/ci-observe\.test\.mjs"/);
});

// Source-policy checks do not establish real GitHub scheduling or execution.
test('actual validation workflows preserve complete budgets, scoped cancellation and bounded fanout', () => {
  const prefixes = {
    'bootstrap.yml': 'bootstrap',
    'ci-parallel.yml': 'ci-parallel',
    'honesty.yml': 'honesty',
    'reproducibility.yml': 'reproducibility',
    'vv.yml': 'vv',
  };
  const actual = Object.fromEntries([...Object.keys(prefixes), 'native-golden.yml'].map(name => [name, workflow(name)]));
  const owners = [
    ['Lint & Dependency Policy', 45, 'cargo fmt --check\ncargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings\ncargo deny --frozen --all-features check'],
    ['Model & Audits', 45, 'bash scripts/bootstrap-verify.sh --check-source\ncargo xtask validate'],
    ['Workspace Unit Tests', 180, './scripts/vv.sh unit-tests'],
    ['Conformance / contracts', 120, './scripts/vv.sh conformance contracts'],
    ['Conformance / core', 120, './scripts/vv.sh conformance core'],
    ['Conformance / holo-stdlib', 180, './scripts/vv.sh conformance holo-stdlib'],
    ['Conformance / desktop-browser', 210, './scripts/vv.sh conformance desktop-browser'],
    ['Conformance / system-oci', 180, './scripts/vv.sh conformance system-oci'],
    ['Conformance / verification-schemas', 90, './scripts/vv.sh conformance verification-schemas'],
    ['Fixtures & Reviewed Golden Outputs', 90, 'cargo xtask check-fixtures\ncargo run --locked --offline --manifest-path tests/holo-codec-oracle/Cargo.toml --target-dir target/holo-codec-oracle\ncargo xtask check-golden'],
    ['Lean 4 Proofs & Formal Verification', 75, 'cargo xtask verify-examples\ncargo run --locked --offline --target-dir target/vendor-lexlean --manifest-path vendor/lexlean/Cargo.toml -- --project lexlean.toml fmt --check\ncargo run --locked --offline --target-dir target/vendor-lexlean --manifest-path vendor/lexlean/Cargo.toml -- --project lexlean.toml lock --check'],
    ['Packaging, API & Upstream Conformance', 120, './scripts/vv.sh package-api'],
  ];
  const budgets = {
    'bootstrap.yml': {'native-gate': 360},
    'ci-parallel.yml': {gate: '${{ matrix.timeout }}'},
    'honesty.yml': {honesty: 45},
    'reproducibility.yml': {reproducibility: 90},
    'vv.yml': {vv: 360},
    'native-golden.yml': {'native-source-review': 360},
  };
  const check = candidate => {
    assert.deepEqual(Object.keys(candidate).sort(), Object.keys(actual).sort());
    for (const [name, prefix] of Object.entries(prefixes)) {
      const suffix = ['bootstrap.yml', 'vv.yml'].includes(name)
        ? '${{ github.repository }}-${{ github.ref }}'
        : '${{ github.repository }}-${{ github.event_name }}-${{ github.event.pull_request.number || github.ref }}';
      assert.deepEqual(candidate[name].concurrency, {group: prefix + '-' + suffix, 'cancel-in-progress': true});
      assert.deepEqual(candidate[name].permissions, {contents: 'read'});
      for (const job of Object.values(candidate[name].jobs)) {
        assert.equal(job['continue-on-error'], undefined);
        assert.equal(job.if, undefined);
      }
    }
    for (const [name, expected] of Object.entries(budgets)) {
      assert.deepEqual(Object.keys(candidate[name].jobs), Object.keys(expected));
      for (const [id, bound] of Object.entries(expected)) assert.equal(candidate[name].jobs[id]['timeout-minutes'], bound);
    }
    const gate = candidate['ci-parallel.yml'].jobs.gate;
    assert.equal(gate.name, '${{ matrix.name }}');
    assert.equal(gate['runs-on'], 'ubuntu-24.04');
    assert.deepEqual(Object.keys(gate.strategy), ['fail-fast', 'max-parallel', 'matrix']);
    assert.equal(gate.strategy['fail-fast'], false);
    assert.equal(gate.strategy['max-parallel'], 3);
    assert.deepEqual(Object.keys(gate.strategy.matrix), ['include']);
    assert.deepEqual(gate.strategy.matrix.include.map(row => {
      assert.deepEqual(Object.keys(row), ['name', 'timeout', 'commands']);
      return {...row, commands: row.commands.trim()};
    }), owners.map(([name, timeout, commands]) => ({name, timeout, commands})));
    assert.deepEqual(gate.steps.map(step => step.uses), [
      'actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683',
      'devcontainers/ci@8bf61b26e9c3a98f69cb6ce2f88d24ff59b785c6',
    ]);
    assert.equal(gate.steps[0].with['fetch-depth'], 0);
    assert.equal(gate.steps[1].with.push, 'never');
    assert.equal(gate.steps[1].with.runCmd, 'set -euo pipefail\n${{ matrix.commands }}\n');
  };
  check(actual);
  const reject = mutate => {
    const candidate = structuredClone(actual);
    mutate(candidate);
    assert.throws(() => check(candidate), {name: 'AssertionError'});
  };
  for (const name of Object.keys(prefixes)) {
    reject(value => { value[name].concurrency.group = '${{ github.repository }}'; });
    reject(value => { value[name].concurrency.group += '-${{ github.run_id }}'; });
    reject(value => { value[name].concurrency['cancel-in-progress'] = false; });
    reject(value => { delete value[name].concurrency; });
    reject(value => { Object.values(value[name].jobs)[0]['timeout-minutes'] = 30; });
    reject(value => { Object.values(value[name].jobs)[0]['continue-on-error'] = true; });
    reject(value => { Object.values(value[name].jobs)[0].if = 'false'; });
  }
  reject(value => { value['vv.yml'].jobs.vv['timeout-minutes'] = 180; });
  reject(value => { value['native-golden.yml'].jobs['native-source-review']['timeout-minutes'] = 180; });
  reject(value => { delete value['ci-parallel.yml'].jobs.gate.strategy['max-parallel']; });
  reject(value => { value['ci-parallel.yml'].jobs.gate.strategy['max-parallel'] = 6; });
  reject(value => { value['ci-parallel.yml'].jobs.gate.strategy['fail-fast'] = true; });
  reject(value => { value['ci-parallel.yml'].jobs.gate.strategy.matrix.include.pop(); });
  reject(value => { value['ci-parallel.yml'].jobs.gate.strategy.matrix.include.push(value['ci-parallel.yml'].jobs.gate.strategy.matrix.include[0]); });
  reject(value => { value['ci-parallel.yml'].jobs.gate.strategy.matrix.include[0].commands = 'cargo fmt --check'; });
  reject(value => { value['ci-parallel.yml'].jobs.gate.strategy.matrix.include[3].commands += ' || true'; });
  reject(value => { delete value['ci-parallel.yml'].jobs.gate; });
  reject(value => { value['ci-parallel.yml'].jobs.extra = structuredClone(value['ci-parallel.yml'].jobs.gate); });
  reject(value => { value['ci-parallel.yml'].jobs.gate.steps[1].with.runCmd = '${{ matrix.commands }}'; });
});

test('actual reusable SDK policy keeps sibling calls independent without narrowing its API or execution', () => {
  const actual = workflow('sdk.yml');
  const inputs = {
    'sdk-image': {required: true, type: 'string'},
    command: {required: true, type: 'string'},
    context: {default: '.', type: 'string'},
    reference: {default: '', type: 'string'},
    release: {default: '', type: 'string'},
    target: {default: '', type: 'string'},
    plan: {default: '', type: 'string'},
    backup: {default: '', type: 'string'},
    'restore-target': {default: '', type: 'string'},
    detach: {default: false, type: 'boolean'},
    authorized: {default: false, type: 'boolean'},
    'secret-directory': {default: '.prismpm/secrets', type: 'string'},
    bundle: {default: '', type: 'string'},
    'trusted-root': {default: '', type: 'string'},
    policy: {default: '', type: 'string'},
    'promotion-to': {default: '', type: 'string'},
    environment: {default: '', type: 'string'},
  };
  const invocationKey = {
    description: 'Optional stable lowercase ASCII caller job/project key (1-64 letters, digits, dots, underscores or hyphens), unique within its workflow; read-only supersession only',
    default: '', type: 'string',
  };
  const outputNames = ['release-digest', 'plan-digest', 'evidence-digest', 'policy-path', 'trusted-root-path', 'signature-referrer'];
  const action = 'UOR-Foundation/PrismPM/action@f785ee46d794886c29b7efe80c2b261e48733a15';
  const forwarding = Object.fromEntries(Object.keys(inputs).map(name => [name, '${{ inputs.' + name + ' }}']));
  const readOnly = 'contains(fromJSON(\'["check", "build", "verify", "test"]\'), inputs.command)';
  const optedIn = readOnly + " && inputs.invocation-key != ''";
  const check = candidate => {
    assert.deepEqual(Object.keys(candidate), ['name', 'on', 'permissions', 'jobs']);
    assert.deepEqual(Object.keys(candidate.on), ['workflow_call']);
    assert.deepEqual(Object.keys(candidate.on.workflow_call), ['inputs', 'outputs']);
    assert.deepEqual(candidate.on.workflow_call.inputs, {...inputs, 'invocation-key': invocationKey});
    assert.deepEqual(candidate.on.workflow_call.outputs, Object.fromEntries(outputNames.map(name => [name, {
      value: '${{ jobs.invoke-readonly.outputs.' + name + ' || jobs.invoke.outputs.' + name + ' }}',
    }])));
    assert.deepEqual(candidate.permissions, {});
    assert.deepEqual(Object.keys(candidate.jobs), ['validate-invocation-key', 'invoke-readonly', 'invoke']);
    const validation = candidate.jobs['validate-invocation-key'];
    const readonly = candidate.jobs['invoke-readonly'];
    const job = candidate.jobs.invoke;
    assert.deepEqual(Object.keys(validation).sort(), ['if', 'runs-on', 'timeout-minutes', 'permissions', 'outputs', 'steps'].sort());
    assert.equal(validation.if, optedIn);
    assert.equal(validation['runs-on'], 'ubuntu-24.04');
    assert.equal(validation['timeout-minutes'], 5);
    assert.deepEqual(validation.permissions, {});
    assert.deepEqual(validation.outputs, {key: '${{ steps.validate.outputs.key }}'});
    assert.deepEqual(validation.steps, [{
      id: 'validate',
      env: {LC_ALL: 'C', INVOCATION_KEY: '${{ inputs.invocation-key }}'},
      run: 'set -euo pipefail\n[[ "$INVOCATION_KEY" =~ ^[a-z0-9][a-z0-9._-]{0,63}$ ]]\nprintf \'key=%s\\n\' "$INVOCATION_KEY" >> "$GITHUB_OUTPUT"\n',
    }]);
    assert.deepEqual(Object.keys(job).sort(), ['runs-on', 'if', 'timeout-minutes', 'permissions', 'outputs', 'steps'].sort());
    assert.equal(job.if, '${{ !(' + optedIn + ') }}');
    assert.equal(job['runs-on'], 'ubuntu-24.04');
    assert.equal(job['timeout-minutes'], 360);
    assert.deepEqual(job.permissions, {contents: 'read'});
    const outputs = Object.fromEntries(outputNames.map(name => [name, '${{ steps.prism.outputs.' + name + ' }}']));
    assert.deepEqual(job.outputs, outputs);
    const steps = [
      {uses: 'actions/checkout@11d5960a326750d5838078e36cf38b85af677262', with: {'fetch-depth': 0}},
      {
        name: 'Fetch locked SDK inputs', if: readOnly, uses: action,
        with: {'sdk-image': forwarding['sdk-image'], command: 'fetch', context: forwarding.context},
      },
      {id: 'prism', uses: action, with: forwarding},
    ];
    assert.deepEqual(job.steps, steps);
    assert.deepEqual(Object.keys(readonly).sort(), ['needs', 'if', 'runs-on', 'timeout-minutes', 'concurrency', 'permissions', 'outputs', 'steps'].sort());
    assert.equal(readonly.needs, 'validate-invocation-key');
    assert.equal(readonly.if, "needs.validate-invocation-key.result == 'success'");
    assert.equal(readonly['runs-on'], 'ubuntu-24.04');
    assert.equal(readonly['timeout-minutes'], 360);
    assert.deepEqual(readonly.permissions, job.permissions);
    assert.deepEqual(readonly.outputs, outputs);
    assert.deepEqual(readonly.concurrency, {
      group: 'prismpm-sdk-readonly-${{ github.repository }}-${{ github.workflow_ref }}-${{ github.ref }}-${{ needs.validate-invocation-key.outputs.key }}',
      'cancel-in-progress': true,
    });
    const fetch = {...steps[1]};
    delete fetch.if;
    assert.deepEqual(readonly.steps, [steps[0], fetch, steps[2]]);
  };
  check(actual);
  const reject = mutate => {
    const candidate = structuredClone(actual);
    mutate(candidate);
    assert.throws(() => check(candidate), {name: 'AssertionError'});
  };
  const unsafeGroup = {group: 'prismpm-sdk-invoke-${{ github.workflow }}-${{ github.ref }}', 'cancel-in-progress': false, queue: 'max'};
  reject(value => { value.jobs.invoke.concurrency = unsafeGroup; });
  reject(value => { value.jobs.invoke.concurrency = {...unsafeGroup, 'cancel-in-progress': true}; });
  reject(value => { value.jobs.invoke.concurrency = {...unsafeGroup, queue: 'single'}; });
  reject(value => { value.jobs.invoke.concurrency = {group: '${{ github.workflow }}-${{ github.ref }}'}; });
  reject(value => { value.jobs.invoke.concurrency = {group: 'prismpm-sdk-invoke-${{ github.workflow }}'}; });
  reject(value => { value.jobs.invoke.concurrency = {group: 'prismpm-sdk-invoke-${{ github.ref }}'}; });
  reject(value => { value.concurrency = unsafeGroup; });
  reject(value => { value.jobs.invoke['timeout-minutes'] = 60; });
  reject(value => { value.jobs.invoke['continue-on-error'] = true; });
  reject(value => { value.jobs.invoke.if = 'false'; });
  reject(value => { value.jobs.invoke.needs = 'validate-invocation-key'; });
  reject(value => { value.jobs.extra = structuredClone(value.jobs.invoke); });
  reject(value => { value.on.workflow_call.inputs['queue-key'] = {type: 'string', default: ''}; });
  reject(value => { delete value.on.workflow_call.inputs.environment; });
  reject(value => { value.on.workflow_call.inputs.authorized.default = true; });
  reject(value => { delete value.on.workflow_call.outputs['evidence-digest']; });
  reject(value => { delete value.jobs.invoke.outputs['evidence-digest']; });
  reject(value => { delete value.jobs.invoke.steps[2].with.release; });
  reject(value => { value.jobs.invoke.steps[2].with.command = 'build'; });
  reject(value => { value.jobs.invoke.steps[2].uses = 'UOR-Foundation/PrismPM/action@main'; });
  reject(value => { value.jobs.invoke.steps[1].if = 'false'; });
  reject(value => { value.jobs.invoke.steps[1].with.command = 'build'; });
  reject(value => { value.jobs.invoke.steps.pop(); });
  reject(value => { value.jobs['invoke-readonly'].if = 'always()'; });
  reject(value => { delete value.jobs['invoke-readonly'].needs; });
  reject(value => { value.jobs['validate-invocation-key'].concurrency = unsafeGroup; });
  reject(value => { value.jobs['invoke-readonly'].concurrency.group = unsafeGroup.group; });
  reject(value => { value.jobs['invoke-readonly'].steps[2].with.command = 'build'; });
  reject(value => { value.jobs['validate-invocation-key'].steps[0].run = 'true\n'; });
});
