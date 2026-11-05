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
import { BUILD_SCRIPT, DEFAULT_SDK_IMAGE, axiomViolations, declarationAt, hasImage, parseDiagnostics, resolveDocker, runInImage, steps } from './sdk.mjs';

const PROVE_SCRIPT_FULL = [
  'echo "@@step prelock"; lexlean lock 2>&1; code=$?; echo "@@exit prelock $code"; [ $code -eq 0 ] || exit $code',
  'echo "@@step format"; lexlean fmt --all 2>&1; code=$?; echo "@@exit format $code"; [ $code -eq 0 ] || exit $code',
  'echo "@@step lock"; lexlean lock 2>&1; code=$?; echo "@@exit lock $code"; [ $code -eq 0 ] || exit $code',
  'echo "@@step check"; prismpm check 2>&1; code=$?; echo "@@exit check $code"; [ $code -eq 0 ] || exit $code',
  'echo "@@step verify"; lexlean verify --all 2>&1; code=$?; echo "@@exit verify $code"; exit $code',
].join('; ');

const MAX_AXIOM_ROUNDS = 8;

/** Make sure Docker, the image and the project's SDK pin are in place. */
export async function prepare(dir, session, log) {
  const d = resolveDocker();
  if (d.error) return { ok: false, message: d.error };
  if (!isBound(dir)) {
    const image = session.image ?? DEFAULT_SDK_IMAGE;
    if (!hasImage(image)) return { ok: false, message: `The SDK image is not on this machine. Run \`npx prism-code setup\` in a terminal to download it (about 21 GB), then try again.\n  ${image}` };
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
export async function prove(dir, session, { log = () => {}, overwrite = false } = {}) {
  const first = writeApp(dir, session, { overwrite });
  if (first.edited) return { ok: false, edited: true, message: `${relative(dir, first.path)} was edited by hand since Prism wrote it. Call the same tool again with overwrite: true to replace it from the spec, or change the spec to match.` };
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
    if (run.code === 0) {
      session.status.properties = Object.fromEntries(spec.properties.map((p) => [p.id, 'proved']));
      session.status.proved = { modelSha: session.status.modelSha, at: new Date().toISOString(), seconds };
      writeSpecDocument(dir, session);
      return {
        ok: true,
        seconds,
        properties: spec.properties.map((p) => ({ id: p.id, theorem: n.properties[p.id].theorem, says: `It ${p.kind} answers ${p.then} when ${describeConditions(p.when)}.`, status: 'proved' })),
        message: spec.properties.length
          ? `Lean proved all ${spec.properties.length} propert${spec.properties.length === 1 ? 'y' : 'ies'} for every request, in ${seconds} s.`
          : `The model checks and verifies in ${seconds} s. No property is stated, so nothing about the rules is proved.`,
      };
    }
    return explainFailure(dir, session, { run, diagnostics, failed, seconds });
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
  const started = Date.now();
  log('Building the app: Lean export, Rust, WebAssembly, the holo and its independent acceptance run. The first build takes ten minutes or more...');
  const run = await runInImage(dir, BUILD_SCRIPT, { socket: true, onLine: (l) => /^(error|warning|verified|built|wrote|PP\d|  \w)/.test(l) && log(l) });
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
