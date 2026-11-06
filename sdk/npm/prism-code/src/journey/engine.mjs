/**
 * Prove and build a spec: write the model, run the SDK image, and turn every
 * answer into something a person can act on. A wrong axiom set is fixed and
 * the proof run again without asking, because it is bookkeeping, not intent.
 */
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { join, relative } from 'node:path';
import { fromFile } from '../node.mjs';
import { counterexample, describeConditions } from './spec.mjs';
import { names } from './render.mjs';
import { bind, isBound, modelPath, newestHolo, rememberModel, writeApp, writeSession, writeSpecDocument } from './project.mjs';
import { BUILD_SCRIPT, DEFAULT_SDK_IMAGE, axiomViolations, declarationAt, docker, hasImage, imageOf, parseDiagnostics, resolveDocker, runInImage, steps } from './sdk.mjs';
import { SDK_IMAGE_DOWNLOAD } from '../setup-machine.mjs';
import { VERSION } from '../version.mjs';

const PROVE_SCRIPT_FULL = [
  'echo "@@step prelock"; lexlean lock 2>&1; code=$?; echo "@@exit prelock $code"; [ $code -eq 0 ] || exit $code',
  'echo "@@step format"; lexlean fmt --all 2>&1; code=$?; echo "@@exit format $code"; [ $code -eq 0 ] || exit $code',
  'echo "@@step lock"; lexlean lock 2>&1; code=$?; echo "@@exit lock $code"; [ $code -eq 0 ] || exit $code',
  'echo "@@step check"; prismpm check 2>&1; code=$?; echo "@@exit check $code"; [ $code -eq 0 ] || exit $code',
  'echo "@@step verify"; lexlean verify --all 2>&1; code=$?; echo "@@exit verify $code"; exit $code',
].join('; ');

const MAX_AXIOM_ROUNDS = 8;

/** The image this app is or will be pinned to. */
const imageFor = (dir, session) => (isBound(dir) ? imageOf(dir) : session.image ?? DEFAULT_SDK_IMAGE);

/**
 * Where Lean proves the app: `local` in the SDK image when Docker and the image
 * are on this machine, else `hosted` on the UOR MCP server. `PRISM_CODE_TIER`
 * set to `local` or `hosted` chooses.
 */
export function proveWhere(dir, session, { env = process.env } = {}) {
  if (env.PRISM_CODE_TIER === 'local' || env.PRISM_CODE_TIER === 'hosted') return env.PRISM_CODE_TIER;
  if (resolveDocker().error) return 'hosted';
  return hasImage(imageFor(dir, session)) ? 'local' : 'hosted';
}

/**
 * Make sure Docker, the image and the project's SDK pin are in place. With
 * `pull`, a missing image is downloaded first and the log says so; without it,
 * a missing image is refused.
 */
export async function prepare(dir, session, log, { pull = false } = {}) {
  const d = resolveDocker();
  if (d.error) return { ok: false, message: d.error };
  const image = imageFor(dir, session);
  if (!hasImage(image)) {
    if (!pull) return { ok: false, message: `The SDK image is not on this machine. Run \`npx prism-code@latest setup --pull\` in a terminal to download it (about ${SDK_IMAGE_DOWNLOAD}), or leave PRISM_CODE_TIER unset so the proof runs hosted.\n  ${image}` };
    log(`The SDK image is not on this machine yet. Downloading it once now, about ${SDK_IMAGE_DOWNLOAD}; this can take a while.`);
    const pulled = await docker(['pull', image], { onLine: (l) => /^(Status|Digest|[0-9a-f]{12}: Pull complete)/.test(l) && log(l) });
    if (pulled.code !== 0) return { ok: false, message: `Could not download the SDK image:\n${tail(pulled.output)}` };
  }
  if (!isBound(dir)) {
    log('Pinning the project to the SDK image (prismpm update)...');
    const r = await bind(dir, image, { onLine: log });
    if (!r.ok) return { ok: false, message: `Could not pin the SDK image:\n${tail(r.output)}` };
  }
  return { ok: true };
}

const tail = (text, n = 25) => String(text).split(/\r?\n/).filter((l) => !l.startsWith('@@')).slice(-n).join('\n');

/**
 * Lean proves every property. Returns `{ ok, properties, message, diagnostics }`,
 * with each property `proved`, `failing` or `unknown`.
 */
export async function prove(dir, session, { log = () => {}, overwrite = false, where = null, client = null } = {}) {
  const first = writeApp(dir, session, { overwrite });
  if (first.edited) return { ok: false, edited: true, message: `${relative(dir, first.path)} was edited by hand since Prism wrote it. Call the same tool again with overwrite: true to replace it from the spec, or change the spec to match.` };
  if ((where ?? proveWhere(dir, session)) === 'hosted') return proveHosted(dir, session, { log, client });
  const ready = await prepare(dir, session, log);
  if (!ready.ok) return { ok: false, message: ready.message };
  overwrite = true;
  const spec = session.spec;
  const n = names(spec);
  const started = Date.now();
  for (let round = 1; round <= MAX_AXIOM_ROUNDS; round++) {
    const written = writeApp(dir, session, { overwrite: overwrite || round > 1 });
    if (written.edited) {
      return { ok: false, edited: true, message: `${relative(dir, written.path)} was edited by hand since Prism wrote it. Call the same tool again with overwrite: true to replace it from the spec, or change the spec to match.` };
    }
    log(round === 1 ? 'Formatting, locking and checking the model, then asking Lean to prove it...' : `Correcting an axiom set and proving again (round ${round})...`);
    const run = await runInImage(dir, PROVE_SCRIPT_FULL, { onLine: (l) => /^(error|warning|verified|checked|check passed)/.test(l) && log(l) });
    rememberModel(dir, session);
    const diagnostics = parseDiagnostics(run.output);
    const failed = steps(run.output).find((s) => s.code !== 0);
    const fixes = axiomViolations(diagnostics);
    if (fixes.length && diagnostics.every((d) => d.code === 'LLV7005')) {
      for (const f of fixes) session.axioms[f.name] = f.axioms;
      continue;
    }
    const seconds = Math.round((Date.now() - started) / 1000);
    if (run.code === 0) return provedResult(dir, session, seconds, { where: 'local' });
    return explainFailure(dir, session, { run, diagnostics, failed, seconds });
  }
  return { ok: false, message: 'The axiom sets did not settle after several rounds. This is a Prism bug; the last output is in prism/.prism-last.log.' };
}

/** Record that every property was proved, and say so. `proof` says where, and for a hosted proof its attestation. */
function provedResult(dir, session, seconds, proof) {
  const spec = session.spec;
  const n = names(spec);
  session.status.properties = Object.fromEntries(spec.properties.map((p) => [p.id, 'proved']));
  session.status.proved = { modelSha: session.status.modelSha, at: new Date().toISOString(), seconds, ...proof };
  writeSpecDocument(dir, session);
  const where = proof.where === 'hosted' ? ' on the hosted verifier' : '';
  return {
    ok: true,
    seconds,
    where: proof.where,
    ...(proof.attestation ? { attestation: proof.attestation } : {}),
    properties: spec.properties.map((p) => ({ id: p.id, theorem: n.properties[p.id].theorem, says: `It ${p.kind} answers ${p.then} when ${describeConditions(p.when)}.`, status: 'proved' })),
    message: spec.properties.length
      ? `Lean proved all ${spec.properties.length} propert${spec.properties.length === 1 ? 'y' : 'ies'} for every request${where}, in ${seconds} s.`
      : `The model checks and verifies${where} in ${seconds} s. No property is stated, so nothing about the rules is proved.`,
  };
}

/** A Lean warning, such as an unused simp argument: reported, never a reason the proof failed. LexLean files Lean's warnings under LLV7002 with the word in the message. */
export const isWarning = (d) => d.severity === 'warning' || /^Lean rejected `[^`]+` \(warning\):/.test(d.message);

/** A `lexlean/diagnostic/1` from the hosted verifier, in the shape `parseDiagnostics` gives. */
export function fromHosted(d) {
  const p = d.primary;
  return { severity: d.severity ?? 'error', code: d.code, message: String(d.message ?? ''), span: p?.path ? { file: p.path, line: p.line_start, column: p.column_start } : null };
}

/**
 * The proof on the hosted verifier: no Docker, no image. Every module under
 * `src/` is named, as `lexlean verify --all` verifies them, so the proofs
 * module, which is not an entrypoint, is proved too, and the answer counts
 * only when each of them comes back verified. Axiom sets are corrected and
 * proved again as in the image.
 */
async function proveHosted(dir, session, { log, client }) {
  const hosted = await import('../hosted.mjs');
  client ??= hosted.hostedClient({ version: VERSION });
  const started = Date.now();
  for (let round = 1; round <= MAX_AXIOM_ROUNDS; round++) {
    writeApp(dir, session, { overwrite: true });
    log(round === 1
      ? `Asking Lean to prove the model on the hosted verifier ${client.url}, so the SDK image is not needed yet. The files under prism/src/ are sent, unlisted but not private.`
      : `Correcting an axiom set and proving again (round ${round})...`);
    let project;
    try {
      project = hosted.hostedProject(dir);
    } catch (error) {
      if (error.code !== 'hosted.layout') throw error;
      return { ok: false, message: `The hosted verifier cannot take this app (${error.message}). This is a Prism bug: the app folder should have the template's shape.` };
    }
    const all = { ...project, entrypoints: project.files.map((f) => f.path).filter((p) => hosted.moduleOf(p)) };
    let outcome;
    try {
      outcome = await hosted.hostedVerify(client, all, { onWait: (s) => s % 30 < 5 && log(`Lean is working, ${s} s...`) });
    } catch (error) {
      if (!String(error.code).startsWith('hosted.')) throw error;
      return { ok: false, message: `The hosted verifier did not prove the model: ${error.message}` };
    }
    rememberModel(dir, session);
    const seconds = Math.round((Date.now() - started) / 1000);
    if (outcome.state === 'verified') {
      const record = hosted.writeHostedRecord(dir, { ...outcome, url: client.url, entrypoints: all.entrypoints });
      return provedResult(dir, session, seconds, { where: 'hosted', attestation: outcome.record.attestation_id, record: relative(dir, record).split('\\').join('/') });
    }
    const diagnostics = outcome.diagnostics.map(fromHosted);
    const fixes = axiomViolations(diagnostics);
    if (fixes.length && diagnostics.filter((d) => !isWarning(d)).every((d) => d.code === 'LLV7005')) {
      for (const f of fixes) session.axioms[f.name] = f.axioms;
      continue;
    }
    const output = diagnostics.map((d) => `${d.severity}[${d.code}]: ${d.message}${d.span ? `\n  --> ${d.span.file}:${d.span.line}:${d.span.column}` : ''}`).join('\n');
    return explainFailure(dir, session, { run: { code: 1, output }, diagnostics, failed: { step: 'verify', code: 1 }, seconds });
  }
  return { ok: false, message: 'The axiom sets did not settle after several rounds. This is a Prism bug; the last output is in prism/.prism-last.log.' };
}

function explainFailure(dir, session, { run, diagnostics, failed, seconds }) {
  const spec = session.spec;
  const n = names(spec);
  writeFileSync(join(dir, '.prism-last.log'), run.output);
  if (run.code === 69 || run.code === 70) return { ok: false, message: tail(run.output) };
  const source = (file) => (existsSync(join(dir, file)) ? readFileSync(join(dir, file), 'utf8') : '');
  const byTheorem = Object.fromEntries(spec.properties.map((p) => [n.properties[p.id].theorem, p]));
  const failing = [];
  const other = [];
  for (const d of diagnostics) {
    if (isWarning(d)) continue;
    const decl = d.span && /\.lex\.tex$/.test(d.span.file) ? declarationAt(source(d.span.file), d.span.line) : null;
    const property = decl && byTheorem[decl];
    if (property && d.code === 'LLV7002') {
      if (!failing.includes(property)) failing.push(property);
    } else if (d.code === 'LLV7005') {
      const fix = axiomViolations([d])[0];
      if (fix) session.axioms[fix.name] = fix.axioms;
    } else other.push(d);
  }
  session.status.properties = Object.fromEntries(spec.properties.map((p) => [p.id, failing.includes(p) ? 'failing' : 'unknown']));
  writeSpecDocument(dir, session);
  if (failing.length && other.length === 0) {
    const explained = failing.map((p) => {
      const found = counterexample(spec, p);
      return found.request !== null
        ? `${p.id} is false. The request \`${found.request}\` meets "${describeConditions(p.when)}" and the rules answer ${found.outcome} (${found.rule ?? 'otherwise'}).`
        : `Lean could not prove ${p.id} ("${p.kind} ${p.then} when ${describeConditions(p.when)}"), and none of ${found.checked} sampled requests breaks it. It may hold only for requests the sampling missed, such as very large numbers. Restate it more narrowly, or drop it and keep it as a tested example.`;
    });
    return { ok: false, seconds, properties: spec.properties.map((p) => ({ id: p.id, status: session.status.properties[p.id] })), message: explained.join('\n'), next: 'Change a rule or restate the property (prism_answer with step rules or properties), then prove again.' };
  }
  const step = failed?.step ?? 'run';
  return {
    ok: false,
    seconds,
    message: `The ${step} step failed. This means Prism generated a model the toolchain refuses, which is a Prism bug, not a problem with your spec.`,
    diagnostics: (other.length ? other : diagnostics).slice(0, 8).map((d) => ({ code: d.code, message: d.message.split('\n').slice(0, 6).join('\n'), at: d.span ? `${d.span.file}:${d.span.line}` : null })),
    output: diagnostics.length ? undefined : tail(run.output),
  };
}

/** Prove, then build the holo and replay its acceptance vectors through the guest. */
export async function build(dir, session, { log = () => {}, overwrite = false } = {}) {
  const proved = session.status.proved && session.status.proved.modelSha === session.status.modelSha && existsSync(modelPath(dir, session.spec)) && session.status.lastSpec === JSON.stringify(session.spec);
  if (!proved) {
    const p = await prove(dir, session, { log, overwrite });
    session.status.lastSpec = JSON.stringify(session.spec);
    writeSession(dir, session);
    if (!p.ok) return { ...p, stage: 'prove' };
  }
  // A build needs the image, and a hosted proof did not: pull it now if it is missing, saying so, and pin the app to it.
  const ready = await prepare(dir, session, log, { pull: true });
  if (!ready.ok) return { ok: false, stage: 'build', message: ready.message };
  const started = Date.now();
  log('Building the app: Lean export, Rust, WebAssembly, the holo and its independent acceptance run. The first build takes ten minutes or more...');
  const run = await runInImage(dir, BUILD_SCRIPT, { socket: true, onLine: (l) => /^(error|warning|verified|built|wrote|PP\d|  \w)/.test(l) && log(l) });
  // The build formats the model; remember it as formatted, so the next proof does not take it for a hand edit.
  const wasProved = session.status.proved?.modelSha === session.status.modelSha;
  rememberModel(dir, session);
  // Formatting changes no meaning, so a proof of the model before it still stands.
  if (wasProved) session.status.proved.modelSha = session.status.modelSha;
  const seconds = Math.round((Date.now() - started) / 1000);
  writeFileSync(join(dir, '.prism-last.log'), run.output);
  if (run.code !== 0) {
    const diagnostics = parseDiagnostics(run.output);
    return { ok: false, stage: 'build', seconds, message: 'The build failed. The last lines are below; the whole log is prism/.prism-last.log.', diagnostics: diagnostics.slice(0, 8), output: diagnostics.length ? undefined : tail(run.output, 30) };
  }
  const holo = newestHolo(dir);
  if (!holo) return { ok: false, stage: 'build', seconds, message: 'The build passed but wrote no holo that Prism could find under prism/.prism/build.' };
  const app = await fromFile(holo);
  const acceptance = await app.acceptance();
  session.status.holo = holo;
  session.status.vectors = acceptance.total;
  session.status.built = { at: new Date().toISOString(), seconds, passed: acceptance.passed, total: acceptance.total };
  writeFileSync(join(dir, 'run-policy.json'), '{"minimum": "P0"}\n');
  writeSpecDocument(dir, session);
  const rel = relative(process.cwd(), holo) || holo;
  const policy = relative(process.cwd(), join(dir, 'run-policy.json')) || join(dir, 'run-policy.json');
  return {
    ok: acceptance.failed.length === 0,
    stage: 'build',
    seconds,
    holo,
    vectors: { passed: acceptance.passed, total: acceptance.total },
    message: acceptance.failed.length === 0 ? `Built and verified in ${seconds} s: ${acceptance.passed} of ${acceptance.total} acceptance vectors pass in the built guest.` : `Built, but ${acceptance.failed.length} of ${acceptance.total} acceptance vectors fail in the guest. That is a Prism bug: the evaluator and the model disagree.`,
    run: {
      serve: `npx prism-code run "${rel}" --policy "${policy}" --port 8080`,
      once: `npx prism-code run "${rel}" "${exampleRequest(session.spec)}" --policy "${policy}"`,
      note: 'The run policy accepts P0, the level every holo has until holos carry signed statements. The proofs were checked when it was built.',
    },
  };
}

function exampleRequest(spec) {
  return spec.examples[0]?.request ?? spec.fields.map((f) => (f.kind === 'choice' ? f.values[0] : String(f.min ?? 1))).join(' ');
}

/** Answer one request: from the built guest when there is one, else from the spec. */
export async function tryRequest(session, request) {
  const { respond, decodeRequest, judge } = await import('./spec.mjs');
  const bytes = new TextEncoder().encode(request);
  const expected = respond(session.spec, bytes);
  const decoded = decodeRequest(session.spec, request);
  const why = decoded ? judge(session.spec, decoded).rule ?? 'otherwise' : 'malformed';
  if (session.status.holo && existsSync(session.status.holo)) {
    const app = await fromFile(session.status.holo);
    const answer = new TextDecoder().decode(await app.dispatch(bytes));
    return { request, answer, from: 'the built app', decidedBy: why, agreesWithSpec: answer === expected };
  }
  return { request, answer: expected, from: 'the spec (not built yet)', decidedBy: why };
}
