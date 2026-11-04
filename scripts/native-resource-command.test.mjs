import assert from 'node:assert/strict';
import {mkdirSync, mkdtempSync, readFileSync, rmSync, truncateSync, writeFileSync} from 'node:fs';
import {dirname, join} from 'node:path';
import {tmpdir} from 'node:os';
import test from 'node:test';
import {diagnosticPushPrefix, diagnosticRef, runDiagnosticCommand} from './native-resource-diagnostic.mjs';

const revision = 'a'.repeat(40);
const environment = {GITHUB_SHA: revision, GITHUB_WORKFLOW_SHA: revision, GITHUB_REPOSITORY: 'UOR-Foundation/PrismPM',
  GITHUB_REF: diagnosticRef, GITHUB_EVENT_NAME: 'workflow_dispatch', GITHUB_RUN_ID: '123', GITHUB_RUN_ATTEMPT: '1'};
const proven = (status = 0) => ({status, signal: null, stdout: Buffer.alloc(0), stderr: Buffer.alloc(0),
  timedOut: false, overflow: false, aborted: false, orphaned: false,
  retirement: {descendants_absent: true, uncertainty: null, supervisor_status: 0, supervisor_signal: null}});
function fixture(t, stream = false) {
  const root = mkdtempSync(join(tmpdir(), 'native-command-unit-')); t.after(() => rmSync(root, {recursive: true, force: true}));
  const control = join(root, '.control'); mkdirSync(control);
  const request = {schema: 'prismpm/native-resource-command/1', source_revision: revision, run_id: '123', run_attempt: '1',
    command_id: 'cmd-1', deadline_ms: Date.now() + 60000, maximum_ms: 30000,
    log_path: join(root, 'commands.log'), capture_path: join(control, 'cmd-1.stdout'),
    result_path: join(control, 'cmd-1.result.json'), cancel_file: join(control, 'cmd-1.cancel.json'),
    stream_output: stream, argv: ['recording-command', 'argument with spaces']};
  const path = join(control, 'cmd-1.request.json');
  const put = () => writeFileSync(path, JSON.stringify(request)); put(); return {root, request, path, put};
}
test('controller uses original owner, complete logging and proven ordinary exit status', async t => {
  const f = fixture(t); let called = 0;
  const r = await runDiagnosticCommand(f.path, async (command, args, options) => {
    called++; assert.equal(command, 'recording-command'); assert.deepEqual(args, ['argument with spaces']);
    assert.equal(options.profile, 'qualification'); assert.equal(options.aggregate, true); assert.equal(options.interruptions, true);
    assert(options.timeout <= 25000 && options.timeout > 24000); assert(options.signal instanceof AbortSignal);
    options.onOutput('stdout', Buffer.from([0, 255, 10])); options.onOutput('stderr', Buffer.from('original stderr'));
    return proven(1);
  }, environment);
  assert.equal(called, 1); assert.equal(r.status, 1); assert.equal(r.reason, null);
  assert.deepEqual(readFileSync(f.request.capture_path), Buffer.from([0, 255, 10]));
  assert(readFileSync(f.request.log_path).includes(Buffer.from('original stderr')));
  assert.deepEqual(JSON.parse(readFileSync(f.request.result_path)), r);
});
test('push command owner requires the same original event proof before execution', async t => {
  for (const invalid of [false, true]) {
    const f = fixture(t), path = join(f.root, 'event.json');
    writeFileSync(path, JSON.stringify({ref: diagnosticPushPrefix + revision, after: revision,
      head_commit: {id: invalid ? 'b'.repeat(40) : revision}, before: '0'.repeat(40),
      created: true, deleted: false, forced: false, repository: {full_name: 'UOR-Foundation/PrismPM', fork: false}}));
    const env = {...environment, GITHUB_EVENT_NAME: 'push', GITHUB_REF: diagnosticPushPrefix + revision,
      GITHUB_EVENT_PATH: path}; let called = 0;
    const run = () => runDiagnosticCommand(f.path, async () => {called++; return proven();}, env);
    if (invalid) {await assert.rejects(run()); assert.equal(called, 0);}
    else {assert.equal((await run()).status, 0); assert.equal(called, 1);}
  }
});
test('leader success cannot waive timeout, abortion, overflow, orphan or missing retirement', async t => {
  for (const mutate of [r => r.timedOut = true, r => r.aborted = true, r => r.overflow = true,
    r => r.orphaned = true, r => r.retirement.descendants_absent = false,
    r => r.retirement.uncertainty = 'lost owner', r => r.signal = 'SIGTERM']) {
    const f = fixture(t), response = proven(); mutate(response);
    const r = await runDiagnosticCommand(f.path, async () => response, environment);
    assert.equal(r.status, 125); assert(r.reason);
  }
});
test('matching cooperative cancellation reaches the original private owner channel', async t => {
  const f = fixture(t);
  const r = await runDiagnosticCommand(f.path, async (_command, _args, options) => {
    writeFileSync(f.request.cancel_file, JSON.stringify({schema: 'prismpm/native-resource-cancel/1',
      source_revision: revision, run_id: '123', run_attempt: '1', command_id: 'cmd-1', reason: 'resource-monitor-unavailable'}));
    await new Promise(resolve => options.signal.addEventListener('abort', resolve, {once: true}));
    return {...proven(), aborted: true};
  }, environment);
  assert.equal(r.status, 125); assert.equal(r.reason, 'resource-monitor-unavailable'); assert.equal(r.aborted, true);
});
test('malformed cooperative cancellation fails rather than being ignored', async t => {
  const f = fixture(t);
  const r = await runDiagnosticCommand(f.path, async (_command, _args, options) => {
    writeFileSync(f.request.cancel_file, '{malformed');
    await new Promise(resolve => options.signal.addEventListener('abort', resolve, {once: true}));
    return {...proven(), aborted: true};
  }, environment);
  assert.equal(r.status, 125); assert.equal(r.aborted, true);
});
test('a cancellation created at command completion still refuses success', async t => {
  const f = fixture(t);
  const r = await runDiagnosticCommand(f.path, async () => {
    writeFileSync(f.request.cancel_file, JSON.stringify({schema: 'prismpm/native-resource-cancel/1',
      source_revision: revision, run_id: '123', run_attempt: '1', command_id: 'cmd-1', reason: 'interrupted'}));
    return proven();
  }, environment);
  assert.equal(r.status, 125); assert.equal(r.reason, 'interrupted');
});
test('readback stdout is strict, while streaming captures are labelled and fully logged', async t => {
  for (const streaming of [false, true]) {
    const f = fixture(t, streaming), bytes = Buffer.alloc(1024 * 1024 + 1, 97);
    const r = await runDiagnosticCommand(f.path, async (_c, _a, options) => {options.onOutput('stdout', bytes); return proven();}, environment);
    assert.equal(r.status, streaming ? 0 : 125); assert.equal(r.stdout_truncated, true);
    assert.equal(readFileSync(f.request.capture_path).length, 1024 * 1024);
    assert(readFileSync(f.request.log_path).includes(bytes));
  }
});
test('substituted command capture or retained log cannot report success', async t => {
  for (const field of ['capture_path', 'log_path']) {
    const f = fixture(t);
    const r = await runDiagnosticCommand(f.path, async (_c, _a, options) => {
      options.onOutput('stdout', Buffer.from('actual original stdout'));
      writeFileSync(f.request[field], 'substituted stdout'); return proven();
    }, environment);
    assert.equal(r.status, 125); assert(r.reason);
  }
});
test('foreign requests, expired clocks and reused output paths never execute', async t => {
  for (const mutate of [r => r.source_revision = 'b'.repeat(40), r => r.run_attempt = '2',
    r => r.deadline_ms = Date.now() + 4000, r => r.result_path = join(dirname(r.result_path), '../elsewhere')]) {
    const f = fixture(t); mutate(f.request); f.put(); let called = 0;
    await assert.rejects(runDiagnosticCommand(f.path, async () => {called++; return proven();}, environment)); assert.equal(called, 0);
  }
  const f = fixture(t); writeFileSync(f.request.capture_path, 'retained');
  await assert.rejects(runDiagnosticCommand(f.path, async () => assert.fail('must not execute'), environment));
  assert.equal(readFileSync(f.request.capture_path, 'utf8'), 'retained');
});
test('bounded shared log admission refuses a full previous log before launch', async t => {
  const f = fixture(t); writeFileSync(f.request.log_path, ''); truncateSync(f.request.log_path, 256 * 1024 ** 2);
  await assert.rejects(runDiagnosticCommand(f.path, async () => assert.fail('must not execute'), environment));
});
