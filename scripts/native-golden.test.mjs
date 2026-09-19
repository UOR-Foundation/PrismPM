// Source-development orchestration tests, not generated baseline acceptance.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync, symlinkSync, existsSync, chmodSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { execute } from './sdk-vv-check.mjs';
import test from 'node:test';
import { canonical, readEnvironmentLock, validateSourceBaseline, collectProfile, validateImageSpace, runReview, validateWorkflow } from './native-golden.mjs';

const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const revision = 'a'.repeat(40), environmentRevision = 'b'.repeat(40);
const recordPaths = ['golden-manifest.json', 'verified/lexlean-attestation.json', 'verified/manifest.json'];
const profile = 'tests/golden/native/linux-arm64-ubuntu-24.04';
function workspace(t) {
  const root = mkdtempSync(join(tmpdir(), 'prism-native-golden-unit-'));
  t.after(() => rmSync(root, {recursive: true, force: true}));
  const source = join(root, 'source'); mkdirSync(source);
  const put = (path, bytes) => { mkdirSync(join(source, path, '..'), {recursive: true}); writeFileSync(join(source, path), bytes); };
  put('stdlib/src/Probe.lex.tex', 'unit-only source\n'); put('tests/golden/stdlib/source/Probe.lex.tex', 'unit-only source\n');
  put('tests/golden/stdlib/golden-manifest.json', '{}');
  return {root, source, put};
}

test('environment lock is a closed immutable ARM64 development-only authority', () => {
  const raw = readFileSync(new URL('../sdk/golden-development.lock.json', import.meta.url));
  const lock = readEnvironmentLock(raw);
  assert.equal(lock.scope, 'source-golden-review-only'); assert.equal(lock.architecture, 'arm64');
  for (const changed of [{...lock, reference: 'example.invalid/sdk:latest'}, {...lock, architecture: 'amd64'},
    {...lock, source_revision: 'current'}, {...lock, extra: true}, {...lock, scope: 'sdk-acceptance'}]) {
    assert.throws(() => readEnvironmentLock(Buffer.from(canonical(changed) + '\n')));
  }
  assert.throws(() => readEnvironmentLock(Buffer.from(JSON.stringify(lock))));
  const measured = 4107752353, reserve = 12 * 1024 ** 3;
  validateImageSpace(reserve + measured, measured);
  assert.throws(() => validateImageSpace(reserve + measured - 1, measured));
  for (const invalid of [-1, NaN, Infinity, 1.5]) assert.throws(() => validateImageSpace(reserve, invalid));
});

test('current shared source baseline is byte-exact before expensive generation', t => {
  const {source, put} = workspace(t); validateSourceBaseline(source);
  put('stdlib/src/Probe.lex.tex', 'changed'); assert.throws(() => validateSourceBaseline(source));
  put('stdlib/src/Probe.lex.tex', 'unit-only source\n'); put('stdlib/src/New.lex.tex', 'new');
  assert.throws(() => validateSourceBaseline(source));
  rmSync(join(source, 'stdlib/src/New.lex.tex')); put('tests/golden/stdlib/source/Extra.lex.tex', 'extra');
  assert.throws(() => validateSourceBaseline(source));
  rmSync(join(source, 'tests/golden/stdlib/source/Extra.lex.tex'));
  rmSync(join(source, 'stdlib/src/Probe.lex.tex')); symlinkSync('../other', join(source, 'stdlib/src/Probe.lex.tex'));
  assert.throws(() => validateSourceBaseline(source));
});

test('native profile collection retains exactly three original raw files', t => {
  const {source, put} = workspace(t);
  for (const path of recordPaths) put(`${profile}/${path}`, Buffer.from(`raw ${path}\n`));
  const actual = collectProfile(source);
  assert.deepEqual(actual.map(row => row.path), recordPaths);
  for (const row of actual) assert.deepEqual(row.bytes, readFileSync(join(source, profile, row.path)));
  for (const directory of ['extra', 'verified/empty']) {
    mkdirSync(join(source, profile, directory)); assert.throws(() => collectProfile(source));
    rmSync(join(source, profile, directory), {recursive: true});
  }
  put(`${profile}/extra.json`, '{}'); assert.throws(() => collectProfile(source));
  rmSync(join(source, profile, 'extra.json')); rmSync(join(source, profile, recordPaths[0]));
  assert.throws(() => collectProfile(source));
});

function fixture(t, fault) {
  const work = workspace(t), calls = [];
  if (fault === 'stale-base') work.put('stdlib/src/Probe.lex.tex', 'changed current model\n');
  if (fault === 'stale-artifact') work.put('.prism/build/prior/result.json', '{}');
  const config = 'sha256:' + 'c'.repeat(64);
  const manifest = Buffer.from(JSON.stringify({schemaVersion: 2, mediaType: 'application/vnd.oci.image.manifest.v1+json', config: {digest: config, size: 100}, layers: []}));
  const child = {mediaType: 'application/vnd.oci.image.manifest.v1+json', digest: 'sha256:' + hash(manifest), size: manifest.length, platform: {os: 'linux', architecture: 'arm64'}};
  const index = Buffer.from(JSON.stringify({schemaVersion: 2, mediaType: 'application/vnd.oci.image.index.v1+json', manifests: [child]}));
  const image = `fixture.invalid/sdk@sha256:${hash(index)}`;
  const lock = {schema: 'prismpm/golden-development-environment/1', scope: 'source-golden-review-only', architecture: 'arm64',
    reference: image, child_digest: child.digest, config_digest: config, source_revision: environmentRevision};
  work.put('sdk/golden-development.lock.json', canonical(lock) + '\n');
  const ok = value => ({status: 0, signal: null, stdout: Buffer.isBuffer(value) ? value : Buffer.from(typeof value === 'string' ? value : JSON.stringify(value)), stderr: Buffer.alloc(0)});
  const bad = () => ({...ok(''), status: 1, stderr: Buffer.from('unit-only genuine command failure')});
  let created, label, generations = 0;
  const transport = async (command, args) => {
    calls.push([command, args]);
    if (command === 'git') {
      if (args.includes('rev-parse')) return ok((fault === 'source-revision' ? environmentRevision : revision) + '\n');
      if (args.includes('status')) return ok(fault === 'dirty-source' ? ' M source.lex.tex\n' : '');
      if (args.includes('diff')) return ok(fault === 'source-changed' ? 'stdlib/src/Probe.lex.tex\n' : '');
      if (args.includes('ls-files')) return ok('');
      assert.fail(args.join(' '));
    }
    assert.equal(command, 'docker');
    if (args[0] === 'buildx') return ok(args.at(-1) === image ? index : manifest);
    if (args[0] === 'pull') return ok('pulled');
    if (args[0] === 'image') {
      const selected = args.includes('--platform');
      const descriptor = selected ? child : {digest: 'sha256:' + hash(index), size: index.length, mediaType: 'application/vnd.oci.image.index.v1+json'};
      return ok([{Id: descriptor.digest, Descriptor: descriptor, RepoDigests: [image], Os: 'linux', Architecture: fault === 'wrong-platform' ? 'amd64' : 'arm64',
        Config: {Labels: {'org.opencontainers.image.revision': environmentRevision}, Volumes: null}}]);
    }
    if (args[0] === 'create') {
      created = args[args.indexOf('--name') + 1]; label = args[args.indexOf('--label') + 1];
      assert.equal(args[args.indexOf('--network') + 1], 'none'); assert(args.includes('--pull=never'));
      assert(!args.includes('--privileged')); assert(!args.some(arg => arg.includes('docker.sock')));
      return ok('created');
    }
    if (args[0] === 'start') return ok('started');
    if (args[0] === 'inspect') return ok([{Image: fault === 'wrong-container-image' ? config : 'sha256:' + hash(index), Config: {Labels: Object.fromEntries([label.split('=')])}}]);
    if (args[0] === 'rm') { if (fault === 'cleanup') return bad(); created = undefined; return ok('removed'); }
    assert.deepEqual(args.slice(0, 2), ['exec', created]);
    if (args[2] === 'node') return ok({architecture: 'arm64', os: 'linux', release: 'ID=ubuntu\nVERSION_ID="24.04"\n'});
    assert.deepEqual(args.slice(2), ['cargo', 'run', '--locked', '--offline', '-p', 'xtask', '--', 'check-golden', ...(generations === 0 ? ['--write'] : [])]);
    generations++;
    work.put('.prism/build/unit/build-artifact.json', '{"diagnostic":"unit-only"}');
    if (fault === 'write-failure' && generations === 1 || fault === 'repeat-failure' && generations === 2) return bad();
    if (fault !== 'missing-profile') for (const path of recordPaths) work.put(`${profile}/${path}`, `raw ${path}\n`);
    if (fault === 'repeat-mutation' && generations === 2) work.put(`${profile}/golden-manifest.json`, 'changed');
    return ok('unit-only generation transcript');
  };
  const context = {architecture: 'arm64', uid: 1000, gid: 1000, environment: {PATH: process.env.PATH, GITHUB_ACTIONS: 'true', RUNNER_ENVIRONMENT: 'github-hosted',
    RUNNER_OS: 'Linux', RUNNER_ARCH: 'ARM64', GITHUB_EVENT_NAME: 'pull_request', GITHUB_RUN_ID: '123', GITHUB_RUN_ATTEMPT: '1'}};
  const destination = join(work.root, 'output');
  return {...work, calls, destination, run: (implementation = runReview) => implementation({source: work.source, revision, destination}, transport, context),
    generations: () => generations, remaining: () => created};
}

test('source review executes exact image, both unchanged golden commands and byte-preserving export', async t => {
  const f = fixture(t); const result = await f.run();
  assert.equal(result.scope, 'source-golden-review-only'); assert.equal(result.status, 'review-required');
  assert.equal(f.generations(), 2); assert.equal(f.remaining(), undefined);
  assert.equal(result.source_revision, revision);
  for (const row of result.records) assert.equal(hash(readFileSync(join(f.destination, 'records', row.path))), row.sha256);
  assert(existsSync(join(f.destination, 'generated/build/unit/build-artifact.json')));
});

test('failure and missing evidence never become a reviewed or accepted SDK baseline', async t => {
  for (const fault of ['source-revision', 'dirty-source', 'stale-base', 'stale-artifact', 'wrong-platform', 'wrong-container-image', 'write-failure', 'repeat-failure',
    'missing-profile', 'repeat-mutation', 'source-changed', 'cleanup']) {
    const f = fixture(t, fault); await assert.rejects(f.run(), undefined, fault);
    assert(!existsSync(join(f.destination, 'review.json')), fault);
    if (fault !== 'cleanup') assert.equal(f.remaining(), undefined, fault);
    if (fault === 'write-failure') {
      assert.equal(f.generations(), 1); assert(existsSync(join(f.destination, 'generated/build/unit/build-artifact.json')));
    }
  }
});

test('PR workflow has no publication policy bypass and keeps exact native generation and failure uploads', () => {
  const raw = readFileSync(new URL('../.github/workflows/native-golden.yml', import.meta.url), 'utf8');
  validateWorkflow(raw);
  for (const changed of [raw.replace('ubuntu-24.04-arm', 'ubuntu-24.04'), raw.replace('pull_request:', 'workflow_dispatch:'),
    raw.replace('contents: read', 'contents: write'), raw.replace('22.23.2', 'latest'), raw.replace('if: always()', 'if: success()'),
    raw.replace('node scripts/native-golden.mjs run', 'true # node scripts/native-golden.mjs run')]) assert.throws(() => validateWorkflow(changed));
  const runtime = JSON.parse(readFileSync(new URL('../sdk/vv-runtime.lock.json', import.meta.url)));
  assert(raw.includes(`driver-opts: image=${runtime.images.buildkit.reference}`));
});

test('actual workflow shell executes both commands and propagates either failure', async t => {
  const raw = readFileSync(new URL('../.github/workflows/native-golden.yml', import.meta.url), 'utf8');
  validateWorkflow(raw);
  const shell = raw.match(/        run: \|\n((?:          .*\n)+)/)[1].split('\n').filter(Boolean).map(line => line.slice(10)).join('\n');
  const {root} = workspace(t), binary = join(root, 'node'), log = join(root, 'calls');
  writeFileSync(binary, `#!/bin/sh\nprintf '%s\\n' "$*" >> "$GOLDEN_CALL_LOG"\nif [ "$2" = "$GOLDEN_FAIL_OPERATION" ]; then exit 37; fi\n`); chmodSync(binary, 0o755);
  for (const fault of ['', 'tests', 'run']) {
    rmSync(log, {force: true});
    const result = await execute('/bin/bash', ['--noprofile', '--norc', '-euo', 'pipefail', '-c', shell],
      {environment: {PATH: `${root}:/usr/bin:/bin`, SOURCE_REVISION: revision, RUNNER_TEMP: root, GOLDEN_CALL_LOG: log, GOLDEN_FAIL_OPERATION: fault}});
    assert.equal(result.status, fault ? 37 : 0);
    const calls = readFileSync(log, 'utf8').trim().split('\n');
    assert.deepEqual(calls, ['scripts/native-golden.mjs tests', ...(fault === 'tests' ? [] : [`scripts/native-golden.mjs run ${revision} ${root}/native-golden`])]);
  }
});

test('executed source-baseline and second-run omission mutants fail owning behavioral checks', async t => {
  const source = readFileSync(new URL('./native-golden.mjs', import.meta.url), 'utf8');
  const load = async text => {
    for (const dependency of ['./sdk-vv-check.mjs', './browser-api-sdk-check.mjs']) text = text.replace(JSON.stringify(dependency), JSON.stringify(new URL(dependency, import.meta.url).href))
      .replace(`'${dependency}'`, JSON.stringify(new URL(dependency, import.meta.url).href));
    return import('data:text/javascript;base64,' + Buffer.from(text).toString('base64'));
  };
  const noRepeat = await load(source.replace('for (const write of [true, false])', 'for (const write of [true])'));
  await assert.rejects(async () => { const f = fixture(t); await f.run(noRepeat.runReview); assert.equal(f.generations(), 2); }, /1 !== 2/);
  const noBaseline = await load(source.replaceAll('validateSourceBaseline(source);', ''));
  await assert.rejects(async () => { const f = fixture(t, 'stale-base'); await assert.rejects(f.run(noBaseline.runReview), /golden source bytes are stale/); }, /Missing expected rejection/);
});
