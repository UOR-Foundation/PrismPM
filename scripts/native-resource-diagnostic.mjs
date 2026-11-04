// Native resource observations only. No image publication or acceptance.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {constants, closeSync, fstatSync, lstatSync, openSync, readSync, readdirSync, realpathSync, writeFileSync, writeSync} from 'node:fs';
import {basename, dirname, join, resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {describeDiagnosticLayout, platformIndex, verifyReproducibility} from './release-phases.mjs';
import {validateConfig} from './sdk-candidate.mjs';
import {executeOwnedSdkCommand} from './sdk-command-owner.mjs';

export const diagnosticRef = 'refs/heads/fix/521-release-build-sets';
export const diagnosticScope = 'unpublished native resource diagnostic only; not shipped, signed, SDK or release acceptance';
const sha = bytes => 'sha256:' + createHash('sha256').update(bytes).digest('hex');
const oid = value => assert.match(value, /^[a-f0-9]{40}$/);
const positive = value => assert(typeof value === 'string' && /^[1-9][0-9]{0,19}$/.test(value));
const keys = (value, expected) => assert.deepEqual(Object.keys(value).sort(), [...expected].sort());
const canonical = value => JSON.stringify(value) + '\n';
const identity = (a, b) => {for (const key of ['dev', 'ino', 'mode', 'nlink', 'size', 'mtimeNs', 'ctimeNs']) assert.equal(a[key], b[key], 'diagnostic input custody changed');};
function original(path, limit = 32 * 1024 ** 2) {
  path = resolve(path); assert.equal(realpathSync(path), path);
  const before = lstatSync(path, {bigint: true});
  assert(before.isFile() && before.nlink === 1n && before.size <= BigInt(limit));
  const fd = openSync(path, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
  try {
    identity(fstatSync(fd, {bigint: true}), before);
    const bytes = Buffer.alloc(Number(before.size));
    for (let offset = 0; offset < bytes.length;) {const n = readSync(fd, bytes, offset, bytes.length - offset, null); assert(n > 0); offset += n;}
    assert.equal(readSync(fd, Buffer.alloc(1), 0, 1, null), 0);
    identity(fstatSync(fd, {bigint: true}), before); identity(lstatSync(path, {bigint: true}), before);
    assert.equal(realpathSync(path), path); return bytes;
  } finally {closeSync(fd);}
}
const write = (path, value) => writeFileSync(path, canonical(value), {flag: 'wx', mode: 0o444});
function digestOriginal(path, limit, hash = createHash('sha256')) {
  assert.equal(realpathSync(path), path);
  const before = lstatSync(path, {bigint: true});
  assert(before.isFile() && before.nlink === 1n && before.size <= BigInt(limit));
  const fd = openSync(path, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
  let length = 0;
  try {
    identity(fstatSync(fd, {bigint: true}), before); const buffer = Buffer.alloc(65536);
    for (;;) {const n = readSync(fd, buffer, 0, buffer.length, null); if (!n) break;
      length += n; assert(BigInt(length) <= before.size); hash.update(buffer.subarray(0, n));}
    assert.equal(BigInt(length), before.size);
    identity(fstatSync(fd, {bigint: true}), before); identity(lstatSync(path, {bigint: true}), before);
    assert.equal(realpathSync(path), path); return {stat: before, length, hash};
  } finally {closeSync(fd);}
}
function context(revision, arch, environment = process.env) {
  oid(revision); assert(['amd64', 'arm64'].includes(arch));
  assert.equal(environment.GITHUB_REPOSITORY, 'UOR-Foundation/PrismPM');
  assert.equal(environment.GITHUB_SHA, revision); assert.equal(environment.GITHUB_WORKFLOW_SHA, revision);
  assert.equal(environment.GITHUB_REF, diagnosticRef); assert.equal(environment.GITHUB_EVENT_NAME, 'workflow_dispatch');
  positive(environment.GITHUB_RUN_ID); positive(environment.GITHUB_RUN_ATTEMPT);
  return {source_revision: revision, architecture: arch, run_id: environment.GITHUB_RUN_ID, run_attempt: environment.GITHUB_RUN_ATTEMPT};
}

export function diagnosticPolicy(environment, now = Date.now()) {
  const revision = environment.DIAGNOSTIC_SOURCE; oid(revision);
  context(revision, 'amd64', environment);
  assert.equal(environment.DISPATCH_VERSION, '0.4.0');
  assert.equal(environment.REHEARSAL, 'true'); assert.equal(environment.PUBLISH_CRATES, 'false');
  assert(Number.isSafeInteger(now) && now > 0);
  return {schema: 'prismpm/native-resource-plan/1', scope: diagnosticScope, source_revision: revision,
    workflow_revision: revision, run_id: environment.GITHUB_RUN_ID, run_attempt: environment.GITHUB_RUN_ATTEMPT,
    work_end_ms: now + 19800000, whole_end_ms: now + 21600000, maximum_native_runners: 2,
    store_bytes: 40 * 1024 ** 3, backing_reserve_bytes: 12 * 1024 ** 3,
    outside_allowance_bytes: 2 * 1024 ** 3, sampled_inner_stop_bytes: 1024 ** 3,
    cpu_quota: 200000, cpu_period: 100000, memory_cap_bytes: 12 * 1024 ** 3,
    host_memory_reserve_bytes: 2 * 1024 ** 3, pids_limit: 4096, extra_swap_bytes: 0,
    workload: {production: ['runtime', 'cli-archive'], pair: ['runtime', 'runtime']},
    unmeasured: ['registry exporter production fit', 'other four SDK-family product exports', 'standalone runtime Dockerfile', 'complete release acceptance']};
}
function policyDigests() {
  return {workflow_sha256: sha(original(new URL('../.github/workflows/release.yml', import.meta.url).pathname)),
    policy_helper_sha256: sha(original(new URL('./native-resource-diagnostic.mjs', import.meta.url).pathname))};
}
export function admitDiagnosticPlan(plan, revision, workEnd, wholeEnd, environment = process.env) {
  assert(Number.isSafeInteger(workEnd) && Number.isSafeInteger(wholeEnd));
  assert(Date.now() < workEnd && wholeEnd - Date.now() <= 21600000);
  const expected = {...diagnosticPolicy({...environment, DIAGNOSTIC_SOURCE: revision,
    DISPATCH_VERSION: '0.4.0', REHEARSAL: 'true', PUBLISH_CRATES: 'false'}, workEnd - 19800000), ...policyDigests()};
  assert.equal(expected.whole_end_ms, wholeEnd);
  assert.deepEqual(plan, expected, 'exact source-bound dispatch plan required');
  return plan;
}

export function diagnosticProductionPlan(revision, arch, builder, output) {
  oid(revision); assert(['amd64', 'arm64'].includes(arch)); assert.match(builder, /^[a-zA-Z0-9][a-zA-Z0-9_.-]{0,127}$/);
  output = resolve(output);
  return {builder, platform: 'linux/' + arch, builds: [
    {target: 'runtime', args: ['--no-cache', '--provenance=false', '--sbom=false', '--build-arg', 'SOURCE_DATE_EPOCH=0',
      '--label', 'org.opencontainers.image.created=1970-01-01T00:00:00Z',
      '--label', 'org.opencontainers.image.revision=' + revision,
      '--label', 'org.opencontainers.image.source=https://github.com/UOR-Foundation/PrismPM',
      '--label', 'org.opencontainers.image.version=0.4.0',
      '--output', 'type=oci,tar=false,dest=' + join(output, 'runtime.oci') + ',rewrite-timestamp=true,oci-mediatypes=true']},
    {target: 'cli-archive', args: ['--build-arg', 'SDK_VERSION=0.4.0', '--output', 'type=local,dest=' + join(output, 'cli')]},
  ]};
}

export function productionRecord(layout, arch, revision, environment = process.env) {
  const selected = context(revision, arch, environment);
  const {index, manifest, config, descriptor} = describeDiagnosticLayout(layout, arch, revision);
  const value = {schema: 'prismpm/native-resource-production/1', scope: diagnosticScope, ...selected,
    production_accepted: false, complete_declared_layout_readback: true,
    descriptor, index_base64: index.toString('base64'), manifest_base64: manifest.toString('base64'), config_base64: config.toString('base64')};
  assert(Buffer.byteLength(canonical(value)) <= 32 * 1024 ** 2); return value;
}
export function diagnosticArchive(directory, revision, arch, environment = process.env) {
  const selected = context(revision, arch, environment);
  directory = resolve(directory); assert.equal(realpathSync(directory), directory);
  const parent = lstatSync(directory, {bigint: true}); assert(parent.isDirectory());
  const name = `prismpm-0.4.0-${arch === 'amd64' ? 'x86_64' : 'aarch64'}-unknown-linux-gnu.tar.gz`;
  assert.deepEqual(readdirSync(directory).sort(), [name, name + '.sha256'].sort());
  const path = join(directory, name), checksum = original(path + '.sha256', 1024).toString('ascii');
  assert.equal(realpathSync(path), path); const before = lstatSync(path, {bigint: true});
  assert(before.isFile() && before.nlink === 1n && before.size > 0n && before.size <= 1024n ** 3n);
  const fd = openSync(path, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
  const hash = createHash('sha256'); let length = 0;
  try {
    identity(fstatSync(fd, {bigint: true}), before); const buffer = Buffer.alloc(65536);
    for (;;) {const n = readSync(fd, buffer, 0, buffer.length, null); if (!n) break; length += n; assert(BigInt(length) <= before.size); hash.update(buffer.subarray(0, n));}
    assert.equal(BigInt(length), before.size); identity(fstatSync(fd, {bigint: true}), before); identity(lstatSync(path, {bigint: true}), before);
  } finally {closeSync(fd);}
  const digest = hash.digest('hex'); assert.equal(checksum, `${digest}  ${name}\n`);
  identity(lstatSync(directory, {bigint: true}), parent);
  return {schema: 'prismpm/native-resource-archive/1', scope: diagnosticScope, ...selected, path: name,
    byte_length: length, sha256: digest, complete_archive_readback: true, production_accepted: false};
}
function decoded(value) {assert(typeof value === 'string' && value.length <= 24 * 1024 ** 2); const bytes = Buffer.from(value, 'base64'); assert.equal(bytes.toString('base64'), value); return bytes;}
export function expectedDiagnosticSubject(records, revision, run, attempt) {
  oid(revision); positive(run); positive(attempt);
  assert(Array.isArray(records) && records.length === 2);
  const rows = ['amd64', 'arm64'].map((arch, position) => {
    const value = records[position];
    keys(value, ['schema','scope','source_revision','architecture','run_id','run_attempt','production_accepted','complete_declared_layout_readback','descriptor','index_base64','manifest_base64','config_base64']);
    assert.equal(value.schema, 'prismpm/native-resource-production/1'); assert.equal(value.scope, diagnosticScope);
    assert.equal(value.source_revision, revision); assert.equal(value.architecture, arch);
    assert.equal(value.run_id, run); assert.equal(value.run_attempt, attempt);
    assert.equal(value.production_accepted, false); assert.equal(value.complete_declared_layout_readback, true);
    const index = JSON.parse(decoded(value.index_base64)), bytes = decoded(value.manifest_base64), config = decoded(value.config_base64);
    assert.equal(index.schemaVersion, 2); assert.equal(index.manifests.length, 1);
    assert.deepEqual(index.manifests[0], value.descriptor);
    assert.equal(sha(bytes), value.descriptor.digest); assert.equal(bytes.length, value.descriptor.size);
    const manifest = JSON.parse(bytes); assert.equal(manifest.config.digest, sha(config)); assert.equal(manifest.config.size, config.length);
    validateConfig(JSON.parse(config), arch, revision);
    return {architecture: arch, digest: value.descriptor.digest, bytes};
  });
  const index = platformIndex(rows);
  return {scope: diagnosticScope, source_revision: revision, index,
    image: 'ghcr.io/uor-foundation/prismpm-sdk@' + sha(index)};
}
export function productionArtifactName(arch, revision, run, attempt) {
  oid(revision); positive(run); positive(attempt); assert(['amd64', 'arm64'].includes(arch));
  return `resource-diagnostic-production-${arch}-${revision}-${run}-${attempt}`;
}
export function selectDiagnosticProduction(metadata, revision, run, attempt) {
  oid(revision); positive(run); positive(attempt); keys(metadata, ['artifacts', 'jobs']);
  assert(Array.isArray(metadata.artifacts) && Array.isArray(metadata.jobs));
  assert(metadata.artifacts.length <= 10000 && metadata.jobs.length <= 10000);
  const selected = ['amd64', 'arm64'].map(arch => {
    for (const job of metadata.jobs.filter(j => j.name === `Native resource production (${arch})`)) {
      assert.equal(String(job.run_id), run); assert.equal(job.head_sha, revision);
      assert(Number.isSafeInteger(job.run_attempt) && job.run_attempt > 0 && BigInt(job.run_attempt) <= BigInt(attempt), 'future diagnostic owning attempt');
    }
    for (const artifact of metadata.artifacts.filter(a => typeof a.name === 'string' && a.name.startsWith(`resource-diagnostic-production-${arch}-`))) {
      const originAttempt = artifact.name.split('-').at(-1); positive(originAttempt);
      assert(BigInt(originAttempt) <= BigInt(attempt), 'future diagnostic artifact');
      assert.equal(artifact.name, productionArtifactName(arch, revision, run, originAttempt), 'foreign diagnostic subject');
      assert.equal(String(artifact.workflow_run?.id), run); assert.equal(artifact.workflow_run.head_sha, revision);
    }
    const jobs = metadata.jobs.filter(j => j.name === `Native resource production (${arch})` && String(j.run_attempt) === attempt);
    assert.equal(jobs.length, 1, 'exact current-attempt diagnostic owner required'); const job = jobs[0];
    assert.equal(String(job.run_id), run); assert.equal(job.head_sha, revision); assert.equal(job.status, 'completed'); assert.equal(job.conclusion, 'success');
    assert(Number.isSafeInteger(job.id) && job.id > 0);
    const artifacts = metadata.artifacts.filter(a => a.name === productionArtifactName(arch, revision, run, attempt));
    assert.equal(artifacts.length, 1, 'exact immutable diagnostic artifact required'); const artifact = artifacts[0];
    assert(Number.isSafeInteger(artifact.id) && artifact.id > 0); assert.equal(artifact.expired, false);
    assert.equal(String(artifact.workflow_run?.id), run); assert.equal(artifact.workflow_run.head_sha, revision);
    assert(Number.isSafeInteger(artifact.size_in_bytes) && artifact.size_in_bytes > 0 && artifact.size_in_bytes <= 64 * 1024 ** 2);
    assert.match(artifact.digest, /^sha256:[a-f0-9]{64}$/);
    return {architecture: arch, id: artifact.id, digest: artifact.digest, name: artifact.name, job_id: job.id};
  });
  assert.notEqual(selected[0].id, selected[1].id); assert.notEqual(selected[0].job_id, selected[1].job_id);
  return {schema: 'prismpm/native-resource-selection/1', scope: diagnosticScope, source_revision: revision, run_id: run, run_attempt: attempt, artifacts: selected};
}

// Reuse the original private Linux subreaper and cancellation protocol. This
// controller never signals a remembered numeric PID or process group.
export async function runDiagnosticCommand(requestPath, execute = executeOwnedSdkCommand, environment = process.env) {
  const request = JSON.parse(original(requestPath, 65536));
  keys(request, ['schema','source_revision','run_id','run_attempt','command_id','deadline_ms','maximum_ms',
    'log_path','capture_path','result_path','cancel_file','stream_output','argv']);
  assert.equal(request.schema, 'prismpm/native-resource-command/1'); oid(request.source_revision);
  assert.equal(request.source_revision, environment.GITHUB_SHA); assert.equal(request.source_revision, environment.GITHUB_WORKFLOW_SHA);
  assert.equal(environment.GITHUB_REPOSITORY, 'UOR-Foundation/PrismPM'); assert.equal(environment.GITHUB_REF, diagnosticRef);
  assert.equal(environment.GITHUB_EVENT_NAME, 'workflow_dispatch');
  for (const field of ['run_id', 'run_attempt']) {positive(request[field]); assert.equal(request[field], environment[field === 'run_id' ? 'GITHUB_RUN_ID' : 'GITHUB_RUN_ATTEMPT']);}
  assert.match(request.command_id, /^[a-z0-9][a-z0-9-]{0,63}$/);
  assert(Number.isSafeInteger(request.deadline_ms) && Number.isSafeInteger(request.maximum_ms) && request.maximum_ms >= 0 && request.maximum_ms <= 21600000);
  assert.equal(typeof request.stream_output, 'boolean');
  assert(Array.isArray(request.argv) && request.argv.length > 0 && request.argv.length <= 256
    && request.argv.every(a => typeof a === 'string' && a.length > 0 && a.length <= 16384 && !a.includes('\0')));
  const control = dirname(resolve(requestPath)); assert.equal(basename(control), '.control'); assert.equal(realpathSync(control), control);
  for (const field of ['capture_path','result_path','cancel_file']) {
    assert.equal(resolve(request[field]), request[field]); assert.equal(dirname(request[field]), control);
    assert.notEqual(request[field], resolve(requestPath));
    assert.equal(lstatSync(request[field], {throwIfNoEntry: false}), undefined, 'fresh command-owned output required');
  }
  assert.equal(new Set([request.capture_path,request.result_path,request.cancel_file]).size, 3);
  assert([join(dirname(control), 'commands.log'), join(dirname(control), 'cleanup.log')].includes(request.log_path));
  const cap = basename(request.log_path) === 'cleanup.log' ? 16 * 1024 ** 2 : 256 * 1024 ** 2;
  const prior = lstatSync(request.log_path, {throwIfNoEntry: false});
  if (prior) assert(prior.isFile() && prior.nlink === 1 && realpathSync(request.log_path) === request.log_path && prior.size < cap);
  const admitted = Math.min(request.deadline_ms - Date.now(), request.maximum_ms || 21600000);
  assert(admitted > 5000 && admitted <= 21600000, 'shared deadline must include private owner retirement');
  const logHash = createHash('sha256'), captureHash = createHash('sha256');
  const initialLog = prior ? digestOriginal(request.log_path, cap, logHash) : undefined;
  assert.equal(initialLog?.length ?? 0, prior?.size ?? 0);
  const abort = new AbortController(), started = performance.now(); let reason = null, timer, response, cancelIdentity;
  let logfd, capturefd, logged = prior?.size ?? 0, captured = 0, truncated = false, stderr = Buffer.alloc(0);
  const completeWrite = (fd, bytes) => {for (let offset = 0; offset < bytes.length;) {const n = writeSync(fd, bytes, offset, bytes.length - offset); assert(n > 0, 'diagnostic write made no progress'); offset += n;}};
  const append = bytes => {assert(logged + bytes.length <= cap, 'aggregate command log limit'); completeWrite(logfd, bytes); logHash.update(bytes); logged += bytes.length;};
  const cancel = () => {
    try {
      if (Date.now() >= request.deadline_ms - 5000 || performance.now() - started >= admitted - 5000) throw Error('shared deadline');
      const stat = lstatSync(request.cancel_file, {throwIfNoEntry: false}); if (!stat) return;
      const value = JSON.parse(original(request.cancel_file, 16384));
      if (cancelIdentity) identity(lstatSync(request.cancel_file, {bigint: true}), cancelIdentity);
      else cancelIdentity = lstatSync(request.cancel_file, {bigint: true});
      keys(value, ['schema','source_revision','run_id','run_attempt','command_id','reason']);
      assert.equal(value.schema, 'prismpm/native-resource-cancel/1');
      for (const field of ['source_revision','run_id','run_attempt','command_id']) assert.equal(value[field], request[field]);
      assert(['interrupted','resource-monitor-unavailable','deadline'].includes(value.reason));
      throw Error(value.reason);
    } catch (error) {reason ??= error.message; abort.abort(error);}
  };
  try {
    logfd = openSync(request.log_path, constants.O_WRONLY | constants.O_APPEND | constants.O_CREAT | constants.O_NOFOLLOW, 0o600);
    const opened = fstatSync(logfd); assert(opened.isFile() && opened.nlink === 1 && opened.size === logged);
    if (initialLog) identity(fstatSync(logfd, {bigint: true}), initialLog.stat);
    capturefd = openSync(request.capture_path, constants.O_WRONLY | constants.O_CREAT | constants.O_EXCL | constants.O_NOFOLLOW, 0o600);
    append(Buffer.from(JSON.stringify({command_id: request.command_id, argv: request.argv, deadline_ms: request.deadline_ms}) + '\n'));
    cancel(); timer = setInterval(cancel, 100);
    if (!abort.signal.aborted) {
      response = await execute(request.argv[0], request.argv.slice(1), {profile: 'qualification', environment,
        timeout: Math.floor(admitted - 5000), limit: cap - logged, aggregate: true, signal: abort.signal,
        cwd: process.cwd(), interruptions: true, onOutput(stream, bytes) {
          append(bytes);
          if (stream === 'stderr') stderr = Buffer.concat([stderr, bytes]).subarray(-4096);
          else {
            const amount = Math.min(bytes.length, 1024 * 1024 - captured);
            if (amount) {const retained = bytes.subarray(0, amount); completeWrite(capturefd, retained); captureHash.update(retained); captured += amount;}
            if (amount < bytes.length) {truncated = true; if (!request.stream_output) throw Error('command stdout capture limit');}
          }
        }});
      cancel();
    }
    const observedCapture = digestOriginal(request.capture_path, 1024 * 1024);
    identity(fstatSync(capturefd, {bigint: true}), observedCapture.stat);
    assert.equal(observedCapture.length, captured); assert.equal(observedCapture.hash.digest('hex'), captureHash.digest('hex'), 'command capture bytes changed');
    const observedLog = digestOriginal(request.log_path, cap);
    identity(fstatSync(logfd, {bigint: true}), observedLog.stat);
    assert.equal(observedLog.length, logged); assert.equal(observedLog.hash.digest('hex'), logHash.digest('hex'), 'command log bytes changed');
    cancel();
  } catch (error) {reason ??= error.message; response = error.result ?? response;}
  finally {clearInterval(timer); if (capturefd !== undefined) closeSync(capturefd); if (logfd !== undefined) closeSync(logfd);}
  const safe = response && Number.isInteger(response.status) && response.status >= 0 && response.status <= 255
    && response.signal === null && !response.timedOut && !response.overflow && !response.aborted && !response.orphaned
    && response.retirement?.descendants_absent === true && response.retirement.uncertainty === null;
  if (!safe) reason ??= 'original process retirement or bounded execution unproven';
  const result = {status: safe && !reason ? response.status : 125, reason,
    stderr: stderr.toString('utf8'), retirement: response?.retirement ?? null,
    timed_out: response?.timedOut ?? false, overflow: response?.overflow ?? false,
    aborted: response?.aborted ?? abort.signal.aborted, orphaned: response?.orphaned ?? false,
    stdout_truncated: truncated, command_id: request.command_id, source_revision: request.source_revision,
    run_id: request.run_id, run_attempt: request.run_attempt};
  write(request.result_path, result); return result;
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const [command, ...args] = process.argv.slice(2);
  if (command === 'policy') {
    assert.equal(args.length, 0); const plan = {...diagnosticPolicy(process.env), ...policyDigests()};
    console.log('source=' + plan.source_revision + '\nwork-end-ms=' + plan.work_end_ms + '\nwhole-end-ms=' + plan.whole_end_ms);
    console.log('plan=' + JSON.stringify(plan));
  } else if (command === 'admit') {
    assert.equal(args.length, 4); const [revision, workEnd, wholeEnd, output] = args;
    assert.match(workEnd, /^[1-9][0-9]{12}$/); assert.match(wholeEnd, /^[1-9][0-9]{12}$/);
    write(output, admitDiagnosticPlan(JSON.parse(process.env.NATIVE_DIAGNOSTIC_PLAN), revision, Number(workEnd), Number(wholeEnd)));
  } else if (command === 'production-plan') {
    assert.equal(args.length, 4); console.log(JSON.stringify(diagnosticProductionPlan(...args)));
  } else if (command === 'metadata') {
    assert.equal(args.length, 4); const [layout, arch, revision, path] = args; write(path, productionRecord(layout, arch, revision));
  } else if (command === 'archive') {
    assert.equal(args.length, 4); const [directory, revision, arch, output] = args; write(output, diagnosticArchive(directory, revision, arch));
  } else if (command === 'select') {
    assert.equal(args.length, 5); const [path, revision, run, attempt, output] = args;
    write(output, selectDiagnosticProduction(JSON.parse(original(path)), revision, run, attempt));
  } else if (command === 'verify') {
    assert.equal(args.length, 6); const [directory, arch, revision, first, second, output] = args;
    const c = context(revision, arch), records = ['amd64', 'arm64'].map(a => JSON.parse(original(join(directory, `production-${a}.json`))));
    assert.notEqual(realpathSync(first), realpathSync(second), 'two independently owned absolute layouts required');
    const subject = expectedDiagnosticSubject(records, revision, c.run_id, c.run_attempt);
    const digest = verifyReproducibility(subject.image, subject.index, arch, revision, first, second);
    write(output, {schema: 'prismpm/native-resource-comparison/1', scope: diagnosticScope, ...c,
      expected_subject: subject.image, platform_manifest: digest, complete_original_pair_readback: true, production_accepted: false});
  } else if (command === 'command') {
    assert.equal(args.length, 1); const result = await runDiagnosticCommand(args[0]); process.exitCode = result.status;
  } else throw Error('unsupported native diagnostic operation');
}
