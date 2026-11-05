// Original Cargo/harness inventory validation only. These records do not grant
// acceptance without the unchanged owning command, source and provider gates.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {posix} from 'node:path';

export const cargoInventoryParts = Object.freeze(['source', 'conformance', 'ho-oc', 'st-sy', 'dk-0-1', 'dk-2', 'dk-3', 'dk-4']);
export const vvInventoryParts = Object.freeze([...cargoInventoryParts, 'gates',
  'standalone-projection', 'standalone-reservation', 'standalone-payload',
  'standalone-recovery-frames', 'standalone-journal-recovery', 'standalone-retention'].sort());
const packages = Object.freeze({'prismpm': 'crates/prismpm', 'repo-conformance': 'crates/conformance', 'repo-model': 'crates/model', xtask: 'xtask'});
const limits = Object.freeze({files: 1024, file: 16 * 1024 ** 2, total: 128 * 1024 ** 2});
const canonical = value => JSON.stringify(value, (_, row) => row && typeof row === 'object' && !Array.isArray(row)
  ? Object.fromEntries(Object.keys(row).sort().map(key => [key, row[key]])) : row);
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const equal = (actual, expected, message) => assert.equal(canonical(actual), canonical(expected), message);
const keys = (value, names) => {assert(value && typeof value === 'object' && !Array.isArray(value)); assert.deepEqual(Object.keys(value).sort(), [...names].sort(), 'closed inventory fields');};
const unique = (rows, message) => {assert.equal(new Set(rows).size, rows.length, message); return [...rows].sort();};
const identifier = value => {assert.equal(typeof value, 'string'); assert.match(value, /^[A-Za-z0-9_][A-Za-z0-9_.-]{0,127}$/); return value;};
const absolute = value => {
  assert.equal(typeof value, 'string'); assert(value.length <= 4096 && value.startsWith('/') && value !== '/' && !/[\0\r\n\\]/.test(value));
  assert.equal(posix.normalize(value), value, 'canonical absolute Cargo path'); return value;
};
const relative = value => {assert.equal(typeof value, 'string'); assert(value.length <= 4096 && /^[A-Za-z0-9_.+/-]+$/.test(value)); assert(value.split('/').every(row => row && row !== '.' && row !== '..'), 'closed relative Cargo path'); return value;};
const keyOf = target => `${target.package}/${target.kind}/${target.name}`;
const ownerOf = target => {
  if (target.package !== 'repo-conformance') return 'source';
  if (target.kind === 'lib' || target.kind === 'bin') return 'conformance';
  assert(target.kind === 'test' && target.name === 'conformance', 'unowned repo-conformance test target'); return 'integration';
};
const selection = owner => owner === 'source' ? ['--workspace', '--exclude', 'repo-conformance']
  : owner === 'conformance' ? ['--package', 'repo-conformance', '--lib', '--bins'] : ['--package', 'repo-conformance', '--test', 'conformance'];
const cargoArgs = selected => ['cargo', 'test', ...selected, '--all-features', '--locked', '--offline'];
const namedFilters = Object.freeze({'ho-oc': ['conformance_ho_', 'conformance_oc_'], 'st-sy': ['conformance_st_', 'conformance_sy_'],
  'dk-0-1': ['conformance_dk_0', 'conformance_dk_1'], 'dk-2': ['conformance_dk_2'], 'dk-3': ['conformance_dk_3'], 'dk-4': ['conformance_dk_40', 'conformance_dk_41']});
function owns(name, part) {
  if (part === 'conformance') return !Object.keys(namedFilters).some(part => owns(name, part));
  if (part === 'dk-4') return name === 'conformance_dk_40' || name === 'conformance_dk_41';
  return namedFilters[part].some(prefix => name.startsWith(prefix));
}
function filters(part) {return part === 'conformance' ? Object.values(namedFilters).flatMap(values => values.flatMap(value => ['--skip', value])) : namedFilters[part];}

function register(bytes) {
  const text = new TextDecoder('utf-8', {fatal: true}).decode(bytes), names = [];
  for (const line of text.split(/\r?\n/)) {
    const trimmed = line.trim(); if (!trimmed.startsWith('test_case!')) continue;
    const match = /^test_case!\((conformance_[a-z0-9_]+), "([A-Z][A-Z0-9]*-[0-9]{2})"\);$/.exec(trimmed);
    assert(match && match[1] === 'conformance_' + match[2].toLowerCase().replaceAll('-', '_'), 'exact conformance registration');
    names.push(match[1]);
  }
  assert(names.length > 0, 'empty conformance register'); unique(names, 'duplicate conformance registration');
  for (const name of names) if (name.startsWith('conformance_dk_')) assert(Object.keys(namedFilters).some(part => owns(name, part)), 'unowned DK conformance test');
  const union = [];
  for (const part of cargoInventoryParts.filter(part => part !== 'source')) {
    const selected = names.filter(name => owns(name, part)); assert(selected.length > 0, 'empty conformance part'); union.push(...selected);
  }
  assert.deepEqual(unique(union, 'overlapping conformance partition'), [...names].sort()); return names.sort();
}

function metadata(bytes) {
  const value = JSON.parse(new TextDecoder('utf-8', {fatal: true}).decode(bytes));
  assert.equal(value.version, 1); const root = absolute(value.workspace_root), targetDirectory = absolute(value.target_directory);
  assert([root + '/target/vv-workspace', root + '/target/vv-workspace-alternate'].includes(targetDirectory), 'original isolated Cargo target directory');
  assert(Array.isArray(value.workspace_members) && value.workspace_members.length === 4);
  const members = unique(value.workspace_members.map(value => {assert.equal(typeof value, 'string'); assert(value.length <= 8192); return value;}), 'duplicate workspace member');
  assert(Array.isArray(value.packages) && value.packages.length === members.length, 'complete no-deps workspace metadata');
  const ids = new Map(), versions = [], targets = new Map(), docs = new Map();
  for (const packageRow of value.packages) {
    assert(members.includes(packageRow.id) && !ids.has(packageRow.id), 'exact workspace package ID');
    const name = identifier(packageRow.name); assert(Object.hasOwn(packages, name), 'closed four-package ownership');
    assert(![...ids.values()].includes(name), 'duplicate workspace package name'); ids.set(packageRow.id, name);
    assert.equal(packageRow.manifest_path, root + '/' + packages[name] + '/Cargo.toml', 'workspace manifest source');
    assert.equal(packageRow.source, null, 'workspace package must be local source');
    assert.equal(typeof packageRow.version, 'string'); assert.match(packageRow.version, /^[0-9]+\.[0-9]+\.[0-9]+(?:[-+][A-Za-z0-9.-]+)?$/);
    versions.push({name, version: packageRow.version, manifest: packages[name] + '/Cargo.toml'});
    assert(Array.isArray(packageRow.targets) && packageRow.targets.length > 0 && packageRow.targets.length <= 256);
    for (const row of packageRow.targets) {
      assert.equal(typeof row.test, 'boolean'); assert.equal(typeof row.doctest, 'boolean');
      if (!row.test && !row.doctest) continue;
      const target = normalizedTarget(name, row, root), key = keyOf(target); ownerOf(target);
      if (row.test) {assert(!targets.has(key), 'duplicate metadata test target'); targets.set(key, target);}
      if (row.doctest) {assert.equal(target.kind, 'lib', 'only library documentation tests'); assert(!docs.has(key), 'duplicate documentation target'); docs.set(key, target);}
    }
  }
  assert.deepEqual([...ids.values()].sort(), Object.keys(packages).sort());
  for (const owner of ['source', 'conformance', 'integration']) assert([...targets.values()].some(target => ownerOf(target) === owner), 'nonempty Cargo target owner');
  assert.equal([...targets.values()].filter(target => ownerOf(target) === 'integration').length, 1, 'one complete conformance integration target');
  for (const owner of ['source', 'conformance']) assert([...docs.values()].some(target => ownerOf(target) === owner), 'nonempty Cargo doc owner');
  return {root, targetDirectory, ids, targets, docs, versions: versions.sort((a, b) => a.name.localeCompare(b.name))};
}
function normalizedTarget(packageName, row, root) {
  assert(Array.isArray(row.kind) && row.kind.length === 1 && ['lib', 'bin', 'test'].includes(row.kind[0]), 'closed test target kind');
  const source = absolute(row.src_path); assert(source.startsWith(root + '/'), 'target outside selected workspace');
  const path = relative(source.slice(root.length + 1)); assert(path.startsWith(packages[packageName] + '/') && path.endsWith('.rs'), 'target outside owning package source');
  return {package: packageName, name: identifier(row.name), kind: row.kind[0], source: path};
}

function artifacts(bytes, data, expected) {
  const text = new TextDecoder('utf-8', {fatal: true}).decode(bytes), result = new Map(); let finished = false;
  for (const line of text.split(/\r?\n/).filter(Boolean)) {
    assert(!finished, 'Cargo records after build completion'); const row = JSON.parse(line);
    if (row.reason === 'build-finished') {assert.equal(row.success, true, 'successful Cargo completion'); finished = true;}
    else if (row.reason === 'compiler-artifact') {
      assert(row.profile && typeof row.profile.test === 'boolean');
      if (!row.profile.test) continue;
      assert(data.ids.has(row.package_id), 'foreign test executable package');
      const target = normalizedTarget(data.ids.get(row.package_id), row.target, data.root), key = keyOf(target);
      assert(expected.has(key), 'unexpected Cargo test target'); equal(target, data.targets.get(key), 'Cargo target metadata differs');
      const executable = absolute(row.executable); assert(executable.startsWith(data.targetDirectory + '/'), 'test executable outside isolated Cargo target');
      assert(Array.isArray(row.filenames) && row.filenames.includes(executable), 'original Cargo executable filename');
      assert(!result.has(key), 'duplicate Cargo executable'); result.set(key, {target, executable});
    } else assert(['compiler-message', 'build-script-executed'].includes(row.reason), 'unknown Cargo JSON record');
  }
  assert(finished, 'missing Cargo completion'); assert.deepEqual([...result.keys()].sort(), [...expected].sort(), 'complete Cargo executable target set'); return result;
}
function list(bytes) {
  const rows = new TextDecoder('utf-8', {fatal: true}).decode(bytes).split(/\r?\n/).filter(Boolean), names = [];
  for (const row of rows) {
    const match = /^((?:[A-Za-z_][A-Za-z0-9_]*::)*[A-Za-z_][A-Za-z0-9_]*): (test|benchmark)$/.exec(row);
    assert(match, 'original terse libtest list row'); names.push(match[1]);
  }
  return unique(names, 'duplicate executable test name');
}
function docList(bytes, data, targetKeys) {
  const lines = new TextDecoder('utf-8', {fatal: true}).decode(bytes).split(/\r?\n/).filter(Boolean), sections = [], seen = new Set(); let rows = [];
  for (const line of lines) {
    const summary = /^(0|[1-9][0-9]*) tests?, (0|[1-9][0-9]*) benchmarks?$/.exec(line);
    if (summary) {
      assert.equal(Number(summary[1]), rows.length, 'complete documentation test count'); assert.equal(Number(summary[2]), 0);
      sections.push(rows); rows = []; continue;
    }
    const match = /^(.+\.rs) - .*\(line [1-9][0-9]*\): test$/.exec(line); assert(match, 'original rustdoc list row');
    const source = match[1].startsWith(data.root + '/') ? match[1].slice(data.root.length + 1) : match[1]; relative(source);
    const owning = targetKeys.filter(key => source.startsWith(packages[data.docs.get(key).package] + '/'));
    assert.equal(owning.length, 1, 'documentation test source owner'); assert(!seen.has(line), 'duplicate documentation test'); seen.add(line);
    rows.push({target: owning[0], name: line});
  }
  assert.equal(rows.length, 0, 'documentation list lacks final count'); assert.equal(sections.length, targetKeys.length, 'every metadata doc target must list');
  const nonempty = sections.filter(rows => rows.length).map(rows => {assert.equal(new Set(rows.map(row => row.target)).size, 1, 'mixed documentation owner section'); return rows[0].target;});
  unique(nonempty, 'duplicate documentation target section'); return sections.flat();
}

// A caller may supply independently source-bound register bytes. Production
// defaults to this exact checkout's register; no inventory/CLI field selects it.
export function validateCargoInventory(files, part, revision, expectedRegister) {
  assert.match(revision, /^[0-9a-f]{40}$/); assert(vvInventoryParts.includes(part), 'closed V&V inventory part'); assert(files instanceof Map);
  if (!cargoInventoryParts.includes(part)) {assert.equal(files.size, 0, 'non-Cargo part cannot claim Cargo inventory'); return null;}
  assert(files.size > 0 && files.size <= limits.files, 'bounded complete inventory directory'); const originals = new Map(); let total = 0;
  for (const [name, bytes] of files) {
    assert(typeof name === 'string' && /^[A-Za-z0-9][A-Za-z0-9.-]{0,126}$/.test(name), 'flat original inventory filename');
    assert(Buffer.isBuffer(bytes) && bytes.length <= limits.file, 'bounded original inventory file'); total += bytes.length; assert(total <= limits.total, 'bounded complete inventory bytes'); originals.set(name, Buffer.from(bytes));
  }
  const used = new Set();
  const take = name => {assert(originals.has(name) && !used.has(name), 'missing or reused original inventory file: ' + name); used.add(name); return originals.get(name);};
  const inventoryBytes = take('inventory.json'), value = JSON.parse(new TextDecoder('utf-8', {fatal: true}).decode(inventoryBytes));
  assert(inventoryBytes.equals(Buffer.from(canonical(value))), 'original canonical inventory.json');
  keys(value, ['schema', 'source_revision', 'part', 'metadata', 'cargo', 'harnesses', 'docs', 'register']);
  assert.equal(value.schema, 'prismpm/vv-cargo-inventory/1'); assert.equal(value.source_revision, revision); assert.equal(value.part, part);
  function command(record, label, argv, doc = false, harness = false) {
    keys(record, ['argv', 'stdout', 'stderr', 'status', ...(doc ? ['target_keys'] : []), ...(harness ? ['execution_path'] : [])]);
    if (harness) {assert.equal(typeof record.execution_path, 'string'); assert.match(record.execution_path, /^\/proc\/[1-9][0-9]*\/fd\/[0-9]+$/, 'original held-descriptor harness execution');}
    equal(record.argv, argv, 'original complete inventory command argv'); assert.equal(record.status, 0, 'successful original inventory command');
    assert.equal(record.stdout, label + '.stdout'); assert.equal(record.stderr, label + '.stderr'); take(record.stderr); return take(record.stdout);
  }
  const data = metadata(command(value.metadata, 'metadata', ['cargo', 'metadata', '--no-deps', '--locked', '--offline', '--format-version', '1']));
  keys(value.register, ['path', 'sha256']); assert.equal(value.register.path, 'conformance-register.rs'); const registerBytes = take(value.register.path);
  const trustedRegister = expectedRegister === undefined ? readFileSync(new URL('../crates/conformance/tests/conformance.rs', import.meta.url)) : expectedRegister;
  assert(Buffer.isBuffer(trustedRegister) && trustedRegister.length > 0 && trustedRegister.length <= limits.file, 'independent bounded source register required');
  assert(registerBytes.equals(trustedRegister), 'original register differs from independently selected source');
  assert.equal(value.register.sha256, sha(registerBytes), 'original conformance register digest'); const registered = register(registerBytes);
  const owners = part === 'source' ? ['source'] : part === 'conformance' ? ['conformance', 'integration'] : ['integration'];
  assert(Array.isArray(value.cargo)); assert.equal(value.cargo.length, owners.length); const executables = new Map();
  owners.forEach((owner, index) => {
    const expected = new Set([...data.targets].filter(([, target]) => ownerOf(target) === owner).map(([key]) => key));
    const observed = artifacts(command(value.cargo[index], `cargo-${index}`, [...cargoArgs(selection(owner)), '--no-run', '--message-format=json']), data, expected);
    for (const [key, row] of observed) {assert(!executables.has(key)); executables.set(key, row);}
  });
  assert(Array.isArray(value.harnesses)); assert.equal(value.harnesses.length, executables.size, 'every Cargo executable must list');
  const harnesses = [], seen = new Set();
  value.harnesses.forEach((row, index) => {
    keys(row, ['target', 'executable_sha256', 'full', 'selected']); keys(row.target, ['package', 'name', 'kind', 'source']);
    const key = keyOf(row.target); assert(executables.has(key) && !seen.has(key), 'exact unique executable harness'); seen.add(key);
    const executable = executables.get(key); equal(row.target, executable.target, 'harness target differs from Cargo executable'); assert.match(row.executable_sha256, /^[0-9a-f]{64}$/);
    const full = list(command(row.full, `harness-${index}-full`, [executable.executable, '--list', '--format=terse'], false, true)); let selected = null;
    if (ownerOf(row.target) === 'integration') {
      assert.deepEqual(full, registered, 'actual executable list differs from register');
      selected = list(command(row.selected, `harness-${index}-selected`, [executable.executable, '--list', '--format=terse', ...filters(part)], false, true));
      assert.equal(row.selected.execution_path, row.full.execution_path, 'both listings use the same held executable');
      const expected = registered.filter(name => owns(name, part)); assert(expected.length > 0); assert.deepEqual(selected, expected, 'actual executable filter differs from part');
    } else assert.equal(row.selected, null, 'unfiltered source/residual executable');
    harnesses.push({target: row.target, executable_sha256: row.executable_sha256, tests: full, selected});
  });
  assert(Array.isArray(value.docs)); const hasDocs = part === 'source' || part === 'conformance';
  const docKeys = hasDocs ? [...data.docs].filter(([, target]) => ownerOf(target) === part).map(([key]) => key).sort() : [];
  assert.equal(value.docs.length, docKeys.length, 'one original doc listing per library target');
  const docTests = docKeys.flatMap((key, index) => {
    assert.deepEqual(value.docs[index].target_keys, [key], 'one exact metadata documentation target');
    return docList(command(value.docs[index], `doc-${index}`, [...cargoArgs(['--package', data.docs.get(key).package]), '--doc', '--', '--list'], true), data, [key]);
  });
  assert.deepEqual([...used].sort(), [...originals.keys()].sort(), 'exact raw inventory file set');
  return {source_revision: revision, part, packages: data.versions,
    all_targets: [...data.targets.values()].sort((a, b) => keyOf(a).localeCompare(keyOf(b))), all_docs: [...data.docs.values()].sort((a, b) => keyOf(a).localeCompare(keyOf(b))),
    targets: [...executables.keys()].sort(), docs: docKeys, harnesses, doc_tests: docTests, registered, register_sha256: sha(registerBytes)};
}

export function validateCargoInventoryUnion(records, revision, expectedRegister) {
  assert(records instanceof Map); assert.deepEqual([...records.keys()].sort(), [...vvInventoryParts], 'complete fifteen-part inventory map');
  const trustedRegister = expectedRegister === undefined ? readFileSync(new URL('../crates/conformance/tests/conformance.rs', import.meta.url)) : expectedRegister;
  const summaries = new Map(vvInventoryParts.map(part => [part, validateCargoInventory(records.get(part), part, revision, trustedRegister)]));
  const source = summaries.get('source'), identity = row => ({packages: row.packages, targets: row.all_targets, docs: row.all_docs, register: row.register_sha256, registered: row.registered});
  for (const part of cargoInventoryParts) equal(identity(summaries.get(part)), identity(source), 'same original workspace and conformance registration across parts');
  const targets = [], docs = [], integration = []; const integrationTarget = source.all_targets.find(target => ownerOf(target) === 'integration');
  for (const part of cargoInventoryParts) {
    const summary = summaries.get(part); docs.push(...summary.docs);
    for (const key of summary.targets) if (key !== keyOf(integrationTarget)) targets.push(key);
    for (const row of summary.harnesses) if (row.selected !== null) integration.push(...row.selected);
  }
  assert.deepEqual(unique(targets, 'overlapping non-integration target owners'), source.all_targets.filter(target => ownerOf(target) !== 'integration').map(keyOf).sort(), 'all source/residual Cargo targets');
  assert.deepEqual(unique(docs, 'overlapping doc target owners'), source.all_docs.map(keyOf).sort(), 'all Cargo documentation targets');
  assert.deepEqual(unique(integration, 'overlapping selected conformance tests'), source.registered, 'complete original integration test union');
  return {schema: 'prismpm/vv-cargo-inventory-union/1', source_revision: revision, parts: [...vvInventoryParts],
    targets: source.all_targets, docs: source.all_docs, integration, register_sha256: source.register_sha256,
    counts: {targets: source.all_targets.length, docs: source.all_docs.length, integration: integration.length}};
}
