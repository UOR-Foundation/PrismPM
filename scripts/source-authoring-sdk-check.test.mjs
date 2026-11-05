import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {checkOutputFailure, decodeProcess, installedIdentity, moduleKindFixtures, sha256} from './source-authoring-sdk-check.mjs';

const success = {operation: 'read', schema: 'prismpm/source-authoring-result/1', value: {float: 1.25, explicit: null}};
const processResult = (value = success) => ({status: 0, signal: null,
  stdout: Buffer.from(JSON.stringify(value) + '\n'), stderr: Buffer.alloc(0)});

test('harness accepts full authoring Value and checks complete process outcome', () => {
  assert.deepEqual(moduleKindFixtures.map(row => row.kind), ['core', 'tooling']);
  assert.equal(moduleKindFixtures[0].value.spec, 'lexlean/core-module/2');
  assert.equal(moduleKindFixtures[1].value.spec, 'lexlean/tooling-module/1');
  assert.deepEqual(decodeProcess(processResult(), {operation: 'read'}).value, success);
  for (const mutation of [
    row => {row.status = 1;}, row => {row.signal = 'SIGTERM';}, row => {row.error = new Error('launch');},
    row => {row.stdout = Buffer.from('{');}, row => {row.stdout = Buffer.from(JSON.stringify(success));},
    row => {row.stdout = Buffer.from(JSON.stringify(success) + '\n' + JSON.stringify(success) + '\n');},
    row => {row.stderr = Buffer.from('unexpected');}, row => {row.stdout = Buffer.from([0xff, 10]);},
    row => {row.stdout = Buffer.from(JSON.stringify({...success, extra: true}) + '\n');},
    row => {row.stdout = Buffer.from(JSON.stringify({...success, operation: 'write'}) + '\n');},
  ]) {
    const row = processResult(); mutation(row);
    assert.throws(() => decodeProcess(row, {operation: 'read'}));
  }
});

test('harness requires the diagnostic and exact nonzero status, never a planted success', () => {
  const error = {schema: 'prismpm/error-result/1', diagnostic: {code: 'PP5803'}};
  const row = {...processResult(error), status: 14};
  decodeProcess(row, {status: 14, code: 'PP5803'});
  assert.throws(() => decodeProcess({...row, status: 0}, {status: 14, code: 'PP5803'}));
  assert.throws(() => decodeProcess(row, {status: 14, code: 'PP5801'}));
});

test('failed stdout requires stderr diagnostic and Unknown disposition without a second stdout result', () => {
  const failure = {schema: 'prismpm/error-result/1', diagnostic: {code: 'PP5804', notes: [{message: 'stdout outcome is Unknown'}]}};
  const row = {status: 1, signal: null, stdout: null, stderr: Buffer.from(JSON.stringify(failure) + '\n')};
  assert.equal(checkOutputFailure(row).disposition, 'Unknown');
  assert.throws(() => checkOutputFailure({...row, status: 0}));
  assert.throws(() => checkOutputFailure({...row, stdout: Buffer.from('{}\n')}));
  assert.throws(() => checkOutputFailure({...row, stderr: Buffer.from(JSON.stringify({schema: failure.schema, diagnostic: {code: 'PP5804'}}) + '\n')}));
});

function identityFixture() {
  const revision = 'a'.repeat(40), files = new Map();
  const cli = Buffer.from('actual test CLI');
  const commands = ['prismpm', 'lexlean', 'cargo', 'rustc', 'lean', 'lake'].map(command => {
    const executable = '/usr/local/bin/' + command, bytes = command === 'prismpm' ? cli : Buffer.from(command);
    files.set(executable, bytes);
    return {command, executable, sha256: sha256(bytes)};
  });
  const policy = {schema: 'prismpm/sdk-vv-input-policy/1', source_revision: revision};
  const sourceFiles = ['Cargo.lock', 'lexlean.lock', 'model/dependencies.toml', 'rust-toolchain.toml', 'lean-toolchain', 'scripts/source-authoring-sdk-check.mjs']
    .map(path => ({path, sha256: sha256(path)}));
  const manifest = {policy, source: {revision, files: sourceFiles}, artifacts: [{path: 'source.pack', sha256: 'b'.repeat(64)}]};
  const policyBytes = Buffer.from(JSON.stringify(policy)), manifestBytes = Buffer.from(JSON.stringify(manifest));
  const artifacts = [
    {id: 'prismpm', digest: 'sha256:' + sha256(cli)},
    {id: 'lexlean', version: '0.4.0', digest: 'sha256:' + sha256('lexlean')},
    {id: 'sdk-vv-source', version: revision, digest: 'sha256:' + 'b'.repeat(64)},
    {id: 'sdk-vv-policy', digest: 'sha256:' + sha256(policyBytes)},
    {id: 'sdk-vv-manifest', digest: 'sha256:' + sha256(manifestBytes)},
  ];
  const inventory = {schema: 'prismpm/sdk-inventory/1', commands, artifacts};
  files.set('/opt/prismpm/share/inventory.json', Buffer.from(JSON.stringify(inventory)));
  files.set('/opt/prismpm/share/vv-input-policy.json', policyBytes);
  files.set('/opt/prismpm/share/vv-inputs/manifest.json', manifestBytes);
  return {files, inventory, manifest, cli, read: path => {assert(files.has(path), 'unexpected source access ' + path); return files.get(path);}};
}

test('installed receipt binds actual CLI/tools, source pack, native platform and complete source locks', () => {
  const f = identityFixture(), value = installedIdentity(f.read, f.cli, 'arm64');
  assert.equal(value.platform, 'linux/arm64');
  assert.equal(value.source_revision, 'a'.repeat(40));
  assert.equal(value.cli_sha256, sha256(f.cli));
  assert.equal(value.source_locks.length, 5);
  assert.equal(value.tool_commands.length, 5);
  assert.equal(value.lexlean.version, '0.4.0');
});

test('installed identity refuses substituted binaries and missing or mismatched inventory authority', () => {
  for (const mutate of [
    f => f.files.set('/usr/local/bin/rustc', Buffer.from('substitute')),
    f => f.inventory.commands.push(f.inventory.commands[0]),
    f => {f.inventory.artifacts.find(row => row.id === 'sdk-vv-policy').digest = 'sha256:' + '0'.repeat(64);},
    f => {f.inventory.artifacts.find(row => row.id === 'sdk-vv-source').version = '0'.repeat(40);},
    f => {f.inventory.artifacts.find(row => row.id === 'sdk-vv-source').digest = 'sha256:' + '0'.repeat(64);},
    f => {f.inventory.artifacts = f.inventory.artifacts.filter(row => row.id !== 'lexlean');},
  ]) {
    const f = identityFixture(); mutate(f);
    f.files.set('/opt/prismpm/share/inventory.json', Buffer.from(JSON.stringify(f.inventory)));
    assert.throws(() => installedIdentity(f.read, f.cli, 'x64'));
  }
  const f = identityFixture();
  assert.throws(() => installedIdentity(f.read, Buffer.from('wrong CLI'), 'x64'));
  assert.throws(() => installedIdentity(f.read, f.cli, 's390x'));
});

test('normal SDK owners retain source-free checks without installing the internal helper', () => {
  const recipe = readFileSync(new URL('../sdk/Dockerfile', import.meta.url), 'utf8');
  assert(recipe.includes('scripts/source-authoring-sdk-check.mjs construction'));
  assert(recipe.indexOf('scripts/source-authoring-sdk-check.mjs construction') > recipe.indexOf('node /opt/prismpm/generate-inventory.mjs /opt/prismpm/share/inventory.json'));
  assert(!recipe.includes('install -m 0755 target/release/prismpm-semantic-source'));
  const owner = readFileSync(new URL('../xtask/src/main.rs', import.meta.url), 'utf8');
  assert(owner.includes('scripts/source-authoring-sdk-installed.test.mjs'));
  assert(owner.includes('scripts/sdk-runtime-exec.test.mjs'));
  assert(owner.includes('scripts/source-authoring-sdk-check.test.mjs'));
});
