// Actual small bytes exercise the shared original parser; no SDK/provider claim.
import assert from 'node:assert/strict';
import test from 'node:test';
import {Readable} from 'node:stream';
import {sourceArchiveFixture} from './sdk-source-ci-archive-fixture.mjs';
import {fixture as legacyFixture} from './sdk-construction-archive-fixture.mjs';
import {verifyConstructionArchiveStream,verifySourceCiArchiveStream,constructionIntegrityArchive} from './sdk-construction-archive.mjs';
import {readConstructionMetadata,readSourceCiMetadata} from './sdk-construction-metadata.mjs';
import {sourceCiArchiveIntegrityPlan} from './sdk-source-ci-contract.mjs';

const stream=(data,step=257)=>Readable.from((function*(){for(let p=0;p<data.length;p+=step)yield data.subarray(p,p+step);})());
const range=f=>async(start,length,signal)=>{assert(!signal.aborted);return f.bytes.subarray(start,start+length);};
test('closed source-CI profile verifies all seven actual ZIP members and complete OCI DAG',async()=>{
 for(const step of[1,7,257,65536])for(const wide of[false,true]){
  const f=sourceArchiveFixture({wide}),input=stream(f.bytes,step),result=await verifySourceCiArchiveStream(input,f.handle);
  assert.equal(result.members.length,7);assert.equal(result.layout.blob_count,3);assert(result.transport_closed&&input.closed);
  assert.deepEqual(constructionIntegrityArchive(result),{byte_length:f.archive.length,digest:f.original.envelope.archive.digest});
  assert.equal(result.scope,'original-stored-zip-and-oci-blob-integrity-only');assert.equal(result.status,undefined);
  assert(result.unclaimed.includes('provider-authentication')&&result.unclaimed.includes('installed-sdk'));
 }
});
test('bounded source-CI metadata reads all six original small members without admitting an SDK',async()=>{
 for(const wide of[false,true]){
  const f=sourceArchiveFixture({wide}),result=await readSourceCiMetadata(f.bytes.length,range(f));
  assert.deepEqual(result.members,f.members);assert(result.selected_bytes<=96*1024**2+1024**2&&result.range_calls<=512);
  assert.equal(result.status,undefined);assert.match(result.scope,/not-authentication-or-complete-archive-integrity/);
 }
});
test('legacy twelve-member and source-CI seven-member profiles cannot substitute for each other',async()=>{
 const a=sourceArchiveFixture(),b=legacyFixture();
 await assert.rejects(readConstructionMetadata(a.bytes.length,range(a)));
 await assert.rejects(readSourceCiMetadata(b.bytes.length,range(b)));
 await assert.rejects(verifyConstructionArchiveStream(stream(a.bytes),sourceCiArchiveIntegrityPlan(a.handle)));
 await assert.rejects(verifySourceCiArchiveStream(stream(b.bytes),a.handle));
 await assert.rejects(verifySourceCiArchiveStream(stream(a.bytes),b.plan));
});
test('copied caller JSON metadata plans and success flags never mint original handles or integrity',async()=>{
 const f=sourceArchiveFixture();for(const fake of [{},{...f.handle},{verified:true},f.original.envelope,sourceCiArchiveIntegrityPlan(f.handle)]){
  assert.throws(()=>sourceCiArchiveIntegrityPlan(fake));const input=stream(f.bytes);
  await assert.rejects(verifySourceCiArchiveStream(input,fake));assert(input.closed);
  assert.throws(()=>constructionIntegrityArchive(fake));
 }
 const data=sourceCiArchiveIntegrityPlan(f.handle);data.members.get('source-ci.json').fill(0);data.archive.digest='sha256:'+'0'.repeat(64);
 const result=await verifySourceCiArchiveStream(stream(f.bytes),f.handle);assert.equal(result.members.length,7);
 assert.throws(()=>constructionIntegrityArchive({...result}),'integrity receipts cannot be copied');
});
test('every shared ZIP local central CRC offset type and descriptor check is live for source-CI',async()=>{
 for(const zip of [f=>f.bytes.writeUInt16LE(1,6),f=>f.bytes.writeUInt16LE(8,8),f=>f.bytes.writeUInt32LE(1,14),
  f=>f.bytes.writeUInt32LE(1,18),f=>f.bytes.writeUInt32LE(1,f.central+16),f=>f.bytes.writeUInt32LE(1,f.central+20),
  f=>f.bytes.writeUInt32LE(1,f.central+42),f=>f.bytes.writeUInt32LE((0o120777<<16)>>>0,f.central+38),
  f=>f.bytes.writeUInt16LE(1,f.central+34),f=>f.bytes[f.rows[0].data]^=1,
  f=>f.bytes.writeUInt32LE(0,f.rows[0].data+f.rows[0].length),f=>f.bytes.writeUInt16LE(1,f.bytes.length-22+4)]){
  const f=sourceArchiveFixture({zip});await assert.rejects(verifySourceCiArchiveStream(stream(f.bytes),f.handle));
 }
});
test('coherently resealed source metadata and ZIP cannot omit replace alias or extend the OCI DAG',async()=>{
 for(const entries of [rows=>rows.filter(r=>r.name!==rows[2].name),
  rows=>rows.map((r,i)=>i===2?{...r,data:Buffer.alloc(r.data.length,1)}:r),rows=>[...rows,rows[2]],
  rows=>rows.map((r,i)=>i===2?{...r,name:'../escape'}:r),rows=>rows.map((r,i)=>i===2?{...r,type:'2'}:r),
  rows=>rows.filter(r=>r.name!=='blobs/'),rows=>rows.map(r=>r.name==='oci-layout'?{...r,data:Buffer.from('{}')}:r)]){
  const f=sourceArchiveFixture({entries});await assert.rejects(verifySourceCiArchiveStream(stream(f.bytes),f.handle));
 }
});
test('truncated trailing and interrupted original transports cannot issue source-CI integrity receipts',async()=>{
 const f=sourceArchiveFixture();for(const data of [f.bytes.subarray(0,-1),Buffer.concat([f.bytes,Buffer.of(0)])]){
  const input=stream(data);await assert.rejects(verifySourceCiArchiveStream(input,f.handle));assert(input.closed);
 }
 const broken=new Readable({read(){this.destroy(Error('actual transport interrupted'));}});
 await assert.rejects(verifySourceCiArchiveStream(broken,f.handle),/actual transport interrupted/);assert(broken.closed);
});
test('source-CI stream completion observes real delayed close and retains uncertain closure failures',async()=>{
 const f=sourceArchiveFixture();let sent=false,closed=false;
 const delayed=new Readable({read(){if(!sent){sent=true;this.push(f.bytes);this.push(null);}},destroy(error,callback){setTimeout(()=>{closed=true;callback(error);},30);}});
 const result=await verifySourceCiArchiveStream(delayed,f.handle);assert(closed&&result.transport_closed&&delayed.closed);
 const uncertain=new Readable({read(){},destroy(){}});
 await assert.rejects(verifySourceCiArchiveStream(uncertain,f.handle,10),error=>error instanceof AggregateError&&
  error.errors.some(e=>/deadline exceeded/.test(e.message))&&error.errors.some(e=>/close not observed/.test(e.message)));
 assert.equal(uncertain.closed,false);
});
test('source-CI metadata acquisition refuses unknown names malformed ZIP64 and timed out ranges',async()=>{
 for(const zip of [f=>f.bytes[f.central+46]=255,f=>f.bytes.writeUInt16LE(8,f.central+10),
  f=>f.bytes.writeBigUInt64LE(8n,f.bytes.length-22-20-56+32),f=>f.bytes.writeUInt32LE(2,f.bytes.length-22-20+16)]){
  const f=sourceArchiveFixture({wide:true,zip});await assert.rejects(readSourceCiMetadata(f.bytes.length,range(f)));
 }
 const f=sourceArchiveFixture();let signal;
 await assert.rejects(readSourceCiMetadata(f.bytes.length,async(a,b,c)=>{signal=c;return new Promise(()=>{});},10),/deadline exceeded/);
 assert(signal.aborted);
});
