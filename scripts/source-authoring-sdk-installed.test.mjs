// Actual native installed-image checks, owned by the existing SDK runtime gate.
// No source implementation or compiler crate is mounted into the consumer.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {randomBytes} from 'node:crypto';
import {chmodSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import {decodeProcess, sha256} from './source-authoring-sdk-check.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const harness = readFileSync(new URL('./source-authoring-sdk-check.mjs', import.meta.url));
const evidenceDirectory = join(root, 'target/source-authoring-sdk-evidence',
  'linux-' + ({x64: 'amd64', arm64: 'arm64'}[process.arch]));
function capture(name, result) {
  mkdirSync(evidenceDirectory, {recursive: true});
  writeFileSync(join(evidenceDirectory, name + '.stdout'), result.stdout ?? Buffer.alloc(0));
  writeFileSync(join(evidenceDirectory, name + '.stderr'), result.stderr ?? Buffer.alloc(0));
  writeFileSync(join(evidenceDirectory, name + '.status.json'), JSON.stringify({scope: 'process-capture-only',
    status: result.status, signal: result.signal, error: result.error?.code ?? null}) + '\n');
}
function docker(args, input, timeout = 45_000, record) {
  const result = spawnSync('docker', args, {input, timeout, maxBuffer: 8 * 1024 * 1024});
  if (record) capture(record, result);
  assert.equal(result.error, undefined, 'Docker launch/timeout');
  assert.equal(result.signal, null, 'Docker signal');
  return result;
}
function imageReference() {
  const image = process.env.PRISMPM_TEST_SDK_IMAGE;
  assert.match(image ?? '', /^[a-z0-9./_:-]+@sha256:[0-9a-f]{64}$/);
  return image;
}
function currentRevision() {
  const result = spawnSync('git', ['rev-parse', 'HEAD'], {cwd: root, encoding: 'utf8', timeout: 10_000});
  assert.equal(result.status, 0);
  const revision = result.stdout.trim();
  assert.match(revision, /^[0-9a-f]{40}$/);
  return revision;
}
function container(image, args, input, extra = [], record) {
  const nonce = randomBytes(16).toString('hex'), name = 'source-authoring-' + nonce;
  try {
    return docker(['run', '--rm', '--interactive', '--name', name,
      '--label', 'org.uor.prismpm.source-authoring-owner=' + nonce,
      '--user', '1000:1000', '--network', 'none', '--read-only', '--cap-drop', 'ALL',
      '--security-opt', 'no-new-privileges', '--memory', '512m', '--pids-limit', '64',
      '--tmpfs', '/tmp:rw,noexec,nosuid,nodev,size=16m,mode=1777',
      '--tmpfs', '/opt/prismpm/source:ro,noexec,nosuid,nodev,size=1m,mode=0555',
      '--tmpfs', '/opt/prismpm/share/conformance-root:ro,noexec,nosuid,nodev,size=1m,mode=0555',
      '--workdir', '/tmp', ...extra, ...args.slice(0, 2), image, ...args.slice(2)], input, 300_000, record);
  } finally {
    // A Docker CLI timeout can leave its container alive. Retire only the exact
    // nonce-owned container; never turn an uncertain retirement into a pass.
    const inspection = docker(['container', 'inspect', name]);
    if (inspection.status === 0) {
      const rows = JSON.parse(inspection.stdout);
      assert.equal(rows.length, 1);
      assert.equal(rows[0].Config.Labels['org.uor.prismpm.source-authoring-owner'], nonce);
      assert.match(rows[0].Id, /^[0-9a-f]{64}$/);
      const removed = docker(['container', 'rm', '--force', rows[0].Id]);
      assert.equal(removed.status, 0, removed.stderr.toString());
    } else {
      assert.match(inspection.stderr.toString(), /No such (object|container)/i,
        'container retirement is uncertain');
    }
  }
}

test('installed native source interface executes with compiler/repository sources masked', () => {
  const image = imageReference(), revision = currentRevision(), harnessHash = sha256(harness);
  const result = container(image, ['--entrypoint', '/usr/local/bin/node', '--input-type=module', '-',
    'installed', image, revision, harnessHash], harness, [], 'authoring');
  assert.equal(result.status, 0, result.stderr.toString());
  assert.equal(result.stderr.length, 0);
  const receipt = JSON.parse(result.stdout);
  assert.equal(receipt.schema, 'prismpm/source-authoring-sdk-check/1');
  assert.equal(receipt.status, 'passed');
  assert.equal(receipt.scope, 'source-free-installed-authoring-component');
  assert.equal(receipt.image, image);
  assert.equal(receipt.source_revision, revision);
  assert.equal(receipt.harness_sha256, harnessHash);
  assert.equal(receipt.source_harness_sha256, harnessHash);
  assert.equal(receipt.platform, 'linux/' + ({x64: 'amd64', arm64: 'arm64'}[process.arch]));
  assert.equal(receipt.checks.length, 36, 'complete owning vector set');
  assert.equal(receipt.output_failure.disposition, 'Unknown');
  assert.equal(receipt.output_failure.diagnostic, 'PP5804');
  writeFileSync(join(evidenceDirectory, 'receipt.json'), result.stdout);
  writeFileSync(join(evidenceDirectory, 'receipt.sha256'), sha256(result.stdout) + '\n');
});

test('source entry refuses planted SDK inventory changes before reading a valid request', () => {
  const image = imageReference(), temporary = mkdtempSync(join(tmpdir(), 'source-authoring-planted-'));
  try {
    chmodSync(temporary, 0o755);
    writeFileSync(join(temporary, 'undeclared-source-tool'), '#!/bin/sh\nexit 0\n', {mode: 0o755});
    const input = Buffer.from(JSON.stringify({schema: 'prismpm/source-authoring-request/1',
      source: '\\begin{semanticmodule}\n\\semanticdata{null}\n\\end{semanticmodule}\n'}));
    const args = ['--entrypoint', '/usr/local/bin/prismpm', '--json', 'source', 'read'];
    const planted = container(image, args, input, [
      '--mount', 'type=bind,source=' + temporary + ',target=/planted,readonly',
      '--env', 'PATH=/planted:/usr/local/elan/bin:/usr/local/cargo/bin:/usr/local/bin:/usr/bin:/bin',
    ], 'inventory-path');
    decodeProcess(planted, {status: 4, code: 'PP5401'});
    const changed = container(image, args, input, [
      '--mount', 'type=bind,source=' + join(temporary, 'undeclared-source-tool') + ',target=/usr/local/bin/just,readonly',
    ], 'inventory-binary');
    decodeProcess(changed, {status: 4, code: 'PP5401'});
  } finally { rmSync(temporary, {recursive: true, force: true}); }
});
