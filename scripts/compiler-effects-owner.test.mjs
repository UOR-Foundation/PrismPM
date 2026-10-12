// Wiring/negative controls only; complete real DK-20 execution remains required.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {prepare} from '../tests/browser-effects/compile.mjs';

test('effects compiler refuses caller tools before creating a model or private build', () => {
  let called = false;
  for (const owner of [{}, {runDriver() {called = true;}, runExporter() {called = true;}, verify() {called = true;}},
    {evidence: {family: 'effects'}}])
    assert.throws(() => prepare(null, owner), /actual fresh compiler owner/);
  assert.equal(called, false);
});

test('effects family tool sharing keeps every fresh model/kernel/native/Wasm stage and all original cases', () => {
  const read = path => readFileSync(new URL('../' + path, import.meta.url), 'utf8');
  const compile = read('tests/browser-effects/compile.mjs');
  const check = source => {
    for (const anchor of ["requireCompilerOwner(compilerOwner, 'effects')", 'shared.runDriver(args, repository)',
      "run('lake', ['update'], project)", "invokeDriver(['verify', join(project, 'lexlean.toml')])",
      "run('lake', ['build', 'PrismGenerated'], lean)", 'shared.runExporter(args, join(lean,',
      "invokeExporter(['--module', 'PrismPM.Fixture'", "invokeDriver(['native', ir, generated, repository])",
      "invokeDriver([mode, ir, guest, repository])", "[['a', 'wasm'], ['b', 'wasm'], ['echo', 'guest']]",
      "run('cargo', ['build', '--locked', '--offline', '--release'], guest",
      'if (shared) shared.verify(); else compiler.unchanged();',
      "run('cargo', ['build', '--locked', '--offline', '--jobs', '1'", 'createPrivateDriverTarget(work)',
      'else {', 'compiler = stageCompiler(work);', 'shared ? null : ensureProdExport(repository, work)',
      'mutation = null, compilerOwner = null', "[null, 'binding', 'trailing', 'unknown']"])
      assert.equal(source.split(anchor).length, 2, 'one original stage: ' + anchor);
  };
  check(compile);
  for (const anchor of ['shared.runDriver(args, repository)', "invokeDriver(['verify', join(project, 'lexlean.toml')])",
    "run('lake', ['build', 'PrismGenerated'], lean)", "invokeDriver([mode, ir, guest, repository])",
    'if (shared) shared.verify(); else compiler.unchanged();', 'createPrivateDriverTarget(work)'])
    assert.throws(() => check(compile.replace(anchor, '/* planted omitted stage */')), /one original stage/);
  const entry = read('sdk/browser/effects-test.mjs');
  for (const anchor of ["createCompilerOwner('effects')", 'verifyCompilerOwnerSubstitutions(owner)',
    'verifyWire(t, owner)', 'verifyModelMutation(kind, owner)', 'owner.close()',
    "['binding', 'trailing', 'unknown']", 'verifyJourneys(child, build)', 'verifyHostMutants(child, build)',
    'timeout: 3500000', 'assert.equal(build.compilerOwner, owner.identity)',
    'assert.equal(result.compilerOwner, owner.identity)']) assert(entry.includes(anchor), anchor);
  for (const anchor of ['assert.deepEqual(substitutions,', 'assert.equal(browser.cases.length, 30)',
    'assert.equal(hostMutants.length, 10)', 'assert.deepEqual(modelMutants.map(row => row.mutation),'])
    assert(entry.includes(anchor), anchor);
  const checks = read('tests/browser-effects/checks.mjs');
  for (const anchor of ['corpus().length, 216', 'vectors.length, 242',
    '[true, false]', 'executeWasm(build.wasmBytes, [...vectors, ...maximum])',
    'prepare(null, compilerOwner)', 'prepare(kind, compilerOwner)',
    'executeWasm(build.wasmBytes, [vector])', "'wire acceptance source closure remained frozen'"])
    assert(checks.includes(anchor), anchor);
  const audit = read('xtask/src/main.rs');
  assert.equal(audit.split('"scripts/compiler-effects-owner.test.mjs"').length, 2,
    'source audit executes the additive controls');
});
