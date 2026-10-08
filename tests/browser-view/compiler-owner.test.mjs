import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {runInNewContext} from 'node:vm';
import test from 'node:test';
import {captureCompilerInputs, createCompilerOwner, requireCompilerOwner} from './compiler-owner.mjs';
import {compilerDriverDirectory} from './compiler-owner-checks.mjs';

test('driver substitution lookup includes P256 and rejects missing or ambiguous source inventory', () => {
  for (const directory of ['browser-view', 'browser-session', 'browser-p256', 'browser-signed-context', 'browser-session-operation', 'browser-pkce', 'holo-primary-component', 'publication-admission', 'publication-context-linkage', 'browser-session-journal-retention']) {
    const path = 'tests/' + directory + '/driver/src/main.rs';
    assert.equal(compilerDriverDirectory({[path]: 'not-authority'}), 'tests/' + directory + '/driver');
  }
  for (const paths of [[], ['tests/browser-p123/driver/src/main.rs'],
    ['tests/browser-p256/driver/src/main.rs', 'tests/browser-session/driver/src/main.rs']])
    assert.throws(() => compilerDriverDirectory(Object.fromEntries(paths.map(path => [path, 'not-authority']))),
      /one source-bound family driver/);
});

test('verification-only source reads retain the complete descriptor and digest checks without whole-file buffers', t => {
  const root=fs.mkdtempSync(join(tmpdir(),'prismpm-source-stream-'));
  t.after(()=>fs.rmSync(root,{recursive:true,force:true}));
  const path=join(root,'source'),bytes=Buffer.alloc(1024*1024,97);
  fs.writeFileSync(path,bytes,{mode:0o600});
  const source=fs.readFileSync(new URL('./compiler-owner.mjs',import.meta.url),'utf8');
  const begin=source.indexOf('function unchanged(before, after) {');
  const end=source.indexOf('\n// A bounded filesystem observation',begin);
  assert(begin>=0&&end>begin);
  const allocations=[],alloc=Buffer.alloc.bind(Buffer);
  const measuredBuffer={alloc(size){allocations.push(size);return alloc(size);}};
  const file=runInNewContext(source.slice(begin,end)+'\nfile;',
    {...fs,assert,createHash,Buffer:measuredBuffer});
  const digest=createHash('sha256').update(bytes).digest('hex');
  assert.equal(file(path,digest,false),undefined);
  assert(Math.max(...allocations)<=65536,'verification streams rather than retaining the entire file');
  assert.deepEqual(Buffer.from(file(path,digest)),bytes,'actual staging still receives the complete source bytes');
  assert.throws(()=>file(path,'0'.repeat(64),false),/captured compiler source/);
  fs.appendFileSync(path,'changed');
  assert.throws(()=>file(path,digest,false),/captured compiler source/);
  fs.writeFileSync(path,bytes);
  const alias=join(root,'alias');fs.linkSync(path,alias);
  assert.throws(()=>file(path,digest,false),/single-link/);
  fs.unlinkSync(alias);fs.symlinkSync(path,alias);
  assert.throws(()=>file(alias,digest,false),/unaliased/);
  for(const anchor of ['file(join(repository, path), digest, false)', 'file(join(work, path), digest, false)'])
    assert.equal(source.split(anchor).length,2,'both actual verification loops select streaming checks');
});

test('compiler owner refuses forged handles and unregistered tool families', () => {
  for (const value of [null, {}, {runDriver() {}, runExporter() {}, verify() {}}])
    assert.throws(() => requireCompilerOwner(value, {}), /actual fresh compiler owner/);
  for (const name of ['', '../escape', 'caller-driver', null, {toString() {throw Error('coercion');}}])
    assert.throws(() => createCompilerOwner(name, {}), /registered compiler family/);
});

test('fresh independent drivers pin hash optimization without changing construction or acceptance',()=>{
  // Construction-argument unit only. Full owning executions remain required.
  const source=fs.readFileSync(new URL('./compiler-owner.mjs',import.meta.url),'utf8');
  const begin=source.indexOf("    run('cargo', ['build', '--locked', '--offline', '--jobs', '1', '--config', 'profile.dev.debug=0',");
  const end=source.indexOf('\n    const driverBuildMs',begin);
  assert(begin>=0&&end>begin);const calls=[];
  runInNewContext(source.slice(begin,end),{run(...args){calls.push(args);},join,
    work:'/actual-private-owner',manifest:'driver/Cargo.toml',target:'/actual-fresh-target'});
  assert.deepEqual(JSON.parse(JSON.stringify(calls)),[['cargo',[
    'build','--locked','--offline','--jobs','1','--config','profile.dev.debug=0',
    '--config','build.incremental=false','--config','profile.dev.package.sha2.opt-level=3',
    '--manifest-path','/actual-private-owner/driver/Cargo.toml'],
    '/actual-private-owner',{CARGO_TARGET_DIR:'/actual-fresh-target'}]]);
});

test('compiler owner refuses malformed or incomplete input closures before building', () => {
  let called = false;
  const accessor = {get malicious() {called = true; return '0'.repeat(64);}};
  for (const inputs of [null, [], accessor, {'../escape': '0'.repeat(64)},
    {'vendor/lexlean/MANIFEST.sha256': 'invalid'}, {'a': null}, {'a': 42}, {}])
    assert.throws(() => createCompilerOwner('view', inputs),
      /compiler input (map|path|digest|closure)/);
  assert.equal(called, false, 'input accessors cannot run before immutable capture');
  for (const name of ['view', 'effects', 'presentation', 'session', 'operation-journal', 'budget', 'custody', 'p256', 'signed-context', 'session-operation', 'pkce', 'holo-primary-component', 'session-retention', 'publication-linkage']) {
    const inputs = captureCompilerInputs(name);
    for (const missing of ['tests/browser-view/compile.mjs', 'tests/browser-view/compiler-artifact.mjs', 'vendor/lean4-prod/lean.tar']) {
      const changed = {...inputs}; delete changed[missing];
      assert.throws(() => createCompilerOwner(name, changed), /complete closed compiler input closure/);
    }
    assert.throws(() => createCompilerOwner(name, {...inputs, unexpected: '0'.repeat(64)}), /complete closed compiler input closure/);
  }
});
