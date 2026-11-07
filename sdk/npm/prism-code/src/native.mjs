/**
 * The fast check, `build --check`, without Docker.
 *
 * `prismpm check` is in-process Rust: the LexLean snapshot, the Holo projection
 * and their validation, in memory, on one thread. It starts no Lean, no cargo
 * and no other program, so the prismpm that `prismpm.lock` pins gives the same
 * verdict on the host as in the SDK image. Measured on Windows 11
 * (2026-11-06): 0.7 s for the Calculator example, against a 5.5 GB image pull
 * before the first check. Only `check` runs on the host; build, verify and
 * conformance compile and stay in the image, which stays the gate.
 *
 * A host prismpm is used only when its `--version` is the version the lock
 * pins. Candidates, first match wins: `$PRISM_PRISMPM`, then
 * `$PRISM_HOME/bin/prismpm-<version>/prismpm[.exe]`, then `prismpm` on the
 * PATH. `PRISM_NATIVE=0` always uses the image. Pure functions here; `bin/`
 * spawns.
 */
import { join } from 'node:path';
import { prismHome } from './agents.mjs';

/**
 * The prismpm version a `prismpm.lock` pins, read from each platform's
 * inventory. Null when the lock cannot be read, pins none, or its platforms
 * disagree: then only the image can be trusted to match.
 */
export function lockedPrismpmVersion(lockText) {
  let lock;
  try {
    lock = JSON.parse(lockText);
  } catch {
    return null;
  }
  const versions = new Set();
  for (const platform of Array.isArray(lock?.platforms) ? lock.platforms : []) {
    for (const item of Array.isArray(platform?.inventory) ? platform.inventory : []) {
      if (item?.id === 'prismpm' && item.kind === 'binary' && typeof item.version === 'string') versions.add(item.version);
    }
  }
  return versions.size === 1 ? [...versions][0] : null;
}

/** The version a `prismpm --version` line names, or null. */
export function parsePrismpmVersion(stdout) {
  const m = /^prismpm (\S+)\s*$/.exec(String(stdout ?? ''));
  return m ? m[1] : null;
}

/** Where a host prismpm is looked for, in order. */
export function nativeCandidates({ version, env = process.env, platform = process.platform } = {}) {
  const exe = platform === 'win32' ? 'prismpm.exe' : 'prismpm';
  const candidates = [];
  if (env.PRISM_PRISMPM) candidates.push({ path: env.PRISM_PRISMPM, source: 'PRISM_PRISMPM' });
  candidates.push({ path: join(prismHome(env), 'bin', `prismpm-${version}`, exe), source: 'cache' });
  candidates.push({ path: exe, source: 'PATH' });
  return candidates;
}

/**
 * Choose the host prismpm for `check`. `probe(path)` runs `<path> --version`
 * and returns its stdout, or null when it does not run.
 *
 * Returns `{ path }` to run on the host, or `{ path: null, notes }` to use the
 * image; each note says why a candidate that ran was passed over.
 */
export function resolveNativePrismpm({ lockText, probe, env = process.env, platform = process.platform }) {
  if (env.PRISM_NATIVE === '0') return { path: null, notes: [] };
  const version = lockedPrismpmVersion(lockText);
  if (!version) return { path: null, notes: [] };
  const notes = [];
  for (const candidate of nativeCandidates({ version, env, platform })) {
    const stdout = probe(candidate.path);
    if (stdout === null || stdout === undefined) {
      if (candidate.source === 'PRISM_PRISMPM') notes.push(`PRISM_PRISMPM=${candidate.path} does not run`);
      continue;
    }
    const found = parsePrismpmVersion(stdout);
    if (found === version) return { path: candidate.path, version, notes };
    notes.push(`${candidate.path} is prismpm ${found ?? '(unknown version)'}, and prismpm.lock pins ${version}`);
  }
  return { path: null, version, notes };
}
