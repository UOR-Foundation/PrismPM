// Recording-only source owners. No genuine SDK or complete V&V owner executes.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import childProcess from 'node:child_process';
import fs from 'node:fs';
import {syncBuiltinESMExports} from 'node:module';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import test from 'node:test';
import {artifactName, releaseContext, selectArtifacts} from './release-gate-evidence.mjs';
import {bootstrapFixture} from './sdk-bootstrap-retention-fixture.mjs';
import {bootstrapNames, canonical, hash} from './sdk-bootstrap-retention.mjs';
import {assembleSourceParts, sourcePartCommand, validateSourcePart} from './source-vv-parts.mjs';
import {assembleVvEvidence, parts} from './vv-assemble.mjs';
import {cargoInventoryParts} from './vv-cargo-inventory.mjs';
import {recordingCargoInventoryParts} from './vv-cargo-inventory.test.mjs';

const revision = 'a'.repeat(40), context = releaseContext(revision, 'amd64', null, '123', '3');
const describe = (path, bytes) => ({path, byte_length: bytes.length, sha256: hash(bytes)});
const bytes = value => Buffer.from(canonical(value));
function retainedFixture(binding = context) {
  const files = new Map(), inventories = recordingCargoInventoryParts(binding.source_revision);
  for (const [index, part] of parts.entries()) {
    const prefix = `source-${part}`, stdout = Buffer.from(`recording ${part}\0\xff`), stderr = Buffer.from('original recording error\n');
    files.set(`${prefix}.stdout`, stdout); files.set(`${prefix}.stderr`, stderr);
    files.set(`${prefix}.json`, bytes({command: 'just', arguments: ['vv-part', part], status: 0, signal: null,
      descendants_reaped: true, capture_failed: false, orphaned: false, overflow: false, interrupted: null,
      stdout: describe(`${prefix}.stdout`, stdout), stderr: describe(`${prefix}.stderr`, stderr)}));
    const vv = bytes({commit: binding.source_revision, part, schema: 'prismpm/vv-part/1', status: 'passed'});
    files.set(`${prefix}-vv.json`, vv);
    let bootstrap = null, inventory = null;
    if (part === 'source') bootstrap = {run: 1, files: [...bootstrapFixture(binding.source_revision).files].map(([name, raw]) => {
      const path = `run-1-${name}`; files.set(path, raw); return describe(path, raw);
    })};
    if (cargoInventoryParts.includes(part)) inventory = {files: [...inventories.get(part)].sort(([a], [b]) => a.localeCompare(b, 'en')).map(([name, raw]) => {
      const path = `${prefix}-inventory-${name}`; files.set(path, raw); return describe(path, raw);
    }).sort((a, b) => a.path < b.path ? -1 : a.path > b.path ? 1 : 0)};
    files.set(`${prefix}-result.json`, bytes({schema: 'prismpm/source-vv-retention/3',
      scope: 'original-source-part-with-executable-inventory-only', ...binding, run_attempt: String(index % 2 + 1), part,
      vv: describe(`${prefix}-vv.json`, vv), bootstrap, inventory}));
  }
  return {files, inventories};
}
function providerFixture(binding = context) {
  const jobs = [], artifacts = [];
  for (const [index, part] of parts.entries()) {
    const owner = `source-vv-part-${part}`, attempt = index % 2 + 1;
    jobs.push({id: 100 + index, run_id: Number(binding.run_id), run_attempt: attempt, name: owner,
      head_sha: binding.source_revision, status: 'completed', conclusion: 'success'});
    artifacts.push({id: 200 + index, name: artifactName(owner, {...binding, run_attempt: String(attempt)}),
      digest: 'sha256:' + String(index % 10).repeat(64), expired: false, size_in_bytes: 10,
      workflow_run: {id: Number(binding.run_id), head_sha: binding.source_revision}});
  }
  return {jobs, artifacts};
}
const selectionFor = (provider, binding = context) => selectArtifacts(provider, binding, parts.map(part => `source-vv-part-${part}`));
const mutateJson = (files, name, mutate) => {const value = JSON.parse(files.get(name)); mutate(value); files.set(name, bytes(value));};
const temp = t => {const path = fs.mkdtempSync(join(tmpdir(), 'source-vv-recording-')); t.after(() => fs.rmSync(path, {recursive: true, force: true})); return path;};
function repositoryFixture(t) {
  const root = temp(t); fs.mkdirSync(join(root, 'bin')); fs.mkdirSync(join(root, 'target'));
  fs.writeFileSync(join(root, '.gitignore'), 'target/\n'); fs.writeFileSync(join(root, 'tracked'), 'clean source\n');
  fs.mkdirSync(join(root, 'crates/conformance/tests'), {recursive: true});
  fs.writeFileSync(join(root, 'crates/conformance/tests/conformance.rs'),
    recordingCargoInventoryParts(revision).get('source').get('conformance-register.rs'));
  const fakeJust = `#!${process.execPath}
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
assert.equal(process.argv[2],'vv-part');const part=process.argv[3];assert.equal(process.argv.length,4);
const payload=JSON.parse(fs.readFileSync('target/recording.json'));
fs.writeFileSync('target/invocation.json',JSON.stringify(process.argv.slice(2)));
process.stdout.write(Buffer.from([0,255,10]));process.stderr.write('recording stderr\\n');
if(process.env.RECORD_STATUS)process.exit(Number(process.env.RECORD_STATUS));
if(process.env.RECORD_MODE==='source-drift')fs.writeFileSync('tracked','changed source');
if(process.env.RECORD_MODE!=='missing-part')fs.writeFileSync('target/vv-part-'+part+'.json',JSON.stringify({commit:payload.revision,part:process.env.RECORD_MODE==='wrong-part'?'gates':part,schema:'prismpm/vv-part/1',status:'passed'}));
if(payload.inventory.length&&process.env.RECORD_MODE!=='missing-inventory'){
 const directory='target/vv-inventory-'+part;fs.mkdirSync(directory);
 for(const[name,data]of payload.inventory)fs.writeFileSync(path.join(directory,name),Buffer.from(data,'base64'));
 if(process.env.RECORD_MODE==='inventory-alias')fs.symlinkSync(path.resolve('tracked'),path.join(directory,'aliased.json'));
}
for(const[name,data]of payload.bootstrap)fs.writeFileSync(path.join('target',name),Buffer.from(data,'base64'));
if(process.env.RECORD_MODE==='bad-bootstrap')fs.writeFileSync('target/bootstrap-evidence.json','{}');
`;
  fs.writeFileSync(join(root, 'bin/just'), fakeJust, {mode: 0o755});
  const git = args => execFileSync('/usr/bin/git', args, {cwd: root, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe']}).trim();
  git(['init', '--quiet']); git(['config', 'user.email', 'recording@example.invalid']); git(['config', 'user.name', 'Recording fixture']);
  git(['add', '.']); git(['commit', '--quiet', '-m', 'recording source only']);
  return {root, revision: git(['rev-parse', 'HEAD']), git};
}
function setPayload(fixture, part, inventories) {
  fs.writeFileSync(join(fixture.root, 'target/recording.json'), canonical({revision: fixture.revision,
    inventory: [...inventories.get(part)].map(([name, raw]) => [name, raw.toString('base64')]),
    bootstrap: part === 'source' ? [...bootstrapFixture(fixture.revision).files].map(([name, raw]) => [name, raw.toString('base64')]) : []}));
}
async function withRecordingPath(t, fixture, body, extra = {}) {
  const saved = {...process.env}; Object.assign(process.env, {PATH: join(fixture.root, 'bin') + ':' + process.env.PATH}, extra);
  // The actual host rejects the independent Linux command owner's admission.
  // At its spawn boundary only, use a recording owner for our short-lived fake
  // just command. Retirement authority is tested by the original owner suite.
  const owner = join(fixture.root, 'target/recording-owner.py'), originalSpawn = childProcess.spawn;
  fs.writeFileSync(owner, `import json,os,subprocess,sys\ndef emit(row):\n os.write(3,(json.dumps(row)+'\\n').encode())\nprocess=subprocess.Popen(sys.argv[1:],close_fds=True)\nemit({'event':'started','pid':process.pid})\nstatus=process.wait()\nemit({'event':'completed','cleanup_verified':True,'error':None,'orphaned':False,'signal':None,'status':status})\n`);
  t.mock.method(childProcess, 'spawn', (command, args, options) => {
    assert.equal(command, '/usr/bin/python3'); assert(args[2].endsWith('/release-command-owner.py'));
    return originalSpawn(command, [...args.slice(0, 2), owner, ...args.slice(3)], options);
  }); syncBuiltinESMExports();
  try {return await body();} finally {
    t.mock.restoreAll(); syncBuiltinESMExports();
    for (const key of Object.keys(process.env)) if (!(key in saved)) delete process.env[key]; Object.assign(process.env, saved);
  }
}
const runArgs = (fixture, part, attempt = '1') => ['source-part-run', fixture.root, join(fixture.root, 'target/source-vv-evidence'),
  fixture.revision, 'amd64', '-', '123', attempt, part];
function writeAggregate(fixture, files, binding) {
  const output = join(fixture.root, 'target/source-vv-evidence'); fs.mkdirSync(output);
  for (const [name, raw] of files) fs.writeFileSync(join(output, name), raw);
  const provider = providerFixture(binding), selection = selectionFor(provider, binding);
  const providerPath = join(fixture.root, 'target/provider.json'), selectionPath = join(fixture.root, 'target/selection.json');
  fs.writeFileSync(providerPath, JSON.stringify(provider)); fs.writeFileSync(selectionPath, canonical(selection) + '\n');
  return {output, providerPath, selectionPath};
}
const assembleArgs = (fixture, paths) => ['source-part-assemble', fixture.root, paths.output, fixture.revision, 'amd64', '-', '123', '3', paths.providerPath, paths.selectionPath];

test('all fifteen originals with selected earlier attempts assemble unchanged V&V v1 bytes and eight inventories', () => {
  const {files} = retainedFixture(), provider = providerFixture(), original = new Map(files);
  const actual = assembleSourceParts(files, context, provider, selectionFor(provider));
  assert(actual.equals(assembleVvEvidence(new Map(parts.map(part => [part, files.get(`source-${part}-vv.json`)])), revision)));
  assert.deepEqual(files, original);
  for (const part of parts) assert.equal(validateSourcePart(files, part, context).inventory.size > 0, cargoInventoryParts.includes(part));
});

test('original provider selection refuses foreign, failed, duplicate, future, missing and self-reported owners', () => {
  const {files} = retainedFixture();
  for (const mutate of [
    p => {p.jobs[0].run_id++;}, p => {p.jobs[0].head_sha = 'f'.repeat(40);},
    p => {p.jobs[0].run_attempt = 4;}, p => {p.jobs[0].status = 'in_progress';},
    p => {p.jobs[0].conclusion = 'failure';}, p => {p.jobs[0].name = 'unowned';},
    p => {p.jobs.push({...p.jobs[0], id: 999});}, p => {p.jobs.shift();},
    p => {p.jobs.push({...p.jobs[0], id: 999, run_attempt: 3, conclusion: 'failure'});},
    p => {p.artifacts[0].workflow_run.id++;}, p => {p.artifacts[0].workflow_run.head_sha = 'f'.repeat(40);},
    p => {p.artifacts[0].expired = true;}, p => {p.artifacts[0].digest = null;},
    p => {p.artifacts[0].name = p.artifacts[0].name.replace(/--1$/, '--4');},
    p => {p.artifacts.push({...p.artifacts[0], id: 999});}, p => {p.artifacts.pop();},
    p => {p.artifacts[1].id = p.artifacts[0].id;},
  ]) {
    const provider = providerFixture(), selection = selectionFor(provider); mutate(provider);
    assert.throws(() => assembleSourceParts(files, context, provider, selection));
  }
  const provider = providerFixture(), selection = selectionFor(provider); selection.artifacts[0].id = '999';
  assert.throws(() => assembleSourceParts(files, context, provider, selection), /selection/);
});

test('source retention refuses missing, extra, changed or coherently resealed original bytes and inventory', () => {
  const provider = providerFixture();
  for (const mutate of [
    f => {f.delete('source-gates.stdout');}, f => {f.set('extra.json', bytes({}));},
    f => {f.set('source-gates.stderr', Buffer.from('changed'));},
    f => mutateJson(f, 'source-gates.json', r => {r.status = 1;}),
    f => mutateJson(f, 'source-gates.json', r => {r.arguments = ['vv'];}),
    f => mutateJson(f, 'source-gates.json', r => {r.descendants_reaped = false;}),
    f => mutateJson(f, 'source-gates-result.json', r => {r.run_attempt = '4';}),
    f => mutateJson(f, 'source-gates-result.json', r => {r.run_attempt = '1';}),
    f => mutateJson(f, 'source-gates-result.json', r => {r.source_revision = 'f'.repeat(40);}),
    f => mutateJson(f, 'source-gates-result.json', r => {r.image_reference = 'ghcr.io/uor-foundation/prismpm-sdk@sha256:' + 'b'.repeat(64);}),
    f => mutateJson(f, 'source-gates-result.json', r => {r.inventory = {files: []};}),
    f => mutateJson(f, 'source-source-result.json', r => {r.inventory = null;}),
    f => {f.delete('run-1-bootstrap-evidence.json');},
    f => {f.set('source-gates-vv.json', bytes({commit: revision, part: 'source', schema: 'prismpm/vv-part/1', status: 'passed'}));},
    f => {const key = [...f.keys()].find(name => name.startsWith('source-source-inventory-')); f.set(key, Buffer.from('changed'));},
    f => mutateJson(f, 'source-source-result.json', r => {r.inventory.files.reverse();}),
    f => mutateJson(f, 'source-source-result.json', r => {r.inventory.files.push(r.inventory.files[0]);}),
  ]) {
    const {files} = retainedFixture(); mutate(files);
    assert.throws(() => assembleSourceParts(files, context, provider, selectionFor(provider)));
  }
});

test('actual recording just owners retain binary pipes, bootstrap and inventories for every source part', async t => {
  const fixture = repositoryFixture(t), binding = {...context, source_revision: fixture.revision};
  const inventories = recordingCargoInventoryParts(fixture.revision), aggregate = new Map();
  await withRecordingPath(t, fixture, async () => {
    for (const [index, part] of parts.entries()) {
      setPayload(fixture, part, inventories); await sourcePartCommand(runArgs(fixture, part, String(index % 2 + 1)));
      assert.deepEqual(JSON.parse(fs.readFileSync(join(fixture.root, 'target/invocation.json'))), ['vv-part', part]);
      const output = join(fixture.root, 'target/source-vv-evidence');
      assert.deepEqual(fs.readFileSync(join(output, `source-${part}.stdout`)), Buffer.from([0, 255, 10]));
      for (const name of fs.readdirSync(output)) {assert(!aggregate.has(name)); aggregate.set(name, fs.readFileSync(join(output, name)));}
      fs.rmSync(output, {recursive: true}); fs.rmSync(join(fixture.root, `target/vv-part-${part}.json`));
      fs.rmSync(join(fixture.root, `target/vv-inventory-${part}`), {recursive: true, force: true});
    }
  });
  const provider = providerFixture(binding);
  assert.equal(JSON.parse(assembleSourceParts(aggregate, binding, provider, selectionFor(provider, binding))).status, 'passed');
});

test('the real release-gate CLI enters the source runner and exits without a module-cycle hang', t => {
  const fixture = repositoryFixture(t); setPayload(fixture, 'gates', recordingCargoInventoryParts(fixture.revision));
  const script = new URL('./release-gate-evidence.mjs', import.meta.url).pathname;
  const receipt = join(fixture.root, 'target/source-vv-evidence/source-gates-result.json');
  try {
    execFileSync(process.execPath, [script, ...runArgs(fixture, 'gates')], {cwd: fixture.root,
      env: {...process.env, PATH: join(fixture.root, 'bin') + ':' + process.env.PATH}, timeout: 20000, stdio: 'pipe'});
    assert.equal(JSON.parse(fs.readFileSync(receipt)).schema, 'prismpm/source-vv-retention/3');
  } catch (error) {
    assert(Number.isInteger(error.status) && error.status !== 0, 'CLI must exit rather than time out on a module cycle');
    assert.match(error.stderr.toString(), /gate process supervision failed|gate process retirement is uncertain/);
    assert(!fs.existsSync(receipt), 'unsupported real owner admission cannot publish retention');
  }
});

test('failed recording commands, source drift, missing owners and malformed inventory never publish retention', async t => {
  for (const [part, extra] of [['gates', {RECORD_STATUS: '19'}], ['gates', {RECORD_MODE: 'missing-part'}],
    ['gates', {RECORD_MODE: 'source-drift'}], ['source', {RECORD_MODE: 'wrong-part'}],
    ['source', {RECORD_MODE: 'missing-inventory'}], ['source', {RECORD_MODE: 'inventory-alias'}],
    ['source', {RECORD_MODE: 'bad-bootstrap'}]]) {
    const fixture = repositoryFixture(t); setPayload(fixture, part, recordingCargoInventoryParts(fixture.revision));
    await withRecordingPath(t, fixture, async () => assert.rejects(sourcePartCommand(runArgs(fixture, part))), extra);
    const captured = JSON.parse(fs.readFileSync(join(fixture.root, `target/source-vv-evidence/source-${part}.json`)));
    assert.equal(captured.status, extra.RECORD_STATUS ? Number(extra.RECORD_STATUS) : 0, 'preserve the original command status');
    assert(!fs.existsSync(join(fixture.root, `target/source-vv-evidence/source-${part}-result.json`)));
  }
});

test('fresh-path admission preserves stale originals, refuses aliases and rejects non-source contexts before commands', async t => {
  for (const kind of ['output', 'inventory', 'part', 'dirty']) {
    const fixture = repositoryFixture(t); setPayload(fixture, 'source', recordingCargoInventoryParts(fixture.revision));
    const stale = kind === 'output' ? 'source-vv-evidence' : kind === 'inventory' ? 'vv-inventory-source' : 'vv-part-source.json';
    if (kind === 'dirty') fs.writeFileSync(join(fixture.root, 'tracked'), 'dirty');
    else fs.writeFileSync(join(fixture.root, 'target', stale), 'untouched stale bytes');
    await withRecordingPath(t, fixture, async () => assert.rejects(sourcePartCommand(runArgs(fixture, 'source'))));
    if (kind !== 'dirty') assert.equal(fs.readFileSync(join(fixture.root, 'target', stale), 'utf8'), 'untouched stale bytes');
    assert(!fs.existsSync(join(fixture.root, 'target/invocation.json')));
  }
  const fixture = repositoryFixture(t), args = runArgs(fixture, 'gates'); args[5] = 'ghcr.io/uor-foundation/prismpm-sdk@sha256:' + 'b'.repeat(64);
  await assert.rejects(sourcePartCommand(args), /source-only/);
});

test('assembler CLI retains originals and creates identical canonical receipts exclusively', async t => {
  const fixture = repositoryFixture(t), binding = {...context, source_revision: fixture.revision};
  const {files} = retainedFixture(binding), paths = writeAggregate(fixture, files, binding);
  await sourcePartCommand(assembleArgs(fixture, paths));
  const receipt = fs.readFileSync(join(paths.output, 'source-vv.json'));
  assert.deepEqual(fs.readFileSync(join(fixture.root, 'target/vv-evidence.json')), receipt);
  for (const [name, original] of files) assert(fs.readFileSync(join(paths.output, name)).equals(original));
  await assert.rejects(sourcePartCommand(assembleArgs(fixture, paths)), /fresh/);
  assert(fs.readFileSync(join(paths.output, 'source-vv.json')).equals(receipt));
});

test('assembler refuses hardlinks, symbolic aliases, extra raw files and stale final receipts', async t => {
  for (const mode of ['hardlink', 'symlink', 'extra', 'stale']) {
    const fixture = repositoryFixture(t), binding = {...context, source_revision: fixture.revision};
    const paths = writeAggregate(fixture, retainedFixture(binding).files, binding);
    if (mode === 'hardlink') fs.linkSync(join(paths.output, 'source-gates.stdout'), join(fixture.root, 'target/linked'));
    if (mode === 'symlink') {fs.unlinkSync(join(paths.output, 'source-gates.stdout')); fs.symlinkSync(join(fixture.root, 'tracked'), join(paths.output, 'source-gates.stdout'));}
    if (mode === 'extra') fs.writeFileSync(join(paths.output, 'extra.json'), '{}');
    if (mode === 'stale') fs.writeFileSync(join(fixture.root, 'target/vv-evidence.json'), 'stale receipt');
    await assert.rejects(sourcePartCommand(assembleArgs(fixture, paths)));
    assert(!fs.existsSync(join(paths.output, 'source-vv.json')));
    if (mode === 'stale') assert.equal(fs.readFileSync(join(fixture.root, 'target/vv-evidence.json'), 'utf8'), 'stale receipt');
  }
});

test('production capture and assembly bind raw inventory registers to the selected clean source root', async t => {
  for (const operation of ['source-part-run', 'source-part-assemble']) {
    const fixture = repositoryFixture(t);
    fs.appendFileSync(join(fixture.root, 'crates/conformance/tests/conformance.rs'), '\n// independent source fixture register\n');
    fixture.git(['add', 'crates/conformance/tests/conformance.rs']); fixture.git(['commit', '--quiet', '-m', 'independent fixture source']);
    fixture.revision = fixture.git(['rev-parse', 'HEAD']);
    const binding = {...context, source_revision: fixture.revision};
    if (operation === 'source-part-run') {
      setPayload(fixture, 'source', recordingCargoInventoryParts(fixture.revision));
      await withRecordingPath(t, fixture, async () => assert.rejects(sourcePartCommand(runArgs(fixture, 'source')), /register differs from exact checked-out source/));
    } else {
      const {files} = retainedFixture(binding), provider = providerFixture(binding);
      // Internal transport is self-consistent against the module's real register;
      // the production CLI additionally binds the independently selected root.
      assembleSourceParts(files, binding, provider, selectionFor(provider, binding));
      const paths = writeAggregate(fixture, files, binding);
      await assert.rejects(sourcePartCommand(assembleArgs(fixture, paths)), /register differs from exact checked-out source/);
    }
    assert(!fs.existsSync(join(fixture.root, 'target/source-vv-evidence/source-source-result.json')) || operation === 'source-part-assemble');
    assert(!fs.existsSync(join(fixture.root, 'target/source-vv-evidence/source-vv.json')));
    assert(!fs.existsSync(join(fixture.root, 'target/vv-evidence.json')));
  }
});

test('inventory manifests enforce bounded members and original filenames before accepting payloads', () => {
  for (const mutate of [
    (files, result) => {result.inventory.files = Array.from({length: 1025}, () => result.inventory.files[0]);},
    (files, result) => {result.inventory.files[0].path = 'source-source-inventory-../outside';},
    (files, result) => {result.inventory.files[0].path = 'source-source-inventory-' + 'a'.repeat(128);},
    (files, result) => {const path = result.inventory.files[0].path, raw = Buffer.alloc(16 * 1024 ** 2 + 1); files.set(path, raw); result.inventory.files[0] = describe(path, raw);},
  ]) {
    const {files} = retainedFixture(), result = JSON.parse(files.get('source-source-result.json'));
    mutate(files, result); files.set('source-source-result.json', bytes(result));
    assert.throws(() => validateSourcePart(files, 'source', context));
  }
});

test('a mutation during final raw reread cannot leave either published receipt', async t => {
  for (const phase of [2, 3]) {
  const fixture = repositoryFixture(t), binding = {...context, source_revision: fixture.revision};
  const paths = writeAggregate(fixture, retainedFixture(binding).files, binding), originalRead = fs.readSync;
  let reads = 0, mutated = false, published = false;
  t.mock.method(fs, 'readSync', (...args) => {
    const count = originalRead(...args);
    const path = fs.readlinkSync(`/proc/self/fd/${args[0]}`);
    if (path === join(paths.output, 'source-gates.stdout') && count && ++reads === phase) {
      published = fs.existsSync(join(paths.output, 'source-vv.json'));
      fs.writeFileSync(join(paths.output, 'source-gates.stderr'), 'late mutation'); mutated = true;
    }
    return count;
  }); syncBuiltinESMExports();
  try {await assert.rejects(sourcePartCommand(assembleArgs(fixture, paths))); assert(mutated); assert.equal(published, phase === 3);}
  finally {t.mock.restoreAll(); syncBuiltinESMExports();}
  assert(!fs.existsSync(join(paths.output, 'source-vv.json')));
  assert(!fs.existsSync(join(fixture.root, 'target/vv-evidence.json')));
  }
});

test('a late metadata reread cannot hide mutation of an already reread original payload', async t => {
  const fixture = repositoryFixture(t), binding = {...context, source_revision: fixture.revision};
  const paths = writeAggregate(fixture, retainedFixture(binding).files, binding), originalRead = fs.readSync;
  let reads = 0, mutated = false;
  t.mock.method(fs, 'readSync', (...args) => {
    const count = originalRead(...args);
    if (fs.readlinkSync(`/proc/self/fd/${args[0]}`) === paths.providerPath && count && ++reads === 3) {
      assert(fs.existsSync(join(paths.output, 'source-vv.json')));
      fs.writeFileSync(join(paths.output, 'source-gates.stdout'), 'changed after final raw read'); mutated = true;
    }
    return count;
  }); syncBuiltinESMExports();
  try {await assert.rejects(sourcePartCommand(assembleArgs(fixture, paths))); assert(mutated);}
  finally {t.mock.restoreAll(); syncBuiltinESMExports();}
  assert(!fs.existsSync(join(paths.output, 'source-vv.json')));
  assert(!fs.existsSync(join(fixture.root, 'target/vv-evidence.json')));
});
