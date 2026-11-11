/**
 * The authoring spec (issue #325, docs/proposals/325-authoring-front-door.md):
 * `prism.toml`, `examples/`, `intent.md` and `types/`. Pure reading and
 * checking; nothing here calls an agent or a toolchain.
 *
 * The digest is the one `crates/prismpm/src/input_lock.rs` pins as
 * `inputs.spec`, so the lock's value and this one can be compared directly.
 *
 * Experimental behind `--unstable-author` (STABILITY.md section 6.1).
 */
import { lstatSync, readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { Blake3, blake3Hex, toHex } from '../blake3.mjs';
import { canonicalJson } from '../archive.mjs';
import { fail } from '../errors.mjs';
import { MANIFEST_FILE, parseManifest } from '../manifest.mjs';

export const INTENT_FILE = 'intent.md';
export const EXAMPLES_DIR = 'examples';
export const TYPES_DIR = 'types';
export const MODEL_DIR = 'model';
/** Written by Prism inside `model/`: what the agent claimed, kept for review. */
export const SIDECAR_FILE = 'author.json';

const SPEC_DOMAIN = 'prismpm/spec-inputs/1\n';
const MODEL_DOMAIN = 'prism/authored-model/1\n';
const MAX_FILES = 4096;
const MAX_BYTES = 512 * 1024 * 1024;
const TYPE_LANGUAGES = Object.freeze({ '.ts': 'typescript', '.json': 'json-schema', '.py': 'pydantic' });

const encoder = new TextEncoder();
const utf8 = new TextDecoder('utf-8', { fatal: true });
const isObject = (value) => value !== null && typeof value === 'object' && !Array.isArray(value);

function statOf(path) {
  try {
    return lstatSync(path);
  } catch (error) {
    if (error.code === 'ENOENT') return null;
    throw error;
  }
}

/** Paths in the bytewise order of their UTF-8, the order the Rust `BTreeMap` keeps. */
function byteOrder(paths) {
  return [...paths].sort((a, b) => Buffer.compare(Buffer.from(a), Buffer.from(b)));
}

/** BLAKE3 over the domain, then each file as (u32 path length, path, u64 length, bytes), in bytewise path order. */
export function framedDigest(domain, files) {
  const hasher = new Blake3();
  hasher.update(encoder.encode(domain));
  for (const path of byteOrder(files.keys())) {
    const name = Buffer.from(path);
    const bytes = files.get(path);
    const pathLength = Buffer.alloc(4);
    pathLength.writeUInt32BE(name.length);
    const contentLength = Buffer.alloc(8);
    contentLength.writeBigUInt64BE(BigInt(bytes.length));
    hasher.update(pathLength).update(name).update(contentLength).update(bytes);
  }
  return toHex(hasher.digest());
}

export const specDigest = (files) => framedDigest(SPEC_DOMAIN, files);
export const modelDigest = (files) => framedDigest(MODEL_DOMAIN, files);

function addFile(root, path, files, budget) {
  budget.bytes += statOf(join(root, path)).size;
  if (files.size >= MAX_FILES || budget.bytes > MAX_BYTES) fail('author.spec', `the files under ${path} exceed ${MAX_FILES} files or ${MAX_BYTES} bytes`);
  files.set(path, new Uint8Array(readFileSync(join(root, path))));
}

/** Add every regular file below `dir`. A symlink is refused, as the Rust reader refuses it. */
function addTree(root, dir, files, budget) {
  if (!statOf(join(root, dir)).isDirectory()) fail('author.spec', `${dir} must be a directory, not a symlink or another node`);
  for (const name of readdirSync(join(root, dir))) {
    const path = `${dir}/${name}`;
    const info = statOf(join(root, path));
    if (info.isDirectory()) addTree(root, path, files, budget);
    else if (info.isFile()) addFile(root, path, files, budget);
    else fail('author.spec', `${path} must be a regular file or directory, not a symlink or another node`);
  }
}

/** The files of `model/` as path to bytes, paths relative to it; null when it does not exist. */
export function readModel(root) {
  if (!statOf(join(root, MODEL_DIR))) return null;
  const files = new Map();
  addTree(root, MODEL_DIR, files, { bytes: 0 });
  return new Map([...files].map(([path, bytes]) => [path.slice(MODEL_DIR.length + 1), bytes]));
}

/** A Lean-safe theorem name, the same for the same sentence on every machine. */
export function theoremName(sentence) {
  const words = sentence.toLowerCase().match(/[a-z0-9]+/g) ?? [];
  const hash = blake3Hex(encoder.encode(sentence.normalize('NFC'))).slice(0, 8);
  return ['intent', ...words.slice(0, 6), hash].join('_');
}

const unescapeLiteral = (text) => text.replace(/\\([tn\\])/g, (_, c) => ({ t: '\t', n: '\n', '\\': '\\' })[c]);

const QUANTIFIED = /\b(never|always) returns? `([^`]+)`(?: when the input contains `([^`]+)`)?\.$/i;
const ONLY_WHEN = /\breturns? `([^`]+)` only when the input contains `([^`]+)`\.$/i;

/** The part of a sentence the examples can judge without an agent, or null. */
function ruleOf(sentence) {
  const only = ONLY_WHEN.exec(sentence);
  if (only) return { kind: 'only', output: unescapeLiteral(only[1]), input: unescapeLiteral(only[2]) };
  const quantified = QUANTIFIED.exec(sentence);
  if (!quantified) return null;
  return { kind: quantified[1].toLowerCase(), output: unescapeLiteral(quantified[2]), input: quantified[3] === undefined ? null : unescapeLiteral(quantified[3]) };
}

/**
 * `intent.md`: one sentence per line. Blank lines and `#` headings are skipped.
 * A sentence ends with a period and uses exactly one of always, never or only,
 * outside any backticks. Returns `{ line, sentence, kind, theorem, rule }` rows.
 */
export function parseIntent(text) {
  const rows = [];
  const seen = new Map();
  for (const [index, raw] of text.split(/\r?\n/).entries()) {
    const sentence = raw.trim();
    if (sentence === '' || sentence.startsWith('#')) continue;
    const at = `${INTENT_FILE} line ${index + 1}`;
    const prose = sentence.replace(/`[^`]*`/g, '`');
    if (!sentence.endsWith('.')) fail('author.intent', `${at}: a sentence ends with a period: ${JSON.stringify(sentence)}`);
    if (/[.!?]\s+\S/.test(prose)) fail('author.intent', `${at}: one sentence per line: ${JSON.stringify(sentence)}`);
    const keywords = prose.match(/\b(always|never|only)\b/gi) ?? [];
    if (keywords.length !== 1) fail('author.intent', `${at}: a sentence uses exactly one of always, never or only, and this one uses ${keywords.length}: ${JSON.stringify(sentence)}`);
    const theorem = theoremName(sentence);
    if (seen.has(theorem)) fail('author.intent', `${at} repeats line ${seen.get(theorem)}: ${JSON.stringify(sentence)}`);
    seen.set(theorem, index + 1);
    rows.push({ line: index + 1, sentence, kind: keywords[0].toLowerCase(), theorem, rule: ruleOf(sentence) });
  }
  return rows;
}

function wireOf(value, where) {
  if (typeof value === 'string') return encoder.encode(value);
  if (value === undefined) fail('author.example', `${where} is missing`);
  if (isObject(value) && Object.keys(value).length === 1 && Object.hasOwn(value, 'hex')) {
    if (typeof value.hex !== 'string' || !/^([0-9a-f]{2})*$/.test(value.hex)) fail('author.example', `${where}.hex must be lowercase hex`);
    return new Uint8Array(Buffer.from(value.hex, 'hex'));
  }
  return encoder.encode(canonicalJson(value));
}

/**
 * One example file: `{ input, output, name? }`. A string is its UTF-8 bytes,
 * `{ "hex": "..." }` is those bytes, and any other value is its canonical JSON.
 * What the file says is what the guest receives and must return.
 */
export function parseExample(path, bytes) {
  if (!path.endsWith('.json')) fail('author.example', `${path}: an example is a .json file`);
  let value;
  try {
    value = JSON.parse(utf8.decode(bytes));
  } catch (cause) {
    fail('author.example', `${path} is not JSON: ${cause.message}`);
  }
  if (!isObject(value)) fail('author.example', `${path} must hold one object`);
  const unknown = Object.keys(value).filter((key) => !['input', 'output', 'name'].includes(key));
  if (unknown.length > 0) fail('author.example', `${path} has unknown members ${unknown.join(', ')}; it has input, output, name`);
  return { path, name: value.name ?? null, request: wireOf(value.input, `${path} input`), response: wireOf(value.output, `${path} output`) };
}

const contains = (haystack, needle) => Buffer.from(haystack).includes(Buffer.from(needle));

/**
 * The sentences the examples prove false, before any agent runs. Only the
 * backticked form has a rule; a plain sentence is left to the kernel.
 * Returns `{ sentence, line, example, reason }` rows.
 */
export function falsified(intent, examples) {
  const rows = [];
  for (const { sentence, line, rule } of intent) {
    if (!rule) continue;
    for (const { path, request, response } of examples) {
      const shown = JSON.stringify(rule.output);
      let reason = null;
      if (rule.kind === 'only') {
        if (contains(response, rule.output) && !contains(request, rule.input)) reason = `its output has ${shown} and its input lacks ${JSON.stringify(rule.input)}`;
      } else if (rule.input === null || contains(request, rule.input)) {
        const has = contains(response, rule.output);
        if (rule.kind === 'never' && has) reason = `its output has ${shown}`;
        if (rule.kind === 'always' && !has) reason = `its output lacks ${shown}`;
      }
      if (reason) {
        rows.push({ sentence, line, example: path, reason });
        break;
      }
    }
  }
  return rows;
}

/**
 * Read the spec under `root`. `level` is 3 when `prism.toml` names a project
 * with no `intent.md` and no `examples/`: a hand-written model, built as today.
 * `prism.toml` is required; the flag is meaningless without one.
 */
export function loadSpec(root) {
  if (!statOf(join(root, MANIFEST_FILE))) fail('author.spec', `--unstable-author reads ${MANIFEST_FILE}, and ${root} has none`);
  const files = new Map();
  const budget = { bytes: 0 };
  addFile(root, MANIFEST_FILE, files, budget);
  const { manifest } = parseManifest(files.get(MANIFEST_FILE));
  const hasIntent = statOf(join(root, INTENT_FILE)) !== null;
  const hasExamples = statOf(join(root, EXAMPLES_DIR)) !== null;
  if (!hasIntent && !hasExamples) return { level: 3, root, manifest };
  if (!hasIntent) fail('author.spec', `${EXAMPLES_DIR}/ needs an ${INTENT_FILE} beside it; the agent proves sentences, not only examples`);
  if (!hasExamples) fail('author.spec', `${INTENT_FILE} needs ${EXAMPLES_DIR}/ beside it; the examples are the acceptance vectors`);
  if (manifest.model?.source !== undefined && manifest.model.source !== MODEL_DIR) {
    fail('author.spec', `[model] source is ${JSON.stringify(manifest.model.source)}, and a generated model lives in ${MODEL_DIR}/; set it to "${MODEL_DIR}" or remove the spec files`);
  }
  if (!statOf(join(root, INTENT_FILE)).isFile()) fail('author.spec', `${INTENT_FILE} must be a regular file`);
  addFile(root, INTENT_FILE, files, budget);
  addTree(root, EXAMPLES_DIR, files, budget);
  if (statOf(join(root, TYPES_DIR))) addTree(root, TYPES_DIR, files, budget);

  const paths = byteOrder(files.keys());
  const examples = paths.filter((p) => p.startsWith(`${EXAMPLES_DIR}/`)).map((p) => parseExample(p, files.get(p)));
  const intent = parseIntent(utf8.decode(files.get(INTENT_FILE)));
  if (examples.length === 0) fail('author.spec', `${EXAMPLES_DIR}/ holds no example`);
  if (intent.length === 0) fail('author.spec', `${INTENT_FILE} holds no sentence`);
  const types = paths
    .filter((p) => p.startsWith(`${TYPES_DIR}/`))
    .map((path) => {
      const language = TYPE_LANGUAGES[/\.[^.]+$/.exec(path)?.[0]];
      if (!language) fail('author.types', `${path}: types are TypeScript (.ts), JSON Schema (.json) or Pydantic (.py)`);
      return { path, language, text: utf8.decode(files.get(path)) };
    });
  return { level: 2, root, manifest, files, digest: specDigest(files), intent, examples, types };
}
