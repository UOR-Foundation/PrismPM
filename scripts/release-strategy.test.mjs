import assert from 'node:assert/strict';
import {mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import {createRequire} from 'node:module';
import {spawnSync} from 'node:child_process';
import test from 'node:test';
import {releaseConstructionPlan} from './release-construction-plan.mjs';

const {load} = createRequire('/opt/prismpm/oracles/package.json')('js-yaml');
const workflow = () => load(readFileSync(new URL('../.github/workflows/release.yml', import.meta.url), 'utf8'));
const sdkTargets = ['runtime', 'adapter-compose', 'adapter-kubernetes', 'adapter-github-pages', 'oracles', 'cli-archive'];
test('one exact release producer plan includes all SDK family outputs and the native archive', () => {
  const revision = 'a'.repeat(40), plan = releaseConstructionPlan(revision, 'amd64', 'owned-builder', '/tmp/release-521');
  assert.equal(plan.platform, 'linux/amd64'); assert.equal(plan.builder, 'owned-builder');
  assert.deepEqual(plan.builds.map(row => row.target), sdkTargets);
  for (const row of plan.builds.slice(0, -1)) {
    assert(row.args.includes('org.opencontainers.image.revision=' + revision));
    assert(row.args.includes('--provenance=false')); assert(row.args.includes('--sbom=false'));
    assert(row.args.includes('SOURCE_DATE_EPOCH=0'));
    assert.match(row.args[row.args.indexOf('--output') + 1], /^type=registry,name=ghcr.io\/uor-foundation\/prismpm-[a-z-]+,push-by-digest=true,name-canonical=true,rewrite-timestamp=true,oci-mediatypes=true$/);
  }
  assert.deepEqual(plan.builds.at(-1), {target: 'cli-archive', args: ['--build-arg', 'SDK_VERSION=0.4.0', '--output', 'type=local,dest=/tmp/release-521/native-release']});
  for (const args of [[revision, 'riscv64', 'b', '/tmp/x'], ['main', 'amd64', 'b', '/tmp/x'], [revision, 'amd64', '--use', '/tmp/x']])
    assert.throws(() => releaseConstructionPlan(...args));
});

function graphContract(value) {
  const j = value.jobs;
  for (const [name, needs] of Object.entries({
    'image-platforms': 'policy', 'sdk-supply': ['policy', 'image-platforms'],
    'product-images': ['policy', 'image-platforms'], images: ['sdk-supply', 'product-images'],
    gate: ['policy', 'sdk-supply'], native: ['policy', 'sdk-supply'],
    reproducibility: ['gate', 'images'], 'installed-sdk': ['gate', 'images'],
    'oci-native': ['gate', 'images', 'native', 'reproducibility', 'installed-sdk'],
    crates: ['gate', 'images', 'native', 'reproducibility', 'installed-sdk', 'oci-native'],
    release: ['gate', 'images', 'native', 'reproducibility', 'installed-sdk', 'oci-native', 'crates'],
  })) {
    assert.deepEqual(j[name].needs, needs, name + ' complete real-input/final-gate contract');
    assert.equal(j[name]['continue-on-error'], undefined);
    if (!['crates', 'release'].includes(name)) assert.equal(j[name].if, undefined, name + ' default failure propagation');
  }
  assert.equal(j['sdk-supply'].strategy.matrix.include[0].name, 'sdk');
  assert.equal(j.release.if, undefined);
  assert.equal(j.crates.if, "${{ needs.gate.outputs.publish-crates == 'true' }}");
  assert.deepEqual(j['product-images'].strategy.matrix.include.map(row => row.name), ['runtime', 'adapter-compose', 'adapter-kubernetes', 'adapter-github-pages', 'oracles']);
  assert.equal(j.reproducibility.strategy.matrix.include.length, 12);
  assert.deepEqual(j.images.steps, [{run: 'true'}]);
}
test('release fanout retains all original fail-closed final gates and owner inventories', () => {
  graphContract(workflow());
  for (const name of ['images', 'reproducibility', 'installed-sdk', 'gate', 'oci-native', 'release']) for (const mutation of [
    v => v.jobs[name].needs = [], v => v.jobs[name].if = 'always()', v => v.jobs[name]['continue-on-error'] = true,
  ]) {
    const value = workflow(); mutation(value); assert.throws(() => graphContract(value));
  }
});
test('all shell blocks parse, source handoff precedes unchanged source owner and native never rebuilds', () => {
  const jobs = workflow().jobs;
  for (const [name, job] of Object.entries(jobs)) for (const step of job.steps)
    for (const script of [step.run, step.with?.runCmd].filter(Boolean))
      assert.equal(spawnSync('bash', ['-n'], {input: script}).status, 0, name);
  const source = jobs.gate.steps.find(step => step.with?.runCmd).with.runCmd;
  assert(source.includes('bash scripts/source-sdk-handoff.sh "$image"'));
  assert(source.includes('bash scripts/vv.sh --with-sdk bash -euo pipefail'));
  assert(source.includes('amd64 "$PRISMPM_TEST_SDK_IMAGE"'));
  assert.equal((source.match(/source-run/g)||[]).length, 2);
  assert(!jobs.native.steps.some(step => step.run?.includes('sdk-image-inputs.mjs build')));
  assert(jobs.native.steps.some(step => step.with?.name === 'native-construction-${{ matrix.arch }}'));
  const repro = jobs.reproducibility.steps.find(step => step.env?.DOCKERFILE).run;
  assert(repro.includes('for root in root-a root-b'));
  assert(repro.includes('--builder "$builder" --no-cache'));
  assert(repro.indexOf('docker buildx rm "$builder"') < repro.indexOf('cmp root-a.digest'));
  assert(!/--cache-from|--cache-to|--import-cache/.test(repro));
  assert(repro.includes('root-a.oci root-b.oci >'));
});

test('provider collector acquires all-attempt current-run originals with create-only output', t => {
  const root = mkdtempSync(join(tmpdir(), 'release-provider-521-'));
  t.after(() => rmSync(root, {recursive: true, force: true}));
  const artifact = {id: 11, untouched: 'artifact'}, job = {id: 12, untouched: 'job'};
  writeFileSync(join(root, 'gh'), `#!${process.execPath}\nconst fs=require('node:fs');const args=process.argv.slice(2);fs.appendFileSync(process.env.RECORD,JSON.stringify(args)+'\\n');console.log(JSON.stringify(args.at(-1).includes('/artifacts?')?[{artifacts:[${JSON.stringify(artifact)}]}]:[{jobs:[${JSON.stringify(job)}]}]));\n`, {mode: 0o755});
  const output = join(root, 'original.json'), record = join(root, 'calls');
  const args = [new URL('./release-artifact-metadata.mjs', import.meta.url).pathname, output, '123'];
  const env = {...process.env, PATH: root + ':' + process.env.PATH, GITHUB_REPOSITORY: 'UOR-Foundation/PrismPM', GITHUB_RUN_ID: '123', RECORD: record};
  assert.equal(spawnSync(process.execPath, args, {env}).status, 0);
  assert.deepEqual(JSON.parse(readFileSync(output)), {artifacts: [artifact], jobs: [job]});
  assert.deepEqual(readFileSync(record, 'utf8').trim().split('\n').map(JSON.parse), [
    ['api', '--paginate', '--slurp', '/repos/UOR-Foundation/PrismPM/actions/runs/123/artifacts?per_page=100'],
    ['api', '--paginate', '--slurp', '/repos/UOR-Foundation/PrismPM/actions/runs/123/jobs?filter=all&per_page=100'],
  ]);
  const original = readFileSync(output);
  assert.notEqual(spawnSync(process.execPath, args, {env}).status, 0);
  assert.deepEqual(readFileSync(output), original);
  assert.notEqual(spawnSync(process.execPath, args, {env: {...env, GITHUB_RUN_ID: '124'}}).status, 0);
});
