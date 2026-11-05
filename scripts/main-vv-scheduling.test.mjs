// Scheduling and original-evidence transport policy; no SDK or hosted execution.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {readFileSync} from 'node:fs';
import {createRequire} from 'node:module';
import test from 'node:test';

const {load} = createRequire('/opt/prismpm/oracles/package.json')('js-yaml');
const workflow = () => load(readFileSync(new URL('../.github/workflows/vv.yml', import.meta.url), 'utf8'));
const parts = ['source', 'gates', 'conformance', 'ho-oc', 'st-sy', 'dk-0-1', 'dk-2', 'dk-3', 'dk-4',
  'standalone-projection', 'standalone-reservation', 'standalone-payload', 'standalone-recovery-frames',
  'standalone-journal-recovery', 'standalone-retention'];
const checkout = 'actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683';
const upload = 'actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02';
const download = 'actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093';
const devcontainer = 'devcontainers/ci@8bf61b26e9c3a98f69cb6ce2f88d24ff59b785c6';
const named = (steps, name) => {
  const found = steps.filter(step => step.name === name); assert.equal(found.length, 1, name); return found[0];
};
const identified = (steps, id) => {
  const found = steps.filter(step => step.id === id); assert.equal(found.length, 1, id); return found[0];
};
const partCommand = `set -euo pipefail
bash scripts/vv.sh --with-sdk bash -euo pipefail -c "$(cat <<'PRISMPM_VV'
node scripts/ci-observe.mjs run target/ci-diagnostics/vv -- node scripts/release-gate-evidence.mjs source-part-run . target/source-vv-evidence "$GITHUB_SHA" amd64 - "$GITHUB_RUN_ID" "$GITHUB_RUN_ATTEMPT" "$VV_PART"
PRISMPM_VV
)"
`;
const namingCommand = `set -euo pipefail
name=$(node scripts/release-gate-evidence.mjs artifact-name "source-vv-part-$VV_PART" "$GITHUB_SHA" amd64 - "$GITHUB_RUN_ID" "$GITHUB_RUN_ATTEMPT")
printf 'name=%s\\n' "$name" >> "$GITHUB_OUTPUT"
`;
const selectionCommand = `set -euo pipefail
mkdir -p target/source-vv-selection
node scripts/release-artifact-metadata.mjs target/source-vv-selection/provider.json "$GITHUB_RUN_ID"
owners=$(node scripts/vv-assemble.mjs --list | sed 's/[^[:space:]]\\+/source-vv-part-&/g')
node scripts/release-gate-evidence.mjs select-artifacts target/source-vv-selection/provider.json target/source-vv-selection/selection.json \\
  "$GITHUB_SHA" amd64 - "$GITHUB_RUN_ID" "$GITHUB_RUN_ATTEMPT" $owners
ids=$(node -e 'const fs=require("node:fs"); console.log(JSON.parse(fs.readFileSync(process.argv[1])).artifacts.map(a=>a.id).join(","))' target/source-vv-selection/selection.json)
printf 'ids=%s\\n' "$ids" >> "$GITHUB_OUTPUT"
`;
const assembleCommand = `set -euo pipefail
node scripts/release-gate-evidence.mjs source-part-assemble . target/source-vv-evidence \\
  "$GITHUB_SHA" amd64 - "$GITHUB_RUN_ID" "$GITHUB_RUN_ATTEMPT" \\
  target/source-vv-selection/provider.json target/source-vv-selection/selection.json
`;
const guardCommand = 'set -euo pipefail\ntest "$SOURCE_PARTS_RESULT" = success\n';

function contract(value) {
  assert.deepEqual(Object.keys(value).sort(), ['concurrency', 'jobs', 'name', 'on', 'permissions']);
  assert.equal(value.name, 'Verification & Validation');
  assert.deepEqual(value.on, {push: {branches: ['main'], tags: ['v*']}, workflow_dispatch: null});
  assert.deepEqual(value.permissions, {contents: 'read'});
  assert.deepEqual(value.concurrency, {group: '${{ github.workflow }}-${{ github.ref }}', 'cancel-in-progress': true});
  assert.deepEqual(Object.keys(value.jobs), ['source-parts', 'vv']);
  const part = value.jobs['source-parts'], final = value.jobs.vv;
  assert.deepEqual(Object.keys(part).sort(), ['name', 'runs-on', 'steps', 'strategy', 'timeout-minutes']);
  assert.equal(part.name, 'source-vv-part-${{ matrix.part }}');
  assert.equal(part['runs-on'], 'ubuntu-24.04'); assert.equal(part['timeout-minutes'], 360);
  assert.deepEqual(part.strategy, {'fail-fast': false, 'max-parallel': 4, matrix: {part: parts}});
  assert.equal(part.steps.length, 12);
  assert.deepEqual(part.steps.map(step => step.id ?? step.name ?? step.uses), [checkout,
    'Check pinned bootstrap source availability', 'Start bounded host resource diagnostics', 'cache',
    'Prepare pinned checkpoint uploader', 'Start disk checkpoints', 'evidence',
    'Run the complete pinned devcontainer source part', 'Stop disk checkpoints', 'Stop host resource diagnostics',
    'Retain CI diagnostics (not acceptance evidence)', 'Retain original source part evidence']);
  assert.deepEqual(part.steps[0], {uses: checkout, with: {'fetch-depth': 0, 'persist-credentials': false}});
  assert.deepEqual(named(part.steps, 'Check pinned bootstrap source availability'),
    {name: 'Check pinned bootstrap source availability', run: 'bash scripts/bootstrap-verify.sh --check-source'});
  const start = named(part.steps, 'Start bounded host resource diagnostics');
  assert.deepEqual(start, {name: start.name, run: 'node scripts/ci-observe.mjs start target/ci-diagnostics/host'});
  assert.deepEqual(identified(part.steps, 'cache'), {id: 'cache', uses: './.github/actions/devcontainer-cache'});
  const checkpoint = named(part.steps, 'Start disk checkpoints');
  assert.deepEqual(checkpoint.env, {PRISMPM_CI_CHECKPOINT_PART: '${{ matrix.part }}'});
  assert.equal(checkpoint['continue-on-error'], true); assert.equal(checkpoint['timeout-minutes'], 1);
  assert(checkpoint.run.includes('node scripts/ci-disk-checkpoint.mjs start "$RUNNER_TEMP/ci-disk-checkpoints" "$RUNNER_TEMP/ci-checkpoint-uploader.js"'));
  const uploader = named(part.steps, 'Prepare pinned checkpoint uploader');
  assert.equal(uploader.uses, checkout); assert.equal(uploader.with.repository, 'actions/upload-artifact');
  assert.equal(uploader.with.ref, upload.split('@')[1]); assert.equal(uploader.with['persist-credentials'], false);
  assert.equal(uploader['continue-on-error'], true); assert.equal(uploader['timeout-minutes'], 2);
  assert.deepEqual(identified(part.steps, 'evidence'), {id: 'evidence', env: {VV_PART: '${{ matrix.part }}'}, run: namingCommand});
  const gate = named(part.steps, 'Run the complete pinned devcontainer source part');
  assert.deepEqual(gate, {name: gate.name, uses: devcontainer, env: {VV_PART: '${{ matrix.part }}'}, with: {
    env: 'GITHUB_SHA\nGITHUB_RUN_ID\nGITHUB_RUN_ATTEMPT\nVV_PART\n',
    cacheFrom: '${{ steps.cache.outputs.cache-from }}', cacheTo: '${{ steps.cache.outputs.cache-to }}',
    push: 'never', runCmd: partCommand}});
  const stopDisk = named(part.steps, 'Stop disk checkpoints');
  assert.equal(stopDisk.if, 'always()'); assert.equal(stopDisk['continue-on-error'], true); assert.equal(stopDisk['timeout-minutes'], 1);
  assert(stopDisk.run.includes('node scripts/ci-disk-checkpoint.mjs stop "$RUNNER_TEMP/ci-disk-checkpoints" || true'));
  assert(stopDisk.run.includes('target/ci-diagnostics/disk-checkpoints/latest.json'));
  assert(stopDisk.run.includes('target/ci-diagnostics/disk-checkpoints/status.json'));
  const stop = named(part.steps, 'Stop host resource diagnostics');
  assert.deepEqual(stop, {name: stop.name, if: 'always()', run: 'node scripts/ci-observe.mjs stop target/ci-diagnostics/host'});
  const diagnostics = named(part.steps, 'Retain CI diagnostics (not acceptance evidence)');
  assert.deepEqual(diagnostics, {name: diagnostics.name, if: 'always()', uses: upload, with: {
    name: 'vv-diagnostics-${{ matrix.part }}-${{ github.run_id }}-${{ github.run_attempt }}',
    path: 'target/ci-diagnostics/', 'if-no-files-found': 'warn', 'retention-days': 7}});
  const retention = named(part.steps, 'Retain original source part evidence');
  assert.deepEqual(retention, {name: retention.name, if: 'always()', uses: upload, with: {
    name: '${{ steps.evidence.outputs.name }}', path: 'target/source-vv-evidence/', 'include-hidden-files': true,
    'if-no-files-found': 'error', overwrite: false, 'retention-days': 30}});
  assert(part.steps.indexOf(start) < part.steps.indexOf(gate));
  assert(part.steps.indexOf(gate) < part.steps.indexOf(stopDisk));
  assert(part.steps.indexOf(stopDisk) < part.steps.indexOf(stop));
  assert(part.steps.indexOf(stop) < part.steps.indexOf(diagnostics));
  assert(part.steps.indexOf(diagnostics) < part.steps.indexOf(retention));
  assert.deepEqual(Object.keys(final).sort(), ['if', 'name', 'needs', 'permissions', 'runs-on', 'steps', 'timeout-minutes']);
  assert.equal(final.name, 'Linux x86-64 normative acceptance'); assert.equal(final.needs, 'source-parts');
  assert.equal(final.if, '${{ always() && !cancelled() }}');
  assert.equal(final['runs-on'], 'ubuntu-24.04'); assert.equal(final['timeout-minutes'], 360);
  assert.deepEqual(final.permissions, {contents: 'read', actions: 'read'});
  assert.equal(final.steps.length, 7);
  assert.deepEqual(final.steps[0], {name: 'Require every source part to succeed',
    env: {SOURCE_PARTS_RESULT: '${{ needs.source-parts.result }}'}, run: guardCommand});
  assert.deepEqual(final.steps[1], part.steps[0]);
  assert.deepEqual(final.steps[2], {uses: 'actions/setup-node@249970729cb0ef3589644e2896645e5dc5ba9c38', with: {'node-version': '22.23.2'}});
  assert.deepEqual(identified(final.steps, 'selection'), {id: 'selection', env: {GH_TOKEN: '${{ github.token }}'}, run: selectionCommand});
  assert.deepEqual(final.steps[4], {uses: download, with: {'artifact-ids': '${{ steps.selection.outputs.ids }}',
    'merge-multiple': true, path: 'target/source-vv-evidence'}});
  assert.deepEqual(final.steps[5], {name: 'Assemble every original source part into normative evidence', run: assembleCommand});
  assert.deepEqual(final.steps[6], {name: 'Retain original source parts and immutable selection', if: 'always()', uses: upload, with: {
    name: 'vv-source-evidence-${{ github.sha }}-${{ github.run_id }}-${{ github.run_attempt }}',
    path: 'target/source-vv-evidence/\ntarget/source-vv-selection/\ntarget/vv-evidence.json\n',
    'include-hidden-files': true, 'if-no-files-found': 'error', overwrite: false, 'retention-days': 30}});
}

test('main normative V&V schedules all fifteen source owners and assembles immutable original evidence', () => {
  contract(workflow());
  const listed = spawnSync(process.execPath, [new URL('./vv-assemble.mjs', import.meta.url).pathname, '--list'], {encoding: 'utf8'});
  assert.equal(listed.status, 0, listed.stderr);
  assert.deepEqual(listed.stdout.trim().split(/\s+/).sort(), [...parts].sort(), 'scheduler must cover the complete original source owner list');
});

test('scheduler policy refuses missing owners, unsafe bounds, altered triggers and skipped execution', () => {
  const original = workflow();
  for (const mutate of [
    v => v.jobs['source-parts'].strategy.matrix.part.pop(),
    v => v.jobs['source-parts'].strategy.matrix.part.push('source'),
    v => v.jobs['source-parts'].strategy.matrix.part[0] = 'unknown',
    v => v.jobs['source-parts'].strategy.matrix.include = [{part: 'source'}],
    v => v.jobs['source-parts'].strategy['max-parallel'] = 15,
    v => v.jobs['source-parts'].strategy['fail-fast'] = true,
    v => v.jobs['source-parts'].name = 'source-${{ matrix.part }}',
    v => v.jobs.vv.name = 'Replacement acceptance',
    v => v.jobs.vv.needs = [],
    v => v.jobs.vv.if = 'always()',
    v => delete v.jobs.vv.if,
    v => v.jobs.vv.if = '${{ success() }}',
    v => v.jobs['source-parts'].if = 'false',
    v => v.jobs['source-parts']['continue-on-error'] = true,
    v => v.jobs.vv['continue-on-error'] = true,
    v => v.jobs['source-parts']['timeout-minutes'] = 361,
    v => v.jobs.vv['timeout-minutes'] = 10,
    v => v.permissions.contents = 'write',
    v => v.jobs.vv.permissions.actions = 'write',
    v => v.on.pull_request = null,
    v => v.on.push.branches.push('other'),
    v => v.on.push.tags = [],
    v => v.on.push.paths = ['crates/**'],
    v => delete v.on.workflow_dispatch,
    v => v.concurrency['cancel-in-progress'] = false,
    v => v.concurrency.group = '${{ github.workflow }}-${{ github.run_id }}',
    v => v.jobs.extra = structuredClone(v.jobs.vv),
    v => v.env = {PRISMPM_TEST_SDK_IMAGE: 'unsafe'},
  ]) {const value = structuredClone(original); mutate(value); assert.throws(() => contract(value));}
});

test('scheduler policy refuses SDK admission changes, hidden failures and mutable artifact transport', () => {
  const original = workflow();
  const gate = v => named(v.jobs['source-parts'].steps, 'Run the complete pinned devcontainer source part');
  const retention = v => named(v.jobs['source-parts'].steps, 'Retain original source part evidence');
  for (const mutate of [
    v => gate(v).with.runCmd = partCommand.replace('--with-sdk', '--without-sdk'),
    v => gate(v).with.runCmd = partCommand.replace('--with-sdk', '--with-shared-sdk'),
    v => gate(v).with.runCmd = partCommand.replace('source-part-run', 'source-run'),
    v => gate(v).with.runCmd = partCommand.replace('"$VV_PART"', 'source'),
    v => gate(v).with.runCmd = partCommand.replace('amd64 -', 'amd64 "$PRISMPM_TEST_SDK_IMAGE"'),
    v => gate(v).with.runCmd += 'true\n',
    v => gate(v).with.runCmd = partCommand.replace('"$VV_PART"', '"$VV_PART" || true'),
    v => gate(v).if = 'false', v => gate(v)['continue-on-error'] = true,
    v => gate(v).with.push = 'always',
    v => gate(v).env.PRISMPM_TEST_SDK_IMAGE = 'unsafe',
    v => gate(v).with.env += 'PRISMPM_TEST_SDK_IMAGE\n',
    v => identified(v.jobs['source-parts'].steps, 'cache').with = {'cache-from': 'unowned'},
    v => named(v.jobs['source-parts'].steps, 'Start disk checkpoints').env = {},
    v => retention(v).if = 'success()',
    v => retention(v).with.name = 'source-part-${{ matrix.part }}',
    v => retention(v).with['include-hidden-files'] = false,
    v => retention(v).with['if-no-files-found'] = 'warn',
    v => retention(v).with.overwrite = true,
    v => identified(v.jobs.vv.steps, 'selection').run = selectionCommand.replace('"$GITHUB_RUN_ATTEMPT"', '1'),
    v => identified(v.jobs.vv.steps, 'selection').run = selectionCommand.replace('--list', '--subset'),
    v => v.jobs.vv.steps[4].with = {name: 'source-vv', path: 'target/source-vv-evidence'},
    v => v.jobs.vv.steps[4].with['artifact-ids'] = '123',
    v => v.jobs.vv.steps[4].with.pattern = 'prismpm-*',
    v => v.jobs.vv.steps[5].run = assembleCommand.replace('source-part-assemble', 'source-assemble'),
    v => v.jobs.vv.steps[5].run += ' || true\n',
    v => v.jobs.vv.steps[5].if = 'always()',
    v => v.jobs.vv.steps[6].if = 'success()',
    v => v.jobs.vv.steps[6].with.path = 'target/vv-evidence.json',
  ]) {const value = structuredClone(original); mutate(value); assert.throws(() => contract(value));}
});

test('the first final-job guard rejects every incomplete matrix outcome before checkout or provider access', () => {
  const original = workflow(), guard = original.jobs.vv.steps[0];
  for (const outcome of ['success', 'failure', 'cancelled', 'skipped', '', 'success || true']) {
    const result = spawnSync('/bin/bash', ['-c', guard.run], {env: {...process.env, SOURCE_PARTS_RESULT: outcome}, encoding: 'utf8'});
    assert.equal(result.status === 0, outcome === 'success', outcome);
  }
  for (const mutate of [
    v => v.jobs.vv.steps.shift(),
    v => {const guard = v.jobs.vv.steps.shift(); v.jobs.vv.steps.splice(1, 0, guard);},
    v => v.jobs.vv.steps[0].run = guardCommand.replace('= success', '!= failure'),
    v => v.jobs.vv.steps[0].run = guardCommand.replace('= success', '= success || true'),
    v => v.jobs.vv.steps[0].run = 'true\n',
    v => v.jobs.vv.steps[0].env.SOURCE_PARTS_RESULT = 'success',
    v => v.jobs.vv.steps[0].if = 'always()',
    v => v.jobs.vv.steps[0]['continue-on-error'] = true,
    v => v.jobs.vv.steps[1].if = 'always()',
    v => identified(v.jobs.vv.steps, 'selection').if = 'always()',
    v => v.jobs.vv.steps[4].if = 'always()',
    v => v.jobs.vv.steps[5].if = 'always()',
  ]) {const value = structuredClone(original); mutate(value); assert.throws(() => contract(value));}
});

test('all source scheduling shell blocks parse without starting any SDK or hosted command', () => {
  for (const job of Object.values(workflow().jobs)) for (const step of job.steps) {
    const body = step.run ?? step.with?.runCmd;
    if (typeof body !== 'string') continue;
    const result = spawnSync('/bin/bash', ['-n'], {input: body, encoding: 'utf8'});
    assert.equal(result.status, 0, result.stderr);
  }
});
