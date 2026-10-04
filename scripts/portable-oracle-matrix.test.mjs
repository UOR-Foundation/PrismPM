import assert from 'node:assert/strict';
import {readFileSync, mkdtempSync, mkdirSync, rmSync, writeFileSync, renameSync, existsSync, realpathSync, chmodSync, linkSync, symlinkSync, copyFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {spawnSync} from 'node:child_process';
import test from 'node:test';
import {capture, requireBoundaryCheck, refuseCargoAncestorConfiguration, snapshotSourceTree, privateGitObjects, privateRegistryDownloads, applyNegativeControl} from './portable-oracle-custody.mjs';

const read = path => readFileSync(new URL('../' + path, import.meta.url), 'utf8');
const matrix = JSON.parse(read('tests/data/portable-oracle-matrix.json'));

test('pinned archive comparison retains complete content and mode custody without requiring root ownership', () => {
  const root = mkdtempSync(join(tmpdir(), 'portable-archive-test-'));
  const archive = new URL('../vendor/hologram-live.tar', import.meta.url).pathname;
  const verifier = new URL('./portable-oracle-source.py', import.meta.url).pathname;
  const check = (source, directory) => {
    const result = spawnSync('/usr/bin/python3', ['-I', '-B', verifier, source, directory], {encoding: 'utf8', timeout: 30000, maxBuffer: 65536});
    assert.ifError(result.error);
    assert.equal(result.signal, null);
    return result;
  };
  const reject = (source, directory, message) => {
    const result = check(source, directory);
    assert.equal(result.status, 1, result.stderr);
    assert(result.stderr.includes(message), result.stderr);
  };
  try {
    for (const mask of ['0022', '0077']) {
      const directory = join(root, mask);
      mkdirSync(directory);
      const extraction = spawnSync('/bin/sh', ['-c', 'umask "$1"; exec /usr/bin/tar --no-same-owner --same-permissions -xf "$2" -C "$3"', 'extract', mask, archive, directory], {timeout: 30000, maxBuffer: 65536});
      assert.ifError(extraction.error);
      assert.equal(extraction.signal, null);
      assert.equal(extraction.status, 0);
      assert.equal(check(archive, directory).status, 0);
      const selected = join(directory, 'Cargo.toml'), original = readFileSync(selected);
      for (const mutation of ['bytes', 'missing', 'extra', 'mode', 'symlink', 'hardlink']) {
        if (mutation === 'bytes') writeFileSync(selected, Buffer.concat([original, Buffer.from('\n')]));
        if (mutation === 'missing') renameSync(selected, join(root, 'removed'));
        if (mutation === 'extra') writeFileSync(join(directory, 'unexpected'), 'extra');
        if (mutation === 'mode') chmodSync(selected, 0o764);
        if (mutation === 'symlink') { renameSync(selected, join(root, 'removed')); symlinkSync(join(root, 'removed'), selected); }
        if (mutation === 'hardlink') linkSync(selected, join(root, 'alias'));
        reject(archive, directory, {bytes: 'source type, mode, size or bytes differ', missing: 'source inventory incomplete',
          extra: 'unexpected source entry', mode: 'source type, mode, size or bytes differ', symlink: 'source alias or special file',
          hardlink: 'source file alias or special file'}[mutation]);
        if (mutation === 'bytes') writeFileSync(selected, original);
        if (mutation === 'missing') renameSync(join(root, 'removed'), selected);
        if (mutation === 'extra') rmSync(join(directory, 'unexpected'));
        if (mutation === 'mode') chmodSync(selected, 0o664);
        if (mutation === 'symlink') { rmSync(selected); renameSync(join(root, 'removed'), selected); }
        if (mutation === 'hardlink') rmSync(join(root, 'alias'));
        assert.equal(check(archive, directory).status, 0, 'restored ' + mutation);
      }
      const changedArchive = join(root, 'changed.tar');
      copyFileSync(archive, changedArchive);
      writeFileSync(changedArchive, Buffer.concat([readFileSync(changedArchive), Buffer.from('x')]));
      reject(changedArchive, directory, 'source archive identity differs');
      const archiveAlias = join(root, 'archive-alias');
      symlinkSync(archive, archiveAlias);
      reject(archiveAlias, directory, 'source file alias or special file');
      rmSync(archiveAlias);
      linkSync(changedArchive, archiveAlias);
      reject(changedArchive, directory, 'source file alias or special file');
      rmSync(archiveAlias);
      const rootAlias = join(root, 'root-alias');
      symlinkSync(directory, rootAlias);
      reject(archive, rootAlias, 'source directory alias');
      rmSync(rootAlias);
      chmodSync(directory, 0o755);
      reject(archive, directory, 'source root mode differs');
      chmodSync(directory, 0o700);
      assert.equal(check(archive, directory).status, 0);
    }
  } finally { rmSync(root, {recursive: true}); }
});

test('wrong-status control changes only its injected route, not other successful journeys', () => {
  const source = read('crates/prismpm/src/embedded/hologram-oracle.browser.mjs');
  assert(source.includes('route.fulfill({status: 200,'));
  const injection = 'uniqueProbe(route.fulfill({status: 200, body: "wrong-envelope"}));';
  const driver = source + '\n' + injection;
  assert.equal(applyNegativeControl(driver, source, injection, 'wrong-status'),
    source + '\n' + injection.replace('status: 200,', 'status: 503,'));
  assert.equal(applyNegativeControl(driver, source, injection, 'noop'), source);
  assert.equal(applyNegativeControl(driver, source, injection, 'none'), driver);
  assert.throws(() => applyNegativeControl(driver + injection, source, injection, 'wrong-status'), /must be unique/);
});

test('registry download sharing cannot import poisoned previously extracted sources', () => {
  const root = mkdtempSync(join(tmpdir(), 'portable-registry-test-'));
  try {
    const source = join(root, 'source'), destination = join(root, 'private');
    for (const name of ['cache', 'index', 'src']) mkdirSync(join(source, 'registry', name), {recursive: true});
    writeFileSync(join(source, 'registry/src/lib.rs'), 'compile_error!("poisoned source");');
    writeFileSync(join(source, 'registry/src/.cargo-ok'), '{"v":1}');
    privateRegistryDownloads(source, destination);
    assert.equal(existsSync(join(destination, 'registry/src')), false);
    for (const name of ['cache', 'index']) assert.equal(realpathSync(join(destination, 'registry', name)), join(source, 'registry', name));
  } finally { rmSync(root, {recursive: true}); }
});

test('fresh Git source cannot import a poisoned caller checkout or Git configuration', () => {
  const root = mkdtempSync(join(tmpdir(), 'portable-git-test-'));
  try {
    const source = join(root, 'source'), destination = join(root, 'private');
    const database = join(source, 'git/db/hologram-ab6b9bff1a591920');
    mkdirSync(join(database, 'objects/pack'), {recursive: true});
    mkdirSync(join(source, 'git/checkouts/hologram/poisoned'), {recursive: true});
    writeFileSync(join(source, 'git/checkouts/hologram/poisoned/lib.rs'), 'compile_error!("poisoned checkout");');
    writeFileSync(join(database, 'config'), '[core]\nhooksPath=/poisoned/hooks\n');
    const fresh = privateGitObjects(source, destination);
    assert.equal(readFileSync(join(fresh, 'config'), 'utf8').includes('poisoned'), false);
    assert.equal(snapshotSourceTree(destination).some(row => row.path.includes('checkouts')), false);
    writeFileSync(join(database, 'objects/pack/poisoned'), 'different objects');
    assert.equal(snapshotSourceTree(join(fresh, 'objects')).some(row => row.path.includes('poisoned')), false);
  } finally { rmSync(root, {recursive: true}); }
});

test('staged upstream tree permits pinned icon names but rejects source substitution', () => {
  const root = mkdtempSync(join(tmpdir(), 'portable-source-test-'));
  try {
    writeFileSync(join(root, 'icon@2x.png'), 'pinned icon');
    writeFileSync(join(root, 'main.rs'), 'fn main() {}');
    const pinned = snapshotSourceTree(root);
    assert.deepEqual(snapshotSourceTree(root), pinned);
    writeFileSync(join(root, 'main.rs'), 'fn main() { panic!(); }');
    assert.notDeepEqual(snapshotSourceTree(root), pinned);
  } finally { rmSync(root, {recursive: true}); }
});

test('private Cargo home cannot hide a planted ancestor wrapper configuration', () => {
  const parent = mkdtempSync(join(tmpdir(), 'portable-cargo-test-'));
  try {
    const work = join(parent, 'work');
    mkdirSync(work);
    refuseCargoAncestorConfiguration(work);
    mkdirSync(join(parent, '.cargo'));
    for (const name of ['config', 'config.toml']) {
      const path = join(parent, '.cargo', name);
      writeFileSync(path, '[build]\nrustc-wrapper = "/planted/wrapper"\n');
      assert.throws(() => refuseCargoAncestorConfiguration(work), /ancestor configuration/);
      rmSync(path);
    }
  } finally { rmSync(parent, {recursive: true}); }
});

test('verified model, archive and Wasm custody reject between-case substitution', () => {
  const directory = mkdtempSync(join(tmpdir(), 'portable-custody-test-'));
  try {
    for (const name of ['model.prism.json', 'Calculator.holo', 'core.wasm']) {
      const path = join(directory, name);
      writeFileSync(path, 'verified bytes');
      const original = capture(path);
      original.verify();
      writeFileSync(path + '.replacement', 'verified bytes');
      renameSync(path + '.replacement', path);
      assert.throws(original.verify, /custody changed/);
      const replacement = capture(path);
      writeFileSync(path, 'different bytes');
      assert.throws(replacement.verify, /custody changed/);
    }
  } finally { rmSync(directory, {recursive: true}); }
});

test('wrong-status and unrelated assertion cannot stand in for envelope or count checks', () => {
  for (const name of ['wrong-response', 'delayed-wrong-response', 'duplicate', 'delayed-duplicate']) {
    const intended = name.includes('response') ? 'response-envelope' : 'single-invocation';
    requireBoundaryCheck(name, {check: intended});
    for (const check of [null, 'response-status', 'request-envelope', 'response-json'])
      assert.throws(() => requireBoundaryCheck(name, {check}), {code: 'PORTABLE_WRONG_CHECK'});
  }
});

test('process owner reaps detached stubborn descendants on exit, timeout, signal and output limit', () => {
  const directory = mkdtempSync(join(tmpdir(), 'portable-owner-test-'));
  try {
    for (const mode of ['exit', 'timeout', 'signal', 'output']) {
      const timeout = mode === 'timeout';
      const receipt = join(directory, `${mode}.json`);
      const pidFile = join(directory, `${mode}.pid`);
      const child = 'import os,signal,time; signal.signal(signal.SIGTERM,signal.SIG_IGN); '
        + `open(${JSON.stringify(pidFile)},"x").write(str(os.getpid())); time.sleep(60)`;
      const parent = 'import subprocess,time,os,signal; '
        + `subprocess.Popen(["/usr/bin/python3","-c",${JSON.stringify(child)}],start_new_session=True); `
        + 'time.sleep(0.2); '
        + (mode === 'signal' ? 'os.kill(os.getppid(),signal.SIGTERM); time.sleep(60)'
          : mode === 'output' ? 'os.write(1,b"x"*2000000); time.sleep(60)'
            : `time.sleep(${timeout ? 60 : 0})`);
      const result = spawnSync('/usr/bin/python3', ['-I', '-B',
        new URL('./portable-oracle-process-owner.py', import.meta.url).pathname,
        '1', receipt, '/usr/bin/python3', '-c', parent], {timeout: 8000, maxBuffer: 65536, encoding: 'utf8'});
      if (mode === 'output') assert.equal(result.error?.code, 'ENOBUFS');
      else {
        assert.ifError(result.error);
        assert.equal(result.status, timeout ? 124 : mode === 'signal' ? 125 : 0, result.stderr);
      }
      const actual = JSON.parse(readFileSync(receipt));
      assert.deepEqual(actual, {
        schema: 'prismpm/portable-process-owner/1', exit_code: timeout ? null : 0,
        timed_out: timeout, interrupted: false, cleanup_verified: true,
        ...(['signal', 'output'].includes(mode) ? {exit_code: actual.exit_code, interrupted: true} : {}),
      });
      const pid = Number(readFileSync(pidFile, 'utf8'));
      assert.throws(() => process.kill(pid, 0), {code: 'ESRCH'});
    }
  } finally { rmSync(directory, {recursive: true}); }
});

test('portable oracle owns the complete two-profile and two-trigger boundary inventory', () => {
  assert.equal(matrix.schema, 'prismpm/portable-oracle-matrix/1');
  assert.deepEqual(matrix.profiles.map(row => [row.name, row.profile, row.project, row.cargo_name]), [
    ['Calculator', 'legacy-numeric', 'examples/Calculator', 'prism-calculator'],
    ['Text Request', 'utf8-text', 'tests/fixtures/holo/ho-11-text-application/project', 'prism-text-request'],
  ]);
  assert.deepEqual(matrix.triggers, ['click', 'keyboard']);
  assert.deepEqual(matrix.interaction_cases, ['positive', 'delayed-completion', 'stuck-busy', 'missing-control',
    'body-unavailable', 'method-rewrite', 'wrong-response', 'fill-failure', 'private-method', 'body-plus-cleanup',
    'wrong-method', 'wrong-payload', 'duplicate', 'navigation', 'trigger-failure',
    'delayed-duplicate', 'delayed-wrong-response', 'delayed-stuck-busy']);
  assert.deepEqual(matrix.infrastructure_cases, ['pretend-body-failure', 'cleanup-failure', 'setup-failure']);
  assert.equal(matrix.profiles.length * (matrix.triggers.length * matrix.interaction_cases.length + matrix.infrastructure_cases.length), 78);
  assert.deepEqual(matrix.profiles[0].vector_indexes, Array.from({length: 15}, (_, index) => index));
  assert.deepEqual(matrix.profiles[1].vector_indexes, [0, 2, 3]);
  const common = ['attachment-assets', 'modeled-vectors', 'input-validation-recovery',
    'transport-failure-recovery', 'pre-init-privacy', 'delayed-init', 'intent-boundaries'];
  assert.deepEqual(matrix.profiles[0].journeys, [...common, 'detached-session']);
  assert.deepEqual(matrix.profiles[1].journeys, [...common, 'text-response-bounds', 'text-safe-rendering', 'detached-session']);
});

test('the existing interoperability binary owns both complete acceptances and one fresh matrix', () => {
  const rust = read('crates/prismpm/tests/hologram_interop.rs');
  assert.match(rust, /#\[test\]\s+fn test_portable_view_profiles_and_owned_failure_matrix\(\)/);
  for (const name of ['calculator_hologram_oracle_interoperability_acceptance', 'text_application_hologram_oracle_interoperability_acceptance']) {
    assert.equal(rust.split(`fn ${name}()`).length, 2);
    assert.equal(rust.split(`= ${name}();`).length, 2);
  }
  assert.equal(rust.split('scripts/portable-oracle-matrix.mjs').length, 2);
  assert.equal(rust.split('#[test]').length - 1, 3, 'source pin, report negatives and both-profile matrix owners remain unconditional');
  assert.equal(/#\[(?:ignore|cfg|cfg_attr)/.test(rust), false);
  const owner = read('scripts/portable-oracle-matrix.mjs');
  assert(owner.includes(`assert.equal(matrix.interaction_cases.length, ${matrix.interaction_cases.length});`));
  assert(owner.includes(`assert.equal(new Set([...matrix.interaction_cases, ...matrix.infrastructure_cases]).size, ${matrix.interaction_cases.length + matrix.infrastructure_cases.length});`));
  assert.equal(owner.split("run('fresh-compiler'").length, 2);
  assert.match(owner, /'build', '--locked', '--offline'/);
  assert.match(owner, /assert\.equal\(outcomes\.length, 78/);
  assert.match(owner, /executable\.verify\(\)/);
});

test('portable probe rejects undeclared triggers and infrastructure keyboard claims before opening inputs', () => {
  for (const [name, trigger, message] of [['positive', 'unknown', /closed trigger/],
    ['setup-failure', 'keyboard', /do not claim keyboard execution/], ['unknown', 'click', /closed case/]]) {
    const result = spawnSync(process.execPath, [new URL('./portable-oracle-submission-probe.mjs', import.meta.url).pathname,
      '/not-opened', '/not-opened', '/not-opened', name, '/not-created', trigger],
    {encoding: 'utf8', timeout: 10000, maxBuffer: 1024 * 1024});
    assert.ifError(result.error);
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, message);
  }
});
