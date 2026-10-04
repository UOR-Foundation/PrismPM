// Workspace test owner: one fresh pinned compiler for the complete matrix.
// A supplied executable or a receipt from an earlier run is never authority.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {chmodSync, copyFileSync, lstatSync, mkdirSync, mkdtempSync, readFileSync,
  realpathSync, rmSync, writeFileSync} from 'node:fs';
import {join, resolve} from 'node:path';
import {tmpdir} from 'node:os';
import {fileURLToPath} from 'node:url';
import {snapshotTree} from '../sdk/exporter-seed.mjs';
import {capture, refuseCargoAncestorConfiguration, snapshotSourceTree, privateGitObjects, privateRegistryDownloads} from './portable-oracle-custody.mjs';

const root = realpathSync(fileURLToPath(new URL('../', import.meta.url)));
assert.equal(process.argv.length, 6, 'two artifacts and their live Controller bindings required');
const artifacts = process.argv.slice(2, 4).map(path => realpathSync(resolve(path)));
const bindings = process.argv.slice(4).map(value => JSON.parse(value));
assert.equal(artifacts.length, 2, 'both complete verified application artifacts required');
assert.equal(process.version, 'v22.23.2');
const matrix = JSON.parse(readFileSync(join(root, 'tests/data/portable-oracle-matrix.json')));
assert.equal(matrix.schema, 'prismpm/portable-oracle-matrix/1');
assert.deepEqual(matrix.profiles.map(profile => profile.name), ['Calculator', 'Text Request']);
assert.deepEqual(matrix.triggers, ['click', 'keyboard']);
assert.equal(matrix.interaction_cases.length, 17);
assert.equal(matrix.infrastructure_cases.length, 3);
assert.equal(new Set([...matrix.interaction_cases, ...matrix.infrastructure_cases]).size, 20);
const browser = process.arch === 'x64'
  ? '/ms-playwright/chromium_headless_shell-1234/chrome-headless-shell-linux64/chrome-headless-shell'
  : '/ms-playwright/chromium_headless_shell-1234/chrome-linux/headless_shell';
assert(['x64', 'arm64'].includes(process.arch));
const inputs = [
  'vendor/hologram-live.tar', 'tests/hologram-oracle/Cargo.toml',
  'tests/hologram-oracle/Cargo.lock', 'tests/hologram-oracle/src/main.rs',
  'crates/prismpm/src/embedded/hologram-oracle.Cargo.toml',
  'crates/prismpm/src/embedded/hologram-oracle.Cargo.lock',
  'crates/prismpm/src/embedded/hologram-oracle.main.rs',
  'crates/prismpm/src/embedded/hologram-oracle.browser.mjs',
  'scripts/portable-oracle-matrix.mjs', 'scripts/portable-oracle-submission-probe.mjs',
  'scripts/portable-oracle-process-owner.py',
  'scripts/portable-oracle-custody.mjs',
  'tests/data/portable-oracle-matrix.json', 'sdk/exporter-seed.mjs',
  'sdk/inventory-metadata.mjs',
];
const captured = inputs.map(path => capture(join(root, path)));
captured.push(capture(process.execPath), capture(browser), capture('/usr/bin/python3'));
assert.equal(captured[0].measurement.sha256, 'caf5c34ef2b21d58c1aa12acf81cb13ace1adaffb3c69a641f54f490ed61cf66');
for (const [actual, embedded] of [['Cargo.toml', 'Cargo.toml'], ['Cargo.lock', 'Cargo.lock'], ['src/main.rs', 'main.rs']])
  assert(readFileSync(join(root, 'tests/hologram-oracle', actual))
    .equals(readFileSync(join(root, 'crates/prismpm/src/embedded/hologram-oracle.' + embedded))));
for (const [index, path] of artifacts.entries()) {
  assert.equal(JSON.parse(readFileSync(join(path, 'model.prism.json'))).application.name, matrix.profiles[index].name);
  const profile = matrix.profiles[index], binding = bindings[index];
  for (const [key, selected] of Object.entries({
    verification: binding.verification_manifest, model: join(path, 'model.prism.json'),
    archive: join(path, profile.name + '.holo'),
    wasm: join(path, 'core-wasm', profile.cargo_name.replaceAll('-', '_') + '_core_wasm.wasm'),
  })) {
    const subject = capture(selected);
    assert.equal(subject.measurement.sha256, binding[key + '_sha256'], 'live verified subject differs');
    captured.push(subject);
  }
}

mkdirSync(join(root, 'target'), {recursive: true});
const evidence = mkdtempSync(join(root, 'target/portable-oracle-matrix-'));
chmodSync(evidence, 0o755);
const work = mkdtempSync(join(tmpdir(), 'prismpm-portable-oracle-owner-'));
const owned = lstatSync(work);
const toolchain = `/usr/local/rustup/toolchains/1.97.1-${process.arch === 'x64' ? 'x86_64' : 'aarch64'}-unknown-linux-gnu`;
const cargo = capture(join(toolchain, 'bin/cargo'));
const rustc = capture(join(toolchain, 'bin/rustc'));
captured.push(cargo, rustc, capture('/usr/bin/cc'), capture('/usr/bin/ld'), capture('/usr/bin/git'), capture('/usr/bin/tar'));
const environment = {PATH: `${toolchain}/bin:/usr/bin:/bin`, HOME: work,
  CARGO_HOME: join(work, 'cargo-home'), RUSTC: rustc.path,
  GIT_CONFIG_NOSYSTEM: '1', GIT_CONFIG_GLOBAL: '/dev/null',
  CARGO_NET_OFFLINE: 'true', CARGO_BUILD_JOBS: '2',
  CARGO_INCREMENTAL: '0', LANG: 'C', LC_ALL: 'C', TMPDIR: work};
function run(name, program, args, directory, seconds, selectedEnvironment = environment, expectedStatus = 0) {
  const cleanupReceipt = join(evidence, `${name}.cleanup.json`);
  const result = spawnSync('/usr/bin/python3', ['-I', '-B', join(root, 'scripts/portable-oracle-process-owner.py'),
    String(seconds), cleanupReceipt, program, ...args],
    {cwd: directory, env: selectedEnvironment, encoding: 'utf8', timeout: (seconds + 10) * 1000, maxBuffer: 16 * 1024 ** 2});
  for (const stream of ['stdout', 'stderr']) writeFileSync(join(evidence, `${name}.${stream}`), result[stream] ?? '', {flag: 'wx'});
  writeFileSync(join(evidence, `${name}.process.json`), JSON.stringify({program, args, cwd: directory,
    exit_code: result.status, signal: result.signal, execution_error: result.error ? String(result.error) : null}) + '\n', {flag: 'wx'});
  // Cleanup uncertainty aborts the matrix, including when the case also failed.
  const cleanup = JSON.parse(readFileSync(cleanupReceipt));
  assert.equal(cleanup.schema, 'prismpm/portable-process-owner/1');
  assert.equal(cleanup.cleanup_verified, true, 'oracle descendants not reaped');
  assert.ifError(result.error);
  assert.equal(result.signal, null, 'oracle owner child terminated by signal');
  assert.equal(result.status, expectedStatus, `oracle owner ${name} failed; retained ${evidence}`);
  return result;
}
const outcomes = [];
const negativeControls = [];
try {
  mkdirSync(environment.CARGO_HOME);
  // Dependency downloads may be shared; caller Cargo configuration, wrappers,
  // targets and credentials are never compiler-construction authority.
  privateRegistryDownloads(process.env.CARGO_HOME, environment.CARGO_HOME);
  const gitDatabase = privateGitObjects(process.env.CARGO_HOME, environment.CARGO_HOME);
  run('git-object-integrity', '/usr/bin/git', ['--git-dir=' + gitDatabase, 'fsck', '--strict', '--no-reflogs'], work, 30);
  for (const [name, revision] of [
    ['arc42-generator-cff72964a489c50e', '2b28a31bac7940fe6a2911fcf775b2b92b7bba92'],
    ['arc42-template-28376a57ea82a6b9', '8dff0d9b1f9640684df8c3bbcdc2ee45f989ca0f'],
    ['req42-framework-f57272977ddce5e8', '246d9cb29ea7991826b242f8d9bb8a039e3be20e'],
  ]) {
    const database = privateGitObjects(process.env.CARGO_HOME, environment.CARGO_HOME, name, revision);
    run('git-object-integrity-' + name, '/usr/bin/git', ['--git-dir=' + database, 'fsck', '--strict', '--no-reflogs'], work, 30);
  }
  mkdirSync(join(work, 'hologram-live'));
  mkdirSync(join(work, 'harness/src'), {recursive: true});
  run('source-extraction', '/usr/bin/tar', ['-xf', captured[0].path, '-C', join(work, 'hologram-live')], work, 30);
  for (const path of ['Cargo.toml', 'Cargo.lock', 'src/main.rs']) {
    const target = join(work, 'harness', path);
    copyFileSync(join(root, 'tests/hologram-oracle', path), target);
  }
  for (const input of captured) input.verify();
  refuseCargoAncestorConfiguration(work);
  run('dependency-resolution', cargo.path, ['metadata', '--locked', '--offline', '--format-version', '1',
    '--manifest-path', join(work, 'harness/Cargo.toml')], work, 120);
  const staged = ['hologram-live', 'harness'].map(path => ({path: join(work, path), files: snapshotSourceTree(join(work, path))}));
  const checkout = join(environment.CARGO_HOME, 'git/checkouts');
  staged.push({path: checkout, excludeGitDatabase: true,
    files: snapshotSourceTree(checkout, {excludeGitDatabase: true})});
  const registrySources = join(environment.CARGO_HOME, 'registry/src');
  staged.push({path: registrySources, files: snapshotSourceTree(registrySources)});
  for (const path of ['Cargo.toml', 'Cargo.lock', 'src/main.rs'])
    assert(readFileSync(join(work, 'harness', path)).equals(readFileSync(join(root, 'tests/hologram-oracle', path))),
      'staged harness differs from pinned source');
  run('source-comparison', '/usr/bin/tar', ['--compare', '-f', captured[0].path, '-C', join(work, 'hologram-live')], work, 30);
  const compilerFiles = snapshotTree(toolchain, {toolchainAliases: true});
  for (const tree of staged) assert.deepEqual(snapshotSourceTree(tree.path, tree), tree.files, 'staged source changed');
  refuseCargoAncestorConfiguration(work);
  run('fresh-compiler', cargo.path, ['build', '--locked', '--offline', '--manifest-path', join(work, 'harness/Cargo.toml'),
    '--target-dir', join(work, 'target')], work, 900, {...environment, CARGO_PROFILE_DEV_DEBUG: '0'});
  refuseCargoAncestorConfiguration(work);
  for (const tree of staged) assert.deepEqual(snapshotSourceTree(tree.path, tree), tree.files, 'staged source changed');
  assert.deepEqual(snapshotTree(toolchain, {toolchainAliases: true}), compilerFiles, 'compiler toolchain changed');
  for (const input of captured) input.verify();
  // Cargo hard-links the top-level binary to its deps output. Execute an owned
  // independent copy so that no second writable alias survives its capture.
  copyFileSync(join(work, 'target/debug/prismpm-hologram-oracle'), join(work, 'oracle'));
  const executable = capture(join(work, 'oracle'));
  writeFileSync(join(evidence, 'construction.json'), JSON.stringify({schema: 'prismpm/portable-oracle-owner/1',
    scope: 'fresh-source-owned-oracle-matrix', inputs: captured.map(({verify, ...row}) => row),
    staged, compiler_files: compilerFiles, environment,
    executable: {path: executable.path, measurement: executable.measurement, identity: executable.identity}}) + '\n', {flag: 'wx'});
  for (const [index, profile] of matrix.profiles.entries()) {
    // Plant defects in the probe itself: an unrelated status failure and a
    // missing mutation must not be reported as envelope-boundary evidence.
    for (const control of ['wrong-status', 'noop']) {
      const id = `${index}-probe-control-${control}`;
      for (const input of captured) input.verify();
      executable.verify();
      const result = run(id, process.execPath, [join(root, 'scripts/portable-oracle-submission-probe.mjs'),
        executable.path, artifacts[index], browser, 'wrong-response', join(evidence, id), 'click', control],
      root, 130, environment, 1);
      const receipt = JSON.parse(result.stdout.trim());
      assert.equal(receipt.control, control);
      assert.equal(receipt.probe_passed, false, 'defective probe must be refused');
      assert.equal(receipt.oracle_sha256, executable.measurement.sha256);
      for (const key of ['model_sha256', 'archive_sha256', 'wasm_sha256'])
        assert.equal(receipt[key], bindings[index][key]);
      if (control === 'noop') {
        assert.equal(receipt.exit_code, 0, 'no-op must exercise a successful real application');
        assert.deepEqual(receipt.diagnostics, []);
      } else {
        assert.equal(receipt.failure_code, 'PORTABLE_WRONG_CHECK');
        assert.equal(receipt.diagnostics.length, 1);
        assert.equal(receipt.diagnostics[0].check, 'response-status');
        assert.equal(receipt.diagnostics[0].failure, 'assertion');
      }
      executable.verify();
      for (const input of captured) input.verify();
      negativeControls.push({id, status: 'refused-as-required'});
    }
    const rows = [...matrix.triggers.flatMap(trigger => matrix.interaction_cases.map(name => ({trigger, name}))),
      ...matrix.infrastructure_cases.map(name => ({trigger: 'click', name}))];
    for (const {trigger, name} of rows) {
      const id = `${index}-${trigger}-${name}`;
      for (const input of captured) input.verify();
      executable.verify();
      try {
        const result = run(id, process.execPath, [join(root, 'scripts/portable-oracle-submission-probe.mjs'),
          executable.path, artifacts[index], browser, name, join(evidence, id), trigger], root, 130);
        const receipt = JSON.parse(result.stdout.trim());
        assert.equal(receipt.probe_passed, true);
        assert.equal(receipt.profile, profile.profile);
        assert.equal(receipt.case, name);
        assert.equal(receipt.trigger, trigger);
        assert.equal(receipt.oracle_sha256, executable.measurement.sha256);
        for (const key of ['model_sha256', 'archive_sha256', 'wasm_sha256'])
          assert.equal(receipt[key], bindings[index][key], 'case used a different verified subject');
        outcomes.push({id, status: 'passed'});
      } catch (error) {
        outcomes.push({id, status: 'failed', error: String(error)});
        assert.equal(JSON.parse(readFileSync(join(evidence, `${id}.cleanup.json`))).cleanup_verified,
          true, 'cannot continue after uncertain process cleanup');
      }
      executable.verify();
      for (const input of captured) input.verify();
    }
  }
  assert.equal(outcomes.length, 74, 'the complete two-profile matrix must execute');
  assert.equal(negativeControls.length, 4);
  assert(outcomes.every(row => row.status === 'passed'), `portable View matrix failed; retained ${evidence}`);
  console.log(JSON.stringify({schema: 'prismpm/portable-oracle-matrix-result/1', cases: outcomes.length,
    profiles: matrix.profiles.map(profile => profile.profile), negative_controls: negativeControls, status: 'passed', evidence}));
} finally {
  writeFileSync(join(evidence, 'outcomes.json'), JSON.stringify(outcomes) + '\n', {flag: 'wx'});
  writeFileSync(join(evidence, 'negative-controls.json'), JSON.stringify(negativeControls) + '\n', {flag: 'wx'});
  const current = lstatSync(work);
  assert(current.isDirectory() && current.dev === owned.dev && current.ino === owned.ino
    && current.uid === owned.uid && (current.mode & 0o7777) === 0o700, 'private compiler owner replaced');
  rmSync(work, {recursive: true});
}
