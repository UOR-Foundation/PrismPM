import { spawnSync, spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { blake3Hex, resolveArchive } from './runtime.mjs';

const BIN = fileURLToPath(import.meta.resolve('prism-code/package.json')).replace(/package\.json$/, 'bin/prism-code.mjs');

/** Run the `prism-code` command line of the installed package to completion: `{ code, out, err }`. */
export function runCli(args, { env = {}, input } = {}) {
  const result = spawnSync(process.execPath, [BIN, ...args], { encoding: 'utf8', input, env: { ...process.env, ...env } });
  return { code: result.status, out: result.stdout, err: result.stderr };
}

/** Start the command line without waiting, for a command that serves until it is stopped. */
export function spawnCli(args, { env = {} } = {}) {
  return spawn(process.execPath, [BIN, ...args], { stdio: ['ignore', 'pipe', 'pipe'], env: { ...process.env, ...env } });
}

/** Both digests of an opened holo's guest and model blobs, the subjects of a `prism/replay/v1` statement. */
export async function buildDigests(app) {
  const { guest, model } = resolveArchive(app.archive);
  const digests = (blob) => ({ blake3: blake3Hex(blob.content), sha256: createHash('sha256').update(blob.content).digest('hex') });
  return { guest: digests(guest), model: digests(model) };
}
