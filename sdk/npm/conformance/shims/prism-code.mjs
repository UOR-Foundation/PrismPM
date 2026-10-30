#!/usr/bin/env node
/**
 * The entrypoint shim for `prism-code`, the JavaScript runtime.
 *
 *   node shims/prism-code.mjs <step-file>
 *
 * Reads one `prism/conformance-step/1` file, runs the matching `prism-code`
 * verb, and prints one `prism/conformance-outcome/1` object: the runtime's
 * exit code, the JSON it printed (or null) and its refusal (or null). The
 * shim exits 0 whenever it produced an outcome, whatever the runtime's exit
 * was, and 2 when it cannot understand the step.
 *
 * `prism-code` is found through PRISM_CODE (the path to its bin/prism-code.mjs),
 * then the repository sibling sdk/npm/prism-code, then the installed package.
 * The contract is in docs/proposals/287-conformance.md.
 */
import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const STEP_SCHEMA = 'prism/conformance-step/1';
const OUTCOME_SCHEMA = 'prism/conformance-outcome/1';
const NAME = 'prism-code';
/** How prism-code prints a refusal: an optional prefix, a dotted code, a message. */
const REFUSAL = /^(?:no verdict: |could not replay: )?([a-z][a-z0-9-]*(?:\.[a-z][a-z0-9-]*)+): (.*)$/;

function die(message) {
  process.stderr.write(`prism-code shim: ${message}\n`);
  process.exit(2);
}

function locate() {
  if (process.env.PRISM_CODE) {
    if (!existsSync(process.env.PRISM_CODE)) die(`PRISM_CODE names ${process.env.PRISM_CODE}, which does not exist`);
    return resolve(process.env.PRISM_CODE);
  }
  // In the repository the sibling is the runtime under test; an installed copy anywhere up the tree is older.
  const sibling = fileURLToPath(new URL('../../prism-code/bin/prism-code.mjs', import.meta.url));
  if (existsSync(sibling)) return sibling;
  try {
    const require = createRequire(import.meta.url);
    return join(dirname(require.resolve('prism-code/package.json')), 'bin', 'prism-code.mjs');
  } catch {
    return die('prism-code is not installed and sdk/npm/prism-code is not beside this package; set PRISM_CODE');
  }
}

function versionOf(bin) {
  return JSON.parse(readFileSync(join(dirname(bin), '..', 'package.json'), 'utf8')).version;
}

function run(bin, args, cwd) {
  const child = spawnSync(process.execPath, [bin, ...args], { cwd, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, windowsHide: true });
  if (child.error) die(`prism-code could not run: ${child.error.message}`);
  return { exit: child.status ?? 70, stdout: child.stdout ?? '', stderr: child.stderr ?? '' };
}

function json(text) {
  const trimmed = text.trim();
  if (!trimmed) return null;
  try {
    return JSON.parse(trimmed);
  } catch {
    return null;
  }
}

/** The refusal prism-code printed, from the first line of stderr, or null. */
function refusal(stderr) {
  const line = stderr.split(/\r?\n/).find((l) => l.trim());
  if (!line) return null;
  const match = REFUSAL.exec(line.trim());
  return match ? { code: match[1], message: match[2] } : { message: line.trim() };
}

function outcome(bin, run, result, error) {
  return { schema: OUTCOME_SCHEMA, runtime: { name: NAME, version: versionOf(bin) }, exit: run.exit, result, error };
}

function main() {
  const [path] = process.argv.slice(2);
  if (!path) die('takes one step file');
  let step;
  try {
    step = JSON.parse(readFileSync(path, 'utf8'));
  } catch (cause) {
    die(`${path} is not a JSON step file: ${cause.message}`);
  }
  if (step?.schema !== STEP_SCHEMA) die(`${path} must declare schema ${STEP_SCHEMA}`);
  if (typeof step.archive !== 'string') die('the step names no archive');
  const bin = locate();
  const cwd = step.scratch ?? process.cwd();
  let out;
  switch (step.action) {
    case 'inspect': {
      const r = run(bin, ['inspect', step.archive, '--json'], cwd);
      out = outcome(bin, r, r.exit === 0 ? json(r.stdout) : null, r.exit === 0 ? null : refusal(r.stderr));
      break;
    }
    case 'vectors': {
      const r = run(bin, ['vectors', step.archive], cwd);
      const match = /^(\d+) of (\d+) vectors passed/m.exec(r.stdout);
      const result = match ? { passed: Number(match[1]), total: Number(match[2]), failed: Number(match[2]) - Number(match[1]) } : null;
      out = outcome(bin, r, result, result && r.exit === 0 ? null : refusal(r.stderr) ?? { message: r.stdout.trim() || `exit ${r.exit}` });
      break;
    }
    case 'history': {
      const file = join(cwd, 'history.json');
      const r = run(bin, ['history', step.archive, '--output', file], cwd);
      const result = r.exit === 0 && existsSync(file) ? json(readFileSync(file, 'utf8')) : null;
      out = outcome(bin, r, result, r.exit === 0 ? null : refusal(r.stderr));
      break;
    }
    case 'replay': {
      if (!Array.isArray(step.histories) || step.histories.length === 0) die('replay takes histories');
      const r = run(bin, ['replay', ...step.histories, '--holo', step.archive, '--json'], cwd);
      const result = r.exit === 0 || r.exit === 1 ? json(r.stdout) : null;
      out = outcome(bin, r, result, r.exit === 0 ? null : refusal(r.stderr) ?? (r.exit === 1 ? { code: 'replay.divergence', message: 'the replay diverged' } : null));
      break;
    }
    case 'check': {
      const args = ['check', step.archive, '--json'];
      if (step.policy !== undefined) {
        const file = join(cwd, 'policy.json');
        writeFileSync(file, `${JSON.stringify(step.policy)}\n`);
        args.push('--policy', file);
      }
      const r = run(bin, args, cwd);
      const result = r.exit === 0 || r.exit === 1 ? json(r.stdout) : null;
      out = outcome(bin, r, result, r.exit === 0 ? null : refusal(r.stderr) ?? (r.exit === 1 ? { code: 'policy.failed', message: 'the policy failed' } : null));
      break;
    }
    default:
      die(`unknown action ${JSON.stringify(step.action)}; this shim knows inspect, vectors, history, replay and check`);
  }
  process.stdout.write(`${JSON.stringify(out)}\n`);
}

main();
