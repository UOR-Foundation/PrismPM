// `prism-code dev` (issue #288): the watch-rebuild-recheck loop and the offline studio it serves.
// Every test runs the loop on a stub builder that copies the reference holo into the build root;
// a real `prismpm build` needs the toolchain, so it stays a manual pass.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawn, spawnSync } from 'node:child_process';
import { createServer } from 'node:http';
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { check, verdictJson, LIMITS } from '../src/check.mjs';
import { PrismCodeError } from '../src/errors.mjs';
import {
  DEFAULT_PORT,
  DEFAULT_WATCH,
  PROJECT_FILES,
  REPLAY_UNAVAILABLE,
  STUDIO_DIR,
  createDevLoop,
  createDevServer,
  describeProject,
  formatRun,
  globToRegExp,
  locateHolo,
  replaySupport,
  watchProject,
  watches,
} from '../src/dev.mjs';

const here = (relative) => fileURLToPath(new URL(relative, import.meta.url));
const bin = here('../bin/prism-code.mjs');
const fixture = here('../fixtures/Calculator.holo');
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

/** A project directory with a prism.toml, a model file and a stub build script that copies the reference holo. */
function project({ manifest = true } = {}) {
  const dir = mkdtempSync(join(tmpdir(), 'prism-dev-'));
  mkdirSync(join(dir, 'src'), { recursive: true });
  writeFileSync(join(dir, 'src', 'App.lex.tex'), '% model\n');
  // The stub build: `prismpm build` writes <build_root>/build/<id>/<Name>.holo; this writes the same layout.
  writeFileSync(join(dir, 'build.mjs'), `import { mkdirSync, copyFileSync, appendFileSync } from 'node:fs';\nmkdirSync('.prism/build/stub', { recursive: true });\ncopyFileSync(${JSON.stringify(fixture)}, '.prism/build/stub/Calculator.holo');\nappendFileSync('builds.log', 'build\\n');\n`);
  if (manifest) {
    writeFileSync(join(dir, 'prism.toml'), ['[application]', 'name = "studio-test"', 'version = "0.1.0"', '', '[model]', 'source = "src"', 'watch = ["src/**/*.lex.tex", "language/**/*.toml"]', `build = "node build.mjs"`, ''].join('\n'));
  }
  return dir;
}

const builds = (dir) => {
  try {
    return readFileSync(join(dir, 'builds.log'), 'utf8').split('\n').filter(Boolean).length;
  } catch {
    return 0;
  }
};

/** The loop over a project, with a stub builder that counts its calls and copies the holo. */
function stubLoop(dir, options = {}) {
  let calls = 0;
  const build = async () => {
    calls += 1;
    mkdirSync(join(dir, '.prism', 'build', 'stub'), { recursive: true });
    copyFileSync(fixture, join(dir, '.prism', 'build', 'stub', 'Calculator.holo'));
    await sleep(options.buildMs ?? 20);
    return { ok: true, code: 0, output: 'built\n' };
  };
  return { build, calls: () => calls };
}

test('globs from prism.toml match project-relative paths; ** spans directories', () => {
  const re = globToRegExp('src/**/*.lex.tex');
  for (const path of ['src/A.lex.tex', 'src/Foundation/View/Model.lex.tex']) assert.ok(re.test(path), path);
  for (const path of ['A.lex.tex', 'src/A.tex', 'other/src/A.lex.tex', 'src/A.lex.tex.bak']) assert.ok(!re.test(path), path);
  assert.ok(globToRegExp('language/**/*.toml').test('language/prism.arch/entries/arch-view.toml'));
  assert.ok(globToRegExp('src/**').test('src/deep/er/file'));
  assert.ok(globToRegExp('intent.md').test('intent.md') && !globToRegExp('intent.md').test('x/intent.md'));
  assert.ok(globToRegExp('examples/*.json').test('examples/a.json') && !globToRegExp('examples/*.json').test('examples/sub/a.json'));
});

test('a project is described from prism.toml: watch globs, build command and the build root; the build tree is never watched', async () => {
  const dir = project();
  writeFileSync(join(dir, 'prismpm.toml'), 'spec = "prismpm/project/1"\nproject = "StudioTest"\nbuild_root = "out"\n');
  const p = await describeProject(dir);
  assert.equal(p.name, 'studio-test');
  assert.deepEqual(p.manifest, { name: 'studio-test', version: '0.1.0' });
  assert.deepEqual(p.watch, ['src/**/*.lex.tex', 'language/**/*.toml']);
  assert.equal(p.buildCommand, 'node build.mjs');
  assert.equal(p.buildRoot, 'out');
  assert.ok(p.ignored.includes('out') && p.ignored.includes('.git'));
  assert.ok(watches(p, 'src/App.lex.tex'));
  assert.ok(watches(p, 'prism.toml') && watches(p, 'lexlean.lock') && watches(p, 'prism.browser.toml'));
  assert.ok(!watches(p, 'out/build/x/Calculator.holo'), 'the build root is not watched, or every build would rebuild');
  assert.ok(!watches(p, '.git/index') && !watches(p, 'README.md') && !watches(p, 'src/notes.txt'));
  assert.ok(watches(p, join('src', 'deep', 'B.lex.tex')), 'platform separators are normalized');

  const bare = await describeProject(project({ manifest: false }));
  assert.equal(bare.manifest, null);
  assert.deepEqual(bare.watch, [...DEFAULT_WATCH]);
  assert.equal(bare.buildCommand, null, 'no manifest: the build is prism-code build');
  assert.equal(bare.buildRoot, '.prism');
  assert.deepEqual(bare.files, [...PROJECT_FILES]);

  const broken = project();
  writeFileSync(join(broken, 'prism.toml'), '[application]\nname = "Bad Name"\nversion = "0.1.0"\n');
  const b = await describeProject(broken);
  assert.equal(b.manifestError.code, 'manifest.invalid');
  assert.deepEqual(b.watch, [...DEFAULT_WATCH], 'a broken manifest keeps the defaults so the fix is seen');
});

test('the newest holo under the build root is the one checked', async () => {
  const dir = project();
  assert.equal(await locateHolo(dir), null);
  mkdirSync(join(dir, '.prism', 'build', 'old'), { recursive: true });
  mkdirSync(join(dir, '.prism', 'build', 'new'), { recursive: true });
  copyFileSync(fixture, join(dir, '.prism', 'build', 'old', 'Calculator.holo'));
  await sleep(30);
  copyFileSync(fixture, join(dir, '.prism', 'build', 'new', 'Calculator.holo'));
  assert.equal(await locateHolo(dir), join(dir, '.prism', 'build', 'new', 'Calculator.holo'));
});

test('the loop builds, locates the holo and checks it: the verdict is the one check returns, with its timing', async () => {
  const dir = project();
  const p = await describeProject(dir);
  const stub = stubLoop(dir);
  const loop = createDevLoop({ project: p, build: stub.build, debounceMs: 30 });
  const states = [];
  loop.on('state', (s) => states.push(s.status));
  await loop.runNow();
  const { state } = loop;
  assert.equal(stub.calls(), 1);
  assert.deepEqual(states, ['building', 'checking', 'idle']);
  assert.equal(state.run, 1);
  assert.equal(state.trigger, 'start');
  assert.ok(state.build.ok);
  assert.equal(state.build.holo, '.prism/build/stub/Calculator.holo');
  assert.deepEqual(state.verdict, await check(join(dir, '.prism', 'build', 'stub', 'Calculator.holo')));
  assert.equal(state.verdict.level, 'P0');
  assert.equal(state.verdict.passed, false, 'every holo today is P0, and the default policy wants P2');
  assert.deepEqual(state.reasons.map((r) => [r.check, r.title]), [
    ['provenance.statement', 'Verification'],
    ['proof.statement', 'ProofMismatch'],
    ['replay.statement', 'Replay'],
    ['policy.level', 'Verification'],
  ]);
  assert.ok(state.timing.elapsedMs >= state.timing.buildMs, 'edit to verdict covers the build');
  assert.ok(state.timing.elapsedMs >= 20 && state.timing.buildMs >= 20, `the stub build sleeps 20 ms: ${JSON.stringify(state.timing)}`);
  assert.ok(typeof state.timing.checkMs === 'number');
  assert.deepEqual(state.limits, [...LIMITS]);
  assert.deepEqual(state.replay, { available: false, reason: REPLAY_UNAVAILABLE, histories: 'histories' });
  assert.equal(state.runs.length, 1);
  assert.deepEqual([state.runs[0].run, state.runs[0].passed, state.runs[0].level, state.runs[0].built], [1, false, 'P0', true]);
  assert.match(formatRun(state), /^run 1 {2}FAILED {2}P0 unverified {2}policy default {2}edit to verdict \d+\.\d\d s \(build \d+\.\d\d s, check \d+\.\d\d s; start\)\n$/);
  loop.close();
});

test('a policy the loop is given reaches check, and a failed build or a no-verdict is reported, not thrown', async () => {
  const dir = project();
  const p = await describeProject(dir);
  const policy = join(dir, 'p0.json');
  writeFileSync(policy, JSON.stringify({ name: 'p0', minimum: 'P0' }));
  const stub = stubLoop(dir);
  const passing = createDevLoop({ project: p, build: stub.build, policy, debounceMs: 30 });
  await passing.runNow();
  assert.deepEqual([passing.state.verdict.passed, passing.state.verdict.policy], [true, { name: 'p0', minimum: 'P0' }]);
  assert.match(formatRun(passing.state), /^run 1 {2}PASSED {2}P0 unverified {2}policy p0 /);
  passing.close();

  const failing = createDevLoop({ project: p, build: async () => ({ ok: false, code: 3, output: 'line one\nerror: no such model\n' }), debounceMs: 30 });
  await failing.runNow();
  assert.deepEqual([failing.state.status, failing.state.build.ok, failing.state.build.code, failing.state.verdict], ['idle', false, 3, null]);
  assert.equal(failing.state.build.problem, 'the build exited 3');
  assert.match(formatRun(failing.state), /^run 1 {2}BUILD FAILED {2}the build exited 3 after \d+\.\d\d s \(start\)\n {4}line one\n {4}error: no such model\n$/);
  failing.close();

  const silent = createDevLoop({ project: p, build: async () => ({ ok: true, code: 0, output: '' }), locate: async () => null, debounceMs: 30 });
  await silent.runNow();
  assert.equal(silent.state.build.problem, 'the build exited 0 but no .holo is under .prism/');
  silent.close();

  const noVerdict = createDevLoop({ project: p, build: stub.build, policy: join(dir, 'missing.json'), debounceMs: 30 });
  await noVerdict.runNow();
  assert.equal(noVerdict.state.verdict, null);
  assert.equal(noVerdict.state.noVerdict.check, 'policy.unreadable');
  assert.match(formatRun(noVerdict.state), /^run 1 {2}NO VERDICT {2}policy\.unreadable: /);
  noVerdict.close();
});

test('a burst of changes is one rebuild, and a change during a run queues exactly one more', async () => {
  const dir = project();
  const p = await describeProject(dir);
  const stub = stubLoop(dir, { buildMs: 120 });
  const loop = createDevLoop({ project: p, build: stub.build, debounceMs: 60 });
  for (let i = 0; i < 12; i++) {
    loop.trigger([`src/File${i}.lex.tex`]);
    await sleep(5);
  }
  assert.ok(loop.busy);
  await loop.settle();
  assert.equal(stub.calls(), 1, 'one burst, one build');
  assert.equal(loop.state.changed.length, 12);
  assert.equal(loop.state.trigger, 'change');
  assert.ok(loop.state.timing.elapsedMs >= 120, 'edit to verdict is measured from the first change of the burst');

  // Changes while a run is in flight: the run finishes, then exactly one more runs with every path of the new burst.
  loop.trigger(['src/A.lex.tex']);
  await sleep(70);
  assert.equal(loop.state.status, 'building');
  for (let i = 0; i < 5; i++) {
    loop.trigger([`src/B${i}.lex.tex`]);
    await sleep(5);
  }
  await loop.settle();
  assert.equal(stub.calls(), 3);
  assert.deepEqual(loop.state.changed, ['src/B0.lex.tex', 'src/B1.lex.tex', 'src/B2.lex.tex', 'src/B3.lex.tex', 'src/B4.lex.tex']);
  assert.equal(loop.state.runs.length, 3);
  loop.close();
});

test('the watcher feeds the loop only the paths the project watches: one rebuild per burst of writes', async () => {
  const dir = project();
  const p = await describeProject(dir);
  const stub = stubLoop(dir, { buildMs: 50 });
  const loop = createDevLoop({ project: p, build: stub.build, debounceMs: 200 });
  const stop = watchProject(p, loop);
  try {
    await sleep(100);
    for (let i = 0; i < 6; i++) writeFileSync(join(dir, 'src', `W${i}.lex.tex`), `% ${i}\n`);
    writeFileSync(join(dir, 'src', 'notes.txt'), 'not a model file\n');
    mkdirSync(join(dir, '.prism', 'scratch'), { recursive: true });
    writeFileSync(join(dir, '.prism', 'scratch', 'Out.holo'), 'not watched\n');
    await sleep(600);
    await loop.settle();
    assert.equal(stub.calls(), 1, 'six writes in one burst are one rebuild');
    assert.ok(loop.state.changed.every((c) => /^src\/W\d\.lex\.tex$/.test(c)), `only model files: ${loop.state.changed.join(', ')}`);
    assert.ok(loop.state.changed.length >= 1);

    writeFileSync(join(dir, 'prism.toml'), readFileSync(join(dir, 'prism.toml'), 'utf8'));
    await sleep(500);
    await loop.settle();
    assert.equal(stub.calls(), 2, 'prism.toml is watched');
    assert.deepEqual(loop.state.changed, ['prism.toml']);
  } finally {
    stop();
    loop.close();
  }
});

/** Start a server on an ephemeral port for a loop that has run once. */
async function served(dir, options = {}) {
  const p = await describeProject(dir);
  const stub = stubLoop(dir);
  const loop = createDevLoop({ project: p, build: stub.build, debounceMs: 30, ...options });
  await loop.runNow();
  const server = await createDevServer(loop, { port: 0 });
  return { loop, server, close: () => server.close().then(() => loop.close()) };
}

test('/verdict is the canonical verdict JSON, byte for byte what check --json prints', async () => {
  const dir = project();
  const { loop, server, close } = await served(dir);
  try {
    const response = await fetch(new URL('verdict', server.url));
    assert.equal(response.status, 200);
    assert.equal(response.headers.get('content-type'), 'application/json; charset=utf-8');
    const body = await response.text();
    assert.equal(body, verdictJson(loop.state.verdict));
    const cli = spawnSync(process.execPath, [bin, 'check', join(dir, '.prism', 'build', 'stub', 'Calculator.holo'), '--json'], { encoding: 'utf8', env: { ...process.env, PRISM_MINIMUM_LEVEL: '' } });
    assert.equal(body, cli.stdout, 'the studio and the CLI print one verdict');
    const verdict = JSON.parse(body);
    assert.equal(verdict.schema, 'prism/verdict/1');
    assert.deepEqual(Object.keys(verdict), [...Object.keys(verdict)].sort(), 'canonical key order');

    const state = await (await fetch(new URL('state', server.url))).json();
    assert.deepEqual(state.verdict, verdict);
    assert.equal(state.project.name, 'studio-test');
    assert.deepEqual(state.limits, [...LIMITS]);
  } finally {
    await close();
  }
});

test('/verdict is 404 until there is one', async () => {
  const dir = project();
  const p = await describeProject(dir);
  const loop = createDevLoop({ project: p, build: async () => ({ ok: false, code: 1, output: 'nope\n' }), debounceMs: 30 });
  await loop.runNow();
  const server = await createDevServer(loop, { port: 0 });
  try {
    const response = await fetch(new URL('verdict', server.url));
    assert.equal(response.status, 404);
    const body = await response.json();
    assert.deepEqual(body, { error: 'no verdict', status: 'idle', build: { ok: false, problem: 'the build exited 1' }, noVerdict: null });
  } finally {
    await server.close();
    loop.close();
  }
});

test('the server refuses a busy port instead of picking another: dev.port, and the CLI exits 69', async () => {
  const dir = project();
  const holder = createServer((_, res) => res.end());
  await new Promise((r) => holder.listen(0, '127.0.0.1', r));
  const { port } = holder.address();
  try {
    const p = await describeProject(dir);
    const loop = createDevLoop({ project: p, build: stubLoop(dir).build });
    await assert.rejects(createDevServer(loop, { port }), (error) => error instanceof PrismCodeError && error.code === 'dev.port' && error.details.port === port && /port \d+ on 127\.0\.0\.1 is in use/.test(error.message));
    loop.close();
    const cli = spawnSync(process.execPath, [bin, 'dev', '--port', String(port)], { cwd: dir, encoding: 'utf8' });
    assert.equal(cli.status, 69, cli.stderr);
    assert.match(cli.stderr, /^dev\.port: port \d+ on 127\.0\.0\.1 is in use; the studio's port is fixed/);
    assert.equal(builds(dir), 0, 'nothing was built');
  } finally {
    await new Promise((r) => holder.close(r));
  }
});

test('the studio is static files from this process with no external URL, and the page forbids every other origin', async () => {
  const files = readdirSync(STUDIO_DIR).sort();
  assert.deepEqual(files, ['index.html', 'studio.css', 'studio.js']);
  for (const name of files) {
    const text = readFileSync(join(STUDIO_DIR, name), 'utf8');
    assert.doesNotMatch(text, /https?:\/\//i, `${name} names a URL`);
    assert.doesNotMatch(text, /\/\/(?:cdn|unpkg|jsdelivr|fonts)/i, `${name} names a CDN`);
    assert.doesNotMatch(text, /<script[^>]*\ssrc=["'](?!studio\.js["'])/, `${name} loads a script from elsewhere`);
    assert.doesNotMatch(text, /@import|url\(/, `${name} imports or loads a resource`);
  }
  const html = readFileSync(join(STUDIO_DIR, 'index.html'), 'utf8');
  assert.doesNotMatch(html, /<script(?![^>]*\ssrc=)[^>]*>[^<]/, 'no inline script: the policy forbids it');
  assert.doesNotMatch(html, /\sstyle=|<style/, 'no inline style: the policy forbids it');
  assert.ok(html.includes('<link rel="stylesheet" href="studio.css">') && html.includes('<script src="studio.js">'));

  const dir = project();
  const { server, close } = await served(dir);
  try {
    const page = await fetch(server.url);
    assert.equal(page.status, 200);
    assert.equal(page.headers.get('content-type'), 'text/html; charset=utf-8');
    assert.match(page.headers.get('content-security-policy'), /^default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self';/);
    assert.equal(await page.text(), html);
    for (const [name, type] of [
      ['studio.css', 'text/css; charset=utf-8'],
      ['studio.js', 'text/javascript; charset=utf-8'],
    ]) {
      const r = await fetch(new URL(name, server.url));
      assert.equal(r.status, 200, name);
      assert.equal(r.headers.get('content-type'), type);
      assert.equal(await r.text(), readFileSync(join(STUDIO_DIR, name), 'utf8'));
    }
    assert.equal((await fetch(new URL('nope.js', server.url))).status, 404);
    assert.equal((await fetch(new URL('state', server.url), { method: 'PUT' })).status, 405);
  } finally {
    await close();
  }
});

test('/events streams the state over Server-Sent Events: the current one at once, then every change', async () => {
  const dir = project();
  const { loop, server, close } = await served(dir);
  try {
    const response = await fetch(new URL('events', server.url), { headers: { accept: 'text/event-stream' } });
    assert.equal(response.headers.get('content-type'), 'text/event-stream; charset=utf-8');
    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    let buffer = '';
    const events = [];
    const read = async (count) => {
      while (events.length < count) {
        const { value, done } = await reader.read();
        if (done) break;
        buffer += decoder.decode(value, { stream: true });
        let end;
        while ((end = buffer.indexOf('\n\n')) >= 0) {
          const frame = buffer.slice(0, end);
          buffer = buffer.slice(end + 2);
          const data = frame.split('\n').find((l) => l.startsWith('data: '));
          if (frame.startsWith('event: state') && data) events.push(JSON.parse(data.slice(6)));
        }
      }
    };
    await read(1);
    assert.equal(events[0].run, 1);
    assert.equal(events[0].status, 'idle');
    assert.deepEqual(events[0].verdict, loop.state.verdict);

    const queued = await fetch(new URL('rebuild', server.url), { method: 'POST' });
    assert.equal(queued.status, 202);
    assert.deepEqual(await queued.json(), { queued: true, run: 2 });
    await read(4);
    assert.deepEqual(events.slice(1).map((e) => [e.run, e.status]), [[2, 'building'], [2, 'checking'], [2, 'idle']]);
    assert.equal(events.at(-1).trigger, 'manual');
    await reader.cancel();
  } finally {
    await close();
  }
});

test('replay is read through a feature check: absent in this build, the tab says so; present, its result is served', async () => {
  const dir = project();
  const absent = await replaySupport(dir);
  assert.deepEqual(absent, { available: false, reason: REPLAY_UNAVAILABLE, histories: 'histories' });

  // The surface PR 415 adds: replay-cli.mjs's replayFiles(app, paths) resolving to { result, files }, a prismpm/replay-result/1.
  const result = { application: 'Calculator', divergenceCount: 0, events: 22, histories: [{ application: 'Calculator', digest: { blake3: 'a'.repeat(64), sha256: 'b'.repeat(64) }, divergences: [], events: 22 }], runtime: { name: 'prism-code', version: '0.3.0' }, statement: null };
  const calls = [];
  const modules = [
    { replayFiles: async (app, paths) => (calls.push([app.kind, paths]), { result, files: paths.map((p) => join(p, 'Calculator.history.json')) }) },
    { fromFile: async (holo) => ({ kind: 'holo', holo }) },
  ];
  const noHistories = await replaySupport(dir, { modules });
  assert.equal(noHistories.available, true);
  assert.deepEqual(await noHistories.run(fixture), { files: [], result: null, note: 'no history: put one or more prism/history/1 files in histories/' });
  assert.equal(calls.length, 0, 'nothing is opened when there is no history');

  mkdirSync(join(dir, 'histories'));
  writeFileSync(join(dir, 'histories', 'Calculator.history.json'), '{}');
  const { loop, server, close } = await served(dir, { replay: await replaySupport(dir, { modules }) });
  try {
    assert.deepEqual(calls, [['holo', [join(dir, 'histories')]]]);
    const { replay } = loop.state;
    assert.deepEqual([replay.available, replay.files, replay.result, replay.error, replay.note], [true, ['histories/Calculator.history.json'], result, null, null]);
    assert.ok(typeof replay.ms === 'number');
    assert.match(formatRun(loop.state), /, replay PASS; start\)\n$/);
    const state = await (await fetch(new URL('state', server.url))).json();
    assert.deepEqual(state.replay.result, result);
  } finally {
    await close();
  }
});

test('the CLI: dev runs the manifest build, checks the holo, prints the run and its timing, and serves the studio', async () => {
  const dir = project();
  const child = spawn(process.execPath, [bin, 'dev', '--port', '0'], { cwd: dir, stdio: ['ignore', 'pipe', 'pipe'], env: { ...process.env, PRISM_MINIMUM_LEVEL: '' } });
  let out = '';
  let err = '';
  child.stderr.on('data', (c) => (err += c));
  const url = await new Promise((resolve, reject) => {
    child.stdout.on('data', (chunk) => {
      out += chunk;
      const match = /^run 1 {2}FAILED {2}P0 unverified {2}policy default {2}edit to verdict (\d+\.\d\d) s \(build (\d+\.\d\d) s, check (\d+\.\d\d) s; start\)$/m.exec(out);
      const studio = /^studio at (http:\/\/127\.0\.0\.1:\d+\/) /m.exec(out);
      if (match && studio) resolve(studio[1]);
    });
    child.on('exit', (code) => reject(new Error(`exited ${code} early: ${out}${err}`)));
  });
  try {
    assert.match(out, /^studio-test in /m);
    assert.match(out, /^watching: prism\.toml, prism\.browser\.toml, prism\.node\.toml, prismpm\.toml, lexlean\.toml, lexlean\.lock, prismpm\.lock, src\/\*\*\/\*\.lex\.tex, language\/\*\*\/\*\.toml \(not \.git, node_modules, \.lake, \.lexlean, \.prism\)$/m);
    assert.match(out, /^build: node build\.mjs; check: policy default$/m);
    assert.match(out, /^replay: replay not available in this build$/m);
    assert.equal(builds(dir), 1);
    const verdict = await (await fetch(new URL('verdict', url))).text();
    assert.equal(verdict, verdictJson(await check(join(dir, '.prism', 'build', 'stub', 'Calculator.holo'))));
    assert.equal(err, '');
  } finally {
    child.kill();
    await new Promise((r) => child.on('exit', r));
  }
});

test('usage errors exit 64, a bad policy exits 2 before anything runs, and the default port is documented', () => {
  const dir = project();
  const run = (args) => spawnSync(process.execPath, [bin, 'dev', ...args], { cwd: dir, encoding: 'utf8' });
  assert.equal(run(['--frobnicate']).status, 64);
  assert.match(run(['extra']).stderr, /dev takes no argument/);
  assert.equal(run(['--port', '70000']).status, 64);
  assert.equal(run(['--policy']).status, 64);
  const bad = run(['--policy', join(dir, 'missing.json')]);
  assert.equal(bad.status, 2);
  assert.match(bad.stderr, /^no verdict: policy\.unreadable: /);
  assert.equal(builds(dir), 0);
  assert.equal(DEFAULT_PORT, 4100);
  const help = spawnSync(process.execPath, [bin, '--help'], { encoding: 'utf8' }).stdout;
  assert.match(help, /prism-code dev \[--port <n>\] \[--policy <file>\]/);
  assert.match(help, /http:\/\/127\.0\.0\.1:4100\//);
});
