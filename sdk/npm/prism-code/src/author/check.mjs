/**
 * `check <holo> --unstable-author`: prove a holo came from this spec, offline.
 * Each row is a check id the conformance suite can name
 * (docs/proposals/325-authoring-front-door.md).
 */
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { inspect } from '../inspect.mjs';
import { fromFile, readManifest } from '../node.mjs';
import { readLock, replayExamplesGate, vectorMismatch } from './build.mjs';
import { loadSpec, modelDigest, readModel } from './spec.mjs';
import { intentVerdicts, readTheorems } from './toolchain.mjs';

/** Why each part of a holo can differ from a reference built from another model. Stated for a reader; measured by the comparison. */
const WHY = Object.freeze({
  guest: 'the guest is compiled from the model, so another function body or declaration order changes its bytes',
  view: 'the view is generated from the model; examples and intent do not state its labels',
  'capability-set': 'the capability set is generated from what the model declares',
  'model-document': 'the model document carries the sources, names and limits of this model',
  'acceptance vectors': 'the vectors come from examples/, so this means examples/ differs from the reference',
});

function digestByRole(inspection) {
  return new Map(inspection.sections.filter((section) => section.role).map((section) => [section.role, section.digest?.blake3]));
}

/** Compare two holos part by part: `{ part, same, why }` rows, Core-Wasm and acceptance first. */
export async function compareHolos(referenceBytes, candidateBytes, referenceVectors, candidateVectors) {
  const reference = digestByRole(await inspect(referenceBytes));
  const candidate = digestByRole(await inspect(candidateBytes));
  const rows = ['guest', 'view', 'capability-set', 'model-document'].map((part) => {
    const same = reference.get(part) === candidate.get(part);
    return { part, same, why: same ? null : WHY[part] };
  });
  const encode = (vectors) => JSON.stringify(vectors.map((v) => [Buffer.from(v.request).toString('hex'), Buffer.from(v.response).toString('hex')]));
  const same = encode(referenceVectors) === encode(candidateVectors);
  rows.splice(1, 0, { part: 'acceptance vectors', same, why: same ? null : WHY['acceptance vectors'] });
  return rows;
}

/**
 * Check a holo against the project that claims it. Resolves to
 * `{ passed, checks, rows, comparison }`.
 *
 * @param {string} holo the holo's path
 * @param {{ root: string, reference?: string }} options
 */
export async function checkAuthoring(holo, { root, reference = null }) {
  const spec = loadSpec(root);
  const checks = [];
  const record = (id, passed, message) => checks.push({ id, passed, message });
  if (spec.level === 3) {
    record('author.level3', true, 'no intent.md or examples/: a hand-written model, nothing to prove against a spec');
    return { passed: true, checks, rows: null, comparison: null, level: 3 };
  }
  const lock = readLock(root).value;
  const authoring = lock.authoring ?? null;

  record('author.lock.record', authoring !== null, authoring ? 'prismpm.lock records how the model was written' : 'prismpm.lock has no authoring record; run build --unstable-author');
  const specMatches = authoring?.spec?.blake3 === spec.digest;
  record('author.spec.digest', specMatches, specMatches ? `the spec is the one the model was written from, blake3:${spec.digest}` : `the spec is blake3:${spec.digest}, and the model was written from blake3:${authoring?.spec?.blake3}`);
  const model = readModel(root);
  const treeDigest = model ? modelDigest(model) : null;
  const treeMatches = authoring?.model_tree?.blake3 === treeDigest && treeDigest !== null;
  record('author.model.digest', treeMatches, treeMatches ? `model/ is the tree the lock pins, blake3:${treeDigest}` : `model/ is blake3:${treeDigest}, and the lock pins blake3:${authoring?.model_tree?.blake3}`);

  const bytes = new Uint8Array(readFileSync(resolve(holo)));
  const { grant } = await readManifest(root);
  const app = await fromFile(resolve(holo), grant);
  const inspected = await inspect(bytes);
  record('author.holo.closed', app.identities.verified, app.identities.verified ? 'the holo matches its own digests and loads under the capabilities prism.toml grants' : 'the holo does not match its own digests');
  const holoModel = inspected.model?.digest?.sha256 ?? null;
  const pinned = lock.inputs?.model?.sha256 ?? null;
  record('author.holo.model', holoModel !== null && holoModel === pinned, holoModel === pinned ? 'the holo carries the model the lock pins' : `the holo carries model ${holoModel}, and the lock pins ${pinned}`);

  const mismatch = vectorMismatch(spec.examples, app.vectors);
  record('author.examples.vectors', mismatch === null, mismatch ? `${mismatch.example ?? 'examples/'}: ${mismatch.reason}` : `every file in examples/ is an acceptance vector of the holo, in order`);
  const replay = await replayExamplesGate(app, spec.examples);
  record(
    'author.examples.replay',
    replay.failed.length === 0 && replay.events > 0,
    replay.failed.length === 0 ? `${replay.events} of ${replay.events} replayed through the guest` : `${replay.failed[0].example ?? 'examples/'}: ${replay.failed[0].reason}`,
  );

  const theorems = pinned ? readTheorems(root, pinned) : null;
  const rows = intentVerdicts(spec.intent, theorems);
  const proved = rows.every((row) => row.verdict === 'checked');
  record('author.intent.verdicts', proved, proved ? `the kernel accepted a theorem for each of ${rows.length} sentences` : theorems ? 'a sentence has no accepted theorem' : 'no kernel evidence for this model on this machine; run verify');

  let comparison = null;
  if (reference) {
    const referenceBytes = new Uint8Array(readFileSync(resolve(reference)));
    const referenceApp = await fromFile(resolve(reference));
    comparison = await compareHolos(referenceBytes, bytes, referenceApp.vectors, app.vectors);
  }
  return { passed: checks.every((check) => check.passed), checks, rows, comparison };
}
