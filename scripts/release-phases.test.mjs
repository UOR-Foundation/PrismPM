import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { chmodSync, linkSync, mkdtempSync, mkdirSync, readFileSync, readdirSync, renameSync, rmSync,
  symlinkSync, writeFileSync } from 'node:fs';
import fs from 'node:fs';
import { createRequire, syncBuiltinESMExports } from 'node:module';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import { parts as vvParts } from './vv-assemble.mjs';
import {sessionFixture} from './vv-sdk-session-fixture.mjs';
import { assetNames, imageVerificationArguments, platformIndex, publicationNotes, publicationPolicy, publishOci, releaseSbomRecord, requirePrerequisites, verifyReproducibility, verifyExistingRelease } from './release-phases.mjs';

const require = createRequire('/opt/prismpm/oracles/package.json');
const { load } = require('js-yaml');
const source = readFileSync(new URL('../.github/workflows/release.yml', import.meta.url), 'utf8');
const workflow = () => load(source);

test('canonical Cargo Git caches retain every object and real locked offline submodule builds', () => {
  execFileSync('python3', ['-I', '-B', new URL('./normalize-cargo-git_test.py', import.meta.url).pathname],
    {timeout: 120000});
});

test('OCI mismatch diagnostics execute bounded real archive fixtures and remain failure-only', () => {
  execFileSync('python3', ['-B', new URL('./oci-repro-diagnostics_test.py', import.meta.url).pathname]);
  const steps = workflow().jobs['reproducibility-parts'].steps;
  const compare = steps.find(step => step.env?.DOCKERFILE);
  assert.match(compare.run, /cmp root-a.digest root-b.digest\n/);
  assert.match(compare.run, /cmp root-a.oci\/index.json root-b.oci\/index.json\n/);
  const diagnostic = steps.find(step => step.run?.includes('oci-repro-diagnostics.py'));
  assert.equal(diagnostic.if, 'failure()');
  assert.equal(diagnostic['timeout-minutes'], 5);
  assert(steps.indexOf(diagnostic) > steps.indexOf(compare));
  const retained = steps.find(step => step.with?.name === 'reproducibility-diagnostics-${{ matrix.name }}');
  assert.equal(retained.if, 'failure()');
  assert.equal(retained.with.path, 'reproducibility-diagnostics.*');
});

const revision = 'a'.repeat(40);
const sha = bytes => `sha256:${createHash('sha256').update(bytes).digest('hex')}`;
const policy = () => ({repository: 'UOR-Foundation/PrismPM', ref: 'refs/heads/release/v0.4',
  event: 'workflow_dispatch', version: '0.4.0', publishCrates: false, rehearsal: false, revision});
const prerequisites = names => Object.fromEntries(names.map(name => [name, {result: 'success'}]));
const ociNeeds = ['gate', 'images', 'native', 'reproducibility', 'installed-sdk'];
const buildkit = 'moby/buildkit@sha256:de10faf919fc71ba4eb1dd7bd6449566d012b0c9436b1c61bfee21d621b009aa';
const environment = () => ({GITHUB_REPOSITORY: 'UOR-Foundation/PrismPM', GITHUB_REF: 'refs/heads/release/v0.4',
  GITHUB_EVENT_NAME: 'workflow_dispatch', GITHUB_SHA: revision, DISPATCH_VERSION: '0.4.0', PUBLISH_CRATES: 'false', REHEARSAL: 'false',
  GITHUB_ACTIONS: 'true', GITHUB_RUN_ID: '123456789', GITHUB_RUN_ATTEMPT: '1'});
const publicationTag = `sdk-oci-${revision}-123456789-1`;

const semanticReadinessMutations = [
  run => run.replace(/^helper=.*\n/m, ''),
  run => run.replace(/^test "\$helper".*\n/m, ''),
  run => run.replace(/^node_path=.*\n/m, ''),
  run => run.replace('release-semantic-target-node-oracles/node_modules', 'unexpected-node-oracles/node_modules'),
  run => run.replace('PRISMPM_SEMANTIC_SOURCE=%s\\n', ''),
  run => run.replace('NODE_PATH=%s\\n', ''),
  run => run.replace('"$helper" "$node_path"', '"$helper" "${NODE_PATH:-}"'),
];

function semanticPrerequisite(job, readers, workingDirectory) {
  const matches = job.steps.filter(step => step.run?.includes('bash scripts/semantic-source-bootstrap.sh'));
  assert.equal(matches.length, 1, 'one same-source semantic helper prerequisite');
  const step = matches[0];
  assert.equal(step.if, undefined); assert.equal(step['continue-on-error'], undefined);
  assert.equal(step['working-directory'], workingDirectory);
  assert.deepEqual(step.env, {CARGO_HOME: '${{ runner.temp }}/release-semantic-cargo', CARGO_NET_OFFLINE: 'true'});
  assert.equal(step.run, `set -euo pipefail
bash scripts/semantic-source-bootstrap.sh "$GITHUB_SHA" \\
  "$RUNNER_TEMP/release-semantic-target" "$RUNNER_TEMP/release-semantic-evidence"
helper=$(cat "$RUNNER_TEMP/release-semantic-evidence/semantic-source-path.txt")
test "$helper" = "$RUNNER_TEMP/release-semantic-target/debug/prismpm-semantic-source"
node_path=$(cat "$RUNNER_TEMP/release-semantic-evidence/node-oracles-path.txt")
test "$node_path" = "$RUNNER_TEMP/release-semantic-target-node-oracles/node_modules"
printf 'PRISMPM_SEMANTIC_SOURCE=%s\\nNODE_PATH=%s\\n' "$helper" "$node_path" >> "$GITHUB_ENV"
`);
  for (const reader of readers) {
    assert(reader, 'complete proof-reading owner required');
    assert(job.steps.indexOf(step) < job.steps.indexOf(reader), 'source helper must precede every proof reader');
  }
}

function validateWorkflow(value) {
  assert.deepEqual(Object.keys(value.on).sort(), ['push', 'workflow_dispatch']);
  assert.deepEqual(value.on.push, {branches: ['release/v0.4', 'resource-diagnostic/*'], tags: ['v0.4.0']});
  assert.equal(value.jobs.policy.if, "${{ inputs.resource-diagnostic-source == '' && !startsWith(github.ref, 'refs/heads/resource-diagnostic/') }}");
  for (const name of ['diagnostic-policy', 'diagnostic-production', 'diagnostic-pair']) {
    assert.equal(value.jobs[name].if, "${{ (github.event_name == 'workflow_dispatch' && inputs.resource-diagnostic-source != '') || (github.event_name == 'push' && startsWith(github.ref, 'refs/heads/resource-diagnostic/')) }}");
    assert.equal(value.jobs[name]['continue-on-error'], undefined);
  }
  assert.equal(value.jobs['diagnostic-policy'].needs, undefined);
  assert.equal(value.jobs['diagnostic-production'].needs, 'diagnostic-policy');
  assert.deepEqual(value.jobs['diagnostic-pair'].needs, ['diagnostic-policy', 'diagnostic-production']);
  assert.deepEqual(value.jobs['oci-native'].needs, ociNeeds);
  assert.equal(value.jobs['oci-native'].if, undefined, 'normal success gating must not be bypassed');
  assert.deepEqual(value.jobs.release.needs, [...ociNeeds, 'oci-native', 'crates']);
  assert.equal(value.jobs.release.if, "${{ needs.gate.outputs.rehearsal != 'true' }}");
  assert.equal(value.jobs.crates.if, "${{ needs.gate.outputs.publish-crates == 'true' && needs.gate.outputs.rehearsal != 'true' }}");
  assert.deepEqual(value.on.workflow_dispatch.inputs.rehearsal, {description: 'Run every check but publish no release assets, discovery tags or crates. Images are still pushed by digest and signed.',
    required: true, default: true, type: 'boolean'});
  for (const name of ['oci-native']) {
    const publishing = value.jobs[name].steps.filter(step => step.run?.includes('publish-oci') || step.uses?.startsWith('actions/attest-build-provenance'));
    assert.equal(publishing.length, 2);
    for (const step of publishing) assert.equal(step.if, "${{ needs.gate.outputs.rehearsal != 'true' }}");
  }
  assert.deepEqual(value.jobs.crates.needs, [...ociNeeds, 'oci-native']);
  const cargoPublication = value.jobs.crates.steps.find(step => step.run?.includes('publish_exact()'))?.run;
  assert.ok(cargoPublication, 'Cargo publication must select and verify the exact archive');
  let previous = -1;
  for (const checkpoint of [
    'package_target=${archive%/package/*}',
    'test "$archive" = "$package_target/package/$package-$version.crate"',
    'release_cargo "$package" package --locked --manifest-path "$manifest" --target-dir "$package_target"',
    'test -f "$archive"',
    'cmp "$archive" "$expected"',
    'local_checksum=$(sha256sum "$archive" | cut -d" " -f1)',
    'release_cargo "$package" publish --dry-run --locked --manifest-path "$manifest" --target-dir "$package_target"',
    'test "$(sha256sum "$archive" | cut -d" " -f1)" = "$local_checksum"',
    './scripts/crate-metadata-check.sh "$archive"',
    'if [ "$UPLOAD" != true ]; then',
    'cargo publish --locked --manifest-path "$manifest" --target-dir "$package_target"',
  ]) {
    const offset = cargoPublication.indexOf(checkpoint);
    assert.ok(offset > previous, `Cargo publication omits or misorders ${checkpoint}`);
    assert.equal(cargoPublication.lastIndexOf(checkpoint), offset, `Cargo publication repeats ${checkpoint}`);
    previous = offset;
  }
  assert.deepEqual(cargoPublication.split('\n').map(line => line.trim())
    .filter(line => /^(release_cargo "\$package" |cargo )(package|publish)/.test(line)), [
    'release_cargo "$package" package --locked --manifest-path "$manifest" --target-dir "$package_target"',
    'release_cargo "$package" publish --dry-run --locked --manifest-path "$manifest" --target-dir "$package_target"',
    'cargo publish --locked --manifest-path "$manifest" --target-dir "$package_target"',
  ], 'Cargo must retain complete verification for every selected-target operation, and only the last one uploads');
  // The upload happens only with a credential: the skip returns before it, the
  // wait for lexlean is conditional on it, and no step hands Cargo a token
  // without the secret or the trusted-publishing switch (#214).
  assert.ok(cargoPublication.indexOf('return 0', cargoPublication.indexOf('if [ "$UPLOAD" != true ]; then'))
    < cargoPublication.indexOf('cargo publish --locked --manifest-path'), 'the skip must return before the upload');
  assert.match(cargoPublication, /if \[ "\$UPLOAD" = true \]; then\n\s+wait_for_crate lexlean 0\.4\.0\n\s+fi/);
  assert.match(cargoPublication, /release_cargo\(\) \{\n\s+name=\$1\n\s+shift\n\s+if \[ "\$UPLOAD" = true \]; then\n\s+cargo "\$@"\n\s+elif \[ "\$name" = prod-codegen \]/);
  for (const dependency of ['lexlean', 'prod-ir', 'prod-codegen', 'prism-stdlib']) {
    assert.ok(cargoPublication.includes(`--config "patch.crates-io.${dependency}.path=`), `${dependency} must resolve from the reviewed copy before upload`);
  }
  assert.equal(value.jobs.crates.env.HAS_CARGO_REGISTRY_TOKEN, "${{ secrets.CARGO_REGISTRY_TOKEN != '' }}");
  assert.equal(value.jobs.crates.env.TRUSTED_PUBLISHING, "${{ vars.CRATES_IO_TRUSTED_PUBLISHING == 'true' }}");
  const authentication = value.jobs.crates.steps.find(step => step.uses?.startsWith('rust-lang/crates-io-auth-action'));
  assert.equal(authentication.if, "${{ env.HAS_CARGO_REGISTRY_TOKEN != 'true' && env.TRUSTED_PUBLISHING == 'true' }}");
  const publicationStep = value.jobs.crates.steps.find(step => step.run?.includes('publish_exact()'));
  assert.equal(publicationStep.env.CARGO_REGISTRY_TOKEN, '${{ secrets.CARGO_REGISTRY_TOKEN || steps.auth.outputs.token }}');
  assert.equal(publicationStep.env.UPLOAD, "${{ env.HAS_CARGO_REGISTRY_TOKEN == 'true' || env.TRUSTED_PUBLISHING == 'true' }}");
  assert.match(publicationStep.run, /--env UPLOAD \\\n/);
  // The job script is one single-quoted `bash -c` argument: an apostrophe in a
  // comment or a message inside it ends the quote and breaks the whole job.
  execFileSync('bash', ['-n'], {input: publicationStep.run, stdio: ['pipe', 'pipe', 'pipe']});
  assert.equal(value.jobs.policy.outputs['publish-crates'], '${{ steps.version.outputs.publish-crates }}');
  assert.equal(value.jobs.policy.outputs.rehearsal, '${{ steps.version.outputs.rehearsal }}');
  assert.equal(value.jobs.gate.outputs['publish-crates'], '${{ needs.policy.outputs.publish-crates }}');
  assert.equal(value.jobs.gate.outputs.rehearsal, '${{ needs.policy.outputs.rehearsal }}');
  assert.match(value.jobs.policy.steps.find(step => step.id === 'version').run, /release-phases\.mjs policy/);
  assert(value.jobs.policy.steps.some(step => step.run === 'node --test scripts/release-native-contract.test.mjs'));
  assert.deepEqual(value.jobs['source-vv'].needs, ['policy', 'sdk-supply'], 'source verification consumes the independently signed SDK');
  const sourceSteps = value.jobs['source-vv'].steps;
  assert(sourceSteps.some(step => step.id === 'cache' && step.uses === './.github/actions/devcontainer-cache'));
  const sourceBuild = sourceSteps.find(step => step.with?.runCmd).with;
  assert.equal(sourceBuild.cacheFrom, '${{ steps.cache.outputs.cache-from }}');
  assert.equal(sourceBuild.cacheTo, undefined, 'source shards consume the construction cache without concurrent writers');
  assert.equal(sourceBuild.runCmd, `set -euo pipefail
bash scripts/sdk-candidate.sh install-oras "$PWD/target/release-tools"
bash scripts/sdk-candidate.sh install-cosign "$PWD/target/release-tools"
export PATH="$PWD/target/release-tools:$PATH"
image=$(cat target/release-sdk/sdk-image.txt)
bash scripts/source-sdk-handoff.sh "$image" '\${{ github.sha }}' amd64 '\${{ github.ref }}' '\${{ github.event_name }}' \\
  bash scripts/vv.sh --with-sdk node scripts/release-gate-evidence.mjs source-run . target/source-vv-evidence \\
  '\${{ github.sha }}' amd64 "$image" '\${{ github.run_id }}' '\${{ github.run_attempt }}' '\${{ matrix.part }}'
`);
  assert.deepEqual(value.jobs['source-vv'].strategy.matrix.include.map(row => row.part).sort(), [...vvParts].sort());
  assert.equal(value.jobs['source-vv'].strategy.matrix.include.length, vvParts.length);
  const sourceUpload = value.jobs.gate.steps.find(step => step.uses?.startsWith('actions/upload-artifact'));
  assert.equal(sourceUpload.if, undefined); assert.equal(sourceUpload.with.path, 'target/source-vv-evidence/');
  assert.equal(sourceUpload.with.name, '${{ steps.evidence.outputs.name }}');
  assert.equal(sourceUpload.with['if-no-files-found'], 'error');
  assert.deepEqual(value.jobs.gate.needs, ['policy', 'sdk-supply', 'source-vv']);
  assert.deepEqual(value.jobs.images.needs, ['sdk-supply', 'product-images']);
  assert.deepEqual(value.jobs.reproducibility.needs, ['reproducibility-parts', 'browser-library']);
  assert.deepEqual(value.jobs['reproducibility-parts'].needs, ['policy', 'images']);
  const installed = value.jobs['installed-sdk'], parts = value.jobs['installed-vv'];
  assert.deepEqual(installed.needs, ['policy', 'sdk-supply', 'installed-vv', 'product-cli']);
  const nativePlatforms = [{os:'ubuntu-24.04',arch:'amd64'}, {os:'ubuntu-24.04-arm',arch:'arm64'}];
  for (const name of ['source-vv', 'installed-vv', 'product-cli', 'browser-library', 'native'])
    assert.deepEqual(value.jobs[name].needs, ['policy', 'sdk-supply']);
  for (const name of ['gate', 'images', 'reproducibility', 'installed-sdk', 'reproducibility-parts', 'installed-vv', 'product-cli', 'browser-library', 'native']) {
    const job = value.jobs[name];
    assert.equal(job.if, undefined); assert.equal(job['continue-on-error'], undefined);
    for (const step of job.steps) assert.equal(step['continue-on-error'], undefined);
  }
  for (const job of [installed, parts, value.jobs['product-cli'], value.jobs['browser-library']])
    assert.equal(job['runs-on'], '${{ matrix.os }}');
  for (const name of ['installed-sdk', 'product-cli', 'browser-library'])
    assert.deepEqual(value.jobs[name].strategy.matrix.include, nativePlatforms);
  const byKey = row => `${row.arch}/${row.part}`;
  assert.deepEqual(parts.strategy.matrix.include.map(byKey).sort(), nativePlatforms
    .flatMap(row => vvParts.map(part => byKey({...row, part}))).sort());
  assert.equal(parts.strategy.matrix.include.length, vvParts.length * 2);
  for (const row of parts.strategy.matrix.include) assert.equal(row.os, row.arch === 'amd64' ? 'ubuntu-24.04' : 'ubuntu-24.04-arm');
  const complete = parts.steps.find(step => step.run?.includes('sdk-vv-outer.sh'));
  assert(complete); assert.equal(complete.if, undefined);
  assert.match(complete.run, /bash root-a\/scripts\/sdk-vv-outer\.sh/);
  const outer = readFileSync(new URL('./sdk-vv-outer.sh', import.meta.url), 'utf8');
  assert(outer.includes('node root-a/scripts/sdk-vv-check.mjs tests; node root-a/scripts/sdk-vv-check.mjs run'));
  assert(outer.includes('--target registry_qualification_tools'));
  assert(parts.steps.some(step => step.with?.name === 'sdk-image' && step.with.path === '.shipped-image'));
  const retained = parts.steps.find(step => step.uses?.startsWith('actions/upload-artifact'));
  assert.equal(retained.with.name, '${{ steps.evidence.outputs.name }}');
  assert.equal(retained.if, 'always()'); assert.equal(retained.with['if-no-files-found'], 'error');
  assert(retained.with.path.includes('!sdk-${{ matrix.arch }}-full-sdk-vv-${{ matrix.part }}/docker/'), 'private transport data must not be uploaded');
  const assembled = installed.steps.find(step => step.run?.includes('assemble-installed'));
  assert(assembled); assert.equal(assembled.if, undefined);
  const product = value.jobs['product-cli'].steps.find(step => step.run?.includes('product-sdk-check.sh'));
  assert(product && product.if === undefined);
  assert.match(product.run, /^set -euo pipefail\n/);
  assert.match(product.run, /node root-a\/scripts\/product-sdk-check\.mjs tests/);
  const browser = value.jobs['browser-library'].steps.find(step => step.run?.includes('browser-api-sdk-check.sh'));
  const library = value.jobs['browser-library'].steps.find(step => step.run?.includes('library-sdk-check.sh'));
  const binary = value.jobs['browser-library'].steps.find(step => step.run?.includes('binary-sdk-check.sh'));
  assert(binary && binary.if === undefined && binary['continue-on-error'] === undefined);
  assert.deepEqual(binary.env, {RESULT_NAME: 'sdk-${{ matrix.arch }}', ARCH: '${{ matrix.arch }}'});
  assert.equal(binary.run, `set -euo pipefail
mkdir "$RUNNER_TEMP/binary-transport-1.1" "$RUNNER_TEMP/binary-transport-1.2"
node root-a/scripts/binary-sdk-check.mjs tests
image=$(cat .shipped-image/sdk-image.txt)
bash root-a/scripts/binary-sdk-check.sh "$image" "$GITHUB_SHA" "$RUNNER_TEMP/binary-transport-1.1/$RESULT_NAME-binary-sdk" > "$RUNNER_TEMP/binary-transport-1.1/$RESULT_NAME-binary-sdk.log" 2>&1
bash root-a/scripts/binary-sdk-check.sh "$image" "$GITHUB_SHA" "$RUNNER_TEMP/binary-transport-1.2/$RESULT_NAME-binary-sdk-language-1.2" 1.2 > "$RUNNER_TEMP/binary-transport-1.2/$RESULT_NAME-binary-sdk-language-1.2.log" 2>&1
node root-a/scripts/binary-sdk-release-evidence.mjs retain "$RUNNER_TEMP/binary-transport-1.1" "$image" "$GITHUB_SHA" "$ARCH" 1.1 "$GITHUB_RUN_ID" "$GITHUB_RUN_ATTEMPT"
node root-a/scripts/binary-sdk-release-evidence.mjs retain "$RUNNER_TEMP/binary-transport-1.2" "$image" "$GITHUB_SHA" "$ARCH" 1.2 "$GITHUB_RUN_ID" "$GITHUB_RUN_ATTEMPT"
`);
  semanticPrerequisite(value.jobs['browser-library'], [browser, library, binary], 'root-a');
  const retainedProof = value.jobs['oci-native'].steps.find(step => step.run?.includes('library-sdk-metadata-evidence.mjs join'));
  semanticPrerequisite(value.jobs['oci-native'], [retainedProof], undefined);
  assert(browser && browser.if === undefined && library && library.if === undefined);
  for (const step of [product, browser, library, binary]) {
    assert.match(step.run, /^set -euo pipefail\n/);
    assert(!step.run.includes('||'), 'independent owner failure must propagate');
    if (step !== binary) assert(step.run.includes('"$(cat .shipped-image/sdk-image.txt)" "$GITHUB_SHA"')
      || step.run.includes('"$(cat .shipped-image/sdk-image.txt)" \\\n  "$GITHUB_SHA"'));
  }
  assert.match(library.run, /node root-a\/scripts\/library-sdk-check\.mjs tests/);
  const platformJob = value.jobs['image-platforms'];
  assert.equal(platformJob.needs, 'policy');
  assert.equal(platformJob['runs-on'], '${{ matrix.os }}');
  assert.deepEqual(platformJob.strategy.matrix.include, nativePlatforms);
  for (const job of Object.values(value.jobs))
    assert.ok(!job.steps.some(step => step.uses?.startsWith('docker/setup-qemu-action')), 'every platform is built natively');
  const builds = platformJob.steps.find(step => step.id === 'build');
  assert.equal(builds.uses, undefined); assert.equal(builds.with, undefined);
  assert.match(builds.run, /plan=\$\(node scripts\/release-construction-plan\.mjs "\$GITHUB_SHA" "\$ARCH" "\$BUILDER" "\$RUNNER_TEMP\/image-platforms-work"\)/);
  assert.match(builds.run, /node scripts\/sdk-image-inputs\.mjs build-set \. "\$GITHUB_SHA" "\$plan"/);
  assert.equal(builds.env.BUILDER, '${{ steps.builder.outputs.name }}');
  assert.match(builds.run, /docker buildx build --builder "\$BUILDER" --file sdk\/runtime\/Dockerfile --platform "linux\/\$ARCH"/);
  assert.match(builds.run, /--output "type=registry,name=\$repository,push-by-digest=true,name-canonical=true,rewrite-timestamp=true,oci-mediatypes=true"/);
  assert.ok(!builds.run.includes('--tag'), 'unaccepted publication must not move discovery aliases');
  assert.match(builds.run, /--provenance=false --sbom=false --build-arg SOURCE_DATE_EPOCH=0/);
  assert.match(builds.run, /node scripts\/sdk-image-inputs\.mjs digest "\$RUNNER_TEMP\/image-platforms-work\/\$name-build\.json"/);
  const table = builds.run.split("<<'SDK_IMAGES'\n")[1].split('SDK_IMAGES\n')[0].trim().split('\n').map(line => line.split(' '));
  const supplyNames = ['sdk-supply', 'product-images'];
  const imageRows = supplyNames.flatMap(name => value.jobs[name].strategy.matrix.include);
  assert.deepEqual(imageRows.map(row => row.name), ['sdk', 'runtime', 'adapter-compose', 'adapter-kubernetes', 'adapter-github-pages', 'oracles']);
  assert.deepEqual(table, imageRows.filter(row => row.dockerfile === 'sdk/Dockerfile').map(row => [row.name, row.target]));
  const native = value.jobs.native;
  assert(native.steps.some(step => step.with?.name === 'native-construction-${{ matrix.arch }}' && step.with.path === '.native-release'));
  assert(!native.steps.some(step => step.run?.includes('sdk-image-inputs.mjs build')), 'native acceptance must consume the original archive');
  const equivalence = native.steps.find(step => step.run?.includes('release-gate-evidence.mjs native'));
  assert.match(equivalence.run, /sha256sum --check/); assert.equal(equivalence.if, undefined);
  const rebuild = value.jobs['reproducibility-parts'].steps.find(step => step.env?.DOCKERFILE);
  for (const key of ['created', 'revision', 'source', 'version']) {
    assert.ok(builds.run.includes(`--label "org.opencontainers.image.${key}=`)
      || builds.run.includes(`--label org.opencontainers.image.${key}=`), `build omits ${key}`);
    assert.ok(rebuild.run.includes(`--label "org.opencontainers.image.${key}=`), `rebuild omits ${key}`);
  }
  for (const [key, expected] of [['revision', '$GITHUB_SHA'], ['source', 'https://github.com/$GITHUB_REPOSITORY'], ['version', '$VERSION'], ['created', '1970-01-01T00:00:00Z']])
    assert(rebuild.run.includes(`--label "org.opencontainers.image.${key}=${expected}"`));
  assert.match(rebuild.run, /for root in root-a root-b; do/);
  assert.match(rebuild.run, /builder="repro-.*-\$root"/);
  assert.match(rebuild.run, /docker buildx create --name "\$builder" --driver docker-container/);
  assert.match(rebuild.run, /--builder "\$builder" --no-cache --provenance=false --sbom=false/);
  assert(!/--cache-(?:from|to)|--keep-state/.test(rebuild.run));
  assert(rebuild.run.indexOf('\n  docker buildx rm "$builder"\n') > rebuild.run.indexOf('"${build[@]}"'));
  assert.match(rebuild.run, /release-phases\.mjs reproducibility/);
  assert.match(rebuild.run, /type=oci,tar=false,dest=\$root\.oci,rewrite-timestamp=true,oci-mediatypes=true/);
  assert(!rebuild.run.includes('.oci.tar'), 'reproducibility retains the complete layouts without duplicate outer archives');
  for (const row of value.jobs['reproducibility-parts'].strategy.matrix.include)
    assert(imageRows.some(image => image.name === row.image && image.dockerfile === row.dockerfile && image.target === row.target));
  assert.equal(value.jobs['reproducibility-parts'].strategy.matrix.include.length, 12);
  const publication = value.jobs['oci-native'].steps;
  assert(publication.some(step => step.run?.includes('release-phases.mjs prerequisites oci-native')));
  assert(publication.some(step => step.run?.includes('release-phases.mjs publish-oci')));
  assert(!publication.some(step => step.run?.includes('imagetools create')));
  assert(!publication.some(step => step.uses?.startsWith('softprops/action-gh-release')),
    'default overwrite-enabled release upload must not replace source-addressed bytes');
  for (const name of ['image-platforms', ...supplyNames, 'oci-native', 'crates', 'release']) {
    const steps = value.jobs[name].steps;
    const policyStep = steps.findIndex(step => step.run?.includes('release-phases.mjs policy'));
    assert.ok(policyStep >= 0, `${name} must check its own publication policy`);
    assert.equal(steps[policyStep].env.PUBLISH_CRATES, '${{ toJSON(inputs.publish-crates) }}');
    const firstCredential = steps.findIndex(step => step.uses?.startsWith('docker/login-action')
      || step.uses?.startsWith('rust-lang/crates-io-auth-action') || step.env?.GH_TOKEN);
    assert(firstCredential < 0 || policyStep < firstCredential);
  }
  for (const name of ['image-platforms', ...supplyNames, 'native', 'reproducibility-parts', 'browser-library', 'product-cli', 'installed-vv', 'installed-sdk', 'release']) {
    const builder = value.jobs[name].steps.find(step => step.uses?.startsWith('docker/setup-buildx-action'));
    assert.equal(builder.with.version, 'v0.28.0');
    assert.equal(builder.with['driver-opts'], `image=${buildkit}`);
  }
  for (const name of supplyNames) {
    const supply = value.jobs[name];
    assert.deepEqual(supply.needs, ['policy', 'image-platforms']);
    assert.equal(supply.if, undefined); assert.equal(supply['continue-on-error'], undefined);
    for (const step of supply.steps) {
      assert.equal(step.if, undefined); assert.equal(step['continue-on-error'], undefined);
    }
    const assembly = supply.steps.find(step => step.id === 'build');
    assert.match(assembly.run, /node scripts\/release-phases\.mjs image-index "\$\{platforms\[@\]\}"/);
    assert.match(assembly.run, /oras manifest push --media-type application\/vnd\.oci\.image\.index\.v1\+json \\\n +"\$REPOSITORY@\$digest"/);
    assert(!assembly.run.includes('--tag') && !assembly.run.includes(':latest'), 'the index is pushed by digest only');
    assert(!supply.steps.some(step => step.uses?.startsWith('actions/attest-sbom')), 'complete SPDX must not enter the inline predicate size limit');
    for (const architecture of ['amd64', 'arm64']) {
      const step = supply.steps.find(step => step.id === `sbom-${architecture}`);
      assert.equal(step.with['syft-version'], 'v1.51.1'); assert.equal(step.with['upload-artifact'], false);
    }
    const transport = supply.steps.find(step => step.run?.includes('release-phases.mjs sbom-publish'));
    assert(transport, 'both platform SBOMs must be signed and re-fetched');
    assert.match(transport.run, /for architecture in amd64 arm64/); assert.match(transport.run, /oras manifest fetch-config/);
    assert.match(transport.run, /release-phases\.mjs sbom-record/);
    assert.match(transport.run, /cmp .*sbom\.spdx\.json.*sbom\.received\.spdx\.json/);
    const indexVerification = supply.steps.find(step => step.run?.includes('docker pull "$REPOSITORY@$DIGEST"'));
    assert.match(indexVerification.run, /^set -euo pipefail\n/);
    assert.match(indexVerification.run, /release-phases\.mjs image-verify "\$REPOSITORY@\$DIGEST" "\$GITHUB_SHA" "\$GITHUB_REF"/);
    assert.match(indexVerification.run, /standards\/trust\/sigstore-trusted-root-cosign-3\.1\.3\.json/);
    assert(!indexVerification.run.includes('certificate-identity-regexp'));
    assert(!indexVerification.run.includes('||'), 'index signature failure must stop publication');
  }
}

test('publication phases preserve all gates and decouple OCI/native from optional Cargo', () => {
  validateWorkflow(workflow());
  for (const mutate of [
    value => { value.on.push.branches = ['main']; },
    value => { value.on.push.branches = ['release/v0.4']; },
    value => { value.on.push.paths = ['scripts/**']; },
    value => { value.on.workflow_call = {}; },
    value => { value.jobs.policy.if = "${{ inputs.resource-diagnostic-source == '' }}"; },
    value => { value.jobs['diagnostic-policy'].if = 'always()'; },
    value => { value.jobs['diagnostic-production'].needs = 'policy'; },
    ...['browser-library', 'oci-native'].flatMap(name => [
      value => { value.jobs[name].steps = value.jobs[name].steps.filter(step => !step.run?.includes('semantic-source-bootstrap.sh')); },
      value => { value.jobs[name].steps.find(step => step.run?.includes('semantic-source-bootstrap.sh')).env.CARGO_NET_OFFLINE = 'false'; },
      value => { value.jobs[name].steps.find(step => step.run?.includes('semantic-source-bootstrap.sh')).if = 'false'; },
      value => { const step = value.jobs[name].steps.find(step => step.run?.includes('semantic-source-bootstrap.sh'));
        value.jobs[name].steps.splice(value.jobs[name].steps.indexOf(step), 1); value.jobs[name].steps.push(step); },
    ]),
    ...['browser-library', 'oci-native'].flatMap(name => semanticReadinessMutations.map(rewrite => value => {
      const step = value.jobs[name].steps.find(candidate => candidate.run?.includes('semantic-source-bootstrap.sh'));
      const changed = rewrite(step.run);
      assert.notEqual(changed, step.run, 'readiness mutation must change the actual prerequisite');
      step.run = changed;
    })),
    value => { value.jobs['browser-library'].steps.find(step => step.run?.includes('binary-sdk-check.sh')).run = 'true'; },
    ...[
      run => run.replace('mkdir "$RUNNER_TEMP/binary-transport-1.1"', 'mkdir -p "$RUNNER_TEMP/binary-transport-1.1"'),
      run => run.replace('node root-a/scripts/binary-sdk-check.mjs tests\n', ''),
      run => run.replace('binary-sdk-release-evidence.mjs retain', 'binary-sdk-release-evidence.mjs omitted'),
      run => run.replace('"$GITHUB_RUN_ID" "$GITHUB_RUN_ATTEMPT"', '"$GITHUB_RUN_ID" 1'),
      run => run.replace('"$ARCH" 1.2', '"$ARCH" 1.1'),
    ].map(rewrite => value => {
      const step = value.jobs['browser-library'].steps.find(step => step.run?.includes('binary-sdk-check.sh'));
      step.run = rewrite(step.run);
    }),
    value => value.jobs['oci-native'].needs.push('crates'),
    value => value.jobs['oci-native'].needs.pop(),
    value => { delete value.jobs['installed-sdk']; },
    value => { value.jobs['installed-sdk'].if = '${{ false }}'; },
    value => { value.jobs['installed-sdk'].strategy.matrix.include.pop(); },
    value => { value.jobs['installed-sdk'].strategy.matrix.include[1].os = 'ubuntu-24.04'; },
    value => { value.jobs['installed-vv'].strategy.matrix.include[3].os = 'ubuntu-24.04'; },
    value => { value.jobs['installed-vv'].steps.find(step => step.run?.includes('sdk-vv-outer.sh')).if = '${{ false }}'; },
    value => { value.jobs['installed-sdk'].steps.find(step => step.run?.includes('assemble-installed')).if = '${{ false }}'; },
    value => { value.jobs['installed-vv'].strategy.matrix.include.pop(); },
    value => { value.jobs['oci-native'].steps.find(step => step.run?.includes('publish-oci')).if = undefined; },
    value => { value.jobs.release.if = undefined; },
    value => { value.on.workflow_dispatch.inputs.rehearsal.default = false; },
    value => { value.jobs['oci-native'].if = '${{ always() }}'; },
    value => { value.jobs.release.if = '${{ always() }}'; },
    value => value.jobs.release.needs.pop(),
    value => { value.jobs.crates.if = '${{ true }}'; },
    ...[
      run => run.replace('package_target=${archive%/package/*}', 'package_target=target'),
      run => run.replace('test "$archive" = "$package_target/package/$package-$version.crate"', 'true'),
      run => run.replace('release_cargo "$package" package --locked --manifest-path "$manifest" --target-dir "$package_target"',
        'release_cargo package --locked --manifest-path "$manifest"'),
      run => run.replace('cargo publish --locked --manifest-path "$manifest" --target-dir "$package_target"\n',
        'cargo publish --locked --manifest-path "$manifest"\n'),
      run => run.replace('cargo publish --locked --manifest-path "$manifest" --target-dir "$package_target"\n',
        'cargo publish --locked --manifest-path "$manifest" --target-dir "$package_target" --no-verify\n'),
      run => run.replace('cmp "$archive" "$expected"', 'true'),
      run => run.replace('cmp "$archive" "$expected"', 'true').replace(
        'cargo publish --locked --manifest-path "$manifest" --target-dir "$package_target"\n',
        'cargo publish --locked --manifest-path "$manifest" --target-dir "$package_target"\n                cmp "$archive" "$expected"\n'),
      // #214: the packaged crate must be dry-run, must stay the compared bytes
      // through the dry run, must meet crates.io metadata, and must not upload
      // without a credential.
      run => run.replace('release_cargo "$package" publish --dry-run --locked --manifest-path "$manifest" --target-dir "$package_target"\n', ''),
      run => run.replace('release_cargo "$package" publish --dry-run --locked --manifest-path "$manifest" --target-dir "$package_target"',
        'release_cargo "$package" publish --dry-run --locked --manifest-path "$manifest" --target-dir "$package_target" --no-verify'),
      run => run.replace('test "$(sha256sum "$archive" | cut -d" " -f1)" = "$local_checksum"\n', ''),
      run => run.replace('./scripts/crate-metadata-check.sh "$archive"\n', ''),
      run => run.replace('if [ "$UPLOAD" != true ]; then', 'if false; then'),
      run => run.replace(/if \[ "\$UPLOAD" = true \]; then\s+wait_for_crate lexlean 0\.4\.0\s+fi/, 'wait_for_crate lexlean 0.4.0'),
      run => run.replace(/\s*--config "patch\.crates-io\.lexlean\.path=[^\n]*\n/, '\n'),
    ].map(mutate => value => {
      const step = value.jobs.crates.steps.find(step => step.run?.includes('publish_exact()'));
      step.run = mutate(step.run);
    }),
    value => { delete value.jobs.crates.env.HAS_CARGO_REGISTRY_TOKEN; },
    value => { delete value.jobs.crates.steps.find(step => step.uses?.startsWith('rust-lang/crates-io-auth-action')).if; },
    value => { value.jobs.crates.steps.find(step => step.run?.includes('publish_exact()')).env.UPLOAD = 'true'; },
    value => { value.jobs.crates.steps.find(step => step.run?.includes('publish_exact()')).env.CARGO_REGISTRY_TOKEN
      = '${{ steps.auth.outputs.token }}'; },
    value => { value.jobs['source-vv'].steps.find(step => step.with?.runCmd).with.runCmd = 'just vv'; },
    value => { value.jobs['reproducibility-parts'].steps.find(step => step.env?.DOCKERFILE).run = 'cmp root-a.digest root-b.digest'; },
    value => { value.jobs['sdk-supply'].steps = value.jobs['sdk-supply'].steps.filter(step => !step.run?.includes('release-phases.mjs policy')); },
    ...[
      run => run + '\n--tag mutable:unaccepted\n',
      run => run.replace('push-by-digest=true', 'push-by-digest=false'),
      run => run.replace('node scripts/sdk-image-inputs.mjs build-set . "$GITHUB_SHA" "$plan"', 'docker buildx build'),
      run => run.replace('--platform "linux/$ARCH"', '--platform linux/amd64'),
      run => run.replace('--provenance=false', '--provenance=true'),
      run => run.replace('oracles oracles\n', ''),
    ].map(mutate => value => { const step = value.jobs['image-platforms'].steps.find(step => step.id === 'build'); step.run = mutate(step.run); }),
    value => { value.jobs['image-platforms'].strategy.matrix.include.pop(); },
    value => { value.jobs['image-platforms'].steps.splice(1, 0, {uses: 'docker/setup-qemu-action@fixture'}); },
    value => { const step = value.jobs['sdk-supply'].steps.find(step => step.id === 'build');
      step.run = step.run.replace('"$REPOSITORY@$digest" "$RUNNER_TEMP/image-index.json"', '"$REPOSITORY:latest" "$RUNNER_TEMP/image-index.json"'); },
    value => { value.jobs['sdk-supply'].steps.push({uses: 'actions/attest-sbom@fixture'}); },
    value => { value.jobs['sdk-supply'].steps.find(step => step.id === 'sbom-amd64').with['syft-version'] = 'latest'; },
    value => { const step = value.jobs['sdk-supply'].steps.find(step => step.run?.includes('release-phases.mjs image-verify'));
      step.run = step.run.replace('"$GITHUB_SHA" "$GITHUB_REF"', '"other-sha" "$GITHUB_REF"'); },
    value => { const step = value.jobs['sdk-supply'].steps.find(step => step.run?.includes('release-phases.mjs image-verify'));
      step.run = step.run.replace('"$GITHUB_SHA" "$GITHUB_REF"', '"$GITHUB_SHA" "refs/heads/release/v0.4"'); },
    value => { const step = value.jobs['sdk-supply'].steps.find(step => step.run?.includes('release-phases.mjs image-verify'));
      step.run = step.run.replace('set -euo pipefail\n', ''); },
    value => { const step = value.jobs['sdk-supply'].steps.find(step => step.run?.includes('release-phases.mjs image-verify'));
      step.run += '\ntrue || true\n'; },
    ...['images', 'reproducibility', 'installed-sdk'].flatMap(name => [
      value => value.jobs[name].needs.pop(),
      value => value.jobs[name].if = '${{ always() }}',
      value => value.jobs[name]['continue-on-error'] = true,
    ]),
    ...['product-cli', 'browser-library'].flatMap(name => [
      value => value.jobs[name].strategy.matrix.include.pop(),
      value => value.jobs[name].needs.push('gate'),
      value => value.jobs[name].steps.find(step => step.run?.includes('-sdk-check.sh')).if = '${{ false }}',
      value => value.jobs[name].steps.find(step => step.run?.includes('-sdk-check.sh')).run += ' || true\n',
    ]),
    ...[
      run => run.replace('--no-cache', ''),
      run => run.replace('matrix.name }}-$root', 'matrix.name }}-shared'),
      run => run.replace('\n  docker buildx rm "$builder"\n', '\n'),
      run => run.replace('tar=false', 'tar=true'),
    ].map(mutate => value => {
      const step = value.jobs['reproducibility-parts'].steps.find(step => step.env?.DOCKERFILE);
      step.run = mutate(step.run);
    }),
    ...[
      value => { value.jobs['product-images'].steps = value.jobs['product-images'].steps.filter(step => !step.run?.includes('release-phases.mjs policy')); },
      value => { value.jobs['product-images'].steps.find(step => step.id === 'sbom-arm64').with['syft-version'] = 'latest'; },
      value => { const step = value.jobs['product-images'].steps.find(step => step.run?.includes('release-phases.mjs image-verify'));
        step.run = step.run.replace('"$GITHUB_SHA" "$GITHUB_REF"', '"other-sha" "$GITHUB_REF"'); },
      value => { value.jobs['product-images'].steps.find(step => step.run?.includes('release-phases.mjs image-verify'))['continue-on-error'] = true; },
    ],
  ]) {
    const changed = workflow(); mutate(changed);
    assert.notDeepEqual(changed, workflow(), 'mutation must change the actual workflow: ' + mutate.toString());
    assert.throws(() => validateWorkflow(changed), undefined, mutate.toString());
  }
});

test('release crates use stage-owned targets despite an inherited Cargo target directory', () => {
  // Exercise real Cargo packaging and its verification builds, not a command
  // stub. The independent package-api gate also checks these archive bytes.
  const directory = mkdtempSync(join(tmpdir(), 'prismpm-package-target-'));
  try {
    const inherited = join(directory, 'caller target');
    const stage = join(directory, 'release crates');
    mkdirSync(inherited);
    writeFileSync(join(inherited, 'caller-owned'), 'preserve caller artifacts\n');
    execFileSync('sh', [new URL('./package-release-crates.sh', import.meta.url).pathname,
      '--prepare', stage], {encoding: 'utf8', timeout: 180_000, stdio: 'pipe',
      env: {...process.env, CARGO_TARGET_DIR: inherited}});
    const archives = [
      ['prod-ir-0.1.0.crate', 'lean4-prod', 'vendor/lean4-prod/crates'],
      ['prod-codegen-0.1.0.crate', 'lean4-prod', 'vendor/lean4-prod/crates'],
      ['prism-stdlib-0.2.0.crate', 'stdlib', 'stdlib/generated'],
    ];
    assert.deepEqual(readdirSync(join(stage, 'packages')).sort(), archives.map(([name]) => name).sort());
    for (const [name, owner, reviewed] of archives) {
      const bytes = readFileSync(join(stage, 'packages', name));
      const produced = readFileSync(join(stage, owner, 'target/package', name));
      const expected = readFileSync(new URL(`../${reviewed}/${name}`, import.meta.url));
      assert.ok(bytes.equals(produced), `${name}: staged archive differs from Cargo output (${sha(bytes)} != ${sha(produced)})`);
      assert.ok(bytes.equals(expected), `${name}: archive differs from reviewed bytes (${sha(bytes)} != ${sha(expected)})`);
    }
    assert.deepEqual(readdirSync(inherited), ['caller-owned']);
    assert.equal(readFileSync(join(inherited, 'caller-owned'), 'utf8'), 'preserve caller artifacts\n');
  } finally { rmSync(directory, {recursive: true, force: true}); }
});

test('Cargo publication target selection binds the exact package and version archive', () => {
  const body = workflow().jobs.crates.steps.find(step => step.run?.includes('publish_exact()')).run;
  const selection = body.slice(body.indexOf('package_target='), body.indexOf('release_cargo "$package" package')).trim();
  const select = (packageName, version, archive) => execFileSync('sh', ['-eu', '-c',
    `${selection}\nprintf '%s\\n' "$package_target"`], {encoding: 'utf8', stdio: 'pipe',
    env: {...process.env, package: packageName, version, archive}});
  for (const [name, version, target] of [
    ['prod-ir', '0.1.0', '/release sources/lean4-prod/target'],
    ['prod-codegen', '0.1.0', '/release sources/lean4-prod/target'],
    ['prism-stdlib', '0.2.0', '/release sources/stdlib/target'],
    ['prismpm', '0.4.0', '/workspace/target'],
  ]) {
    assert.equal(select(name, version, `${target}/package/${name}-${version}.crate`), `${target}\n`);
    for (const archive of [`${target}/${name}-${version}.crate`,
      `${target}/package/other-${version}.crate`, `${target}/package/${name}-99.0.0.crate`]) {
      assert.throws(() => select(name, version, archive), error => error.status !== 0);
    }
  }
});

test('explicit OCI-only dispatch is distinct from Cargo publication and rejects foreign refs', () => {
  assert.deepEqual(publicationPolicy(policy()), {version: '0.4.0', publishCrates: false, rehearsal: false});
  assert.equal(publicationPolicy({...policy(), rehearsal: true}).rehearsal, true);
  assert.throws(() => publicationPolicy({...policy(), rehearsal: true, publishCrates: true}), /rehearsal cannot publish/);
  assert.throws(() => publicationPolicy({...policy(), rehearsal: 'true'}));
  assert.throws(() => publicationPolicy({...policy(), rehearsal: null}));
  assert.throws(() => publicationPolicy({...policy(), event: 'push', ref: 'refs/tags/v0.4.0', publishCrates: null, rehearsal: true}));
  assert.equal(publicationPolicy({...policy(), publishCrates: true}).publishCrates, true);
  assert.equal(publicationPolicy({...policy(), event: 'push', ref: 'refs/tags/v0.4.0', publishCrates: null, rehearsal: null}).publishCrates, true);
  for (const change of [{publishCrates: 'false'}, {publishCrates: null}, {repository: 'other/PrismPM'},
    {ref: 'refs/heads/main'}, {ref: 'refs/heads/feature'}, {ref: 'refs/heads/release/v0.5'}, {ref: 'refs/tags/v0.3.0'}, {event: 'pull_request'},
    {version: '0.5.0'}, {revision: 'main'}, {extra: true}]) {
    assert.throws(() => publicationPolicy({...policy(), ...change}));
  }
});

test('release branch pushes force rehearsal and preserve tag publication policy', () => {
  const pushed = {...policy(), event: 'push', publishCrates: null, rehearsal: null};
  assert.deepEqual(publicationPolicy(pushed), {version: '0.4.0', publishCrates: false, rehearsal: true});
  for (const ref of ['refs/heads/main', 'refs/heads/feature', 'refs/heads/release/v0.5', 'refs/tags/v0.3.0']) {
    assert.throws(() => publicationPolicy({...pushed, ref}));
  }
  for (const change of [{publishCrates: true}, {rehearsal: false}, {version: '0.5.0'}]) {
    assert.throws(() => publicationPolicy({...pushed, ...change}));
  }
  const helper = new URL('./release-phases.mjs', import.meta.url).pathname;
  const output = execFileSync(process.execPath, [helper, 'policy'], {encoding: 'utf8',
    env: {...process.env, ...environment(), GITHUB_EVENT_NAME: 'push', GITHUB_REF_NAME: 'release/v0.4',
      DISPATCH_VERSION: '9.9.9', PUBLISH_CRATES: 'true', REHEARSAL: 'false'}});
  assert.equal(output, 'version=0.4.0\npublish-crates=false\nrehearsal=true\n');
});

test('image-index signatures require the exact current source, ref, trigger, name, issuer and repository', () => {
  const imageDigest = `sha256:${'b'.repeat(64)}`;
  for (const name of ['sdk', 'runtime', 'adapter-compose', 'adapter-kubernetes', 'adapter-github-pages', 'oracles']) {
    for (const [ref, event] of [['refs/heads/release/v0.4', 'workflow_dispatch'],
      ['refs/heads/release/v0.4', 'push'], ['refs/tags/v0.4.0', 'push']]) {
      const image = `ghcr.io/uor-foundation/prismpm-${name}@${imageDigest}`;
      assert.deepEqual(imageVerificationArguments(image, revision, ref, '/trusted-root', event), [
        'verify', '--trusted-root', '/trusted-root', '--certificate-identity',
        `https://github.com/UOR-Foundation/PrismPM/.github/workflows/release.yml@${ref}`,
        '--certificate-oidc-issuer', 'https://token.actions.githubusercontent.com',
        '--certificate-github-workflow-repository', 'UOR-Foundation/PrismPM',
        '--certificate-github-workflow-ref', ref, '--certificate-github-workflow-sha', revision,
        '--certificate-github-workflow-trigger', event,
        '--certificate-github-workflow-name', 'PrismPM OCI/native and Cargo publication', image,
      ]);
    }
  }
  const image = `ghcr.io/uor-foundation/prismpm-sdk@${imageDigest}`;
  for (const [ref, event] of [['refs/heads/release/v0.4', 'pull_request'],
    ['refs/heads/release/v0.4', 'push|workflow_dispatch'], ['refs/tags/v0.4.0', 'workflow_dispatch']]) {
    assert.throws(() => imageVerificationArguments(image, revision, ref, '/trusted-root', event));
  }
  for (const [subject, source, ref] of [
    [image.replace('uor-foundation', 'other'), revision, 'refs/heads/release/v0.4'],
    [image.replace('prismpm-sdk', 'prismpm-sdk-candidate'), revision, 'refs/heads/release/v0.4'],
    [image.replace(imageDigest, 'latest'), revision, 'refs/heads/release/v0.4'],
    [`${image}@${imageDigest}`, revision, 'refs/heads/release/v0.4'],
    [image, 'main', 'refs/heads/release/v0.4'], [image, revision, 'refs/heads/feature'], [image, revision, 'refs/heads/main'],
    [image, revision, 'refs/tags/v0.3.0'],
  ]) assert.throws(() => imageVerificationArguments(subject, source, ref, '/trusted-root'));
});

test('image-index verifier CLI checks the pinned root and propagates signature-verifier failures', () => {
  // This command fixture proves invocation/failure propagation only. The
  // separate pinned Sigstore suite exercises real signed upstream fixtures.
  const directory = mkdtempSync(join(tmpdir(), 'prismpm-index-verifier-'));
  try {
    const helper = new URL('./release-phases.mjs', import.meta.url).pathname;
    const trust = new URL('../standards/trust/sigstore-trusted-root-cosign-3.1.3.json', import.meta.url).pathname;
    const image = `ghcr.io/uor-foundation/prismpm-sdk@sha256:${'b'.repeat(64)}`;
    const args = imageVerificationArguments(image, revision, 'refs/heads/release/v0.4', trust);
    const executable = join(directory, 'cosign');
    writeFileSync(executable, `#!${process.execPath}\nrequire('node:assert/strict').deepEqual(process.argv.slice(2), JSON.parse(process.env.EXPECTED_ARGS));\nprocess.stdout.write('verified-command-fixture\\n');\nprocess.exit(Number(process.env.VERIFIER_STATUS));\n`);
    chmodSync(executable, 0o700);
    const invoke = (status, root = trust, sourceRevision = revision, event = 'workflow_dispatch', expected = args) => execFileSync(process.execPath,
      [helper, 'image-verify', image, sourceRevision, 'refs/heads/release/v0.4', root], {stdio: 'pipe', encoding: 'utf8',
        env: {...process.env, PATH: `${directory}:${process.env.PATH}`, EXPECTED_ARGS: JSON.stringify(expected), GITHUB_EVENT_NAME: event,
          VERIFIER_STATUS: String(status)}});
    assert.equal(invoke(0), 'verified-command-fixture\n');
    const pushArgs = imageVerificationArguments(image, revision, 'refs/heads/release/v0.4', trust, 'push');
    assert.equal(invoke(0, trust, revision, 'push', pushArgs), 'verified-command-fixture\n');
    assert.throws(() => invoke(0, trust, revision, 'push'), error => error.status !== 0);
    assert.throws(() => invoke(0, trust, revision, 'pull_request'), error => error.status !== 0 && error.stdout === '');
    assert.throws(() => invoke(1), error => error.status !== 0);
    assert.throws(() => invoke(0, trust, 'c'.repeat(40)), error => error.status !== 0);
    const changedRoot = join(directory, 'untrusted.json'); writeFileSync(changedRoot, '{}');
    assert.throws(() => invoke(0, changedRoot), error => error.status !== 0 && error.stdout === '');
    assert.throws(() => invoke(0, join(directory, 'absent.json')), error => error.status !== 0 && error.stdout === '');
  } finally { rmSync(directory, {recursive: true, force: true}); }
});

test('the release part shell must fail when its one invocation fails', t => {
  // This checks the actual shell sequence's failure propagation. The helper's
  // fixed just-vv-part command and real process capture have separate owning tests.
  const bodies = [workflow().jobs['source-vv'].steps.find(step => step.with?.runCmd).with.runCmd.replaceAll('${{ matrix.part }}', 'source')];
  const verify = body => {
    for (const first of [0, 1]) {
      const root = mkdtempSync(join(tmpdir(), 'prismpm-release-session-'));
      t.after(() => rmSync(root, {recursive: true, force: true}));
      const env = sessionFixture(root, process.env, {first, release: true,
        image: `ghcr.io/uor-foundation/prismpm-sdk@sha256:${'b'.repeat(64)}`});
      const image = env.PRISMPM_TEST_SDK_IMAGE;
      delete env.PRISMPM_TEST_SDK_IMAGE;
      // Exercise the actual workflow and handoff shell. Only tool installation
      // and signature acquisition are transport fixtures here; their real
      // verification is owned by source-sdk-handoff's dedicated tests.
      mkdirSync(join(root, 'target/release-sdk'), {recursive: true});
      writeFileSync(join(root, 'target/release-sdk/sdk-image.txt'), image + '\n');
      writeFileSync(join(root, 'scripts/sdk-candidate.sh'), 'set -eu\ncase "$1" in install-oras|install-cosign) ;; *) exit 64 ;; esac\ntest "$#" -eq 2\nmkdir -p "$2"\n');
      writeFileSync(join(root, 'scripts/source-sdk-handoff.sh'), readFileSync(new URL('./source-sdk-handoff.sh', import.meta.url)));
      writeFileSync(join(root, 'scripts/source-sdk-handoff.mjs'), `
        import assert from 'node:assert/strict';
        assert.deepEqual(process.argv.slice(2), ${JSON.stringify([image,
          '${{ github.sha }}', 'amd64', '${{ github.ref }}', '${{ github.event_name }}'])});
      `);
      if (first === 0) {
        assert.equal(execFileSync('bash', ['-c', body], {cwd: root, env, encoding: 'utf8', timeout: 15_000}), 'call:1\n');
      } else {
        assert.throws(() => execFileSync('bash', ['-c', body], {cwd: root, env, encoding: 'utf8', timeout: 15_000}), error => {
          assert.equal(error.status, 1);
          assert.equal(error.stdout, 'call:1\n');
          return true;
        });
      }
      const rows = readFileSync(join(root, 'session.jsonl'), 'utf8').trim().split('\n').map(JSON.parse);
      assert(!rows.some(row => ['build', 'docker'].includes(row.kind)), 'verified SDK handoff must prevent source reconstruction');
      const calls = rows.filter(row => row.kind === 'source-run'); assert.equal(calls.length, 1);
      assert.equal(calls[0].image, image); assert.equal(calls[0].args[6], image);
    }
  };
  for (const body of bodies) {
    verify(body);
    assert.throws(() => verify(body.replace("'source'\n", "'source' || true\n")));
  }
});

test('publication rejects missing, skipped, failed, cancelled or extraneous prerequisites', () => {
  requirePrerequisites('oci-native', prerequisites(ociNeeds));
  const releaseNeeds = [...ociNeeds, 'oci-native', 'crates'];
  requirePrerequisites('release', prerequisites(releaseNeeds));
  for (const phase of ['oci-native', 'release']) {
    const names = phase === 'release' ? releaseNeeds : ociNeeds;
    for (const name of names) {
      const missing = prerequisites(names); delete missing[name];
      assert.throws(() => requirePrerequisites(phase, missing));
      for (const result of ['failure', 'cancelled', 'skipped', 'neutral', '', undefined]) {
        assert.throws(() => requirePrerequisites(phase, {...prerequisites(names), [name]: {result}}));
      }
    }
    assert.throws(() => requirePrerequisites(phase, {...prerequisites(names), unrelated: {result: 'success'}}));
  }
  assert.throws(() => requirePrerequisites('unknown', {}));
});

function layout(directory, architecture, marker = 'same') {
  mkdirSync(join(directory, 'blobs/sha256'), {recursive: true});
  const putBytes = bytes => {
    const digest = sha(bytes);
    writeFileSync(join(directory, 'blobs/sha256', digest.slice(7)), bytes);
    return {mediaType: 'application/vnd.oci.image.manifest.v1+json', digest, size: bytes.length};
  };
  const put = value => putBytes(Buffer.from(JSON.stringify(value)));
  const config = {...put({architecture, os: 'linux', config: {Labels: {
    'org.opencontainers.image.created': '1970-01-01T00:00:00Z',
    'org.opencontainers.image.revision': revision,
    'org.opencontainers.image.source': 'https://github.com/UOR-Foundation/PrismPM',
    'org.opencontainers.image.version': '0.4.0', marker}}}), mediaType: 'application/vnd.oci.image.config.v1+json'};
  const layers = [Buffer.alloc(200_003, 97), Buffer.from('second layer\n')].map(bytes =>
    ({...putBytes(bytes), mediaType: 'application/vnd.oci.image.layer.v1.tar'}));
  const manifest = put({schemaVersion: 2, mediaType: 'application/vnd.oci.image.manifest.v1+json', config, layers});
  writeFileSync(join(directory, 'index.json'), JSON.stringify({schemaVersion: 2, manifests: [manifest]}));
  return {...manifest, platform: {os: 'linux', architecture}};
}

test('two byte-equal rebuilds must also equal the shipped platform and source-labelled config', () => {
  const directory = mkdtempSync(join(tmpdir(), 'prismpm-release-repro-'));
  try {
    const a = join(directory, 'a'), b = join(directory, 'b');
    const first = layout(a, 'amd64'); layout(b, 'amd64');
    const arm = {...first, digest: `sha256:${'b'.repeat(64)}`, platform: {os: 'linux', architecture: 'arm64'}};
    const bytes = Buffer.from(JSON.stringify({schemaVersion: 2, mediaType: 'application/vnd.oci.image.index.v1+json', manifests: [first, arm]}));
    const image = `ghcr.io/uor-foundation/prismpm-sdk@${sha(bytes)}`;
    const verify = (reference = image, index = bytes, architecture = 'amd64', sourceRevision = revision) =>
      verifyReproducibility(reference, index, architecture, sourceRevision, a, b);
    assert.equal(verify(), first.digest);
    assert.throws(() => verify(`${image.slice(0, -1)}0`));
    assert.throws(() => verify(image, Buffer.concat([bytes, Buffer.from('\n')])));
    assert.throws(() => verify(image, bytes, 'arm64'));
    assert.throws(() => verify(image, bytes, 'amd64', 'c'.repeat(40)));
    layout(b, 'amd64', 'different'); assert.throws(() => verify());
    layout(a, 'amd64', 'different'); assert.throws(() => verify(), 'equal rebuilds cannot substitute unshipped bytes');
    layout(a, 'amd64'); layout(b, 'amd64');
    writeFileSync(join(a, 'blobs/sha256', first.digest.slice(7)), '{}');
    assert.throws(() => verify());
  } finally { rmSync(directory, {recursive: true, force: true}); }
});

test('reproducibility freshly verifies every layer and refuses damaged or aliased bytes in either layout', () => {
  const directory = mkdtempSync(join(tmpdir(), 'prismpm-release-layers-'));
  try {
    for (const side of ['a', 'b']) {
      for (const damage of ['missing', 'changed', 'truncated', 'grown', 'symlink', 'hardlink', 'fifo', 'parent-alias']) {
        const root = join(directory, `${side}-${damage}`), a = join(root, 'a'), b = join(root, 'b');
        const first = layout(a, 'amd64'); layout(b, 'amd64');
        const arm = {...first, platform: {os: 'linux', architecture: 'arm64'}};
        const index = Buffer.from(JSON.stringify({schemaVersion: 2,
          mediaType: 'application/vnd.oci.image.index.v1+json', manifests: [first, arm]}));
        const verify = () => verifyReproducibility(`ghcr.io/uor-foundation/prismpm-sdk@${sha(index)}`,
          index, 'amd64', revision, a, b);
        assert.equal(verify(), first.digest);
        const selected = join(root, side), manifest = JSON.parse(readFileSync(join(selected, 'blobs/sha256', first.digest.slice(7))));
        // Damage the last layer as well as the multi-chunk first layer: neither
        // the first descriptor nor a previous successful read is acceptance.
        const layer = manifest.layers[damage === 'truncated' ? 0 : 1];
        const path = join(selected, 'blobs/sha256', layer.digest.slice(7)), original = readFileSync(path);
        if (damage === 'missing') rmSync(path);
        else if (damage === 'changed') writeFileSync(path, Buffer.alloc(original.length, 98));
        else if (damage === 'truncated') writeFileSync(path, original.subarray(0, -1));
        else if (damage === 'grown') writeFileSync(path, Buffer.concat([original, Buffer.from('x')]));
        else if (damage === 'parent-alias') {
          const actual = join(selected, 'actual-blobs'); renameSync(join(selected, 'blobs'), actual);
          symlinkSync(actual, join(selected, 'blobs'));
        } else {
          rmSync(path);
          if (damage === 'fifo') execFileSync('mkfifo', [path]);
          else {
            const actual = join(root, 'external-layer'); writeFileSync(actual, original);
            if (damage === 'symlink') symlinkSync(actual, path);
            else linkSync(actual, path);
          }
        }
        assert.throws(verify, undefined, `${side}: ${damage} layer must refuse`);
      }
    }
  } finally { rmSync(directory, {recursive: true, force: true}); }
});

test('reproducibility rejects malformed or over-bound layer inventories', () => {
  const directory = mkdtempSync(join(tmpdir(), 'prismpm-release-layer-descriptors-'));
  try {
    const mutations = [
      manifest => { delete manifest.layers; },
      manifest => { manifest.layers = {}; },
      manifest => { manifest.layers = Array(4097).fill(manifest.layers[0]); },
      manifest => { manifest.layers = Array.from({length: 5}, () => ({...manifest.layers[0], size: 32 * 1024 ** 3})); },
      ...[{digest: '../../escape'}, {digest: `sha256:${'B'.repeat(64)}`}, {size: -1}, {size: 1.5},
        {size: '14'}, {size: 32 * 1024 ** 3 + 1}, {mediaType: 'application/json'}]
        .map(change => manifest => { Object.assign(manifest.layers[0], change); }),
    ];
    for (const [number, mutate] of mutations.entries()) {
      const root = join(directory, String(number)), child = layout(root, 'amd64');
      const manifest = JSON.parse(readFileSync(join(root, 'blobs/sha256', child.digest.slice(7))));
      mutate(manifest);
      const bytes = Buffer.from(JSON.stringify(manifest)), descriptor = {...child, digest: sha(bytes), size: bytes.length};
      writeFileSync(join(root, 'blobs/sha256', descriptor.digest.slice(7)), bytes);
      writeFileSync(join(root, 'index.json'), JSON.stringify({schemaVersion: 2, manifests: [descriptor]}));
      const index = Buffer.from(JSON.stringify({schemaVersion: 2, mediaType: 'application/vnd.oci.image.index.v1+json',
        manifests: [descriptor, {...descriptor, platform: {os: 'linux', architecture: 'arm64'}}]}));
      assert.throws(() => verifyReproducibility(`ghcr.io/uor-foundation/prismpm-sdk@${sha(index)}`,
        index, 'amd64', revision, root, root), number === 3 ? /OCI layer closure exceeds its byte bound/ : undefined,
      `descriptor mutation ${number} must refuse`);
    }
  } finally { rmSync(directory, {recursive: true, force: true}); }
});

test('both layout owners retain custody until the complete comparison finishes; removing the final sweep is killed', async () => {
  const helper = new URL('./release-phases.mjs', import.meta.url), source = readFileSync(helper, 'utf8');
  const withoutSweep = source.replace('  for (const owner of owners) owner();', '  // planted omitted final two-layout sweep');
  assert.notEqual(source, withoutSweep);
  const absoluteImports = withoutSweep.replace(/from '(\.\/[^']+)'/g, (_, path) => `from '${new URL(path, helper).href}'`);
  const mutant = await import(`data:text/javascript;base64,${Buffer.from(absoluteImports).toString('base64')}`);
  const check = (implementation, mutation) => {
    const directory = mkdtempSync(join(tmpdir(), 'prismpm-release-during-read-'));
    const originalRead = fs.readSync;
    let changed = false;
    try {
      const a = join(directory, 'a'), b = join(directory, 'b'), first = layout(a, 'amd64'); layout(b, 'amd64');
      const manifest = JSON.parse(readFileSync(join(a, 'blobs/sha256', first.digest.slice(7))));
      const layer = manifest.layers[0].digest.slice(7), victim = join(a, 'blobs/sha256', layer), trigger = join(b, 'blobs/sha256', layer);
      const index = Buffer.from(JSON.stringify({schemaVersion: 2, mediaType: 'application/vnd.oci.image.index.v1+json',
        manifests: [first, {...first, platform: {os: 'linux', architecture: 'arm64'}}]}));
      fs.readSync = (fd, ...args) => {
        const count = originalRead(fd, ...args);
        if (!changed && fs.readlinkSync(`/proc/self/fd/${fd}`) === trigger) {
          changed = true;
          if (mutation === 'inode') {
            const bytes = readFileSync(victim); renameSync(victim, victim + '.retained'); writeFileSync(victim, bytes);
          } else writeFileSync(victim, Buffer.alloc(manifest.layers[0].size, 98));
        }
        return count;
      };
      syncBuiltinESMExports();
      assert.throws(() => implementation(`ghcr.io/uor-foundation/prismpm-sdk@${sha(index)}`,
        index, 'amd64', revision, a, b), /OCI input changed during read/);
      assert.equal(changed, true, 'actual second-layout layer read must trigger the substitution');
    } finally {
      fs.readSync = originalRead; syncBuiltinESMExports();
      rmSync(directory, {recursive: true, force: true});
    }
  };
  for (const mutation of ['content', 'inode']) {
    check(verifyReproducibility, mutation);
    assert.throws(() => check(mutant.verifyReproducibility, mutation), /Missing expected exception/);
  }
});

test('existing source-addressed publication is reusable only with exact metadata and every byte', () => {
  const tag = `sdk-oci-${revision}`, notes = 'Explicitly unaccepted OCI/native publication.\n';
  const files = [{name: 'archive.tar.gz', size: 3, digest: sha(Buffer.from('abc'))}];
  const release = {tag_name: tag, target_commitish: revision, draft: false, prerelease: true,
    body: notes, assets: [{name: files[0].name, size: files[0].size}]};
  const check = (value = release, actual = files, commit = revision) =>
    verifyExistingRelease(value, {tag, revision, notes, files}, actual, commit);
  check();
  for (const change of [{tag_name: 'v0.4.0'}, {target_commitish: 'main'}, {draft: true},
    {prerelease: false}, {body: 'accepted'}, {assets: []},
    {assets: [...release.assets, ...release.assets]}, {assets: [{...release.assets[0], size: 4}]}]) {
    assert.throws(() => check({...release, ...change}));
  }
  assert.throws(() => check(release, [{...files[0], digest: sha(Buffer.from('abd'))}]));
  assert.throws(() => check(release, [], revision));
  assert.throws(() => check(release, files, 'd'.repeat(40)));
});

test('release SBOM input binds exact index, child, config, platform, source and unmodified SPDX', () => {
  const directory = mkdtempSync(join(tmpdir(), 'prismpm-release-sbom-'));
  try {
    const child = layout(directory, 'amd64');
    const childBytes = readFileSync(join(directory, 'blobs/sha256', child.digest.slice(7)));
    const configBytes = readFileSync(join(directory, 'blobs/sha256', JSON.parse(childBytes).config.digest.slice(7)));
    const arm = {...child, digest: `sha256:${'b'.repeat(64)}`, platform: {os: 'linux', architecture: 'arm64'}};
    const index = Buffer.from(JSON.stringify({schemaVersion: 2, mediaType: 'application/vnd.oci.image.index.v1+json', manifests: [child, arm]}));
    const spdx = Buffer.from(JSON.stringify({spdxVersion: 'SPDX-2.3', SPDXID: 'SPDXRef-DOCUMENT', dataLicense: 'CC0-1.0',
      documentNamespace: 'https://example.test/release-sbom-fixture', packages: [{}], files: [{}], relationships: [{}],
      hasExtractedLicensingInfos: [{licenseId: 'LicenseRef-Fixture', extractedText: 'full unmodified text'}]}));
    const image = `ghcr.io/uor-foundation/prismpm-sdk@${sha(index)}`;
    const args = [image, index, childBytes, configBytes, spdx, 'amd64', revision];
    const record = releaseSbomRecord(...args);
    assert.equal(record.image_reference, image);
    assert.equal(record.image_config_digest, sha(configBytes));
    assert.equal(record.spdx.digest, sha(spdx));
    assert.equal(record.spdx.size, spdx.length);
    assert.equal(record.production_accepted, false);
    assert.equal(record.development_only, undefined, 'release transport must not invent candidate smoke results');
    assert.equal(record.inventory_digest, undefined, 'runtime image must not be misreported as an SDK inventory');
    for (const [index, value] of [[0, image.replace('uor-foundation', 'other')], [1, Buffer.from('{}')],
      [2, Buffer.from('{}')], [3, Buffer.from('{}')], [4, spdx.subarray(0, -1)],
      [5, 'arm64'], [6, 'b'.repeat(40)]]) {
      const bad = [...args]; bad[index] = value;
      assert.throws(() => releaseSbomRecord(...bad));
    }
    for (const [name, bytes] of [['index.json', index], ['manifest.json', childBytes], ['config.json', configBytes],
      ['sbom.spdx.json', spdx]]) writeFileSync(join(directory, name), bytes);
    const helper = new URL('./release-phases.mjs', import.meta.url).pathname;
    const invoke = (command, ...extra) => execFileSync(process.execPath,
      [helper, command, directory, 'amd64', revision, image, 'refs/heads/release/v0.4', ...extra],
      {stdio: 'pipe', env: {...process.env, GITHUB_EVENT_NAME: 'workflow_dispatch'}});
    invoke('sbom-record');
    assert.deepEqual(JSON.parse(readFileSync(join(directory, 'sbom-record.json'))), record);
    const trust = new URL('../standards/trust/sigstore-trusted-root-cosign-3.1.3.json', import.meta.url).pathname;
    const executable = join(directory, 'cosign');
    writeFileSync(executable, `#!${process.execPath}\nconst args=process.argv.slice(2);\nrequire('node:assert/strict').equal(args[args.indexOf('--certificate-github-workflow-trigger')+1],process.env.GITHUB_EVENT_NAME);\nprocess.stderr.write('exact-event-verified\\n');\nprocess.exit(19);\n`);
    chmodSync(executable, 0o700);
    for (const event of ['workflow_dispatch', 'push', 'pull_request']) {
      // Stop at the verifier: no registry interaction or fabricated signature acceptance.
      assert.throws(() => execFileSync(process.execPath,
        [helper, 'sbom-verify', directory, 'amd64', revision, image, 'refs/heads/release/v0.4', trust],
        {stdio: 'pipe', env: {...process.env, GITHUB_EVENT_NAME: event, PATH: `${directory}:${process.env.PATH}`}}),
      error => event === 'pull_request' ? error.stderr.toString().includes('exact release ref/event pair required')
        : error.stderr.toString().includes('exact-event-verified'));
    }

    writeFileSync(join(directory, 'sbom-record.json'), JSON.stringify({...record, production_accepted: true}));
    assert.throws(() => invoke('sbom-verify', '/absent-trusted-root'));
    invoke('sbom-record');
    writeFileSync(join(directory, 'config.json'), '{}');
    assert.throws(() => invoke('sbom-record'));
  } finally { rmSync(directory, {recursive: true, force: true}); }
});

test('publisher reuses identical release without writes and refuses all substitutions or partial drafts', async () => {
  const directory = mkdtempSync(join(tmpdir(), 'prismpm-existing-release-'));
  try {
    const names = assetNames();
    for (const name of names) writeFileSync(join(directory, name), 'abc');
    const initial = {tag_name: publicationTag, target_commitish: revision,
      draft: false, prerelease: true, body: publicationNotes(revision),
      assets: names.map(name => ({name, size: 3}))};
    const fixture = (change = {}, content = 'abc', tagCommit = revision) => {
      const calls = [];
      const release = {...structuredClone(initial), ...change};
      const client = {environment: environment(),
        api: path => path.includes('/commits/') ? {sha: tagCommit} : release,
        checked: args => {
          calls.push(args);
          assert.equal(args[1], 'download', 'existing releases must never be written');
          const target = args[args.indexOf('--dir') + 1];
          for (const name of names) writeFileSync(join(target, name), content);
        }};
      return {client, calls};
    };
    const same = fixture();
    assert.equal(await publishOci(directory, revision, same.client),
      `https://github.com/UOR-Foundation/PrismPM/releases/tag/${publicationTag}`);
    assert.equal(same.calls.length, 1);
    for (const [change, content, tagCommit] of [
      [{}, 'abd', revision], [{draft: true, assets: []}, 'abc', revision],
      [{prerelease: false}, 'abc', revision], [{body: 'accepted'}, 'abc', revision],
      [{assets: initial.assets.slice(1)}, 'abc', revision], [{}, 'abc', 'b'.repeat(40)],
      [{assets: [...initial.assets.slice(1), initial.assets[1]]}, 'abc', revision],
    ]) {
      const changed = fixture(change, content, tagCommit);
      await assert.rejects(publishOci(directory, revision, changed.client));
      assert.ok(changed.calls.every(args => args[1] === 'download'));
    }
    rmSync(join(directory, names[0]));
    const missing = fixture();
    await assert.rejects(publishOci(directory, revision, missing.client), /asset closure is incomplete/);
    assert.equal(missing.calls.length, 0);
  } finally { rmSync(directory, {recursive: true, force: true}); }
});

test('new and complete staged publications verify all bytes before becoming public', async () => {
  const directory = mkdtempSync(join(tmpdir(), 'prismpm-new-release-'));
  try {
    const names = assetNames();
    for (const name of names) writeFileSync(join(directory, name), 'abc');
    for (const [corrupt, resume] of [[false, false], [true, false], [false, true], [true, true]]) {
      let release = resume ? {tag_name: publicationTag, target_commitish: revision,
        draft: true, prerelease: true, body: publicationNotes(revision),
        assets: names.map(name => ({name, size: 3}))} : null;
      let tag = null;
      const calls = [];
      const client = {environment: environment(),
        api: path => path.includes('/commits/') ? tag : release,
        checked: args => {
          calls.push(args);
          assert.ok(!args.includes('--clobber'));
          if (args[1] === 'create') {
            assert.ok(args.includes('--draft') && args.includes('--prerelease'));
            release = {tag_name: publicationTag, target_commitish: revision, draft: true,
              prerelease: true, body: args[args.indexOf('--notes') + 1], assets: []};
          } else if (args[1] === 'upload') {
            release.assets = names.map(name => ({name, size: 3}));
          } else if (args[1] === 'download') {
            const target = args[args.indexOf('--dir') + 1];
            for (const name of names) writeFileSync(join(target, name), corrupt ? 'bad' : 'abc');
          } else if (args[1] === 'edit') {
            assert.deepEqual(args.slice(-1), ['--draft=false']);
            release.draft = false; tag = {sha: revision};
          } else assert.fail(`unexpected publication command: ${args[1]}`);
        }};
      if (corrupt) {
        await assert.rejects(publishOci(directory, revision, client), /cannot be overwritten/);
        assert.equal(release.draft, true);
        assert.ok(!calls.some(args => args[1] === 'edit'));
      } else {
        await publishOci(directory, revision, client);
        assert.deepEqual(calls.map(args => args[1]), resume ? ['download', 'edit'] : ['create', 'upload', 'download', 'edit']);
      }
    }
  } finally { rmSync(directory, {recursive: true, force: true}); }
});

test('publication attempts have distinct discovery tags and invalid run contexts perform no I/O', async t => {
  const directory = mkdtempSync(join(tmpdir(), 'prismpm-attempt-release-'));
  t.after(() => rmSync(directory, {recursive: true, force: true}));
  const names = assetNames();
  for (const name of names) writeFileSync(join(directory, name), 'abc');
  const releases = new Map(), commits = new Map(), endpoints = [], writes = [];
  const client = {environment: environment(), api: path => {
    endpoints.push(path);
    if (path.includes('/commits/')) return commits.get(path.split('/').at(-1)) ?? null;
    return releases.get(path.split('/').at(-1)) ?? null;
  }, checked: args => {
    const tag = args[2];
    if (args[1] !== 'download') writes.push(args);
    if (args[1] === 'create') {
      assert(!releases.has(tag));
      releases.set(tag, {tag_name: tag, target_commitish: revision, draft: true, prerelease: true,
        body: args[args.indexOf('--notes') + 1], assets: []});
    } else if (args[1] === 'upload') {
      releases.get(tag).assets = names.map(name => ({name, size: 3}));
    } else if (args[1] === 'download') {
      const target = args[args.indexOf('--dir') + 1];
      for (const name of names) writeFileSync(join(target, name), 'abc');
    } else if (args[1] === 'edit') {
      releases.get(tag).draft = false; commits.set(tag, {sha: revision});
    } else assert.fail('unexpected release operation');
  }};
  for (const [run, attempt] of [['123456789', '1'], ['123456789', '2'], ['987654321', '1']]) {
    client.environment = {...environment(), GITHUB_RUN_ID: run, GITHUB_RUN_ATTEMPT: attempt};
    const expected = `sdk-oci-${revision}-${run}-${attempt}`;
    assert.equal(await publishOci(directory, revision, client),
      `https://github.com/UOR-Foundation/PrismPM/releases/tag/${expected}`);
    assert(endpoints.some(path => path.endsWith('/tags/' + expected)));
  }
  assert.equal(releases.size, 3, 'reruns must not collide with earlier evidence');
  const before = writes.length;
  await publishOci(directory, revision, client);
  assert.equal(writes.length, before, 'identical same-attempt publication is read-only');
  writeFileSync(join(directory, names[0]), 'abd');
  await assert.rejects(publishOci(directory, revision, client), /cannot be overwritten/);
  assert.equal(writes.length, before);
  for (const change of [{GITHUB_ACTIONS: undefined}, {GITHUB_ACTIONS: 'false'},
    ...['', '0', '01', '-1', '1.0', '1e3', ' 1', '1\n', '../main', undefined]
      .flatMap(value => [{GITHUB_RUN_ID: value}, {GITHUB_RUN_ATTEMPT: value}])]) {
    client.environment = {...environment(), ...change};
    const requests = endpoints.length;
    await assert.rejects(publishOci(directory, revision, client));
    assert.equal(endpoints.length, requests, 'invalid run identity must fail before API access');
    assert.equal(writes.length, before);
  }
});

test('the platform index names exactly the two pushed manifests, canonically', () => {
  const manifest = architecture => Buffer.from(JSON.stringify({schemaVersion: 2, mediaType: 'application/vnd.oci.image.manifest.v1+json',
    config: {mediaType: 'application/vnd.oci.image.config.v1+json', digest: sha(architecture), size: architecture.length}, layers: []}));
  const platforms = ['amd64', 'arm64'].map(architecture => ({architecture, digest: sha(manifest(architecture)), bytes: manifest(architecture)}));
  const index = platformIndex(platforms);
  assert.deepEqual(index, platformIndex(platforms.map(row => ({...row}))));
  assert.deepEqual(JSON.parse(index), {schemaVersion: 2, mediaType: 'application/vnd.oci.image.index.v1+json',
    manifests: platforms.map(({architecture, digest, bytes}) => ({mediaType: 'application/vnd.oci.image.manifest.v1+json',
      digest, size: bytes.length, platform: {architecture, os: 'linux'}}))});
  for (const broken of [
    platforms.slice(0, 1), [...platforms].reverse(), [platforms[0], {...platforms[1], digest: platforms[0].digest}],
    [platforms[0], {...platforms[1], bytes: Buffer.from(JSON.stringify({schemaVersion: 2, mediaType: 'application/vnd.oci.image.index.v1+json'}))}],
  ]) assert.throws(() => platformIndex(broken));
});

test('pinned Buildx pushes each platform by digest and the assembled two-platform OCI index creates no tags', {timeout: 240_000}, async () => {
  const directory = mkdtempSync(join(tmpdir(), 'prismpm-digest-export-'));
  const registry = `prismpm-digest-export-${process.pid}-${Date.now()}`;
  const docker = args => execFileSync('docker', args, {encoding: 'utf8', timeout: 90_000, maxBuffer: 8 * 1024 * 1024});
  let registryCreated = false, builderCreated = false;
  try {
    assert.match(docker(['buildx', 'version']), /github\.com\/docker\/buildx v0\.28\.0 /);
    const cached = JSON.parse(docker(['image', 'inspect', buildkit]))[0].RepoDigests;
    assert.ok(cached.includes(`docker.io/${buildkit}`) || cached.includes(buildkit),
    'run scripts/fetch.sh before offline exporter verification');
    const fetchScript = readFileSync(new URL('./fetch.sh', import.meta.url), 'utf8');
    assert.ok(fetchScript.includes(`buildkit_image='${buildkit}'`));
    const config = join(directory, 'zot.json');
    writeFileSync(config, JSON.stringify({distSpecVersion: '1.1.1', http: {address: '0.0.0.0', port: 5000},
      log: {level: 'warn'}, storage: {rootDirectory: '/tmp/zot'}}));
    docker(['create', '--pull=never', '--name', registry,
      'ghcr.io/project-zot/zot@sha256:cd2aea942f428630bcb4190542be6abd35e14177aab84fc7ccad0dca8ecb363d',
      'serve', '/tmp/config.json']);
    registryCreated = true;
    docker(['cp', config, `${registry}:/tmp/config.json`]);
    docker(['start', registry]);
    const address = docker(['inspect', '--format', '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}', registry]).trim();
    assert.match(address, /^\d+\.\d+\.\d+\.\d+$/);
    const endpoint = `${address}:5000`;
    let ready = false;
    for (let attempt = 0; attempt < 30; attempt++) {
      try { if ((await fetch(`http://${endpoint}/v2/`, {signal: AbortSignal.timeout(1000)})).ok) { ready = true; break; } } catch {}
      await new Promise(resolve => setTimeout(resolve, 100));
    }
    assert.ok(ready, 'pinned isolated registry did not become ready');
    docker(['buildx', 'create', '--name', registry, '--driver', 'docker-container', '--driver-opt', `image=${buildkit}`]);
    builderCreated = true;
    docker(['buildx', 'inspect', '--bootstrap', registry]);
    const context = join(directory, 'context'); mkdirSync(context);
    writeFileSync(join(context, 'Dockerfile'), 'FROM scratch\nARG SOURCE_DATE_EPOCH=0\nCOPY marker /marker\n');
    writeFileSync(join(context, 'marker'), 'synthetic export fixture, not an SDK\n');
    const args = ['buildx', 'build', '--builder', registry, '--no-cache', '--provenance=false', '--sbom=false',
      '--build-arg', 'SOURCE_DATE_EPOCH=0', '--label', 'org.opencontainers.image.created=1970-01-01T00:00:00Z',
      '--label', `org.opencontainers.image.revision=${revision}`, '--label',
      'org.opencontainers.image.source=https://github.com/UOR-Foundation/PrismPM',
      '--label', 'org.opencontainers.image.version=0.4.0'];
    const platforms = [];
    for (const architecture of ['amd64', 'arm64']) {
      const metadata = join(directory, `${architecture}-metadata.json`);
      docker([...args, '--platform', `linux/${architecture}`, '--metadata-file', metadata, '--output',
        `type=registry,name=${endpoint}/fixture/sdk,push-by-digest=true,name-canonical=true,rewrite-timestamp=true,oci-mediatypes=true,registry.insecure=true`, context]);
      const platformDigest = JSON.parse(readFileSync(metadata))['containerimage.digest'];
      assert.match(platformDigest, /^sha256:[0-9a-f]{64}$/);
      const fetched = await fetch(`http://${endpoint}/v2/fixture/sdk/manifests/${platformDigest}`,
        {headers: {Accept: 'application/vnd.oci.image.manifest.v1+json'}, signal: AbortSignal.timeout(5000)});
      assert.equal(fetched.status, 200);
      platforms.push({architecture, digest: platformDigest, bytes: Buffer.from(await fetched.arrayBuffer())});
    }
    const assembled = platformIndex(platforms), digest = sha(assembled);
    const pushed = await fetch(`http://${endpoint}/v2/fixture/sdk/manifests/${digest}`, {method: 'PUT', body: assembled,
      headers: {'Content-Type': 'application/vnd.oci.image.index.v1+json'}, signal: AbortSignal.timeout(5000)});
    assert.equal(pushed.status, 201);
    const response = await fetch(`http://${endpoint}/v2/fixture/sdk/manifests/${digest}`,
      {headers: {Accept: 'application/vnd.oci.image.index.v1+json'}, signal: AbortSignal.timeout(5000)});
    assert.equal(response.status, 200);
    const index = Buffer.from(await response.arrayBuffer()); assert.equal(sha(index), digest);
    const tags = await fetch(`http://${endpoint}/v2/fixture/sdk/tags/list`, {signal: AbortSignal.timeout(5000)});
    assert.equal(tags.status, 200); assert.deepEqual((await tags.json()).tags ?? [], []);
    for (const platform of ['amd64', 'arm64']) {
      for (const name of ['a', 'b']) {
        const target = join(directory, `${platform}-${name}`); mkdirSync(target);
        const archive = `${target}.tar`;
        docker([...args, '--platform', `linux/${platform}`, '--output',
          `type=oci,dest=${archive},rewrite-timestamp=true,oci-mediatypes=true`, context]);
        execFileSync('tar', ['-xf', archive, '-C', target]);
      }
      verifyReproducibility(`ghcr.io/uor-foundation/prismpm-sdk@${digest}`, index, platform, revision,
        join(directory, `${platform}-a`), join(directory, `${platform}-b`));
      for (const side of ['a', 'b']) {
        const target = join(directory, `${platform}-${side}`), localIndex = JSON.parse(readFileSync(join(target, 'index.json')));
        const manifest = JSON.parse(readFileSync(join(target, 'blobs/sha256', localIndex.manifests[0].digest.slice(7))));
        assert.ok(manifest.layers.length > 0, 'genuine Buildx fixture must exercise actual layer verification');
        const path = join(target, 'blobs/sha256', manifest.layers[0].digest.slice(7)), original = readFileSync(path);
        for (const damage of ['missing', 'changed', 'truncated', 'alias']) {
          if (damage === 'missing') rmSync(path);
          else if (damage === 'changed') writeFileSync(path, Buffer.alloc(original.length, 0));
          else if (damage === 'truncated') writeFileSync(path, original.subarray(0, -1));
          else {
            const actual = join(directory, 'external-buildx-layer'); writeFileSync(actual, original);
            rmSync(path); symlinkSync(actual, path);
          }
          assert.throws(() => verifyReproducibility(`ghcr.io/uor-foundation/prismpm-sdk@${digest}`,
            index, platform, revision, join(directory, `${platform}-a`), join(directory, `${platform}-b`)),
          undefined, `actual ${platform}/${side} ${damage} must refuse`);
          if (damage !== 'missing') rmSync(path);
          writeFileSync(path, original);
        }
      }
    }
  } finally {
    try { if (builderCreated) docker(['buildx', 'rm', '--force', registry]); }
    finally {
      try { if (registryCreated) docker(['rm', '--force', registry]); } finally { rmSync(directory, {recursive: true, force: true}); }
    }
  }
});
