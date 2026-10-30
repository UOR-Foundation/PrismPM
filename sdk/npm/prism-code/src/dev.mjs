/**
 * `prism-code dev` (issue #288): watch the model and `prism.toml`, rebuild,
 * re-check, and serve a local studio that shows the latest verdict.
 *
 * The loop is the one a developer runs by hand, in order: the project's build
 * (`model.build` in `prism.toml`, else `prism-code build`, which is `prismpm
 * build` in the locked SDK image), then `check` on the holo the build wrote,
 * with the same policy `prism-code check` takes. The verdict stays in memory;
 * nothing is written outside the build root. Bursts of file events are
 * debounced into one rebuild, and a change that arrives while a run is in
 * flight queues exactly one more run.
 *
 * The studio is static files served from this process (`../studio`), with
 * no CDN and no network call: a `Content-Security-Policy` of `'self'` makes
 * the browser refuse any other origin, so the studio works offline. Live
 * updates go over Server-Sent Events (`GET /events`), never polling. The
 * state carries the edit-to-verdict time of every run, so the done-when of
 * #288 (under the SDK image's rebuild floor) can be measured.
 *
 *   GET  /            the studio
 *   GET  /state       the whole state, JSON
 *   GET  /verdict     the latest verdict, exactly as `check --json` prints it (404 until there is one)
 *   GET  /events      Server-Sent Events: `state` on every change
 *   POST /rebuild     run the loop once, now
 *
 * The port is fixed, `DEFAULT_PORT`, and a port in use refuses to start
 * (`dev.port`) instead of picking another, so a bookmark and a script always
 * reach the same studio. Replay (#284, PR 415) is read through a feature
 * check: until it is in this build the Replay tab says so.
 *
 * Experimental until the v0.6 freeze (STABILITY.md section 6).
 */
import { watch } from 'node:fs';
import { readdir, readFile, stat } from 'node:fs/promises';
import { createServer } from 'node:http';
import { spawn } from 'node:child_process';
import { EventEmitter } from 'node:events';
import { basename, dirname, extname, join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { check, LIMITS, verdictJson, titleOf } from './check.mjs';
import { ASSURANCE } from './vendor/verify.mjs';
import { parseToml } from './toml.mjs';
import { parseManifest, MANIFEST_FILE, MANIFEST_TARGETS } from './manifest.mjs';
import { PrismCodeError, PrismError } from './errors.mjs';

export const DEFAULT_PORT = 4100;
export const DEFAULT_HOST = '127.0.0.1';
export const DEBOUNCE_MS = 150;
/** What the Replay tab says until #284's `replay` is in this build. */
export const REPLAY_UNAVAILABLE = 'replay not available in this build';
/** Where the studio looks for histories, relative to the project: every `.json` directly in it, as `prism-code replay <dir>` reads them. */
export const HISTORIES_DIR = 'histories';
export const STUDIO_DIR = join(dirname(fileURLToPath(import.meta.url)), '..', 'studio');
const BIN = join(dirname(fileURLToPath(import.meta.url)), '..', 'bin', 'prism-code.mjs');

/** The project files the loop always watches, besides the model globs. */
export const PROJECT_FILES = Object.freeze([MANIFEST_FILE, ...MANIFEST_TARGETS.map((t) => `prism.${t}.toml`), 'prismpm.toml', 'lexlean.toml', 'lexlean.lock', 'prismpm.lock']);
/** Directories never watched: version control, dependencies, and the Lean and LexLean build trees. The build root joins them. */
export const IGNORED_DIRS = Object.freeze(['.git', 'node_modules', '.lake', '.lexlean']);
/** The model globs when `prism.toml` names none: the init template's sources and lexicons. */
export const DEFAULT_WATCH = Object.freeze(['src/**/*.lex.tex', 'language/**/*.toml']);
const DEFAULT_BUILD_ROOT = '.prism';
const OUTPUT_TAIL = 64 * 1024;
const HISTORY_RUNS = 20;
const HOLO_DEPTH = 4;
const HEARTBEAT_MS = 15_000;

const CONTENT_TYPES = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.css': 'text/css; charset=utf-8', '.svg': 'image/svg+xml' };
const POSIX = (path) => path.split(sep).join('/');

/**
 * A glob as `prism.toml`'s `model.watch` spells it, as a regular expression
 * over a project-relative POSIX path: `**` is any run of directories, `*`
 * and `?` stay inside one segment.
 */
export function globToRegExp(glob) {
  const parts = glob.split('/');
  let source = '^';
  parts.forEach((part, i) => {
    const last = i === parts.length - 1;
    if (part === '**') {
      source += last ? '.*' : '(?:[^/]+/)*';
      return;
    }
    for (const ch of part) {
      if (ch === '*') source += '[^/]*';
      else if (ch === '?') source += '[^/]';
      else source += ch.replace(/[.+^${}()|[\]\\]/, '\\$&');
    }
    if (!last) source += '/';
  });
  return new RegExp(`${source}$`);
}

async function readText(path) {
  return readFile(path, 'utf8').catch(() => null);
}

/**
 * What the loop knows about a project directory: the manifest when there is
 * one, the globs to watch, the build command and the build root. A manifest
 * that does not parse is reported, not fatal: the loop still watches, so the
 * fix shows up in the studio.
 *
 * @param {string} directory
 * @param {{ build?: string }} [options] `build` overrides the build command
 */
export async function describeProject(directory, options = {}) {
  const root = resolve(directory);
  const project = {
    directory: root,
    name: basename(root),
    manifest: null,
    manifestError: null,
    watch: [...DEFAULT_WATCH],
    files: [...PROJECT_FILES],
    buildRoot: DEFAULT_BUILD_ROOT,
    buildCommand: options.build ?? null,
    ignored: [...IGNORED_DIRS],
  };
  const manifestText = await readText(join(root, MANIFEST_FILE));
  if (manifestText !== null) {
    try {
      const { manifest } = parseManifest(manifestText);
      project.manifest = { name: manifest.application.name, version: manifest.application.version };
      project.name = manifest.application.name;
      const model = manifest.model ?? {};
      if (model.watch) project.watch = [...model.watch];
      else if (model.source) project.watch = [`${model.source.replace(/\/$/, '')}/**`];
      if (model.build && project.buildCommand === null) project.buildCommand = model.build;
    } catch (error) {
      if (!(error instanceof PrismCodeError)) throw error;
      project.manifestError = { code: error.code, message: error.message };
    }
  }
  for (const [file, key] of [
    ['prismpm.toml', 'build_root'],
    ['lexlean.toml', 'build_root'],
  ]) {
    const text = await readText(join(root, file));
    if (text === null) continue;
    try {
      const value = parseToml(text)[key];
      if (typeof value === 'string' && value !== '' && !value.includes('..')) {
        if (file === 'prismpm.toml') project.buildRoot = value.replace(/\/$/, '');
        else project.ignored.push(value.replace(/\/$/, ''));
      }
    } catch {
      // A file the strict reader refuses keeps the defaults; the build will say what is wrong with it.
    }
  }
  if (!project.ignored.includes(project.buildRoot)) project.ignored.push(project.buildRoot);
  return project;
}

/** True when a project-relative path is one the loop rebuilds for. */
export function watches(project, path) {
  const posix = POSIX(path);
  const first = posix.split('/')[0];
  if (project.ignored.includes(first)) return false;
  if (project.files.includes(posix)) return true;
  return project.watch.some((glob) => globToRegExp(glob).test(posix));
}

/**
 * The newest `.holo` under the build root (`<root>/build/<id>/<Name>.holo`,
 * as `prismpm build` writes it), or null.
 */
export async function locateHolo(directory, buildRoot = DEFAULT_BUILD_ROOT) {
  const found = [];
  async function walk(dir, depth) {
    let entries;
    try {
      entries = await readdir(dir, { withFileTypes: true });
    } catch {
      return;
    }
    for (const entry of entries) {
      const path = join(dir, entry.name);
      if (entry.isDirectory()) {
        if (depth > 0) await walk(path, depth - 1);
      } else if (/\.holo$/i.test(entry.name)) {
        const info = await stat(path).catch(() => null);
        if (info) found.push({ path, mtime: info.mtimeMs });
      }
    }
  }
  await walk(join(resolve(directory), buildRoot), HOLO_DEPTH);
  found.sort((a, b) => b.mtime - a.mtime || a.path.localeCompare(b.path));
  return found[0]?.path ?? null;
}

/**
 * Run the project's build: `command` through the shell when the manifest or
 * the caller names one, otherwise `prism-code build` in the project, which is
 * `prismpm build` in the SDK image `prismpm.lock` pins. Resolves to
 * `{ ok, code, output }`, where `output` is the tail of what it printed.
 */
export function runBuild(directory, command, { env = process.env } = {}) {
  return new Promise((done) => {
    const child = command ? spawn(command, { cwd: directory, shell: true, env, stdio: ['ignore', 'pipe', 'pipe'], windowsHide: true }) : spawn(process.execPath, [BIN, 'build'], { cwd: directory, env, stdio: ['ignore', 'pipe', 'pipe'], windowsHide: true });
    let output = '';
    const collect = (chunk) => {
      output += chunk;
      if (output.length > OUTPUT_TAIL) output = output.slice(-OUTPUT_TAIL);
    };
    child.stdout.on('data', collect);
    child.stderr.on('data', collect);
    child.on('error', (error) => done({ ok: false, code: null, output: `${output}${error.message}\n` }));
    child.on('exit', (code, signal) => done({ ok: code === 0, code: code ?? (signal ? 1 : 0), output }));
  });
}

/**
 * Replay as the studio reads it (#284): the file side of `prism-code replay`
 * when this build has it, a `{ available: false }` otherwise. Histories are
 * every `.json` directly in `<project>/histories`, as `prism-code replay <dir>`
 * reads them.
 */
export async function replaySupport(directory, { historiesDir = HISTORIES_DIR, modules = null } = {}) {
  let cli;
  let node;
  try {
    [cli, node] = modules ?? (await Promise.all([import('./replay-cli.mjs'), import('./node.mjs')]));
  } catch (error) {
    if (error?.code === 'ERR_MODULE_NOT_FOUND') return { available: false, reason: REPLAY_UNAVAILABLE, histories: historiesDir };
    throw error;
  }
  if (typeof cli.replayFiles !== 'function') return { available: false, reason: REPLAY_UNAVAILABLE, histories: historiesDir };
  return {
    available: true,
    histories: historiesDir,
    async run(holo) {
      const dir = join(resolve(directory), historiesDir);
      const info = await stat(dir).catch(() => null);
      const count = info?.isDirectory() ? (await readdir(dir)).filter((name) => /\.json$/i.test(name)).length : 0;
      if (count === 0) return { files: [], result: null, note: `no history: put one or more prism/history/1 files in ${historiesDir}/` };
      const app = await node.fromFile(holo);
      const { result, files } = await cli.replayFiles(app, [dir]);
      return { files: files.map((f) => POSIX(relative(resolve(directory), f))), result, note: null };
    },
  };
}

const round = (ms) => Math.round(ms * 10) / 10;

/**
 * The loop: build, locate the holo, check, replay. `trigger` collects a burst
 * of changed paths and runs once after `debounceMs` of quiet; a trigger during
 * a run queues exactly one more. Emits `state` after every change of state.
 *
 * Every step is a function, so tests run the loop on a stub builder:
 *
 * @param {object} options
 * @param {object} options.project from `describeProject`
 * @param {(context: { directory: string, command: string | null, run: number }) => Promise<{ ok: boolean, code: number | null, output: string }>} [options.build]
 * @param {() => Promise<string | null>} [options.locate] the holo the build wrote
 * @param {(holo: string, options: { policy?: string }) => Promise<object>} [options.judge] `check`
 * @param {{ available: boolean, reason?: string, run?: (holo: string) => Promise<object> }} [options.replay]
 * @param {string} [options.policy] the policy file, as `--policy`
 * @param {number} [options.debounceMs]
 * @param {() => number} [options.now] a monotonic clock in milliseconds
 */
export function createDevLoop({ project, build = (c) => runBuild(c.directory, c.command), locate = () => locateHolo(project.directory, project.buildRoot), judge = check, replay = { available: false, reason: REPLAY_UNAVAILABLE }, policy = undefined, debounceMs = DEBOUNCE_MS, now = () => performance.now() }) {
  const events = new EventEmitter();
  const state = {
    project: {
      directory: project.directory,
      name: project.name,
      manifest: project.manifest,
      manifestError: project.manifestError,
      watch: project.watch,
      files: project.files,
      ignored: project.ignored,
      buildRoot: project.buildRoot,
      build: project.buildCommand ?? 'prism-code build',
      policy: policy ?? null,
    },
    levels: ASSURANCE.NAMES,
    limits: [...LIMITS],
    status: 'idle',
    run: 0,
    trigger: null,
    changed: [],
    build: null,
    verdict: null,
    noVerdict: null,
    reasons: [],
    check: null,
    replay: replay.available ? { available: true, histories: replay.histories ?? HISTORIES_DIR, files: [], result: null, note: null, error: null, ms: null } : { available: false, reason: replay.reason ?? REPLAY_UNAVAILABLE, histories: replay.histories ?? HISTORIES_DIR },
    timing: null,
    runs: [],
    updatedAt: null,
  };
  let pending = null;
  let timer = null;
  let running = null;
  let queued = false;
  let closed = false;

  const emit = () => {
    state.updatedAt = new Date().toISOString();
    events.emit('state', state);
  };

  async function run() {
    const burst = pending ?? { paths: new Set(), trigger: 'start', editAt: now(), wall: Date.now() };
    pending = null;
    state.run += 1;
    state.trigger = burst.trigger;
    state.changed = [...burst.paths].sort();
    state.status = 'building';
    state.build = null;
    state.check = null;
    state.timing = null;
    emit();
    const t0 = now();
    const built = await build({ directory: project.directory, command: project.buildCommand, run: state.run });
    const holo = built.ok ? await locate() : null;
    state.build = {
      ok: built.ok && holo !== null,
      code: built.code,
      ms: round(now() - t0),
      output: built.output,
      holo: holo === null ? null : POSIX(relative(project.directory, holo)),
      problem: built.ok ? (holo === null ? `the build exited 0 but no .holo is under ${project.buildRoot}/` : null) : `the build exited ${built.code ?? 'on a signal'}`,
    };
    if (!state.build.ok) {
      state.verdict = null;
      state.noVerdict = null;
      state.reasons = [];
      finish(burst, false);
      return;
    }
    state.status = 'checking';
    emit();
    const t1 = now();
    try {
      state.verdict = await judge(holo, { policy });
      state.noVerdict = null;
      state.reasons = state.verdict.reasons.map((r) => ({ ...r, title: titleOf(r) }));
    } catch (error) {
      if (!(error instanceof PrismError)) throw error;
      state.verdict = null;
      state.noVerdict = { check: error.check ?? error.code, code: error.code, message: error.message };
      state.reasons = [];
    }
    state.check = { ms: round(now() - t1) };
    const verdictAt = now();
    state.timing = { editAt: new Date(burst.wall).toISOString(), elapsedMs: round(verdictAt - burst.editAt), buildMs: state.build.ms, checkMs: state.check.ms };
    if (replay.available) {
      state.status = 'replaying';
      emit();
      const t2 = now();
      try {
        const outcome = await replay.run(holo);
        state.replay = { available: true, histories: replay.histories ?? HISTORIES_DIR, files: outcome.files, result: outcome.result, note: outcome.note ?? null, error: null, ms: round(now() - t2) };
      } catch (error) {
        if (!(error instanceof PrismError)) throw error;
        state.replay = { available: true, histories: replay.histories ?? HISTORIES_DIR, files: [], result: null, note: null, error: { check: error.check ?? error.code, code: error.code, message: error.message }, ms: round(now() - t2) };
      }
    }
    finish(burst, true);
  }

  function finish(burst, checked) {
    state.status = 'idle';
    state.runs.unshift({
      run: state.run,
      at: new Date(burst.wall).toISOString(),
      trigger: burst.trigger,
      changed: state.changed.length,
      built: state.build.ok,
      checked,
      passed: state.verdict?.passed ?? null,
      level: state.verdict?.level ?? null,
      noVerdict: state.noVerdict?.check ?? null,
      elapsedMs: state.timing?.elapsedMs ?? round(now() - burst.editAt),
      buildMs: state.build.ms,
      checkMs: state.check?.ms ?? null,
    });
    if (state.runs.length > HISTORY_RUNS) state.runs.length = HISTORY_RUNS;
    emit();
  }

  function start() {
    if (closed || running) return;
    running = run()
      .catch((error) => events.emit('error', error))
      .finally(() => {
        running = null;
        if (queued && !closed) {
          queued = false;
          start();
        }
      });
  }

  function fire() {
    timer = null;
    if (running) queued = true;
    else start();
  }

  return Object.freeze({
    state,
    on: (event, listener) => (events.on(event, listener), () => events.off(event, listener)),
    /** A burst member: a changed path (or none) and why. Runs once `debounceMs` after the last one. */
    trigger(paths = [], trigger = 'change') {
      if (closed) return;
      if (!pending) pending = { paths: new Set(), trigger, editAt: now(), wall: Date.now() };
      for (const path of paths) pending.paths.add(POSIX(path));
      if (state.status === 'idle' && !running && !timer) state.status = 'queued';
      if (timer) clearTimeout(timer);
      timer = setTimeout(fire, debounceMs);
    },
    /** Run now, with no debounce: the first run. Resolves when it has finished. */
    async runNow(trigger = 'start') {
      if (closed) return;
      if (!pending) pending = { paths: new Set(), trigger, editAt: now(), wall: Date.now() };
      if (timer) clearTimeout(timer);
      timer = null;
      if (running) queued = true;
      else start();
      await running;
    },
    /** True while a run is in flight or one is queued or debouncing. */
    get busy() {
      return running !== null || queued || timer !== null;
    },
    /** Resolves once nothing is in flight, queued or debouncing. */
    async settle() {
      while (running || timer || queued) {
        if (running) await running;
        else await new Promise((r) => setTimeout(r, Math.max(5, debounceMs / 4)));
      }
    },
    close() {
      closed = true;
      if (timer) clearTimeout(timer);
      timer = null;
      queued = false;
      events.removeAllListeners();
    },
  });
}

/**
 * Watch the project directory and feed the loop every path it rebuilds for.
 * Returns a function that stops watching.
 */
export function watchProject(project, loop, { onError = () => {} } = {}) {
  const watcher = watch(project.directory, { recursive: true }, (eventType, filename) => {
    if (!filename) return;
    const path = POSIX(String(filename));
    if (watches(project, path)) loop.trigger([path], eventType);
  });
  watcher.on('error', onError);
  return () => watcher.close();
}

/** The studio files, read once. */
async function readStudio(dir) {
  const files = new Map();
  for (const name of await readdir(dir)) {
    const type = CONTENT_TYPES[extname(name)];
    if (type) files.set(name, { type, bytes: await readFile(join(dir, name)) });
  }
  if (!files.has('index.html')) throw new PrismCodeError('dev.studio', `${dir} has no index.html`);
  return files;
}

// Everything the page loads comes from this server. The browser enforces it, so the studio cannot reach a CDN by accident.
const CSP = "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self' data:; font-src 'self'; base-uri 'none'; form-action 'none'";

/**
 * Serve the studio and the loop's state. Resolves to `{ server, url, close }`.
 * A port in use rejects with `dev.port`: the port is fixed on purpose.
 */
export async function createDevServer(loop, { port = DEFAULT_PORT, host = DEFAULT_HOST, studioDir = STUDIO_DIR } = {}) {
  const studio = await readStudio(studioDir);
  const clients = new Set();
  const json = (response, status, value) => {
    response.writeHead(status, { 'content-type': 'application/json; charset=utf-8', 'cache-control': 'no-store' });
    response.end(`${JSON.stringify(value, null, 2)}\n`);
  };
  const server = createServer((request, response) => {
    const url = new URL(request.url ?? '/', 'http://localhost');
    if (request.method === 'POST' && url.pathname === '/rebuild') {
      loop.trigger([], 'manual');
      json(response, 202, { queued: true, run: loop.state.run + 1 });
      return;
    }
    if (request.method !== 'GET' && request.method !== 'HEAD') {
      response.writeHead(405, { allow: 'GET, HEAD' }).end();
      return;
    }
    if (url.pathname === '/state') {
      json(response, 200, loop.state);
      return;
    }
    if (url.pathname === '/verdict') {
      if (!loop.state.verdict) {
        json(response, 404, { error: 'no verdict', status: loop.state.status, build: loop.state.build && { ok: loop.state.build.ok, problem: loop.state.build.problem }, noVerdict: loop.state.noVerdict });
        return;
      }
      response.writeHead(200, { 'content-type': 'application/json; charset=utf-8', 'cache-control': 'no-store' });
      response.end(verdictJson(loop.state.verdict));
      return;
    }
    if (url.pathname === '/events') {
      response.writeHead(200, { 'content-type': 'text/event-stream; charset=utf-8', 'cache-control': 'no-store', connection: 'keep-alive', 'x-accel-buffering': 'no' });
      response.write('retry: 1000\n\n');
      response.write(`event: state\ndata: ${JSON.stringify(loop.state)}\n\n`);
      clients.add(response);
      request.on('close', () => clients.delete(response));
      return;
    }
    const name = url.pathname === '/' ? 'index.html' : decodeURIComponent(url.pathname.slice(1));
    const file = studio.get(name);
    if (!file) {
      response.writeHead(404, { 'content-type': 'text/plain; charset=utf-8' }).end('not found');
      return;
    }
    const headers = { 'content-type': file.type, 'content-length': file.bytes.length, 'cache-control': 'no-store' };
    if (name === 'index.html') headers['content-security-policy'] = CSP;
    response.writeHead(200, headers);
    response.end(request.method === 'HEAD' ? undefined : file.bytes);
  });
  const off = loop.on('state', (state) => {
    const frame = `event: state\ndata: ${JSON.stringify(state)}\n\n`;
    for (const client of clients) client.write(frame);
  });
  const heartbeat = setInterval(() => {
    for (const client of clients) client.write(': ping\n\n');
  }, HEARTBEAT_MS);
  heartbeat.unref();
  await new Promise((listening, reject) => {
    server.once('error', (error) => {
      clearInterval(heartbeat);
      off();
      if (error.code === 'EADDRINUSE') reject(new PrismCodeError('dev.port', `port ${port} on ${host} is in use; the studio's port is fixed, so stop what holds it or pass --port <n>`, { port, host }));
      else reject(error);
    });
    server.listen(port, host, listening);
  });
  const address = server.address();
  return {
    server,
    url: `http://${host}:${address.port}/`,
    close: () =>
      new Promise((done) => {
        clearInterval(heartbeat);
        off();
        for (const client of clients) client.end();
        clients.clear();
        server.close(() => done());
      }),
  };
}

const seconds = (ms) => (ms === null || ms === undefined ? '-' : `${(ms / 1000).toFixed(2)} s`);

/** One line per run, for the terminal. */
export function formatRun(state) {
  const head = `run ${state.run}`;
  const why = state.trigger === 'start' ? 'start' : state.changed.length > 0 ? `${state.changed.length} changed: ${state.changed.slice(0, 3).join(', ')}${state.changed.length > 3 ? ', ...' : ''}` : state.trigger;
  if (!state.build?.ok) {
    const tail = state.build?.output?.trim().split('\n').slice(-5).map((l) => `    ${l}`).join('\n');
    return `${head}  BUILD FAILED  ${state.build?.problem ?? ''} after ${seconds(state.build?.ms)} (${why})${tail ? `\n${tail}` : ''}\n`;
  }
  if (state.noVerdict) return `${head}  NO VERDICT  ${state.noVerdict.check}: ${state.noVerdict.message} (${why})\n`;
  const v = state.verdict;
  const replay = state.replay.available && state.replay.result ? `, replay ${state.replay.result.divergenceCount === 0 ? 'PASS' : `FAIL ${state.replay.result.divergenceCount}`}` : '';
  return `${head}  ${v.passed ? 'PASSED' : 'FAILED'}  ${v.level} ${ASSURANCE.NAMES[v.level]}  policy ${v.policy.name}  edit to verdict ${seconds(state.timing.elapsedMs)} (build ${seconds(state.timing.buildMs)}, check ${seconds(state.timing.checkMs)}${replay}; ${why})\n`;
}
