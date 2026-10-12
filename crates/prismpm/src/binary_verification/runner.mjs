// Actual generated binary artifact execution; byte comparisons are independent of the core.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {readFileSync,writeFileSync,existsSync,mkdirSync,openSync,closeSync,symlinkSync} from 'node:fs';
import {join,resolve} from 'node:path';
import {inspectBinaryModule} from './memory-inspector.mjs';
const [programPath,wasmPath,cliPath,workPath] = process.argv.slice(2);
assert.equal(process.argv.length,6);
assert.equal(process.platform,'linux','binary package acceptance requires the supported Linux SDK');
const program=JSON.parse(readFileSync(programPath)),work=resolve(workPath),cli=resolve(cliPath);
assert.equal(program.profile,'prismpm/binary-program/1');
assert.equal(program.cli.profile,'prismpm/raw-file-cli/1');
assert.ok(program.acceptance_vectors.length>0);
const vectors=program.acceptance_vectors;
const wasm=readFileSync(wasmPath);
const memory=inspectBinaryModule(wasm,program.memory_pages);
assert.equal(memory.maximumPages,program.memory_pages,'exact declared memory bound');
const module=await WebAssembly.compile(wasm);
assert.deepEqual(WebAssembly.Module.imports(module),[]);
const exports=WebAssembly.Module.exports(module);
for(const [name,kind] of [['memory','memory'],['holo_alloc','function'],['binary_run','function']]) {
 assert.equal(exports.filter(row=>row.name===name&&row.kind===kind).length,1,'required binary ABI');
}
assert.ok(exports.every(row=>['memory','holo_alloc','binary_run','__data_end','__heap_base'].includes(row.name)),'unexpected binary ABI export');
for(const vector of vectors){
 const {exports:e}=new WebAssembly.Instance(module,{});
 const at=e.holo_alloc(vector.request.length)>>>0;
 assert.ok(at+vector.request.length<=e.memory.buffer.byteLength);
 new Uint8Array(e.memory.buffer,at,vector.request.length).set(vector.request);
 const packed=BigInt.asUintN(64,e.binary_run(at,vector.request.length));
 const offset=Number(packed>>32n),size=Number(packed&0xffffffffn);
 assert.ok(size<=program.response_maximum&&offset+size<=e.memory.buffer.byteLength,'bounded Wasm result');
 assert.deepEqual(Buffer.from(e.memory.buffer,offset,size),Buffer.from(vector.response),'exact Wasm response bytes');
 assert.ok(e.memory.buffer.byteLength<=program.memory_pages*65536);
}
{
 const {exports:e}=new WebAssembly.Instance(module,{});
 assert.throws(()=>e.holo_alloc(program.request_maximum+1),WebAssembly.RuntimeError);
 assert.throws(()=>e.binary_run(-1,0),WebAssembly.RuntimeError);
 assert.throws(()=>e.binary_run(0,program.request_maximum+1),WebAssembly.RuntimeError);
 assert.throws(()=>e.memory.grow(program.memory_pages+1),RangeError);
}
function execute(args,input,stdio){
 const result=spawnSync(cli,args,{input,stdio,encoding:null,timeout:30000,maxBuffer:program.response_maximum+65536});
 assert.equal(result.error,undefined,'adapter invocation error');assert.equal(result.signal,null,'adapter abnormal termination');
 return result;
}
function succeeded(result,expected){
 assert.equal(result.status,0,'adapter exit');assert.deepEqual(result.stderr,Buffer.alloc(0));
 if(expected!==null)assert.deepEqual(result.stdout,Buffer.from(expected),'exact adapter response bytes');
}
function failed(args,input,kind,publication='NotPublished',stdio){
 const result=execute(args,input,stdio);assert.equal(result.status,1,'typed adapter failure exit');
 assert.deepEqual(result.stderr,Buffer.from(`${kind}:${publication}\n`));
 if(result.stdout!==null)assert.deepEqual(result.stdout,Buffer.alloc(0));
}
mkdirSync(work,{recursive:true});
for(let index=0;index<vectors.length;index++){
 const vector=vectors[index],input=join(work,`${index}.input`),output=join(work,`${index}.output`);
 const mixed=join(work,`${index}.mixed`);writeFileSync(input,Buffer.from(vector.request));
 succeeded(execute(['--input','-','--output','-'],Buffer.from(vector.request)),vector.response);
 succeeded(execute(['--input',input,'--output',output]),[]);
 assert.deepEqual(readFileSync(output),Buffer.from(vector.response));
 succeeded(execute(['--input',input,'--output','-']),vector.response);
 succeeded(execute(['--input','-','--output',mixed],Buffer.from(vector.request)),[]);
 assert.deepEqual(readFileSync(mixed),Buffer.from(vector.response));
 failed(['--input',input,'--output',output],undefined,'OutputExists');
 assert.deepEqual(readFileSync(output),Buffer.from(vector.response),'no clobber existing output');
}
const oversized=Buffer.alloc(program.request_maximum+1,0xa5),oversizedPath=join(work,'oversized.input');
writeFileSync(oversizedPath,oversized);
for(const [input,bytes] of [['-',oversized],[oversizedPath,undefined]]) {
 const output=join(work,input==='-'?'oversized-stdin.output':'oversized-file.output');
 failed(['--input',input,'--output',output],bytes,'InputLimit');assert.equal(existsSync(output),false);
}
for(const args of [[],['--input','-'],['--input','-','--output','-','extra'],['--output','-','--input','-'],['--input','','--output','-']])failed(args,Buffer.alloc(0),'InvalidArguments');
failed(['--input',join(work,'absent'),'--output','-'],undefined,'InputOpen');
failed(['--input',work,'--output','-'],undefined,'InputType');
failed(['--input','-','--output',join(work,'absent-directory','output')],Buffer.alloc(0),'OutputOpen');
// Linux SDK gives deterministic real read/write failures without a simulated OS adapter.
{
 failed(['--input','/proc/self/mem','--output','-'],undefined,'InputRead');
 const full=openSync('/dev/full','w');
 try{
  const nonempty=vectors.find(vector=>vector.response.length>0);
  if(nonempty)failed(['--input','-','--output','-'],Buffer.from(nonempty.request),'OutputWrite','Unknown',['pipe',full,'pipe']);
 }finally{closeSync(full);}
 const target=join(work,'symlink-target'),link=join(work,'symlink-output');writeFileSync(target,Buffer.from([91]));symlinkSync(target,link);
 failed(['--input','-','--output',link],Buffer.alloc(0),'OutputExists');assert.deepEqual(readFileSync(target),Buffer.from([91]));
}
const io_coverage={platform:'linux',output_write:vectors.some(vector=>vector.response.length>0)?'passed':'not-exercised-empty-responses'};
console.log(JSON.stringify({executions:['core-wasm','cli-stdio','cli-file','cli-mixed'].map(mode=>({mode,vector_count:vectors.length,status:'passed'})),io_coverage}));
