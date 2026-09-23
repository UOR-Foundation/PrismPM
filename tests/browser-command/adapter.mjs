import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {withBrowser} from '../../sdk/browser/browser-test-server.mjs';
import {adapterFixture} from './adapter-fixture.mjs';
import {sha} from './compile.mjs';
export async function verifyAdapter(build,run,literalVectors){
 const commandBytes=build.wasmBytes,journalBytes=build.journalWasm,vectors=literalVectors.map(v=>({id:v.id,request:v.request.toString('hex'),response:v.response.toString('hex')}));
  const source=readFileSync(new URL('../../sdk/browser/commands.mjs',import.meta.url),'utf8');
  const result=await withBrowser(async({browser,baseURL})=>{
    const page=await browser.newPage();await page.route('**/commands.mjs',route=>route.fulfill({status:200,contentType:'text/javascript',body:source}));await page.goto(baseURL);
    return page.evaluate(adapterFixture,{commandBytes:Array.from(commandBytes),journalBytes:Array.from(journalBytes),vectors});
  });
  assert.equal(result.cases.length,22);assert.equal(result.maximum,50*65536);assert.ok(result.calls.length>500);
  const rows=result.calls.map((call,index)=>{assert.deepEqual(Object.keys(call).sort(),['kind','request','response']);assert.ok(['Command','Journal'].includes(call.kind));assert.match(call.request,/^(?:[0-9a-f]{2})*$/);assert.match(call.response,/^(?:[0-9a-f]{2})+$/);return call.kind+index+'\t'+call.request+'\t'+call.response+'\n';});
  const path=join(build.work,'adapter-transcript.tsv');writeFileSync(path,rows.join(''));
  const binary=join(build.work,'native-target/release/browser-workspace-command-runner'),runner=join(build.work,'runner');
  const output=run(binary,[path],runner);assert.match(output,new RegExp('PASS '+rows.length+' complete generated command vectors twice'));
  const changed=rows.slice(),parts=changed[0].trimEnd().split('\t');parts[2]=(parseInt(parts[2].slice(0,2),16)^1).toString(16).padStart(2,'0')+parts[2].slice(2);changed[0]=parts.join('\t')+'\n';
  const mutant=join(build.work,'adapter-transcript-mutant.tsv');writeFileSync(mutant,changed.join(''));assert.throws(()=>run(binary,[mutant],runner),/Command0 bytes|Command0 length/);
  return {diagnostics:result.diagnostics,cases:result.cases,calls:rows.length,maximum:result.maximum,source:sha(source),transcript:sha(rows.join(''))};
}
export async function verifyAdapterMutants(build,literalVectors){
 const rejected=[],commandBytes=build.wasmBytes,journalBytes=build.journalWasm,vectors=literalVectors.map(v=>({id:v.id,request:v.request.toString('hex'),response:v.response.toString('hex')}));
  const source=readFileSync(new URL('../../sdk/browser/commands.mjs',import.meta.url),'utf8');
  const mutants=[
    ['mutable body alias',source.replace('body=bytesCopy(descriptors.body.value,4096)','body=descriptors.body.value'),/synchronous input copy/],
    ['uncaptured identity',source.replace('const captured=await validateIdentity(identity);','await validateIdentity(identity);const captured=identity;'),/command-rejected/],
    ['omitted hash head refresh',source.replace('await this.#refresh();\n      current=plan(this.#call(completion(1,','/* planted missing refresh */\n      current=plan(this.#call(completion(1,'),/stale rejection must occur at the actual digest completion/],
    ['omitted signature head refresh',source.replace('await this.#refresh();\n      current=plan(this.#call(completion(2,','/* planted missing refresh */\n      current=plan(this.#call(completion(2,'),/expected command-rejected, got storage-rejected/],
    ['expanded admission capacity',source.replace('const outstandingMaximum=2;','const outstandingMaximum=3;'),/saturated submit must reject adapter-busy before retention/],
    ['retained queued input on close',source.replace("for(const entry of pending){this.#outstanding--;entry.operation=null;entry.reject(fail('adapter-closed'));}","this.#pending=pending;"),/close must release queued input without waiting for active effect/],
  ];
  for(const[label,changed,expected]of mutants){
    assert.notEqual(changed,source,label+' source changed');
    await assert.rejects(withBrowser(async({browser,baseURL})=>{
      const page=await browser.newPage();await page.route('**/commands.mjs',route=>route.fulfill({status:200,contentType:'text/javascript',body:changed}));await page.goto(baseURL);
      return page.evaluate(adapterFixture,{commandBytes:Array.from(commandBytes),journalBytes:Array.from(journalBytes),vectors});
    }),expected,label+' must fail actual browser acceptance');
    rejected.push(label);
  }
 return rejected;
}
