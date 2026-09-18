import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {withBrowser} from '../../sdk/browser/browser-test-server.mjs';

export async function verifyEntropy(build){
 const source=readFileSync(new URL('../../sdk/browser/queries.mjs',import.meta.url),'utf8');
 const check=body=>withBrowser(async({browser,baseURL})=>{
  const page=await browser.newPage();await page.route('**/queries.mjs',r=>r.fulfill({contentType:'text/javascript',body}));await page.goto(baseURL);
  return page.evaluate(async bytes=>{
   const {openQueries}=await import('/queries.mjs'),{createIdentity}=await import('/identity.mjs'),{openStore}=await import('/store.mjs');
   const queryModule=await WebAssembly.compile(new Uint8Array(bytes)),store=await openStore('query-entropy-'+crypto.randomUUID());
   await store.saveIdentity(await createIdentity());const cases=[];
   try{
    for(const mode of['throwing','missing']){
     const original=crypto.getRandomValues;let calls=0,reads=0,error;
     const storage={loadIdentity:()=>store.loadIdentity(),readHead:(...args)=>{reads++;return store.readHead(...args);}};
     crypto.getRandomValues=function(value){calls++;if(calls<=2){const result=original.call(crypto,value);if(mode==='missing'&&calls===2)crypto.getRandomValues=undefined;return result;}throw new DOMException('private entropy failure','OperationError');};
     try{await openQueries({queryModule,store:storage,headName:'workspace'});}catch(value){error=value;}
     finally{crypto.getRandomValues=original;}
     if(error?.name!=='BrowserEffectError'||error.code!=='crypto-unavailable'||error.message!=='crypto-unavailable'||error.cause!==undefined||error.payload!==undefined)throw Error('session entropy '+mode+' must be closed BrowserEffectError, got '+error?.name+'/'+error?.code);
     if(calls!==(mode==='missing'?2:3)||reads!==0)throw Error('entropy failure must follow real key possession and precede journal reads');
     cases.push(mode+' session entropy is sanitized before journal access');
    }
   }finally{store.close();}
   return cases;
  },Array.from(build.wasmBytes));
 });
 const cases=await check(source);assert.equal(cases.length,2);
 const guarded="let session;try{session=crypto.getRandomValues(new Uint8Array(32));}catch{throw new BrowserEffectError('crypto-unavailable');}";
 const unguarded='const session=crypto.getRandomValues(new Uint8Array(32));';
 assert.equal(source.split(guarded).length,2,'exact private entropy boundary');
 await assert.rejects(check(source.replace(guarded,unguarded)),/session entropy throwing must be closed BrowserEffectError/);
 return cases;
}
