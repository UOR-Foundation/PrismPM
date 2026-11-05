// Preserve original SDK gate evidence. This is not an acceptance authority.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {constants, closeSync, fstatSync, linkSync, lstatSync, mkdirSync, mkdtempSync, openSync,
  readSync, readdirSync, realpathSync, rmSync, writeFileSync} from 'node:fs';
import {dirname, join, resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {validateExecution} from './sdk-vv-check.mjs';
import {sourceAuthoringFiles} from './sdk-vv-run.mjs';
import {verifyResult as verifyProduct} from './product-sdk-check.mjs';
import {bootstrapNames, validateBootstrapRetention} from './sdk-bootstrap-retention.mjs';
import {releaseContext, validateSourceRun, validateNative} from './release-gate-evidence.mjs';

const MAX_FILE = 64 * 1024 * 1024, MAX_STDOUT = 256 * 1024 * 1024, MAX_TOTAL = 1024 * 1024 * 1024;
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const canonical = value => JSON.stringify(value, (_, item) => item && typeof item === 'object' && !Array.isArray(item)
  ? Object.fromEntries(Object.keys(item).sort().map(key => [key, item[key]])) : item);
const keys = (value, expected) => assert.deepEqual(Object.keys(value).sort(), expected.slice().sort());
const productFiles = ['source.json', 'image.json', 'inventory.json', 'standards.lock', 'prismpm.lock',
  'acquire.stdout.json', 'acquire.stderr.txt', 'acquisition.json', 'build.stdout.json',
  'build.stderr.txt', 'build.json', 'result.json', 'receiver.stderr.txt'].sort();
const vvFiles = ['acceptance.json', 'execution.json', 'run-1.json', 'run-2.json',
  'image-plan.json', 'loaded-identities.json', 'bootstrap.json',
  ...[1, 2].flatMap(run => bootstrapNames.map(name => `run-${run}-${name}`))];
const MAX_COMMANDS = 9999;
const MAX_VV_ENTRIES = vvFiles.length + MAX_COMMANDS * 3 + 2; // Excluded docker/ and the one closed source-authoring/ role.
const authoringLeaves = [...sourceAuthoringFiles, 'collection.json'].sort();
const authoringPaths = [1, 2].flatMap(run => authoringLeaves.map(name => `source-authoring/run-${run}/${name}`));
const identity = ['dev', 'ino', 'uid', 'gid', 'mode', 'nlink', 'size', 'mtimeNs', 'ctimeNs'];
const hex = value => assert.match(value, /^[0-9a-f]{64}$/);
const authoringJson = bytes => JSON.parse(new TextDecoder('utf-8', {fatal: true}).decode(bytes));

function validateAuthoring(files, run, context, execution, observed) {
  const prefix = `source-authoring/run-${run}/`, get = name => files.get(prefix + name);
  const collectionBytes = get('collection.json');
  assert(collectionBytes.length <= 32 * 1024 * 1024, 'bounded source authoring collection before decoding');
  const collection = authoringJson(collectionBytes);
  assert.deepEqual(collectionBytes, Buffer.from(canonical(collection)), 'original canonical authoring collection');
  keys(collection, ['schema', 'scope', 'run', 'execution_status', 'status', 'files']);
  assert.equal(collection.schema, 'prismpm/source-authoring-retention/1');
  assert.equal(collection.scope, 'diagnostic-retention-only');
  assert.equal(collection.run, run); assert.equal(collection.execution_status, 0); assert.equal(collection.status, 'retained');
  assert(Array.isArray(collection.files));
  assert.deepEqual(collection.files.map(row => row.path).sort(), [...sourceAuthoringFiles].sort(), 'complete original authoring files');
  let total = 0;
  for (const row of collection.files) {
    keys(row, ['path', 'byte_length', 'sha256', 'base64']);
    const bytes = get(row.path); assert(Buffer.isBuffer(bytes)); total += bytes.length;
    assert(total <= 16 * 1024 * 1024 && bytes.length <= 8 * 1024 * 1024, 'bounded original authoring bytes');
    assert.equal(row.byte_length, bytes.length); hex(row.sha256); assert.equal(row.sha256, hash(bytes));
    assert.equal(row.base64, bytes.toString('base64'), 'collection must retain exact original bytes');
  }
  assert.deepEqual(get('receipt.json'), get('authoring.stdout'));
  assert.equal(get('receipt.sha256').toString(), hash(get('receipt.json')) + '\n');
  for (const [name, status] of [['authoring', 0], ['inventory-path', 4], ['inventory-binary', 4]]) {
    const row = authoringJson(get(name + '.status.json'));
    keys(row, ['scope', 'status', 'signal', 'error']);
    assert.deepEqual(row, {scope: 'process-capture-only', status, signal: null, error: null});
    if (name !== 'authoring') {
      const failure = authoringJson(get(name + '.stdout'));
      assert.equal(failure.schema, 'prismpm/error-result/1'); assert.equal(failure.diagnostic.code, 'PP5401');
    }
  }
  assert.equal(get('authoring.stderr').length, 0);
  assert.equal(get('receipt.json').at(-1), 10, 'original LF-framed authoring receipt');
  const receipt = authoringJson(get('receipt.json'));
  keys(receipt, ['schema', 'status', 'scope', 'image', 'source_revision', 'platform', 'cli_sha256', 'inventory_sha256',
    'policy_sha256', 'manifest_sha256', 'source_artifact', 'lexlean', 'source_locks', 'tool_commands',
    'source_harness_sha256', 'harness_sha256', 'checks', 'output_failure', 'unclaimed']);
  assert.equal(receipt.schema, 'prismpm/source-authoring-sdk-check/1'); assert.equal(receipt.status, 'passed');
  assert.equal(receipt.scope, 'source-free-installed-authoring-component');
  assert.equal(receipt.image, context.image_reference); assert.equal(receipt.source_revision, context.source_revision);
  assert.equal(receipt.platform, 'linux/' + context.architecture);
  for (const [key, original] of [['cli_sha256', 'cli_sha256'], ['inventory_sha256', 'inventory_sha256'],
    ['policy_sha256', 'input_policy_sha256'], ['manifest_sha256', 'input_manifest_sha256']]) {
    hex(receipt[key]); assert.equal(receipt[key], execution[original]);
  }
  // Read the already-verified original SDK cat transcripts, never metadata
  // invented by the component receipt or by a new runtime acquisition.
  const input = (digest, path, maximum) => {
    const rows = observed.filter(row => row.status === 0 && row.stdout_sha256 === digest
      && canonical(row.arguments.slice(-2)) === canonical(['cat', path]));
    assert(rows.length > 0, 'missing original SDK metadata transcript ' + path);
    const bytes = files.get(rows[0].stdout_path); assert(bytes.length <= maximum);
    return authoringJson(bytes);
  };
  const inventory = input(receipt.inventory_sha256, '/opt/prismpm/share/inventory.json', 8 * 1024 * 1024);
  const policy = input(receipt.policy_sha256, '/opt/prismpm/share/vv-input-policy.json', 16384);
  const manifest = input(receipt.manifest_sha256, '/opt/prismpm/share/vv-inputs/manifest.json', 32 * 1024 * 1024);
  assert.equal(inventory.schema, 'prismpm/sdk-inventory/1');
  assert.equal(policy.source_revision, context.source_revision); assert.equal(manifest.source.revision, context.source_revision);
  assert.deepEqual(manifest.policy, policy);
  const artifact = id => {const rows = inventory.artifacts.filter(row => row.id === id); assert.equal(rows.length, 1); return rows[0];};
  assert.deepEqual(receipt.source_artifact, artifact('sdk-vv-source'));
  assert.equal(receipt.source_artifact.version, context.source_revision);
  const packed = manifest.artifacts.filter(row => row.path === 'source.pack'); assert.equal(packed.length, 1);
  assert.equal(receipt.source_artifact.digest, 'sha256:' + packed[0].sha256);
  assert.deepEqual(receipt.lexlean, artifact('lexlean'));
  assert.equal(artifact('prismpm').digest, 'sha256:' + receipt.cli_sha256);
  assert.equal(artifact('sdk-vv-policy').digest, 'sha256:' + receipt.policy_sha256);
  assert.equal(artifact('sdk-vv-manifest').digest, 'sha256:' + receipt.manifest_sha256);
  const sourceRow = path => {const rows = manifest.source.files.filter(row => row.path === path); assert.equal(rows.length, 1); hex(rows[0].sha256); return rows[0];};
  const locks = ['Cargo.lock', 'lexlean.lock', 'model/dependencies.toml', 'rust-toolchain.toml', 'lean-toolchain'];
  assert.deepEqual(receipt.source_locks, locks.map(path => ({path, sha256: sourceRow(path).sha256})));
  const harness = sourceRow('scripts/source-authoring-sdk-check.mjs').sha256;
  assert.equal(receipt.harness_sha256, harness); assert.equal(receipt.source_harness_sha256, harness);
  const tools = ['lexlean', 'cargo', 'rustc', 'lean', 'lake'].map(command => {
    const rows = inventory.commands.filter(row => row.command === command); assert.equal(rows.length, 1); return rows[0];
  });
  assert.deepEqual(receipt.tool_commands, tools);
  assert.equal(receipt.lexlean.digest, 'sha256:' + tools[0].sha256);
  assert(Array.isArray(receipt.checks) && receipt.checks.length === 36);
  for (const row of receipt.checks) {
    keys(row, ['operation', 'arguments', 'input_sha256', 'output_sha256', 'output_bytes', 'status', 'diagnostic']);
    assert(['read', 'write', 'to-notation'].includes(row.operation));
    assert(Array.isArray(row.arguments) && (row.arguments.length === 0 || (row.arguments.length === 2
      && ['--max-input-bytes', '--max-output-bytes'].includes(row.arguments[0]) && typeof row.arguments[1] === 'string'
      && /^[1-9][0-9]*$/.test(row.arguments[1]))));
    hex(row.input_sha256); hex(row.output_sha256); assert(Number.isSafeInteger(row.output_bytes) && row.output_bytes > 0 && row.output_bytes <= 8 * 1024 * 1024);
    assert.equal(row.status, new Map([[null, 0], ['PP5801', 2], ['PP5802', 1], ['PP5803', 14]]).get(row.diagnostic));
  }
  keys(receipt.output_failure, ['sink', 'status', 'diagnostic', 'disposition', 'stderr_sha256']);
  assert.equal(receipt.output_failure.sink, '/dev/full'); assert.equal(receipt.output_failure.status, 1);
  assert.equal(receipt.output_failure.diagnostic, 'PP5804'); assert.equal(receipt.output_failure.disposition, 'Unknown');
  hex(receipt.output_failure.stderr_sha256);
  assert.deepEqual(receipt.unclaimed, ['semantic-validity', 'linked-snapshot', 'product-targets', 'whole-sdk-qualification', 'release-acceptance']);
}

function regular(path, maximum) {
  const fd = openSync(path, constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK);
  try {
    const before = fstatSync(fd, {bigint: true});
    assert(before.isFile() && before.nlink === 1n && before.size <= BigInt(maximum), 'bounded unaliased evidence file');
    const bytes = Buffer.alloc(Number(before.size));
    for (let offset = 0; offset < bytes.length;) {
      const count = readSync(fd, bytes, offset, bytes.length - offset, null);
      assert(count > 0, 'evidence shortened'); offset += count;
    }
    assert.equal(readSync(fd, Buffer.alloc(1), 0, 1, null), 0, 'evidence grew');
    for (const after of [fstatSync(fd, {bigint: true}), lstatSync(path, {bigint: true})]) {
      assert(after.isFile() && !after.isSymbolicLink());
      for (const key of ['dev', 'ino', 'mode', 'nlink', 'size', 'mtimeNs', 'ctimeNs']) assert.equal(after[key], before[key], 'evidence changed');
    }
    return bytes;
  } finally { closeSync(fd); }
}

export function captureEvidence(directory, kind, context) {
  const originalGate = ['source-vv', 'native-equivalence'].includes(kind);
  keys(context, ['source_revision', 'image_reference', 'architecture', ...(originalGate ? ['run_id', 'run_attempt'] : [])]);
  assert.match(context.source_revision, /^[0-9a-f]{40}$/);
  if (kind !== 'source-vv' || context.image_reference !== null) assert.match(context.image_reference, /^ghcr\.io\/uor-foundation\/prismpm-sdk@sha256:[0-9a-f]{64}$/);
  assert(['amd64', 'arm64'].includes(context.architecture));
  assert(['full-sdk-vv', 'product-cli', 'source-vv', 'native-equivalence'].includes(kind));
  if (originalGate) releaseContext(context.source_revision, context.architecture, context.image_reference, context.run_id, context.run_attempt);
  directory = resolve(directory);
  assert.equal(realpathSync(directory), directory, 'evidence directory cannot be aliased');
  const directoryIdentity = lstatSync(directory, {bigint: true}); assert(directoryIdentity.isDirectory());
  const names = readdirSync(directory).sort();
  assert(names.length <= (kind === 'product-cli' ? productFiles.length : MAX_VV_ENTRIES), 'evidence file-count limit');
  const files = new Map(), directories = new Map(); let total = 0;
  const read = (name, maximum) => {
    const bytes = regular(join(directory, name), Math.min(maximum, MAX_TOTAL - total));
    total += bytes.length; assert(total <= MAX_TOTAL, 'complete evidence exceeds one GiB'); files.set(name, bytes);
  };
  const closedDirectory = (name, expected) => {
    const path = join(directory, name); assert.equal(realpathSync(path), path, 'nested evidence directory is aliased');
    const before = lstatSync(path, {bigint: true}); assert(before.isDirectory() && !before.isSymbolicLink());
    directories.set(path, before); assert.deepEqual(readdirSync(path).sort(), expected, 'closed source-authoring directory entries');
  };
  for (const name of names) {
    if (kind === 'full-sdk-vv' && name === 'docker') {
      const path = join(directory, name), stat = lstatSync(path);
      assert(stat.isDirectory() && !stat.isSymbolicLink());
      // Buildx also writes private transport configuration here. It is not
      // gate evidence; never inspect, copy or publish it (including credentials).
      continue;
    }
    if (kind === 'full-sdk-vv' && name === 'source-authoring') {
      closedDirectory(name, ['run-1', 'run-2']);
      for (const run of [1, 2]) {
        closedDirectory(`${name}/run-${run}`, authoringLeaves);
        for (const leaf of authoringLeaves) read(`${name}/run-${run}/${leaf}`, leaf === 'collection.json' ? 32 * 1024 * 1024 : 8 * 1024 * 1024);
      }
      continue;
    }
    assert(/^[a-zA-Z0-9][a-zA-Z0-9._-]{0,127}$/.test(name), 'closed flat evidence paths');
    // sdk-vv-check retains the genuine CLI binary in one raw stdout, whose
    // producer limit is 256 MiB. Do not truncate it to the metadata bound.
    const maximum = (kind === 'full-sdk-vv' && /^\d{4}\.stdout$/.test(name)) || (originalGate && name.endsWith('.stdout')) ? MAX_STDOUT : MAX_FILE;
    read(name, maximum);
  }
  for (const [path, before] of directories) {
    assert.equal(realpathSync(path), path);
    const after = lstatSync(path, {bigint: true}); assert(after.isDirectory() && !after.isSymbolicLink());
    for (const key of identity) assert.equal(after[key], before[key], 'nested evidence directory changed');
  }
  for (const key of ['dev', 'ino', 'mtimeNs', 'ctimeNs']) assert.equal(lstatSync(directory, {bigint: true})[key], directoryIdentity[key], 'evidence directory changed');
  const json = name => JSON.parse(files.get(name));
  if (kind === 'source-vv') {
    assert.deepEqual([...files.keys()], [1, 2].flatMap(run => [
      ...['json', 'stdout', 'stderr'].map(suffix => `source-${run}.${suffix}`),
      `source-${run}-vv.json`, `source-${run}-result.json`, ...bootstrapNames.map(name => `run-${run}-${name}`),
    ]).sort());
    for (const run of [1, 2]) validateSourceRun(files, run, context);
  } else if (kind === 'native-equivalence') {
    assert.deepEqual([...files.keys()], ['native-result.json', ...['native-check', 'container-check', 'native-error', 'container-error']
      .flatMap(prefix => ['json', 'stdout', 'stderr'].map(suffix => `${prefix}.${suffix}`))].sort());
    validateNative(files, context);
  } else if (kind === 'product-cli') {
    assert.deepEqual([...files.keys()], productFiles, 'complete product CLI evidence required');
    verifyProduct(json('result.json'), context.image_reference);
    assert.equal(json('source.json').revision, context.source_revision);
    assert.equal(json('prismpm.lock').sdk_image, context.image_reference);
    assert.equal(json('build.json').sdk_image, context.image_reference);
    const images = json('image.json'); assert(Array.isArray(images) && images.length === 1);
    assert.equal(images[0].Architecture, context.architecture); assert.equal(images[0].Os, 'linux');
    assert.equal(images[0].Config.Labels['org.opencontainers.image.revision'], context.source_revision);
  } else {
    for (const name of vvFiles) assert(files.has(name), 'missing original ' + name);
    const records = [...files.keys()].filter(name => /^\d{4}\.json$/.test(name));
    assert(records.length > 0 && records.length <= MAX_COMMANDS);
    const expected = [...vvFiles, ...authoringPaths], observed = [];
    for (let index = 0; index < records.length; index++) {
      const number = String(index + 1).padStart(4, '0');
      for (const suffix of ['json', 'stdout', 'stderr']) expected.push(number + '.' + suffix);
      const row = json(number + '.json');
      keys(row, ['arguments', 'status', 'signal', 'stdout_sha256', 'stderr_sha256']);
      assert(Array.isArray(row.arguments) && row.arguments.every(value => typeof value === 'string'));
      assert(Number.isInteger(row.status) && row.status >= 0 && row.status <= 255); assert.equal(row.signal, null);
      for (const stream of ['stdout', 'stderr']) assert.equal(hash(files.get(number + '.' + stream)), row[stream + '_sha256']);
      observed.push({...row, stdout_path: number + '.stdout'});
    }
    assert.deepEqual([...files.keys()], expected.sort(), 'complete original command transcript required');
    const execution = files.get('execution.json');
    validateExecution(execution, [files.get('run-1.json'), files.get('run-2.json')],
      context.image_reference, context.source_revision, context.architecture);
    for (const run of [1, 2]) validateAuthoring(files, run, context, JSON.parse(execution), observed);
    validateBootstrapRetention(files.get('bootstrap.json'), new Map([1, 2].flatMap(run => bootstrapNames.map(name => {
      const path = `run-${run}-${name}`; return [path, files.get(path)];
    }))), context.source_revision);
    const result = json('acceptance.json');
    assert.equal(result.schema, 'prismpm/sdk-installed-vv-check/1'); assert.equal(result.status, 'passed');
    assert.equal(result.scope, 'isolated-installed-two-run-vv');
    for (const key of Object.keys(context)) assert.equal(result[key], context[key]);
    assert.equal(result.execution_sha256, hash(execution));
    assert.deepEqual(result.phases, ['exact-native-images-acquired', 'external-network-disconnected',
      'isolated-native-sdk-probed', 'both-full-vv-records-verified', 'owned-resources-removed']);
    assert.deepEqual(result.unclaimed, ['hardware-attestation', 'sdk-release', 'product-readiness']);
  }
  return {files, manifest: {schema: 'prismpm/sdk-gate-evidence/1', scope: 'original-gate-evidence-only',
    kind, ...context, excluded: kind === 'full-sdk-vv' ? ['docker/'] : [],
    files: [...files].map(([path, bytes]) => ({path, size: bytes.length, sha256: hash(bytes)})),
    unclaimed: ['sdk-acceptance', 'crates.io-publication', 'ecosystem-acceptance', 'foundry-readiness', 'deployment']}};
}

export function packEvidence(directory, kind, output, context) {
  const captured = captureEvidence(directory, kind, context);
  output = resolve(output); const parent = dirname(output);
  assert.equal(realpathSync(parent), parent); assert(!lstatSync(output, {throwIfNoEntry: false}), 'fresh archive required');
  const stage = mkdtempSync(join(parent, '.sdk-evidence-'));
  try {
    const root = join(stage, 'evidence'); mkdirSync(root, {mode: 0o700});
    for (const [name, bytes] of captured.files) {
      mkdirSync(dirname(join(root, name)), {recursive: true, mode: 0o700});
      writeFileSync(join(root, name), bytes, {flag: 'wx', mode: 0o444});
    }
    writeFileSync(join(root, 'evidence-manifest.json'), canonical(captured.manifest) + '\n', {flag: 'wx', mode: 0o444});
    const archive = join(stage, 'evidence.tar');
    execFileSync('tar', ['--format=ustar', '--sort=name', '--mtime=@0', '--owner=0', '--group=0', '--numeric-owner',
      '--mode=0444', '-cf', archive, '-C', root, ...[...captured.files.keys(), 'evidence-manifest.json'].sort()],
    {timeout: 120000, maxBuffer: 1024 * 1024});
    // No caller-controlled tar input is executed or extracted by this command.
    linkSync(archive, output); // Atomic create-only publication; never overwrite.
    return captured.manifest;
  } finally { rmSync(stage, {recursive: true, force: true}); }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [kind, directory, output, image, source_revision, architecture, ...extra] = process.argv.slice(2);
  const original = ['source-vv', 'native-equivalence'].includes(kind);
  assert.equal(extra.length, original ? 2 : 0);
  packEvidence(directory, kind, output, {image_reference: image === '-' ? null : image, source_revision, architecture,
    ...(original ? {run_id: extra[0], run_attempt: extra[1]} : {})});
}
