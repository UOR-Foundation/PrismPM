#!/usr/bin/env node
// The release route of the Prism factory (factory/src/Factory/Release.lex.tex).
//
// An obligation is established once for each content identity κ of its
// inputs. κ is the SHA-256 of the source tree without the release plumbing:
// the workflow and scripts that carry evidence from stage to stage and the
// tests that pin them. Those paths are not inputs of source V&V; the plumbing
// check of every release run executes their own tests fresh. When an earlier
// release run of an ancestor commit with the same κ passed source V&V, its
// retained evidence is the receipt and the obligation routes to "proved"
// instead of being established again.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {execFileSync, spawnSync} from 'node:child_process';
import {readFileSync, writeFileSync} from 'node:fs';
import {join, resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {parts as vvParts} from './vv-assemble.mjs';

const repository = 'UOR-Foundation/PrismPM';
const releaseRef = 'refs/heads/release/v0.4';

// Mirrors `plumbing` in factory/src/Factory/Release.lex.tex; factory.test.mjs
// keeps the two equal. Entries ending in `/` are directory prefixes.
export const plumbing = Object.freeze([
  '.github/workflows/release-qualify.yml',
  '.github/workflows/release.yml',
  'factory/',
  'scripts/browser-api-sdk-check.test.mjs',
  'scripts/ci-capacity.test.mjs',
  'scripts/factory.mjs',
  'scripts/factory.test.mjs',
  'scripts/library-sdk-check.test.mjs',
  'scripts/product-sdk-check.test.mjs',
  'scripts/release-gate-evidence.mjs',
  'scripts/release-gate-evidence.test.mjs',
  'scripts/release-native-contract.test.mjs',
  'scripts/release-phases.md',
  'scripts/release-phases.mjs',
  'scripts/release-phases.test.mjs',
  'scripts/release-rebuild.sh',
  'scripts/rust-oracle-tools-inputs.test.mjs',
  'scripts/sdk-candidate-sbom.test.mjs',
  'scripts/sdk-construction-handoff.test.mjs',
  'scripts/sdk-image-inputs.test.mjs',
  'scripts/sdk-release-evidence.mjs',
  'scripts/sdk-release-evidence.test.mjs',
  'scripts/sdk-vv-check.test.mjs',
  'scripts/unpublished-sdk-qualification.test.mjs',
]);

export const isPlumbing = path => plumbing.some(entry => entry.endsWith('/') ? path.startsWith(entry) : path === entry);

const git = (root, args) => execFileSync('git', args, {cwd: root, maxBuffer: 256 * 1024 * 1024});
const oid = value => assert.match(value, /^[0-9a-f]{40}$/);

// κ of one commit: every tracked entry (mode, type, object id, path) of its
// tree in Git's order, without the plumbing.
export function kappa(root, revision) {
  oid(revision);
  const listing = git(root, ['ls-tree', '-r', '-z', '--full-tree', revision]);
  const hash = createHash('sha256');
  let entries = 0;
  for (const row of listing.toString('utf8').split('\0')) {
    if (row === '') continue;
    const match = row.match(/^([0-7]{6}) (blob|commit) ([0-9a-f]{40})\t(.+)$/s);
    assert(match, 'unexpected Git tree entry');
    if (isPlumbing(match[4])) continue;
    hash.update(`${match[1]} ${match[2]} ${match[3]}\t${match[4]}\n`);
    entries += 1;
  }
  assert(entries > 0, 'empty source tree');
  return hash.digest('hex');
}

// The plumbing's own tests: run fresh by every release, since κ leaves them out.
export function plumbingTests(root) {
  const files = git(root, ['ls-files', '-z']).toString('utf8').split('\0');
  return files.filter(path => path.endsWith('.test.mjs') && isPlumbing(path)).sort();
}

export const isAncestor = (root, ancestor, revision) =>
  spawnSync('git', ['merge-base', '--is-ancestor', ancestor, revision], {cwd: root}).status === 0;

// A receipt is the retained source V&V of another run of this workflow on the
// release branch whose gate passed, for this commit or an ancestor with the
// same κ. The tag run of a rehearsed commit is proved by that rehearsal.
export function findReceipt(root, revision, client, runId = '') {
  const current = kappa(root, revision);
  const runs = client.api(`repos/${repository}/actions/workflows/release.yml/runs?branch=release/v0.4&per_page=50`).workflow_runs;
  for (const run of runs) {
    if (!['push', 'workflow_dispatch'].includes(run.event) || run.head_branch !== 'release/v0.4') continue;
    if (run.repository?.full_name !== repository || run.head_repository?.full_name !== repository) continue;
    if (run.path !== '.github/workflows/release.yml' && !String(run.path).startsWith('.github/workflows/release.yml@')) continue;
    if (String(run.id) === String(runId)) continue;
    if (!/^[0-9a-f]{40}$/.test(run.head_sha) || !isAncestor(root, run.head_sha, revision)) continue;
    if (kappa(root, run.head_sha) !== current) continue;
    // The run must have established the evidence itself: every source part
    // ran and passed, and its gate passed. A run that was itself proved by a
    // receipt carries that receipt's evidence, not its own.
    const jobs = client.api(`repos/${repository}/actions/runs/${run.id}/jobs?filter=latest&per_page=100`).jobs;
    const passed = name => {const rows = jobs.filter(job => job.name === name); return rows.length === 1 && rows[0].conclusion === 'success';};
    if (!passed('gate') || !vvParts.every(part => passed(`source-vv (${part})`))) continue;
    const artifacts = client.api(`repos/${repository}/actions/runs/${run.id}/artifacts?name=source-vv`).artifacts;
    if (!artifacts.some(artifact => artifact.name === 'source-vv' && !artifact.expired)) continue;
    return {route: 'proved', kappa: current, source_revision: run.head_sha,
      run_id: String(run.id), run_attempt: String(run.run_attempt)};
  }
  return {route: 'establish', kappa: current, source_revision: revision, run_id: '', run_attempt: ''};
}

// The build receipt. Images, their reproducibility, the native archives and
// the installed-SDK check are established once per build identity: κ plus the
// release workflow's definitions of those jobs and the plumbing they execute.
// When another release-line run of this commit or an ancestor with the same
// build identity passed every one of those jobs and retains their outputs,
// those outputs are the receipt and the jobs are not run again: publication
// packs the evidence of the run that established it, as for source V&V.
export const buildJobs = Object.freeze({'image-platforms': 2, images: 6, reproducibility: 12, native: 2, 'installed-sdk': 2});
export const buildPlumbing = Object.freeze(['scripts/release-gate-evidence.mjs', 'scripts/release-phases.mjs', 'scripts/release-rebuild.sh']);
// Every output publication downloads from the build: name or name pattern, and how many.
export const buildArtifacts = Object.freeze([
  [/^[a-z-]+-image$/, 6], [/^[a-z-]+-supply-chain$/, 6], [/^native-(x86_64|aarch64)-unknown-linux-gnu$/, 2],
  [/^reproducibility-(?!diagnostics-)[a-z-]+-(amd64|arm64)$/, 12], [/^product-cli-sdk-(amd64|arm64)$/, 2],
  [/^browser-sdk-sdk-(amd64|arm64)$/, 2], [/^library-sdk-sdk-(amd64|arm64)$/, 2], [/^sdk-equivalence-(amd64|arm64)$/, 2],
]);

// The release workflow split into its header and one block per job, as written.
export function workflowJobs(text) {
  const lines = text.split('\n'), start = lines.indexOf('jobs:');
  assert(start > 0, 'release workflow without jobs');
  const blocks = {header: lines.slice(0, start).join('\n')};
  let name = null;
  for (const line of lines.slice(start + 1)) {
    const match = line.match(/^  ([a-z0-9-]+):$/);
    if (match) { assert(!(match[1] in blocks), `duplicate job ${match[1]}`); name = match[1]; blocks[name] = ''; }
    if (name !== null) blocks[name] += `${line}\n`;
  }
  return blocks;
}

export function buildIdentity(root, revision) {
  oid(revision);
  const jobs = workflowJobs(git(root, ['show', `${revision}:.github/workflows/release.yml`]).toString('utf8'));
  const hash = createHash('sha256').update(`kappa ${kappa(root, revision)}\n`).update(`header ${jobs.header}\n`);
  for (const name of Object.keys(buildJobs)) {
    assert(typeof jobs[name] === 'string', `release workflow lacks ${name}`);
    hash.update(`job ${name}\n${jobs[name]}`);
  }
  for (const path of buildPlumbing) hash.update(`blob ${path} ${git(root, ['rev-parse', `${revision}:${path}`]).toString('utf8').trim()}\n`);
  return hash.digest('hex');
}

const releaseRun = run => ['push', 'workflow_dispatch'].includes(run.event) && run.head_branch === 'release/v0.4'
  && run.repository?.full_name === repository && run.head_repository?.full_name === repository
  && (run.path === '.github/workflows/release.yml' || String(run.path).startsWith('.github/workflows/release.yml@'))
  && /^[0-9a-f]{40}$/.test(run.head_sha);

export function findBuildReceipt(root, revision, client, runId = '') {
  const current = buildIdentity(root, revision);
  const runs = client.api(`repos/${repository}/actions/workflows/release.yml/runs?branch=release/v0.4&per_page=50`).workflow_runs;
  for (const run of runs) {
    if (!releaseRun(run) || String(run.id) === String(runId) || !isAncestor(root, run.head_sha, revision)) continue;
    if (buildIdentity(root, run.head_sha) !== current) continue;
    // Every build job ran and passed in this run: a run that was itself
    // proved by a receipt skipped them and is never a receipt.
    const jobs = client.api(`repos/${repository}/actions/runs/${run.id}/jobs?filter=latest&per_page=100`).jobs;
    if (!Object.entries(buildJobs).every(([name, count]) => {
      const rows = jobs.filter(job => job.name === name || job.name.startsWith(`${name} (`));
      return rows.length === count && rows.every(job => job.conclusion === 'success');
    })) continue;
    const artifacts = client.api(`repos/${repository}/actions/runs/${run.id}/artifacts?per_page=100`).artifacts
      .filter(artifact => !artifact.expired).map(artifact => artifact.name);
    if (!buildArtifacts.every(([pattern, count]) => artifacts.filter(name => pattern.test(name)).length === count)) continue;
    return {build_route: 'proved', build_identity: current, build_revision: run.head_sha,
      build_run_id: String(run.id), build_run_attempt: String(run.run_attempt)};
  }
  return {build_route: 'establish', build_identity: current, build_revision: revision, build_run_id: '', build_run_attempt: ''};
}

// A qualification candidate: the newest release-line run whose images were
// all built, indexed and signed, with their identities still retained. Check
// changes are proved against these images before they merge
// (.github/workflows/release-qualify.yml); the merged commit's own release run
// still checks its own images.
export const imageJobs = 6;
export function findCandidate(root, revision, client) {
  oid(revision);
  const runs = client.api(`repos/${repository}/actions/workflows/release.yml/runs?branch=release/v0.4&per_page=50`).workflow_runs;
  for (const run of runs) {
    if (!['push', 'workflow_dispatch'].includes(run.event) || run.head_branch !== 'release/v0.4') continue;
    if (run.repository?.full_name !== repository || run.head_repository?.full_name !== repository) continue;
    if (run.path !== '.github/workflows/release.yml' && !String(run.path).startsWith('.github/workflows/release.yml@')) continue;
    if (!/^[0-9a-f]{40}$/.test(run.head_sha) || !isAncestor(root, run.head_sha, revision)) continue;
    const jobs = client.api(`repos/${repository}/actions/runs/${run.id}/jobs?filter=latest&per_page=100`).jobs;
    const images = jobs.filter(job => job.name.startsWith('images ('));
    const platforms = jobs.filter(job => job.name.startsWith('image-platforms ('));
    if (images.length !== imageJobs || platforms.length !== 2) continue;
    if (![...images, ...platforms].every(job => job.conclusion === 'success')) continue;
    const artifacts = client.api(`repos/${repository}/actions/runs/${run.id}/artifacts?name=sdk-image`).artifacts;
    if (!artifacts.some(artifact => artifact.name === 'sdk-image' && !artifact.expired)) continue;
    return {run_id: String(run.id), revision: run.head_sha};
  }
  throw new Error('no release-line run with retained, signed candidate images is an ancestor of this revision');
}

const canonical = value => JSON.stringify(value, (_, item) => item && typeof item === 'object' && !Array.isArray(item)
  ? Object.fromEntries(Object.keys(item).sort().map(key => [key, item[key]])) : item) + '\n';

// The route record published with every release: which κ the source V&V
// evidence is for and which build identity the build evidence is for, and for
// each whether this run established it or proved it by another run's receipt.
export function routeRecord(root, revision, route, source, sourceVv, build) {
  assert(['established', 'proved'].includes(route));
  oid(revision); oid(source.source_revision);
  for (const value of [source.run_id, source.run_attempt]) assert.match(value, /^[1-9][0-9]{0,19}$/);
  if (route === 'established') assert.equal(source.source_revision, revision);
  else assert(isAncestor(root, source.source_revision, revision), 'a receipt comes from this commit or an ancestor');
  const current = kappa(root, revision);
  assert.equal(kappa(root, source.source_revision), current, 'the receipt is for a different κ');
  assert(['established', 'proved'].includes(build?.route), 'build route');
  oid(build.revision);
  for (const value of [build.run_id, build.run_attempt]) assert.match(value, /^[1-9][0-9]{0,19}$/);
  if (build.route === 'established') assert.equal(build.revision, revision);
  else assert(isAncestor(root, build.revision, revision), 'a build receipt comes from this commit or an ancestor');
  const identity = buildIdentity(root, revision);
  assert.equal(buildIdentity(root, build.revision), identity, 'the build receipt is for a different build identity');
  return canonical({schema: 'prismpm/factory-route/2', revision, kappa: current, route,
    receipt: {source_revision: source.source_revision, run_id: source.run_id, run_attempt: source.run_attempt,
      source_vv_sha256: createHash('sha256').update(sourceVv).digest('hex')},
    build: {route: build.route, identity, revision: build.revision, run_id: build.run_id, run_attempt: build.run_attempt,
      jobs: Object.keys(buildJobs), plumbing: [...buildPlumbing]},
    plumbing: [...plumbing]});
}

function gh(path) {
  const result = spawnSync('gh', ['api', path], {encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, timeout: 120_000});
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `GitHub API request failed: ${result.stderr}`);
  return JSON.parse(result.stdout);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [operation, ...args] = process.argv.slice(2);
  const root = resolve('.');
  if (operation === 'kappa' && args.length === 1) process.stdout.write(`${kappa(root, args[0])}\n`);
  else if (operation === 'tests' && args.length === 0) process.stdout.write(plumbingTests(root).join('\n') + '\n');
  else if (operation === 'checks' && args.length === 0) {
    const result = spawnSync(process.execPath, ['--test', '--test-concurrency=1', ...plumbingTests(root)], {stdio: 'inherit'});
    process.exit(result.status ?? 1);
  } else if (operation === 'find-receipt' && args.length === 1) {
    // Receipts are found only for runs of the release line itself or its tag.
    assert(process.env.GITHUB_REF === releaseRef || process.env.GITHUB_REF === 'refs/tags/v0.4.0', 'release ref required');
    const found = {...findReceipt(root, args[0], {api: gh}, process.env.GITHUB_RUN_ID ?? ''),
      ...findBuildReceipt(root, args[0], {api: gh}, process.env.GITHUB_RUN_ID ?? '')};
    process.stdout.write(Object.entries(found).map(([key, value]) => `${key.replaceAll('_', '-')}=${value}\n`).join(''));
  } else if (operation === 'candidate' && args.length === 1) {
    // Qualification runs only for changes proposed to the release line.
    assert.equal(process.env.GITHUB_BASE_REF, 'release/v0.4', 'release-line pull request required');
    const found = findCandidate(root, args[0], {api: gh});
    process.stdout.write(`run-id=${found.run_id}\nrevision=${found.revision}\n`);
  } else if (operation === 'route' && args.length === 10) {
    const [output, revision, route, sourceRevision, runId, runAttempt, buildRoute, buildRevision, buildRunId, buildRunAttempt] = args;
    const sourceVv = readFileSync(join(root, 'target/source-vv-evidence/source-vv.json'));
    writeFileSync(output, routeRecord(root, revision, route,
      {source_revision: sourceRevision, run_id: runId, run_attempt: runAttempt}, sourceVv,
      {route: buildRoute, revision: buildRevision, run_id: buildRunId, run_attempt: buildRunAttempt}), {flag: 'wx', mode: 0o644});
  } else {
    console.error('usage: factory.mjs kappa REV | tests | checks | find-receipt REV | candidate REV | route OUT REV ROUTE SOURCE_REV RUN_ID RUN_ATTEMPT BUILD_ROUTE BUILD_REV BUILD_RUN_ID BUILD_RUN_ATTEMPT');
    process.exit(64);
  }
}
