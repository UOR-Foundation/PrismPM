// Evidence preservation tests, not execution or acceptance of an SDK image.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {createRequire} from 'node:module';
import {linkSync, lstatSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, symlinkSync, truncateSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {pathToFileURL} from 'node:url';
import test from 'node:test';
import {assembleInstalled, captureEvidence, packEvidence} from './sdk-release-evidence.mjs';
import {assetNames} from './release-phases.mjs';
import {parts as vvParts} from './vv-assemble.mjs';
import {bootstrapFixture} from './sdk-bootstrap-retention-fixture.mjs';
const require = createRequire('/opt/prismpm/oracles/package.json');
const {load} = require('js-yaml');
const canonical = value => JSON.stringify(value, (_, item) => item && typeof item === 'object' && !Array.isArray(item)
  ? Object.fromEntries(Object.keys(item).sort().map(key => [key, item[key]])) : item);
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const context = {source_revision: 'a'.repeat(40), architecture: 'amd64',
  image_reference: 'ghcr.io/uor-foundation/prismpm-sdk@sha256:' + 'b'.repeat(64)};
const partContext = {...context, part: 'source'}, baseContext = context;
const temporary = t => {const root = mkdtempSync(join(tmpdir(), 'prismpm-evidence-test-')); t.after(() => rmSync(root, {recursive: true, force: true})); return root;};
const put = (root, name, value) => writeFileSync(join(root, name), typeof value === 'string' ? value : canonical(value));
function vv(root, part = 'source') {
  const bootstrap = bootstrapFixture(context.source_revision);
  if (part === 'source') for (const [name, bytes] of [...bootstrap.retained, ['bootstrap.json', bootstrap.manifest]]) writeFileSync(join(root, name), bytes);
  mkdirSync(join(root, 'docker'));
  put(join(root, 'docker'), 'config.json', 'private transport data that must never ship');
  const raw = canonical({commit: context.source_revision, part, schema: 'prismpm/vv-part/1', status: 'passed'});
  put(root, 'vv-part.json', raw);
  const execution = canonical({schema: 'prismpm/sdk-vv-execution/2', scope: 'one-vv-part-execution-only', part,
    source_revision: context.source_revision, image_reference: context.image_reference, process_architecture: 'x64',
    inventory_sha256: 'c'.repeat(64), cli_sha256: 'd'.repeat(64), input_policy_sha256: 'e'.repeat(64),
    input_manifest_sha256: 'f'.repeat(64), advisory_revision: '1'.repeat(40),
    run: {part, path: 'part/vv-part.json', byte_length: raw.length, sha256: sha(raw)},
    unclaimed: ['native-host', 'network-isolation', 'oci-image-identity', 'sdk-release', 'product-readiness']});
  put(root, 'execution.json', execution);
  put(root, 'acceptance.json', {schema: 'prismpm/sdk-installed-vv-check/2', status: 'passed', scope: 'isolated-installed-vv-part', part,
    ...context, execution_sha256: sha(execution), phases: ['exact-native-images-acquired', 'external-network-disconnected',
      'isolated-native-sdk-probed', 'vv-part-record-verified', 'owned-resources-removed'],
    unclaimed: ['hardware-attestation', 'sdk-release', 'product-readiness']});
  for (const name of ['image-plan.json', 'loaded-identities.json']) put(root, name, {fixture: 'transport-only'});
  put(root, '0001.stdout', 'unit transcript\n'); put(root, '0001.stderr', '');
  put(root, '0001.json', {arguments: ['info'], status: 0, signal: null,
    stdout_sha256: sha('unit transcript\n'), stderr_sha256: sha('')});
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

function validateWorkflow(workflow) {
  const upload = workflow.jobs['browser-library'].steps.filter(step => step.with?.name === 'library-sdk-sdk-${{ matrix.arch }}');
  assert.equal(upload.length, 1);
  assert.equal(upload[0].uses, 'actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02');
  assert.equal(upload[0].if, 'always()');
  assert.equal(upload[0]['continue-on-error'], undefined);
  assert.deepEqual(upload[0].with, {name: 'library-sdk-sdk-${{ matrix.arch }}',
    path: 'sdk-${{ matrix.arch }}-library-sdk.log\nroot-a/target/library-sdk-evidence/\n',
    'if-no-files-found': 'error', 'retention-days': 30});
  assert(assetNames().includes('sdk-metadata-native-join.json'));
  const steps = workflow.jobs['oci-native'].steps;
  const pack = steps.find(step => step.run?.includes('scripts/sdk-release-evidence.mjs'));
  assert(pack && pack.if === undefined && pack['continue-on-error'] === undefined);
  semanticPrerequisite(workflow.jobs['oci-native'], [pack], undefined);
  const download = (name, path) => {
    const matching = steps.filter(step => step.with?.name === name);
    assert.equal(matching.length, 1, 'missing or duplicate gate download ' + name);
    const step = matching[0];
    assert.equal(step.uses, 'actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093');
    assert.deepEqual(step.with, {name, path});
    assert.equal(step.if, undefined); assert.equal(step['continue-on-error'], undefined);
    assert(steps.indexOf(step) < steps.indexOf(pack), 'evidence must arrive before capture');
    return step;
  };
  const sdkImage = download('sdk-image', 'target/release-sdk');
  const selected = (id, arch, owner, path) => {
    const selectors = steps.filter(step => step.id === id);
    assert.equal(selectors.length, 1, 'missing or duplicate immutable selector ' + id);
    const selector = selectors[0];
    assert.equal(selector.if, undefined); assert.equal(selector['continue-on-error'], undefined);
    assert.equal(selector.env.GH_TOKEN, '${{ github.token }}');
    assert(steps.indexOf(sdkImage) < steps.indexOf(selector), 'exact SDK identity must arrive before artifact selection');
    assert(selector.run.startsWith('set -euo pipefail\n'));
    assert(selector.run.includes('scripts/release-artifact-metadata.mjs'));
    assert(selector.run.includes('scripts/release-gate-evidence.mjs select-artifacts'));
    assert(selector.run.includes(`"$GITHUB_SHA" ${arch} "$image" "$GITHUB_RUN_ID" "$GITHUB_RUN_ATTEMPT" ${owner}`));
    const matching = steps.filter(step => step.with?.['artifact-ids'] === '${{ steps.' + id + '.outputs.ids }}');
    assert.equal(matching.length, 1, 'missing or duplicate immutable evidence download ' + id);
    const step = matching[0];
    assert.equal(step.uses, 'actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093');
    assert.deepEqual(step.with, {'artifact-ids': '${{ steps.' + id + '.outputs.ids }}', 'merge-multiple': true, path});
    assert.equal(step.if, undefined); assert.equal(step['continue-on-error'], undefined);
    assert(steps.indexOf(selector) < steps.indexOf(step));
    assert(steps.indexOf(step) < steps.indexOf(pack), 'evidence must arrive before capture');
  };
  assert.equal(steps.filter(step => step.with?.pattern === 'full-sdk-vv-sdk-*').length, 0);
  for (const arch of ['amd64', 'arm64']) {
    selected(`full-${arch}`, arch, `$(node scripts/vv-assemble.mjs --list | sed "s/[^[:space:]]\\+/full-sdk-vv-sdk-${arch}-&/g")`, '.sdk-gate-evidence/full-sdk-vv');
    selected(`native-${arch}`, arch, `native-${arch === 'amd64' ? 'x86_64' : 'aarch64'}-unknown-linux-gnu`, 'release');
    download(`product-cli-sdk-${arch}`, `.sdk-gate-evidence/product-cli-sdk-${arch}`);
    assert(assetNames().includes(`sdk-${arch}-product-cli.tar`));
    assert(assetNames().includes(`sdk-${arch}-vv-evidence.json`));
    for (const part of vvParts) assert(assetNames().includes(`sdk-${arch}-full-sdk-vv-${part}.tar`));
    download(`browser-sdk-sdk-${arch}`, 'release');
    download(`library-sdk-sdk-${arch}`, `.sdk-gate-evidence/library-sdk-sdk-${arch}`);
    for (const kind of ['browser', 'library']) assert(assetNames().includes(`sdk-${arch}-${kind}-sdk.log`));
    assert(assetNames().includes(`sdk-${arch}-native-metadata.tar`));
    const selectors = steps.filter(step => step.id === `binary-${arch}`);
    assert.equal(selectors.length, 1, 'one Binary provider-backed selector per architecture');
    const selector = selectors[0];
    assert.equal(selector.if, undefined); assert.equal(selector['continue-on-error'], undefined);
    assert.deepEqual(selector.env, {GH_TOKEN: '${{ github.token }}'});
    assert.equal(selector.run, [
      'set -euo pipefail', 'mkdir -p target/release-selection',
      `node scripts/release-artifact-metadata.mjs target/release-selection/binary-${arch}-provider.json "$GITHUB_RUN_ID"`,
      'image=$(cat target/release-sdk/sdk-image.txt)',
      `node scripts/release-gate-evidence.mjs select-artifacts target/release-selection/binary-${arch}-provider.json target/release-selection/binary-${arch}.json \\`,
      `  "$GITHUB_SHA" ${arch} "$image" "$GITHUB_RUN_ID" "$GITHUB_RUN_ATTEMPT" binary-sdk-sdk-${arch} binary-language-1.2-sdk-sdk-${arch}`,
      `language11=$(node -e 'const fs=require("node:fs"); console.log(JSON.parse(fs.readFileSync(process.argv[1])).artifacts[0].id)' target/release-selection/binary-${arch}.json)`,
      `language12=$(node -e 'const fs=require("node:fs"); console.log(JSON.parse(fs.readFileSync(process.argv[1])).artifacts[1].id)' target/release-selection/binary-${arch}.json)`,
      'printf \'language11=%s\\nlanguage12=%s\\n\' "$language11" "$language12" >> "$GITHUB_OUTPUT"', '',
    ].join('\n'));
    assert(steps.indexOf(sdkImage) < steps.indexOf(selector));
    for (const [name, language, output] of [[`binary-sdk-sdk-${arch}`, '1.1', 'language11'], [`binary-language-1.2-sdk-sdk-${arch}`, '1.2', 'language12']]) {
      const ids = '${{ steps.binary-' + arch + '.outputs.' + output + ' }}';
      const matches = steps.filter(step => step.with?.['artifact-ids'] === ids);
      assert.equal(matches.length, 1, 'one immutable Binary download per architecture and language');
      const step = matches[0];
      assert.equal(step.uses, 'actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093');
      assert.deepEqual(step.with, {'artifact-ids': ids, 'merge-multiple': true, path: `.sdk-gate-evidence/${name}`});
      assert.equal(step.if, undefined); assert.equal(step['continue-on-error'], undefined);
      assert(steps.indexOf(selector) < steps.indexOf(step));
      assert(steps.indexOf(step) < steps.indexOf(pack));
      const languageSuffix = language === '1.1' ? '' : '-language-1.2';
      assert(assetNames().includes(`sdk-${arch}-binary-sdk${languageSuffix}.tar`));
    }
  }
  assert.equal(steps.filter(step => step.uses?.startsWith('actions/download-artifact@')
    && /^\.sdk-gate-evidence\/binary(?:-sdk|-language-1\.2-sdk)-sdk-/.test(step.with?.path ?? '')).length, 4,
  'exactly four Binary transport downloads, with no later named download overlay');
  assert.deepEqual(assetNames().filter(name => /^sdk-(amd64|arm64)-binary-sdk(?:-language-1\.2)?\.tar$/.test(name)).sort(),
    ['amd64', 'arm64'].flatMap(arch => [`sdk-${arch}-binary-sdk.tar`, `sdk-${arch}-binary-sdk-language-1.2.tar`]).sort());
  const nativeOwners = workflow.jobs['browser-library'];
  const binary = nativeOwners.steps.find(step => step.run?.includes('binary-sdk-check.sh'));
  assert(binary && binary.if === undefined && binary['continue-on-error'] === undefined);
  semanticPrerequisite(nativeOwners, [binary], 'root-a');
  assert.equal(nativeOwners.name, 'browser-library-${{ matrix.arch }}');
  const naming = nativeOwners.steps.filter(step => step.id === 'binary-artifacts');
  assert.equal(naming.length, 1);
  assert.equal(naming[0].if, undefined); assert.equal(naming[0]['continue-on-error'], undefined);
  assert.equal(naming[0].shell, 'bash');
  assert.deepEqual(naming[0].env, {ARCH: '${{ matrix.arch }}'});
  assert.equal(naming[0].run, [
    'set -euo pipefail', 'image=$(cat .shipped-image/sdk-image.txt)',
    'language11=$(node root-a/scripts/release-gate-evidence.mjs artifact-name "binary-sdk-sdk-$ARCH" "$GITHUB_SHA" "$ARCH" "$image" "$GITHUB_RUN_ID" "$GITHUB_RUN_ATTEMPT")',
    'language12=$(node root-a/scripts/release-gate-evidence.mjs artifact-name "binary-language-1.2-sdk-sdk-$ARCH" "$GITHUB_SHA" "$ARCH" "$image" "$GITHUB_RUN_ID" "$GITHUB_RUN_ATTEMPT")',
    'printf \'language11=%s\\nlanguage12=%s\\n\' "$language11" "$language12" >> "$GITHUB_OUTPUT"', '',
  ].join('\n'));
  assert(nativeOwners.steps.indexOf(naming[0]) < nativeOwners.steps.indexOf(binary));
  for (const [language, output] of [['1.1', 'language11'], ['1.2', 'language12']]) {
    const name = '${{ steps.binary-artifacts.outputs.' + output + ' }}';
    const uploads = nativeOwners.steps.filter(step => step.with?.name === name);
    assert.equal(uploads.length, 1);
    const upload = uploads[0];
    assert.equal(upload.if, 'always()'); assert.equal(upload['continue-on-error'], undefined);
    assert.equal(upload.uses, 'actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02');
    assert.deepEqual(upload.with, {name, path: '${{ runner.temp }}/binary-transport-' + language + '/',
      'include-hidden-files': true, 'if-no-files-found': 'error', overwrite: false, 'retention-days': 30});
    assert(nativeOwners.steps.indexOf(binary) < nativeOwners.steps.indexOf(upload));
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
    '  release/source-vv.tar "$image" "$GITHUB_SHA" amd64 "$GITHUB_RUN_ID" "$GITHUB_RUN_ATTEMPT"',
    'parts=$(node scripts/vv-assemble.mjs --list)',
    'for architecture in amd64 arm64; do',
    '  node scripts/library-sdk-metadata-evidence.mjs pack \\',
    '    ".sdk-gate-evidence/library-sdk-sdk-$architecture/root-a/target/library-sdk-evidence/linux-$architecture" \\',
    '    "release/sdk-$architecture-native-metadata.tar" . "$image" "$GITHUB_SHA" "$architecture"',
    '  node scripts/binary-sdk-release-evidence.mjs pack \\',
    '    ".sdk-gate-evidence/binary-sdk-sdk-$architecture" "release/sdk-$architecture-binary-sdk.tar" \\',
    '    "$image" "$GITHUB_SHA" "$architecture" 1.1 "$GITHUB_RUN_ID" "$GITHUB_RUN_ATTEMPT" \\',
    '    "target/release-selection/binary-$architecture-provider.json" "target/release-selection/binary-$architecture.json"',
    '  node scripts/binary-sdk-release-evidence.mjs pack \\',
    '    ".sdk-gate-evidence/binary-language-1.2-sdk-sdk-$architecture" "release/sdk-$architecture-binary-sdk-language-1.2.tar" \\',
    '    "$image" "$GITHUB_SHA" "$architecture" 1.2 "$GITHUB_RUN_ID" "$GITHUB_RUN_ATTEMPT" \\',
    '    "target/release-selection/binary-$architecture-provider.json" "target/release-selection/binary-$architecture.json"',
    '  for part in $parts; do',
    '    node scripts/sdk-release-evidence.mjs full-sdk-vv \\',
    '      ".sdk-gate-evidence/full-sdk-vv/sdk-$architecture-full-sdk-vv-$part" "release/sdk-$architecture-full-sdk-vv-$part.tar" \\',
    '      "$image" "$GITHUB_SHA" "$architecture" "$part"', '  done',
    '  node scripts/sdk-release-evidence.mjs product-cli \\',
    '    ".sdk-gate-evidence/product-cli-sdk-$architecture" "release/sdk-$architecture-product-cli.tar" \\',
    '    "$image" "$GITHUB_SHA" "$architecture"',
    '  node scripts/sdk-release-evidence.mjs native-equivalence \\',
    '    ".sdk-gate-evidence/native-equivalence-sdk-$architecture" "release/sdk-$architecture-native-equivalence.tar" \\',
    '    "$image" "$GITHUB_SHA" "$architecture" "$GITHUB_RUN_ID" "$GITHUB_RUN_ATTEMPT"',
    'done', '',
  ].join('\n'));
  selected('source-vv', 'amd64', 'source-vv', '.sdk-gate-evidence/source-vv');
  for (const arch of ['amd64', 'arm64']) selected(`sdk-equivalence-${arch}`, arch, `sdk-equivalence-${arch}`, `.sdk-gate-evidence/native-equivalence-sdk-${arch}`);
  assert(steps.indexOf(pack) < steps.findIndex(step => step.run?.includes('checksums=$(mktemp)')));
  assert.deepEqual(workflow.jobs['oci-native'].needs, ['gate', 'images', 'native', 'reproducibility', 'installed-sdk']);
  assert(workflow.jobs.release.needs.includes('crates'));
}

test('an architecture counts only with every validated part', t => {
  const root = temporary(t), output = temporary(t);
  for (const part of vvParts) {
    const directory = join(root, `sdk-amd64-full-sdk-vv-${part}`); mkdirSync(directory); vv(directory, part);
  }
  assert.throws(() => assembleInstalled(root, join(output, 'wrong.json'), {...context, architecture: 'arm64'}));
  assembleInstalled(root, join(output, 'good.json'), context);
  assert.deepEqual(JSON.parse(readFileSync(join(output, 'good.json'))), {commit: context.source_revision,
    gates: Array.from({length: 15}, (_, index) => index + 1), schema: 'prismpm/vv-evidence/1', status: 'passed'});
  rmSync(join(root, 'sdk-amd64-full-sdk-vv-dk-2'), {recursive: true});
  assert.throws(() => assembleInstalled(root, join(output, 'partial.json'), context));
  assert.equal(readdirSync(output).length, 1);
});

test('OCI publication retains both native gate closures without granting SDK acceptance', () => {
  const workflow = load(readFileSync(new URL('../.github/workflows/release.yml', import.meta.url), 'utf8'));
  validateWorkflow(workflow);
  const steps = value => value.jobs['oci-native'].steps;
  const pack = value => steps(value).find(step => step.run?.includes('scripts/sdk-release-evidence.mjs'));
  const gate = value => steps(value).find(step => step.with?.['artifact-ids'] === '${{ steps.full-amd64.outputs.ids }}');
  for (const mutate of [
    value => {steps(value).splice(steps(value).findIndex(step => step.with?.name === 'sdk-image'), 1);},
    value => steps(value).splice(steps(value).indexOf(gate(value)), 1),
    value => steps(value).push(structuredClone(gate(value))),
    value => {gate(value).with.path = 'release';},
    value => {gate(value).uses = 'actions/download-artifact@main';},
    value => {gate(value).if = 'always()';},
    value => {gate(value)['continue-on-error'] = true;},
    value => {const record = gate(value); steps(value).splice(steps(value).indexOf(record), 1); steps(value).push(record);},
    value => {steps(value).splice(steps(value).findIndex(step => step.with?.['artifact-ids'] === '${{ steps.binary-amd64.outputs.language11 }}'), 1);},
    value => {steps(value).splice(steps(value).findIndex(step => step.with?.['artifact-ids'] === '${{ steps.binary-arm64.outputs.language12 }}'), 1);},
    ...['amd64', 'arm64'].flatMap(arch => [
      value => {steps(value).find(step => step.id === `binary-${arch}`).if = 'always()';},
      value => {steps(value).find(step => step.id === `binary-${arch}`)['continue-on-error'] = true;},
      value => {steps(value).find(step => step.id === `binary-${arch}`).env.GH_TOKEN = '';},
      ...[
        run => run.replace('scripts/release-artifact-metadata.mjs', 'scripts/omitted-provider.mjs'),
        run => run.replace('select-artifacts', 'artifact-name'),
        run => run.replace('"$GITHUB_RUN_ATTEMPT"', '1'),
        run => run.replace('artifacts[0].id', 'artifacts[1].id'),
        run => run.replace('-provider.json', '-ambient.json'),
      ].map(rewrite => value => {const step = steps(value).find(step => step.id === `binary-${arch}`); step.run = rewrite(step.run);}),
      ...['language11', 'language12'].flatMap(output => [
        value => {const step = steps(value).find(step => step.with?.['artifact-ids'] === '${{ steps.binary-' + arch + '.outputs.' + output + ' }}'); step.if = 'always()';},
        value => {const step = steps(value).find(step => step.with?.['artifact-ids'] === '${{ steps.binary-' + arch + '.outputs.' + output + ' }}'); step.with.name = 'mutable-binary-name'; delete step.with['artifact-ids'];},
        value => {const step = steps(value).find(step => step.with?.['artifact-ids'] === '${{ steps.binary-' + arch + '.outputs.' + output + ' }}'); step.with['merge-multiple'] = false;},
        value => {const step = steps(value).find(step => step.with?.['artifact-ids'] === '${{ steps.binary-' + arch + '.outputs.' + output + ' }}'); steps(value).splice(steps(value).indexOf(step), 1); steps(value).push(step);},
      ]),
    ]),
    value => {steps(value).splice(steps(value).indexOf(pack(value)), 0, {
      uses: 'actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093',
      with: {name: 'mutable-overlay', path: '.sdk-gate-evidence/binary-sdk-sdk-amd64'},
    });},
    value => {delete value.jobs['browser-library'].name;},
    value => {value.jobs['browser-library'].steps.find(step => step.id === 'binary-artifacts').run = 'echo unbound';},
    value => {const step = value.jobs['browser-library'].steps.find(step => step.id === 'binary-artifacts'); step.run = step.run.replace('"$GITHUB_RUN_ATTEMPT"', '1');},
    ...['language11', 'language12'].flatMap(output => [
      value => {value.jobs['browser-library'].steps.find(step => step.with?.name === '${{ steps.binary-artifacts.outputs.' + output + ' }}').with.overwrite = true;},
      value => {value.jobs['browser-library'].steps.find(step => step.with?.name === '${{ steps.binary-artifacts.outputs.' + output + ' }}').with.path += '*.log';},
      value => {value.jobs['browser-library'].steps.find(step => step.with?.name === '${{ steps.binary-artifacts.outputs.' + output + ' }}').with['if-no-files-found'] = 'ignore';},
    ]),
    ...[
      run => run.replace('binary-sdk-release-evidence.mjs pack', 'binary-sdk-release-evidence.mjs'),
      run => run.replace('1.1 "$GITHUB_RUN_ID" "$GITHUB_RUN_ATTEMPT"', '1.1 "$GITHUB_RUN_ID" 1'),
      run => run.replace('binary-$architecture-provider.json', 'binary-$architecture.json'),
      run => run.replace('"target/release-selection/binary-$architecture.json"', '"unverified-selection.json"'),
    ].map(rewrite => value => {pack(value).run = rewrite(pack(value).run);}),
    value => {pack(value).run = pack(value).run.replace('binary-sdk-release-evidence.mjs', 'omitted-binary-reader.mjs');},
    value => {pack(value).run = pack(value).run.replace('"$architecture" 1.2', '"$architecture" 1.1');},
    value => {steps(value).find(step => step.run?.includes('semantic-source-bootstrap.sh')).if = 'false';},
    value => {const helper = steps(value).find(step => step.run?.includes('semantic-source-bootstrap.sh'));
      steps(value).splice(steps(value).indexOf(helper), 1); steps(value).push(helper);},
    value => {value.jobs['browser-library'].steps.find(step => step.with?.name === '${{ steps.binary-artifacts.outputs.language11 }}').with.path = 'sdk-${{ matrix.arch }}-binary-sdk.log';},
    value => {value.jobs['browser-library'].steps.find(step => step.with?.name === '${{ steps.binary-artifacts.outputs.language12 }}').with['include-hidden-files'] = false;},
    ...['browser-library', 'oci-native'].flatMap(name => semanticReadinessMutations.map(rewrite => value => {
      const step = value.jobs[name].steps.find(candidate => candidate.run?.includes('semantic-source-bootstrap.sh'));
      const changed = rewrite(step.run);
      assert.notEqual(changed, step.run, 'readiness mutation must change the actual prerequisite');
      step.run = changed;
    })),
    value => {pack(value).if = 'false';},
    value => {pack(value)['continue-on-error'] = true;},
    value => {pack(value).run = pack(value).run.replace('set -euo pipefail', 'set -uo pipefail');},
    value => {pack(value).run = pack(value).run.replace('amd64 arm64', 'amd64');},
    value => {pack(value).run = pack(value).run.replace('library-sdk-metadata-evidence.mjs join', 'library-sdk-metadata-evidence.mjs omitted');},
    value => {pack(value).run = pack(value).run.replace('library-sdk-metadata-evidence.mjs pack', 'library-sdk-metadata-evidence.mjs omitted');},
    value => {value.jobs['browser-library'].steps.find(step => step.with?.name === 'library-sdk-sdk-${{ matrix.arch }}').with.path = 'sdk-${{ matrix.arch }}-library-sdk.log';},
    value => {value.jobs['browser-library'].steps.find(step => step.with?.name === 'library-sdk-sdk-${{ matrix.arch }}').with.path = 'root-a/target/library-sdk-evidence/';},
    value => {value.jobs['browser-library'].steps.find(step => step.with?.name === 'library-sdk-sdk-${{ matrix.arch }}').with['if-no-files-found'] = 'ignore';},
    value => {steps(value).find(step => step.id === 'full-amd64').run = 'echo unverified';},
    value => {steps(value).find(step => step.id === 'full-amd64').run = steps(value).find(step => step.id === 'full-amd64').run.replace('"$GITHUB_RUN_ID" "$GITHUB_RUN_ATTEMPT"', '"124" "3"');},
    value => {pack(value).run = pack(value).run.replace('"$GITHUB_SHA"', '"stale"');},
    value => {pack(value).run = pack(value).run.replace('"$architecture"', '"$architecture" || true');},
    value => {const record = pack(value); steps(value).splice(steps(value).indexOf(record), 1); steps(value).push(record);},
    value => {value.jobs['oci-native'].needs.pop();},
    value => {value.jobs.release.needs = value.jobs.release.needs.filter(name => name !== 'crates');},
  ]) {
    const changed = structuredClone(workflow); mutate(changed);
    assert.notDeepEqual(changed, workflow, 'mutation must change the workflow: ' + mutate.toString());
    assert.throws(() => validateWorkflow(changed), undefined, mutate.toString());
  }
});

test('real deterministic USTAR archives retain every original byte and explicitly exclude private transport configuration', t => {
  for (const [kind, fixture, context] of [['full-sdk-vv', vv, partContext], ['product-cli', product, baseContext]]) {
    const source = temporary(t), output = temporary(t); fixture(source);
    const captured = captureEvidence(source, kind, context);
    const first = join(output, 'one.tar'), second = join(output, 'two.tar');
    packEvidence(source, kind, first, context); packEvidence(source, kind, second, context);
    assert.deepEqual(readFileSync(first), readFileSync(second));
    assert.throws(() => packEvidence(source, kind, first, context), /fresh archive/);
    const extracted = join(output, 'extracted'); mkdirSync(extracted);
    execFileSync('tar', ['-xf', first, '-C', extracted]);
    assert.deepEqual(readdirSync(extracted).sort(), [...captured.files.keys(), 'evidence-manifest.json'].sort());
    for (const [name, bytes] of captured.files) assert.deepEqual(readFileSync(join(extracted, name)), bytes);
    assert.deepEqual(JSON.parse(readFileSync(join(extracted, 'evidence-manifest.json'))), captured.manifest);
    assert.equal(captured.manifest.scope, 'original-gate-evidence-only');
    assert(captured.manifest.unclaimed.includes('sdk-acceptance'));
    assert(!readFileSync(first).includes(Buffer.from('private transport data')));
  }
});

test('the exact 9999-command file closure includes its excluded private directory and rejects one extra entry', {timeout: 60000}, t => {
  const source = temporary(t); vv(source);
  const empty = sha('');
  for (let index = 2; index <= 9999; index++) {
    const number = String(index).padStart(4, '0');
    put(source, number + '.stdout', ''); put(source, number + '.stderr', '');
    put(source, number + '.json', {arguments: ['info'], status: 0, signal: null,
      stdout_sha256: empty, stderr_sha256: empty});
  }
  assert.equal(readdirSync(source).length, 30008);
  const captured = captureEvidence(source, 'full-sdk-vv', partContext);
  assert.equal(captured.files.size, 30007);
  assert.equal(captured.manifest.files.length, 30007);
  assert.deepEqual(captured.manifest.excluded, ['docker/']);
  assert(!captured.files.has('docker'));
  assert.equal(readFileSync(join(source, 'docker', 'config.json'), 'utf8'), 'private transport data that must never ship');
  put(source, '10000.json', '{}');
  assert.equal(readdirSync(source).length, 30009);
  assert.throws(() => captureEvidence(source, 'full-sdk-vv', partContext), /evidence file-count limit/);
});

test('missing, extra, stale and changed original evidence fails before archive creation', t => {
  for (const change of [
    root => rmSync(join(root, 'vv-part.json')),
    root => rmSync(join(root, 'run-1-bootstrap-evidence.json')),
    root => put(root, 'run-1-bootstrap-current-capture.json', '{}'),
    root => put(root, 'extra.json', {}),
    root => put(root, '0001.stdout', 'changed'),
    root => put(root, 'vv-part.json', {status: 'passed'}),
    root => {const value = JSON.parse(readFileSync(join(root, 'acceptance.json'))); value.source_revision = '2'.repeat(40); put(root, 'acceptance.json', value);},
    root => {const value = JSON.parse(readFileSync(join(root, 'acceptance.json'))); value.phases.pop(); put(root, 'acceptance.json', value);},
    root => {const value = JSON.parse(readFileSync(join(root, 'execution.json'))); value.part = 'dk-2'; put(root, 'execution.json', value);},
  ]) {
    const source = temporary(t), output = temporary(t); vv(source); change(source);
    assert.throws(() => packEvidence(source, 'full-sdk-vv', join(output, 'bad.tar'), partContext));
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
  ]) {const source = temporary(t); vv(source); change(source); assert.throws(() => captureEvidence(source, 'full-sdk-vv', partContext));}
  const source = temporary(t); vv(source); const parent = temporary(t), alias = join(parent, 'alias'); symlinkSync(source, alias);
  assert.throws(() => captureEvidence(alias, 'full-sdk-vv', partContext));
});

test('actual maximum file archive and one-GiB captured closure retain exact bytes without clamping', {timeout: 120000}, t => {
  const maximum = 268435456, zero = Buffer.alloc(maximum), fullHash = sha(zero);
  const row = digest => ({arguments: ['info'], status: 0, signal: null, stdout_sha256: digest, stderr_sha256: sha('')});
  const source = temporary(t); vv(source);
  truncateSync(join(source, '0001.stdout'), 0); truncateSync(join(source, '0001.stdout'), maximum);
  put(source, '0001.json', row(fullHash));
  const output = join(temporary(t), 'maximum.tar');
  const manifest = packEvidence(source, 'full-sdk-vv', output, partContext);
  assert.equal(manifest.files.find(file => file.path === '0001.stdout').size, maximum);
  const restored = execFileSync('tar', ['-xOf', output, '0001.stdout'], {maxBuffer: maximum + 1024, timeout: 30000});
  assert.equal(restored.length, maximum); assert.equal(sha(restored), fullHash);

  // Sparse original stdout files exercise the complete allocation/read/hash
  // boundary without reserving one GiB of persistent development disk space.
  const aggregate = temporary(t); vv(aggregate);
  for (let index = 1; index <= 4; index++) {
    const number = String(index).padStart(4, '0');
    put(aggregate, number + '.stdout', ''); truncateSync(join(aggregate, number + '.stdout'), maximum);
    put(aggregate, number + '.stderr', ''); put(aggregate, number + '.json', row(fullHash));
  }
  const size = () => readdirSync(aggregate).filter(name => name !== 'docker')
    .reduce((sum, name) => sum + lstatSync(join(aggregate, name)).size, 0);
  const total = 1024 ** 3, lastSize = maximum - (size() - total);
  assert(lastSize > 0 && lastSize < maximum);
  truncateSync(join(aggregate, '0004.stdout'), lastSize);
  put(aggregate, '0004.json', row(sha(zero.subarray(0, lastSize))));
  assert.equal(size(), total);
  const captured = captureEvidence(aggregate, 'full-sdk-vv', partContext);
  assert.equal(captured.manifest.files.reduce((sum, file) => sum + file.size, 0), total);
  captured.files.clear();
  truncateSync(join(aggregate, '0004.stdout'), lastSize + 1);
  assert.equal(size(), total + 1);
  assert.throws(() => captureEvidence(aggregate, 'full-sdk-vv', partContext), /bounded unaliased evidence file/);
});

test('removing the real transcript digest guard is detected by the behavioral owning assertion', async t => {
  const source = temporary(t); vv(source); put(source, '0001.stdout', 'substituted bytes');
  const check = capture => assert.throws(() => capture(source, 'full-sdk-vv', partContext));
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
