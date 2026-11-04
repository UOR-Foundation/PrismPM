import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {createRequire} from 'node:module';
import {cpSync, linkSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import test from 'node:test';
import {admitDiagnosticPlan, diagnosticPolicy, diagnosticRef, diagnosticScope, diagnosticProductionPlan, productionRecord,
  diagnosticArchive, expectedDiagnosticSubject, productionArtifactName, selectDiagnosticProduction} from './native-resource-diagnostic.mjs';

const {load} = createRequire('/opt/prismpm/oracles/package.json')('js-yaml');
const workflow = () => load(readFileSync(new URL('../.github/workflows/release.yml', import.meta.url), 'utf8'));
const revision = 'a'.repeat(40), sha = bytes => 'sha256:' + createHash('sha256').update(bytes).digest('hex');
const environment = () => ({...process.env, GITHUB_REPOSITORY: 'UOR-Foundation/PrismPM', GITHUB_REF: diagnosticRef,
  GITHUB_EVENT_NAME: 'workflow_dispatch', GITHUB_SHA: revision, GITHUB_WORKFLOW_SHA: revision,
  GITHUB_RUN_ID: '123', GITHUB_RUN_ATTEMPT: '2', DIAGNOSTIC_SOURCE: revision,
  DISPATCH_VERSION: '0.4.0', REHEARSAL: 'true', PUBLISH_CRATES: 'false'});
test('resource observer pure refusal and custody units pass without privileged work', () => {
  const result = spawnSync('/usr/bin/python3', ['-I', '-B', new URL('./native-resource-monitor_test.py', import.meta.url).pathname],
    {encoding: 'utf8', timeout: 60000, maxBuffer: 1024 * 1024});
  assert.equal(result.status, 0, result.stdout + result.stderr);
});
function temporary(t) {const root = mkdtempSync(join(tmpdir(), 'native-resource-unit-')); t.after(() => rmSync(root, {recursive: true, force: true})); return root;}
function layout(root, arch, marker = 'same') {
  mkdirSync(join(root, 'blobs/sha256'), {recursive: true});
  const put = (value, mediaType) => {const bytes = Buffer.isBuffer(value) ? value : Buffer.from(JSON.stringify(value)); const digest = sha(bytes); writeFileSync(join(root, 'blobs/sha256', digest.slice(7)), bytes); return {digest, size: bytes.length, mediaType};};
  const config = put({architecture: arch, os: 'linux', config: {Labels: {
    'org.opencontainers.image.revision': revision, 'org.opencontainers.image.source': 'https://github.com/UOR-Foundation/PrismPM',
    'org.opencontainers.image.version': '0.4.0', 'org.opencontainers.image.created': '1970-01-01T00:00:00Z', marker}}}, 'application/vnd.oci.image.config.v1+json');
  const layer = put(Buffer.from('unit-only full layer bytes'), 'application/vnd.oci.image.layer.v1.tar');
  const descriptor = put({schemaVersion: 2, mediaType: 'application/vnd.oci.image.manifest.v1+json', config, layers: [layer]}, 'application/vnd.oci.image.manifest.v1+json');
  writeFileSync(join(root, 'index.json'), JSON.stringify({schemaVersion: 2, manifests: [descriptor]}));
  return {descriptor, layer, config};
}

test('diagnostic policy is exact-source exclusive, unpublished and one shared six-hour owner', () => {
  const p = diagnosticPolicy(environment(), 1000);
  assert.equal(p.scope, diagnosticScope); assert.equal(p.work_end_ms, 19801000); assert.equal(p.whole_end_ms, 21601000);
  assert.equal(p.maximum_native_runners, 2); assert.equal(p.store_bytes, 40 * 1024 ** 3);
  assert.equal(p.backing_reserve_bytes, 12 * 1024 ** 3); assert.equal(p.outside_allowance_bytes, 2 * 1024 ** 3);
  assert.equal(p.pids_limit, 4096); assert.equal(p.extra_swap_bytes, 0);
  for (const change of [{DIAGNOSTIC_SOURCE: ''}, {GITHUB_SHA: 'b'.repeat(40)}, {GITHUB_WORKFLOW_SHA: 'b'.repeat(40)},
    {GITHUB_REF: 'refs/heads/release/v0.4'}, {GITHUB_REF: 'refs/heads/main'}, {GITHUB_EVENT_NAME: 'push'},
    {REHEARSAL: 'false'}, {PUBLISH_CRATES: 'true'}, {REHEARSAL: 'null'}, {DISPATCH_VERSION: '0.5.0'},
    {GITHUB_RUN_ID: '0123'}, {GITHUB_REPOSITORY: 'foreign/PrismPM'}]) assert.throws(() => diagnosticPolicy({...environment(), ...change}, 1000));
});
test('every native owner requires the original source-bound policy and inherited deadlines', () => {
  const now = Date.now(), p = {...diagnosticPolicy(environment(), now),
    workflow_sha256: sha(readFileSync(new URL('../.github/workflows/release.yml', import.meta.url))),
    policy_helper_sha256: sha(readFileSync(new URL('./native-resource-diagnostic.mjs', import.meta.url)))};
  assert.deepEqual(admitDiagnosticPlan(p, revision, p.work_end_ms, p.whole_end_ms, environment()), p);
  for (const mutate of [p => p.source_revision = 'b'.repeat(40), p => p.work_end_ms++,
    p => p.workflow_sha256 = 'sha256:' + 'f'.repeat(64), p => p.run_attempt = '1',
    p => p.store_bytes++, p => p.production_accepted = true]) {
    const changed = structuredClone(p); mutate(changed);
    assert.throws(() => admitDiagnosticPlan(changed, revision, p.work_end_ms, p.whole_end_ms, environment()));
  }
});
test('P constructs only runtime and archive with a fresh builder and local outputs', () => {
  const p = diagnosticProductionPlan(revision, 'amd64', 'owned-diagnostic', '/tmp/diagnostic-exports');
  assert.deepEqual(p.builds.map(b => b.target), ['runtime', 'cli-archive']);
  assert.equal(p.builder, 'owned-diagnostic'); assert.equal(p.platform, 'linux/amd64');
  assert(p.builds[0].args.includes('--no-cache')); assert(!p.builds[1].args.includes('--no-cache'));
  for (const b of p.builds) assert(!/registry|--cache-from|--cache-to|--push|--load|--tag/.test(b.args.join(' ')));
  assert(p.builds[0].args.includes('type=oci,tar=false,dest=/tmp/diagnostic-exports/runtime.oci,rewrite-timestamp=true,oci-mediatypes=true'));
  assert.throws(() => diagnosticProductionPlan(revision, 'amd64', '--use', '/tmp/x'));
});
test('P metadata reads every layer and retains both genuine native child chains', t => {
  const root = temporary(t), records = ['amd64', 'arm64'].map(arch => {const path = join(root, arch); layout(path, arch); return productionRecord(path, arch, revision, environment());});
  const subject = expectedDiagnosticSubject(records, revision, '123', '2');
  assert.equal(subject.scope, diagnosticScope); assert.equal(JSON.parse(subject.index).manifests.length, 2);
  for (const mutate of [r => r[0].source_revision = 'b'.repeat(40), r => r[0].run_attempt = '1',
    r => r[1].architecture = 'amd64', r => r[1].production_accepted = true,
    r => r[0].complete_declared_layout_readback = false, r => r[1].manifest_base64 += '!',
    r => r[0].descriptor.digest = 'sha256:' + 'f'.repeat(64)]) {
    const changed = structuredClone(records); mutate(changed); assert.throws(() => expectedDiagnosticSubject(changed, revision, '123', '2'));
  }
  const manifest = JSON.parse(Buffer.from(records[0].manifest_base64, 'base64'));
  writeFileSync(join(root, 'amd64/blobs/sha256', manifest.layers[0].digest.slice(7)), Buffer.from('tampered unit layer bytes!'));
  assert.throws(() => productionRecord(join(root, 'amd64'), 'amd64', revision, environment()));
});
test('pair uses unchanged whole-byte verifier, separate raw roots and create-only output', t => {
  const root = temporary(t), records = ['amd64', 'arm64'].map(arch => {const path = join(root, 'p-' + arch); layout(path, arch); return productionRecord(path, arch, revision, environment());});
  const selected = join(root, 'subjects'); mkdirSync(selected);
  for (const record of records) writeFileSync(join(selected, 'production-' + record.architecture + '.json'), JSON.stringify(record));
  const a = join(root, 'a'), b = join(root, 'b'); cpSync(join(root, 'p-amd64'), a, {recursive: true}); cpSync(a, b, {recursive: true});
  const output = join(root, 'comparison.json'), helper = new URL('./native-resource-diagnostic.mjs', import.meta.url).pathname;
  const run = (first = a, second = b) => spawnSync(process.execPath, [helper, 'verify', selected, 'amd64', revision, first, second, output], {env: environment(), encoding: 'utf8'});
  assert.equal(run().status, 0); const original = readFileSync(output), record = JSON.parse(original);
  assert.equal(record.production_accepted, false); assert.equal(record.complete_original_pair_readback, true);
  assert.notEqual(run().status, 0); assert.deepEqual(readFileSync(output), original);
  rmSync(output); assert.notEqual(run(a, a).status, 0);
  layout(b, 'amd64', 'different'); assert.notEqual(run().status, 0); assert.throws(() => readFileSync(output));
});
test('archive diagnostics read complete bytes and refuse substituted or extra output', t => {
  const root = temporary(t), name = 'prismpm-0.4.0-x86_64-unknown-linux-gnu.tar.gz', bytes = Buffer.alloc(130003, 97);
  const path = join(root, name); writeFileSync(path, bytes); writeFileSync(path + '.sha256', sha(bytes).slice(7) + '  ' + name + '\n');
  const r = diagnosticArchive(root, revision, 'amd64', environment()); assert.equal(r.byte_length, bytes.length); assert.equal(r.production_accepted, false);
  writeFileSync(path, Buffer.alloc(bytes.length, 98)); assert.throws(() => diagnosticArchive(root, revision, 'amd64', environment()));
  writeFileSync(path, bytes); linkSync(path, join(root, 'extra')); assert.throws(() => diagnosticArchive(root, revision, 'amd64', environment()));
});
function provider() {return {jobs: ['amd64', 'arm64'].map((a, i) => ({id: i + 1, name: `Native resource production (${a})`, run_id: 123, run_attempt: 2, head_sha: revision, status: 'completed', conclusion: 'success'})), artifacts: ['amd64', 'arm64'].map((a, i) => ({id: 11 + i, name: productionArtifactName(a, revision, '123', '2'), workflow_run: {id: 123, head_sha: revision}, expired: false, size_in_bytes: 1234, digest: 'sha256:' + 'a'.repeat(64)}))};}
test('only successful current-attempt exact diagnostic owners supply the subject', () => {
  assert.deepEqual(selectDiagnosticProduction(provider(), revision, '123', '2').artifacts.map(a => a.id), [11, 12]);
  for (const mutate of [m => m.jobs.pop(), m => m.jobs[0].conclusion = 'failure', m => m.jobs[0].status = 'in_progress',
    m => m.jobs[0].run_attempt = 1, m => m.jobs.push({...m.jobs[0], run_attempt: 3}),
    m => m.artifacts.push({...m.artifacts[0], id: 99}), m => m.artifacts[1].expired = true,
    m => m.artifacts[0].workflow_run.id = 124, m => m.artifacts[0].workflow_run.head_sha = 'b'.repeat(40),
    m => m.artifacts[0].name = productionArtifactName('amd64', revision, '123', '3'),
    m => m.artifacts[0].size_in_bytes = 65 * 1024 ** 2, m => m.artifacts[0].digest = 'unknown']) {
    const changed = provider(); mutate(changed); assert.throws(() => selectDiagnosticProduction(changed, revision, '123', '2'));
  }
});
function isolation(w) {
  const jobs = w.jobs, diagnostics = ['diagnostic-policy', 'diagnostic-production', 'diagnostic-pair'];
  assert.equal(jobs.policy.if, "${{ inputs.resource-diagnostic-source == '' }}");
  assert.equal(jobs['diagnostic-policy'].if, "${{ inputs.resource-diagnostic-source != '' }}");
  function reachesPolicy(name, seen = new Set()) {assert(!seen.has(name)); if (name === 'policy') return true; const needs = jobs[name]?.needs; assert(needs); return (typeof needs === 'string' ? [needs] : needs).some(n => reachesPolicy(n, new Set([...seen, name])));}
  for (const name of Object.keys(jobs).filter(n => !diagnostics.includes(n))) {assert(!/always\(|failure\(|cancelled\(/.test(jobs[name].if ?? '')); assert(reachesPolicy(name));}
  for (const [name, needs] of [['diagnostic-production', 'diagnostic-policy'], ['diagnostic-pair', ['diagnostic-policy', 'diagnostic-production']]]) {
    const job = jobs[name]; assert.deepEqual(job.needs, needs); assert.equal(job.if, "${{ inputs.resource-diagnostic-source != '' }}");
    assert.equal(job['timeout-minutes'], 360); assert.equal(job.strategy['max-parallel'], 2); assert.equal(job.strategy['fail-fast'], false);
    assert.deepEqual(job.strategy.matrix.include, [{os: 'ubuntu-24.04', arch: 'amd64'}, {os: 'ubuntu-24.04-arm', arch: 'arm64'}]);
    assert.deepEqual(job.permissions, name === 'diagnostic-pair' ? {contents: 'read', actions: 'read'} : {contents: 'read'});
    const owner = job.steps.find(s => s.run?.includes('bash scripts/native-resource-diagnostic.sh')); assert(owner);
    assert.equal(owner.env.RESOURCE_WORK_END, '${{ needs.diagnostic-policy.outputs.work-end-ms }}'); assert.equal(owner.env.RESOURCE_WHOLE_END, '${{ needs.diagnostic-policy.outputs.whole-end-ms }}');
    for (const step of job.steps) {assert(!/login-action|attest|gh-release|cargo.*publish/.test(step.uses ?? '')); assert(!/--push|docker (?:image )?push|oras (?:manifest )?push|publish-oci|cosign sign|gh release|cargo publish|--cache-from|--cache-to/.test(step.run ?? ''));}
  }
}
test('diagnostic dispatch cannot enter ordinary release or publication owners', () => {
  isolation(workflow());
  for (const mutate of [w => delete w.jobs.policy.if, w => w.jobs.release.if = 'always()', w => w.jobs['image-platforms'].needs = [],
    w => w.jobs['diagnostic-production'].permissions.packages = 'write', w => w.jobs['diagnostic-pair'].needs = ['diagnostic-policy'],
    w => w.jobs['diagnostic-pair'].strategy['max-parallel'] = 3, w => w.jobs['diagnostic-production'].steps.push({run: 'docker push some-image'})]) {
    const w = workflow(); mutate(w); assert.throws(() => isolation(w));
  }
});
