// Archive/custody units. The explicit proof-reader seam below is never CLI or
// installed acceptance evidence; source/inventory/result readers stay original.
import assert from 'node:assert/strict';
import {execFileSync, spawnSync} from 'node:child_process';
import {chmodSync, closeSync, copyFileSync, existsSync, linkSync, lstatSync, mkdirSync, mkdtempSync,
  readFileSync, readdirSync, rmSync, symlinkSync, truncateSync, writeFileSync} from 'node:fs';
import {dirname, join, resolve} from 'node:path';
import {tmpdir} from 'node:os';
import test from 'node:test';
import {capture, canonical, hash, qualificationChecks, sourceAliases, sourceRoots, unclaimed} from './binary-sdk-check.mjs';
import {inventory, parserFixture} from './binary-sdk-check-fixtures.mjs';
import {artifactName, releaseContext, selectArtifacts} from './release-gate-evidence.mjs';
import {packBinaryEvidence, retainBinaryEvidence, binaryEvidenceLimits} from './binary-sdk-release-evidence.mjs';

const repository = resolve(new URL('..', import.meta.url).pathname);
const revision = 'a'.repeat(40), image = 'ghcr.io/uor-foundation/prismpm-sdk@sha256:' + 'b'.repeat(64);
const put = (root, relative, bytes) => {const path = join(root, relative); mkdirSync(dirname(path), {recursive: true}); writeFileSync(path, bytes); return path;};
const context = (architecture = 'amd64', language = '1.1') => ({source_revision: revision, image_reference: image, architecture, language, run_id: '1234', run_attempt: '2'});
function temporary(t) {const root = mkdtempSync(join(tmpdir(), 'binary-evidence-unit-')); t.after(() => rmSync(root, {recursive: true, force: true})); return root;}
function sourceFixture(root) {
  for (const path of sourceRoots) if (!Object.hasOwn(sourceAliases, path)) put(root, path + '/input', 'source fixture only');
  for (const [path, target] of Object.entries(sourceAliases)) {
    const full = join(root, path); mkdirSync(dirname(full), {recursive: true});
    if (path.includes('/embedded/')) put(dirname(full), target, 'bound source alias fixture');
    symlinkSync(target, full);
  }
  rmSync(join(root, 'lean-toolchain'), {recursive: true});
  put(root, 'lean-toolchain', readFileSync(join(repository, 'lean-toolchain')));
  copyFileSync(join(repository, 'vendor/lean4-prod/lean.tar'), put(root, 'vendor/lean4-prod/lean.tar', ''));
}
function fixture(t, selected = context()) {
  const root = temporary(t), sourceRoot = join(root, 'source'), artifact = join(root, 'downloaded');
  mkdirSync(sourceRoot); mkdirSync(artifact); sourceFixture(sourceRoot);
  const name = `sdk-${selected.architecture}-binary-sdk${selected.language === '1.2' ? '-language-1.2' : ''}`;
  const source = capture(sourceRoot, revision), sourceBytes = Buffer.from(canonical(source) + '\n');
  const inventoryBytes = Buffer.from(canonical(inventory()) + '\n');
  const receipt = {schema: 'prismpm/verify-result/1', build_id: 'd'.repeat(64), attestation_id: 'e'.repeat(64), verified_root: '.prism/verified/' + 'e'.repeat(64)};
  const result = {schema: selected.language === '1.1' ? 'prismpm/installed-binary-check/1' : 'prismpm/installed-binary-language-1.2-check/1',
    scope: 'installed-binary-package-only', status: 'passed', sdk_image: image, source_revision: revision,
    source_sha256: hash(canonical(source)), inventory_sha256: hash(inventoryBytes), architecture: selected.architecture,
    build_id: receipt.build_id, attestation_id: receipt.attestation_id, checks: qualificationChecks(selected.language), unclaimed};
  put(artifact, name + '.log', Buffer.from([0, 65, 255, 10]));
  put(artifact, name + '/source.json', sourceBytes); put(artifact, name + '/inventory.json', inventoryBytes);
  put(artifact, name + '/result.json', canonical(result)); put(artifact, name + '/proof/receipt.json', canonical(receipt));
  put(artifact, name + '/proof/proof.prism', 'raw proof.prism unit bytes\n');
  put(artifact, name + '/proof/.prism/build/' + receipt.build_id + '/bytes.bin', Buffer.from([0, 255, 17]));
  mkdirSync(join(artifact, name, 'proof/empty-directory'));
  const owners = [`binary-sdk-sdk-${selected.architecture}`, `binary-language-1.2-sdk-sdk-${selected.architecture}`];
  const providerContext = releaseContext(revision, selected.architecture, image, selected.run_id, selected.run_attempt);
  const metadata = {jobs: [{id: 41, run_id: 1234, head_sha: revision, name: `browser-library-${selected.architecture}`,
    run_attempt: 1, status: 'completed', conclusion: 'success'}], artifacts: owners.map((owner, index) => ({id: 51 + index,
    name: artifactName(owner, {...providerContext, run_attempt: '1'}), workflow_run: {id: 1234, head_sha: revision},
    expired: false, size_in_bytes: 123, digest: 'sha256:' + 'c'.repeat(64)}))};
  const provider = put(root, 'provider.json', JSON.stringify(metadata)), selection = put(root, 'selection.json', canonical(selectArtifacts(metadata, providerContext, owners)));
  const directories = [], files = [];
  function describe(relative = '') {
    for (const entry of readdirSync(join(artifact, relative)).sort()) {
      const path = relative ? relative + '/' + entry : entry, full = join(artifact, path);
      if (lstatSync(full).isDirectory()) {directories.push(path); describe(path);}
      else {const bytes = readFileSync(full); files.push({path, size: bytes.length, sha256: hash(bytes)});}
    }
  }
  describe(); directories.sort(); files.sort((a, b) => a.path < b.path ? -1 : a.path > b.path ? 1 : 0);
  const retention = {schema: 'prismpm/binary-sdk-transport-retention/1', scope: 'original-binary-gate-transport-only',
    ...selected, run_attempt: '1', owner: owners[selected.language === '1.2' ? 1 : 0], job_name: `browser-library-${selected.architecture}`, directories, files};
  put(artifact, 'retention.json', canonical(retention) + '\n');
  return {root, sourceRoot, artifact, name, receipt, result, selected, provider, selection, metadata, retention,
    output: join(root, 'evidence.tar'), proof: join(artifact, name, 'proof')};
}
let unitSequence = 0;
async function unitPackager(t, reader = () => {}, transform = value => value) {
  const key = 'prismpm.binary.evidence.unit.' + ++unitSequence;
  globalThis[Symbol.for(key)] = reader; t.after(() => delete globalThis[Symbol.for(key)]);
  const url = new URL('./binary-sdk-release-evidence.mjs', import.meta.url);
  let source = readFileSync(url, 'utf8');
  assert.equal(source.split('const validateProof = checkAccepted;').length, 2, 'production must retain the original proof reader');
  source = source.replace('const validateProof = checkAccepted;', `const validateProof = (...args) => globalThis[Symbol.for(${JSON.stringify(key)})](...args);`)
    .replace("const repository = resolve(dirname(fileURLToPath(import.meta.url)), '..');", `const repository = ${JSON.stringify(repository)};`)
    .replace(/from '(\.[^']+)'/g, (_, path) => `from '${new URL(path, url).href}'`);
  source = transform(source);
  return import('data:text/javascript;base64,' + Buffer.from(source + '\nexport {verifyArchive as unitArchive};\n').toString('base64'));
}

for (const architecture of ['amd64', 'arm64']) for (const language of ['1.1', '1.2']) {
  test(`archive unit retains all raw bytes for ${architecture} language ${language}, including traversable directories`, async t => {
    const f = fixture(t, context(architecture, language)); let calls = 0;
    const unit = await unitPackager(t, (proof, receipt, authority, selected) => {
      calls++; assert.equal(proof, f.proof); assert.deepEqual(receipt, f.receipt); assert.equal(selected, language);
      assert.equal(authority.archive_sha256, hash(readFileSync(join(repository, 'vendor/lean4-prod/lean.tar'))));
      assert.equal(authority.toolchain, readFileSync(join(repository, 'lean-toolchain'), 'utf8').trim());
    });
    const manifest = unit.packBinaryEvidence(f.artifact, f.output, f.selected, {provider: f.provider, selection: f.selection}, f.sourceRoot);
    assert.equal(calls, 2, 'original receipt path must be revalidated before publication');
    assert.equal(manifest.scope, 'original-binary-gate-evidence-only');
    assert(manifest.files.some(row => row.path === f.name + '/proof/proof.prism'));
    const listing = execFileSync('/usr/bin/tar', ['--numeric-owner', '-tvf', f.output], {encoding: 'utf8'});
    for (const line of listing.trim().split('\n')) assert.match(line, /^(?:dr-xr-xr-x|-r--r--r--) 0\/0 /);
    for (const row of manifest.files) {
      const archived = execFileSync('/usr/bin/tar', ['-xOf', f.output, '--', row.path], {maxBuffer: binaryEvidenceLimits.file});
      assert(archived.equals(readFileSync(join(f.artifact, row.path)))); assert.equal(hash(archived), row.sha256);
    }
    assert.deepEqual(JSON.parse(execFileSync('/usr/bin/tar', ['-xOf', f.output, 'evidence-manifest.json'])), manifest);
    assert(!readdirSync(f.root).some(name => name.startsWith('.binary-sdk-evidence-')));
  });
}

test('archive bytes are deterministic across fresh absolute roots', async t => {
  const first = fixture(t), second = fixture(t), unit = await unitPackager(t);
  unit.packBinaryEvidence(first.artifact, first.output, first.selected, {provider: first.provider, selection: first.selection}, first.sourceRoot);
  unit.packBinaryEvidence(second.artifact, second.output, second.selected, {provider: second.provider, selection: second.selection}, second.sourceRoot);
  assert(readFileSync(first.output).equals(readFileSync(second.output)));
});

test('ambient tar configuration cannot change the deterministic archive recipe', async t => {
  const f = fixture(t), unit = await unitPackager(t), before = process.env.TAR_OPTIONS;
  try {
    process.env.TAR_OPTIONS = '--mode=0000';
    unit.packBinaryEvidence(f.artifact, f.output, f.selected, {provider: f.provider, selection: f.selection}, f.sourceRoot);
  } finally {
    if (before === undefined) delete process.env.TAR_OPTIONS; else process.env.TAR_OPTIONS = before;
  }
  assert(existsSync(f.output));
});

test('current source, exact native context, inventory and original result remain mandatory', async t => {
  const unit = await unitPackager(t);
  const cases = [
    f => put(f.sourceRoot, 'scripts/binary-sdk-check.mjs/input', 'source changed'),
    f => {const value = {...f.result, source_sha256: 'f'.repeat(64)}; put(f.artifact, f.name + '/result.json', canonical(value));},
    f => {const value = {...f.result, inventory_sha256: 'f'.repeat(64)}; put(f.artifact, f.name + '/result.json', canonical(value));},
    f => {const value = {...f.result, checks: f.result.checks.slice(1)}; put(f.artifact, f.name + '/result.json', canonical(value));},
    f => {put(f.artifact, f.name + '/proof/receipt.json', canonical({...f.receipt, attestation_id: 'f'.repeat(64)}));},
    f => {f.selected.architecture = 'arm64';},
    f => {f.selected.language = '1.2';},
    f => {f.selected.image_reference = image.replace('b'.repeat(64), 'f'.repeat(64));},
    f => {f.selected.source_revision = 'f'.repeat(40);},
  ];
  for (const change of cases) {const f = fixture(t); change(f); assert.throws(() => unit.packBinaryEvidence(f.artifact, f.output, f.selected, {provider: f.provider, selection: f.selection}, f.sourceRoot)); assert(!existsSync(f.output));}
});

test('unknown/missing artifact members, symlinks, hardlinks, special files and size/entry budgets fail closed', async t => {
  const unit = await unitPackager(t);
  const changes = [
    f => put(f.artifact, 'extra', 'unregistered'),
    f => rmSync(join(f.artifact, f.name + '.log')),
    f => rmSync(join(f.artifact, f.name, 'inventory.json')),
    f => put(f.artifact, f.name + '/extra', 'unregistered'),
    f => symlinkSync('/etc/hosts', join(f.proof, 'alias')),
    f => linkSync(join(f.artifact, f.name + '.log'), join(f.root, 'outside-hardlink')),
    f => execFileSync('/usr/bin/mkfifo', [join(f.proof, 'fifo')]),
    f => truncateSync(join(f.artifact, f.name + '.log'), binaryEvidenceLimits.file + 1),
    f => {for (let n = 0; n < binaryEvidenceLimits.entries; n++) put(f.proof, `entry-${n}`, '');},
  ];
  for (const change of changes) {const f = fixture(t); change(f); assert.throws(() => unit.packBinaryEvidence(f.artifact, f.output, f.selected, {provider: f.provider, selection: f.selection}, f.sourceRoot)); assert(!existsSync(f.output));}
});

test('proof refusal propagates, and metadata/bytes cannot change between validation and publication', async t => {
  const rejected = fixture(t), refuse = await unitPackager(t, () => {throw Error('original proof reader refused unit fixture');});
  assert.throws(() => refuse.packBinaryEvidence(rejected.artifact, rejected.output, rejected.selected, {provider: rejected.provider, selection: rejected.selection}, rejected.sourceRoot), /original proof reader refused/);
  for (const change of [
    f => put(f.artifact, f.name + '.log', 'changed log'),
    f => put(f.proof, 'late-file', 'late'),
    f => put(f.sourceRoot, 'vendor/lean4-prod/lean.tar', 'changed authority'),
  ]) {
    const f = fixture(t); let calls = 0;
    const unit = await unitPackager(t, () => {if (++calls === 2) change(f);});
    assert.throws(() => unit.packBinaryEvidence(f.artifact, f.output, f.selected, {provider: f.provider, selection: f.selection}, f.sourceRoot)); assert(!existsSync(f.output));
  }
});

test('fresh output is exclusive, including a destination created after validation', async t => {
  const f = fixture(t); let calls = 0;
  const unit = await unitPackager(t, () => {if (++calls === 2) writeFileSync(f.output, 'retain sentinel');});
  assert.throws(() => unit.packBinaryEvidence(f.artifact, f.output, f.selected, {provider: f.provider, selection: f.selection}, f.sourceRoot));
  assert.equal(readFileSync(f.output, 'utf8'), 'retain sentinel');
  assert.throws(() => unit.packBinaryEvidence(f.artifact, f.output, f.selected, {provider: f.provider, selection: f.selection}, f.sourceRoot), /fresh/);
});

test('combined raw-byte budget is enforced independently of the per-file bound', async t => {
  const f = fixture(t);
  put(f.proof, 'first.bin', Buffer.alloc(160 * 1024)); put(f.proof, 'second.bin', Buffer.alloc(160 * 1024));
  const unit = await unitPackager(t, () => {}, source => source.replace('total: 1024 ** 3', 'total: 300 * 1024'));
  assert.throws(() => unit.packBinaryEvidence(f.artifact, f.output, f.selected, {provider: f.provider, selection: f.selection}, f.sourceRoot), /bounded|exceeds/);
  assert(!existsSync(f.output));
});

function tarRows(bytes) {
  const rows = []; let offset = 0;
  while (!bytes.subarray(offset, offset + 512).every(value => value === 0)) {
    const header = bytes.subarray(offset, offset + 512), field = value => value.toString('ascii').replace(/\0.*$/, '');
    const prefix = field(header.subarray(345, 500)), name = (prefix ? prefix + '/' : '') + field(header.subarray(0, 100));
    const size = Number.parseInt(field(header.subarray(124, 136)), 8);
    rows.push({offset, name, size, payload: offset + 512}); offset += 512 + Math.ceil(size / 512) * 512;
  }
  return {rows, terminator: offset};
}
function checksum(bytes, offset) {
  bytes.fill(32, offset + 148, offset + 156);
  const sum = bytes.subarray(offset, offset + 512).reduce((total, value) => total + value, 0);
  bytes.write(sum.toString(8).padStart(6, '0') + '\0 ', offset + 148, 'ascii');
}
test('read-only archive validation rejects altered payloads, graph, modes, links, traversal, checksum and trailers', async t => {
  const f = fixture(t), unit = await unitPackager(t);
  const manifest = unit.packBinaryEvidence(f.artifact, f.output, f.selected, {provider: f.provider, selection: f.selection}, f.sourceRoot);
  const original = readFileSync(f.output), parsed = tarRows(original);
  const proof = parsed.rows.find(row => row.name.endsWith('/proof/proof.prism'));
  const manifestBytes = execFileSync('/usr/bin/tar', ['-xOf', f.output, 'evidence-manifest.json']);
  const files = new Map(manifest.files.map(row => [row.path, row]));
  for (const [name, path] of [['provider-metadata.json', f.provider], ['artifact-selection.json', f.selection]]) {const bytes = readFileSync(path); files.set(name, {size: bytes.length, sha256: hash(bytes)});}
  files.set('evidence-manifest.json', {size: manifestBytes.length, sha256: hash(manifestBytes)});
  const directories = new Set(manifest.directories);
  const mutations = [
    bytes => {bytes[proof.payload] ^= 1;},
    bytes => {bytes.write('0000400\0', proof.offset + 100); checksum(bytes, proof.offset);},
    bytes => {bytes[proof.offset + 156] = 50; bytes.write('/etc/hosts', proof.offset + 157); checksum(bytes, proof.offset);},
    bytes => {bytes[proof.offset + 156] = 51; checksum(bytes, proof.offset);},
    bytes => {bytes.fill(0, proof.offset, proof.offset + 100); bytes.write('../escape', proof.offset); checksum(bytes, proof.offset);},
    bytes => {bytes[proof.offset] ^= 1;},
    bytes => {bytes[proof.payload + proof.size] = 1;},
    bytes => {bytes[parsed.terminator + 512] = 1;},
    bytes => {bytes.fill(0, proof.offset, proof.offset + 100); bytes.write('evidence-manifest.json', proof.offset); checksum(bytes, proof.offset);},
    bytes => {bytes.fill(0, proof.offset, proof.offset + 100); bytes.write('unregistered', proof.offset); checksum(bytes, proof.offset);},
    bytes => {bytes.write((proof.size + 1).toString(8).padStart(11, '0') + '\0', proof.offset + 124); checksum(bytes, proof.offset);},
  ];
  for (const [index, mutate] of mutations.entries()) {
    const bytes = Buffer.from(original); mutate(bytes);
    const path = put(f.root, `mutant-${index}.tar`, bytes);
    assert.throws(() => unit.unitArchive(path, files, directories), undefined, `archive mutant ${index}`);
  }
  const valid = unit.unitArchive(f.output, files, directories); closeSync(valid.fd);
  const alias = join(f.root, 'archive-alias'); symlinkSync(f.output, alias);
  assert.throws(() => unit.unitArchive(alias, files, directories), /alias/);
});

test('publication revalidates the linked inode and removes only its exact own failed output', async t => {
  for (const foreignReplacement of [false, true]) {
    const f = fixture(t), key = 'prismpm.binary.link.unit.' + ++unitSequence;
    globalThis[Symbol.for(key)] = (command, args, options) => {
      execFileSync(command, args, options);
      if (foreignReplacement) {rmSync(f.output); writeFileSync(f.output, 'foreign replacement');}
      else {chmodSync(f.output, 0o644); writeFileSync(f.output, 'changed linked archive');}
    };
    t.after(() => delete globalThis[Symbol.for(key)]);
    const unit = await unitPackager(t, () => {}, source => source.replace("execFileSync('/usr/bin/ln',", `globalThis[Symbol.for(${JSON.stringify(key)})]('/usr/bin/ln',`));
    assert.throws(() => unit.packBinaryEvidence(f.artifact, f.output, f.selected, {provider: f.provider, selection: f.selection}, f.sourceRoot));
    if (foreignReplacement) assert.equal(readFileSync(f.output, 'utf8'), 'foreign replacement');
    else assert(!existsSync(f.output));
  }
});

test('mutation during the final original-byte reread cannot escape final published custody', async t => {
  const f = fixture(t), key = 'prismpm.binary.final-read.unit.' + ++unitSequence;
  globalThis[Symbol.for(key)] = () => {chmodSync(f.output, 0o644); writeFileSync(f.output, 'late mutation after archive validation');};
  t.after(() => delete globalThis[Symbol.for(key)]);
  const unit = await unitPackager(t, () => {}, source => source.replace('recheck(raw); checkAuthority(authority); checkProvenance(provider);',
    `recheck(raw); checkAuthority(authority); checkProvenance(provider); globalThis[Symbol.for(${JSON.stringify(key)})]();`));
  assert.throws(() => unit.packBinaryEvidence(f.artifact, f.output, f.selected, {provider: f.provider, selection: f.selection}, f.sourceRoot));
  assert(!existsSync(f.output));
});

test('retention producer preserves all original bytes, validates original readers, and creates exclusively', async t => {
  for (const language of ['1.1', '1.2']) {
    const f = fixture(t, context('arm64', language)); rmSync(join(f.artifact, 'retention.json'));
    const before = new Map(f.retention.files.map(row => [row.path, readFileSync(join(f.artifact, row.path))]));
    let calls = 0; const unit = await unitPackager(t, () => calls++);
    const receipt = unit.retainBinaryEvidence(f.artifact, {...f.selected, run_attempt: '1'}, f.sourceRoot);
    assert.equal(calls, 2); assert.deepEqual(receipt, f.retention);
    for (const [path, bytes] of before) assert(readFileSync(join(f.artifact, path)).equals(bytes));
    assert.throws(() => unit.retainBinaryEvidence(f.artifact, f.selected, f.sourceRoot));
    const manifest = unit.packBinaryEvidence(f.artifact, f.output, f.selected, {provider: f.provider, selection: f.selection}, f.sourceRoot);
    assert.equal(manifest.transport.selected.run_attempt, '1'); assert.equal(manifest.run_attempt, '2');
    assert.equal(manifest.transport.selected.owner, receipt.owner);
    for (const [name, path] of [['provider-metadata.json', f.provider], ['artifact-selection.json', f.selection]]) {
      assert(execFileSync('/usr/bin/tar', ['-xOf', f.output, name]).equals(readFileSync(path)));
    }
  }
});

test('provider-selected execution rejects foreign, future, failed, duplicate and missing provenance', async t => {
  const unit = await unitPackager(t);
  const cases = [
    f => {f.metadata.jobs[0].run_id = 9876;},
    f => {f.metadata.jobs[0].run_attempt = 3;},
    f => {f.metadata.jobs[0].conclusion = 'failure';},
    f => {f.metadata.jobs[0].status = 'in_progress';},
    f => {f.metadata.jobs.push({...f.metadata.jobs[0]});},
    f => {f.metadata.artifacts.push({...f.metadata.artifacts[0], id: 53});},
    f => {f.metadata.artifacts.pop();},
    f => {f.metadata.artifacts[0].workflow_run.id = 9876;},
    f => {f.metadata.artifacts[0].workflow_run.head_sha = 'f'.repeat(40);},
    f => {f.metadata.artifacts[0].digest = 'sha256:' + 'f'.repeat(64);},
    f => {f.metadata.jobs[0].id = 100;},
    f => {f.metadata.artifacts[0].id = 100;},
  ];
  for (const change of cases) {
    const f = fixture(t); change(f); writeFileSync(f.provider, JSON.stringify(f.metadata));
    assert.throws(() => unit.packBinaryEvidence(f.artifact, f.output, f.selected, {provider: f.provider, selection: f.selection}, f.sourceRoot));
    assert(!existsSync(f.output));
  }
});

test('transport retention cannot substitute run, attempt, owner, context, raw bytes or self-reported selection', async t => {
  const unit = await unitPackager(t);
  const cases = [
    f => {f.retention.run_id = '9876';},
    f => {f.retention.run_attempt = '2';},
    f => {f.retention.run_attempt = '3';},
    f => {f.retention.owner = 'binary-language-1.2-sdk-sdk-amd64';},
    f => {f.retention.job_name = 'unrelated';},
    f => {f.retention.source_revision = 'f'.repeat(40);},
    f => {f.retention.image_reference = image.replace('b'.repeat(64), 'f'.repeat(64));},
    f => {f.retention.architecture = 'arm64';},
    f => {f.retention.language = '1.2';},
    f => {f.retention.files.pop();},
    f => {f.retention.files[0].sha256 = 'f'.repeat(64);},
    f => {put(f.proof, 'proof.prism', 'changed original proof bytes');},
    f => {put(f.artifact, f.name + '.log', 'changed original log');},
  ];
  for (const change of cases) {
    const f = fixture(t); change(f); put(f.artifact, 'retention.json', canonical(f.retention) + '\n');
    assert.throws(() => unit.packBinaryEvidence(f.artifact, f.output, f.selected, {provider: f.provider, selection: f.selection}, f.sourceRoot));
    assert(!existsSync(f.output));
  }
  const f = fixture(t), selection = JSON.parse(readFileSync(f.selection)); selection.artifacts[0].id = '999';
  writeFileSync(f.selection, canonical(selection));
  assert.throws(() => unit.packBinaryEvidence(f.artifact, f.output, f.selected, {provider: f.provider, selection: f.selection}, f.sourceRoot));
  const missing = fixture(t); rmSync(join(missing.artifact, 'retention.json'));
  assert.throws(() => unit.packBinaryEvidence(missing.artifact, missing.output, missing.selected, {provider: missing.provider, selection: missing.selection}, missing.sourceRoot));
});

test('producer refusal or changed original bytes never leave a retention envelope', async t => {
  for (const mode of ['refused', 'changed']) {
    const f = fixture(t); rmSync(join(f.artifact, 'retention.json')); let calls = 0;
    const unit = await unitPackager(t, () => {
      if (mode === 'refused') throw Error('original reader refused');
      if (++calls === 2) put(f.artifact, f.name + '.log', 'late bytes');
    });
    assert.throws(() => unit.retainBinaryEvidence(f.artifact, f.selected, f.sourceRoot));
    assert(!existsSync(join(f.artifact, 'retention.json')));
  }
});

test('provider, selection and retention mutations during original readers or final sweep fail closed', async t => {
  for (const member of ['provider', 'selection', 'retention']) {
    const f = fixture(t); let calls = 0;
    const unit = await unitPackager(t, () => {
      if (++calls === 2) writeFileSync(member === 'retention' ? join(f.artifact, 'retention.json') : f[member], '{}');
    });
    assert.throws(() => unit.packBinaryEvidence(f.artifact, f.output, f.selected, {provider: f.provider, selection: f.selection}, f.sourceRoot));
    assert(!existsSync(f.output));
  }
  for (const member of ['provider', 'selection', 'retention']) {
    const f = fixture(t), key = 'prismpm.binary.provenance.final.' + ++unitSequence;
    globalThis[Symbol.for(key)] = () => writeFileSync(member === 'retention' ? join(f.artifact, 'retention.json') : f[member], '{}');
    t.after(() => delete globalThis[Symbol.for(key)]);
    const unit = await unitPackager(t, () => {}, source => source.replace('    checkRawCustody(raw); checkProvenanceCustody(provider);',
      `    globalThis[Symbol.for(${JSON.stringify(key)})](); checkRawCustody(raw); checkProvenanceCustody(provider);`));
    assert.throws(() => unit.packBinaryEvidence(f.artifact, f.output, f.selected, {provider: f.provider, selection: f.selection}, f.sourceRoot));
    assert(!existsSync(f.output));
  }
});

test('production entry point has no proof-reader replacement switch', () => {
  const source = readFileSync(new URL('./binary-sdk-release-evidence.mjs', import.meta.url), 'utf8');
  assert(source.includes('const validateProof = checkAccepted;'));
  assert(!source.includes('NODE_TEST_CONTEXT')); assert(!source.includes('PRISMPM_BINARY_EVIDENCE'));
  assert.equal(packBinaryEvidence.length, 4);
  assert.equal(retainBinaryEvidence.length, 2);
});

const semanticHelper = process.env.PRISMPM_SEMANTIC_SOURCE;
const cargo = spawnSync('cargo', ['--version'], {encoding: 'utf8', timeout: 5000});
test('original proof reader integration with parser data (not genuine SDK acceptance)',
  {skip: !semanticHelper && cargo.error ? 'Cargo and the source-owned semantic helper are unavailable; original checkAccepted cannot execute runtimeAudit' : false}, t => {
    const f = fixture(t); rmSync(f.proof, {recursive: true}); mkdirSync(f.proof);
    const authority = {archive_sha256: hash(readFileSync(join(f.sourceRoot, 'vendor/lean4-prod/lean.tar'))),
      toolchain: readFileSync(join(f.sourceRoot, 'lean-toolchain'), 'utf8').trim()};
    const receipt = parserFixture(f.proof, undefined, undefined, authority, '1.1');
    put(f.proof, 'receipt.json', canonical(receipt));
    put(f.artifact, f.name + '/result.json', canonical({...f.result, build_id: receipt.build_id, attestation_id: receipt.attestation_id}));
    rmSync(join(f.artifact, 'retention.json'));
    retainBinaryEvidence(f.artifact, {...f.selected, run_attempt: '1'}, f.sourceRoot);
    packBinaryEvidence(f.artifact, f.output, f.selected, {provider: f.provider, selection: f.selection}, f.sourceRoot);
    assert(existsSync(f.output));
  });
