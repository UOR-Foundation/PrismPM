import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {createRequire} from 'node:module';
import {cpSync, linkSync, mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, truncateSync, writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import test from 'node:test';
import {admitDiagnosticPlan, diagnosticPolicy, diagnosticRef, diagnosticPushPrefix, diagnosticScope, diagnosticProductionPlan, productionRecord,
  diagnosticArchive, expectedDiagnosticSubject, productionArtifactName, selectDiagnosticProduction} from './native-resource-diagnostic.mjs';
import {diagnosticInitialAncestor} from './native-resource-ancestry.mjs';

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
function pushed(t) {
  const root = temporary(t), path = join(root, 'event.json');
  const event = {ref: diagnosticPushPrefix + revision, before: '0'.repeat(40), after: revision,
    head_commit: {id: revision}, created: true, deleted: false, forced: false,
    repository: {full_name: 'UOR-Foundation/PrismPM', fork: false}};
  const env = {...environment(), GITHUB_EVENT_NAME: 'push', GITHUB_REF: event.ref, GITHUB_EVENT_PATH: path,
    DIAGNOSTIC_SOURCE: '', DISPATCH_VERSION: '', REHEARSAL: 'null', PUBLISH_CRATES: 'null'};
  const put = () => writeFileSync(path, JSON.stringify(event)); put(); return {root, path, event, env, put};
}
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
test('create-only diagnostic push binds full ref, event head, source and workflow under unchanged ceilings', t => {
  const f = pushed(t), p = diagnosticPolicy(f.env, 1000), dispatched = diagnosticPolicy(environment(), 1000);
  assert.deepEqual(p.trigger, {event: 'push', ref: f.event.ref, source_revision: revision,
    before: '0'.repeat(40), after: revision, head_commit: revision, created: true, deleted: false,
    forced: false, ancestor_proof: null});
  delete p.trigger; delete dispatched.trigger; assert.deepEqual(p, dispatched);
  // Push mode has no dispatch inputs: it fixes unpublished behavior only after
  // validating the original push, rather than treating absent input as release.
  assert.equal(diagnosticPolicy({...f.env, PUBLISH_CRATES: 'true', REHEARSAL: 'false'}, 1000).scope, diagnosticScope);
});
test('pinned fast-forward verifies the actual local commit ancestry independently of its event', t => {
  const f = pushed(t), result = spawnSync('/usr/bin/git', ['rev-parse', 'HEAD'], {encoding: 'utf8', timeout: 10000});
  assert.equal(result.status, 0); const head = result.stdout.trim();
  f.env.GITHUB_SHA = head; f.env.GITHUB_WORKFLOW_SHA = head; f.env.GITHUB_REF = diagnosticPushPrefix + head;
  Object.assign(f.event, {ref: f.env.GITHUB_REF, before: diagnosticInitialAncestor, after: head,
    head_commit: {id: head}, created: false}); f.put();
  const p = diagnosticPolicy(f.env, 1000);
  assert.deepEqual(p.trigger.ancestor_proof, {before: diagnosticInitialAncestor, after: head, strict_ancestor: true});
  assert.equal(p.trigger.created, false); assert.equal(p.maximum_native_runners, 2);
  for (const change of [{before: 'b'.repeat(40)}, {before: '0'.repeat(40)}, {created: true},
    {forced: true}, {deleted: true}, {repository: {full_name: 'UOR-Foundation/PrismPM', fork: true}}]) {
    const prior = structuredClone(f.event); Object.assign(f.event, change); f.put();
    assert.throws(() => diagnosticPolicy(f.env)); Object.assign(f.event, prior); f.put();
  }
});
test('diagnostic push refuses every foreign, partial, stale or non-create identity', t => {
  for (const change of [{GITHUB_SHA: 'b'.repeat(40)}, {GITHUB_WORKFLOW_SHA: 'b'.repeat(40)},
    {GITHUB_REPOSITORY: 'foreign/PrismPM'}, {GITHUB_REF: diagnosticRef},
    {GITHUB_REF: 'refs/heads/main'}, {GITHUB_REF: 'refs/heads/release/v0.4'},
    {GITHUB_REF: 'refs/heads/RESOURCE-DIAGNOSTIC/' + revision},
    {GITHUB_REF: 'refs/tags/resource-diagnostic/' + revision}, {GITHUB_REF: 'refs/tags/v0.4.0'},
    ...[revision.slice(0, 39), revision + 'a', revision.toUpperCase(), revision + '/extra', 'latest']
      .map(suffix => ({GITHUB_REF: diagnosticPushPrefix + suffix})),
    ...['pull_request', 'workflow_run', 'create', 'repository_dispatch'].map(name => ({GITHUB_EVENT_NAME: name}))]) {
    const f = pushed(t); assert.throws(() => diagnosticPolicy({...f.env, ...change}));
  }
  for (const mutate of [e => e.ref = diagnosticPushPrefix + 'b'.repeat(40), e => e.after = 'b'.repeat(40),
    e => e.head_commit.id = 'b'.repeat(40), e => delete e.head_commit, e => e.head_commit = null,
    e => e.head_commit = {}, e => e.head_commit = 'invented', e => e.repository.full_name = 'foreign/PrismPM',
    e => e.repository.fork = true, e => delete e.repository.fork,
    e => e.created = false, e => e.deleted = true, e => e.forced = true, e => e.before = '1'.repeat(40),
    e => e.created = 'true', e => delete e.deleted]) {
    const f = pushed(t); mutate(f.event); f.put(); assert.throws(() => diagnosticPolicy(f.env));
  }
});
test('push event must remain a bounded original file and is rechecked by native admission', t => {
  for (const change of [f => rmSync(f.path), f => writeFileSync(f.path, '{malformed'),
    f => truncateSync(f.path, 32 * 1024 ** 2 + 1),
    f => {const target = join(f.root, 'other'); writeFileSync(target, readFileSync(f.path)); rmSync(f.path); symlinkSync(target, f.path);},
    f => linkSync(f.path, join(f.root, 'hardlink'))]) {
    const f = pushed(t); change(f); assert.throws(() => diagnosticPolicy(f.env));
  }
  const f = pushed(t), now = Date.now(), p = {...diagnosticPolicy(f.env, now),
    workflow_sha256: sha(readFileSync(new URL('../.github/workflows/release.yml', import.meta.url))),
    policy_helper_sha256: sha(readFileSync(new URL('./native-resource-diagnostic.mjs', import.meta.url)))};
  assert.deepEqual(admitDiagnosticPlan(p, revision, p.work_end_ms, p.whole_end_ms, f.env), p);
  f.event.head_commit.id = 'b'.repeat(40); f.put();
  assert.throws(() => admitDiagnosticPlan(p, revision, p.work_end_ms, p.whole_end_ms, f.env));
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
  const diagnosticGuard = "${{ (github.event_name == 'workflow_dispatch' && inputs.resource-diagnostic-source != '') || (github.event_name == 'push' && startsWith(github.ref, 'refs/heads/resource-diagnostic/')) }}";
  assert.equal(jobs.policy.if, "${{ inputs.resource-diagnostic-source == '' && !startsWith(github.ref, 'refs/heads/resource-diagnostic/') }}");
  assert.equal(jobs['diagnostic-policy'].if, diagnosticGuard);
  function reachesPolicy(name, seen = new Set()) {assert(!seen.has(name)); if (name === 'policy') return true; const needs = jobs[name]?.needs; assert(needs); return (typeof needs === 'string' ? [needs] : needs).some(n => reachesPolicy(n, new Set([...seen, name])));}
  for (const name of Object.keys(jobs).filter(n => !diagnostics.includes(n))) {assert(!/always\(|failure\(|cancelled\(/.test(jobs[name].if ?? '')); assert(reachesPolicy(name));}
  for (const [name, needs] of [['diagnostic-production', 'diagnostic-policy'], ['diagnostic-pair', ['diagnostic-policy', 'diagnostic-production']]]) {
    const job = jobs[name]; assert.deepEqual(job.needs, needs); assert.equal(job.if, diagnosticGuard);
    assert.equal(job['timeout-minutes'], 360); assert.equal(job.strategy['max-parallel'], 2); assert.equal(job.strategy['fail-fast'], false);
    assert.deepEqual(job.strategy.matrix.include, [{os: 'ubuntu-24.04', arch: 'amd64'}, {os: 'ubuntu-24.04-arm', arch: 'arm64'}]);
    assert.deepEqual(job.permissions, name === 'diagnostic-pair' ? {contents: 'read', actions: 'read'} : {contents: 'read'});
    const owner = job.steps.find(s => s.run?.includes('bash scripts/native-resource-diagnostic.sh')); assert(owner);
    assert.equal(owner.env.RESOURCE_WORK_END, '${{ needs.diagnostic-policy.outputs.work-end-ms }}'); assert.equal(owner.env.RESOURCE_WHOLE_END, '${{ needs.diagnostic-policy.outputs.whole-end-ms }}');
    for (const step of job.steps) {assert(!/login-action|attest|gh-release|cargo.*publish/.test(step.uses ?? '')); assert(!/--push|docker (?:image )?push|oras (?:manifest )?push|publish-oci|cosign sign|gh release|cargo publish|--cache-from|--cache-to/.test(step.run ?? ''));}
  }
}
function diagnosticNodePrerequisite(w) {
  isolation(w);
  const policy = w.jobs['diagnostic-policy'], steps = policy.steps;
  assert.equal(policy['runs-on'], 'ubuntu-24.04');
  assert.equal(policy['timeout-minutes'], 10);
  assert.deepEqual(policy.permissions, {contents: 'read'});
  const matching = predicate => {
    const found = steps.map((step, index) => ({step, index})).filter(({step}) => predicate(step));
    assert.equal(found.length, 1, 'one exact diagnostic prerequisite/contract owner required');
    return found[0];
  };
  const node = matching(step => step.uses?.startsWith('actions/setup-node@'));
  assert.equal(node.step.uses, 'actions/setup-node@249970729cb0ef3589644e2896645e5dc5ba9c38');
  assert.equal(node.step.with['node-version'], '22.23.2');
  const plan = matching(step => step.id === 'plan');
  assert.match(plan.step.run, /^\s*node scripts\/native-resource-diagnostic\.mjs policy >> "\$GITHUB_OUTPUT"\s*$/m);
  const bootstrap = matching(step => step.name === 'Prepare exact source-owned Node oracles before diagnostic contracts');
  const bootstrapLines = [
    'set -euo pipefail',
    'bash scripts/source-oracles-bootstrap.sh "$GITHUB_SHA" "$RUNNER_TEMP/diagnostic-node-oracles" "$RUNNER_TEMP/diagnostic-node-evidence"',
    'node_path=$(cat "$RUNNER_TEMP/diagnostic-node-evidence/node-oracles-path.txt")',
    'test "$node_path" = "$RUNNER_TEMP/diagnostic-node-oracles/node_modules"',
    String.raw`printf 'NODE_PATH=%s\n' "$node_path" >> "$GITHUB_ENV"`,
  ];
  assert.deepEqual(bootstrap.step.run.replace(/\\\r?\n[ \t]*/g, '').trim().split('\n').map(line => line.trim()), bootstrapLines,
    'diagnostic Node setup must conserve the source and export only its exact private modules');
  const contracts = matching(step => step.run?.includes('node --test'));
  const command = contracts.step.run.trim().split(/\s+/);
  assert.deepEqual(command.slice(0, 2), ['node', '--test']);
  assert.deepEqual(command.slice(2).sort(), ['scripts/native-resource-ancestry.test.mjs',
    'scripts/native-resource-command.test.mjs', 'scripts/native-resource-diagnostic.test.mjs',
    'scripts/native-resource-driver.test.mjs'], 'all original diagnostic contracts and ancestry must run without skips');
  assert(node.index < plan.index && plan.index < bootstrap.index && bootstrap.index < contracts.index,
    'pinned Node and source admission must precede dependency acquisition and every contract');
  const upload = matching(step => step.uses?.startsWith('actions/upload-artifact@'));
  assert.equal(upload.step.uses, 'actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02');
  assert.equal(upload.step.if, 'always()');
  assert.equal(upload.step.with.path, '${{ runner.temp }}/diagnostic-node-evidence/');
  assert.equal(upload.step.with['if-no-files-found'], 'error');
  assert(bootstrap.index < upload.index, 'retain the original prerequisite diagnostics');
  for (const {step} of [node, plan, bootstrap, contracts]) {
    assert.equal(step.if, undefined, 'prerequisite and contracts must use normal success gating');
    assert.equal(step['continue-on-error'], undefined, 'prerequisite and contract failures must stop policy');
  }
  for (const step of steps) {
    assert(!/rustup|\bcargo\b|semantic-source-bootstrap/.test(step.run ?? ''), 'diagnostic policy needs Node only');
    assert.equal(step.env?.NODE_PATH, undefined, 'do not override the validated private Node dependency path');
    assert.equal(step.env?.NODE_OPTIONS, undefined, 'do not inject diagnostic Node behavior');
  }
}
test('diagnostic policy prepares locked source-owned Node dependencies before every original contract', () => {
  diagnosticNodePrerequisite(workflow());
});
test('diagnostic Node prerequisite contract refuses omission, widening, bypass and reordered acquisition', () => {
  const named = w => w.jobs['diagnostic-policy'].steps.find(step => step.name === 'Prepare exact source-owned Node oracles before diagnostic contracts');
  const contracts = w => w.jobs['diagnostic-policy'].steps.find(step => step.run?.includes('node --test'));
  for (const mutate of [
    w => w.jobs['diagnostic-policy'].steps.find(step => step.uses?.startsWith('actions/setup-node@')).with['node-version'] = '22',
    w => w.jobs['diagnostic-policy'].steps.find(step => step.uses?.startsWith('actions/setup-node@')).uses = 'actions/setup-node@main',
    w => {const steps = w.jobs['diagnostic-policy'].steps, index = steps.findIndex(step => step.id === 'plan');
      const bootstrap = steps.splice(steps.findIndex(step => step === named(w)), 1)[0]; steps.splice(index, 0, bootstrap);},
    w => {const steps = w.jobs['diagnostic-policy'].steps, index = steps.findIndex(step => step === contracts(w));
      const node = steps.splice(steps.findIndex(step => step.uses?.startsWith('actions/setup-node@')), 1)[0]; steps.splice(index, 0, node);},
    w => named(w).run = named(w).run.replace('"$GITHUB_SHA"', '"$OTHER_SHA"'),
    w => named(w).run = named(w).run.replaceAll('diagnostic-node-oracles', 'node_modules'),
    w => named(w).run = named(w).run.replace('test "$node_path" = "$RUNNER_TEMP/diagnostic-node-oracles/node_modules"', ':'),
    w => named(w).run = named(w).run.replace('node-oracles-path.txt', 'package-lock.json'),
    w => named(w).run = named(w).run.replace('"$node_path" >> "$GITHUB_ENV"', '"/opt/prismpm/oracles/node_modules" >> "$GITHUB_ENV"'),
    w => named(w).run = named(w).run.replace('set -euo pipefail', 'set +e'),
    w => named(w)['continue-on-error'] = true,
    w => named(w).if = 'always()',
    w => named(w).run += '\nrustup run 1.97.1 cargo build\n',
    w => {w.jobs['diagnostic-policy'].steps = w.jobs['diagnostic-policy'].steps.filter(step => step !== named(w));},
    w => contracts(w).run = contracts(w).run.replace('scripts/native-resource-driver.test.mjs', ''),
    w => contracts(w).run = contracts(w).run.replace('scripts/native-resource-ancestry.test.mjs', ''),
    w => contracts(w).run = contracts(w).run.replace('node --test', 'node --test --test-skip-pattern=resource'),
    w => contracts(w).run += ' || true',
    w => contracts(w)['continue-on-error'] = true,
    w => contracts(w).env = {NODE_PATH: '/opt/prismpm/oracles/node_modules'},
    w => w.jobs['diagnostic-policy'].steps.find(step => step.uses?.startsWith('actions/upload-artifact@')).if = 'success()',
    w => w.jobs['diagnostic-policy'].steps.find(step => step.uses?.startsWith('actions/upload-artifact@')).with['if-no-files-found'] = 'warn',
    w => w.jobs['diagnostic-policy'].steps.find(step => step.uses?.startsWith('actions/upload-artifact@')).with.path = '${{ runner.temp }}/other-evidence/',
  ]) {
    const w = workflow(); mutate(w); assert.throws(() => diagnosticNodePrerequisite(w));
  }
});
test('diagnostic dispatch cannot enter ordinary release or publication owners', () => {
  isolation(workflow());
  for (const mutate of [w => delete w.jobs.policy.if, w => w.jobs.release.if = 'always()', w => w.jobs['image-platforms'].needs = [],
    w => w.jobs['diagnostic-production'].permissions.packages = 'write', w => w.jobs['diagnostic-pair'].needs = ['diagnostic-policy'],
    w => w.jobs['diagnostic-pair'].strategy['max-parallel'] = 3, w => w.jobs['diagnostic-production'].steps.push({run: 'docker push some-image'})]) {
    const w = workflow(); mutate(w); assert.throws(() => isolation(w));
  }
});
