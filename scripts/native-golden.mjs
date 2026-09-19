// PR source-development baseline collection. Never SDK or release acceptance.
import assert from 'node:assert/strict';
import { createHash, randomBytes } from 'node:crypto';
import { constants, closeSync, fstatSync, lstatSync, mkdirSync, mkdtempSync, openSync,
  readdirSync, readSync, realpathSync, rmSync, statfsSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { acquireImageMetadata, execute, inspectLoadedImage } from './sdk-vv-check.mjs';
import { verifyTap } from './browser-api-sdk-check.mjs';

export const canonical = value => JSON.stringify(value, (_, item) => item && typeof item === 'object' && !Array.isArray(item)
  ? Object.fromEntries(Object.keys(item).sort().map(key => [key, item[key]])) : item);
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const PROFILE = 'tests/golden/native/linux-arm64-ubuntu-24.04';
const RECORDS = ['golden-manifest.json', 'verified/lexlean-attestation.json', 'verified/manifest.json'];
const REASON = 'Review original Ubuntu 24.04 ARM64 source-development records against the committed shared baseline.';

function regular(path, limit = 128 * 1024 ** 2) {
  assert.equal(realpathSync(dirname(path)), dirname(path), 'input ancestor is aliased');
  assert(lstatSync(path).isFile(), 'regular non-aliased input required');
  const fd = openSync(path, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
  try {
    const before = fstatSync(fd, {bigint: true});
    assert(before.isFile() && before.size <= BigInt(limit), 'bounded regular input required');
    const bytes = Buffer.alloc(Number(before.size));
    for (let offset = 0; offset < bytes.length;) {
      const count = readSync(fd, bytes, offset, bytes.length - offset, null);
      assert(count > 0, 'input shortened'); offset += count;
    }
    assert.equal(readSync(fd, Buffer.alloc(1), 0, 1, null), 0, 'input grew');
    for (const after of [fstatSync(fd, {bigint: true}), lstatSync(path, {bigint: true})]) {
      for (const key of ['dev', 'ino', 'size', 'mode', 'mtimeNs', 'ctimeNs']) assert.equal(after[key], before[key]);
    }
    return bytes;
  } finally { closeSync(fd); }
}

function files(root, selected = () => true) {
  assert.equal(realpathSync(root), resolve(root), 'input directory ancestor is aliased');
  assert(lstatSync(root).isDirectory(), 'non-aliased input directory required');
  const result = []; let total = 0, count = 0;
  const walk = (directory, relative, depth) => {
    assert(depth <= 32, 'tree depth exceeded');
    for (const name of readdirSync(directory).sort()) {
      assert(++count <= 20000 && !/[\x00-\x1f\\]/.test(name), 'bounded confined tree required');
      const path = join(directory, name), rel = relative ? `${relative}/${name}` : name, stat = lstatSync(path);
      assert(stat.isDirectory() || stat.isFile(), 'aliased or special tree member');
      if (stat.isDirectory()) walk(path, rel, depth + 1);
      else if (selected(rel)) {
        total += stat.size; assert(total <= 1024 ** 3, 'aggregate evidence bound exceeded');
        result.push({path: rel, bytes: regular(path)});
      }
    }
  };
  walk(root, '', 0); return result;
}

export function readEnvironmentLock(bytes) {
  assert(Buffer.isBuffer(bytes) && bytes.length <= 8192);
  const value = JSON.parse(bytes);
  assert.equal(bytes.toString(), canonical(value) + '\n');
  assert.deepEqual(Object.keys(value).sort(), ['architecture', 'child_digest', 'config_digest', 'reference', 'schema', 'scope', 'source_revision']);
  assert.equal(value.schema, 'prismpm/golden-development-environment/1');
  assert.equal(value.scope, 'source-golden-review-only'); assert.equal(value.architecture, 'arm64');
  assert.match(value.reference, /^[a-z0-9][a-z0-9./:_-]*@sha256:[0-9a-f]{64}$/);
  for (const key of ['child_digest', 'config_digest']) assert.match(value[key], /^sha256:[0-9a-f]{64}$/);
  assert.match(value.source_revision, /^[0-9a-f]{40}$/);
  return value;
}

export function validateSourceBaseline(source) {
  regular(join(source, 'tests/golden/stdlib/golden-manifest.json'));
  const current = files(join(source, 'stdlib/src'), path => path.endsWith('.lex.tex'));
  const reviewed = files(join(source, 'tests/golden/stdlib/source'));
  assert(current.length > 0, 'stdlib source is absent');
  assert.deepEqual(current.map(row => row.path), reviewed.map(row => row.path), 'committed shared golden source path set is stale');
  for (const [index, row] of current.entries()) assert.equal(hash(row.bytes), hash(reviewed[index].bytes), `committed shared golden source bytes are stale: ${row.path}`);
}

export function collectProfile(source) {
  const rows = files(join(source, PROFILE));
  assert.deepEqual(rows.map(row => row.path), RECORDS, 'exact three raw native records required');
  assert.deepEqual(readdirSync(join(source, PROFILE)).sort(), ['golden-manifest.json', 'verified'], 'extra native record directory');
  assert.deepEqual(readdirSync(join(source, PROFILE, 'verified')).sort(), ['lexlean-attestation.json', 'manifest.json'], 'extra native verification directory');
  return rows;
}

export function validateImageSpace(available, compressedBytes) {
  assert(Number.isSafeInteger(available) && available >= 0 && Number.isSafeInteger(compressedBytes) && compressedBytes >= 0);
  assert(available >= 12 * 1024 ** 3 + compressedBytes, '12 GiB reserve plus independently measured image payload required');
}

function success(result, label) {
  assert.equal(result.signal, null, `${label} interrupted`);
  assert.equal(result.status, 0, `${label} failed: ${result.stderr.toString().slice(-4096)}`);
  return result.stdout;
}

// Transport/context injection is a unit seam, never available from CLI or env.
export async function runReview({source, revision, destination}, transport = execute,
  context = {architecture: process.arch, uid: process.getuid(), gid: process.getgid(), environment: process.env}) {
  assert.match(revision, /^[a-f0-9]{40}$/);
  const environment = context.environment;
  assert.equal(context.architecture, 'arm64'); assert.equal(environment.GITHUB_ACTIONS, 'true');
  assert.equal(environment.RUNNER_ENVIRONMENT, 'github-hosted'); assert.equal(environment.RUNNER_OS, 'Linux');
  assert.equal(environment.RUNNER_ARCH, 'ARM64'); assert.equal(environment.GITHUB_EVENT_NAME, 'pull_request');
  for (const name of ['GITHUB_RUN_ID', 'GITHUB_RUN_ATTEMPT']) assert.match(environment[name], /^[1-9][0-9]*$/);
  source = realpathSync(source); destination = resolve(destination);
  assert(!/[\x00-\x1f,]/.test(source), 'source path cannot alter Docker mount options');
  assert(destination !== source && !destination.startsWith(source + '/'), 'evidence must be outside the source');
  assert.equal(realpathSync(dirname(destination)), dirname(destination));
  mkdirSync(destination, {mode: 0o700});
  const runtime = mkdtempSync(join(tmpdir(), 'prismpm-native-golden-'));
  const commandEnvironment = {PATH: environment.PATH, HOME: runtime, DOCKER_CONFIG: runtime,
    DOCKER_HOST: 'unix:///var/run/docker.sock', LANG: 'C.UTF-8', LC_ALL: 'C.UTF-8',
    GIT_CONFIG_GLOBAL: '/dev/null', GIT_CONFIG_NOSYSTEM: '1', GIT_TERMINAL_PROMPT: '0'};
  const {uid, gid} = context;
  assert(Number.isSafeInteger(uid) && uid > 0 && Number.isSafeInteger(gid) && gid >= 0, 'unprivileged host UID required');
  const name = `prismpm-native-golden-${randomBytes(12).toString('hex')}`, owner = 'org.uor.prismpm.golden-review';
  let sequence = 0, created = false, failure, result, interrupted;
  const handlers = ['SIGHUP', 'SIGINT', 'SIGTERM'].map(signal => [signal, () => { interrupted ??= Error(`interrupted by ${signal}`); }]);
  for (const [signal, handler] of handlers) process.on(signal, handler);
  const call = async (command, args, options = {}) => {
    const stem = String(++sequence).padStart(3, '0'); let value;
    try { value = await transport(command, args, {environment: commandEnvironment, timeout: 120000, limit: 16 * 1024 ** 2, ...options}); }
    catch (error) { value = error.result; throw error; }
    finally {
      writeFileSync(join(destination, `${stem}.command.json`), canonical({command, args, status: value?.status ?? null, signal: value?.signal ?? null}), {flag: 'wx'});
      if (value) for (const stream of ['stdout', 'stderr']) writeFileSync(join(destination, `${stem}.${stream}`), value[stream], {flag: 'wx'});
    }
    return value;
  };
  const docker = args => call('docker', args);
  const git = async args => success(await call('git', ['-c', 'core.fsmonitor=false', '-c', 'core.hooksPath=/dev/null', '-c', 'core.quotePath=false', '-C', source, ...args]), 'source Git check').toString();
  const run = async (args, options) => success(await call('docker', args, options), 'development container command');
  const checkRevision = async () => assert.equal((await git(['rev-parse', 'HEAD'])).trim(), revision, 'source revision changed');
  const requireSpace = (compressedBytes = 0) => { const space = statfsSync(destination); validateImageSpace(space.bavail * space.bsize, compressedBytes); };
  try {
    requireSpace(); await checkRevision(); assert.equal(await git(['status', '--porcelain=v1', '--untracked-files=all']), '', 'clean committed source required');
    validateSourceBaseline(source);
    for (const kind of ['build', 'verified']) assert.equal(lstatSync(join(source, '.prism', kind), {throwIfNoEntry: false}), undefined, 'fresh source outputs required');
    const lock = readEnvironmentLock(regular(join(source, 'sdk/golden-development.lock.json'), 8192));
    const chain = await acquireImageMetadata(docker, lock.reference, 'arm64');
    assert.equal(chain.child_digest, lock.child_digest); assert.equal(chain.config_digest, lock.config_digest);
    // Exact compressed layers are measured from the hashed immutable manifest;
    // this is not an estimate of expanded layers or subsequent compiler use.
    requireSpace(chain.compressed_bytes);
    writeFileSync(join(destination, 'environment.json'), canonical({lock, chain}), {flag: 'wx'});
    await run(['pull', '--platform', 'linux/arm64', lock.reference], {timeout: 1200000}); requireSpace();
    const identity = await inspectLoadedImage(docker, chain, 'arm64', lock.source_revision);
    if (interrupted) throw interrupted;
    created = true;
    await run(['create', '--name', name, '--label', `${owner}=${name}`, '--pull=never', '--platform', 'linux/arm64', '--network', 'none',
      '--user', `${uid}:${gid}`, '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--pids-limit', '4096', '--init',
      '--mount', `type=bind,source=${source},target=/workspace`, '--workdir', '/workspace',
      '--env', 'HOME=/tmp/prismpm-golden-home', '--env', 'CARGO_NET_OFFLINE=true', '--env', 'CARGO_TARGET_DIR=/workspace/target/native-golden',
      '--env', 'CARGO_BUILD_JOBS=2', '--env', 'CARGO_PROFILE_DEV_DEBUG=0', '--env', 'CARGO_PROFILE_TEST_DEBUG=0',
      '--env', `PRISMPM_GOLDEN_REASON=${REASON}`, '--entrypoint', '/bin/bash', lock.reference,
      '-ec', 'mkdir -p /tmp/prismpm-golden-home; exec sleep infinity']);
    await run(['start', name]);
    const container = JSON.parse(await run(['inspect', name]));
    assert(Array.isArray(container) && container.length === 1); assert.equal(container[0].Image, identity.id);
    assert.equal(container[0].Config.Labels[owner], name);
    const platform = JSON.parse(await run(['exec', name, 'node', '-e',
      'console.log(JSON.stringify({architecture:process.arch,os:process.platform,release:require("node:fs").readFileSync("/etc/os-release","utf8")}))']));
    assert.equal(platform.architecture, 'arm64'); assert.equal(platform.os, 'linux');
    assert.match(platform.release, /^ID=ubuntu$/m); assert.match(platform.release, /^VERSION_ID="24\.04"$/m);
    let records;
    for (const write of [true, false]) {
      if (interrupted) throw interrupted;
      await run(['exec', name, 'cargo', 'run', '--locked', '--offline', '-p', 'xtask', '--', 'check-golden', ...(write ? ['--write'] : [])], {timeout: 7200000});
      requireSpace();
      const observed = collectProfile(source);
      if (write) records = observed;
      else for (const [index, row] of observed.entries()) assert.equal(hash(row.bytes), hash(records[index].bytes), `repeat changed original native record: ${row.path}`);
    }
    await checkRevision(); validateSourceBaseline(source);
    for (const command of [['diff', '--name-only', 'HEAD'], ['ls-files', '--others', '--exclude-standard']]) {
      const changed = (await git(command)).trim().split('\n').filter(Boolean);
      assert(changed.every(path => RECORDS.some(record => path === `${PROFILE}/${record}`)), 'generation modified unrelated source');
    }
    for (const row of records) {
      const output = join(destination, 'records', row.path); mkdirSync(dirname(output), {recursive: true});
      writeFileSync(output, row.bytes, {flag: 'wx'});
    }
    result = {schema: 'prismpm/native-golden-review/1', scope: 'source-golden-review-only', status: 'review-required',
      source_revision: revision, environment_revision: lock.source_revision, environment_image: lock.reference, identity,
      platform: 'linux-arm64-ubuntu-24.04', native_basis: 'github-hosted-ARM64-runner-selection',
      run_id: environment.GITHUB_RUN_ID, run_attempt: environment.GITHUB_RUN_ATTEMPT,
      records: records.map(({path, bytes}) => ({path, byte_length: bytes.length, sha256: hash(bytes)})),
      unclaimed: ['hardware-attestation', 'reviewed-baseline', 'current-sdk-acceptance', 'sdk-release', 'product-readiness']};
  } catch (error) { failure = error; }
  finally {
    // Stop the owned container before retaining evidence. A timed-out Docker
    // exec client can leave its in-container compiler running and writing.
    if (created) try {
      const container = JSON.parse(await run(['inspect', name]));
      assert(Array.isArray(container) && container.length === 1 && container[0].Config.Labels[owner] === name, 'cleanup ownership differs');
      await run(['rm', '--force', '--volumes', name]);
      created = false;
    } catch (error) { failure ??= error; }
    try {
      for (const kind of created ? [] : ['build', 'verified']) {
        const directory = join(source, '.prism', kind);
        if (lstatSync(directory, {throwIfNoEntry: false})) for (const row of files(directory)) {
          const output = join(destination, 'generated', kind, row.path); mkdirSync(dirname(output), {recursive: true});
          writeFileSync(output, row.bytes, {flag: 'wx'});
        }
      }
    } catch (error) { failure ??= error; }
    for (const [signal, handler] of handlers) process.removeListener(signal, handler);
    rmSync(runtime, {recursive: true});
  }
  failure ??= interrupted;
  if (failure) { writeFileSync(join(destination, 'failure.json'), canonical({scope: 'source-golden-review-only', message: failure.message}), {flag: 'wx'}); throw failure; }
  writeFileSync(join(destination, 'review.json'), canonical(result), {flag: 'wx'});
  return result;
}

export function validateWorkflow(text) {
  assert.match(text, /^  pull_request:$/m); assert(!/workflow_dispatch:|environment:|packages:|id-token:|attestations:|pull_request_target:/.test(text));
  assert.match(text, /^    runs-on: ubuntu-24\.04-arm$/m);
  assert.match(text, /^      contents: read$/m); assert(!/: write\b/.test(text));
  assert.match(text, /^          node-version: 22\.23\.2$/m);
  assert.match(text, /^          ref: \$\{\{ github\.event\.pull_request\.head\.sha \}\}$/m);
  assert.match(text, /^          persist-credentials: false$/m);
  const run = text.match(/      - name: Generate review-only native records\n        env:\n          SOURCE_REVISION: \$\{\{ github\.event\.pull_request\.head\.sha \}\}\n        run: \|\n((?:          .*\n)+)/)?.[1];
  assert.equal(run, '          node scripts/native-golden.mjs tests\n          node scripts/native-golden.mjs run "$SOURCE_REVISION" "$RUNNER_TEMP/native-golden"\n');
  assert.match(text, /^        if: always\(\)$/m); assert.match(text, /^          path: \$\{\{ runner\.temp \}\}\/native-golden\/$/m);
  assert(!/setup-qemu|docker\/login-action|sdk-candidate.mjs|sdk-image-inputs.mjs/.test(text));
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [operation, revision, destination, ...extra] = process.argv.slice(2);
  if (operation === 'tests' && process.argv.length === 3) {
    const environment = {...process.env}; delete environment.NODE_TEST_CONTEXT;
    const result = await execute(process.execPath, ['--test', '--test-reporter=tap', '--test-timeout=120000', join(dirname(process.argv[1]), 'native-golden.test.mjs')], {environment});
    process.stdout.write(result.stdout); process.stderr.write(result.stderr); success(result, 'owning source-review tests');
    assert.equal(verifyTap(result.stdout.toString(), 8), 8);
  } else {
    assert.equal(operation, 'run'); assert(revision && destination && extra.length === 0, 'usage: native-golden.mjs tests | run SOURCE_COMMIT FRESH_OUTPUT');
    await runReview({source: process.cwd(), revision, destination});
  }
}
