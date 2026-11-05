import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import {parts} from './vv-assemble.mjs';
import { bounds, checkpoint, checkpointPart, runtimeEnvironment, sanitizeSample, uploadEnvironment, uploadOnce } from './ci-disk-checkpoint.mjs';

function temporary(t) {
  const path = mkdtempSync(join(tmpdir(), 'ci-checkpoints-'));
  t.after(() => rmSync(path, {recursive: true, force: true}));
  return path;
}
function requireRemoveFifo(path) { rmSync(path, {force: true}); }
const env = {PATH: process.env.PATH, GITHUB_RUN_ID: '123', GITHUB_RUN_ATTEMPT: '1', ACTIONS_RUNTIME_TOKEN: 'secret'};
function fake(root, source) { const path = join(root, 'action.cjs'); writeFileSync(path, source); return path; }

test('uploader receives only explicit context and closed inputs', () => {
  const actual = uploadEnvironment({...env, PASSWORD: 'not-for-child', CUSTOM_SECRET: 'not-for-child'}, '/tmp/snapshot', '/tmp/output', 'vv-disk-123-1-0');
  assert.equal(actual.ACTIONS_RUNTIME_TOKEN, 'secret');
  assert.equal(actual.INPUT_OVERWRITE, 'false');
  assert.equal(actual['INPUT_IF-NO-FILES-FOUND'], 'error');
  assert.ok(!('PASSWORD' in actual)); assert.ok(!('CUSTOM_SECRET' in actual));
});

test('upload handles success, error, timeout, false success and oversized output without revealing child output', async t => {
  const root = temporary(t), path = join(root, 'snapshot'), output = join(root, 'output'); writeFileSync(path, '{}');
  const run = async source => {
    writeFileSync(output, '');
    return uploadOnce({entry: fake(root, source), path, output, name: 'vv-disk-123-1-0', env, timeoutMs: 2000});
  };
  assert.equal((await run("require('fs').writeFileSync(process.env.GITHUB_OUTPUT, 'artifact-id<<DELIM\\n1234\\nDELIM\\n')")).status, 'uploaded');
  assert.equal((await run("process.stderr.write('secret'); process.exitCode=1")).status, 'failed');
  assert.equal((await run("process.exitCode=0")).status, 'unconfirmed');
  assert.equal((await run("require('fs').writeFileSync(process.env.GITHUB_OUTPUT, 'X'.repeat(5000))")).status, 'unconfirmed');
  assert.equal((await run("require('fs').unlinkSync(process.env.GITHUB_OUTPUT); require('child_process').execFileSync('/usr/bin/mkfifo', [process.env.GITHUB_OUTPUT])")).status, 'unconfirmed');
  requireRemoveFifo(output);
  writeFileSync(output, '');
  const result = await uploadOnce({entry: fake(root, 'setInterval(() => {}, 1000)'), path, output, name: 'x', env, timeoutMs: 100});
  assert.equal(result.status, 'timeout');
  const stopFile = join(root, 'stop');
  setTimeout(() => writeFileSync(stopFile, ''), 100);
  const cancelled = await uploadOnce({entry: fake(root, 'setInterval(() => {}, 1000)'), path, output, name: 'x', env, timeoutMs: 2000, stopFile});
  assert.equal(cancelled.status, 'cancelled');
  await assert.rejects(() => uploadOnce({entry: '', path, output, name: 'x', env, timeoutMs: bounds.uploadMs + 1}));
});

test('checkpoints bound attempts, expose previous failure, remove staging and do not retry uncertain uploads', async t => {
  const root = temporary(t), snapshots = [], names = [], offsets = [];
  await checkpoint(root, 'unused', {env, attempts: 3, intervalMs: 1,
    collect: async ({componentOffset}) => { offsets.push(componentOffset); return {status: 'ok', bytes: 3}; },
    upload: async ({path, name}) => {
      snapshots.push(JSON.parse(readFileSync(path, 'utf8'))); names.push(name);
      return {status: snapshots.length === 1 ? 'timeout' : 'uploaded'};
    }});
  assert.deepEqual(names, ['vv-disk-123-1-0', 'vv-disk-123-1-1', 'vv-disk-123-1-2']);
  assert.deepEqual(offsets, [0, 1, 2]);
  assert(!Object.hasOwn(snapshots[0], 'part'), 'default snapshot schema stays unchanged');
  assert.equal(snapshots[0].previousUpload, 'none'); assert.equal(snapshots[1].previousUpload, 'timeout');
  assert.equal(snapshots[2].previousUpload, 'uploaded');
  assert.ok(!readdirSync(root).some(name => /^snapshot-\d+\.json$|^output-\d+$|^latest-\d+$/.test(name)));
  assert.equal(JSON.parse(readFileSync(join(root, 'latest.json'))).sequence, 2);
  assert.deepEqual(JSON.parse(readFileSync(join(root, 'status.json'))), {attempts: 3, lastUpload: 'uploaded'});
  await assert.rejects(() => checkpoint(root, 'unused', {env, attempts: bounds.attempts + 1}));
});

test('hostile collector output is filtered to bounded labels and numbers', async t => {
  const root = temporary(t); let calls = 0;
  const hostile = {components: {'workspace-target': {status: 'secret', allocatedBytes: 1, entriesVisited: 'secret',
    leakedPath: '/secret/path'}, secret: {status: 'complete'}}};
  const filtered = sanitizeSample(hostile);
  assert.equal(filtered.components['workspace-target'].status, 'invalid-output');
  assert.equal(filtered.components['workspace-target'].allocatedBytes, 1);
  assert.equal(filtered.components['workspace-target'].entriesVisited, null);
  assert.ok(!JSON.stringify(filtered).includes('secret'));
  await checkpoint(root, 'unused', {env, attempts: 2, intervalMs: 1,
    collect: async () => ++calls === 1 ? hostile : (() => { throw new Error('secret'); })(),
    upload: async ({path}) => { const data = readFileSync(path, 'utf8'); assert.ok(data.length < bounds.snapshotBytes);
      assert.ok(!data.includes('secret')); return {status: 'failed'}; }});
  assert.deepEqual(JSON.parse(readFileSync(join(root, 'status.json'))), {attempts: 2, lastUpload: 'failed'});
});

test('all closed source parts namespace checkpoint names, snapshots, status and detached child environment', async t => {
  assert.equal(parts.length, 15); assert.equal(new Set(parts).size, 15);
  const names = [];
  for (const part of parts) {
    const root = temporary(t), selected = {...env, PRISMPM_CI_CHECKPOINT_PART: part};
    const childEnv = runtimeEnvironment({...selected, PASSWORD: 'not-for-child'});
    assert.equal(childEnv.PRISMPM_CI_CHECKPOINT_PART, part); assert(!('PASSWORD' in childEnv));
    await checkpoint(root, 'unused', {env: childEnv, attempts: 1, intervalMs: 1, collect: async () => ({}),
      upload: async ({name, path, env: uploadEnv}) => {
        names.push(name); assert.equal(name, `vv-disk-123-1-0-${part}`);
        assert.equal(JSON.parse(readFileSync(path)).part, part);
        assert.equal(uploadEnv.PRISMPM_CI_CHECKPOINT_PART, part); return {status: 'uploaded'};
      }});
    assert.deepEqual(JSON.parse(readFileSync(join(root, 'status.json'))), {part, attempts: 1, lastUpload: 'uploaded'});
    assert.equal(JSON.parse(readFileSync(join(root, 'latest.json'))).part, part);
  }
  assert.equal(new Set(names).size, 15);
  assert.equal(checkpointPart(env), undefined);
  assert(!Object.hasOwn(runtimeEnvironment(env), 'PRISMPM_CI_CHECKPOINT_PART'));
});

test('empty, unknown, unsafe or foreign part namespaces fail before collection or upload', async t => {
  for (const part of ['', 'unknown', '../validate', 'validate/other', 'unit\nsecret', 'source-vv', '${{ matrix.part }}', 'sdk-amd64', null, 1, undefined]) {
    const root = temporary(t), selected = {...env, PRISMPM_CI_CHECKPOINT_PART: part};
    assert.throws(() => runtimeEnvironment(selected), /invalid checkpoint part/);
    assert.throws(() => uploadEnvironment(selected, 'input', 'output', 'name'), /invalid checkpoint part/);
    await assert.rejects(() => checkpoint(root, 'unused', {env: selected, attempts: 1, intervalMs: 1,
      collect: async () => {assert.fail('invalid namespace collected');},
      upload: async () => {assert.fail('invalid namespace uploaded');}}), /invalid checkpoint part/);
    assert.deepEqual(readdirSync(root), []);
  }
});

test('workflow brackets every original source part with pinned best-effort checkpoints', () => {
  const text = readFileSync(new URL('../.github/workflows/vv.yml', import.meta.url), 'utf8');
  const start = text.indexOf('      - name: Start disk checkpoints');
  const gate = text.indexOf('      - name: Run the complete pinned devcontainer source part');
  const stop = text.indexOf('      - name: Stop disk checkpoints');
  assert.ok(start >= 0 && start < gate && gate < stop);
  assert.match(text.slice(start, gate), /continue-on-error: true/);
  assert.match(text.slice(stop), /if: always\(\)\n        continue-on-error: true/);
  assert.match(text.slice(start, gate), /timeout-minutes: 1/);
  assert.match(text.slice(stop), /timeout-minutes: 1/);
  assert.match(text, /ref: ea165f8d65b6e75b540449e92b4886f43607fa02/);
  const gateText = text.slice(gate, stop);
  assert.match(gateText, /devcontainers\/ci@8bf61b26e9c3a98f69cb6ce2f88d24ff59b785c6/);
  assert.match(gateText, /node scripts\/ci-observe\.mjs run target\/ci-diagnostics\/vv -- node scripts\/release-gate-evidence\.mjs source-part-run/);
  assert.match(text.slice(start, gate), /PRISMPM_CI_CHECKPOINT_PART: \$\{\{ matrix\.part \}\}/);
  assert.match(readFileSync(new URL('./ci-disk-checkpoint.mjs', import.meta.url), 'utf8'), /env: runtimeEnvironment\(process\.env\)/);
  assert.doesNotMatch(gateText, /continue-on-error|\|\| true/);
  assert.match(readFileSync(new URL('../xtask/src/main.rs', import.meta.url), 'utf8'), /"scripts\/ci-disk-checkpoint\.test\.mjs"/);
});

test('staging failure remains diagnostic and never throws into the gate', async t => {
  const root = temporary(t); rmSync(root, {recursive: true});
  await assert.doesNotReject(() => checkpoint(root, 'unused', {env, attempts: 1, intervalMs: 1,
    collect: async () => ({}), upload: async () => { throw new Error('must not be called'); }}));
});
