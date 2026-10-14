// DATA custody only: this cannot admit a provider artifact, verify an OCI DAG,
// start an importer, or mint an existing construction-stage authority handle.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {constants,closeSync,fstatSync,lstatSync,openSync,readSync,realpathSync} from 'node:fs';
import {dirname,basename,resolve} from 'node:path';
import {setImmediate as yieldTurn} from 'node:timers/promises';
import {sourceCiArchiveDataDescriptor} from './sdk-source-ci-contract.mjs';

const loans=new WeakMap(),fields=['dev','ino','mode','uid','gid','nlink','size','mtimeNs','ctimeNs'];
const same=(a,b,selected=fields)=>{for(const k of selected)assert.equal(a[k],b[k],'source-CI input custody changed: '+k);};
const bounded=n=>assert(Number.isSafeInteger(n)&&n>0&&n<=1800000,'original bounded data loan required');
const deadline=end=>assert(performance.now()<=end,'source-CI data loan deadline exceeded');
function custody(row){
 const directory=fstatSync(row.root,{bigint:true});same(directory,row.directory);
 same(lstatSync(row.parent,{bigint:true}),directory);assert.equal(realpathSync(row.parent),row.parent,'aliased input parent refused');
 const file=fstatSync(row.fd,{bigint:true});assert(file.isFile()&&file.nlink===1n,'independent regular input required');same(file,row.file);
 same(lstatSync(row.anchored,{bigint:true}),file);return file;
}
async function verify(row,end){
 custody(row);const hash=createHash('sha256'),buffer=Buffer.alloc(65536);let at=0;
 while(at<row.archive.byte_length){deadline(end);const n=readSync(row.fd,buffer,0,Math.min(buffer.length,row.archive.byte_length-at),at);assert(n>0,'source-CI input shortened');hash.update(buffer.subarray(0,n));at+=n;await yieldTurn();}
 assert.equal(readSync(row.fd,buffer,0,1,at),0,'source-CI input grew');custody(row);deadline(end);
 assert.equal('sha256:'+hash.digest('hex'),row.archive.digest,'original source-CI byte identity differs');
}
function close(row,verifyCustody=true){
 const errors=[];if(verifyCustody)try{custody(row);}catch(e){errors.push(e);}
 for(const key of ['fd','root'])if(row[key]!==undefined){try{closeSync(row[key]);}catch(e){errors.push(e);}row[key]=undefined;}
 row.closed=true;if(errors.length)throw new AggregateError(errors,'source-CI data descriptor closure uncertain');
 return Object.freeze({scope:'read-only-data-descriptors-closed-not-staging-retirement',descriptors_closed:true,file_deleted:false});
}
// The enclosing whole-archive/staging owner must authenticate and verify the
// closed source-CI ZIP/OCI before any executable use. This seam only proves the
// expected original byte identity on an already private, sealed regular file.
export async function holdSourceCiArchiveData(metadataHandle,path,timeout=1800000){
 bounded(timeout);assert.equal(process.platform,'linux');assert.equal(typeof path,'string');assert(path.length<4096&&path===resolve(path));
 const descriptor=sourceCiArchiveDataDescriptor(metadataHandle),archive={byte_length:descriptor.byte_length,digest:descriptor.digest};
 const parent=dirname(path);assert(parent!=='/'&&realpathSync(parent)===parent);
 const directory=lstatSync(parent,{bigint:true});assert(directory.isDirectory()&&directory.uid===BigInt(process.geteuid())&&(directory.mode&0o7777n)===0o700n,'private owned input parent required');
 const row={parent,directory,archive,root:undefined,fd:undefined,closed:false,active:false,poisoned:false};
 try{
  row.root=openSync(parent,constants.O_RDONLY|constants.O_DIRECTORY|constants.O_NOFOLLOW|constants.O_NONBLOCK);
  same(fstatSync(row.root,{bigint:true}),directory);row.anchored='/proc/self/fd/'+row.root+'/'+basename(path);
  row.fd=openSync(row.anchored,constants.O_RDONLY|constants.O_NOFOLLOW|constants.O_NONBLOCK);
  row.file=fstatSync(row.fd,{bigint:true});assert(row.file.isFile()&&row.file.nlink===1n&&row.file.uid===BigInt(process.geteuid())&&
   (row.file.mode&0o7777n)===0o400n&&row.file.size===BigInt(archive.byte_length),'sealed single-owner original regular input required');
  await verify(row,performance.now()+timeout);const handle=Object.freeze({});loans.set(handle,row);return handle;
 }catch(error){try{close(row,false);}catch(cleanup){throw new AggregateError([error,cleanup],'source-CI data admission and closure failed');}throw error;}
}
export async function loanSourceCiArchiveData(handle,borrower,timeout=1800000){
 bounded(timeout);assert.equal(typeof borrower,'function');const row=loans.get(handle);
 assert(row&&!row.closed&&!row.active&&!row.poisoned,'live exclusive unpoisoned data handle required');row.active=true;
 const end=performance.now()+timeout;let timer,settled=false,expired=false;
 try{
  await verify(row,end);
  // A private O_RDONLY descriptor permits ORAS's existing seekable /proc path;
  // no second tar copy, writable fd, normalized Docker archive or stdin guess.
  const work=Promise.resolve().then(()=>borrower(Object.freeze({path:'/proc/'+process.pid+'/fd/'+row.fd,
   byte_length:row.archive.byte_length,digest:row.archive.digest,scope:'read-only-original-data-not-execution-authority'})))
   .then(value=>{settled=true;return value;},error=>{settled=true;throw error;});
  const timeoutFailure=new Promise((_,reject)=>{timer=setTimeout(()=>{expired=true;row.poisoned=true;reject(Error('source-CI data borrower deadline exceeded; unsettled borrower quarantined'));},Math.max(1,end-performance.now()));});
  let failure;
  try{await Promise.race([work,timeoutFailure]);}catch(error){failure=error;}
  finally{
   clearTimeout(timer);
   if(expired&&!settled){
    // A timed-out Promise is NOT observed child retirement. Never release or
    // lend/close this fd under an unresolved borrower. Keep the rejection owned.
    work.then(()=>{row.active=false;},()=>{row.active=false;});
   }
  }
  // A rejected but settled borrower still owes the complete post-read custody
  // check. Preserve its primary error together with any independent drift.
  if(!expired||settled)try{await verify(row,end);}catch(error){failure=failure?new AggregateError([failure,error],'borrower and post-loan custody failed'):error;}
  if(failure)throw failure;
  return Object.freeze({scope:'original-read-only-byte-loan-only-not-import-or-sdk-acceptance',byte_length:row.archive.byte_length,digest:row.archive.digest});
 }catch(error){row.poisoned=true;throw error;}
 finally{clearTimeout(timer);if(!expired||settled)row.active=false;}
}
export function closeSourceCiArchiveData(handle){
 const row=loans.get(handle);assert(row&&!row.closed&&!row.active,'live inactive data handle required');return close(row);
}
