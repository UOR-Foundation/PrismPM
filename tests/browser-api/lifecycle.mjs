import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {withBrowser} from '../../sdk/browser/browser-test-server.mjs';
async function reentrantFixture({commandBytes,journalBytes,queryBytes}){
 const {openCommands}=await import('/commands.mjs'),{createIdentity}=await import('/identity.mjs'),{openStore}=await import('/store.mjs');
 const commandModule=await WebAssembly.compile(new Uint8Array(commandBytes)),journalModule=await WebAssembly.compile(new Uint8Array(journalBytes));
 const store=await openStore('reentrant-'+crypto.randomUUID());await store.saveIdentity(await createIdentity());
 const commands=await openCommands({commandModule,journalModule,store,headName:'workspace'}),workspace=crypto.getRandomValues(new Uint8Array(32));
 await commands.submit({action:0,workspace,body:new Uint8Array()});
 let adapter,invoke,input;
 if(queryBytes){const {openQueries}=await import('/queries.mjs');adapter=await openQueries({queryModule:await WebAssembly.compile(new Uint8Array(queryBytes)),journalModule,store,headName:'workspace'});invoke=v=>adapter.query(v);input=()=>({table:0,workspace,cursor:new Uint8Array()});}
 else{adapter=commands;invoke=v=>adapter.submit(v);input=()=>({action:4,workspace,body:new TextEncoder().encode('pending')});}
 const original=crypto.subtle.digest;let entered,release,claimed=false;const ready=new Promise(r=>{entered=r;}),pause=new Promise(r=>{release=r;});
 crypto.subtle.digest=async(...args)=>{const bytes=new Uint8Array(args[1]);const owned=!claimed&&(queryBytes?bytes[0]===80&&bytes[1]===87&&bytes[2]===74:new TextDecoder().decode(bytes.subarray(0,27))==='prismpm/browser-signature/1');if(owned)claimed=true;const result=await original.apply(crypto.subtle,args);if(owned){entered();await pause;}return result;};
 const caught=p=>p.then(()=>({code:'unexpected-success'}),error=>({code:error.code}));let active;
 try{
  active=caught(invoke(input()));await ready;
  let captures=0;const proxy=new Proxy(input(),{getPrototypeOf(target){captures++;adapter.close();return Reflect.getPrototypeOf(target);}});
  const late=caught(invoke(proxy));
  const result=await Promise.race([late,new Promise(r=>setTimeout(()=>r({code:'retained-after-close'}),100))]);
  if(result.code!=='adapter-closed')throw Error('reentrant close must reject promptly; got '+result.code);
  if(captures!==1)throw Error('capture proxy not exercised');release();const first=await active;if(first.code!=='adapter-closed')throw Error('active close rejection');
  return{captureCount:captures,late:result.code,active:first.code};
 }finally{release();crypto.subtle.digest=original;await active;adapter.close();commands.close();store.close();}
}
async function storageFixture({kind,wasm,journal}){
 const {BrowserEffectError}=await import('/identity.mjs');
 const {openCommands}=await import('/commands.mjs'),{openQueries}=await import('/queries.mjs');
 const module=await WebAssembly.compile(new Uint8Array(wasm)),journalModule=await WebAssembly.compile(new Uint8Array(journal));
 const examples=[
  ['throwing exception prototype',new Proxy({}, {getPrototypeOf(){throw Error('private-exception-prototype');}}),'storage-outcome-unknown'],
  ['unknown class code',new BrowserEffectError('private-arbitrary-code'),'storage-outcome-unknown'],
  ['throwing class code',Object.defineProperty(new BrowserEffectError('store-closed'),'code',{get(){throw Error('private-code-accessor');}}),'storage-outcome-unknown'],
  ['known typed failure with private details',Object.assign(new BrowserEffectError('store-closed'),{message:'private-backend-message',cause:Error('private-cause'),payload:'private-payload'}),'store-closed'],
 ];
 for(const[label,error,expected]of examples){const store={loadIdentity:async()=>{throw error;}};let caught;try{if(kind==='Command')await openCommands({commandModule:module,journalModule,store,headName:'workspace'});else await openQueries({queryModule:module,journalModule,store,headName:'workspace'});}catch(value){caught=value;}
  if(caught?.code!==expected||caught.message!==expected||caught.cause!==undefined||caught.payload!==undefined)throw Error('sanitize '+label+'; observed '+caught?.code+' / '+caught?.message);
 }
 return examples.length;
}
export async function verifyReentrantClose(kind,commandBuild,queryBuild=null){
 const commandBytes=commandBuild.wasmBytes,journalBytes=commandBuild.journalWasm;
 const queries=readFileSync(new URL('../../sdk/browser/queries.mjs',import.meta.url),'utf8'),commands=readFileSync(new URL('../../sdk/browser/commands.mjs',import.meta.url),'utf8');
 const invoke=(querySource,commandSource)=>withBrowser(async({browser,baseURL})=>{const page=await browser.newPage();await page.route('**/queries.mjs',r=>r.fulfill({status:200,contentType:'text/javascript',body:querySource}));await page.route('**/commands.mjs',r=>r.fulfill({status:200,contentType:'text/javascript',body:commandSource}));await page.goto(baseURL);return page.evaluate(reentrantFixture,{commandBytes:Array.from(commandBytes),journalBytes:Array.from(journalBytes),queryBytes:kind==='Query'?Array.from(queryBuild.wasmBytes):null});});
 const result=await invoke(queries,commands);assert.deepEqual(result,{captureCount:1,late:'adapter-closed',active:'adapter-closed'});
 const needle=kind==='Query'?'intent=capture(value);this.#open();':'captured=capture(value);this.#open();';
 const source=kind==='Query'?queries:commands,mutant=source.replace(needle,needle.replace('this.#open();',''));assert.notEqual(mutant,source);
 await assert.rejects(invoke(kind==='Query'?mutant:queries,kind==='Command'?mutant:commands),/reentrant close must reject promptly; got retained-after-close/);
}
export async function verifyStorageErrors(kind,commandBuild,queryBuild=null){
 const wasm=kind==='Command'?commandBuild.wasmBytes:queryBuild.wasmBytes,journal=commandBuild.journalWasm;
 const result=await withBrowser(async({browser,baseURL})=>{const page=await browser.newPage();for(const[name,url]of[['queries',new URL('../../sdk/browser/queries.mjs',import.meta.url)],['commands',new URL('../../sdk/browser/commands.mjs',import.meta.url)]])await page.route('**/'+name+'.mjs',r=>r.fulfill({status:200,contentType:'text/javascript',body:readFileSync(url,'utf8')}));await page.goto(baseURL);return page.evaluate(storageFixture,{kind,wasm:Array.from(wasm),journal:Array.from(journal)});});assert.equal(result,4);
}
