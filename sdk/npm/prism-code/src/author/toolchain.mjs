/**
 * The toolchain seam of `build --unstable-author` and the evidence it leaves.
 * The toolchain is the pinned SDK image, reached as every delegated verb is;
 * a test passes another object with the same `run`. The evidence is read back
 * in the shape `prism/proof/v1` lists its theorems, so the join below works on
 * that predicate unchanged once a holo carries it (#267).
 */
import { spawn } from 'node:child_process';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { hostDockerArguments, sdkImageFromLock } from '../docker.mjs';
import { parseToml } from '../toml.mjs';

const MAX_OUTPUT = 256 * 1024;
export const NO_DOCKER = 69;

/**
 * Runs `prismpm <command>` in the image `prismpm.lock` names. Output goes to
 * stderr as it arrives and the tail comes back, since the agent reads it as
 * diagnostics. `{ code, output }`; `code` is 69 when Docker is not on the PATH.
 */
export function dockerToolchain(root) {
  return {
    run: (command, args = []) =>
      new Promise((resolve) => {
        const image = sdkImageFromLock(readFileSync(join(root, 'prismpm.lock'), 'utf8'));
        const child = spawn('docker', hostDockerArguments({ image, cwd: root, command, args }), { stdio: ['ignore', 'pipe', 'pipe'], windowsHide: true });
        let output = '';
        const collect = (chunk) => {
          process.stderr.write(chunk);
          output = (output + chunk.toString('utf8')).slice(-MAX_OUTPUT);
        };
        child.stdout.on('data', collect);
        child.stderr.on('data', collect);
        child.on('error', (error) => resolve({ code: error.code === 'ENOENT' ? NO_DOCKER : 1, output: error.message }));
        child.on('close', (code, signal) => resolve({ code: code ?? (signal ? 1 : 0), output }));
      }),
  };
}

function readJson(path) {
  try {
    return JSON.parse(readFileSync(path, 'utf8'));
  } catch {
    return null;
  }
}

/** The configured build root; `.prism` when the project does not say. */
function buildRoot(root) {
  const path = join(root, 'prismpm.toml');
  return existsSync(path) ? (parseToml(readFileSync(path, 'utf8')).build_root ?? '.prism') : '.prism';
}

const directories = (path) => (existsSync(path) ? readdirSync(path) : []);

/** The holo the build of this model published, by its build manifest's `inputs.model_id`; null if none. */
export function findHolo(root, modelSha256) {
  const parent = join(root, buildRoot(root), 'build');
  for (const id of directories(parent)) {
    const manifest = readJson(join(parent, id, 'manifest.json'));
    if (manifest?.inputs?.model_id !== modelSha256) continue;
    const row = manifest.files?.find((file) => /^[^/]+\.holo$/.test(file.path));
    if (row) return join(parent, id, row.path);
  }
  return null;
}

/**
 * What the kernel accepted for this model, as `prism/proof/v1` `theorems[]`
 * rows: `{ name, verdict, axioms }`. Read from the verified directory whose
 * manifest names the model (SPEC section 8). Null when there is none.
 */
export function readTheorems(root, modelSha256) {
  const parent = join(root, buildRoot(root), 'verified');
  for (const id of directories(parent)) {
    if (readJson(join(parent, id, 'manifest.json'))?.model_sha256 !== modelSha256) continue;
    const attestation = readJson(join(parent, id, 'lexlean-attestation.json'));
    if (attestation?.status !== 'verified' || !Array.isArray(attestation.declarations)) continue;
    return attestation.declarations.map(({ name, result, observed, policy }) => ({
      name,
      verdict: result === 'ok' ? 'PASSED' : 'FAILED',
      axioms: { policy, observed },
    }));
  }
  return null;
}

/**
 * Join sentences to the kernel's rows. A sentence is `checked` only when its
 * theorem is there, passed, and leans on no `sorryAx`; the agent's claim plays
 * no part. Returns `{ line, sentence, theorem, verdict, why }` rows, where
 * `verdict` is `checked` or `UNPROVED`.
 */
export function intentVerdicts(intent, theorems) {
  return intent.map(({ line, sentence, theorem }) => {
    const row = theorems?.find(({ name }) => name === theorem || name.endsWith(`.${theorem}`));
    let why = null;
    if (!theorems) why = 'no kernel evidence for this model';
    else if (!row) why = 'the model has no theorem for this sentence';
    else if (row.verdict !== 'PASSED') why = 'the kernel did not accept its theorem';
    else if (row.axioms?.observed?.includes('sorryAx')) why = 'its proof leans on sorry';
    return { line, sentence, theorem, verdict: why ? 'UNPROVED' : 'checked', why };
  });
}
