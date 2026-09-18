import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {withBrowser} from '../../sdk/browser/browser-test-server.mjs';
import {queryBrowserFixture} from './browser-fixture.mjs';
import {run,sha,draft} from './compile.mjs';
export async function verifyBrowser(build,command,vectors){
 const commandBytes=command.wasmBytes;
 const source=readFileSync(new URL('../../sdk/browser/queries.mjs',import.meta.url),'utf8'),commands=readFileSync(new URL('../../sdk/browser/commands.mjs',import.meta.url),'utf8');
 const invoke=async body=>withBrowser(async({browser,baseURL})=>{const page=await browser.newPage();await page.route('**/queries.mjs',route=>route.fulfill({status:200,contentType:'text/javascript',body}));await page.route('**/commands.mjs',route=>route.fulfill({status:200,contentType:'text/javascript',body:commands}));await page.goto(baseURL);return page.evaluate(queryBrowserFixture,{queryBytes:Array.from(build.wasmBytes),journalBytes:Array.from(build.journalWasm),commandBytes:Array.from(commandBytes),probeBytes:Array.from(build.probeWasm),vectors:vectors.map(v=>({id:v.id,request:v.request.toString('hex'),response:v.response.toString('hex')}))});});
 const result=await invoke(source);assert.deepEqual(result.cases,[
  'all62 vectors twice, complete64-member and256-message traversal',
  'exhaustive256octets and closed empty/long probe twice in Chromium',
  'closed intent, no snapshot escape, synchronous capture and detached pages',
  'possessed persisted identity capture, wrong private key rejected',
  'mutable persisted key/principal across actual bootstrap digest stays captured',
  'real signatures/replay roles, pagination, private session, revoke and stale cursor',
  'actual cross-tab commit during digest rejects before any rows escape',
  'bounded admission, queued capture, success/failure release',
  'close releases queued read and refuses late active disclosure',
  'tampered stored envelope cannot supply a synthetic authenticated context',
  'failed replay discloses no partial rows; explicit new query replays completely',
 ]);assert.ok(result.maximum<=512*65536);assert.ok(result.calls.length>200);
 const groups={Query:[],Command:[]};
 for(const call of result.calls){assert.deepEqual(Object.keys(call).sort(),['kind','request','response']);assert.match(call.request,/^(?:[0-9a-f]{2})*$/);assert.match(call.response,/^(?:[0-9a-f]{2})+$/);assert.ok(['Query','Journal','Command','Decode'].includes(call.kind));groups[call.kind==='Command'?'Command':'Query'].push(call);}
 for(const[kind,calls]of Object.entries(groups)){const work=kind==='Command'?command.work:build.work;const path=join(work,'query-browser-'+kind.toLowerCase()+'.tsv');const rows=calls.map((c,i)=>c.kind+i+'\t'+c.request+'\t'+c.response+'\n');writeFileSync(path,rows.join(''));
  const binary=join(work,'native-target/release/browser-workspace-'+kind.toLowerCase()+'-runner'),output=run(binary,[path],join(work,'runner'));assert.match(output,new RegExp('PASS '+rows.length+' complete generated command vectors twice'));
  if(kind==='Query'){const changed=rows.slice(),parts=changed[0].trimEnd().split('\t');parts[2]=(parseInt(parts[2].slice(0,2),16)^1).toString(16).padStart(2,'0')+parts[2].slice(2);changed[0]=parts.join('\t')+'\n';const mutant=join(work,'query-browser-transcript-mutant.tsv');writeFileSync(mutant,changed.join(''));assert.throws(()=>run(binary,[mutant],join(work,'runner')),/Query0 bytes|Query0 length/,'changed generated query transcript must fail');}
 }
 const mutants=[
  ['identity capture',source.replace('const captured=await validateIdentity(identity);','await validateIdentity(identity);const captured=identity;'),/identity must be captured|query-rejected/],
  ['context race',source.replace("if(!same(head,current.head)||!same(state,current.state))throw fail('query-context-changed');","/* planted omitted context guard */"),/expected query-context-changed/],
  ['missing second authenticated replay',source.replace('await this.#journal.refresh();this.#open();const current=this.#journal.snapshot();','this.#open();const current=this.#journal.snapshot();'),/expected query-context-changed/],
  ['admission cap',source.replace('const maximumOutstanding=2;','const maximumOutstanding=3;'),/expected adapter-busy/],
 ];
 for(const[label,changed,expected]of mutants){assert.notEqual(changed,source,label);await assert.rejects(invoke(changed),expected,label);}
 return {diagnostics:result.diagnostics,cases:result.cases,calls:result.calls.length,maximum:result.maximum,source:sha(source)};
}
