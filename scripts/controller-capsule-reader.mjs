// Read-only integrity and composition checks, never host execution authority.
// Source-built producer authentication remains the same external envelope as
// cold acquisition. A JSON receipt cannot mint the private live ToolSeed.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const keys=(v,names)=>{assert(v&&typeof v==='object'&&!Array.isArray(v));assert.deepEqual(Object.keys(v).sort(),names.slice().sort());};
const hex=v=>assert.match(v,/^[0-9a-f]{64}$/);
const scalarString=v=>{assert.equal(typeof v,'string');assert.equal(Buffer.from(v,'utf8').toString('utf8'),v,'Unicode scalar string required');return v;};
const pathName=p=>{scalarString(p);assert(!p.includes('\0')&&!p.startsWith('/')&&p.split('/').every(v=>v!==''&&v!=='.'&&v!=='..'));};
const octal=bytes=>{assert(/^ *[0-7]+[ \0]*$/.test(bytes.toString('latin1')));const n=parseInt(bytes.toString('latin1'),8);assert(Number.isSafeInteger(n)&&n>=0);return n;};
const field=bytes=>{const z=bytes.indexOf(0);assert(z<0||bytes.subarray(z).every(x=>x===0));return new TextDecoder('utf-8',{fatal:true}).decode(z<0?bytes:bytes.subarray(0,z));};

// The repository-pinned compiled source is ordinary USTAR. Nothing is extracted,
// and this closed source profile does not replace the SDK layer/archive readers.
export function capsuleSourceRows(archive=readFileSync(new URL('../vendor/lean4-prod/lean.tar',import.meta.url))){
 assert(Buffer.isBuffer(archive)&&archive.length<=32*1024*1024&&archive.length%512===0);
 const rows=new Map(),headers=new Set();let at=0,total=0,ended=false;
 while(at+512<=archive.length){
  const h=archive.subarray(at,at+512);at+=512;
  if(h.every(x=>x===0)){assert(archive.length-at>=512&&archive.subarray(at).every(x=>x===0));ended=true;break;}
  assert.equal(octal(h.subarray(148,156)),h.reduce((n,x,i)=>n+(i>=148&&i<156?32:x),0));
  assert.equal(h.subarray(257,265).toString('latin1'),'ustar\0'+'00');
  assert.equal(field(h.subarray(157,257)),'');assert(h.subarray(500).every(x=>x===0));
  const prefix=field(h.subarray(345,500)),type=h[156],directory=type===53;
  assert([0,48,53].includes(type));let path=(prefix?prefix+'/':'')+field(h.subarray(0,100));
  if(directory&&path.endsWith('/'))path=path.slice(0,-1);pathName(path);assert(Buffer.byteLength(path)<=4096);
  assert(path!=='.lake'&&!path.startsWith('.lake/')&&!headers.has(path));headers.add(path);
  const mode=octal(h.subarray(100,108)),size=octal(h.subarray(124,136));assert(mode<=0o777);
  total+=size;assert(total<=16*1024*1024&&(!directory||size===0));
  assert(at+size<=archive.length);const body=archive.subarray(at,at+size);at+=Math.ceil(size/512)*512;
  assert(at<=archive.length&&archive.subarray(at-(512-size%512)%512,at).every(x=>x===0));
  const parts=path.split('/');for(let n=1;n<parts.length;n++){
   const parent=parts.slice(0,n).join('/');assert(!rows.has(parent)||rows.get(parent).kind==='directory');
   if(!rows.has(parent))rows.set(parent,{path:parent,kind:'directory'});
  }
  assert(!rows.has(path)||(directory&&rows.get(path).kind==='directory'&&rows.get(path).mode===undefined));
  rows.set(path,{path,kind:directory?'directory':'file',mode,...(directory?{}:{byte_length:size,sha256:hash(body)})});assert(rows.size<=4096);
 }
 assert(ended&&rows.has('Prod/Export.lean')&&rows.has('lean-toolchain'));
 return {archive_sha256:hash(archive),files:[...rows.values()].sort((a,b)=>Buffer.from(a.path).compare(Buffer.from(b.path)))};
}

export function validateControllerCapsule(value,executable,authority,context){
 keys(value,['archive_sha256','compiler_revision','construction','executable','executable_sha256','lean_sha256','mode','platform','schema','toolchain']);
 keys(authority,['archive_sha256','toolchain']);keys(context,['compiler_revision','platform','lean_sha256','lake_sha256']);
 assert.equal(value.schema,'prismpm/exporter-acquisition/1');assert.equal(value.mode,'controller-capsule');
 for(const n of ['archive_sha256','executable_sha256','lean_sha256'])hex(value[n]);
 assert.match(value.compiler_revision,/^[0-9a-f]{40}$/);assert.match(value.toolchain,/^leanprover\/lean4:v[0-9]+\.[0-9]+\.[0-9]+$/);
 for(const [name,expected]of Object.entries({archive_sha256:authority.archive_sha256,toolchain:authority.toolchain,
  compiler_revision:context.compiler_revision,platform:context.platform,lean_sha256:context.lean_sha256,executable_sha256:executable.sha256}))assert.equal(value[name],expected);
 assert(['linux/amd64','linux/arm64'].includes(value.platform));hex(context.lake_sha256);
 const construction=value.construction;keys(construction,['tool','argv','executable_sha256','exit_code','stdout','stderr']);
 assert.equal(construction.tool,'lean4-prod-build');assert.deepEqual(construction.argv,['build','prod-export']);
 assert.equal(construction.executable_sha256,context.lake_sha256);assert.equal(construction.exit_code,0);
 for(const n of ['stdout','stderr']){scalarString(construction[n]);assert(Buffer.byteLength(construction[n])<=16*1024*1024&&!construction[n].includes('\r'));}
 assert.equal(capsuleSourceRows().archive_sha256,authority.archive_sha256);
 // The public identity describes the genuine source-built compiler, not the
 // physical Lake cache. All cache bytes/native identities remain in the live
 // private constructor/copy owner; JSON cannot supply that authority.
 keys(executable,['byte_length','mode','sha256']);keys(value.executable,['byte_length','mode','sha256']);
 assert(Number.isSafeInteger(executable.byte_length)&&executable.byte_length>0&&executable.byte_length<=256*1024*1024);
 assert(Number.isSafeInteger(executable.mode)&&executable.mode>=0&&executable.mode<=0o777&&(executable.mode&0o111));hex(executable.sha256);
 assert.deepEqual(value.executable,executable);assert.equal(value.executable.sha256,value.executable_sha256);
 return value;
}
