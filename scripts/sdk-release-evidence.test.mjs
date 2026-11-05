// Evidence preservation tests, not execution or acceptance of an SDK image.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {createRequire} from 'node:module';
import {linkSync, lstatSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, symlinkSync, truncateSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname, join} from 'node:path';
import {pathToFileURL} from 'node:url';
import test from 'node:test';
import {captureEvidence, packEvidence} from './sdk-release-evidence.mjs';
import {assetNames} from './release-phases.mjs';
import {bootstrapFixture} from './sdk-bootstrap-retention-fixture.mjs';
import {sourceAuthoringFiles} from './sdk-vv-run.mjs';
const require = createRequire('/opt/prismpm/oracles/package.json');
const {load} = require('js-yaml');
const canonical = value => JSON.stringify(value, (_, item) => item && typeof item === 'object' && !Array.isArray(item)
  ? Object.fromEntries(Object.keys(item).sort().map(key => [key, item[key]])) : item);
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const context = {source_revision: 'a'.repeat(40), architecture: 'amd64',
  image_reference: 'ghcr.io/uor-foundation/prismpm-sdk@sha256:' + 'b'.repeat(64)};
const temporary = t => {const root = mkdtempSync(join(tmpdir(), 'prismpm-evidence-test-')); t.after(() => rmSync(root, {recursive: true, force: true})); return root;};
const put = (root, name, value) => writeFileSync(join(root, name), typeof value === 'string' ? value : canonical(value));
function authoringMetadata() {
  const policy = {schema: 'prismpm/sdk-vv-input-policy/1', source_revision: context.source_revision};
  const sourceFiles = ['Cargo.lock', 'lexlean.lock', 'model/dependencies.toml', 'rust-toolchain.toml', 'lean-toolchain', 'scripts/source-authoring-sdk-check.mjs']
    .map(path => ({path, sha256: sha('unit-only source ' + path)}));
  const manifest = {policy, source: {revision: context.source_revision, files: sourceFiles},
    artifacts: [{path: 'source.pack', sha256: sha('unit-only source pack')}]};
  const policyBytes = canonical(policy) + '\n', manifestBytes = canonical(manifest) + '\n';
  const commands = ['lexlean', 'cargo', 'rustc', 'lean', 'lake'].map(command =>
    ({command, executable: '/usr/local/bin/' + command, sha256: sha('unit-only tool ' + command)}));
  const sourceArtifact = {id: 'sdk-vv-source', kind: 'test-corpus', version: context.source_revision, digest: 'sha256:' + manifest.artifacts[0].sha256};
  const lexlean = {id: 'lexlean', kind: 'binary', version: '0.4.0', digest: 'sha256:' + commands[0].sha256};
  const inventory = {schema: 'prismpm/sdk-inventory/1', commands, artifacts: [sourceArtifact, lexlean,
    {id: 'prismpm', kind: 'binary', version: '0.4.0', digest: 'sha256:' + 'd'.repeat(64)},
    {id: 'sdk-vv-policy', kind: 'test-corpus', version: '1', digest: 'sha256:' + sha(policyBytes)},
    {id: 'sdk-vv-manifest', kind: 'test-corpus', version: '1', digest: 'sha256:' + sha(manifestBytes)}]};
  const inventoryBytes = canonical(inventory) + '\n';
  return {policyBytes, manifestBytes, inventoryBytes, sourceFiles, sourceArtifact, lexlean, commands};
}
function authoring(root, metadata) {
  const harness = metadata.sourceFiles.at(-1).sha256;
  const receipt = {schema: 'prismpm/source-authoring-sdk-check/1', status: 'passed', scope: 'source-free-installed-authoring-component',
    image: context.image_reference, source_revision: context.source_revision, platform: 'linux/amd64',
    cli_sha256: 'd'.repeat(64), inventory_sha256: sha(metadata.inventoryBytes), policy_sha256: sha(metadata.policyBytes),
    manifest_sha256: sha(metadata.manifestBytes), source_artifact: metadata.sourceArtifact, lexlean: metadata.lexlean,
    source_locks: metadata.sourceFiles.slice(0, -1), tool_commands: metadata.commands,
    source_harness_sha256: harness, harness_sha256: harness,
    checks: Array.from({length: 36}, (_, index) => ({operation: ['read', 'write', 'to-notation'][index % 3], arguments: [],
      input_sha256: sha('unit-only input ' + index), output_sha256: sha('unit-only output ' + index), output_bytes: 1, status: 0, diagnostic: null})),
    output_failure: {sink: '/dev/full', status: 1, diagnostic: 'PP5804', disposition: 'Unknown', stderr_sha256: sha('unit-only unknown output')},
    unclaimed: ['semantic-validity', 'linked-snapshot', 'product-targets', 'whole-sdk-qualification', 'release-acceptance']};
  for (const run of [1, 2]) {
    const directory = join(root, 'source-authoring', `run-${run}`); mkdirSync(directory, {recursive: true});
    const raw = JSON.stringify(receipt) + '\n';
    for (const [name, status] of [['authoring', 0], ['inventory-path', 4], ['inventory-binary', 4]]) {
      put(directory, name + '.stdout', name === 'authoring' ? raw
        : canonical({schema: 'prismpm/error-result/1', diagnostic: {code: 'PP5401', message: 'unit-only inventory refusal'}}) + '\n');
      put(directory, name + '.stderr', '');
      put(directory, name + '.status.json', canonical({scope: 'process-capture-only', status, signal: null, error: null}) + '\n');
    }
    put(directory, 'receipt.json', raw); put(directory, 'receipt.sha256', sha(raw) + '\n');
    refreshCollection(root, run);
  }
}
function refreshCollection(root, run) {
  const directory = join(root, 'source-authoring', `run-${run}`);
  put(directory, 'collection.json', {schema: 'prismpm/source-authoring-retention/1', scope: 'diagnostic-retention-only',
    run, execution_status: 0, status: 'retained', files: sourceAuthoringFiles.map(path => {
      const bytes = readFileSync(join(directory, path));
      return {path, byte_length: bytes.length, sha256: sha(bytes), base64: bytes.toString('base64')};
    })});
}
function changeReceipt(root, mutate) {
  const directory = join(root, 'source-authoring/run-1'), receipt = JSON.parse(readFileSync(join(directory, 'receipt.json')));
  mutate(receipt); const raw = JSON.stringify(receipt) + '\n';
  put(directory, 'receipt.json', raw); put(directory, 'authoring.stdout', raw); put(directory, 'receipt.sha256', sha(raw) + '\n');
  refreshCollection(root, 1);
}
function vv(root) {
  const metadata = authoringMetadata();
  const bootstrap = bootstrapFixture(context.source_revision);
  for (const [name, bytes] of [...bootstrap.retained, ['bootstrap.json', bootstrap.manifest]]) writeFileSync(join(root, name), bytes);
  mkdirSync(join(root, 'docker'));
  put(join(root, 'docker'), 'config.json', 'private transport data that must never ship');
  const raw = canonical({schema: 'prismpm/vv-evidence/1', commit: context.source_revision,
    gates: Array.from({length: 15}, (_, index) => index + 1), status: 'passed'});
  for (const run of [1, 2]) put(root, `run-${run}.json`, raw);
  const execution = canonical({schema: 'prismpm/sdk-vv-execution/1', scope: 'two-full-vv-executions-only',
    source_revision: context.source_revision, image_reference: context.image_reference, process_architecture: 'x64',
    inventory_sha256: sha(metadata.inventoryBytes), cli_sha256: 'd'.repeat(64), input_policy_sha256: sha(metadata.policyBytes),
    input_manifest_sha256: sha(metadata.manifestBytes), advisory_revision: '1'.repeat(40),
    runs: [1, 2].map(run => ({run, path: `run-${run}/vv-evidence.json`, byte_length: raw.length, sha256: sha(raw)})),
    unclaimed: ['native-host', 'network-isolation', 'oci-image-identity', 'sdk-release', 'product-readiness']});
  put(root, 'execution.json', execution);
  put(root, 'acceptance.json', {schema: 'prismpm/sdk-installed-vv-check/1', status: 'passed', scope: 'isolated-installed-two-run-vv',
    ...context, execution_sha256: sha(execution), phases: ['exact-native-images-acquired', 'external-network-disconnected',
      'isolated-native-sdk-probed', 'both-full-vv-records-verified', 'owned-resources-removed'],
    unclaimed: ['hardware-attestation', 'sdk-release', 'product-readiness']});
  for (const name of ['image-plan.json', 'loaded-identities.json']) put(root, name, {fixture: 'transport-only'});
  put(root, '0001.stdout', 'unit transcript\n'); put(root, '0001.stderr', '');
  put(root, '0001.json', {arguments: ['info'], status: 0, signal: null,
    stdout_sha256: sha('unit transcript\n'), stderr_sha256: sha('')});
  for (const [index, path, bytes] of [[2, '/opt/prismpm/share/inventory.json', metadata.inventoryBytes],
    [3, '/opt/prismpm/share/vv-input-policy.json', metadata.policyBytes], [4, '/opt/prismpm/share/vv-inputs/manifest.json', metadata.manifestBytes]]) {
    const number = String(index).padStart(4, '0');
    put(root, number + '.stdout', bytes); put(root, number + '.stderr', '');
    put(root, number + '.json', {arguments: ['exec', 'unit-sdk', 'cat', path], status: 0, signal: null,
      stdout_sha256: sha(bytes), stderr_sha256: sha('')});
  }
  authoring(root, metadata);
}
function product(root) {
  const names = ['source.json', 'image.json', 'inventory.json', 'standards.lock', 'prismpm.lock',
    'acquire.stdout.json', 'acquire.stderr.txt', 'acquisition.json', 'build.stdout.json', 'build.stderr.txt',
    'build.json', 'result.json', 'receiver.stderr.txt'];
  for (const name of names) put(root, name, {fixture: 'transport-only'});
  put(root, 'source.json', {revision: context.source_revision});
  for (const name of ['build.json', 'prismpm.lock']) put(root, name, {sdk_image: context.image_reference});
  put(root, 'image.json', [{Architecture: context.architecture, Os: 'linux', Config: {Labels: {'org.opencontainers.image.revision': context.source_revision}}}]);
  const files = ['app.css', 'app.js', 'index.html', 'prism_calculator.js', 'prism_calculator_bg.wasm', 'provenance.json']
    .map(path => ({path, size: 1, digest: 'sha256:' + 'c'.repeat(64)}));
  put(root, 'result.json', {schema: 'prismpm/installed-product-cli-check/1', scope: 'installed-cli-product-build-and-source-free-integrity',
    status: 'passed', sdk_image: context.image_reference, releases: ['A', 'B'].map((release, index) => ({release,
      release_digest: 'sha256:' + String(index).repeat(64), model_digest: 'sha256:' + 'd'.repeat(64),
      build_digest: 'sha256:' + 'e'.repeat(64), tree_digest: 'sha256:' + sha(canonical(files)), files,
      checks: ['source-free-proof-replay', 'exact-browser-export', 'missing-proof-refused', 'changed-proof-refused', 'restored-export']})),
    unclaimed: ['sdk-release', 'crates.io-publication', 'producer-authorization', 'deployment', 'foundry-readiness']});
}

function validateWorkflow(workflow) {
  const upload = workflow.jobs.reproducibility.steps.filter(step => step.with?.name === 'library-sdk-${{ matrix.name }}');
  assert.equal(upload.length, 1);
  assert.equal(upload[0].uses, 'actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02');
  assert.equal(upload[0].if, "always() && matrix.image == 'sdk'");
  assert.equal(upload[0]['continue-on-error'], undefined);
  assert.deepEqual(upload[0].with, {name: 'library-sdk-${{ matrix.name }}',
    path: '${{ matrix.name }}-library-sdk.log\nroot-a/target/library-sdk-evidence/\n',
    'if-no-files-found': 'error', 'retention-days': 30});
  assert(assetNames().includes('sdk-metadata-native-join.json'));
  const steps = workflow.jobs['oci-native'].steps;
  const pack = steps.find(step => step.run?.includes('scripts/sdk-release-evidence.mjs'));
  assert(pack && pack.if === undefined && pack['continue-on-error'] === undefined);
  const download = (name, path) => {
    const owner = name === 'source-vv' ? 'source' : name.startsWith('full-sdk-vv-sdk-') ? 'full-' + name.split('-').at(-1)
      : name.startsWith('sdk-equivalence-') ? 'equivalence-' + name.split('-').at(-1) : null;
    const identity = owner ? '${{ steps.' + owner + '.outputs.ids }}' : null;
    const matching = steps.filter(step => owner ? step.with?.['artifact-ids'] === identity : step.with?.name === name);
    assert.equal(matching.length, 1, 'missing or duplicate gate download ' + name);
    const step = matching[0];
    assert.equal(step.uses, 'actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093');
    assert.deepEqual(step.with, owner ? {'artifact-ids': identity, 'merge-multiple': true, path} : {name, path});
    if (owner) {
      const selection = steps.find(candidate => candidate.id === owner);
      assert(selection.run.includes('scripts/release-artifact-metadata.mjs'));
      assert(selection.run.includes('scripts/release-gate-evidence.mjs select-artifacts'));
      assert(selection.run.includes(name));
      assert(steps.indexOf(selection) < steps.indexOf(step));
    }
    assert.equal(step.if, undefined); assert.equal(step['continue-on-error'], undefined);
    assert(steps.indexOf(step) < steps.indexOf(pack), 'evidence must arrive before capture');
  };
  for (const arch of ['amd64', 'arm64']) {
    for (const [artifact, suffix] of [['full-sdk-vv', 'full-sdk-vv'], ['product-cli', 'product-cli']]) {
      const name = `${artifact}-sdk-${arch}`;
      download(name, `.sdk-gate-evidence/${name}`);
      assert(assetNames().includes(`sdk-${arch}-${suffix}.tar`));
    }
    download(`browser-sdk-sdk-${arch}`, 'release');
    download(`library-sdk-sdk-${arch}`, `.sdk-gate-evidence/library-sdk-sdk-${arch}`);
    for (const kind of ['browser', 'library']) assert(assetNames().includes(`sdk-${arch}-${kind}-sdk.log`));
    assert(assetNames().includes(`sdk-${arch}-native-metadata.tar`));
  }
  assert.equal(pack.shell, 'bash');
  assert.equal(pack.run, [
    'set -euo pipefail', 'image=$(cat release/sdk-image.txt)',
    'node scripts/library-sdk-metadata-evidence.mjs join \\',
    '  .sdk-gate-evidence/library-sdk-sdk-amd64/root-a/target/library-sdk-evidence/linux-amd64 \\',
    '  .sdk-gate-evidence/library-sdk-sdk-arm64/root-a/target/library-sdk-evidence/linux-arm64 \\',
    '  . "$image" "$GITHUB_SHA" > release/sdk-metadata-native-join.json',
    'cp .sdk-gate-evidence/library-sdk-sdk-amd64/sdk-amd64-library-sdk.log release/',
    'cp .sdk-gate-evidence/library-sdk-sdk-arm64/sdk-arm64-library-sdk.log release/',
    'node scripts/sdk-release-evidence.mjs source-vv .sdk-gate-evidence/source-vv \\',
    "  release/source-vv.tar \"$image\" \"$GITHUB_SHA\" amd64 \"$GITHUB_RUN_ID\" \"$(node -e 'console.log(require(process.argv[1]).artifacts[0].run_attempt)' \"$PWD/target/release-selection/source.json\")\"",
    'for architecture in amd64 arm64; do',
    '  node scripts/library-sdk-metadata-evidence.mjs pack \\',
    '    ".sdk-gate-evidence/library-sdk-sdk-$architecture/root-a/target/library-sdk-evidence/linux-$architecture" \\',
    '    "release/sdk-$architecture-native-metadata.tar" . "$image" "$GITHUB_SHA" "$architecture"',
    '  for kind in full-sdk-vv product-cli; do',
    '    evidence=".sdk-gate-evidence/$kind-sdk-$architecture"',
    '    if test "$kind" = full-sdk-vv; then evidence="$evidence/sdk-$architecture-full-sdk-vv"; fi',
    '    node scripts/sdk-release-evidence.mjs "$kind" \\',
    '      "$evidence" "release/sdk-$architecture-$kind.tar" \\',
    '      "$image" "$GITHUB_SHA" "$architecture"', '  done',
    '  node scripts/sdk-release-evidence.mjs native-equivalence \\',
    '    ".sdk-gate-evidence/native-equivalence-sdk-$architecture" "release/sdk-$architecture-native-equivalence.tar" \\',
    "    \"$image\" \"$GITHUB_SHA\" \"$architecture\" \"$GITHUB_RUN_ID\" \"$(node -e 'console.log(require(process.argv[1]).artifacts[0].run_attempt)' \"$PWD/target/release-selection/equivalence-$architecture.json\")\"",
    'done', '',
  ].join('\n'));
  download('source-vv', '.sdk-gate-evidence/source-vv');
  for (const arch of ['amd64', 'arm64']) download(`sdk-equivalence-${arch}`, `.sdk-gate-evidence/native-equivalence-sdk-${arch}`);
  assert(steps.indexOf(pack) < steps.findIndex(step => step.run?.includes('checksums=$(mktemp)')));
  assert.deepEqual(workflow.jobs['oci-native'].needs, ['gate', 'images', 'native', 'reproducibility', 'installed-sdk']);
  assert(workflow.jobs.release.needs.includes('crates'));
}

test('OCI publication retains both native gate closures without granting SDK acceptance', () => {
  const workflow = load(readFileSync(new URL('../.github/workflows/release.yml', import.meta.url), 'utf8'));
  validateWorkflow(workflow);
  const steps = value => value.jobs['oci-native'].steps;
  const pack = value => steps(value).find(step => step.run?.includes('scripts/sdk-release-evidence.mjs'));
  const gate = value => steps(value).find(step => step.with?.['artifact-ids'] === '${{ steps.full-arm64.outputs.ids }}');
  for (const mutate of [
    value => steps(value).splice(steps(value).indexOf(gate(value)), 1),
    value => steps(value).push(structuredClone(gate(value))),
    value => {gate(value).with.path = 'release';},
    value => {gate(value).uses = 'actions/download-artifact@main';},
    value => {gate(value).if = 'always()';},
    value => {gate(value)['continue-on-error'] = true;},
    value => {const record = gate(value); steps(value).splice(steps(value).indexOf(record), 1); steps(value).push(record);},
    value => {pack(value).run = pack(value).run.replace('/sdk-$architecture-full-sdk-vv', '');},
    value => {pack(value).if = 'false';},
    value => {pack(value)['continue-on-error'] = true;},
    value => {pack(value).run = pack(value).run.replace('set -euo pipefail', 'set -uo pipefail');},
    value => {pack(value).run = pack(value).run.replace('amd64 arm64', 'amd64');},
    value => {pack(value).run = pack(value).run.replace('library-sdk-metadata-evidence.mjs join', 'library-sdk-metadata-evidence.mjs omitted');},
    value => {pack(value).run = pack(value).run.replace('library-sdk-metadata-evidence.mjs pack', 'library-sdk-metadata-evidence.mjs omitted');},
    value => {value.jobs.reproducibility.steps.find(step => step.with?.name === 'library-sdk-${{ matrix.name }}').with.path = '${{ matrix.name }}-library-sdk.log';},
    value => {value.jobs.reproducibility.steps.find(step => step.with?.name === 'library-sdk-${{ matrix.name }}').with.path = 'root-a/target/library-sdk-evidence/';},
    value => {value.jobs.reproducibility.steps.find(step => step.with?.name === 'library-sdk-${{ matrix.name }}').with['if-no-files-found'] = 'ignore';},
    value => {pack(value).run = pack(value).run.replace('"$GITHUB_SHA"', '"stale"');},
    value => {pack(value).run = pack(value).run.replace('"$architecture"', '"$architecture" || true');},
    value => {const record = pack(value); steps(value).splice(steps(value).indexOf(record), 1); steps(value).push(record);},
    value => {value.jobs['oci-native'].needs.pop();},
    value => {value.jobs.release.needs = value.jobs.release.needs.filter(name => name !== 'crates');},
  ]) {
    const changed = structuredClone(workflow); mutate(changed); assert.throws(() => validateWorkflow(changed));
  }
});

test('real deterministic USTAR archives retain every original byte and explicitly exclude private transport configuration', t => {
  for (const [kind, fixture] of [['full-sdk-vv', vv], ['product-cli', product]]) {
    const source = temporary(t), output = temporary(t); fixture(source);
    const captured = captureEvidence(source, kind, context);
    if (kind === 'full-sdk-vv') assert.equal([...captured.files.keys()].filter(name => name.startsWith('source-authoring/')).length, 24);
    const first = join(output, 'one.tar'), second = join(output, 'two.tar');
    packEvidence(source, kind, first, context); packEvidence(source, kind, second, context);
    assert.deepEqual(readFileSync(first), readFileSync(second));
    assert.throws(() => packEvidence(source, kind, first, context), /fresh archive/);
    const extracted = join(output, 'extracted'); mkdirSync(extracted);
    execFileSync('tar', ['-xf', first, '-C', extracted]);
    assert.deepEqual(readdirSync(extracted).sort(), [...new Set([...captured.files.keys()].map(name => name.split('/')[0])), 'evidence-manifest.json'].sort());
    for (const [name, bytes] of captured.files) assert.deepEqual(readFileSync(join(extracted, name)), bytes);
    assert.deepEqual(JSON.parse(readFileSync(join(extracted, 'evidence-manifest.json'))), captured.manifest);
    assert.equal(captured.manifest.scope, 'original-gate-evidence-only');
    assert(captured.manifest.unclaimed.includes('sdk-acceptance'));
    assert(!readFileSync(first).includes(Buffer.from('private transport data')));
  }
});

test('closed source-authoring directories reject missing, extra, aliased and oversized originals', t => {
  for (const change of [
    root => rmSync(join(root, 'source-authoring'), {recursive: true}),
    root => rmSync(join(root, 'source-authoring/run-2'), {recursive: true}),
    root => rmSync(join(root, 'source-authoring/run-1/authoring.status.json')),
    root => mkdirSync(join(root, 'source-authoring/run-3')),
    root => put(join(root, 'source-authoring/run-1'), 'capture-error.json', '{}'),
    root => mkdirSync(join(root, 'source-authoring/run-1/deeper')),
    root => {rmSync(join(root, 'source-authoring'), {recursive: true}); symlinkSync('.', join(root, 'source-authoring'));},
    root => {rmSync(join(root, 'source-authoring/run-1'), {recursive: true}); symlinkSync('run-2', join(root, 'source-authoring/run-1'));},
    root => {rmSync(join(root, 'source-authoring/run-1/receipt.json')); symlinkSync('authoring.stdout', join(root, 'source-authoring/run-1/receipt.json'));},
    root => {rmSync(join(root, 'source-authoring/run-1/inventory-path.stderr')); linkSync(join(root, 'source-authoring/run-1/inventory-binary.stderr'), join(root, 'source-authoring/run-1/inventory-path.stderr'));},
    root => truncateSync(join(root, 'source-authoring/run-1/authoring.stdout'), 8 * 1024 * 1024 + 1),
    root => truncateSync(join(root, 'source-authoring/run-1/collection.json'), 32 * 1024 * 1024 + 1),
  ]) {
    const source = temporary(t), output = temporary(t); vv(source); change(source);
    assert.throws(() => packEvidence(source, 'full-sdk-vv', join(output, 'bad.tar'), context));
    assert.deepEqual(readdirSync(output), []);
  }
});

test('self-consistent source-authoring changes cannot replace original SDK identity or process outcomes', t => {
  for (const mutate of [
    row => {row.image = context.image_reference.replace(/b/g, 'c');}, row => {row.source_revision = '1'.repeat(40);},
    row => {row.platform = 'linux/arm64';}, row => {row.cli_sha256 = '0'.repeat(64);},
    row => {row.inventory_sha256 = '0'.repeat(64);}, row => {row.policy_sha256 = '0'.repeat(64);},
    row => {row.manifest_sha256 = '0'.repeat(64);}, row => {row.source_artifact.digest = 'sha256:' + '0'.repeat(64);},
    row => {row.lexlean.digest = 'sha256:' + '0'.repeat(64);}, row => {row.source_locks[0].sha256 = '0'.repeat(64);},
    row => {row.tool_commands[2].sha256 = '0'.repeat(64);},
    row => {row.harness_sha256 = row.source_harness_sha256 = '0'.repeat(64);},
    row => {row.checks.pop();}, row => {row.checks[0].status = 2;}, row => {row.output_failure.disposition = 'known';},
    row => {row.unclaimed.pop();}, row => {row.extra = true;},
  ]) {
    const source = temporary(t); vv(source); changeReceipt(source, mutate);
    assert.throws(() => captureEvidence(source, 'full-sdk-vv', context));
  }
  for (const change of [
    root => {put(join(root, 'source-authoring/run-1'), 'authoring.status.json', {scope: 'process-capture-only', status: 1, signal: null, error: null}); refreshCollection(root, 1);},
    root => {put(join(root, 'source-authoring/run-1'), 'inventory-path.status.json', {scope: 'process-capture-only', status: 0, signal: null, error: null}); refreshCollection(root, 1);},
    root => {put(join(root, 'source-authoring/run-1'), 'inventory-binary.stdout', {schema: 'prismpm/error-result/1', diagnostic: {code: 'PP5801'}}); refreshCollection(root, 1);},
    root => {
      const path = join(root, 'source-authoring/run-1/inventory-path.stdout');
      writeFileSync(path, Buffer.concat([Buffer.from('{"schema":"prismpm/error-result/1","diagnostic":{"code":"PP5401","message":"'), Buffer.from([255]), Buffer.from('"}}\n')]));
      refreshCollection(root, 1);
    },
    root => {const row = JSON.parse(readFileSync(join(root, '0002.json'))); row.arguments[row.arguments.length - 1] = '/wrong/inventory.json'; put(root, '0002.json', row);},
  ]) {
    const source = temporary(t); vv(source); change(source);
    assert.throws(() => captureEvidence(source, 'full-sdk-vv', context));
  }
});

test('component collection retains exact original roles and refuses rewritten or duplicate metadata', t => {
  for (const mutate of [
    row => {row.run = 2;}, row => {row.execution_status = 7;}, row => {row.status = 'not-produced';},
    row => {row.files.pop();}, row => {row.files[0] = row.files[1];}, row => {row.files[0].path = '../escape';},
    row => {row.files[0].sha256 = '0'.repeat(64);}, row => {row.files[0].base64 += '\n';},
    row => {row.files[0].byte_length++;}, row => {row.extra = true;},
  ]) {
    const source = temporary(t); vv(source);
    const path = join(source, 'source-authoring/run-1/collection.json'), row = JSON.parse(readFileSync(path));
    mutate(row); put(source, 'source-authoring/run-1/collection.json', row);
    assert.throws(() => captureEvidence(source, 'full-sdk-vv', context));
  }
});

test('the exact 9999-command file closure includes its excluded private directory and rejects one extra entry', {timeout: 60000}, t => {
  const source = temporary(t); vv(source);
  const empty = sha('');
  for (let index = 5; index <= 9999; index++) {
    const number = String(index).padStart(4, '0');
    put(source, number + '.stdout', ''); put(source, number + '.stderr', '');
    put(source, number + '.json', {arguments: ['info'], status: 0, signal: null,
      stdout_sha256: empty, stderr_sha256: empty});
  }
  assert.equal(readdirSync(source).length, 30014);
  const captured = captureEvidence(source, 'full-sdk-vv', context);
  assert.equal(captured.files.size, 30036);
  assert.equal(captured.manifest.files.length, 30036);
  assert.deepEqual(captured.manifest.excluded, ['docker/']);
  assert(!captured.files.has('docker'));
  assert.equal(readFileSync(join(source, 'docker', 'config.json'), 'utf8'), 'private transport data that must never ship');
  put(source, '10000.json', '{}');
  assert.equal(readdirSync(source).length, 30015);
  assert.throws(() => captureEvidence(source, 'full-sdk-vv', context), /evidence file-count limit/);
});

test('missing, extra, stale and changed original evidence fails before archive creation', t => {
  for (const change of [
    root => rmSync(join(root, 'run-2.json')),
    root => rmSync(join(root, 'run-2-bootstrap-evidence.json')),
    root => put(root, 'run-1-bootstrap-current-capture.json', '{}'),
    root => put(root, 'extra.json', {}),
    root => put(root, '0001.stdout', 'changed'),
    root => put(root, 'run-1.json', {status: 'passed'}),
    root => {const value = JSON.parse(readFileSync(join(root, 'acceptance.json'))); value.source_revision = '2'.repeat(40); put(root, 'acceptance.json', value);},
    root => {const value = JSON.parse(readFileSync(join(root, 'acceptance.json'))); value.phases.pop(); put(root, 'acceptance.json', value);},
    root => {const value = JSON.parse(readFileSync(join(root, 'execution.json'))); value.runs[1] = value.runs[0]; put(root, 'execution.json', value);},
  ]) {
    const source = temporary(t), output = temporary(t); vv(source); change(source);
    assert.throws(() => packEvidence(source, 'full-sdk-vv', join(output, 'bad.tar'), context));
    assert.deepEqual(readdirSync(output), []);
  }
  const source = temporary(t); product(source);
  for (const change of [{architecture: 'arm64'}, {image_reference: context.image_reference.replace(/b/g, 'c')}, {source_revision: '2'.repeat(40)}, {extra: true}]) {
    assert.throws(() => captureEvidence(source, 'product-cli', {...context, ...change}));
  }
});

test('aliases, nonregular files, private-directory aliases and oversized evidence reject', t => {
  for (const change of [
    root => {rmSync(join(root, '0001.stdout')); symlinkSync('0001.stderr', join(root, '0001.stdout'));},
    root => {rmSync(join(root, '0001.stdout')); linkSync(join(root, '0001.stderr'), join(root, '0001.stdout'));},
    root => {rmSync(join(root, '0001.stdout')); mkdirSync(join(root, '0001.stdout'));},
    root => truncateSync(join(root, '0001.stdout'), 268435457),
    root => truncateSync(join(root, '0001.stderr'), 67108865),
    root => {rmSync(join(root, 'docker'), {recursive: true}); symlinkSync('.', join(root, 'docker'));},
  ]) {const source = temporary(t); vv(source); change(source); assert.throws(() => captureEvidence(source, 'full-sdk-vv', context));}
  const source = temporary(t); vv(source); const parent = temporary(t), alias = join(parent, 'alias'); symlinkSync(source, alias);
  assert.throws(() => captureEvidence(alias, 'full-sdk-vv', context));
});

test('actual maximum file archive and one-GiB captured closure retain exact bytes without clamping', {timeout: 120000}, t => {
  const maximum = 268435456, zero = Buffer.alloc(maximum), fullHash = sha(zero);
  const row = digest => ({arguments: ['info'], status: 0, signal: null, stdout_sha256: digest, stderr_sha256: sha('')});
  const source = temporary(t); vv(source);
  truncateSync(join(source, '0001.stdout'), 0); truncateSync(join(source, '0001.stdout'), maximum);
  put(source, '0001.json', row(fullHash));
  const output = join(temporary(t), 'maximum.tar');
  const manifest = packEvidence(source, 'full-sdk-vv', output, context);
  assert.equal(manifest.files.find(file => file.path === '0001.stdout').size, maximum);
  const restored = execFileSync('tar', ['-xOf', output, '0001.stdout'], {maxBuffer: maximum + 1024, timeout: 30000});
  assert.equal(restored.length, maximum); assert.equal(sha(restored), fullHash);

  // Sparse original stdout files exercise the complete allocation/read/hash
  // boundary without reserving one GiB of persistent development disk space.
  const aggregate = temporary(t); vv(aggregate);
  for (let index = 5; index <= 8; index++) {
    const number = String(index).padStart(4, '0');
    put(aggregate, number + '.stdout', ''); truncateSync(join(aggregate, number + '.stdout'), maximum);
    put(aggregate, number + '.stderr', ''); put(aggregate, number + '.json', row(fullHash));
  }
  const size = (directory = aggregate) => readdirSync(directory).filter(name => name !== 'docker')
    .reduce((sum, name) => {const path = join(directory, name), stat = lstatSync(path); return sum + (stat.isDirectory() ? size(path) : stat.size);}, 0);
  const total = 1024 ** 3, lastSize = maximum - (size() - total);
  assert(lastSize > 0 && lastSize < maximum);
  truncateSync(join(aggregate, '0008.stdout'), lastSize);
  put(aggregate, '0008.json', row(sha(zero.subarray(0, lastSize))));
  assert.equal(size(), total);
  const captured = captureEvidence(aggregate, 'full-sdk-vv', context);
  assert.equal(captured.manifest.files.reduce((sum, file) => sum + file.size, 0), total);
  captured.files.clear();
  truncateSync(join(aggregate, '0008.stdout'), lastSize + 1);
  assert.equal(size(), total + 1);
  assert.throws(() => captureEvidence(aggregate, 'full-sdk-vv', context), /bounded unaliased evidence file/);
});

test('removing the real transcript digest guard is detected by the behavioral owning assertion', async t => {
  const source = temporary(t); vv(source); put(source, '0001.stdout', 'substituted bytes');
  const check = capture => assert.throws(() => capture(source, 'full-sdk-vv', context));
  check(captureEvidence);
  const original = readFileSync(new URL('./sdk-release-evidence.mjs', import.meta.url), 'utf8');
  const guard = "for (const stream of ['stdout', 'stderr']) assert.equal(hash(files.get(number + '.' + stream)), row[stream + '_sha256']);";
  assert.equal(original.split(guard).length, 2);
  const changed = original.replace(guard, '/* planted transcript check omission */')
    .replaceAll("from './sdk-vv-check.mjs'", `from '${new URL('./sdk-vv-check.mjs', import.meta.url)}'`)
    .replaceAll("from './sdk-vv-run.mjs'", `from '${new URL('./sdk-vv-run.mjs', import.meta.url)}'`)
    .replaceAll("from './product-sdk-check.mjs'", `from '${new URL('./product-sdk-check.mjs', import.meta.url)}'`)
    .replaceAll("from './sdk-bootstrap-retention.mjs'", `from '${new URL('./sdk-bootstrap-retention.mjs', import.meta.url)}'`)
    .replaceAll("from './release-gate-evidence.mjs'", `from '${new URL('./release-gate-evidence.mjs', import.meta.url)}'`);
  const path = join(temporary(t), 'mutant.mjs'); writeFileSync(path, changed);
  const mutant = await import(pathToFileURL(path));
  assert.throws(() => check(mutant.captureEvidence), /Missing expected exception/);
});
