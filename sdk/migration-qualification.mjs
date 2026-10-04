// Installed migration qualification; fixture provenance is historical, not SDK admission.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {chmodSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync} from 'node:fs';
import {join} from 'node:path';

export const canonical = value => JSON.stringify(value, (_, item) => item && typeof item === 'object' && !Array.isArray(item)
  ? Object.fromEntries(Object.keys(item).sort().map(key => [key, item[key]])) : item);
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
export function historicalLock() {
  const fixture = JSON.parse(readFileSync(new URL('./fixtures/hologram-live-historical-lock.json', import.meta.url)));
  assert.deepEqual(fixture.source, {repository: 'https://github.com/Hologram-Technologies/hologram-live',
    revision: '419759164f6cfae768fd5536da5cd8422efbf032', path: 'prismpm.lock',
    git_blob: 'cec41c29f6caf8ace4ea2cbf1fb38db3d242bdaa',
    sha256: '5ee081904bacc478e07a0706421c4a09fe02121e0e0c65391ca2395ec1b41bac'});
  assert.equal(hash(fixture.document), fixture.source.sha256);
  assert.equal(canonical(JSON.parse(fixture.document)), fixture.document);
  assert.equal(JSON.parse(fixture.document).schema, 'prismpm/sdk-lock/1');
  return fixture;
}
export const migrationChecks = Object.freeze(['historical-refusal', 'read-only-proposal', 'exact-root-replay',
  'native-admission', 'changed-native-inventory-refusal', 'swapped-platform-refusal']);
export function inventoryMutants(target, platform) {
  const changed = structuredClone(target), native = changed.platforms.find(row => row.platform === platform);
  assert.notEqual(native.inventory[0].digest, 'sha256:' + '0'.repeat(64));
  native.inventory[0].digest = 'sha256:' + '0'.repeat(64);
  const document = JSON.parse(native.inventory_document); document.artifacts = native.inventory;
  native.inventory_document = canonical(document) + '\n';
  native.inventory_digest = 'sha256:' + hash(native.inventory_document);
  const swapped = structuredClone(target);
  for (const field of ['inventory', 'inventory_document', 'inventory_digest'])
    [swapped.platforms[0][field], swapped.platforms[1][field]] = [swapped.platforms[1][field], swapped.platforms[0][field]];
  assert.notEqual(swapped.platforms[0].inventory_digest, swapped.platforms[1].inventory_digest,
    'foreign native inventory probe must alter native authority');
  return [changed, swapped];
}
export function expectedMigrationProcesses(target, platform) {
  const check = ['lock', 'check'];
  return [
    {argv: check, exit_code: 4, input_sha256: historicalLock().source.sha256},
    {argv: ['lock', 'migrate', '--sdk-image', target.sdk_image, '--standards-lock', target.standards_lock], exit_code: 0, input_sha256: historicalLock().source.sha256},
    {argv: check, exit_code: 0, input_sha256: hash(canonical(target))},
    ...inventoryMutants(target, platform).map(value => ({argv: check, exit_code: 4, input_sha256: hash(canonical(value))})),
  ];
}
export function verifyMigration(value, lockBytes) {
  assert.deepEqual(Object.keys(value).sort(), ['checks', 'historical_sha256', 'platform', 'processes', 'proposal', 'schema', 'target_sha256']);
  assert.equal(value.schema, 'prismpm/installed-lock-migration/1');
  assert(['linux/amd64', 'linux/arm64'].includes(value.platform));
  assert.equal(value.historical_sha256, historicalLock().source.sha256);
  assert.equal(value.target_sha256, hash(lockBytes));
  assert.deepEqual(value.checks, migrationChecks);
  const old = JSON.parse(historicalLock().document), target = JSON.parse(lockBytes);
  assert.equal(target.schema, 'prismpm/sdk-lock/2');
  assert.equal(canonical(target), lockBytes.toString());
  assert.deepEqual(value.proposal, {schema: 'prismpm/sdk-lock-migration/1',
    compatibility_review: 'required', generated_output_diff: 'required', security_review: 'required',
    patch: [{op: 'test', path: '', value: old}, {op: 'replace', path: '', value: target}]});
  // Apply only after the original full document passes the ordered test.
  let applied = old;
  for (const operation of value.proposal.patch) {
    assert.equal(operation.path, '');
    if (operation.op === 'test') assert.deepEqual(applied, operation.value);
    else { assert.equal(operation.op, 'replace'); applied = operation.value; }
  }
  assert.equal(canonical(applied), lockBytes.toString());
  assert.equal(value.processes.length, 5);
  const expected = expectedMigrationProcesses(target, value.platform);
  for (const [index, row] of value.processes.entries()) {
    const {stdout_sha256, stderr_sha256, ...identity} = row;
    assert.match(stdout_sha256, /^[a-f0-9]{64}$/); assert.match(stderr_sha256, /^[a-f0-9]{64}$/);
    assert.deepEqual(identity, expected[index]);
  }
}

export function qualifyMigration(lockBytes) {
  const original = historicalLock(), target = JSON.parse(lockBytes);
  const platform = 'linux/' + ({x64: 'amd64', arm64: 'arm64'}[process.arch]);
  assert(['linux/amd64', 'linux/arm64'].includes(platform));
  const root = mkdtempSync('/tmp/prismpm-migration-'), path = join(root, 'prismpm.lock');
  const processes = [];
  function invoke(args, refusal) {
    const before = readFileSync(path);
    const result = spawnSync('/usr/local/bin/prismpm', ['--json', '--project', root, ...args],
      {encoding: 'utf8', timeout: 240000, maxBuffer: 192 * 1024 * 1024});
    assert.ifError(result.error); assert.equal(result.signal, null);
    assert(readFileSync(path).equals(before), 'installed command changed lock');
    assert.deepEqual(readdirSync(root), ['prismpm.lock'], 'installed command wrote project output');
    const value = JSON.parse(result.stdout);
    if (refusal) {
      assert.equal(result.status, 4);
      assert.equal(value.schema, 'prismpm/error-result/1');
      assert.equal(value.diagnostic.code, 'PP5401');
      assert.equal(value.diagnostic.message, refusal);
    } else assert.equal(result.status, 0);
    processes.push({argv: args, exit_code: result.status, input_sha256: hash(before),
      stdout_sha256: hash(result.stdout), stderr_sha256: hash(result.stderr)});
    return value;
  }
  const put = bytes => { chmodSync(root, 0o700); writeFileSync(path, bytes, {mode: 0o600}); };
  try {
    put(original.document);
    invoke(['lock', 'check'], 'legacy SDK inventory disagrees with the running SDK');
    chmodSync(path, 0o400); chmodSync(root, 0o500);
    const proposal = invoke(['lock', 'migrate', '--sdk-image', target.sdk_image, '--standards-lock', target.standards_lock]);
    assert.deepEqual(proposal, {schema: 'prismpm/sdk-lock-migration/1', compatibility_review: 'required',
      generated_output_diff: 'required', security_review: 'required',
      patch: [{op: 'test', path: '', value: JSON.parse(original.document)}, {op: 'replace', path: '', value: target}]});
    chmodSync(root, 0o700); chmodSync(path, 0o600);
    put(canonical(proposal.patch[1].value));
    assert.deepEqual(invoke(['lock', 'check']), target);
    for (const mutant of inventoryMutants(target, platform)) {
      put(canonical(mutant)); invoke(['lock', 'check'], 'SDK platform inventory disagrees with the running SDK');
    }
    assert.equal(processes.length, 5);
    const evidence = {schema: 'prismpm/installed-lock-migration/1', platform, processes,
      historical_sha256: original.source.sha256, target_sha256: hash(lockBytes), proposal, checks: [...migrationChecks]};
    verifyMigration(evidence, lockBytes);
    return evidence;
  } finally { chmodSync(root, 0o700); rmSync(root, {recursive: true}); }
}
