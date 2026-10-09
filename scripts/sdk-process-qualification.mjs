// Complete real native process-owner units; never installed-SDK acceptance.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {readFileSync, writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {execute} from './sdk-vv-check.mjs';
import {captureQualificationFiles} from './sdk-registry-qualification.mjs';
import {verifyTap, verifyFileCompletions} from './browser-api-sdk-check.mjs';

const output = '/evidence', source = process.env.SOURCE_REVISION;
assert.match(source, /^[a-f0-9]{40}$/);
assert.equal(process.version, 'v22.23.2');
assert.equal(process.env.GITHUB_ACTIONS, 'true');
assert.equal(process.env.RUNNER_ENVIRONMENT, 'github-hosted');
assert.equal(process.env.RUNNER_OS, 'Linux'); assert.equal(process.platform, 'linux');
assert(['amd64', 'arm64'].includes(process.env.QUALIFICATION_ARCH));
assert.equal(process.arch, process.env.QUALIFICATION_ARCH === 'amd64' ? 'x64' : 'arm64');
assert.equal(process.env.RUNNER_ARCH, process.arch === 'x64' ? 'X64' : 'ARM64');
const git = args => execFileSync('git', args, {encoding: 'utf8', maxBuffer: 4 * 1024 ** 2});
assert.equal(git(['rev-parse', 'HEAD']).trim(), source);
assert.equal(git(['status', '--porcelain']), '');
const paths = git(['ls-files', '-z']).split('\0').filter(Boolean);
const inputs = captureQualificationFiles(paths);
const write = (name, value) => writeFileSync(output + '/' + name, JSON.stringify(value) + '\n', {flag: 'wx'});
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
write('inputs.json', {source, files: inputs});
const args = ['--test', '--test-concurrency=1', '--test-timeout=120000',
  '--test-reporter=./scripts/owning-node-reporter.mjs', 'scripts/sdk-vv-check.test.mjs'];
write('command.json', {source, program: process.execPath, args, timeout_ms: 150000,
  architecture: process.arch, scope: 'complete original17 process-owner units only'});
let result, failure, sourceUnchanged = false;
try {
  try { result = await execute(process.execPath, args, {timeout: 150000, limit: 16 * 1024 ** 2}); }
  catch (error) { failure = error; result = error.result; }
  assert(result, 'original process result must be retained');
  writeFileSync(output + '/owner.log', result.stdout, {flag: 'wx'});
  writeFileSync(output + '/owner.stderr', result.stderr, {flag: 'wx'});
  assert.deepEqual(captureQualificationFiles(paths), inputs);
  assert.equal(git(['rev-parse', 'HEAD']).trim(), source); assert.equal(git(['status', '--porcelain']), '');
  sourceUnchanged = true;
  if (failure) throw failure;
  assert.equal(result.status, 0); assert.equal(result.signal, null);
  assert.equal(verifyTap(result.stdout.toString(), 17), 17);
  verifyFileCompletions(result.stdout.toString(), [resolve('scripts/sdk-vv-check.test.mjs')], 17);
} catch (error) { failure = error; }
finally {
  const memory = Object.fromEntries(['memory.current', 'memory.peak', 'memory.events',
    'memory.swap.current', 'memory.swap.peak', 'memory.swap.events']
    .map(key => [key, readFileSync('/sys/fs/cgroup/' + key, 'utf8')]));
  write('completed.json', {source, architecture: process.arch, status: result?.status ?? null,
    signal: result?.signal ?? null, passed: !failure, source_unchanged: sourceUnchanged, memory,
    ...(result ? {stdout_sha256: hash(result.stdout), stderr_sha256: hash(result.stderr),
      ...(result.retirement ? {process_retirement: result.retirement} : {})} : {}),
    failure: failure ? String(failure) : null,
    scope: 'native whole process-owner units only; no SDK/fullVV/escaped adoption or release acceptance'});
}
if (failure) throw failure;
