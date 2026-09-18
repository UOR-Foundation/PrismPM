import assert from 'node:assert/strict';
import {writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {withBrowser} from '../../sdk/browser/browser-test-server.mjs';
import {commandBrowserFixture} from './browser-fixture.mjs';
export async function verifyBrowser(build,run,vectors){
 const result=await withBrowser(async({browser,baseURL})=>{
  const page=await browser.newPage();await page.goto(baseURL);
  return page.evaluate(commandBrowserFixture,{commandBytes:Array.from(build.wasmBytes),journalBytes:Array.from(build.journalWasm),vectors:vectors.map(v=>({id:v.id,request:v.request.toString('hex'),response:v.response.toString('hex')}))});
 });
 assert.equal(result.cases.length,26);assert.equal(result.count,9);
 assert.ok(result.calls.length>1500);assert.ok(result.maximum<=64*65536);
 const operations={Command:new Set(),Journal:new Set()};
 for(const call of result.calls){assert.deepEqual(Object.keys(call).sort(),['kind','request','response']);assert.match(call.request,/^(?:[0-9a-f]{2})*$/);assert.match(call.response,/^(?:[0-9a-f]{2})+$/);if(call.request.length)operations[call.kind].add(parseInt(call.request.slice(0,2),16));else assert.equal(call.kind,'Command','only the modeled command empty-input negative');}
 assert.deepEqual([...operations.Command].sort((a,b)=>a-b),[0,1,2,255]);assert.deepEqual([...operations.Journal].sort(),[0,1,2,3,4,5,6]);
 const rows=result.calls.map((call,index)=>call.kind+index+'\t'+call.request+'\t'+call.response+'\n');
 const path=join(build.work,'browser-transcript.tsv');writeFileSync(path,rows.join(''),{flag:'wx'});
 const native=build.compileNative(true),output=run(native,[path],build.runner);
 assert.match(output,new RegExp('PASS '+rows.length+' complete generated command vectors twice'));
 const changed=rows.slice(),parts=changed[0].trimEnd().split('\t');parts[2]=(parseInt(parts[2].slice(0,2),16)^1).toString(16).padStart(2,'0')+parts[2].slice(2);changed[0]=parts.join('\t')+'\n';
 const mutant=join(build.work,'browser-transcript-mutant.tsv');writeFileSync(mutant,changed.join(''),{flag:'wx'});
 assert.throws(()=>run(native,[mutant],build.runner),/Command0 bytes|Command0 length/,'changed native transcript must fail');
 return {cases:result.cases,count:result.count,calls:rows.length,maximum:result.maximum};
}
