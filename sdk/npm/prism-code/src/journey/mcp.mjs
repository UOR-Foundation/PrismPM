/**
 * `prism-code mcp`: the Prism MCP server, over stdio, with no dependencies.
 *
 * It owns the whole journey: the greeting, the spec session, the model,
 * the proof and the build. An agent only renders what it returns. Proving
 * and building take minutes, longer than some agents wait for one tool call,
 * so both run as jobs: a call waits up to `wait` seconds, and a later call
 * of the same tool picks the job up where it is.
 */
import { createInterface } from 'node:readline';
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { createRequire } from 'node:module';
import { SpecError, validateSpec } from './spec.mjs';
import { answersFromSpec, nextTurn, STEPS } from './session.mjs';
import { appDir, modelPath, readSession, writeSession, writeSpecDocument } from './project.mjs';
import { build, prove, tryRequest } from './engine.mjs';

const require = createRequire(import.meta.url);
const { version } = require('../../package.json');

export const PROTOCOL_VERSIONS = ['2025-06-18', '2025-03-26', '2024-11-05'];

export const INSTRUCTIONS = `Prism builds formally verified apps from a spec. When the user asks for Prism, a verified app, or types /prism, call prism_start and follow what it returns.
Ask the user each question prism_start or prism_answer returns, one at a time, in your own words, and pass their answer to prism_answer. Use the example to show the format. When the user answers in plain language, rewrite it into the format yourself and say what you wrote.
When a result has an issue, explain it in one or two sentences and ask the user how to resolve it.
Never edit files under prism/ by hand. Change the app only through prism_answer or prism_spec.
prism_prove and prism_build take minutes. When they return state "running", tell the user what is happening and call the same tool again.Say "proved" only for what Lean proved, and "tested" for what acceptance vectors cover.`;

const GREETING = 'Prism turns a short spec into an app whose rules are proved by Lean, not just tested. I will ask six short questions, then prove and build it.';

const TOOLS = [
  {
    name: 'prism_start',
    description: 'Start or resume the Prism session in this project. Returns a short greeting and the next question to ask the user. Call it first, and whenever you lose track.',
    inputSchema: { type: 'object', properties: { restart: { type: 'boolean', description: 'Discard the answers so far and start over.' } }, additionalProperties: false },
  },
  {
    name: 'prism_answer',
    description: 'Record the user\'s answer to the current question and get the next one. Pass step to change an earlier answer (purpose, fields, outcomes, rules, properties, examples, review). When the result has an issue, ask the user about it and answer the same step again.',
    inputSchema: {
      type: 'object',
      properties: { answer: { type: 'string', description: 'The answer, in the format the question shows.' }, step: { type: 'string', enum: STEPS, description: 'The step this answers; default the current one.' } },
      required: ['answer'],
      additionalProperties: false,
    },
  },
  {
    name: 'prism_spec',
    description: 'Read the whole spec as prism/spec/1 JSON, or replace it in one call when the user already gave everything. A replaced spec is checked exactly as answers are.',
    inputSchema: { type: 'object', properties: { spec: { type: 'object', description: 'A whole prism/spec/1 spec. Omit to read.' } }, additionalProperties: false },
  },
  {
    name: 'prism_prove',
    description: 'Write the LexLean model and ask Lean to prove every property: in the SDK image when it is on this machine, else on the hosted verifier at https://mcp.uor.foundation/, which receives the files under prism/src/ and stores them unlisted (not searchable, but readable by anyone with its draft ID). Takes one to five minutes. Returns state running while it works; call again to keep waiting.',
    inputSchema: { type: 'object', properties: { wait: { type: 'number', description: 'Seconds to wait before returning, default 45.' }, overwrite: { type: 'boolean', description: 'Replace a model that was edited by hand.' } }, additionalProperties: false },
  },
  {
    name: 'prism_build',
    description: 'Prove, then build the app into a holo and replay its acceptance vectors through the built guest. Where it builds is chosen for the user: in the SDK image when it is on this machine, else on GitHub Actions when the user is signed in to GitHub (no Docker here; this machine verifies what comes back), else in the SDK image after downloading it (about 5.5 GB). A GitHub build of an app with no repository returns needs "repository" and a suggested name: ask the user to approve creating that public repository, then call again with repository set. Prism then commits only its own files (the app folder, the build workflow, the SDK lock), creates the repository and builds. A GitHub build takes five to fifteen minutes. Returns state running while it works; call again to keep waiting.',
    inputSchema: {
      type: 'object',
      properties: {
        wait: { type: 'number', description: 'Seconds to wait before returning, default 45.' },
        overwrite: { type: 'boolean', description: 'Replace a model that was edited by hand.' },
        remote: { type: 'string', enum: ['github', 'local'], description: 'Where to build, when the user asks for one: github (the repository\'s Actions) or local (the SDK image). Omit to let Prism choose.' },
        repository: { type: 'string', description: 'owner/name of the GitHub repository to create for the app, only after the user approved it.' },
      },
      additionalProperties: false,
    },
  },
  {
    name: 'prism_try',
    description: 'Answer one request, from the built app when there is one, else from the spec, and say which rule decided it.',
    inputSchema: { type: 'object', properties: { request: { type: 'string', description: 'One request line, the fields separated by single spaces.' } }, required: ['request'], additionalProperties: false },
  },
  {
    name: 'prism_status',
    description: 'Where the session is: the next question, what is proved, what is built, and any job in progress.',
    inputSchema: { type: 'object', properties: {}, additionalProperties: false },
  },
];

const PROMPTS = [{ name: 'prism', description: 'Build a formally verified app with Prism', arguments: [] }];

/** A server bound to a working directory. `send` writes one JSON-RPC message. */
/** Whether a process still runs. */
const alive = (pid) => {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    return error.code === 'EPERM';
  }
};

export function createServer({ cwd = process.cwd(), send }) {
  const dir = appDir(cwd);
  const jobs = new Map();

  const load = () => readSession(dir);
  const save = (session) => writeSession(dir, session);

  function turn(session) {
    const t = nextTurn(session.answers);
    session.step = t.step;
    if (t.spec) session.spec = t.spec;
    save(session);
    if (session.spec) writeSpecDocument(dir, session);
    const { spec, ...rest } = t;
    return rest;
  }

  function startJob(kind, run) {
    const job = { kind, state: 'running', log: [], started: Date.now(), result: null };
    const log = (line) => {
      job.log.push(line);
      if (job.log.length > 200) job.log.shift();
      job.onLog?.(line);
    };
    job.promise = run(log)
      .then((result) => Object.assign(job, { state: 'done', result }))
      .catch((error) => Object.assign(job, { state: 'done', result: { ok: false, message: `Prism failed: ${error.message}` } }));
    jobs.set(kind, job);
    return job;
  }

  async function awaitJob(job, waitSeconds, progress) {
    const timer = new Promise((done) => setTimeout(done, Math.max(0, Math.min(waitSeconds, 600)) * 1000));
    let tick = 0;
    job.onLog = (line) => progress?.(++tick, line);
    await Promise.race([job.promise, timer]);
    job.onLog = null;
    const elapsed = Math.round((Date.now() - job.started) / 1000);
    if (job.state === 'running') return { state: 'running', elapsed, latest: job.log.slice(-3), next: `Still working after ${elapsed} s. Call ${job.kind === 'build' ? 'prism_build' : 'prism_prove'} again to keep waiting.` };
    jobs.delete(job.kind);
    return { state: 'done', elapsed, ...job.result };
  }

  async function longTool(kind, args, progress) {
    const session = load();
    if (!session.spec || session.step !== 'ready') {
      const t = nextTurn(session.answers);
      if (t.step !== 'ready') return { state: 'blocked', message: `The spec is not complete yet. Next question: ${t.question}`, step: t.step };
      session.spec = t.spec;
    }
    const other = jobs.get(kind === 'build' ? 'prove' : 'build');
    if (other) return { state: 'busy', message: `A ${other.kind} is running; call prism_${other.kind} to follow it.` };
    // A job another Prism server started (an agent that restarted its server) is not
    // started twice; that server finishes it and saves the result here.
    const elsewhere = session.status.job;
    if (!jobs.get(kind) && elsewhere && elsewhere.pid !== process.pid && alive(elsewhere.pid)) {
      const elapsed = Math.round((Date.now() - elsewhere.started) / 1000);
      return { state: 'running', elapsed, next: `A ${elsewhere.kind} another Prism server started ${elapsed} s ago is still running. Call prism_${kind} again to keep waiting; it returns the result when that one finishes.` };
    }
    if (!jobs.get(kind) && kind === 'build' && session.status.lastBuild && session.status.lastBuild.modelSha === session.status.modelSha && session.status.lastSpec === JSON.stringify(session.spec) && existsSync(session.status.lastBuild.holo ?? '')) {
      return { state: 'done', elapsed: 0, ...session.status.lastBuild.result, note: 'Nothing changed since this build, so it was not built again.' };
    }
    const job =
      jobs.get(kind) ??
      startJob(kind, async (log) => {
        const s = load();
        s.spec = validateSpec(s.spec ?? session.spec);
        s.status.job = { kind, pid: process.pid, started: Date.now() };
        save(s);
        const result = kind === 'build' ? await build(dir, s, { log, overwrite: args.overwrite, remote: args.remote ?? null, root: cwd, repository: args.repository ?? null }) : await prove(dir, s, { log, overwrite: args.overwrite });
        if (kind === 'prove' || result.ok) s.status.lastSpec = JSON.stringify(s.spec);
        if (kind === 'build' && result.ok) s.status.lastBuild = { modelSha: s.status.modelSha, holo: result.holo, result };
        delete s.status.job;
        // Answers given while the job ran stay; only what the job learned is merged in.
        const latest = load();
        save({ ...latest, status: s.status, axioms: s.axioms, image: s.image ?? latest.image });
        return result;
      });
    return awaitJob(job, args.wait ?? 45, progress);
  }

  const tools = {
    prism_start(args) {
      const session = args.restart ? { ...load(), step: 'purpose', answers: {}, spec: null, status: {}, axioms: {} } : load();
      const t = turn(session);
      const resumed = Object.keys(session.answers).length > 0;
      return { greeting: resumed ? `Welcome back. ${t.progress}.` : GREETING, project: dir, ...t };
    },
    prism_answer(args) {
      const session = load();
      const step = args.step ?? session.step ?? nextTurn(session.answers).step;
      if (step === 'ready') return { message: 'The spec is complete. Call prism_build to prove and build it, or pass step to change an answer.' };
      session.answers[step] = args.answer;
      if (step !== 'review') delete session.answers.review;
      return turn(session);
    },
    prism_spec(args) {
      const session = load();
      if (!args.spec) return { spec: session.spec ?? nextTurn(session.answers).spec ?? null, step: session.step };
      session.answers = { ...answersFromSpec(args.spec), review: undefined };
      delete session.answers.review;
      return turn(session);
    },
    prism_prove: (args, progress) => longTool('prove', args, progress),
    prism_build: (args, progress) => longTool('build', args, progress),
    async prism_try(args) {
      const session = load();
      const t = nextTurn(session.answers);
      if (!t.spec && !session.spec) return { message: 'There is no complete spec yet to answer from.' };
      return tryRequest({ ...session, spec: session.spec ?? t.spec }, args.request);
    },
    prism_status() {
      const session = load();
      const t = nextTurn(session.answers);
      return {
        project: dir,
        step: t.step,
        question: t.question,
        properties: session.status.properties ?? {},
        proved: session.status.proved ?? null,
        built: session.status.built ?? null,
        holo: session.status.holo ?? null,
        jobs: [...jobs.values()].map((j) => ({ kind: j.kind, state: j.state, elapsed: Math.round((Date.now() - j.started) / 1000), latest: j.log.slice(-3) })),
      };
    },
  };

  const text = (value) => (typeof value === 'string' ? value : JSON.stringify(value, null, 2));
  const reply = (id, result) => send({ jsonrpc: '2.0', id, result });
  const error = (id, code, message) => send({ jsonrpc: '2.0', id, error: { code, message } });

  async function handle(message) {
    const { id, method, params = {} } = message;
    const isRequest = id !== undefined && id !== null;
    try {
      switch (method) {
        case 'initialize': {
          const requested = params.protocolVersion;
          return reply(id, {
            protocolVersion: PROTOCOL_VERSIONS.includes(requested) ? requested : PROTOCOL_VERSIONS[0],
            capabilities: { tools: {}, prompts: {}, resources: {} },
            serverInfo: { name: 'prism', title: 'Prism', version },
            instructions: INSTRUCTIONS,
          });
        }
        case 'ping':
          return reply(id, {});
        case 'tools/list':
          return reply(id, { tools: TOOLS });
        case 'tools/call': {
          const tool = tools[params.name];
          if (!tool) return error(id, -32602, `Unknown tool: ${params.name}`);
          const token = params._meta?.progressToken;
          const progress = token === undefined ? null : (n, line) => send({ jsonrpc: '2.0', method: 'notifications/progress', params: { progressToken: token, progress: n, message: line } });
          let result;
          try {
            result = await tool(params.arguments ?? {}, progress);
          } catch (e) {
            if (e instanceof SpecError) return reply(id, { content: [{ type: 'text', text: e.message }], structuredContent: { issue: e.message }, isError: true });
            throw e;
          }
          const isError = result && (result.ok === false || result.state === 'blocked');
          return reply(id, { content: [{ type: 'text', text: text(result) }], structuredContent: result, ...(isError ? { isError: true } : {}) });
        }
        case 'prompts/list':
          return reply(id, { prompts: PROMPTS });
        case 'prompts/get':
          if (params.name !== 'prism') return error(id, -32602, `Unknown prompt: ${params.name}`);
          return reply(id, { description: PROMPTS[0].description, messages: [{ role: 'user', content: { type: 'text', text: 'Call the prism_start tool and follow what it returns: ask me its questions one at a time.' } }] });
        case 'resources/list':
          return reply(id, {
            resources: [
              { uri: 'prism://spec', name: 'SPEC.md', description: 'The spec in prose, with what is proved', mimeType: 'text/markdown' },
              { uri: 'prism://model', name: 'model', description: 'The generated LexLean model', mimeType: 'application/x-tex' },
              { uri: 'prism://status', name: 'status', description: 'Session progress, proofs and build', mimeType: 'application/json' },
            ],
          });
        case 'resources/templates/list':
          return reply(id, { resourceTemplates: [] });
        case 'resources/read': {
          const session = load();
          const read = (path) => (existsSync(path) ? readFileSync(path, 'utf8') : '');
          const contents = {
            'prism://spec': () => ({ mimeType: 'text/markdown', text: read(join(dir, 'SPEC.md')) || 'No spec yet.' }),
            'prism://model': () => ({ mimeType: 'application/x-tex', text: session.spec ? read(modelPath(dir, session.spec)) || 'Not generated yet.' : 'Not generated yet.' }),
            'prism://status': () => ({ mimeType: 'application/json', text: text(tools.prism_status()) }),
          }[params.uri];
          if (!contents) return error(id, -32602, `Unknown resource: ${params.uri}`);
          return reply(id, { contents: [{ uri: params.uri, ...contents() }] });
        }
        case 'logging/setLevel':
          return reply(id, {});
        case 'completion/complete':
          return reply(id, { completion: { values: [], hasMore: false } });
        default:
          if (method?.startsWith('notifications/')) return undefined;
          if (isRequest) return error(id, -32601, `Method not found: ${method}`);
          return undefined;
      }
    } catch (e) {
      if (isRequest) return error(id, -32603, e.message);
      return undefined;
    }
  }
  /** Settles when no job runs, so a closing server still saves what its jobs learn. */
  const idle = () => Promise.allSettled([...jobs.values()].map((j) => j.promise));
  return { handle, dir, idle };
}

/** Serve on stdin and stdout until stdin closes. */
export function serveStdio({ cwd = process.cwd(), input = process.stdin, output = process.stdout } = {}) {
  const server = createServer({ cwd, send: (message) => output.write(`${JSON.stringify(message)}\n`) });
  const lines = createInterface({ input, crlfDelay: Infinity });
  const pending = new Set();
  lines.on('line', (line) => {
    if (!line.trim()) return;
    let message;
    try {
      message = JSON.parse(line);
    } catch {
      output.write(`${JSON.stringify({ jsonrpc: '2.0', id: null, error: { code: -32700, message: 'Parse error' } })}\n`);
      return;
    }
    for (const m of Array.isArray(message) ? message : [message]) {
      const p = server.handle(m);
      pending.add(p);
      p.finally(() => pending.delete(p));
    }
  });
  return new Promise((done) => lines.on('close', () => Promise.allSettled([...pending]).then(() => server.idle()).then(() => done(server))));
}
