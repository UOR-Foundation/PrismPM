// Real small filesystem/descriptor adversaries. No OCI/provider/SDK claim.
import assert from 'node:assert/strict';
import test from 'node:test';
import {spawnSync} from 'node:child_process';
import {constants,chmodSync,existsSync,fstatSync,linkSync,mkdirSync,mkdtempSync,
 readFileSync,readlinkSync,readdirSync,renameSync,rmSync,symlinkSync,writeFileSync,writeSync} from 'node:fs';
import {join,basename} from 'node:path';
import {tmpdir} from 'node:os';
import {setImmediate as yieldTurn} from 'node:timers/promises';
import {validateSourceCiContract} from './sdk-source-ci-contract.mjs';
import {holdSourceCiArchiveData,loanSourceCiArchiveData,closeSourceCiArchiveData} from './sdk-source-ci-archive-loan.mjs';
import {fixture,bytes,sha,clone} from './sdk-source-ci-contract-fixture.mjs';
const admit=f=>validateSourceCiContract(bytes(f.provider),bytes(f.envelope),f.files,f.expected,f.now);
async function temporary(run){const root=mkdtempSync(join(tmpdir(),'source-ci-data-test-'));chmodSync(root,0o700);
 try{const f=fixture(),path=join(root,'sdk.oci.tar');writeFileSync(path,f.archive,{flag:'wx',mode:0o400});return await run(root,path,f);}
 finally{rmSync(root,{recursive:true});}}
const held=()=>new Map(readdirSync('/proc/self/fd').flatMap(fd=>{try{return [[fd,readlinkSync('/proc/self/fd/'+fd)]];}catch(e){if(e.code==='ENOENT')return [];throw e;}}));
test('original small bytes stay in one real O_RDONLY descriptor and close never deletes caller data',async()=>temporary(async(root,path,f)=>{
 const h=await holdSourceCiArchiveData(admit(f),path);assert(Object.isFrozen(h));assert.deepEqual(Object.keys(h),[]);let fd;
 const result=await loanSourceCiArchiveData(h,async input=>{
  assert(Object.isFrozen(input));assert.match(input.scope,/not-execution-authority/);assert.deepEqual(readFileSync(input.path),f.archive);
  fd=Number(basename(input.path));assert.equal(readlinkSync('/proc/self/fd/'+fd),path);
  const flags=/^flags:\s+([0-7]+)$/m.exec(readFileSync('/proc/self/fdinfo/'+fd,'utf8'));assert(flags);assert.equal(parseInt(flags[1],8)&3,constants.O_RDONLY);
  assert.throws(()=>writeSync(fd,Buffer.of(0),0,1,0),{code:'EBADF'});await yieldTurn();
 });
 assert.equal(result.digest,sha(f.archive));assert.match(result.scope,/not-import-or-sdk-acceptance/);assert.equal(result.status,undefined);
 assert.deepEqual(closeSourceCiArchiveData(h),{scope:'read-only-data-descriptors-closed-not-staging-retirement',descriptors_closed:true,file_deleted:false});
 assert.throws(()=>fstatSync(fd),{code:'EBADF'});assert.deepEqual(readFileSync(path),f.archive);assert.equal(readdirSync(root).length,1);
 assert.throws(()=>closeSourceCiArchiveData(h));await assert.rejects(loanSourceCiArchiveData(h,()=>{}));
}));
test('arbitrary copied or JSON metadata cannot impersonate held DATA custody',async()=>temporary(async(root,path,f)=>{
 const metadata=admit(f),h=await holdSourceCiArchiveData(metadata,path);
 for(const fake of [{},clone(h),metadata,{verified:true},{path,fd:5}]){
  await assert.rejects(loanSourceCiArchiveData(fake,()=>{}));assert.throws(()=>closeSourceCiArchiveData(fake));
 }
 await assert.rejects(holdSourceCiArchiveData(clone(metadata),path));closeSourceCiArchiveData(h);
}));
test('unsafe ownership modes parent aliases file symlinks hardlinks and special files are refused',async()=>temporary(async(root,path,f)=>{
 const metadata=admit(f);chmodSync(path,0o600);await assert.rejects(holdSourceCiArchiveData(metadata,path));chmodSync(path,0o400);
 const shared=join(root,'shared');linkSync(path,shared);await assert.rejects(holdSourceCiArchiveData(metadata,path));rmSync(shared);
 const link=join(root,'link');symlinkSync(path,link);await assert.rejects(holdSourceCiArchiveData(metadata,link));
 const directory=join(root,'directory');mkdirSync(directory,{mode:0o700});await assert.rejects(holdSourceCiArchiveData(metadata,directory));
 const pipe=join(root,'pipe'),made=spawnSync('mkfifo',[pipe],{timeout:5000,stdio:'pipe'});assert.equal(made.error,undefined);assert.equal(made.signal,null);assert.equal(made.status,0);
 await assert.rejects(holdSourceCiArchiveData(metadata,pipe));
 const privateRoot=join(root,'private');mkdirSync(privateRoot,{mode:0o700});const inner=join(privateRoot,'sdk.oci.tar');writeFileSync(inner,f.archive,{flag:'wx',mode:0o400});
 const alias=join(root,'alias');symlinkSync(privateRoot,alias);await assert.rejects(holdSourceCiArchiveData(metadata,join(alias,'sdk.oci.tar')));
 chmodSync(privateRoot,0o755);await assert.rejects(holdSourceCiArchiveData(metadata,inner));
 await assert.rejects(holdSourceCiArchiveData(metadata,root+'/private/../sdk.oci.tar'));
 assert.deepEqual(readFileSync(path),f.archive);
}));
test('wrong original byte digest and length cannot leak or publish a held descriptor',async()=>temporary(async(root,path,f)=>{
 for(const mutate of [e=>e.archive.digest='sha256:'+'0'.repeat(64),e=>e.archive.byte_length++]){
  const before=held(),bad=fixture();mutate(bad.envelope);await assert.rejects(holdSourceCiArchiveData(admit(bad),path));
  assert.deepEqual(held(),before,'failed original-byte admission must close both acquired descriptors');
 }
 for(const timeout of [0,-1,NaN,Infinity,1800001])await assert.rejects(holdSourceCiArchiveData(admit(f),path,timeout));
 assert.deepEqual(readFileSync(path),f.archive);
}));
test('one borrower owns the exact input and concurrent borrow or closure is refused',async()=>temporary(async(root,path,f)=>{
 const h=await holdSourceCiArchiveData(admit(f),path);let enter,release;const entered=new Promise(r=>enter=r),wait=new Promise(r=>release=r);
 const first=loanSourceCiArchiveData(h,async input=>{assert.deepEqual(readFileSync(input.path),f.archive);enter();await wait;});await entered;
 await assert.rejects(loanSourceCiArchiveData(h,()=>{}),/exclusive/);assert.throws(()=>closeSourceCiArchiveData(h),/inactive/);
 release();await first;await loanSourceCiArchiveData(h,()=>{});closeSourceCiArchiveData(h);
}));
test('a timed-out unresolved borrower is quarantined, never treated as observed process retirement',async()=>temporary(async(root,path,f)=>{
 const h=await holdSourceCiArchiveData(admit(f),path);let release,fd;const wait=new Promise(r=>release=r);
 await assert.rejects(loanSourceCiArchiveData(h,async input=>{fd=Number(basename(input.path));await wait;},200),/unsettled borrower quarantined/);
 assert(Number.isInteger(fd));assert(fstatSync(fd).isFile());assert.throws(()=>closeSourceCiArchiveData(h),/inactive/);
 await assert.rejects(loanSourceCiArchiveData(h,()=>{}),/exclusive/);release();await yieldTurn();await yieldTurn();
 await assert.rejects(loanSourceCiArchiveData(h,()=>{}),/unpoisoned/);closeSourceCiArchiveData(h);assert.throws(()=>fstatSync(fd),{code:'EBADF'});
}));
test('settled borrower failures poison reuse but still owe the complete post-loan custody check',async()=>temporary(async(root,path,f)=>{
 const h=await holdSourceCiArchiveData(admit(f),path);
 await assert.rejects(loanSourceCiArchiveData(h,()=>{throw Error('original borrower failure');}),/original borrower failure/);
 await assert.rejects(loanSourceCiArchiveData(h,()=>{}),/unpoisoned/);closeSourceCiArchiveData(h);
 const again=await holdSourceCiArchiveData(admit(f),path);let fd;
 await assert.rejects(loanSourceCiArchiveData(again,input=>{fd=Number(basename(input.path));chmodSync(path,0o600);writeFileSync(path,Buffer.alloc(f.archive.length,1));chmodSync(path,0o400);
  throw Error('original borrower failure with real mutation');}),error=>{
   assert(error instanceof AggregateError);assert.equal(error.errors.length,2);assert.match(error.errors[0].message,/original borrower failure/);assert.match(error.errors[1].message,/custody changed/);return true;
  });
 assert.throws(()=>closeSourceCiArchiveData(again),/closure uncertain/);assert.throws(()=>fstatSync(fd),{code:'EBADF'});assert(existsSync(path),'unowned changed caller file is preserved');
}));
test('post-loan source-byte mode alias and replacement drift never yields acceptance or unsafe deletion',async()=>temporary(async(root,path,f)=>{
 for(const mutate of [p=>chmodSync(p,0o600),p=>linkSync(p,join(root,'alias')),
  p=>{renameSync(p,join(root,'moved'));writeFileSync(p,'unrelated replacement',{mode:0o400});}]){
  const h=await holdSourceCiArchiveData(admit(f),path);let fd;
  await assert.rejects(loanSourceCiArchiveData(h,input=>{fd=Number(basename(input.path));mutate(path);}),/custody changed|independent regular/);
  assert.throws(()=>closeSourceCiArchiveData(h));assert.throws(()=>fstatSync(fd),{code:'EBADF'});assert(existsSync(path));
  // Test-fixture reset only, after the refused handle has closed every fd.
  for(const name of ['alias','moved','sdk.oci.tar'])rmSync(join(root,name),{force:true});writeFileSync(path,f.archive,{flag:'wx',mode:0o400});
 }
}));
test('directory substitution closes original held descriptors without touching replacements',async()=>temporary(async(root,path,f)=>{
 const parent=join(root,'owned');mkdirSync(parent,{mode:0o700});const original=join(parent,'sdk.oci.tar');writeFileSync(original,f.archive,{flag:'wx',mode:0o400});
 const h=await holdSourceCiArchiveData(admit(f),original);let fd;
 await assert.rejects(loanSourceCiArchiveData(h,input=>{fd=Number(basename(input.path));renameSync(parent,join(root,'moved-parent'));mkdirSync(parent,{mode:0o700});writeFileSync(join(parent,'preserve'),'unrelated');}),/custody changed/);
 assert.throws(()=>closeSourceCiArchiveData(h));assert.throws(()=>fstatSync(fd),{code:'EBADF'});assert.equal(readFileSync(join(parent,'preserve'),'utf8'),'unrelated');
 assert.deepEqual(readFileSync(join(root,'moved-parent','sdk.oci.tar')),f.archive);
}));
