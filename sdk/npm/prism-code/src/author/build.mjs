/**
 * `build --unstable-author` (issue #325): examples and intent in, an agent's
 * LexLean out, proof as the gate. The agent is not trusted; everything it says
 * is checked by the pinned toolchain, the kernel and the examples. `agent` and
 * `toolchain` are parameters, so the loop runs the same under a test.
 *
 * A failed build leaves `model/` as it found it and the rejected attempt under
 * `.prism-author/rejected/`. The lock's `authoring` record is written last, so
 * it only ever names a model that passed.
 */
import { existsSync, mkdirSync, readFileSync, renameSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { canonicalJson } from '../archive.mjs';
import { fail, ReplayError } from '../errors.mjs';
import { fromFile, readManifest } from '../node.mjs';
import { acceptanceHistory, displayOutcome, experimental_replay } from '../replay.mjs';
import { authorRequest, DEFAULT_MODEL, unmappedRecords, validateResponse } from './agent.mjs';
import { formatFailure, formatReport } from './report.mjs';
import { falsified, loadSpec, MODEL_DIR, modelDigest, readModel, SIDECAR_FILE } from './spec.mjs';
import { findHolo, intentVerdicts, NO_DOCKER, readTheorems } from './toolchain.mjs';

const WORK_DIR = '.prism-author';
const SDK_LOCK_V3 = 'prismpm/sdk-lock/3';
const SIDECAR_SCHEMA = 'prism/author-sidecar/1';
const encoder = new TextEncoder();
const decoder = new TextDecoder();

/**
 * Replay `examples/` through the holo's guest (#284). The holo's acceptance
 * vectors are its first history, and the gate requires them to equal the
 * examples, so each event is one file. Resolves to `{ events, failed }`, each
 * failure `{ example, reason }`; a replay that cannot be carried out is a
 * failure with no example.
 */
export async function replayExamplesGate(app, examples) {
  try {
    const { events, histories } = await experimental_replay(app, await acceptanceHistory(app), { throw: false });
    const failed = histories[0].divergences.map(({ index, expected, actual }) => ({
      example: examples[index]?.path ?? null,
      reason: `vector ${index} expected ${displayOutcome(expected)}, and the guest returned ${displayOutcome(actual)}`,
    }));
    return { events, failed };
  } catch (error) {
    if (!(error instanceof ReplayError)) throw error;
    return { events: 0, failed: [{ example: null, reason: error.message }] };
  }
}

export function readLock(root) {
  let text;
  try {
    text = readFileSync(join(root, 'prismpm.lock'), 'utf8');
  } catch {
    fail('lock.missing', 'prismpm.lock not found in the project; bind an SDK first, with prism-code init --sdk-image <ref>');
  }
  try {
    return { text, value: JSON.parse(text) };
  } catch (cause) {
    fail('lock.invalid', `prismpm.lock is not JSON: ${cause.message}`);
  }
}

/** Add the `authoring` record, atomically, never over bytes this build did not read. Canonical bytes, as PrismPM writes them. */
function recordAuthoring(root, record) {
  const { text, value } = readLock(root);
  if (canonicalJson(value.authoring ?? null) === canonicalJson(record)) return;
  const staged = join(root, `.prismpm.lock.${process.pid}`);
  writeFileSync(staged, canonicalJson({ ...value, authoring: record }));
  if (readFileSync(join(root, 'prismpm.lock'), 'utf8') !== text) {
    rmSync(staged);
    fail('lock.changed', 'prismpm.lock changed while the build was running; rerun the build');
  }
  renameSync(staged, join(root, 'prismpm.lock'));
}

const sidecarOf = (model) => {
  const bytes = model?.get(SIDECAR_FILE);
  if (!bytes) return null;
  try {
    return JSON.parse(decoder.decode(bytes));
  } catch {
    return null;
  }
};

/**
 * What `model/` is against the spec and the lock. `current` needs no agent.
 * `stale` and `absent` do. A model nobody generated, or a generated one edited
 * since, is refused: the agent must not overwrite a person's work.
 */
export function modelState(spec, lock, root, noCache = false) {
  const model = readModel(root);
  if (model === null) return { state: 'absent', digest: null };
  const digest = modelDigest(model);
  const record = lock?.authoring ?? null;
  const sidecar = sidecarOf(model);
  if (record?.spec?.blake3 === spec.digest) {
    if (noCache) return { state: 'stale', digest };
    if (record.model_tree?.blake3 !== digest) {
      fail('author.model', `${MODEL_DIR}/ changed after it was generated from this spec (prismpm.lock pins blake3:${record.model_tree?.blake3}, it is blake3:${digest}); restore it, or change the spec to regenerate it, or remove the spec files to own the model by hand`);
    }
    return { state: 'current', digest, agent: record.agent };
  }
  if (!sidecar) {
    fail('author.model', `${MODEL_DIR}/ has no ${SIDECAR_FILE}, so it was written by hand; a hand-written model takes no intent.md or examples/. Remove them to keep it, or move ${MODEL_DIR}/ away to generate one`);
  }
  // `prismpm lock update` proposes against the /2 projection and drops the record; the sidecar still says what this model came from.
  if (!record && sidecar.spec === spec.digest) return { state: noCache ? 'stale' : 'current', digest, agent: sidecar.agent, rebind: true };
  return { state: 'stale', digest };
}

function writeTree(directory, files) {
  for (const [path, bytes] of files) {
    const full = join(directory, path);
    mkdirSync(dirname(full), { recursive: true });
    writeFileSync(full, bytes);
  }
}

/**
 * Replace `model/` with an attempt: the scaffold, then what the agent wrote
 * over it. The first attempt holds the old model aside and then sets the
 * `attempted` marker, so `rollback` can always tell whose `model/` it sees.
 */
function writeAttempt(root, response, scaffoldFiles, spec, pin) {
  const work = join(root, WORK_DIR);
  const model = join(root, MODEL_DIR);
  mkdirSync(work, { recursive: true });
  if (existsSync(join(work, 'attempted'))) {
    rmSync(model, { recursive: true, force: true });
  } else {
    if (existsSync(model)) renameSync(model, join(work, 'previous'));
    writeFileSync(join(work, 'attempted'), '');
  }
  const files = new Map(scaffoldFiles);
  for (const { path, text } of response.files) files.set(path, encoder.encode(text));
  const sidecar = {
    schema: SIDECAR_SCHEMA,
    spec: spec.digest,
    agent: pin,
    sentences: response.sentences.map(({ theorem, status, statement = null }) => ({ sentence: spec.intent.find((row) => row.theorem === theorem).sentence, theorem, status, statement })),
    types: response.types ?? [],
  };
  files.set(SIDECAR_FILE, encoder.encode(`${JSON.stringify(sidecar, null, 2)}\n`));
  writeTree(model, files);
}

/** Put the old model back and keep the attempt for the reader. Also recovers a build that was killed. */
function rollback(root) {
  const work = join(root, WORK_DIR);
  const model = join(root, MODEL_DIR);
  if (existsSync(join(work, 'attempted'))) {
    rmSync(join(work, 'rejected'), { recursive: true, force: true });
    if (existsSync(model)) renameSync(model, join(work, 'rejected'));
    rmSync(join(work, 'attempted'));
  }
  if (existsSync(join(work, 'previous'))) renameSync(join(work, 'previous'), model);
}

function commit(root) {
  rmSync(join(root, WORK_DIR), { recursive: true, force: true });
}

/** The root `prismpm.toml` that points the toolchain at `model/`. A project that has one keeps it. */
function ensureProjectFile(root, spec) {
  const path = join(root, 'prismpm.toml');
  if (existsSync(path)) return;
  writeFileSync(
    path,
    [
      'spec = "prismpm/project/1"',
      `project = ${JSON.stringify(spec.manifest.application.name)}`,
      `lexlean_project = "${MODEL_DIR}/lexlean.toml"`,
      'build_root = ".prism"',
      '',
      '[limits]',
      'max_holo_bytes = 16777216',
      'max_entities = 100000',
      'max_diagnostics = 256',
      '',
    ].join('\n'),
  );
}

/** Which example, if any, the holo's vectors disagree with: the same bytes, in the same order. */
export function vectorMismatch(examples, vectors) {
  if (examples.length !== vectors.length) return { example: null, reason: `the holo carries ${vectors.length} acceptance vectors and examples/ holds ${examples.length}` };
  for (const [index, { path, request, response }] of examples.entries()) {
    const vector = vectors[index];
    if (Buffer.compare(Buffer.from(request), Buffer.from(vector.request)) !== 0) return { example: path, reason: 'the holo carries a different input for it' };
    if (Buffer.compare(Buffer.from(response), Buffer.from(vector.response)) !== 0) return { example: path, reason: 'the holo carries a different output for it' };
  }
  return null;
}

/**
 * The checks every build passes, with or without a new model: toolchain check
 * and build, the holo's vectors against `examples/`, the guest's replay of
 * them, the holo's effects against the grant, and the kernel's word on every
 * sentence. `{ ok, stage, diagnostics, rows, replayed, holo }`; `fatal` stops
 * the loop, since no new model can fix a missing Docker.
 */
async function gate({ root, spec, toolchain, update }) {
  const failed = (stage, diagnostics, extra = {}) => ({ ok: false, stage, diagnostics, rows: null, ...extra });
  const step = async (name, args = []) => {
    const result = await toolchain.run(name, args);
    if (result.code === NO_DOCKER) return failed(name, result.output, { fatal: NO_DOCKER });
    return result.code === 0 ? null : failed(name, result.output);
  };
  const check = await step('check');
  if (check) return check;
  const built = await step('build', update ? ['--update'] : []);
  if (built) return built;

  const lock = readLock(root).value;
  const modelSha256 = lock.inputs?.model?.sha256;
  if (modelSha256 === undefined) return failed('build', 'prismpm.lock pins no model digest after the build; it needs prismpm/sdk-lock/3');
  if (lock.inputs.spec?.blake3 !== spec.digest) return failed('build', `prismpm.lock pins spec blake3:${lock.inputs.spec?.blake3}, and this spec is blake3:${spec.digest}; the toolchain and prism-code read different spec files`);
  const holo = findHolo(root, modelSha256);
  if (!holo) return failed('build', `no build under the build root names model ${modelSha256}`);

  const { grant } = await readManifest(root);
  let app;
  try {
    app = await fromFile(holo, grant);
  } catch (error) {
    return failed('holo', `the holo cannot load under the capabilities prism.toml grants: ${error.message}`);
  }
  if (!app.identities.verified) return failed('holo', 'the holo does not match its own digests');
  const mismatch = vectorMismatch(spec.examples, app.vectors);
  if (mismatch) return failed('examples', mismatch.example ? `${mismatch.example}: ${mismatch.reason}` : mismatch.reason, { example: mismatch.example });
  const replay = await replayExamplesGate(app, spec.examples);
  if (replay.failed.length > 0) {
    return failed('examples', replay.failed.map((f) => `${f.example ?? 'examples/'}: ${f.reason}`).join('\n'), { example: replay.failed[0].example });
  }

  if (!readTheorems(root, modelSha256)) {
    const verified = await step('verify');
    if (verified) return verified;
  }
  const rows = intentVerdicts(spec.intent, readTheorems(root, modelSha256));
  const unproved = rows.filter((row) => row.verdict !== 'checked');
  if (unproved.length > 0) return failed('proof', unproved.map((row) => `${row.sentence}: ${row.why}`).join('\n'), { rows });
  return { ok: true, rows, replayed: replay.events, holo, lock };
}

/**
 * Run the front door. Returns `{ code, text }`, or `{ delegate: true }` for a
 * level 3 project, which builds exactly as it does without the flag.
 *
 * @param {object} options
 * @param {string} options.root the project directory
 * @param {{ adapter: string, turn: (request: object) => Promise<object> }} options.agent
 * @param {{ run: (command: string, args?: string[]) => Promise<{ code: number, output: string }> }} options.toolchain
 * @param {Map<string, Uint8Array>} options.scaffold the files every model starts from
 * @param {object} options.scaffoldContext what the agent is shown of them
 */
export async function authorBuild({ root, agent, toolchain, scaffold, scaffoldContext, model = null, maxIterations = 10, maxTokens = 2_000_000, noCache = false, checkOnly = false, env = process.env }) {
  const spec = loadSpec(root);
  if (spec.level === 3) return { delegate: true };

  const wrong = falsified(spec.intent, spec.examples);
  if (wrong.length > 0) return { code: 1, text: formatFailure({ spec, falsified: wrong }) };

  rollback(root);
  if (checkOnly) {
    const lock = existsSync(join(root, 'prismpm.lock')) ? readLock(root).value : null;
    const { state } = modelState(spec, lock, root, noCache);
    return {
      code: 0,
      text: `Spec blake3:${spec.digest}: ${spec.examples.length} examples, ${spec.intent.length} sentences, ${spec.types.length} type files. None contradicts an example.\nModel: ${state === 'current' ? 'current, no agent call' : `${state}, a build calls the agent`}\n`,
    };
  }

  const lock = readLock(root).value;
  const found = modelState(spec, lock, root, noCache);
  const pin = { adapter: agent.adapter, model: model ?? lock.authoring?.agent?.model ?? DEFAULT_MODEL };
  ensureProjectFile(root, spec);

  const finish = (result, source) => {
    recordAuthoring(root, {
      agent: source.agent,
      model_tree: { blake3: modelDigest(readModel(root)) },
      spec: { blake3: spec.digest },
    });
    commit(root);
    return { code: 0, text: formatReport({ spec, rows: result.rows, replayed: result.replayed, source, holo: result.holo }) };
  };

  if (found.state === 'current') {
    const result = await gate({ root, spec, toolchain, update: lock.schema !== SDK_LOCK_V3 || found.rebind === true });
    if (result.fatal) return { code: result.fatal, text: `${result.diagnostics}\n` };
    if (!result.ok) return { code: 1, text: formatFailure({ spec, gate: result, attempts: 0 }) };
    return finish(result, { cached: true, agent: found.agent });
  }

  // A failed attempt may have moved the lock's inputs; only a pass may keep that.
  const lockBefore = readLock(root).text;
  let succeeded = false;
  let previous = null;
  let usage = 0;
  let attempt = 0;
  let last = null;
  try {
    while (attempt < maxIterations) {
      attempt += 1;
      const request = authorRequest({ spec, pin, attempt, maxAttempts: maxIterations, scaffold: scaffoldContext, previous, env });
      let answer;
      try {
        answer = await agent.turn(request);
      } catch (error) {
        if (error.authorCode) return { code: 1, text: formatFailure({ spec, agentError: error.message, attempts: attempt }) };
        throw error;
      }
      usage += Number(answer?.usage?.total_tokens ?? 0);
      let response = null;
      try {
        response = validateResponse(answer, spec);
      } catch (error) {
        if (error.code !== 'author.agent') throw error;
        last = { ok: false, stage: 'response', diagnostics: error.message };
        previous = { files: [], stage: last.stage, diagnostics: last.diagnostics };
      }
      if (response) {
        const givenUp = response.sentences.filter((row) => row.status === 'unproved');
        if (givenUp.length > 0) return { code: 1, text: formatFailure({ spec, unproved: givenUp, attempts: attempt, tokens: usage }) };
        const missing = unmappedRecords(response.types, response.files);
        if (missing.length > 0) {
          last = { ok: false, stage: 'types', diagnostics: missing.map(({ source, record }) => `${source} maps to ${record}, which no source names`).join('\n') };
        } else {
          writeAttempt(root, response, scaffold, spec, pin);
          last = await gate({ root, spec, toolchain, update: true });
          if (last.fatal) return { code: last.fatal, text: `${last.diagnostics}\n` };
          if (last.ok) {
            const done = finish(last, { cached: false, agent: pin, attempts: attempt, tokens: usage });
            succeeded = true;
            return done;
          }
        }
        previous = { files: response.files, stage: last.stage, diagnostics: last.diagnostics };
      }
      if (usage > maxTokens) return { code: 1, text: formatFailure({ spec, gate: last, attempts: attempt, tokens: usage, budget: `the token budget of ${maxTokens} is spent` }) };
    }
    return { code: 1, text: formatFailure({ spec, gate: last, attempts: attempt, tokens: usage, budget: `${maxIterations} attempts are spent` }) };
  } finally {
    if (!succeeded) {
      rollback(root);
      writeFileSync(join(root, 'prismpm.lock'), lockBefore);
    }
  }
}
