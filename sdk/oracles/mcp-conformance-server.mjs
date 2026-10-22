#!/usr/bin/env node
// Server-conformance oracle for MCP 2026-07-28.
//
// Starts the submitted server on loopback, runs the pinned conformance suite's
// `server --url ... --requirements 2026-07-28` runner against it, and decides
// from the suite's per-scenario check files, not from its exit status alone,
// because that runner also exits 1 for harness errors.
//
// usage: mcp-conformance-server [--suite-root DIR] SUBJECT_DIRECTORY
//
// The subject directory holds `prismpm-mcp-subject.json`:
//   { "version": "prismpm/mcp-server-subject/1", "spec": "2026-07-28",
//     "entry": "server.mjs", "path": "/mcp", "modules": "none" | "suite-fixture" }
// The server reads PORT (and HOST, always 127.0.0.1) from its environment.
//
// Exit 0: the frozen set of required scenarios all pass and the unscored ones
//   ran. Exit 1: a required scenario fails or the subject cannot be started;
//   the diagnostic names each failing check. Exit 2: the oracle cannot run, or
//   the suite's frozen requirements differ from the locked file.
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import net from 'node:net';
import { cpSync, existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, statSync, symlinkSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { isAbsolute, join, normalize, sep } from 'node:path';

const REVISION = '2026-07-28';
const SUBJECT_VERSION = 'prismpm/mcp-server-subject/1';
const MANIFEST = 'prismpm-mcp-subject.json';
// The frozen requirements file of the locked suite commit 232a9014.
const FROZEN_REQUIREMENTS_SHA256 = 'ae2f4f6210fd729e2e318edd5bbfa31a43cee0bc608e48052fa26dbf1d939b57';
const READY_MILLIS = 30000;
const SUITE_MILLIS = 240000;
const TAIL_BYTES = 8192;
const MAX_DIAGNOSTICS = 32;

class Invalid extends Error {}
class Unavailable extends Error {}

function usage() {
  process.stderr.write('usage: mcp-conformance-server [--suite-root DIR] SUBJECT_DIRECTORY\n');
  process.exit(2);
}

let suiteRoot = '/opt/prismpm/mcp-conformance';
const positional = [];
const args = process.argv.slice(2);
for (let index = 0; index < args.length; index += 1) {
  if (args[index] === '--suite-root') {
    suiteRoot = args[++index];
    if (!suiteRoot) usage();
  } else {
    positional.push(args[index]);
  }
}
if (positional.length !== 1) usage();
const subjectDirectory = positional[0];

const tail = (buffer) => buffer.subarray(Math.max(0, buffer.length - TAIL_BYTES)).toString('utf8');

function collect(stream) {
  const state = { chunks: [], size: 0 };
  stream.on('data', (chunk) => {
    state.chunks.push(chunk);
    state.size += chunk.length;
    while (state.size > TAIL_BYTES * 4 && state.chunks.length > 1) state.size -= state.chunks.shift().length;
  });
  return () => tail(Buffer.concat(state.chunks));
}

function loadManifest() {
  let manifest;
  try {
    manifest = JSON.parse(readFileSync(join(subjectDirectory, MANIFEST), 'utf8'));
  } catch (error) {
    throw new Invalid(`${MANIFEST} is missing or not JSON: ${error.message}`);
  }
  if (manifest?.version !== SUBJECT_VERSION) throw new Invalid(`${MANIFEST} version must be ${SUBJECT_VERSION}`);
  if (manifest.spec !== REVISION) throw new Invalid(`${MANIFEST} must declare specification ${REVISION}`);
  if (typeof manifest.path !== 'string' || !/^\/[A-Za-z0-9._~/-]*$/.test(manifest.path)) {
    throw new Invalid(`${MANIFEST} path must be an absolute URL path`);
  }
  if (!['none', 'suite-fixture'].includes(manifest.modules)) {
    throw new Invalid(`${MANIFEST} modules must be none or suite-fixture`);
  }
  const entry = manifest.entry;
  if (typeof entry !== 'string' || !/\.(mjs|js|ts)$/.test(entry) || isAbsolute(entry) || normalize(entry).split(sep).includes('..')) {
    throw new Invalid(`${MANIFEST} entry must be a relative .mjs, .js or .ts path inside the subject`);
  }
  if (entry.endsWith('.ts') && manifest.modules !== 'suite-fixture') {
    throw new Invalid(`${MANIFEST} a .ts entry needs the suite-fixture modules that provide tsx`);
  }
  let kind;
  try {
    kind = statSync(join(subjectDirectory, entry));
  } catch {
    kind = null;
  }
  if (!kind?.isFile()) throw new Invalid(`${MANIFEST} entry ${entry} is not a file in the subject`);
  return manifest;
}

// The suite owns which scenarios are required; this oracle re-reads the same
// frozen file and refuses to run if it is not the one the lock was built for.
function loadRequirements() {
  const file = join(suiteRoot, 'requirements', `${REVISION}.yaml`);
  let bytes;
  try {
    bytes = readFileSync(file);
  } catch (error) {
    throw new Unavailable(`the frozen requirements file is unreadable: ${error.message}`);
  }
  const digest = createHash('sha256').update(bytes).digest('hex');
  if (digest !== FROZEN_REQUIREMENTS_SHA256) {
    throw new Unavailable(`the suite's ${REVISION} requirements differ from the frozen file: expected ${FROZEN_REQUIREMENTS_SHA256}, observed ${digest}`);
  }
  const { parse } = createRequire(join(suiteRoot, 'package.json'))('yaml');
  const parsed = parse(bytes.toString('utf8'));
  const required = parsed.server ?? [];
  const unscored = (parsed.not_scored ?? []).filter((entry) => entry.leg === 'server').map((entry) => entry.scenario);
  if (required.length === 0 || new Set([...required, ...unscored]).size !== required.length + unscored.length) {
    throw new Unavailable('the frozen requirements do not name distinct server scenarios');
  }
  return { required, unscored };
}

function freePort() {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const { port } = server.address();
      server.close(() => resolve(port));
    });
  });
}

function accepts(port) {
  return new Promise((resolve) => {
    const socket = net.connect({ port, host: '127.0.0.1' });
    socket.once('connect', () => { socket.destroy(); resolve(true); });
    socket.once('error', () => resolve(false));
  });
}

async function startSubject(manifest, runDirectory, port) {
  cpSync(subjectDirectory, runDirectory, { recursive: true, verbatimSymlinks: false });
  if (manifest.modules === 'suite-fixture') {
    symlinkSync(join(suiteRoot, 'fixture', 'node_modules'), join(runDirectory, 'node_modules'), 'dir');
  }
  const argv = manifest.entry.endsWith('.ts') ? ['--import', 'tsx', manifest.entry] : [manifest.entry];
  const child = spawn(process.execPath, argv, {
    cwd: runDirectory,
    env: { PATH: process.env.PATH ?? '/usr/bin:/bin', HOME: runDirectory, TMPDIR: runDirectory, PORT: String(port), HOST: '127.0.0.1' },
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  const stdout = collect(child.stdout);
  const stderr = collect(child.stderr);
  let exited = null;
  child.once('exit', (code, signal) => { exited = { code, signal }; });
  child.once('error', (error) => { exited = { code: null, signal: error.message }; });
  const deadline = Date.now() + READY_MILLIS;
  while (Date.now() < deadline) {
    if (exited) {
      throw new Invalid(`the server exited before listening (code ${exited.code}, signal ${exited.signal}): ${stderr() || stdout()}`);
    }
    if (await accepts(port)) return { child, stderr };
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  child.kill('SIGKILL');
  throw new Invalid(`the server did not listen on PORT within ${READY_MILLIS} ms: ${stderr() || stdout()}`);
}

function stop(child) {
  return new Promise((resolve) => {
    if (child.exitCode !== null || child.signalCode !== null) return resolve();
    child.once('exit', () => resolve());
    child.kill('SIGTERM');
    setTimeout(() => child.kill('SIGKILL'), 2000).unref();
  });
}

function runSuite(url, outputDirectory) {
  return new Promise((resolve, reject) => {
    const child = spawn(process.execPath, [join(suiteRoot, 'dist', 'index.js'), 'server', '--url', url, '--requirements', REVISION, '-o', outputDirectory], {
      cwd: suiteRoot,
      env: { PATH: process.env.PATH ?? '/usr/bin:/bin', HOME: outputDirectory, TMPDIR: outputDirectory },
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    const stdout = collect(child.stdout);
    const stderr = collect(child.stderr);
    const timer = setTimeout(() => child.kill('SIGKILL'), SUITE_MILLIS);
    child.once('error', (error) => { clearTimeout(timer); reject(new Unavailable(`the suite could not start: ${error.message}`)); });
    child.once('exit', (code) => { clearTimeout(timer); resolve({ code, stdout: stdout(), stderr: stderr() }); });
  });
}

// `server-<scenario>-<ISO timestamp>/checks.json`, one directory per scenario run.
function readResults(outputDirectory) {
  const results = new Map();
  for (const name of readdirSync(outputDirectory)) {
    const match = /^server-(.+)-\d{4}-\d{2}-\d{2}T[\d-]+Z$/.exec(name);
    if (!match) continue;
    const file = join(outputDirectory, name, 'checks.json');
    if (!existsSync(file)) continue;
    const checks = JSON.parse(readFileSync(file, 'utf8'));
    if (!Array.isArray(checks)) throw new Unavailable(`${name}/checks.json is not a list`);
    results.set(match[1], (results.get(match[1]) ?? []).concat(checks));
  }
  return results;
}

async function main() {
  const manifest = loadManifest();
  const { required, unscored } = loadRequirements();
  if (!existsSync(join(suiteRoot, 'dist', 'index.js'))) throw new Unavailable('the built conformance suite is absent');

  const work = mkdtempSync(join(process.env.TMPDIR || tmpdir(), 'mcp-conformance-'));
  const runDirectory = join(work, 'subject');
  const outputDirectory = join(work, 'results');
  mkdirSync(runDirectory);
  mkdirSync(outputDirectory);
  let subject;
  try {
    const port = await freePort();
    subject = await startSubject(manifest, runDirectory, port);
    const suite = await runSuite(`http://127.0.0.1:${port}${manifest.path}`, outputDirectory);
    const results = readResults(outputDirectory);

    const failing = [];
    const diagnostics = [];
    for (const scenario of required) {
      const checks = results.get(scenario);
      if (!checks || checks.length === 0) {
        failing.push(scenario);
        diagnostics.push(`${scenario}: the scenario produced no checks`);
        continue;
      }
      const failures = checks.filter((check) => check.status === 'FAILURE');
      if (failures.length > 0) failing.push(scenario);
      for (const check of failures) {
        diagnostics.push(`${scenario}: ${check.id}: ${check.errorMessage ?? check.description ?? 'failed'}`);
      }
    }
    const missing = unscored.filter((scenario) => !results.has(scenario));
    const unscoredFailing = unscored.filter((scenario) => (results.get(scenario) ?? []).some((check) => check.status === 'FAILURE'));
    const unexpected = [...results.keys()].filter((scenario) => !required.includes(scenario) && !unscored.includes(scenario));
    const report = {
      oracle: `mcp-conformance/232a9014+${REVISION}`,
      requirements_sha256: FROZEN_REQUIREMENTS_SHA256,
      scored: { scenarios: required.length, failing },
      not_scored: { scenarios: unscored.length, run: unscored.length - missing.length, failing: unscoredFailing },
      valid: failing.length === 0,
    };
    process.stdout.write(`${JSON.stringify(report)}\n`);

    if (failing.length > 0) {
      for (const line of diagnostics.slice(0, MAX_DIAGNOSTICS)) process.stderr.write(`${line}\n`);
      if (diagnostics.length > MAX_DIAGNOSTICS) process.stderr.write(`... ${diagnostics.length - MAX_DIAGNOSTICS} further failing checks\n`);
      // The suite exits 1 for a scored failure; any other status means the
      // check files and the runner disagree, which is a harness fault.
      if (suite.code !== 1) throw new Unavailable(`the suite exited ${suite.code} although required scenarios failed`);
      return 1;
    }
    if (missing.length > 0 || unexpected.length > 0) {
      throw new Unavailable(`the suite ran a different scenario set (missing unscored: ${missing.join(', ') || 'none'}; unexpected: ${unexpected.join(', ') || 'none'})`);
    }
    if (suite.code !== 0) throw new Unavailable(`the suite exited ${suite.code} with every required scenario passing: ${suite.stderr}`);
    return 0;
  } finally {
    if (subject) await stop(subject.child);
    rmSync(work, { recursive: true, force: true });
  }
}

try {
  process.exitCode = await main();
} catch (error) {
  if (error instanceof Invalid) {
    process.stderr.write(`${error.message}\n`);
    process.exit(1);
  }
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(2);
}
