// Source-custody checks only; the complete generated owner remains mandatory.
import assert from 'node:assert/strict';
import test from 'node:test';
import {chmodSync,copyFileSync,linkSync,lstatSync,mkdirSync,mkdtempSync,readFileSync,renameSync,rmSync,symlinkSync,unlinkSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname,join} from 'node:path';
import {pathToFileURL} from 'node:url';
import {provenanceProfile,presentationOutputRows} from '../browser-presentation/provenance.mjs';
import {modules,frozenInputs,assertFrozenInputs,repository,sha} from './compile.mjs';
import {sourceRoots,suites} from '../../scripts/browser-api-sdk-check.mjs';

test('session provenance admits only the registered complete family, never a caller subset',()=>{
  for(const value of [null,[],{},'','session/../presentation','caller-family',
    {toString(){throw Error('coercion');}}])
    assert.throws(()=>provenanceProfile(value),/registered complete provenance family/);
  const profile=provenanceProfile('session');
  assert.deepEqual(profile.outputRows.filter(row=>row.kind==='lean').map(row=>
    row.path.slice('modules/PrismPM/'.length,-'.lean'.length).replaceAll('/','.')),modules);
  assert.equal(profile.outputRows.length,50);
  assert.deepEqual(provenanceProfile('presentation').outputRows,presentationOutputRows);
  assert.equal(presentationOutputRows.length,30);
  assert.throws(()=>{profile.outputRows.pop();},TypeError);
  assert.throws(()=>{profile.outputRows[0].path='omitted';},TypeError);
});

test('session frozen input authority cannot be forged by copying coherent hash labels',()=>{
  const inputs=frozenInputs();
  assertFrozenInputs(inputs);
  assert.throws(()=>assertFrozenInputs({...inputs}),/actual frozen session input closure/);
  for(const path of Object.keys(inputs))
    assert.ok(sourceRoots.some(root=>path===root||path.startsWith(root+'/')),
      'installed SDK covers complete session owner input '+path);
});

test('every session barrier freshly checks original file custody and manifest membership',async t=>{
  const root=mkdtempSync(join(tmpdir(),'prismpm-session-input-custody-'));
  t.after(()=>rmSync(root,{recursive:true,force:true}));
  const original=frozenInputs();
  for(const path of Object.keys(original)){
    const destination=join(root,path);mkdirSync(dirname(destination),{recursive:true});
    copyFileSync(join(repository,path),destination);
  }
  const isolated=await import(pathToFileURL(join(root,'tests/browser-session/compile.mjs')).href);
  const observe=()=>{const inputs=isolated.frozenInputs();isolated.assertFrozenInputs(inputs);return inputs;};
  const first=observe(),member='tests/browser-session/runner.rs',path=join(root,member);
  for(const forged of [Object.freeze({...first}),Object.freeze({...first,[member]:'0'.repeat(64)}),
    Object.freeze(Object.fromEntries(Object.entries(first).filter(([key])=>key!==member)))])
    assert.throws(()=>isolated.assertFrozenInputs(forged),/actual frozen session input closure/);
  const bytes=readFileSync(path),mode=lstatSync(path).mode&0o777;
  for(const kind of ['content','deleted','inode','mode','hardlink','symlink']){
    const inputs=observe(),saved=path+'.original';
    try{
      if(kind==='content')writeFileSync(path,Buffer.concat([bytes,Buffer.from('\n')]));
      if(kind==='mode')chmodSync(path,mode^0o040);
      if(['deleted','inode','hardlink','symlink'].includes(kind)){
        renameSync(path,saved);
        if(kind==='inode')copyFileSync(saved,path);
        if(kind==='hardlink')linkSync(saved,path);
        if(kind==='symlink')symlinkSync(saved,path);
      }
      assert.throws(()=>isolated.assertFrozenInputs(inputs),undefined,'actual '+kind+' substitution');
    }finally{
      if(['deleted','inode','hardlink','symlink'].includes(kind)){
        if(kind!=='deleted')unlinkSync(path);
        renameSync(saved,path);
      }else {writeFileSync(path,bytes);chmodSync(path,mode);}
    }
  }
  // Coherently re-pinning changed vendor bytes must not replace the ORIGINAL
  // lifetime capture, even when a new capture accepts the changed pin set.
  const manifestPath=join(root,'vendor/lexlean/MANIFEST.sha256');
  const dependenciesPath=join(root,'model/dependencies.toml');
  const manifest=readFileSync(manifestPath,'utf8'),dependencies=readFileSync(dependenciesPath,'utf8');
  const [,digest,vendorMember]=/^([0-9a-f]{64})  (.+)$/m.exec(manifest);
  const vendorPath=join(root,'vendor/lexlean',vendorMember),vendorBytes=readFileSync(vendorPath);
  for(const kind of ['changed-member','added-member']){
    const inputs=observe(),added=join(root,'vendor/lexlean/custody-added-input');
    try{
      let changed;
      if(kind==='changed-member'){
        const next=Buffer.concat([vendorBytes,Buffer.from('\n')]);writeFileSync(vendorPath,next);
        changed=manifest.replace(digest+'  '+vendorMember,sha(next)+'  '+vendorMember);
      }else {
        writeFileSync(added,'new manifest member\n',{flag:'wx'});
        changed=manifest+sha(readFileSync(added))+'  custody-added-input\n';
      }
      writeFileSync(manifestPath,changed);
      const oldPin='sha256 = "'+sha(Buffer.from(manifest))+'"';assert(dependencies.includes(oldPin));
      writeFileSync(dependenciesPath,dependencies.replace(oldPin,'sha256 = "'+sha(Buffer.from(changed))+'"'));
      observe();
      assert.throws(()=>isolated.assertFrozenInputs(inputs),/custody/,'coherent '+kind+' must refuse original capture');
    }finally{
      writeFileSync(vendorPath,vendorBytes);writeFileSync(manifestPath,manifest);
      writeFileSync(dependenciesPath,dependencies);
      if(kind==='added-member')unlinkSync(added);
    }
  }
  const inputs=observe();isolated.assertFrozenInputs(inputs);
  writeFileSync(path,Buffer.concat([bytes,Buffer.from('\n')]));
  assert.throws(()=>isolated.assertFrozenInputs(inputs),/custody/,'success cannot memoize the next barrier');
  writeFileSync(path,bytes);
  const compilerPath=join(root,'tests/browser-session/compile.mjs'),compiler=readFileSync(compilerPath,'utf8');
  const check='for (const [path,evidence] of captured) capturedFile(join(repository,path),evidence);';
  assert.equal(compiler.split(check).length,2,'one actual per-member barrier to mutate');
  const mutantPath=join(root,'tests/browser-session/compile-omitted-custody.mjs');
  writeFileSync(mutantPath,compiler.replace(check,
    'for (const [path,evidence] of captured) if(path!=="tests/browser-session/runner.rs") capturedFile(join(repository,path),evidence);'),{flag:'wx'});
  const mutant=await import(pathToFileURL(mutantPath).href),mutantInputs=mutant.frozenInputs();
  mutant.assertFrozenInputs(mutantInputs);
  writeFileSync(path,Buffer.concat([bytes,Buffer.from('\n')]));
  const mustReject=()=>assert.throws(()=>mutant.assertFrozenInputs(mutantInputs),/custody/);
  assert.throws(mustReject,/Missing expected exception/,'the real changed-member assertion kills an omitted-check mutant');
});

test('source and installed session registration retain the entire original owner and added custody checks',()=>{
  const owner=suites.find(row=>row.id==='DK-26');
  assert.deepEqual(owner.files,['sdk/browser/session-model-test.mjs',
    'tests/browser-session/wire.test.mjs','tests/browser-session/provenance.test.mjs']);
  assert.equal(owner.minimum,37);assert.equal(owner.deadline,3600000);
});
