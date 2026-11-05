// Installed source-authoring component evidence. This harness never invokes
// Cargo, the internal semantic-source helper, or an external LexLean process.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {closeSync, mkdtempSync, openSync, readFileSync, readSync, readdirSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';

const requestSchema = 'prismpm/source-authoring-request/1';
const resultSchema = 'prismpm/source-authoring-result/1';
const executable = '/usr/local/bin/prismpm';
export const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
function hashFile(path) {
  const fd = openSync(path, 'r'), hash = createHash('sha256'), buffer = Buffer.alloc(64 * 1024);
  try {
    for (let count; (count = readSync(fd, buffer)) > 0;) hash.update(buffer.subarray(0, count));
    return hash.digest('hex');
  } finally { closeSync(fd); }
}
const keys = (value, expected) => assert.deepEqual(Object.keys(value).sort(), [...expected].sort());
const source = (value, kind = 'semantic') => '\\begin{lexlean}{Main}\n\\title{Natural number}\n\\begin{' + kind + 'module}\n\\' + kind + 'data{'
  + JSON.stringify(value) + '}\n\\end{' + kind + 'module}\n\\end{lexlean}\n';

// Full raw authoring payload, including optional presence, production metadata,
// recursion, proof and extension data. Reading does not assert semantic validity.
export const rawModule = {
  spec: 'lexlean/semantic-module/1',
  declarations: [{kind: 'definition', name: 'sample', parameters: [],
    type: {kind: 'const', name: 'Nat'}, value: {kind: 'nat', value: 7},
    executable: false, axioms: [], production: {targets: ['rust-std'], effects: ['allocation', 'overflow']},
    recursion: {kind: 'structural', parameter: 'n'}, proof: {kind: 'decide'}}],
  extension: {presentNull: null, presentFalse: false, presentEmpty: [], label: 'λ <script> & \u0000'},
};
const notationModule = {spec: 'lexlean/semantic-module/2', declarations: [
  {kind: 'theorem', name: 't', parameters: [], statement: {kind: 'bool', value: true}, proof: {kind: 'decide'}, axioms: []},
  // Actual v2 field shapes from the pinned semantic-module-v2 schema. This
  // authoring round trip does not ask the engine to check/link this module.
  {kind: 'definition', name: 'identity', parameters: [{name: 'n', type: {kind: 'nat'}}],
    result: {kind: 'nat'}, body: {kind: 'var', name: 'n'}, recursive_argument: 'n',
    executable: false, axioms: [], production: {effects: ['recursion'], targets: ['rust-std']}},
]};

// Pinned SDK source fixtures, copied as data rather than invoking any helper:
// vendor/lexlean/src/surface/core.rs::tests::sample and
// vendor/lexlean/src/surface/tooling.rs::tests::module (e0b74b81).
export const moduleKindFixtures = [
  {kind: 'core', value: {spec: 'lexlean/core-module/2', imports: ['Init'], proof_nodes: [3],
    nodes: [
      {k: 's', l: {a: {a: 'u', k: 'p'}, k: 's'}}, {k: 'c', n: 'Nat', u: []},
      {f: 1, k: 'a', x: 1}, {i: 0, k: 'b'},
      {b: 'e', k: 'l', n: 'n', t: 1, v: 3}, {b: 'e', k: 'p', n: '', t: 2, v: 2},
    ], declarations: [{class: false, generated: false, kind: 'definition', levels: ['u'], name: 'T',
      policy: {kind: 'none'}, reducibility: {height: 0, kind: 'regular'}, transparency: 'semireducible', type: 5, value: 4}]}},
  {kind: 'tooling', value: {spec: 'lexlean/tooling-module/1', imports: [],
    authority: {id: 'A', lean_githash: 'g', lean_version: '4.32.1'},
    declarations: [{constructors: [{fields: [], name: 'false'}], kind: 'type', name: 'Bool', parameters: []},
      {kind: 'call', name: 'f'}]}},
];

export function decodeProcess(result, {status = 0, operation, code} = {}) {
  assert.equal(result.error, undefined, 'CLI launch failed');
  assert.equal(result.signal, null, 'CLI was killed');
  assert.equal(result.status, status, 'CLI exit status');
  const bytes = Buffer.from(result.stdout);
  assert(bytes.length > 1 && bytes.at(-1) === 10, 'one complete LF-framed JSON result required');
  const text = new TextDecoder('utf-8', {fatal: true}).decode(bytes);
  const value = JSON.parse(text);
  assert(!text.slice(0, -1).includes('\n'), 'no embedded framing newline');
  if (code) {
    assert.equal(value.schema, 'prismpm/error-result/1');
    assert.equal(value.diagnostic.code, code);
  } else {
    keys(value, ['schema', 'operation', operation === 'read' ? 'value' : 'source']);
    assert.equal(value.schema, resultSchema);
    assert.equal(value.operation, operation);
    assert.equal(Buffer.from(result.stderr).length, 0, 'successful CLI stderr');
  }
  return {value, bytes};
}

export function checkVectors(launch) {
  const records = [];
  function call(operation, payload, expected = {}, options = []) {
    const input = Buffer.isBuffer(payload) ? payload : Buffer.from(JSON.stringify(payload));
    const args = ['--json', 'source', ...options, operation];
    const result = decodeProcess(launch(args, input), {...expected, operation});
    records.push({operation, arguments: options, input_sha256: sha256(input),
      output_sha256: sha256(result.bytes), output_bytes: result.bytes.length,
      status: expected.status ?? 0, diagnostic: expected.code ?? null});
    return result;
  }
  const request = (text, fields = {}) => ({schema: requestSchema, source: text, ...fields});
  const original = source(rawModule);
  assert.deepEqual(call('read', request(original)).value.value, rawModule);
  assert.deepEqual(call('read', request(original)).value.value, rawModule, 'repeat read');
  // The SDK reader accepts raw JSON null, while its canonical writer refuses
  // null. Preserve both facts instead of extending the upstream writer domain.
  const {presentNull, ...canonicalExtension} = rawModule.extension;
  assert.equal(presentNull, null);
  const writableModule = {...rawModule, extension: canonicalExtension};
  const rewritten = call('write', request(original, {value: writableModule})).value.source;
  assert.equal(typeof rewritten, 'string');
  assert(rewritten.startsWith('\\begin{lexlean}{Main}\n'));
  assert(rewritten.endsWith('\\end{lexlean}\n'), 'source trailing newline preserved');
  assert.deepEqual(call('read', request(rewritten)).value.value, writableModule);
  call('write', request(original, {value: null}), {status: 1, code: 'PP5802'});
  assert.equal(call('read', request(source(null))).value.value, null, 'reader retains explicit null');
  const floating = {spec: 'unvalidated-extension', positive: 1.25, negative: -0.5, huge: 1e30};
  assert.deepEqual(call('read', request(source(floating))).value.value, floating, 'raw JSON floats retained');
  const notation = call('to-notation', request(source(notationModule))).value.source;
  assert(!notation.includes('\\semanticdata{'));
  assert.deepEqual(call('read', request(notation)).value.value, notationModule);
  assert.equal(call('to-notation', request(source(notationModule))).value.source, notation, 'deterministic notation');
  const changed = {...notationModule, declarations: notationModule.declarations.map((declaration, index) =>
    index === 0 ? {...declaration, name: 'u'} : declaration)};
  const changedSource = call('write', request(notation, {value: changed})).value.source;
  assert.deepEqual(call('read', request(changedSource)).value.value, changed);
  for (const {kind, value} of moduleKindFixtures) {
    const data = source(value, kind);
    assert.deepEqual(call('read', request(data)).value.value, value, kind + ' data');
    const printed = call('to-notation', request(data)).value.source;
    assert(!printed.includes('\\' + kind + 'data{'), kind + ' uses notation');
    assert.deepEqual(call('read', request(printed)).value.value, value, kind + ' notation');
    assert.equal(call('write', request(printed, {value})).value.source, printed, kind + ' notation writer');
  }

  for (const [operation, payload] of [
    ['read', {}], ['read', {...request(original), extra: true}],
    ['read', {schema: requestSchema, source: null}],
    ['write', request(original)], ['read', request(original, {value: null})],
    ['read', Buffer.from('{"schema":"' + requestSchema + '","schema":"' + requestSchema + '","source":"x"}')],
    ['read', Buffer.from([0xff])], ['read', Buffer.from('{')],
  ]) call(operation, payload, {status: 2, code: 'PP5801'});
  call('read', request('no module here'), {status: 1, code: 'PP5802'});
  call('to-notation', request(original), {status: 1, code: 'PP5802'});
  call('write', request(notation, {value: null}), {status: 1, code: 'PP5802'});

  const payload = request(original);
  const inputLength = Buffer.byteLength(JSON.stringify(payload));
  call('read', payload, {}, ['--max-input-bytes', String(inputLength)]);
  call('read', payload, {status: 14, code: 'PP5803'}, ['--max-input-bytes', String(inputLength - 1)]);
  const size = call('read', payload).bytes.length;
  call('read', payload, {}, ['--max-output-bytes', String(size)]);
  call('read', payload, {status: 14, code: 'PP5803'}, ['--max-output-bytes', String(size - 1)]);
  return records;
}

export function installedIdentity(read, actualCli, architecture, measure = path => sha256(read(path))) {
  const cliHash = typeof actualCli === 'string' ? actualCli : sha256(actualCli);
  assert.match(cliHash, /^[0-9a-f]{64}$/);
  const inventoryBytes = read('/opt/prismpm/share/inventory.json');
  const inventory = JSON.parse(inventoryBytes);
  assert.equal(inventory.schema, 'prismpm/sdk-inventory/1');
  const artifact = id => {
    const rows = inventory.artifacts.filter(row => row.id === id);
    assert.equal(rows.length, 1, 'one installed artifact ' + id);
    return rows[0];
  };
  const cli = inventory.commands.filter(row => row.command === 'prismpm');
  assert.equal(cli.length, 1);
  assert.equal(cli[0].executable, executable);
  assert.equal(cli[0].sha256, cliHash);
  assert.equal(artifact('prismpm').digest, 'sha256:' + cliHash);
  const policyBytes = read('/opt/prismpm/share/vv-input-policy.json');
  const manifestBytes = read('/opt/prismpm/share/vv-inputs/manifest.json');
  const policy = JSON.parse(policyBytes), manifest = JSON.parse(manifestBytes);
  assert.equal(policy.schema, 'prismpm/sdk-vv-input-policy/1');
  assert.match(policy.source_revision, /^[0-9a-f]{40}$/);
  assert.deepEqual(manifest.policy, policy);
  assert.equal(artifact('sdk-vv-policy').digest, 'sha256:' + sha256(policyBytes));
  assert.equal(artifact('sdk-vv-manifest').digest, 'sha256:' + sha256(manifestBytes));
  assert.equal(artifact('sdk-vv-source').version, policy.source_revision);
  assert.equal(manifest.source.revision, policy.source_revision);
  const packedSource = manifest.artifacts.filter(row => row.path === 'source.pack');
  assert.equal(packedSource.length, 1);
  assert.equal(artifact('sdk-vv-source').digest, 'sha256:' + packedSource[0].sha256);
  assert(['x64', 'arm64'].includes(architecture));
  const locks = ['Cargo.lock', 'lexlean.lock', 'model/dependencies.toml', 'rust-toolchain.toml', 'lean-toolchain'].map(path => {
    const rows = manifest.source.files.filter(row => row.path === path);
    assert.equal(rows.length, 1, 'one captured source lock ' + path);
    assert.match(rows[0].sha256, /^[0-9a-f]{64}$/);
    return {path, sha256: rows[0].sha256};
  });
  const tools = ['lexlean', 'cargo', 'rustc', 'lean', 'lake'].map(command => {
    const rows = inventory.commands.filter(row => row.command === command);
    assert.equal(rows.length, 1, 'one installed tool ' + command);
    assert.equal(measure(rows[0].executable), rows[0].sha256, 'actual installed tool ' + command);
    return rows[0];
  });
  assert.equal(artifact('lexlean').digest, 'sha256:' + tools[0].sha256);
  const harness = manifest.source.files.filter(row => row.path === 'scripts/source-authoring-sdk-check.mjs');
  assert.equal(harness.length, 1);
  return {source_revision: policy.source_revision, platform: 'linux/' + ({x64: 'amd64', arm64: 'arm64'}[architecture]),
    cli_sha256: cliHash, inventory_sha256: sha256(inventoryBytes),
    policy_sha256: sha256(policyBytes), manifest_sha256: sha256(manifestBytes),
    source_artifact: artifact('sdk-vv-source'), lexlean: artifact('lexlean'), source_locks: locks,
    tool_commands: tools, source_harness_sha256: harness[0].sha256};
}

export function checkOutputFailure(result) {
  assert.equal(result.stdout, null, 'stdout must be attached directly to the failing sink');
  const failure = decodeProcess({...result, stdout: result.stderr, stderr: Buffer.alloc(0)},
    {status: 1, code: 'PP5804'});
  assert(JSON.stringify(failure.value.diagnostic).includes('Unknown'), 'truthful output disposition required');
  return {sink: '/dev/full', status: 1, diagnostic: 'PP5804', disposition: 'Unknown',
    stderr_sha256: sha256(result.stderr)};
}

export function runInstalled({mode, image = null, expectedRevision = null, harnessSha256}) {
  assert(['construction', 'installed'].includes(mode));
  assert.equal(process.getuid(), 1000, 'unprivileged installed consumer required');
  assert.match(harnessSha256, /^[0-9a-f]{64}$/);
  if (mode === 'installed') {
    assert.match(image, /^[a-z0-9./_:-]+@sha256:[0-9a-f]{64}$/);
    assert.match(expectedRevision, /^[0-9a-f]{40}$/);
    for (const directory of ['/opt/prismpm/source', '/opt/prismpm/share/conformance-root']) {
      assert.deepEqual(readdirSync(directory), [], 'source directory must be masked');
    }
  }
  const identity = installedIdentity(readFileSync, hashFile(executable), process.arch, hashFile);
  assert.equal(identity.source_harness_sha256, harnessSha256, 'harness differs from captured SDK source');
  if (expectedRevision) assert.equal(identity.source_revision, expectedRevision);
  const temporary = mkdtempSync(join(tmpdir(), 'source-authoring-check-'));
  try {
    const records = checkVectors((args, input) => spawnSync(executable, args, {
      cwd: temporary, input, encoding: null, timeout: 60_000, maxBuffer: 8 * 1024 * 1024,
      env: {...process.env, PRISMPM_SEMANTIC_SOURCE: '/unavailable/internal-helper'},
    }));
    const sink = openSync('/dev/full', 'w');
    let outputFailure;
    try {
      outputFailure = checkOutputFailure(spawnSync(executable, ['--json', 'source', 'read'], {
        cwd: temporary, input: JSON.stringify({schema: requestSchema, source: source(rawModule)}),
        stdio: ['pipe', sink, 'pipe'], timeout: 60_000, maxBuffer: 1024 * 1024,
      }));
    } finally { closeSync(sink); }
    assert.deepEqual(readdirSync(temporary), [], 'source authoring must not create project artifacts');
    return {schema: 'prismpm/source-authoring-sdk-check/1', status: 'passed',
      scope: mode === 'installed' ? 'source-free-installed-authoring-component' : 'image-unbound-construction-self-check',
      image, ...identity, harness_sha256: harnessSha256, checks: records, output_failure: outputFailure,
      unclaimed: ['semantic-validity', 'linked-snapshot', 'product-targets', 'whole-sdk-qualification', 'release-acceptance']};
  } finally { rmSync(temporary, {recursive: true, force: true}); }
}

if (['construction', 'installed'].includes(process.argv[2])) {
  const mode = process.argv[2];
  const harnessSha256 = mode === 'installed' ? process.argv[5] : sha256(readFileSync(process.argv[1]));
  console.log(JSON.stringify(runInstalled({mode, image: process.argv[3] ?? null,
    expectedRevision: process.argv[4] ?? null, harnessSha256})));
}
