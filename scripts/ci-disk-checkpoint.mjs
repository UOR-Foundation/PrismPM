// Diagnostic-only, bounded early snapshots. Never an acceptance result.
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { constants, closeSync, existsSync, fstatSync, mkdirSync, openSync, readFileSync, readSync, renameSync, rmSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { performance } from 'node:perf_hooks';
import { setTimeout as delay } from 'node:timers/promises';
import { diskComponents } from './ci-disk-components.mjs';
import { parts } from './vv-assemble.mjs';

const script = fileURLToPath(import.meta.url);
const uploaderHash = '0165b8a75330f3228f2c7a234b4ff8a107b9139c2b519147f4c8e9fe99b262d8';
export const bounds = Object.freeze({attempts: 40, intervalMs: 10 * 60_000, uploadMs: 30_000, snapshotBytes: 16_384});
const numericId = value => /^(?:0|[1-9][0-9]{0,19})$/.test(value ?? '');

const labels = ['workspace-target', 'workspace-other', 'docker-storage', 'runner-installation', 'hosted-toolchains'];
const componentStatuses = new Set(['complete', 'partial', 'timed-out', 'unavailable', 'entry-limit',
  'depth-limit', 'output-limit', 'failed', 'invalid-output', 'not-sampled']);
export function sanitizeSample(raw) {
  const components = {};
  for (const label of labels) {
    const value = raw?.components?.[label];
    const row = {status: componentStatuses.has(value?.status) ? value.status : 'invalid-output'};
    for (const field of ['allocatedBytes', 'entriesVisited', 'symlinks', 'skippedMounts', 'errors']) {
      const n = value?.[field];
      row[field] = Number.isSafeInteger(n) && n >= 0 ? n : null;
    }
    components[label] = row;
  }
  const capacity = {};
  for (const label of ['workspace', 'docker']) {
    const value = raw?.capacity?.[label];
    const allowed = new Set(['ok', 'unavailable', 'timed-out', 'output-limit', 'failed', 'invalid-output', 'not-sampled']);
    const row = {status: allowed.has(value?.status) ? value.status : 'invalid-output'};
    for (const field of ['totalBytes', 'availableBytes', 'freeFiles']) {
      const n = value?.[field];
      row[field] = Number.isSafeInteger(n) && n >= 0 ? n : null;
    }
    capacity[label] = row;
  }
  return {schema: 1, components, capacity};
}

// Keep only the artifact runtime and fixed GitHub context. Never echo child
// output: it can contain service endpoints or credentials on a failed request.
export function checkpointPart(env) {
  if (!Object.hasOwn(env, 'PRISMPM_CI_CHECKPOINT_PART')) return undefined;
  const part = env.PRISMPM_CI_CHECKPOINT_PART;
  if (typeof part !== 'string' || !parts.includes(part)) throw new Error('invalid checkpoint part');
  return part;
}
export function runtimeEnvironment(env) {
  const result = {};
  for (const key of ['PATH', 'HOME', 'ACTIONS_RUNTIME_TOKEN', 'ACTIONS_RUNTIME_URL', 'ACTIONS_RESULTS_URL',
    'GITHUB_RUN_ID', 'GITHUB_RUN_ATTEMPT', 'GITHUB_REPOSITORY', 'GITHUB_SERVER_URL', 'GITHUB_WORKSPACE', 'RUNNER_TRACKING_ID']) {
    if (typeof env[key] === 'string') result[key] = env[key];
  }
  const part = checkpointPart(env);
  if (part !== undefined) result.PRISMPM_CI_CHECKPOINT_PART = part;
  return result;
}
export function uploadEnvironment(env, path, output, name) {
  return {...runtimeEnvironment(env), GITHUB_OUTPUT: output, INPUT_NAME: name, INPUT_PATH: path,
    'INPUT_IF-NO-FILES-FOUND': 'error', 'INPUT_RETENTION-DAYS': '7', INPUT_OVERWRITE: 'false',
    'INPUT_COMPRESSION-LEVEL': '0', 'INPUT_INCLUDE-HIDDEN-FILES': 'false'};
}

export async function uploadOnce({entry, path, output, name, env = process.env, timeoutMs = bounds.uploadMs, stopFile}) {
  if (!Number.isSafeInteger(timeoutMs) || timeoutMs < 1 || timeoutMs > bounds.uploadMs) throw new Error('invalid upload bound');
  return await new Promise(resolveResult => {
    let settled = false, timedOut = false, cancelled = false, exited = false;
    const child = spawn(process.execPath, [entry], {detached: true, stdio: 'ignore', env: uploadEnvironment(env, path, output, name)});
    const finish = result => { if (!settled) { settled = true; clearTimeout(timer); clearInterval(poller); resolveResult(result); } };
    const terminate = () => {
      if (!exited && child.exitCode === null && child.signalCode === null) {
        try { process.kill(-child.pid, 'SIGKILL'); } catch {}
        return true;
      }
      return false;
    };
    const timer = setTimeout(() => { timedOut = terminate(); }, timeoutMs);
    const poller = setInterval(() => {
      if (stopFile && existsSync(stopFile)) cancelled = terminate() || cancelled;
    }, 100);
    child.once('exit', () => { exited = true; });
    child.once('error', () => finish({status: 'unavailable'}));
    child.once('close', code => {
      if (cancelled) return finish({status: 'cancelled'});
      if (timedOut) return finish({status: 'timeout'});
      if (code !== 0) return finish({status: 'failed'});
      try {
        const fd = openSync(output, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
        let body;
        try {
          const info = fstatSync(fd);
          if (!info.isFile() || info.size > 4096) return finish({status: 'unconfirmed'});
          const bytes = Buffer.alloc(4097), length = readSync(fd, bytes, 0, bytes.length, 0);
          if (length > 4096) return finish({status: 'unconfirmed'});
          body = bytes.subarray(0, length).toString('utf8');
        } finally { closeSync(fd); }
        const matches = [...body.matchAll(/^artifact-id<<([^\r\n]{1,120})\r?\n([0-9]{1,20})\r?\n\1$/gm)];
        if (matches.length !== 1 || !numericId(matches[0][2])) return finish({status: 'unconfirmed'});
        return finish({status: 'uploaded'});
      } catch { return finish({status: 'unconfirmed'}); }
    });
  });
}

export async function checkpoint(directory, entry, {env = process.env, attempts = bounds.attempts,
  intervalMs = bounds.intervalMs, collect = diskComponents, upload = uploadOnce} = {}) {
  if (!Number.isSafeInteger(attempts) || attempts < 1 || attempts > bounds.attempts ||
      !Number.isSafeInteger(intervalMs) || intervalMs < 1 || intervalMs > bounds.intervalMs) throw new Error('invalid checkpoint bounds');
  const part = checkpointPart(env);
  const namespace = part === undefined ? {} : {part};
  if (!numericId(env.GITHUB_RUN_ID) || !numericId(env.GITHUB_RUN_ATTEMPT)) throw new Error('missing run identity');
  let previous = 'none', made = 0;
  const deadline = performance.now() + 6 * 60 * 60_000;
  while (made < attempts && performance.now() < deadline && !existsSync(join(directory, 'stop'))) {
    const sequence = made++;
    let sample;
    try { sample = sanitizeSample(await collect({componentOffset: sequence % 5})); } catch { sample = sanitizeSample(null); }
    // The collector supplies only closed labels and numeric/status values. No
    // filesystem paths, filenames, environment or process output are retained.
    const snapshot = Buffer.from(JSON.stringify({version: 1, ...namespace, sequence, time: new Date().toISOString(), previousUpload: previous, sample}) + '\n');
    const path = join(directory, `snapshot-${sequence}.json`), output = join(directory, `output-${sequence}`);
    if (snapshot.length > bounds.snapshotBytes) { previous = 'oversize'; }
    else {
      let ownsPath = false, ownsOutput = false;
      try {
        writeFileSync(path, snapshot, {mode: 0o600, flag: 'wx'}); ownsPath = true;
        const latest = join(directory, `latest-${sequence}`);
        let ownsLatest = false;
        try {
          writeFileSync(latest, snapshot, {mode: 0o600, flag: 'wx'}); ownsLatest = true;
          renameSync(latest, join(directory, 'latest.json')); ownsLatest = false;
        } catch {} finally { try { if (ownsLatest) rmSync(latest, {force: true}); } catch {} }
        writeFileSync(output, '', {mode: 0o600, flag: 'wx'}); ownsOutput = true;
        if (sequence === 0) writeFileSync(join(directory, 'ready'), '', {mode: 0o600, flag: 'wx'});
        const name = `vv-disk-${env.GITHUB_RUN_ID}-${env.GITHUB_RUN_ATTEMPT}-${sequence}${part === undefined ? '' : '-' + part}`;
        const result = await upload({entry, path, output, name, env, stopFile: join(directory, 'stop')});
        previous = ['uploaded', 'failed', 'timeout', 'unavailable', 'unconfirmed', 'cancelled'].includes(result?.status) ? result.status : 'unconfirmed';
      } catch { previous = 'failed'; }
      finally {
        try { if (ownsPath) rmSync(path, {force: true}); } catch {}
        try { if (ownsOutput) rmSync(output, {force: true}); } catch {}
      }
    }
    const status = join(directory, `status-${sequence}`);
    let ownsStatus = false;
    try {
      writeFileSync(status, JSON.stringify({...namespace, attempts: made, lastUpload: previous}) + '\n', {mode: 0o600, flag: 'wx'});
      ownsStatus = true; renameSync(status, join(directory, 'status.json')); ownsStatus = false;
    } catch {} finally { try { if (ownsStatus) rmSync(status, {force: true}); } catch {} }
    if (made >= attempts) break;
    const next = Math.min(deadline, performance.now() + intervalMs);
    while (performance.now() < next && !existsSync(join(directory, 'stop'))) await delay(Math.min(250, next - performance.now()));
  }
}

async function main() {
  const [mode, directoryArg, entryArg] = process.argv.slice(2);
  if (!directoryArg || !['start', 'stop', '_monitor'].includes(mode)) throw new Error('invalid invocation');
  const directory = resolve(directoryArg);
  if (mode === 'stop') {
    if (!existsSync(directory)) return;
    closeSync(openSync(join(directory, 'stop'), constants.O_WRONLY | constants.O_CREAT | constants.O_NOFOLLOW | constants.O_NONBLOCK, 0o600));
    const deadline = performance.now() + 8_000;
    while (!existsSync(join(directory, 'done'))) {
      if (performance.now() >= deadline) throw new Error('checkpoint did not stop');
      await delay(100);
    }
    return;
  }
  checkpointPart(process.env);
  if (!entryArg) throw new Error('missing uploader');
  const entry = resolve(entryArg);
  if (mode === 'start') {
    const fd = openSync(entry, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
    try {
      const info = fstatSync(fd);
      if (!info.isFile() || info.size !== 5_051_718) throw new Error('uploader pin mismatch');
      const hash = createHash('sha256'), bytes = Buffer.alloc(65_536);
      let offset = 0;
      while (offset < info.size) {
        const count = readSync(fd, bytes, 0, Math.min(bytes.length, info.size - offset), offset);
        if (!count) throw new Error('uploader truncated');
        hash.update(bytes.subarray(0, count)); offset += count;
      }
      if (readSync(fd, bytes, 0, 1, offset) !== 0 || hash.digest('hex') !== uploaderHash)
        throw new Error('uploader pin mismatch');
    } finally { closeSync(fd); }
    mkdirSync(directory, {recursive: false, mode: 0o700});
    const child = spawn(process.execPath, [script, '_monitor', directory, entry], {detached: true, stdio: 'ignore', env: runtimeEnvironment(process.env)});
    let failed = false, exited = false;
    child.once('error', () => { failed = true; }); child.once('exit', () => { exited = true; }); child.unref();
    const deadline = performance.now() + 30_000;
    while (!existsSync(join(directory, 'ready'))) {
      if (failed || performance.now() >= deadline) {
        try { closeSync(openSync(join(directory, 'stop'), constants.O_WRONLY | constants.O_CREAT | constants.O_NOFOLLOW | constants.O_NONBLOCK, 0o600)); } catch {}
        if (!exited && child.exitCode === null && child.signalCode === null) { try { process.kill(-child.pid, 'SIGKILL'); } catch {} }
        throw new Error('checkpoint did not start');
      }
      await delay(100);
    }
    return;
  }
  try { await checkpoint(directory, entry); }
  finally { try { writeFileSync(join(directory, 'done'), '', {mode: 0o600}); } catch {} }
}
if (process.argv[1] && resolve(process.argv[1]) === script) main().catch(() => {
  process.stderr.write('Disk checkpoint diagnostics unavailable.\n'); process.exitCode = 1;
});
