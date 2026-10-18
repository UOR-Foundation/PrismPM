// HO-18 executes the unchanged generated adapter against its manifest-bound
// source corpus, including the actual large multibyte result.
import assert from 'node:assert/strict';
import {readFileSync,lstatSync,realpathSync} from 'node:fs';
import {resolve,sep} from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
assert.equal(process.argv.length,3);
const root=realpathSync(resolve(process.argv[2]));
const sha=b=>createHash('sha256').update(b).digest('hex');
const manifest=JSON.parse(readFileSync(resolve(root,'manifest.json')));
function file(path){
  assert.equal(typeof path,'string');const absolute=resolve(root,path);
  assert(absolute.startsWith(root+sep));assert(lstatSync(absolute).isFile());
  assert.equal(realpathSync(absolute),absolute);
  const rows=manifest.files.filter(row=>row.path===path);assert.equal(rows.length,1);
  const bytes=readFileSync(absolute);assert.equal(bytes.length,rows[0].byte_length);assert.equal(sha(bytes),rows[0].sha256);return bytes;
}
const modelBytes=file('model.prism.json'),model=JSON.parse(modelBytes),app=model.application;
assert.equal(model.schema,'prismpm/model-document/7');assert.equal(app.profile,'prismpm/text-application/4');
assert.equal(app.request_maximum,2097152);assert.equal(app.response_maximum,4194304);assert.equal(app.guest_allocation_maximum,4194304);
assert.equal(app.acceptance_vectors.length,8);
const stem=app.cargo_name.replaceAll('-','_'),js='view/browser/'+stem+'.js',wasm='view/browser/'+stem+'_bg.wasm';
file(js);const binary=file(wasm),provenance=JSON.parse(file('view/browser/provenance.json'));
assert.equal(provenance.model_id,sha(modelBytes));assert.match(file('view/browser/index.html').toString('utf8'),new RegExp('src="app\\.js\\?model_id='+provenance.model_id+'"'));assert.match(file('view/browser/index.html').toString('utf8'),new RegExp('href="app\\.css\\?model_id='+provenance.model_id+'"'));assert.equal(provenance.adapter_wasm_sha256,sha(binary));
const adapter=await import(pathToFileURL(resolve(root,js)));await adapter.default({module_or_path:binary});
assert.equal(typeof adapter.model_id,'function');assert.equal(adapter.model_id(),provenance.model_id);
let answers=0,lengthRefusals=0,largeOutput=0;
for(const vector of app.acceptance_vectors){
 const request=Uint8Array.from(vector.request);
 if(request.length>app.request_maximum){assert.throws(()=>adapter.invoke_bytes(request),error=>error==='input byte limit');lengthRefusals++;continue;}
 const actual=adapter.invoke_bytes(request);assert.deepEqual(Buffer.from(actual),Buffer.from(vector.response));answers++;
 if(actual.length===4194304){assert.equal(new TextDecoder('utf-8',{fatal:true}).decode(actual),'🌱'.repeat(1048576));largeOutput++;}
}
assert.equal(answers,6);assert.equal(lengthRefusals,2);assert.equal(largeOutput,1);
assert.throws(()=>adapter.invoke_bytes(Uint8Array.of(255)),error=>error==='invalid input UTF-8');
const exact=new TextEncoder().encode('é'.repeat(1048576));assert.equal(exact.length,2097152);adapter.invoke_bytes(exact);
const over=new Uint8Array(exact.length+1);over.set(exact);over[exact.length]=120;assert.throws(()=>adapter.invoke_bytes(over),error=>error==='input byte limit');
console.log(JSON.stringify({schema:'prismpm/research-generated-adapter-check/1',model_id:provenance.model_id,vectors:8,answers,length_refusals:lengthRefusals,exact_multibyte_output_bytes:4194304,extra_utf8_refusal:true,extra_multibyte_boundary:true}));
