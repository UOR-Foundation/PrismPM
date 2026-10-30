/**
 * The runtimes the matrix has a column for (`runtimes.json`) and each one's
 * declared expected failures (`xfail/<runtime>.json`).
 */
import { join } from 'node:path';
import { fail, isObject, readJson } from './util.mjs';
import { ID, PACKAGE_ROOT } from './scenarios.mjs';

export const RUNTIMES_SCHEMA = 'prism/conformance-runtimes/1';
export const XFAIL_SCHEMA = 'prism/conformance-xfail/1';
const RUNTIME_MEMBERS = Object.freeze(['language', 'entrypoint', 'xfail']);
const XFAIL_MEMBERS = Object.freeze(['schema', 'runtime', 'xfail', 'notApplicable']);
const EXPANDED_ID = /^[a-z][a-z0-9]*(-[a-z0-9]+)*(\[[A-Za-z0-9_.-]+\])?$/;

export function loadRuntimes(root = PACKAGE_ROOT) {
  const path = join(root, 'runtimes.json');
  const file = readJson(path);
  if (!isObject(file) || file.schema !== RUNTIMES_SCHEMA) fail('runtimes.schema', `${path} must declare schema ${RUNTIMES_SCHEMA}`);
  if (!isObject(file.runtimes) || Object.keys(file.runtimes).length === 0) fail('runtimes.empty', `${path} names no runtime`);
  for (const [name, runtime] of Object.entries(file.runtimes)) {
    if (!ID.test(name)) fail('runtimes.name', `${path}: runtime ${name} is not lowercase words joined by hyphens`);
    if (!isObject(runtime)) fail('runtimes.member', `${path}: runtime ${name} is not an object`);
    const unknown = Object.keys(runtime).filter((key) => !RUNTIME_MEMBERS.includes(key));
    if (unknown.length > 0) fail('runtimes.member', `${path}: runtime ${name} has unknown members ${unknown.join(', ')}`);
    if (typeof runtime.language !== 'string' || !runtime.language) fail('runtimes.language', `${path}: runtime ${name} names no language`);
    if (runtime.entrypoint !== null && (!Array.isArray(runtime.entrypoint) || runtime.entrypoint.length === 0 || !runtime.entrypoint.every((a) => typeof a === 'string' && a))) {
      fail('runtimes.entrypoint', `${path}: runtime ${name}: entrypoint is null or a non-empty array of strings`);
    }
    if (typeof runtime.xfail !== 'string' || !runtime.xfail.startsWith('xfail/') || !runtime.xfail.endsWith('.json')) {
      fail('runtimes.xfail', `${path}: runtime ${name}: xfail names a file under xfail/`);
    }
  }
  return file;
}

/** The expected failures of one runtime: `{ runtime, xfail: { id: reason }, notApplicable: { feature: reason } }`. */
export function loadXfail(root, name, runtimes = loadRuntimes(root)) {
  const runtime = runtimes.runtimes[name];
  if (!runtime) fail('runtimes.unknown', `runtime ${name} is not in runtimes.json (${Object.keys(runtimes.runtimes).join(', ')})`);
  const path = join(root, runtime.xfail);
  const file = readJson(path);
  if (!isObject(file) || file.schema !== XFAIL_SCHEMA) fail('xfail.schema', `${path} must declare schema ${XFAIL_SCHEMA}`);
  const unknown = Object.keys(file).filter((key) => !XFAIL_MEMBERS.includes(key));
  if (unknown.length > 0) fail('xfail.member', `${path} has unknown members ${unknown.join(', ')}`);
  if (file.runtime !== name) fail('xfail.runtime', `${path} declares runtime ${JSON.stringify(file.runtime)}, expected ${name}`);
  for (const member of ['xfail', 'notApplicable']) {
    if (!isObject(file[member])) fail('xfail.member', `${path}: ${member} must be an object of reasons`);
    for (const [key, reason] of Object.entries(file[member])) {
      if (typeof reason !== 'string' || !reason.trim()) fail('xfail.reason', `${path}: ${member} ${key} has no reason`);
      if (member === 'xfail' && !EXPANDED_ID.test(key)) fail('xfail.id', `${path}: ${key} is not a scenario id or an expanded one such as load-archive[Calculator]`);
      if (member === 'notApplicable' && !ID.test(key)) fail('xfail.feature', `${path}: ${key} is not a feature name`);
    }
  }
  return file;
}

/**
 * Check the declarations against the suite: every xfail names a scenario or
 * an expanded run, every notApplicable names a feature.
 */
export function checkXfail(xfail, { runs, features }) {
  const ids = new Set(runs.flatMap((run) => [run.id, run.scenario.id]));
  for (const id of Object.keys(xfail.xfail)) {
    if (!ids.has(id)) fail('xfail.unknown', `${xfail.runtime} expects ${id} to fail, but no scenario has that id`);
  }
  for (const feature of Object.keys(xfail.notApplicable)) {
    if (!Object.hasOwn(features.features, feature)) fail('xfail.unknown', `${xfail.runtime} declares feature ${feature} not applicable, but features.json does not declare it`);
  }
  return xfail;
}

/**
 * The command line that runs a shim: `node` is this Node, a relative script
 * path is resolved against the package root, and anything else is passed as
 * written. An override from the command line replaces the registered one.
 */
export function resolveEntrypoint(root, entrypoint, override = null) {
  const argv = override ?? entrypoint;
  if (!argv) return null;
  return argv.map((argument, index) => {
    if (index === 0 && argument === 'node') return process.execPath;
    if (!override && /^[a-z]/.test(argument) && argument.includes('/') && !argument.startsWith('-')) return join(root, argument);
    return argument;
  });
}
