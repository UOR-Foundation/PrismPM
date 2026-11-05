// Recording controls for scheduling; these are not generated-product owners.
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,rmSync,symlinkSync} from 'node:fs';
import {join,dirname,resolve} from 'node:path';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
import test from 'node:test';
import {standaloneOwnerIds,standaloneSuites,runStandaloneOwner} from './browser-api-sdk-check.mjs';
import {parts} from './vv-assemble.mjs';

function fixture(t){
 const root=mkdtempSync(join(tmpdir(),'main-vv-standalone-'));
 t.after(()=>rmSync(root,{recursive:true,force:true}));
 for(const row of standaloneSuites)for(const [i,file]of row.files.entries()){
  const path=join(root,file);mkdirSync(dirname(path),{recursive:true});
  writeFileSync(path,"import test from 'node:test';\n"+Array.from({length:row.exactTests[i]},(_,n)=>`test('recording ${n}',()=>{});\n`).join(''));
 }
 return root;
}
test('the six scheduled owners are the exact additional pairs, counts and original deadlines',async t=>{
 const root=fixture(t),seen=[];
 assert.equal(Object.keys(standaloneOwnerIds).length,6);
 for(const [owner,id]of Object.entries(standaloneOwnerIds)){
  assert(parts.includes('standalone-'+owner));
  const row=standaloneSuites.find(row=>row.id===id),calls=[];
  const launch=(program,args,options)=>{calls.push({program,args,options});return spawnSync(program,args,options);};
  assert.deepEqual(await runStandaloneOwner(root,owner,launch,()=>{}),[{id,tests:row.minimum}]);
  assert.equal(calls.length,1);const call=calls[0];
  assert.equal(call.program,process.execPath);assert.deepEqual(call.args.slice(4),row.files);
  assert.equal(call.args[3],'--test-timeout='+row.deadline);
  assert.equal(call.options.timeout,row.deadline);assert.equal(call.options.owned,true);
  assert.equal(call.options.maxBuffer,64*1024*1024);assert.equal(call.options.env.NODE_TEST_CONTEXT,undefined);
  seen.push(...row.files);
 }
 assert.equal(new Set(seen).size,12);
 assert.deepEqual(standaloneSuites.map(row=>row.exactTests),[[7,1],[10,23],[4,30],[3,1],[7,1],[4,18]]);
 assert.deepEqual(standaloneSuites.map(row=>row.deadline),[3600000,3600000,3600000,3600000,7200000,3600000]);
});
test('unknown owners, missing or aliased pair files fail before launch',async t=>{
 const root=fixture(t);let calls=0;
 const launch=()=>{calls++;throw Error('must not dispatch');};
 for(const owner of ['', 'unknown','../projection','Session Projection'])await assert.rejects(runStandaloneOwner(root,owner,launch,()=>{}),/closed/);
 const path=join(root,standaloneSuites[0].files[1]);rmSync(path);
 await assert.rejects(runStandaloneOwner(root,'projection',launch,()=>{}));
 symlinkSync(resolve(root,standaloneSuites[0].files[0]),path);
 await assert.rejects(runStandaloneOwner(root,'projection',launch,()=>{}),/alias/);
 assert.equal(calls,0);
});
test('coherent test redistribution cannot replace either original complete file',async t=>{
 const root=fixture(t),row=standaloneSuites[0];
 for(const [i,file]of row.files.entries())writeFileSync(join(root,file),"import test from 'node:test';\n"+Array.from({length:row.exactTests[1-i]},(_,n)=>`test('redistributed ${n}',()=>{});\n`).join(''));
 await assert.rejects(runStandaloneOwner(root,'projection',(p,a,o)=>spawnSync(p,a,o),()=>{}),/exact original standalone file test count/);
});
