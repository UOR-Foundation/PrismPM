// Recording Cargo/JSON/libtest data only; these units never execute Cargo,
// compile targets, run owning suites, or grant source/SDK acceptance.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {pathToFileURL} from 'node:url';
import test from 'node:test';
import {cargoInventoryParts, vvInventoryParts, validateCargoInventory, validateCargoInventoryUnion} from './vv-cargo-inventory.mjs';

const revision = 'a'.repeat(40);
const canonical = value => JSON.stringify(value, (_, row) => row && typeof row === 'object' && !Array.isArray(row)
  ? Object.fromEntries(Object.keys(row).sort().map(key => [key, row[key]])) : row);
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const packagePaths = {prismpm: 'crates/prismpm', 'repo-model': 'crates/model', 'repo-conformance': 'crates/conformance', xtask: 'xtask'};
const groups = {'ho-oc': ['conformance_ho_', 'conformance_oc_'], 'st-sy': ['conformance_st_', 'conformance_sy_'],
  'dk-0-1': ['conformance_dk_0', 'conformance_dk_1'], 'dk-2': ['conformance_dk_2'], 'dk-3': ['conformance_dk_3'], 'dk-4': ['conformance_dk_40', 'conformance_dk_41']};
const selected = (name, part) => part === 'conformance' ? !Object.keys(groups).some(part => selected(name, part))
  : part === 'dk-4' ? ['conformance_dk_40', 'conformance_dk_41'].includes(name) : groups[part].some(prefix => name.startsWith(prefix));
const filterArgs = part => part === 'conformance' ? Object.values(groups).flatMap(values => values.flatMap(value => ['--skip', value])) : groups[part];
const owner = target => target.package !== 'repo-conformance' ? 'source' : target.kind === 'test' ? 'integration' : 'conformance';
const keyOf = target => `${target.package}/${target.kind}/${target.name}`;
const cargo = selection => ['cargo', 'test', ...selection, '--all-features', '--locked', '--offline'];
const selectOwner = name => name === 'source' ? ['--workspace', '--exclude', 'repo-conformance']
  : name === 'conformance' ? ['--package', 'repo-conformance', '--lib', '--bins'] : ['--package', 'repo-conformance', '--test', 'conformance'];
const sampleNames = ['conformance_rp_01', 'conformance_ho_01', 'conformance_oc_01', 'conformance_st_01', 'conformance_sy_01',
  'conformance_dk_00', 'conformance_dk_01', 'conformance_dk_10', 'conformance_dk_20', 'conformance_dk_30', 'conformance_dk_36', 'conformance_dk_40', 'conformance_dk_41'];
const actualRegister = readFileSync(new URL('../crates/conformance/tests/conformance.rs', import.meta.url));
const actualNames = [...actualRegister.toString().matchAll(/^test_case!\((conformance_[a-z0-9_]+), /gm)].map(match => match[1]);

// This factory is exported ONLY from a test module for transport recording units.
// Importing it does not register this file's tests. No production validator seam
// accepts a fixture builder or substitutes original command/executable readers.
export function recordingCargoInventoryParts(sourceRevision = revision, options = {}) {
  const root = options.root ?? '/recording/workspace', targetDirectory = root + '/target/vv-workspace';
  const registerSource = options.registerSource ?? actualRegister.toString();
  const names = [...registerSource.matchAll(/^test_case!\((conformance_[a-z0-9_]+), /gm)].map(match => match[1]).sort();
  const allTargets = [], packageRows = [];
  for (const [packageName, directory] of Object.entries(packagePaths)) {
    const targets = [];
    const add = (name, kind, source) => {const target = {package: packageName, name, kind, source}; allTargets.push(target);
      targets.push({name, kind: [kind], src_path: root + '/' + source, test: true, doctest: kind === 'lib', crate_types: [kind === 'test' ? 'bin' : kind], edition: '2021'});};
    if (packageName !== 'xtask') add(packageName.replaceAll('-', '_'), 'lib', directory + '/src/lib.rs');
    const number = packageName === 'prismpm' ? 34 : packageName === 'repo-model' ? 1 : 3;
    for (let index = 0; index < number; index++) add('recording_' + index, 'bin', directory + '/src/bin/recording_' + index + '.rs');
    if (packageName === 'repo-conformance') add('conformance', 'test', directory + '/tests/conformance.rs');
    packageRows.push({name: packageName, version: '0.4.0', id: 'path+file://' + root + '/' + directory + '#' + packageName + '@0.4.0',
      source: null, manifest_path: root + '/' + directory + '/Cargo.toml', targets});
  }
  if (options.extraTarget) {
    const target = {package: 'prismpm', name: 'additional', kind: 'test', source: 'crates/prismpm/tests/additional.rs'}; allTargets.push(target);
    packageRows.find(row => row.name === target.package).targets.push({name: target.name, kind: [target.kind], src_path: root + '/' + target.source, test: true, doctest: false});
  }
  const metadata = {version: 1, packages: packageRows, workspace_members: packageRows.map(row => row.id), workspace_root: root, target_directory: targetDirectory};
  const records = new Map();
  for (const part of vvInventoryParts) {
    const files = new Map(); records.set(part, files); if (!cargoInventoryParts.includes(part)) continue;
    const put = (name, value) => files.set(name, Buffer.isBuffer(value) ? value : Buffer.from(value));
    const command = (label, argv, stdout) => {put(label + '.stdout', stdout); put(label + '.stderr', 'recording stderr\n'); return {argv, stdout: label + '.stdout', stderr: label + '.stderr', status: 0};};
    const value = {schema: 'prismpm/vv-cargo-inventory/1', source_revision: sourceRevision, part,
      metadata: command('metadata', ['cargo', 'metadata', '--no-deps', '--locked', '--offline', '--format-version', '1'], JSON.stringify(metadata) + '\n'),
      cargo: [], harnesses: [], docs: [], register: {path: 'conformance-register.rs', sha256: sha(registerSource)}};
    put('conformance-register.rs', registerSource);
    const owners = part === 'source' ? ['source'] : part === 'conformance' ? ['conformance', 'integration'] : ['integration'];
    for (const [index, name] of owners.entries()) {
      const targets = allTargets.filter(target => owner(target) === name).sort((a, b) => keyOf(a).localeCompare(keyOf(b)));
      const executable = target => targetDirectory + '/debug/deps/' + target.name + '-' + sha(keyOf(target)).slice(0, 16);
      const artifacts = targets.map(target => ({reason: 'compiler-artifact', package_id: packageRows.find(row => row.name === target.package).id,
        target: packageRows.find(row => row.name === target.package).targets.find(row => row.name === target.name && row.kind[0] === target.kind),
        profile: {test: true}, executable: executable(target), filenames: [executable(target)]}));
      value.cargo.push(command('cargo-' + index, [...cargo(selectOwner(name)), '--no-run', '--message-format=json'],
        [...artifacts, {reason: 'build-finished', success: true}].map(row => JSON.stringify(row)).join('\n') + '\n'));
      for (const target of targets) {
        const number = value.harnesses.length, fullNames = name === 'integration' ? names : ['tests::recording_case'];
        const full = {...command(`harness-${number}-full`, [executable(target), '--list', '--format=terse'], fullNames.map(name => name + ': test\n').join('')), execution_path: `/proc/4242/fd/${number + 3}`};
        const selectedRecord = name === 'integration' ? {...command(`harness-${number}-selected`, [executable(target), '--list', '--format=terse', ...filterArgs(part)], names.filter(name => selected(name, part)).map(name => name + ': test\n').join('')), execution_path: full.execution_path} : null;
        value.harnesses.push({target, executable_sha256: sha('recording executable ' + keyOf(target)), full, selected: selectedRecord});
      }
    }
    if (part === 'source' || part === 'conformance') {
      const docs = allTargets.filter(target => target.kind === 'lib' && owner(target) === part).sort((a, b) => keyOf(a).localeCompare(keyOf(b)));
      docs.forEach((target, index) => value.docs.push({...command('doc-' + index, [...cargo(['--package', target.package]), '--doc', '--', '--list'], '0 tests, 0 benchmarks\n'), target_keys: [keyOf(target)]}));
    }
    put('inventory.json', canonical(value));
  }
  return records;
}
const readInventory = files => JSON.parse(files.get('inventory.json'));
const mutateInventory = (files, change) => {const value = readInventory(files); change(value); files.set('inventory.json', Buffer.from(canonical(value)));};
const mutateJson = (files, name, change) => {const value = JSON.parse(files.get(name)); change(value); files.set(name, Buffer.from(JSON.stringify(value) + '\n'));};
const mutateLines = (files, name, change) => {const rows = files.get(name).toString().trim().split('\n').map(JSON.parse); change(rows); files.set(name, Buffer.from(rows.map(row => JSON.stringify(row)).join('\n') + '\n'));};

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  test('recorded original Cargo metadata, executable lists and per-library docs form the complete target union', () => {
    const records = recordingCargoInventoryParts();
    const result = validateCargoInventoryUnion(records, revision);
    assert.deepEqual(result.counts, {targets: 45, docs: 3, integration: actualNames.length});
    assert.equal(validateCargoInventory(records.get('source'), 'source', revision).targets.length, 40);
    const residual = validateCargoInventory(records.get('conformance'), 'conformance', revision);
    assert.equal(residual.targets.length, 5); assert.equal(residual.docs.length, 1);
    for (const part of ['gates', ...vvInventoryParts.filter(part => part.startsWith('standalone-'))]) assert.equal(validateCargoInventory(records.get(part), part, revision), null);
    const expanded = validateCargoInventoryUnion(recordingCargoInventoryParts(revision, {extraTarget: true}), revision);
    assert.equal(expanded.counts.targets, 46, 'targets derive from metadata, not a fixed target count');
  });

  test('exact source, command, file-set and byte bounds reject incomplete or rewritten original inventories', () => {
    for (const change of [
      files => files.delete('metadata.stderr'), files => files.set('unknown.stdout', Buffer.alloc(0)),
      files => files.set('../metadata.stdout', Buffer.alloc(0)), files => files.set('alias/path', Buffer.alloc(0)),
      files => files.set('metadata.stdout', Buffer.from([255])),
      files => mutateInventory(files, row => row.source_revision = 'b'.repeat(40)),
      files => mutateInventory(files, row => row.part = 'dk-2'),
      files => mutateInventory(files, row => row.schema = 'other'),
      files => mutateInventory(files, row => row.extra = true),
      files => mutateInventory(files, row => row.metadata.status = 1),
      files => mutateInventory(files, row => row.metadata.argv.splice(3, 1)),
      files => mutateInventory(files, row => row.cargo[0].argv.push('--lib')),
      files => mutateInventory(files, row => row.metadata.stderr = row.metadata.stdout),
      files => files.set('inventory.json', Buffer.from(JSON.stringify(readInventory(files), null, 2))),
      files => files.set('metadata.stdout', Buffer.alloc(16 * 1024 ** 2 + 1)),
    ]) {const files = recordingCargoInventoryParts().get('source'); change(files); assert.throws(() => validateCargoInventory(files, 'source', revision));}
    const tooMany = new Map(Array.from({length: 1025}, (_, index) => ['recording-' + index, Buffer.alloc(0)]));
    assert.throws(() => validateCargoInventory(tooMany, 'source', revision));
    const block = Buffer.alloc(16 * 1024 ** 2), tooLarge = new Map(Array.from({length: 9}, (_, index) => ['recording-' + index, block]));
    assert.throws(() => validateCargoInventory(tooLarge, 'source', revision), /complete inventory bytes/);
    assert.throws(() => validateCargoInventory(new Map(), 'unknown', revision));
    assert.throws(() => validateCargoInventory(new Map(), 'source', 'main'));
    assert.throws(() => validateCargoInventory(new Map([['inventory.json', Buffer.from('{}')]]), 'gates', revision));
  });

  test('raw Cargo metadata rejects foreign packages, aliased sources and unsupported target ownership', () => {
    for (const change of [
      row => row.workspace_members.pop(), row => row.workspace_members.push(row.workspace_members[0]),
      row => row.packages[0].name = 'foreign', row => row.packages[0].manifest_path += '/other',
      row => row.packages[0].source = 'registry+foreign', row => row.workspace_root += '/..',
      row => row.target_directory = '/tmp/unowned', row => row.packages[0].targets[0].src_path = '/outside/src/lib.rs',
      row => row.packages[0].targets[0].src_path = row.workspace_root + '/crates/model/src/lib.rs',
      row => row.packages[0].targets[0].kind = ['example'], row => row.packages[0].targets.push(row.packages[0].targets[0]),
      row => row.packages.find(row => row.name === 'repo-conformance').targets.at(-1).name = 'unowned',
      row => row.packages[0].targets[1].doctest = true,
    ]) {const files = recordingCargoInventoryParts().get('source'); mutateJson(files, 'metadata.stdout', change); assert.throws(() => validateCargoInventory(files, 'source', revision));}
  });

  test('actual no-run JSON must finish successfully and bind every unique metadata test executable', () => {
    for (const change of [
      rows => rows.pop(), rows => rows.at(-1).success = false, rows => rows.push(rows.at(-1)),
      rows => rows.splice(0, 1), rows => rows.splice(0, 0, rows[0]),
      rows => rows[0].profile.test = false, rows => rows[0].package_id = 'foreign',
      rows => rows[0].executable = null, rows => rows[0].executable = '/outside/executable',
      rows => rows[0].filenames = [], rows => rows[0].target.src_path += '.changed',
      rows => rows[0].reason = 'unknown-record', rows => rows.push({reason: 'compiler-message'}),
    ]) {const files = recordingCargoInventoryParts().get('source'); mutateLines(files, 'cargo-0.stdout', change); assert.throws(() => validateCargoInventory(files, 'source', revision));}
  });

  test('executable full/selected lists require real grammar and the exact registered disjoint conformance partition', () => {
    for (const change of [
      files => files.set('harness-0-full.stdout', Buffer.from('not a harness list\n')),
      files => files.set('harness-0-full.stdout', Buffer.from('conformance_dk_40: test\nconformance_dk_40: test\n')),
      files => files.set('harness-0-selected.stdout', Buffer.alloc(0)),
      files => files.set('harness-0-selected.stdout', Buffer.from('conformance_dk_41: test\n')),
      files => files.set('harness-0-selected.stdout', Buffer.from('conformance_dk_40: test\nconformance_dk_41: test\nconformance_dk_36: test\n')),
      files => mutateInventory(files, row => row.harnesses[0].full.argv[0] += '-different'),
      files => mutateInventory(files, row => {delete row.harnesses[0].full.execution_path;}),
      files => mutateInventory(files, row => row.harnesses[0].full.execution_path = '/tmp/not-held'),
      files => mutateInventory(files, row => row.harnesses[0].selected.execution_path = '/proc/4242/fd/999'),
      files => mutateInventory(files, row => row.harnesses[0].selected.argv.pop()),
      files => mutateInventory(files, row => row.harnesses[0].selected.status = 1),
      files => mutateInventory(files, row => row.harnesses[0].executable_sha256 = 'invalid'),
      files => mutateInventory(files, row => row.harnesses.push(row.harnesses[0])),
    ]) {const files = recordingCargoInventoryParts().get('dk-4'); change(files); assert.throws(() => validateCargoInventory(files, 'dk-4', revision), undefined, change.toString());}
    const register = sampleNames.map(name => `test_case!(${name}, "${name.slice(12).toUpperCase().replaceAll('_', '-')}");`).join('\n');
    for (const changed of [register + '\ntest_case!(conformance_dk_42, "DK-42");', register.replace(/.*conformance_dk_40.*\n?/, '').replace(/.*conformance_dk_41.*\n?/, ''),
      register + '\n' + register.split('\n')[0], register.replace('"RP-01"', '"RP-02"'), '']) {
      const files = recordingCargoInventoryParts(revision, {registerSource: changed}).get('dk-4'); assert.throws(() => validateCargoInventory(files, 'dk-4', revision, Buffer.from(changed)));
    }
    const files = recordingCargoInventoryParts().get('source'); mutateInventory(files, row => row.harnesses[0].selected = row.harnesses[0].full);
    assert.throws(() => validateCargoInventory(files, 'source', revision));
  });

  test('documentation ownership requires per-library original argv plus complete actual rustdoc listing bytes', () => {
    for (const change of [
      files => files.set('doc-0.stdout', Buffer.alloc(0)),
      files => files.set('doc-0.stdout', Buffer.from('0 tests, 0 benchmarks\n0 tests, 0 benchmarks\n')),
      files => files.set('doc-0.stdout', Buffer.from('crates/prismpm/src/lib.rs - probe (line 1): test\n0 tests, 0 benchmarks\n')),
      files => files.set('doc-0.stdout', Buffer.from('crates/conformance/src/lib.rs - probe (line 1): test\n1 test, 0 benchmarks\n')),
      files => mutateInventory(files, row => row.docs[0].target_keys.push(row.docs[1].target_keys[0])),
      files => mutateInventory(files, row => row.docs[0].argv = ['cargo', 'test', '--workspace', '--doc', '--', '--list']),
      files => mutateInventory(files, row => row.docs[0].status = 1),
      files => mutateInventory(files, row => row.docs.pop()),
    ]) {const files = recordingCargoInventoryParts().get('source'); change(files); assert.throws(() => validateCargoInventory(files, 'source', revision));}
    const files = recordingCargoInventoryParts().get('source'); files.set('doc-0.stdout', Buffer.from('crates/prismpm/src/lib.rs - probe (line 1): test\n\n1 test, 0 benchmarks\n'));
    assert.equal(validateCargoInventory(files, 'source', revision).doc_tests.length, 1);
  });

  test('union refuses missing, duplicate, foreign or overlapping parts and coherently changed source registrations', () => {
    for (const change of [records => records.delete('gates'), records => records.set('unknown', new Map()),
      records => records.set('dk-4', records.get('dk-3')),
      records => records.set('source', recordingCargoInventoryParts('b'.repeat(40)).get('source')),
      records => records.set('source', recordingCargoInventoryParts(revision, {extraTarget: true}).get('source')),
      records => records.set('conformance', recordingCargoInventoryParts(revision, {registerSource: sampleNames.map(name => `test_case!(${name}, "${name.slice(12).toUpperCase().replaceAll('_', '-')}");`).join('\n') + '\ntest_case!(conformance_rp_02, "RP-02");\n'}).get('conformance')),
    ]) {const records = recordingCargoInventoryParts(); change(records); assert.throws(() => validateCargoInventoryUnion(records, revision));}
  });

  test('new registered cases remain bound to independently selected source even when every retained list is coherently resealed', () => {
    let finalSource = actualRegister.toString();
    for (const [name, id] of [['conformance_dk_36', 'DK-36'], ['conformance_dk_41', 'DK-41']])
      if (!finalSource.includes('test_case!(' + name + ',')) finalSource += `\ntest_case!(${name}, "${id}");\n`;
    const trusted = Buffer.from(finalSource), records = recordingCargoInventoryParts(revision, {registerSource: finalSource});
    const result = validateCargoInventoryUnion(records, revision, trusted);
    assert(result.integration.includes('conformance_dk_36')); assert(result.integration.includes('conformance_dk_41'));
    assert.equal(result.counts.integration, [...finalSource.matchAll(/^test_case!\(/gm)].length);
    for (const name of ['conformance_dk_36', 'conformance_dk_41']) {
      const omitted = finalSource.split('\n').filter(line => !line.startsWith('test_case!(' + name + ',')).join('\n');
      const resealed = recordingCargoInventoryParts(revision, {registerSource: omitted});
      assert.throws(() => validateCargoInventoryUnion(resealed, revision, trusted), /independently selected source/);
    }
    const extra = finalSource + '\ntest_case!(conformance_dk_42, "DK-42");\n';
    assert.throws(() => validateCargoInventoryUnion(recordingCargoInventoryParts(revision, {registerSource: extra}), revision, Buffer.from(extra)), /unowned DK/);
  });
}
