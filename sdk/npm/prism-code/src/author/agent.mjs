/**
 * The agent seam of `build --unstable-author` (issue #325). An agent is a
 * process: one `prism/author-request/1` JSON document on stdin, one
 * `prism/author-response/1` document on stdout, once per attempt. Prism owns
 * the loop, so any agent fits behind this seam and its answer is only ever
 * evidence: the toolchain and the kernel decide.
 *
 * `lexlean-mcp` has no running gateway yet (UOR-Foundation/lexlean-mcp#17), so
 * the request names one through `PRISM_LEXLEAN_MCP_URL` only when it is set.
 */
import { spawn } from 'node:child_process';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { fail } from '../errors.mjs';

export const REQUEST_SCHEMA = 'prism/author-request/1';
export const RESPONSE_SCHEMA = 'prism/author-response/1';
export const DOCS_INDEX = 'https://universal-coordinate-foundation.lovable.app/docs/llms.txt';
/** The adapter Prism runs when `--author-command` is not given, and the model it pins. */
export const DEFAULT_COMMAND = join(dirname(fileURLToPath(import.meta.url)), 'anthropic-cli.mjs');
export const DEFAULT_ADAPTER = 'anthropic';
export const DEFAULT_MODEL = 'claude-opus-5-5';

// The limits LexLean applies to a project (`lexlean.toml` `[limits]`).
const MAX_FILE_BYTES = 4 * 1024 * 1024;
const MAX_TOTAL_BYTES = 64 * 1024 * 1024;
const MAX_RESPONSE_BYTES = 128 * 1024 * 1024;

const RULES = Object.freeze([
  'Write the model in LexLean language 1.2 notation. Never write a .lean file: Lean is generated.',
  'For each intent row, declare a theorem with exactly the name in its theorem member, whose statement says what the sentence says, for every input the types allow. Do not weaken a sentence to make it provable.',
  'List every example as an acceptance vector, in the order given, with exactly the bytes given.',
  'Use no axiom beyond those the compiler allows and never sorry. If a sentence cannot be proved, or is false, report it as unproved with an explanation a developer can act on.',
  'Declare no host effect that the manifest does not grant.',
  'Map each type file to stdlib records where a stdlib record fits, and name the mapping in the types member.',
]);

const isObject = (value) => value !== null && typeof value === 'object' && !Array.isArray(value);
const decoder = new TextDecoder('utf-8', { fatal: true });

/** Bytes as the request shows them: UTF-8 text when they are, else `{ hex }`. */
function shown(bytes) {
  try {
    return decoder.decode(bytes);
  } catch {
    return { hex: Buffer.from(bytes).toString('hex') };
  }
}

/** One attempt's request. `previous` is the rejected attempt and why; null on the first. */
export function authorRequest({ spec, pin, attempt, maxAttempts, scaffold, previous = null, env = process.env }) {
  return {
    schema: REQUEST_SCHEMA,
    attempt,
    max_attempts: maxAttempts,
    agent: pin,
    application: spec.manifest.application,
    capabilities: spec.manifest.capabilities ?? {},
    spec: {
      digest: spec.digest,
      examples: spec.examples.map(({ path, name, request, response }) => ({ path, name, input: shown(request), output: shown(response) })),
      intent: spec.intent.map(({ line, sentence, kind, theorem }) => ({ line, sentence, kind, theorem })),
      types: spec.types,
    },
    context: {
      docs: DOCS_INDEX,
      lexlean_mcp: env.PRISM_LEXLEAN_MCP_URL ?? null,
      scaffold,
      rules: RULES,
    },
    previous,
  };
}

/** A path the agent may write inside `model/`: LexLean sources and project files, never Lean, never out of the tree. */
export function modelPathAllowed(path) {
  if (typeof path !== 'string' || path === '' || path.includes('\\') || path.startsWith('/') || /^[A-Za-z]:/.test(path)) return false;
  const parts = path.split('/');
  if (parts.some((part) => part === '' || part.startsWith('.'))) return false;
  if (path.endsWith('.lex.tex')) return true;
  if (['lexlean.toml', 'lexlean.lock', 'lakefile.toml', 'lean-toolchain', 'lake-manifest.json'].includes(path)) return true;
  return parts[0] === 'language' && path.endsWith('.toml');
}

/**
 * Check a response against the request's spec. The sentences must be answered
 * one to one: a missing or an invented theorem is refused here, before any
 * compile. Returns the response; throws `author.agent` naming what is wrong.
 */
export function validateResponse(response, spec) {
  const refuse = (message) => fail('author.agent', `the agent's answer is refused: ${message}`);
  if (!isObject(response) || response.schema !== RESPONSE_SCHEMA) refuse(`it is not a ${RESPONSE_SCHEMA} document`);
  if (!Array.isArray(response.files) || !Array.isArray(response.sentences)) refuse('it needs files and sentences arrays');
  let total = 0;
  const seen = new Set();
  for (const file of response.files) {
    if (!isObject(file) || typeof file.text !== 'string') refuse('a file needs a path and a text');
    if (!modelPathAllowed(file.path)) refuse(`${JSON.stringify(file.path)} is not a model source; it may write .lex.tex sources, the LexLean and Lake project files and language/**/*.toml`);
    if (seen.has(file.path)) refuse(`it writes ${file.path} twice`);
    seen.add(file.path);
    const size = Buffer.byteLength(file.text);
    total += size;
    if (size > MAX_FILE_BYTES) refuse(`${file.path} exceeds ${MAX_FILE_BYTES} bytes`);
  }
  if (total > MAX_TOTAL_BYTES) refuse(`its files exceed ${MAX_TOTAL_BYTES} bytes`);
  const expected = new Map(spec.intent.map((row) => [row.theorem, row]));
  const answered = new Set();
  for (const row of response.sentences) {
    if (!isObject(row) || !expected.has(row.theorem)) refuse(`it answers ${JSON.stringify(row?.theorem)}, which is no sentence of intent.md`);
    if (answered.has(row.theorem)) refuse(`it answers ${JSON.stringify(expected.get(row.theorem).sentence)} twice`);
    answered.add(row.theorem);
    if (row.status !== 'proved' && row.status !== 'unproved') refuse(`its status for ${JSON.stringify(expected.get(row.theorem).sentence)} must be proved or unproved`);
    if (row.status === 'unproved' && (typeof row.explanation !== 'string' || row.explanation === '')) refuse(`it leaves ${JSON.stringify(expected.get(row.theorem).sentence)} unproved with no explanation`);
  }
  for (const [theorem, row] of expected) {
    if (!answered.has(theorem)) refuse(`it does not answer ${JSON.stringify(row.sentence)}`);
  }
  const mapped = response.types ?? [];
  if (!Array.isArray(mapped) || mapped.some((row) => !isObject(row) || typeof row.source !== 'string' || typeof row.record !== 'string')) refuse('types must be { source, record } rows');
  for (const { path } of spec.types) {
    if (!mapped.some((row) => row.source === path || row.source.startsWith(`${path}#`))) refuse(`it maps nothing from ${path} to a record`);
  }
  return response;
}

/**
 * Check a type mapping against the written sources: each record must appear in
 * the model. A name check only; what the record means is for the kernel and
 * the reviewer. Returns the rows whose record no source names.
 */
export function unmappedRecords(types, files) {
  const sources = files.filter((file) => file.path.endsWith('.lex.tex')).map((file) => file.text);
  return (types ?? []).filter(({ record }) => !sources.some((text) => text.includes(record)));
}

/**
 * The process agent. A path ending in `.mjs` or `.js` runs under this Node. No
 * shell is involved, so the path is never interpreted.
 *
 * @param {{ command?: string, timeoutMs?: number, adapter?: string }} [options]
 */
export function processAgent({ command = DEFAULT_COMMAND, timeoutMs = 600_000, adapter = DEFAULT_ADAPTER } = {}) {
  const script = /\.m?js$/.test(command);
  const executable = script ? process.execPath : command;
  const args = script ? [command] : [];
  return {
    adapter,
    turn: (request) =>
      new Promise((resolve, reject) => {
        const child = spawn(executable, args, { stdio: ['pipe', 'pipe', 'inherit'], windowsHide: true });
        const chunks = [];
        let size = 0;
        let refused = null;
        const stop = (code, message) => {
          refused ??= { code, message };
          child.kill();
        };
        const timer = setTimeout(() => stop('author.timeout', `the agent did not answer in ${timeoutMs} ms`), timeoutMs);
        child.stdout.on('data', (chunk) => {
          size += chunk.length;
          if (size > MAX_RESPONSE_BYTES) stop('author.agent', `the agent's answer exceeds ${MAX_RESPONSE_BYTES} bytes`);
          else chunks.push(chunk);
        });
        child.stdin.on('error', () => {});
        child.on('error', (error) => {
          clearTimeout(timer);
          reject(Object.assign(new Error(`the agent ${command} could not start: ${error.message}`), { authorCode: 'author.agent' }));
        });
        child.on('close', (code, signal) => {
          clearTimeout(timer);
          if (refused) return reject(Object.assign(new Error(refused.message), { authorCode: refused.code }));
          if (code !== 0) return reject(Object.assign(new Error(`the agent exited with ${code ?? signal}`), { authorCode: 'author.agent' }));
          try {
            resolve(JSON.parse(Buffer.concat(chunks).toString('utf8')));
          } catch (cause) {
            reject(Object.assign(new Error(`the agent's answer is not JSON: ${cause.message}`), { authorCode: 'author.agent' }));
          }
        });
        child.stdin.end(JSON.stringify(request));
      }),
  };
}
