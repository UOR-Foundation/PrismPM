import assert from 'node:assert/strict';
import {copyFileSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname, join, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {mutateSource} from './mutations.mjs';
import {retireCompletedCompilerCaches} from '../browser-view/driver-cache.mjs';
// Reuse the existing pinned-toolchain, override-refusing process boundary.
import {run, sha} from '../browser-view/compile.mjs';
export {run, sha};
export const draft = dirname(fileURLToPath(import.meta.url));
export const repository = resolve(draft, '../..');
export const modules = Object.freeze(['Fixture', 'Foundation.Browser.Application.V1.Effects', 'Foundation.Browser.Application.V1.EffectsWire',
  'Foundation.Browser.Application.V1.Session', 'Foundation.Browser.Application.V1.SessionWire',
  'Foundation.View.Browser.V1.Model', 'Foundation.View.Browser.V1.Wire', 'Foundation.Bytes',
  'Foundation.Codec', 'Foundation.Codec.Cbor.V1.Primitive'].sort());
const modulePath = name => (name === 'Fixture' ? 'tests/browser-session' : 'stdlib') + '/src/' + name.replaceAll('.', '/') + '.lex.tex';

export function frozenInputs() {
  pins();
  const files = new Set([
    ...modules.map(modulePath),
    ...['checks.mjs','compile.mjs','corpus.mjs','maxima.mjs','maximum-runner.mjs','mutations.mjs','size-corpus.mjs',
      'wire.mjs','wire.test.mjs','budget.mjs','runner.rs','driver/Cargo.toml','driver/Cargo.lock','driver/src/main.rs'].map(path=>'tests/browser-session/'+path),
    'sdk/browser/session-model-test.mjs','sdk/browser/identity.mjs','sdk/browser/effects-wire.mjs','sdk/browser/effects-module.mjs','sdk/browser/presentation-wire.mjs',
    'tests/browser-view/compile.mjs','tests/browser-view/driver-cache.mjs','tests/browser-view/prerequisites.mjs','tests/browser-effects/corpus.mjs',
    'tests/browser-presentation/corpus.mjs','tests/browser-presentation/maximum-fixtures.mjs',
    'tests/browser-workspace/src/main.rs','tests/browser-journal/driver/src/main.rs',
    'tests/fixtures/library/native-library/project/lexlean.toml','rust-toolchain.toml','lean-toolchain',
    'LICENSE-MIT','LICENSE-APACHE','model/authorities.toml','model/dependencies.toml',
    'vendor/lean4-prod/lean.tar','vendor/lean4-prod/rust/MANIFEST.sha256','vendor/lexlean/MANIFEST.sha256',
  ]);
  for (const tree of ['vendor/lexlean','vendor/lean4-prod/rust']) {
    for (const line of readFileSync(join(repository,tree,'MANIFEST.sha256'),'utf8').trimEnd().split('\n')) {
      const row=/^([0-9a-f]{64})  ([A-Za-z0-9_./-]+)$/.exec(line); assert.ok(row);
      files.add(tree+'/'+row[2]);
    }
  }
  return Object.freeze(Object.fromEntries([...files].sort().map(path=>[path,sha(readFileSync(join(repository,path)))])));
}

export function assertFrozenInputs(expected) {
  assert.deepEqual(frozenInputs(),expected,'complete frozen session owner inputs');
}

function capturedInput(path, inputs) {
  const bytes=readFileSync(join(repository,path));
  assert.equal(sha(bytes),inputs[path],'frozen captured source/tool input '+path);
  return bytes;
}

export function assertBaselineSources(actual, expected) {
  assert.ok(expected instanceof Map, 'actual positive source closure required before mutations');
  assert.deepEqual([...actual.keys()].sort(), [...expected.keys()].sort(), 'complete positive source module inventory');
  for (const [module, bytes] of actual) assert.deepEqual(bytes, expected.get(module), 'immutable positive source before mutation ' + module);
}

function pins() {
  const artifacts = readFileSync(join(repository, 'model/dependencies.toml'), 'utf8')
    .split('[[dependency.artifact]]').slice(1).map(section => {
      const text = section.split('[[dependency]]')[0];
      return {path: /^path = "([^"]+)"$/m.exec(text)?.[1],
        hash: /^sha256 = "([0-9a-f]{64})"$/m.exec(text)?.[1],
        tree: /^tree_root = "([^"]+)"$/m.exec(text)?.[1]};
    });
  for (const name of ['vendor/lean4-prod/lean.tar', 'vendor/lean4-prod/rust/MANIFEST.sha256', 'vendor/lexlean/MANIFEST.sha256']) {
    const rows = artifacts.filter(row => row.path === name); assert.equal(rows.length, 1);
    const [{hash, tree}] = rows, bytes = readFileSync(join(repository, name));
    assert.equal(sha(bytes), hash, name);
    if (!tree) continue;
    const seen = new Set();
    for (const line of bytes.toString('utf8').trimEnd().split('\n')) {
      const row = /^([0-9a-f]{64})  ([A-Za-z0-9_./-]+)$/.exec(line); assert.ok(row);
      const [, digest, path] = row;
      assert.ok(!path.startsWith('/') && !path.split('/').some(part => !part || part === '.' || part === '..'));
      assert.ok(!seen.has(path)); seen.add(path);
      assert.equal(sha(readFileSync(join(repository, tree, path))), digest, path);
    }
  }
}

function stageCompiler(work, inputs) {
  // Cargo resolves inherited workspace lint/dependency policy while loading
  // path dependencies. Keep each pinned vendor workspace intact, outside any
  // containing checkout/worktree workspace; do not override its policy.
  const captured = [];
  for (const tree of ['vendor/lexlean', 'vendor/lean4-prod/rust']) {
    const entries = readFileSync(join(repository, tree, 'MANIFEST.sha256'), 'utf8').trimEnd().split('\n');
    for (const entry of entries) {
      const [, digest, relative] = /^([0-9a-f]{64})  ([A-Za-z0-9_./-]+)$/.exec(entry);
      const source = join(repository, tree, relative), destination = join(work, tree, relative);
      const bytes = readFileSync(source); assert.equal(sha(bytes), digest);
      assert.equal(digest,inputs[tree+'/'+relative],'frozen staged vendor input');
      mkdirSync(dirname(destination), {recursive: true}); writeFileSync(destination, bytes, {flag: 'wx'});
      captured.push({source, destination, digest});
    }
  }
  for (const relative of ['Cargo.toml', 'Cargo.lock', 'src/main.rs']) {
    const source = join(draft, 'driver', relative), destination = join(work, 'tests/browser-session/driver', relative);
    const bytes = readFileSync(source), digest = sha(bytes);
    assert.equal(digest,inputs['tests/browser-session/driver/'+relative],'frozen staged driver input');
    mkdirSync(dirname(destination), {recursive: true}); writeFileSync(destination, bytes, {flag: 'wx'});
    captured.push({source, destination, digest});
  }
  return {manifest: join(work, 'tests/browser-session/driver/Cargo.toml'), unchanged() {
    for (const {source, destination, digest} of captured) {
      assert.equal(sha(readFileSync(source)), digest, 'frozen compiler source ' + source);
      assert.equal(sha(readFileSync(destination)), digest, 'frozen staged compiler ' + destination);
    }
  }};
}

export function prepare(mutation = null, baseline = null, inputs = frozenInputs()) {
  for (const key of Object.keys(process.env)) assert.ok(!key.startsWith('PRISMPM_SESSION_'), 'session acceptance refuses bypass ' + key);
  assertFrozenInputs(inputs);
  const work = mkdtempSync(join(tmpdir(), 'prismpm-session-'));
  const sourcePath = name => join(name === 'Fixture' ? draft : join(repository, 'stdlib'), 'src', ...name.split('.')) + '.lex.tex';
  const sources = new Map(modules.map(name => [name, readFileSync(sourcePath(name))]));
  const originals = new Map(sources);
  for (const [module, bytes] of originals) assert.equal(sha(bytes),inputs[modulePath(module)],'frozen captured original source '+module);
  if (mutation !== null) assertBaselineSources(originals, baseline);
  const planted = mutation === null ? null : mutateSource(sources, mutation);
  let completed = false;
  try {
    const project = join(work, 'project');
    for (const [module, bytes] of sources) {
      const destination = join(project, 'src', ...module.split('.')) + '.lex.tex';
      mkdirSync(dirname(destination), {recursive: true}); writeFileSync(destination, bytes, {flag: 'wx'});
    }
    const base = capturedInput('tests/fixtures/library/native-library/project/lexlean.toml',inputs).toString('utf8')
      .replace('name = "library-probe"', 'name = "session-conformance"')
      .replace('module_prefix = "LibraryProbe"', 'module_prefix = "PrismPM"')
      .replace('src/Probe.lex.tex', 'src/Fixture.lex.tex');
    writeFileSync(join(project, 'lexlean.toml'), base, {flag: 'wx'});
    writeFileSync(join(project, 'lakefile.toml'), 'name = "session_conformance"\nversion = "0.1.0"\n', {flag: 'wx'});
    writeFileSync(join(project,'lean-toolchain'),capturedInput('lean-toolchain',inputs),{flag:'wx'});
    writeFileSync(join(work,'rust-toolchain.toml'),capturedInput('rust-toolchain.toml',inputs),{flag:'wx'});
    const driverTarget = join(work, 'driver-target');
    const compiler = stageCompiler(work,inputs);
    run('cargo', ['build', '--locked', '--offline', '--jobs', '1', '--config', 'profile.dev.debug=0',
      '--config', 'build.incremental=false', '--manifest-path', compiler.manifest], work, {CARGO_TARGET_DIR: driverTarget});
    const driver = join(driverTarget, 'debug/browser-session-driver');
    run('lake', ['update'], project);
    const verified = JSON.parse(run(driver, ['verify', join(project, 'lexlean.toml')], repository));
    assert.deepEqual(verified.modules, modules);
    const manifestBytes = readFileSync(join(verified.root, 'build-manifest.json'));
    const attestationBytes = readFileSync(join(verified.root, 'attestation.json'));
    const attestation = JSON.parse(attestationBytes);
    assert.equal(attestation.status, 'verified');
    assert.equal(attestation.build_manifest.sha256, sha(manifestBytes));
    assert.equal(attestation.attestation_id, verified.attestation_id);
    const expected = new Map([...sources].flatMap(([module, bytes]) => {
      const semantic = JSON.parse(/\\semanticdata\{(.*)\}/.exec(bytes.toString('utf8'))[1]);
      return semantic.declarations.map(declaration => ['PrismPM.' + module + '.' + declaration.name,
        {kind: declaration.axioms?.length ? 'exact' : 'none', axioms: declaration.axioms ?? []}]);
    }));
    assert.equal(attestation.declarations.length, expected.size);
    for (const audit of attestation.declarations) {
      assert.equal(audit.result, 'ok'); assert.ok(expected.has(audit.name));
      const policy = expected.get(audit.name); assert.deepEqual(audit.policy, policy);
      assert.deepEqual(audit.observed, policy.axioms); expected.delete(audit.name);
    }
    assert.equal(expected.size, 0);
    const lean = join(work, 'lean'); mkdirSync(lean);
    for (const module of modules) {
      const relative = join('PrismPM', ...module.split('.')) + '.lean', destination = join(lean, relative);
      mkdirSync(dirname(destination), {recursive: true}); copyFileSync(join(verified.root, 'modules', relative), destination);
    }
    writeFileSync(join(lean,'lean-toolchain'),capturedInput('lean-toolchain',inputs),{flag:'wx'});
    writeFileSync(join(lean, 'lakefile.toml'), 'name = "session_probe"\nversion = "0.1.0"\n[[lean_lib]]\nname = "PrismGenerated"\nroots = [' + modules.map(name => '"PrismPM.' + name + '"').join(',') + ']\n', {flag: 'wx'});
    run('lake', ['build', 'PrismGenerated'], lean);
    const exporter = join(work, 'exporter'); mkdirSync(exporter);
    const exporterArchive=join(work,'exporter-source.tar');
    writeFileSync(exporterArchive,capturedInput('vendor/lean4-prod/lean.tar',inputs),{flag:'wx'});
    run('tar', ['-xf', exporterArchive, '-C', exporter], repository);
    run('lake', ['build', 'prod-export'], exporter);
    const exported = join(work, 'export');
    const roots = ['PrismPM.Foundation.Browser.Application.V1.SessionWire.sourceSessionWireBytes',
      'PrismPM.Fixture.fixtureSourceDispatchBytes', 'PrismPM.Fixture.fixtureSourceSizeBytes'].sort();
    run(join(exporter, '.lake/build/bin/prod-export'), ['--module', 'PrismPM.Fixture', ...roots.flatMap(root => ['--root', root]),
      '--ir-module', 'BrowserSession', '--out', exported], exporter, {LEAN_PATH: join(lean, '.lake/build/lib/lean')});
    const generated = join(work, 'generated'), ir = join(exported, 'kernel.ir');
    const licenses=join(work,'licenses');mkdirSync(licenses);
    for(const path of ['LICENSE-MIT','LICENSE-APACHE'])writeFileSync(join(licenses,path),capturedInput(path,inputs),{flag:'wx'});
    const generation = JSON.parse(run(driver, ['native', ir, generated, licenses], repository));
    const runner = join(work, 'runner'); mkdirSync(join(runner, 'src'), {recursive: true});
    writeFileSync(join(runner,'src/main.rs'),capturedInput('tests/browser-session/runner.rs',inputs),{flag:'wx'});
    writeFileSync(join(runner, 'Cargo.lock'), 'version = 4\n[[package]]\nname = "browser-session-core-probe"\nversion = "0.1.0"\n[[package]]\nname = "browser-session-runner"\nversion = "0.1.0"\ndependencies = ["browser-session-core-probe"]\n', {flag: 'wx'});
    function compileNative(standard) {
      writeFileSync(join(runner, 'Cargo.toml'), '[package]\nname = "browser-session-runner"\nversion = "0.1.0"\nedition = "2021"\npublish = false\n[workspace]\n[dependencies]\nbrowser-session-core-probe = {path = "../generated", default-features = ' + standard + '}\n');
      run('cargo', ['build', '--locked', '--offline', '--release', '--manifest-path', join(runner, 'Cargo.toml')], runner, {CARGO_TARGET_DIR: join(work, 'native-target')});
      return join(work, 'native-target/release/browser-session-runner');
    }
    const guests = [], fixtures = [], sizes = [];
    const modes=planted ? [[planted.fixture?'fixture-mutant':'mutant',planted.fixture?'fixture':'wasm']]
      : [['a', 'wasm'], ['b', 'wasm'], ['fixture-a', 'fixture'], ['fixture-b', 'fixture'], ['size-a', 'size'], ['size-b', 'size']];
    for (const [label, mode] of modes) {
      const guest = join(work, 'guest-' + label);
      assert.deepEqual(JSON.parse(run(driver, [mode, ir, guest, licenses], repository)), generation);
      run('cargo', ['build', '--locked', '--offline', '--release'], guest, {CARGO_TARGET_DIR: join(guest, 'target')});
      (mode === 'fixture' ? fixtures : mode === 'size' ? sizes : guests).push(readFileSync(join(guest, 'target/wasm32-unknown-unknown/release/browser_session_' + (mode === 'wasm' ? 'wire' : mode) + '_probe.wasm')));
    }
    if(!planted) {
      assert.deepEqual(guests[0], guests[1], 'two independent generated Core-Wasm packages');
      assert.deepEqual(fixtures[0], fixtures[1], 'two independent generated source wrapper packages');
      assert.deepEqual(sizes[0], sizes[1], 'two independent generated real writer-size parity packages');
    }
    pins(); compiler.unchanged(); assert.equal(generation.ir_sha256, sha(readFileSync(ir)));
    for (const [module, bytes] of originals) assert.deepEqual(readFileSync(sourcePath(module)), bytes, 'frozen source ' + module);
    for (const [module, bytes] of sources) assert.deepEqual(readFileSync(join(project, 'src', ...module.split('.')) + '.lex.tex'), bytes, 'exact staged source after compiler execution ' + module);
    const cacheRetirement = retireCompletedCompilerCaches(work, 'session');
    pins(); compiler.unchanged(); assert.equal(generation.ir_sha256, sha(readFileSync(ir)));
    assert.deepEqual(readFileSync(join(verified.root, 'build-manifest.json')), manifestBytes);
    assert.deepEqual(readFileSync(join(verified.root, 'attestation.json')), attestationBytes);
    for (const [module, bytes] of originals) assert.deepEqual(readFileSync(sourcePath(module)), bytes, 'unchanged source after tool cache retirement ' + module);
    for (const [module, bytes] of sources) assert.deepEqual(readFileSync(join(project, 'src', ...module.split('.')) + '.lex.tex'), bytes, 'unchanged staged source after tool cache retirement ' + module);
    for(const path of ['LICENSE-MIT','LICENSE-APACHE'])assert.equal(sha(readFileSync(join(licenses,path))),inputs[path],'unchanged staged license');
    assert.equal(sha(readFileSync(exporterArchive)),inputs['vendor/lean4-prod/lean.tar'],'unchanged staged exporter source');
    assertFrozenInputs(inputs);
    completed = true;
    return {work, sources, verified, generation, compileNative, runner, wasmBytes: guests[0], fixtureBytes: fixtures[0], sizeBytes: sizes[0], mutation:planted, cacheRetirement};
  } finally { if (!completed) process.stderr.write('Retained incomplete session diagnostic build ' + work + '\n'); }
}
